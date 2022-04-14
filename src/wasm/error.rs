use thiserror::Error;

use crate::ast::{Opcode, Statement};

use super::Primitive;

#[derive(Error, Debug)]
pub enum WasmCodegenError {
    #[error("Statement {0:?} must have a return value")]
    MustHaveReturnValue(String),
    #[error("Cannot use struct in operation: {0:?}")]
    NoStructInOp(String),
    #[error("Comparison must have the same type on both sides: {0:?} {1:?}")]
    CompNonEqualType(Primitive, Primitive),
}
