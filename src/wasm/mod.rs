// FIXME: As soon as this isn't in heavy development anymore, REMOVE THIS
#![allow(unused_imports, dead_code, unused_variables)]
use crate::ast::{Ident, Number, Opcode, Statement, StatementList, Type};
use linked_hash_map::LinkedHashMap;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use walrus::ir::{BinaryOp, Instr, InstrSeq, Value};
use walrus::{InstrSeqBuilder, LocalId, ModuleLocals, ModuleTypes, ValType};

use self::error::WasmCodegenError;

mod error;

pub struct LocalMap {
    pub names: HashMap<String, (Option<LocalId>, ChipType)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Primitive {
    F64,
    I64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChipType {
    Struct(Rc<LinkedHashMap<String, ChipType>>),
    Primitive(Primitive),
}

impl ChipType {
    pub fn flatten(&self, name: &str) -> LinkedHashMap<String, Primitive> {
        let mut map = LinkedHashMap::new();

        ChipType::flatten_inner(name, &self, &mut map);

        map
    }

    fn flatten_inner(name: &str, type_: &ChipType, map: &mut LinkedHashMap<String, Primitive>) {
        match type_ {
            ChipType::Struct(struct_) => struct_.iter().for_each(|(field_name, field_type)| {
                let mut recurse_name = name.to_string();
                recurse_name.push_str(".");
                recurse_name.push_str(field_name);

                ChipType::flatten_inner(&recurse_name, field_type, map);
            }),
            ChipType::Primitive(primitive) => {
                map.insert(name.into(), *primitive);
            }
        }
    }
}

pub fn compile_statement_wasm<'a>(
    builder: &mut InstrSeqBuilder,
    func_locals: &'a mut LocalMap,
    module_locals: &mut ModuleLocals,
    statement: &'a Statement<'a>,
) -> anyhow::Result<Option<ChipType>> {
    match statement {
        Statement::Number(num) => {
            return match num {
                Number::Int(int) => {
                    builder.i64_const(*int);

                    Ok(Some(ChipType::Primitive(Primitive::I64)))
                }
                Number::Float(float) => {
                    builder.f64_const(*float);

                    Ok(Some(ChipType::Primitive(Primitive::F64)))
                }
            };
        }
        Statement::Op(statement_1, comp, statement_2) => {
            let type_1 = compile_statement_wasm(builder, func_locals, module_locals, statement_1)?
                .ok_or(WasmCodegenError::MustHaveReturnValue(format!(
                    "{statement_1:?}"
                )))?;
            let type_2 = compile_statement_wasm(builder, func_locals, module_locals, statement_2)?
                .ok_or(WasmCodegenError::MustHaveReturnValue(format!(
                    "{statement_2:?}"
                )))?;

            let primitive_1 = match type_1 {
                ChipType::Struct(_) => {
                    Err(WasmCodegenError::NoStructInOp(format!("{statement_1:?}")))?
                }
                ChipType::Primitive(prim) => prim,
            };

            let primitive_2 = match type_2 {
                ChipType::Struct(_) => {
                    Err(WasmCodegenError::NoStructInOp(format!("{statement_2:?}")))?
                }
                ChipType::Primitive(prim) => prim,
            };

            if primitive_1 != primitive_2 {
                Err(WasmCodegenError::CompNonEqualType(primitive_1, primitive_2))?;
            }

            let int_or_float = primitive_1 == Primitive::I64;

            match comp {
                Opcode::Mul => builder.binop(if int_or_float {
                    BinaryOp::I64Mul
                } else {
                    BinaryOp::F64Mul
                }),
                Opcode::Div => builder.binop(if int_or_float {
                    BinaryOp::I64DivS
                } else {
                    BinaryOp::F64Div
                }),
                Opcode::Add => builder.binop(if int_or_float {
                    BinaryOp::I64Add
                } else {
                    BinaryOp::F64Add
                }),
                Opcode::Sub => builder.binop(if int_or_float {
                    BinaryOp::I64Sub
                } else {
                    BinaryOp::F64Sub
                }),
                Opcode::Gt => builder.binop(if int_or_float {
                    BinaryOp::I64GtS
                } else {
                    BinaryOp::F64Gt
                }),
                Opcode::Ge => builder.binop(if int_or_float {
                    BinaryOp::I64GeS
                } else {
                    BinaryOp::F64Ge
                }),
                Opcode::Lt => builder.binop(if int_or_float {
                    BinaryOp::I64LtS
                } else {
                    BinaryOp::F64Lt
                }),
                Opcode::Le => builder.binop(if int_or_float {
                    BinaryOp::I64LeS
                } else {
                    BinaryOp::F64Le
                }),
                Opcode::Eq => builder.binop(if int_or_float {
                    BinaryOp::I64Eq
                } else {
                    BinaryOp::F64Eq
                }),
                Opcode::Ne => builder.binop(if int_or_float {
                    BinaryOp::I64Ne
                } else {
                    BinaryOp::F64Ne
                }),
            };

            //Specifically make sure that there is a value here
            return Ok(Some(type_2));
        }
        Statement::FunctionCall(_) => {}
        Statement::If(condition, block) => {
            // let mut consequent = builder.dangling_instr_seq(None);
            //
            // let no_else = builder.dangling_instr_seq(None);
            //
            // parse_statement(builder, func_locals, module_locals, &condition);
            //
            // let block_return_type = block.0.iter().map(|statement| parse_statement(
            //     &mut consequent,
            //     func_locals,
            //     module_locals,
            //     &statement
            // )).last().flatten();
            //
            // consequent.finish(vec![], )
            //
            // builder.instr(Instr::IfElse {
            //     consequent,
            //     alternative: ()
            // })

            // builder.if_else(match block_return_type {
            //     None => None,
            //     Some(type_) => match type_ {
            //         ChipType::Struct(_) => panic!("Cannot return a non-bool value in if conditional"),
            //         ChipType::Primitive(prim) => match prim {
            //             Primitive::F64 => ValType::F64,
            //             Primitive::I64 => ValType::I64
            //         }
            //     }
            // }, );
        }
        Statement::IfElse(_, _, _) => {}
        Statement::Assign(ident, assign_statement) => {
            //Push the right hand value onto the stack
            let statement_type =
                compile_statement_wasm(builder, func_locals, module_locals, &assign_statement)?
                    .expect("Right-hand side of let assignment must have return type");

            let left_name: String = ident
                .0
                .iter()
                .rev()
                .map(|ident| ident.0)
                .collect::<Vec<&str>>()
                .join(".");

            println!("{:?}\n{}", func_locals.names, left_name);
            let ident_type = func_locals.names.get(&left_name).unwrap();

            if statement_type != ident_type.1 {
                panic!(
                    "Assignment `{:?}` must have same type on left and right hand side",
                    statement
                );
            }

            let primitives = ident_type.1.flatten(&left_name);

            //Reverse the order because of how stacks are
            primitives.iter().rev().for_each(|(k, v)| {
                let local = func_locals.names.get(k).unwrap().0.unwrap();

                builder.local_set(local);
            });
        }
        Statement::Ident(ident) => {
            let local = func_locals
                .names
                .get(ident.0)
                .expect(&format!("Undeclared local variable {}", ident.0));

            let primitives = local.1.flatten(ident.0);

            primitives.iter().for_each(|(k, _)| {
                dbg!(k);
                let local = func_locals.names.get(k).unwrap().0;

                builder.local_get(local.unwrap());
            });

            return Ok(Some(local.1.clone()));
        }
        Statement::Let(_) => {}
        Statement::LetAssign(ident, statement) => {
            //Push the right hand value onto the stack
            let type_ = compile_statement_wasm(builder, func_locals, module_locals, &statement)?
                .expect("Right-hand side of let assignment must have return type");

            let primitives = type_.flatten(ident.0);

            func_locals.names.insert(ident.0.into(), (None, type_));

            //Reverse the order because of how stacks are
            primitives.iter().rev().for_each(|(k, v)| {
                let local = module_locals.add(match v {
                    Primitive::F64 => ValType::F64,
                    Primitive::I64 => ValType::I64,
                });

                func_locals
                    .names
                    .insert(k.into(), (Some(local), ChipType::Primitive(*v)));

                builder.local_set(local);
            });
        }
        Statement::Block(statements) => {
            let mut last = None;
            for statement in &statements.0 {
                last = Some(compile_statement_wasm(
                    builder,
                    func_locals,
                    module_locals,
                    &statement,
                )?);
            }
            return Ok(last.flatten());
        }
        Statement::Error => {}
        Statement::Tag(_) => todo!(),
    }

    Ok(None)
}
