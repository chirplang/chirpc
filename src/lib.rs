#[macro_use]
extern crate lalrpop_util;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MainError {
    InputTooBig,
}

lalrpop_mod!(pub main_parser);
pub mod ast;

#[test]
fn parse_test() {
    let mut errors = vec![];
    let expr = main_parser::ExprParser::new()
        .parse(&mut errors, "22 * 44 + 66")
        .unwrap();
    assert_eq!(&format!("{:?}", expr), "((22 * 44) + 66)");
}

#[test]
fn number_too_big() {
    let mut errors = vec![];
    let expr = main_parser::ExprParser::new().parse(&mut errors, "2147483648");
    assert!(expr.is_err());
    assert_eq!(
        expr.unwrap_err(),
        lalrpop_util::ParseError::User {
            error: MainError::InputTooBig
        }
    );
}

#[test]
fn parse() {
    let mut errors = vec![];
    let expr = main_parser::ExprParser::new()
        .parse(&mut errors, "3 * * 2 + 7")
        .unwrap();
    assert_eq!(&format!("{:?}", expr), "(((3 * error) * 2) + 7)");
}

#[test]
fn negative_int_literal() {
    let mut errors = vec![];
    let expr = main_parser::ExprParser::new().parse(&mut errors, "-200");
    assert!(expr.is_ok());
    assert_eq!(&format!("{:?}", expr.unwrap()), "-200");
}
