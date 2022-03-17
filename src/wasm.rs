use std::collections::HashMap;
use walrus::{InstrSeqBuilder, LocalId};
use crate::ast::{Ident, Number, Opcode, Statement, StatementList, Type};
use walrus::ir::{BinaryOp, Instr, InstrSeq, Value};

pub struct LocalMap {
    pub names: HashMap<String, LocalId>
}

pub fn parse_statement<'a>(builder: &mut InstrSeqBuilder, locals: &mut LocalMap, statement: &Statement<'a>) -> Option<Type<'a>> {
    match statement {
        Statement::Number(num) => {
           return match num {
                Number::Int(int) => {
                    builder.i64_const(*int);

                    Some(Type {
                        ident: Ident("i64"),
                        generics: vec![]
                    })
                },
                Number::Float(float) => {
                    builder.f64_const(*float);

                    Some(Type {
                        ident: Ident("f64"),
                        generics: vec![]
                    })
                }
            };
        }
        Statement::Op(statement_1, comp, statement_2) => {
            let st_1 = parse_statement(builder, locals, statement_1);
            let st_2 = parse_statement(builder, locals, statement_2);

            //TODO: check that these are the same type as number can be a float or int

            // match st_1.1 {
            //     None => {}
            //     Some(_type1) => {
            //         match st_2.1 {
            //             None => {}
            //             Some(_) => {}
            //         }
            //     }
            // }
            //
            // let int_or_float = false;
            //
            // out.push(match comp {
            //     Opcode::Mul => Instr::Binop {
            //         op: match st_1 {
            //
            //         }
            //     },
            //     Opcode::Div => {}
            //     Opcode::Add => {}
            //     Opcode::Sub => {}
            //     Opcode::Gt => {}
            //     Opcode::Ge => {}
            //     Opcode::Lt => {}
            //     Opcode::Le => {}
            //     Opcode::Eq => {}
            //     Opcode::Ne => {}
            // });
        }
        Statement::FunctionCall(_) => {}
        Statement::If(_, _) => {}
        Statement::IfElse(_, _, _) => {}
        Statement::Assign(_, _) => {}
        Statement::Ident(_) => {}
        Statement::Let(_) => {}
        Statement::LetAssign(ident, statement) => {
            // builder.
            // locals.names.insert(ident.0.into(), builder.);

            //Push the right hand value onto the stack
            parse_statement(builder, locals, &statement);
        }
        Statement::Block(statements) => {
            statements.0.iter().for_each(|statement| {
                parse_statement(builder, locals, &statement);
            });
        }
        Statement::Error => {}
    }

    None
}