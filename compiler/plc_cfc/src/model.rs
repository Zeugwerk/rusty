use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    UnexpectedEndOfFile,

    UnexpectedElement(String),

    Encoding(Utf8Error),

    MissingAttribute(String),

    ReadEvent(quick_xml::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedEndOfFile => write!(f, "{self:#?}"),
            Error::MissingAttribute(key) => write!(f, "Expected attribute {key} but found none"),
            Error::ReadEvent(why) => write!(f, "Failed to read XML; {why}"),
            Error::UnexpectedElement(element) => write!(f, "{element}"),
            Error::Encoding(why) => write!(f, "{why:#?}"),
        }
    }
}

#[derive(Debug)]
pub struct FunctionBlockDiagram {
    pub blocks: Vec<Block>,
    pub variables: Vec<FunctionBlockVariable>,
}

#[derive(Debug)]
pub struct Block {
    pub local_id: String,
    pub type_name: String,
    pub instance_name: Option<String>,
    pub execution_order_id: Option<String>,
    pub variables: Vec<BlockVariable>,
}

#[derive(Debug)]
pub struct BlockVariable {
    pub kind: VariableKind,
    pub formal_parameter: String,
    pub negated: String,
    pub ref_local_id: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum VariableKind {
    Input,
    Output,
    InOut,
}

#[derive(Debug)]
pub struct FunctionBlockVariable {
    pub kind: VariableKind,
    pub local_id: String,
    pub negated: String,
    pub expression: String,
    pub execution_order_id: Option<String>,
    pub ref_local_id: Option<String>,
}

#[derive(Debug)]
pub struct Body {
    pub function_block_diagram: FunctionBlockDiagram,
}

#[derive(Debug)]
pub struct Pou {
    // TODO: interface
    pub name: String,
    pub pou_type: PouType,
    pub body: Body,
}

#[derive(Debug)]
pub enum PouType {
    Program,
    Function,
    FunctionBlock,
}

impl TryFrom<&str> for PouType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "program" => Ok(PouType::Program),
            "function" => Ok(PouType::Function),
            "functionBlock" => Ok(PouType::FunctionBlock),
            _ => Err(Error::UnexpectedElement(value.to_string())),
        }
    }
}

impl TryFrom<&[u8]> for VariableKind {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"inputVariables" | b"inVariable" => Ok(VariableKind::Input),
            b"outputVariables" | b"outVariable" => Ok(VariableKind::Output),
            b"inOutVariables" | b"inOutVariable" => Ok(VariableKind::InOut),
            _ => {
                let value = std::str::from_utf8(value).map_err(Error::Encoding)?;
                Err(Error::UnexpectedElement(value.to_string()))
            }
        }
    }
}
