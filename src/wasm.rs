use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use linked_hash_map::LinkedHashMap;
use walrus::{InstrSeqBuilder, LocalId, ModuleLocals, ModuleTypes, ValType};
use crate::ast::{Ident, Number, Opcode, Statement, StatementList, Type};
use walrus::ir::{BinaryOp, Instr, InstrSeq, Value};
use crate::checking::{ChipType, LocalMap, Primitive};

pub fn compile_statement_wasm<'a>(builder: &mut InstrSeqBuilder, func_locals: &'a mut LocalMap<LocalId>, module_locals: &mut ModuleLocals, statement: &Statement) -> Option<ChipType> {
    match statement {
        Statement::Number(num, range) => {
           return match num {
                Number::Int(int) => {
                    builder.i64_const(*int);

                    Some(ChipType::Primitive(Primitive::I64))
                },
                Number::Float(float) => {
                    builder.f64_const(*float);

                    Some(ChipType::Primitive(Primitive::F64))
                }
            };
        }
        Statement::Op(statement_1, comp, statement_2, range) => {
            let st_1 = compile_statement_wasm(builder, func_locals, module_locals, statement_1);
            let st_2 = compile_statement_wasm(builder, func_locals, module_locals, statement_2);

            let int_or_float = st_1 == st_2 && st_1 == Some(ChipType::Primitive(Primitive::I64));

            match comp {
                Opcode::Mul => builder.binop(if int_or_float { BinaryOp::I64Mul } else { BinaryOp::F64Mul }),
                Opcode::Div => builder.binop(if int_or_float { BinaryOp::I64DivS } else { BinaryOp::F64Div }),
                Opcode::Add => builder.binop(if int_or_float { BinaryOp::I64Add } else { BinaryOp::F64Add }),
                Opcode::Sub => builder.binop(if int_or_float { BinaryOp::I64Sub } else { BinaryOp::F64Sub }),
                Opcode::Gt => builder.binop(if int_or_float { BinaryOp::I64GtS } else { BinaryOp::F64Gt }),
                Opcode::Ge => builder.binop(if int_or_float { BinaryOp::I64GeS } else { BinaryOp::F64Ge }),
                Opcode::Lt => builder.binop(if int_or_float { BinaryOp::I64LtS } else { BinaryOp::F64Lt }),
                Opcode::Le => builder.binop(if int_or_float { BinaryOp::I64LeS } else { BinaryOp::F64Le }),
                Opcode::Eq => builder.binop(if int_or_float { BinaryOp::I64Eq } else { BinaryOp::F64Eq }),
                Opcode::Ne => builder.binop(if int_or_float { BinaryOp::I64Ne } else { BinaryOp::F64Ne }),
            };

            //Specifically make sure that there is a value here
            return Some(st_1.unwrap());
        }
        Statement::FunctionCall(_, range) => {}
        Statement::If(condition, block, range) => {
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
        Statement::IfElse(_, _, _, range) => {}
        Statement::Assign(ident, assign_statement, range) => {
            //Push the right hand value onto the stack
            let statement_type = compile_statement_wasm(builder, func_locals, module_locals, &assign_statement)
                .unwrap();

            let left_name: String = ident.0.iter().rev()
                .map(|ident| ident.0).collect::<Vec<&str>>().join(".");

            let ident_type = func_locals.names.get(&left_name).unwrap();

            let primitives = ident_type.1.flatten(
                &left_name
            );

            //Reverse the order because of how stacks are
            primitives.iter().rev().for_each(|(k, v)| {
                let local = func_locals.names.get(k).unwrap().0.unwrap();

                builder.local_set(local);
            });
        }
        Statement::Ident(ident, range) => {
            let local = func_locals.names.get(ident.0)
                .unwrap();

            let primitives = local.1.flatten(ident.0);

            primitives.iter().for_each(|(k, _)| {
                dbg!(k);
                let local = func_locals.names.get(k).unwrap().0;

                builder.local_get(local.unwrap());
            });

            return Some(local.1.clone());
        }
        Statement::Let(_, range) => {}
        Statement::LetAssign(ident, statement, range) => {
            //Push the right hand value onto the stack
            let type_ = compile_statement_wasm(builder, func_locals, module_locals, &statement)
                .unwrap();

            let primitives = type_.flatten(ident.0);

            func_locals.names.insert(ident.0.into(), (None, type_));

            //Reverse the order because of how stacks are
            primitives.iter().rev().for_each(|(k, v)| {
                let local = module_locals.add(match v {
                    Primitive::F64 => ValType::F64,
                    Primitive::I64 => ValType::I64
                });

                func_locals.names.insert(k.into(), (Some(local), ChipType::Primitive(*v)));

                builder.local_set(local);
            });
        }
        Statement::Block(statements, range) => {
            return statements.0.iter().map(|statement| {
                compile_statement_wasm(builder, func_locals, module_locals, &statement)
            }).last().flatten()
        }
        Statement::Error => {}
    }

    None
}