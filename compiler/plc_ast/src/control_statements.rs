use std::fmt::Debug;

use serde::Serialize;

use crate::ast::AstNode;

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize)]
pub struct IfStatement {
    pub blocks: Vec<ConditionalBlock>,
    pub else_block: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize)]
pub struct ForLoopStatement {
    pub counter: Box<AstNode>,
    pub start: Box<AstNode>,
    pub end: Box<AstNode>,
    pub by_step: Option<Box<AstNode>>,
    pub body: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
/// used for While and Repeat loops
#[derive(Serialize)]
pub struct LoopStatement {
    pub condition: Box<AstNode>,
    pub body: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize)]
pub struct CaseStatement {
    pub selector: Box<AstNode>,
    pub case_blocks: Vec<ConditionalBlock>,
    pub else_block: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize)]
pub enum AstControlStatement {
    If(IfStatement),
    ForLoop(ForLoopStatement),
    WhileLoop(LoopStatement),
    RepeatLoop(LoopStatement),
    Case(CaseStatement),
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize)]
pub struct ConditionalBlock {
    pub condition: Box<AstNode>,
    pub body: Vec<AstNode>,
}

#[derive(Debug, Clone, PartialEq)]
#[derive(Serialize)]
pub struct ReturnStatement {
    /// Indicates that the given condition must evaluate to true in order for the return to take place.
    /// Only used in CFC where the condition may be [`Some`] and [`None`] otherwise.
    pub condition: Option<Box<AstNode>>,
}
