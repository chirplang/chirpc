#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub main_parser);
mod ast;
mod text;
mod wasm;

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs,
    io::{self},
    path::{Path, PathBuf},
    rc::Rc,
};

use lalrpop_util::{lexer::Token, ErrorRecovery};

#[derive(Debug)]
pub enum ChipError<'a> {
    Multiple(Vec<Box<dyn Error + 'a>>),
    InputTooBig,
    TagClosedIncorrectly,
    ParserError(
        Rc<text::CodeText>,
        Vec<ErrorRecovery<usize, Token<'a>, ChipError<'a>>>,
        Box<dyn Error + 'a>,
    ),
}

impl Display for ChipError<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            InputTooBig => write!(f, "[MainError InputTooBig]"),
            TagClosedIncorrectly => write!(f, "[MainError TagClosedIncorrectly]"),
            ChipError::ParserError(text, errorvec, err) => write!(
                f,
                "[MainError Parser on file {:?}:  Error {:?} with error vec {:?}",
                text.path(),
                err,
                errorvec
            ),
            ChipError::Multiple(errs) => {
                for err in errs {
                    write!(f, "{:?}\n", err)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for ChipError<'_> {}

/// Compiles every .chip file in the current directory
pub fn compile_root() -> Result<(), Box<dyn Error>> {
    let path = std::env::current_dir()?;
    compile_folder(&path)
}

/// Compiles every .chip file in the specified directory
pub fn compile_folder<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let chip_files = chip_files(path)?;
    let mut results = vec![];
    // TODO: Multithreading
    for chip_file in chip_files {
        results.push(compile_file(chip_file));
    }
    let err_results = vec![];
    for res in results {
        if res.is_err() {
            err_results.push(res.unwrap_err());
        }
    }
    if err_results.is_empty() {
        Ok(())
    } else {
        Err(Box::new(ChipError::Multiple(err_results)))
    }
}

#[allow(dead_code, unused_variables)]
fn compile_file(path: PathBuf) -> Result<(), Box<dyn Error>> {
    let rs_file = resolve_rust_file(path.as_path());
    println!("[{:?}] Starting compilation...", path);

    if let Some(path) = rs_file.parent() {
        fs::create_dir_all(path)?;
    }

    let in_file = Rc::new(text::CodeText::from_path(path.clone())?);
    if rs_file.exists() {
        std::fs::remove_file(rs_file.clone())?;
    }
    let out_file = fs::File::create(rs_file)?;

    let ast = {
        let mut e = vec![];
        let ast = main_parser::CompilationUnitParser::new().parse(&mut e, in_file.text());
        match ast {
            Err(err) => {
                return Err(Box::new(ChipError::ParserError(
                    in_file.clone(),
                    e,
                    Box::new(err),
                )));
            }
            Ok(ast) => ast,
        }
    };

    Ok(())
}

fn resolve_rust_file(path: &Path) -> PathBuf {
    let in_dir = Path::new(".");
    let out_dir = in_dir.clone();

    out_dir
        .join(path.strip_prefix(&in_dir).unwrap_or(path))
        .with_extension("chip")
}

/// Copied from lalrpop
fn chip_files<P: AsRef<Path>>(root_dir: P) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut result = vec![];
    for entry in fs::read_dir(root_dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let path = entry.path();

        if file_type.is_dir() {
            result.extend(chip_files(&path)?);
        }

        let is_symlink_file = || -> io::Result<bool> {
            if !file_type.is_symlink() {
                Ok(false)
            } else {
                // Ensure all symlinks are resolved
                Ok(fs::metadata(&path)?.is_file())
            }
        };

        if (file_type.is_file() || is_symlink_file()?)
            && path.extension().is_some()
            && path.extension().unwrap() == "chip"
        {
            result.push(path);
        }
    }
    Ok(result)
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use crate::wasm::{compile_statement_wasm, ChipType, LocalMap, Primitive};
    use lalrpop_util::lexer::Token;
    use linked_hash_map::LinkedHashMap;
    use std::rc::Rc;
    use std::vec;
    use walrus::{FunctionBuilder, ValType};

    use super::*;

    #[test]
    fn parse_test() {
        parse_statement_expect("22 * 44 + 66", "((22 * 44) + 66)");
    }

    #[test]
    fn parse_test_simple_addition() {
        parse_statement_expect("22 + 33 - -12", "((22 + 33) - -12)");
    }

    #[test]
    fn parse_addition_idents() {
        parse_statement_expect_same("a = (b + c)");
    }

    #[test]
    fn number_too_big() {
        let mut errors = vec![];
        let expr = main_parser::StatementParser::new().parse(&mut errors, "9223372036854775808");
        assert!(expr.is_err());
        assert_eq!(
            format!("{:?}", expr.unwrap_err()),
            format!(
                "{:?}",
                lalrpop_util::ParseError::User::<usize, Token, _> {
                    error: ChipError::InputTooBig
                }
            )
        );
    }

    #[test]
    fn error() {
        parse_statement_expect("3 * * 2 + 7", "(((3 * error) * 2) + 7)");
    }

    #[test]
    fn negative_int_literal() {
        parse_statement_expect_same("-200");
    }

    #[test]
    fn float() {
        parse_statement_expect_same("-13.37");
    }

    #[test]
    fn float_exponent() {
        parse_statement_expect("-20.6e20", "-2060000000000000000000");
    }

    #[test]
    fn float_exponent_without_dot() {
        parse_statement_expect("-2e5", "-200000");
    }

    #[test]
    fn hex_int() {
        parse_statement_expect("0x70Aa", "28842");
    }

    #[test]
    fn function_call() {
        parse_statement_expect_same("test_function(one, two)");
    }

    #[test]
    fn function_call_no_args() {
        parse_statement_expect_same("test_function()");
    }

    #[test]
    fn simple_if_test() {
        parse_statement_expect_same("if something() { \ndo_other();\n }");
    }

    #[test]
    fn simple_if_else_test() {
        parse_statement_expect_same("if something { \ndo_if();\n } else { \nother();\n }");
    }

    #[test]
    fn assign() {
        parse_statement_expect_same("this = that");
    }

    #[test]
    fn equals() {
        parse_statement_expect("this == that", "(this == that)");
    }

    #[test]
    fn let_assign() {
        parse_statement_expect_same("let this = 3");
    }

    #[test]
    fn compilation_unit() {
        parse_compilation_unit_expect(
            "
            export Test { <Tag/> }
            view() -> Test { test_func(); a = b + c }
            ",
            "export Test {\n <Tag/> }\nview() -> Test { \ntest_func();\na = (b + c);\n }",
        );
    }

    #[test]
    fn parse_tag_all_cases() {
        parse_statement_expect("let a = (<Tag/>)", "let a = <Tag/>");
        parse_statement_expect(
            "let a = (<Tag disable=true/>)",
            "let a = <Tag disable=true/>",
        );
        parse_statement_expect(
            "let a = (<Tag disable=true> <OtherTag this=that/> </Tag>)",
            "let a = <Tag disable=true> <OtherTag this=that/> </Tag>",
        );
    }

    /// `parse_statement_expect(l, l);`
    fn parse_compilation_unit_expect_same(l: &str) {
        parse_compilation_unit_expect(l, l);
    }

    fn parse_compilation_unit_expect(l: &str, r: &str) {
        println!("Testing that parsed {:?} == {:?}", l, r);
        let mut e = vec![];
        let expr = main_parser::CompilationUnitParser::new().parse(&mut e, l);
        println!("{:?} with error vec {:?}", expr, e);
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), r);
    }

    /// `parse_statement_expect(l, l);`
    fn parse_statement_expect_same(l: &str) {
        parse_statement_expect(l, l);
    }

    /// Convenience func, this parses l and makes sure it's string representation equals r
    fn parse_statement_expect(l: &str, r: &str) {
        println!("Testing that parsed {:?} == {:?}", l, r);
        let mut e = vec![];
        let expr = main_parser::StatementParser::new().parse(&mut e, l);
        println!("{:?} with error vec {:?}", expr, e);
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), r);
    }

    #[test]
    fn wasm_let_binding() {
        let mut e = vec![];

        if !e.is_empty() {
            panic!("{:?}", e);
        }

        let statement = main_parser::StatementParser::new().parse(
            &mut e,
            r#"
            {   let a = test_struct;
                a.field_1 = test_struct;
            }
            "#,
        );

        let mut module = walrus::Module::default();

        let module_locals = &mut module.locals;

        let mut func = FunctionBuilder::new(&mut module.types, &[], &[ValType::I64]);

        let mut func_locals = LocalMap {
            names: Default::default(),
        };

        let mut test_struct_def = LinkedHashMap::new();

        (0..5).for_each(|index| {
            test_struct_def.insert(
                format!("field_{}", index),
                ChipType::Primitive(Primitive::I64),
            );

            let test_field_local = module_locals.add(ValType::I64);

            func_locals.names.insert(
                format!("test_struct.field_{}", index),
                (Some(test_field_local), ChipType::Primitive(Primitive::I64)),
            );
        });

        func_locals.names.insert(
            "test_struct".into(),
            (None, ChipType::Struct(Rc::new(test_struct_def))),
        );

        compile_statement_wasm(
            &mut func.func_body(),
            &mut func_locals,
            module_locals,
            &statement.unwrap(),
        );

        dbg!(
            "{:?}",
            func.func_body()
                .instrs()
                .iter()
                .map(|t| &t.0)
                .collect::<Vec<_>>()
        );
    }
}
