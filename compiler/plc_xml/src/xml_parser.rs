use ast::{
    ast::{AstId, AstNode, CompilationUnit, Implementation, LinkageType, PouType as AstPouType},
    provider::IdProvider,
};
use plc::{lexer, parser::expressions_parser::parse_expression};
use plc_diagnostics::{diagnostician::Diagnostician, diagnostics::Diagnostic};

use plc_source::{
    source_location::{SourceLocation, SourceLocationFactory},
    SourceCode, SourceContainer,
};
use quick_xml::events::Event;

use crate::{
    error::Error,
    model::{pou::PouType, project::Project},
    reader::PeekableReader,
};

mod action;
mod block;
mod control;
mod fbd;
mod pou;
#[cfg(test)]
mod tests;
mod variables;

pub(crate) trait Parseable {
    type Item;
    fn visit(reader: &mut PeekableReader) -> Result<Self::Item, Error>;
}

pub(crate) fn visit(content: &str) -> Result<Project, Error> {
    let mut reader = PeekableReader::new(content);
    loop {
        match reader.peek()? {
            Event::Start(tag) if tag.name().as_ref() == b"pou" => return Project::pou_entry(&mut reader),
            Event::Start(tag) if tag.name().as_ref() == b"project" => return Project::visit(&mut reader),
            Event::Eof => return Err(Error::UnexpectedEndOfFile(vec![b"pou"])),
            _ => reader.consume()?,
        }
    }
}

pub fn parse_file(
    source: SourceCode,
    linkage: LinkageType,
    id_provider: IdProvider,
    diagnostician: &mut Diagnostician,
) -> CompilationUnit {
    // let _ = validate_xml(&source.source);
    let (unit, errors) = parse(&source, linkage, id_provider);
    //Register the source file with the diagnostician
    diagnostician.register_file(source.get_location_str().to_string(), source.source.to_string());
    diagnostician.handle(&errors);
    unit
}

fn validate_xml(content: &str) -> Result<(), Diagnostic> {
    use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
    let mut xsdparser = SchemaParserContext::from_file("model/resources/tc6_xml_v201.xsd");
    let xsd = SchemaValidationContext::from_parser(&mut xsdparser);
    todo!()
}

fn parse(
    source: &SourceCode,
    linkage: LinkageType,
    id_provider: IdProvider,
) -> (CompilationUnit, Vec<Diagnostic>) {
    // transform the xml file to a data model.
    // XXX: consecutive call-statements are nested in a single ast-statement. this will be broken up with temporary variables in the future
    let project = match visit(&source.source) {
        Ok(project) => project,
        Err(xml_err) => {
            let unit = CompilationUnit::new(source.get_location_str());
            let diagnostics = vec![Diagnostic::from(xml_err)];
            return (unit, diagnostics);
        }
    };
    // let Ok(project) = visit(&source.source) else {
    //     todo!("cfc errors need to be transformed into diagnostics")
    // };

    // Create a new parse session
    let source_location_factory = SourceLocationFactory::for_source(source);
    let parser =
        ParseSession::new(&project, source.get_location_str(), id_provider, linkage, source_location_factory);

    // Parse the declaration data field
    let Some((unit, mut diagnostics)) = parser.try_parse_declaration() else {
        unimplemented!("XML schemas without text declarations are not yet supported")
    };

    // Transform the data-model into an AST
    let (implementations, parser_diagnostics) = parser.parse_model();
    diagnostics.extend(parser_diagnostics);

    (unit.with_implementations(implementations), diagnostics)
}

pub(crate) struct ParseSession<'parse> {
    project: &'parse Project,
    id_provider: IdProvider,
    linkage: LinkageType,
    file_name: &'static str,
    range_factory: SourceLocationFactory,
    diagnostics: Vec<Diagnostic>,
}

impl<'parse> ParseSession<'parse> {
    fn new(
        project: &'parse Project,
        file_name: &'static str,
        id_provider: IdProvider,
        linkage: LinkageType,
        range_factory: SourceLocationFactory,
    ) -> Self {
        ParseSession { project, id_provider, linkage, file_name, range_factory, diagnostics: Vec::new() }
    }

    /// parse the compilation unit from the addData field
    fn try_parse_declaration(&self) -> Option<(CompilationUnit, Vec<Diagnostic>)> {
        let Some(content) = self
            .project
            .pous
            .first()
            .and_then(|it| it.interface.as_ref().and_then(|it| it.get_data_content()))
        else {
            return None;
        };

        //TODO: if our ST parser returns a diagnostic here, we might not have a text declaration and need to rely on the XML to provide us with
        // the necessary data. for now, we will assume to always have a text declaration
        Some(plc::parser::parse(
            lexer::lex_with_ids(content, self.id_provider.clone(), self.range_factory.clone()),
            self.linkage,
            self.file_name,
        ))
    }

    fn parse_expression(&self, expr: &str, local_id: usize, execution_order: Option<usize>) -> AstNode {
        let mut exp = parse_expression(&mut lexer::lex_with_ids(
            html_escape::decode_html_entities_to_string(expr, &mut String::new()),
            self.id_provider.clone(),
            self.range_factory.clone(),
        ));
        let loc = exp.get_location();
        exp.set_location(self.range_factory.create_block_location(local_id, execution_order).span(&loc));
        exp
    }

    fn parse_model(mut self) -> (Vec<Implementation>, Vec<Diagnostic>) {
        let mut implementations = vec![];
        for pou in &self.project.pous {
            // transform body
            implementations.push(pou.build_implementation(&mut self));
            // transform actions
            pou.actions.iter().for_each(|action| implementations.push(action.build_implementation(&self)));
        }

        (implementations, self.diagnostics)
    }

    fn next_id(&self) -> AstId {
        self.id_provider.clone().next_id()
    }

    fn create_range(&self, range: core::ops::Range<usize>) -> SourceLocation {
        self.range_factory.create_range(range)
    }

    fn create_block_location(&self, local_id: usize, execution_order: Option<usize>) -> SourceLocation {
        self.range_factory.create_block_location(local_id, execution_order)
    }

    fn create_file_only_location(&self) -> SourceLocation {
        self.range_factory.create_file_only_location()
    }
}

impl From<PouType> for AstPouType {
    fn from(value: PouType) -> Self {
        match value {
            PouType::Program => AstPouType::Program,
            PouType::Function => AstPouType::Function,
            PouType::FunctionBlock => AstPouType::FunctionBlock,
        }
    }
}

#[cfg(test)]
mod test {
    use std::{env, path::PathBuf, str::FromStr};

    use super::{parse, Parseable};
    use crate::serializer::{with_header, XBody, XExpression, XFbd, XInVariable, XOutVariable, XPou};
    use ast::{
        ast::{CompilationUnit, LinkageType},
        provider::IdProvider,
    };
    use insta::assert_debug_snapshot;
    use plc_diagnostics::diagnostics::Diagnostic;
    use plc_source::SourceCode;

    fn parse_test(source: impl Into<String>) -> (CompilationUnit, Vec<Diagnostic>) {
        let mut path = PathBuf::new();
        path.push("test");
        let source = SourceCode::new(source, path);
        parse(&source, LinkageType::Internal, IdProvider::default())
    }

    #[test]
    #[ignore = "consumable reader refactor needed. currently hits an infinite loop"]
    fn unclosed_xml_element_does_not_cause_infinite_loop() {
        // GIVEN valid xml but with an unclosed element
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
        <pou xmlns="http://www.plcopen.org/xml/tc6_0201" name="pass_through" pouType="functionBlock">"#;

        // WHEN trying to parse
        let (_, diagnostics) = parse_test(content);
        // THEN the parser does not run in an infinite loop but reports unexpected EOF
        assert_debug_snapshot!(diagnostics);
    }

    #[test]
    fn missing_attribute_is_reported() {
        // GIVEN xml content with an element that is missing required attributes
        let content = with_header(
            &XPou::init(
                "myFunction",
                "function",
                "FUNCTION myfunction : DINT
                VAR_INPUT
                    a : DINT;
                END_VAR",
            )
            .with_body(
                XBody::new().with_fbd(
                    XFbd::new()
                        .with_in_variable(
                            XInVariable::new().with_expression(XExpression::new().with_data("a")),
                        )
                        .with_out_variable(XOutVariable::new()),
                ),
            )
            .serialize(),
        );

        // WHEN trying to parse
        let (_, diagnostics) = parse_test(content);
        // THEN missing attribute is reported
        assert_debug_snapshot!(diagnostics);
    }

    #[test]
    fn unexpected_element_is_reported() {
        // GIVEN xml content with an element that is missing required attributes
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
        <pou xmlns="http://www.plcopen.org/xml/tc6_0201" name="conditional_return" pouType="UNEXPECTED_ELEMENT">
        <interface>
            <localVars/>
            <addData>
                <data name="www.bachmann.at/plc/plcopenxml" handleUnknown="implementation">
                    <textDeclaration>
                        <content>
                            FUNCTION_BLOCK conditional_return
                            VAR_INPUT
                                val : DINT;
                            END_VAR
                        </content>
                    </textDeclaration>
                </data>
            </addData>
        </interface>
        </pou>
            "#;

        // WHEN trying to parse
        let (_, diagnostics) = parse_test(content);
        // THEN missing attribute is reported
        assert_debug_snapshot!(diagnostics);
    }

    #[test]
    fn unclosed_line_reports_read_event() {
        // GIVEN xml content that's missing closing angle brackets
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
        <pou xmlns="http://www.plcopen.org/xml/tc6_0201" name="pass_through" pouType="functionBlock""#;

        // WHEN trying to parse
        let (_, diagnostics) = parse_test(content);
        // THEN a read-event with unexpected EOF is reported
        assert_debug_snapshot!(diagnostics);
    }

    #[test]
    fn int_parse_error_is_reported() {
        let content = with_header(
            &XPou::init(
                "myFunction",
                "function",
                "
                FUNCTION myFunction : DINT
                VAR_INPUT
                a : DINT;
                END_VAR
            ",
            )
            .with_body(
                XBody::new().with_fbd(XFbd::new().with_in_variable(XInVariable::init("not an int", false))),
            )
            .serialize(),
        );

        // WHEN trying to parse
        let (_, diagnostics) = parse_test(content);
        // THEN an encoding-error is reported
        assert_debug_snapshot!(diagnostics);
    }

    #[test]
    #[ignore = "infinite loop"]
    fn different_invalid_utf8_reports_encoding_error() {
        // GIVEN xml content with utf8 decoding error
        let invalid_utf8 = String::from_utf16_lossy(&[0xD800_u16]); // unpaired lead surrogate
        let content = format!(
            r#"<?xm{invalid_utf8} version="1.0" encoding="UTF-8"?>
        <pou xmlns="http://www.plcopen.org/xml/tc6_0201" name="test" pouType="functionBlock">"#
        );

        // WHEN trying to parse
        let (_, diagnostics) = parse_test(content);
        // THEN an encoding-error is reported
        assert_debug_snapshot!(diagnostics);
    }

    #[test]
    fn valid_xml_validates() {
        use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
        let mut xsdparser = SchemaParserContext::from_file("src/model/resources/tc6_xml_v201.xsd");
        let res = match SchemaValidationContext::from_parser(&mut xsdparser) {
            Ok(mut xsd) => xsd.validate_file("../../tests/integration/data/cfc/assigning.cfc"),
            Err(errors) => {
                for error in errors {
                    println!("{}", error.message.as_ref().unwrap());
                }

                panic!("xsd")
            }
        };
        dbg!(&res);
        assert!(res.is_ok())
    }
}
