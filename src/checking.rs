use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::rc::Rc;
use codespan_reporting::diagnostic::{Diagnostic, Label, LabelStyle, Severity};
use linked_hash_map::LinkedHashMap;
use walrus::{InstrSeqBuilder, LocalId, ModuleLocals, ModuleTypes, ValType};
use crate::ast::{Ident, Number, Opcode, Statement, StatementList, Type};
use walrus::ir::{BinaryOp, Instr, InstrSeq, Value};

pub struct LocalMap<LocalIdentifier: Copy + Clone> {
    pub names: HashMap<String, (Option<LocalIdentifier>, ChipType)>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Primitive {
    F64,
    I64
}

#[derive(Clone, PartialEq)]
pub enum ChipType {
    Struct(Rc<(String, LinkedHashMap<String, ChipType>)>),
    Primitive(Primitive)
}

impl Debug for ChipType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            ChipType::Struct(rc) => write!(f, "{}", &rc.0),
            ChipType::Primitive(prim) => write!(f, "{:?}", prim)
        }
    }
}

impl ChipType {

    pub fn flatten(&self, name: &str) -> LinkedHashMap<String, Primitive> {
        let mut map = LinkedHashMap::new();

        ChipType::flatten_inner(name, &self, &mut map);

        map
    }

    fn flatten_inner(name: &str, type_: &ChipType, map: &mut LinkedHashMap<String, Primitive>) {
        match type_ {
            ChipType::Struct(struct_) => {
                struct_.1.iter().for_each(|(field_name, field_type)| {
                    let mut recurse_name = name.to_string();
                    recurse_name.push_str(".");
                    recurse_name.push_str(field_name);

                    ChipType::flatten_inner(&recurse_name, field_type, map);
                })
            }
            ChipType::Primitive(primitive) => { map.insert(name.into(), *primitive); }
        }
    }

}

pub fn type_check_statement<'a, T: Copy + Clone + Debug>(
    func_locals: &'a mut LocalMap<T>,
    statement: &Statement
) -> Result<(Option<ChipType>, Range<usize>), Diagnostic<()>> {

    match statement {
        Statement::Number(num, range) => {
            return Ok((match num {
                Number::Int(int) => {
                    Some(ChipType::Primitive(Primitive::I64))
                },
                Number::Float(float) => {
                    Some(ChipType::Primitive(Primitive::F64))
                }
            }, range.0..range.1));
        }
        Statement::Op(statement_1, comp, statement_2, range) => {
            let (st_1, range_1) = type_check_statement(func_locals,  statement_1)?;
            let (st_2, range_2) = type_check_statement(func_locals, statement_2)?;



            if st_1 != st_2 {
                return Err(Diagnostic::new(Severity::Error)
                    .with_message("Incompatible types")
                    .with_labels(vec![
                        Label::primary((), range.0..range.1)
                            .with_message("The two sides of this operation have different types"),
                        Label::secondary((), range_1)
                            .with_message(format!("Type {:?}", st_1.as_ref().unwrap())),
                        Label::secondary((), range_2)
                            .with_message(format!("Type {:?}", st_2.as_ref().unwrap()))
                    ]));
            }

            return Ok((st_1.and_then(|val| {
                Some(val)
            }), range.0..range.1));
        },
        Statement::FunctionCall(_, range) => unimplemented!(),
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

            unimplemented!()
        }
        Statement::IfElse(_, _, _, range) => unimplemented!(),
        Statement::Assign(ident, assign_statement, range) => {
            //Push the right hand value onto the stack
            let (chip_type, assign_range) = type_check_statement(func_locals, &assign_statement)?;

            let chip_type = chip_type.ok_or(
                Diagnostic::new(Severity::Error)
                        .with_message("Right-hand side of assign must have a return value")
                        .with_labels(vec![
                            Label::primary((), assign_range.clone())
                                .with_message("This statement returns nothing")
                    ])
            )?;

            let left_name: String = ident.0.iter().rev()
                .map(|ident| ident.0).collect::<Vec<&str>>().join(".");

            let ident_type = func_locals.names.get(&left_name).unwrap();

            if chip_type != ident_type.1 {
                return Err(
                    Diagnostic::new(Severity::Error)
                        .with_message("Incompatible types")
                        .with_labels(vec![
                            Label::primary((), range.0..range.1)
                                .with_message("Trying to assign wrong value to a let-binding"),
                            Label::secondary((), assign_range)
                                .with_message(format!("Expected type {:?}, found {:?}", ident_type.1, chip_type))
                        ])
                )
            }

            Ok((None, range.0..range.1))
        }
        Statement::Ident(ident, range) => {
            match func_locals.names.get(ident.0) {
                None => {
                    return Err(
                        Diagnostic::new(Severity::Error)
                            .with_message("Undefined identifier")
                            .with_labels(vec![
                                Label::primary((), range.0..range.1)
                                    .with_message("Undefined")
                            ])
                            .with_notes(vec![
                                format!("Hint: define `{}` with a let-binding\n\ne.g. `let {} = 123;`", ident.0, ident.0)
                            ])
                    );
                }
                Some(local) => return Ok((Some(local.1.clone()), range.0..range.1))
            }
        }
        Statement::Let(_, range) => unimplemented!(),
        Statement::LetAssign(ident, statement, range) => {
            let (chip_type, statement_range) = type_check_statement(func_locals, &statement)?;

            //Push the right hand value onto the stack
            let chip_type = match chip_type {
                None => return Err(
                    Diagnostic::new(Severity::Error)
                        .with_message("Right-hand side of let-assign must have a return value")
                        .with_labels(vec![
                            Label::primary((), range.0..range.1)
                                .with_message("Incompatible types")
                        ])
                ),
                Some(type_) => type_
            };

            let primitives = chip_type.flatten(ident.0);

            func_locals.names.insert(ident.0.into(), (None, chip_type));

            primitives.iter().for_each(|(k, v)| {
                func_locals.names.insert(k.into(), (None, ChipType::Primitive(*v)));
            });

            Ok((None, range.0..range.1))
        }
        Statement::Block(statements, range) => {
            return Ok((statements.0.iter().map(|statement| {
                type_check_statement(func_locals, &statement)
            }).last().transpose()?.and_then(|return_type| return_type.0), range.0..range.1))
        }
        Statement::Error => unimplemented!()
    }
}