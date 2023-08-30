use itertools::Itertools;
use plc_ast::{
    ast::{AstFactory, AstStatement, CompilationUnit, ReferenceAccess, SourceRange},
    control_statements::{
        AstControlStatement, CaseStatement, ConditionalBlock, ForLoopStatement, LoopStatement,
    },
};

macro_rules! fold_all {
    ( $this:ident, $stmt:expr ) => {{
        $stmt.drain(..).map(|it| $this.fold(it)).collect_vec()
    }};
}
pub struct DefaultFolder{
}

impl AstFolder for DefaultFolder{}

pub trait AstFolder {
    fn fold_unit(&mut self, mut unit: CompilationUnit) -> CompilationUnit {

        // TODO: visit all expressions in variable-declarations
        // for b in unit.units.iter_mut().flat_map(|it| it.variable_blocks.iter_mut()) {
        // }

        for implementation in unit.implementations.iter_mut() {
            let mut new_statements = Vec::with_capacity(implementation.statements.len());
            implementation.statements.drain(..).for_each(|stmt| new_statements.push(self.fold(stmt)));
            implementation.statements.extend(new_statements);
        }

        unit
    }

    fn fold(&mut self, stmt: AstStatement) -> AstStatement {
        match stmt {
            AstStatement::ReferenceExpr { access, base, id, location } => {
                self.fold_reference_expression(access, base, id, location)
            }
            AstStatement::DirectAccess { access, index, location, id } => {
                self.fold_direct_access(access, *index, location, id)
            }
            AstStatement::HardwareAccess { direction, access, address, location, id } => {
                self.fold_hardware_access(direction, access, address, location, id)
            }
            AstStatement::BinaryExpression { operator, left, right, id } => {
                self.fold_binary_expression(operator, *left, *right, id)
            }
            AstStatement::UnaryExpression { operator, value, location, id } => {
                self.fold_unary_expression(operator, *value, location, id)
            }
            AstStatement::Assignment { left, right, id } => self.fold_assignment(*left, *right, id),
            AstStatement::OutputAssignment { left, right, id } => {
                self.fold_output_assignment(*left, *right, id)
            }
            AstStatement::CallStatement { operator, parameters, location, id } => {
                self.fold_call_statement(*operator, *parameters, location, id)
            }
            AstStatement::CaseCondition { condition, id } => self.fold_case_condition(*condition, id),
            AstStatement::ControlStatement { kind, location, id } => match kind {
                AstControlStatement::If(if_stmt) => self.fold_if_statement(if_stmt, location, id),
                AstControlStatement::ForLoop(for_loop) => self.fold_for_loop(for_loop, location, id),
                AstControlStatement::WhileLoop(loop_stmt) => self.fold_while_loop(loop_stmt, location, id),
                AstControlStatement::RepeatLoop(loop_stmt) => self.fold_repeat_loop(loop_stmt, location, id),
                AstControlStatement::Case(case_stmt) => self.fold_case_statement(case_stmt, location, id),
            },
            // leaf-statements that dont have child-AST-elements
            // dont change these, just return the original
            AstStatement::Identifier { .. }
            | AstStatement::VlaRangeStatement { .. }
            | AstStatement::EmptyStatement { .. }
            | AstStatement::DefaultValue { .. }
            | AstStatement::Literal { .. }
            | AstStatement::ExitStatement { .. }
            | AstStatement::ContinueStatement { .. }
            | AstStatement::ReturnStatement { .. } => stmt,
            AstStatement::VoidStmt {} => todo!(),
            _ => stmt,
        }
    }

    fn fold_reference_expression(
        &mut self,
        access: ReferenceAccess,
        base: Option<Box<AstStatement>>,
        id: usize,
        location: SourceRange,
    ) -> AstStatement {
        let access = match access {
            plc_ast::ast::ReferenceAccess::Member(s) => ReferenceAccess::Member(Box::new(self.fold(*s))),
            plc_ast::ast::ReferenceAccess::Index(s) => ReferenceAccess::Index(Box::new(self.fold(*s))),
            plc_ast::ast::ReferenceAccess::Cast(s) => ReferenceAccess::Cast(Box::new(self.fold(*s))),
            _ => access,
        };
        AstStatement::ReferenceExpr { access, base: base.map(|b| Box::new(self.fold(*b))), id, location }
    }

    fn fold_direct_access(
        &mut self,
        access: plc_ast::ast::DirectAccessType,
        index: AstStatement,
        location: SourceRange,
        id: usize,
    ) -> AstStatement {
        AstStatement::DirectAccess { access, index: Box::new(self.fold(index)), location, id }
    }

    fn fold_hardware_access(
        &mut self,
        direction: plc_ast::ast::HardwareAccessType,
        access: plc_ast::ast::DirectAccessType,
        mut address: Vec<AstStatement>,
        location: SourceRange,
        id: usize,
    ) -> AstStatement {
        AstStatement::HardwareAccess { direction, access, address: fold_all!(self, address), location, id }
    }

    fn fold_binary_expression(
        &mut self,
        operator: plc_ast::ast::Operator,
        left: AstStatement,
        right: AstStatement,
        id: usize,
    ) -> AstStatement {
        AstStatement::BinaryExpression {
            operator,
            left: Box::new(self.fold(left)),
            right: Box::new(self.fold(right)),
            id,
        }
    }

    fn fold_unary_expression(
        &mut self,
        operator: plc_ast::ast::Operator,
        value: AstStatement,
        location: SourceRange,
        id: usize,
    ) -> AstStatement {
        AstStatement::UnaryExpression { operator, value: Box::new(self.fold(value)), location, id }
    }

    fn fold_assignment(&mut self, left: AstStatement, right: AstStatement, id: usize) -> AstStatement {
        AstStatement::Assignment { left: Box::new(self.fold(left)), right: Box::new(self.fold(right)), id }
    }

    fn fold_output_assignment(
        &mut self,
        left: AstStatement,
        right: AstStatement,
        id: usize,
    ) -> AstStatement {
        AstStatement::OutputAssignment {
            left: Box::new(self.fold(left)),
            right: Box::new(self.fold(right)),
            id,
        }
    }

    fn fold_call_statement(
        &mut self,
        operator: AstStatement,
        parameters: Option<AstStatement>,
        location: SourceRange,
        id: usize,
    ) -> AstStatement {
        AstStatement::CallStatement {
            operator: Box::new(self.fold(operator)),
            parameters: Box::new(parameters.map(|it| self.fold(it))),
            location,
            id,
        }
    }

    fn fold_case_condition(&mut self, condition: AstStatement, id: usize) -> AstStatement {
        AstStatement::CaseCondition { condition: Box::new(self.fold(condition)), id }
    }

    fn fold_if_statement(
        &mut self,
        mut if_stmt: plc_ast::control_statements::IfStatement,
        location: SourceRange,
        id: usize,
    ) -> AstStatement {
        AstFactory::create_if_statement(
            if_stmt
                .blocks
                .into_iter()
                .map(|c| {
                    let ConditionalBlock { mut body, condition } = c;
                    ConditionalBlock {
                        condition: Box::new(self.fold(*condition)),
                        body: fold_all!(self, body),
                    }
                })
                .collect_vec(),
            fold_all!(self, if_stmt.else_block),
            location,
            id,
        )
    }

    fn fold_for_loop(&mut self, for_loop: ForLoopStatement, location: SourceRange, id: usize) -> AstStatement {
        let ForLoopStatement { counter, start, end, by_step, mut body } = for_loop;
        AstFactory::create_for_loop(
            self.fold(*counter),
            self.fold(*start),
            self.fold(*end),
            by_step.map(|bs| self.fold(*bs)),
            fold_all!(self, body),
            location,
            id,
        )
    }

    fn fold_while_loop(&mut self, loop_stmt: LoopStatement, location: SourceRange, id: usize) -> AstStatement {
        let LoopStatement { mut body, condition } = loop_stmt;
        AstFactory::create_while_statement(self.fold(*condition), fold_all!(self, body), location, id)
    }

    fn fold_repeat_loop(&mut self, loop_stmt: LoopStatement, location: SourceRange, id: usize) -> AstStatement {
        let LoopStatement { mut body, condition } = loop_stmt;
        AstFactory::create_repeat_statement(self.fold(*condition), fold_all!(self, body), location, id)
    }

    fn fold_case_statement(
        &mut self,
        case_stmt: CaseStatement,
        location: SourceRange,
        id: usize,
    ) -> AstStatement {
        let CaseStatement { mut case_blocks, mut else_block, selector } = case_stmt;
        AstFactory::create_case_statement(
            self.fold(*selector),
            case_blocks
                .drain(..)
                .map(|ConditionalBlock { condition, mut body }| ConditionalBlock {
                    condition: Box::new(self.fold(*condition)),
                    body: fold_all!(self, body),
                })
                .collect_vec(),
            fold_all!(self, else_block),
            location,
            id,
        )
    }
}
