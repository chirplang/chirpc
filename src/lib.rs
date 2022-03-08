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
}

#[test]
fn parse() {
    let mut errors = vec![];
    let expr = main_parser::ExprParser::new()
        .parse(&mut errors, "3 * * 2 + 7")
        .unwrap();
    println!("{:?}", expr);
    println!("{:?}", errors);
}

// #[test]
// fn number_too_big() {
//     let mut errors = vec![];
//     let expr = main_parser::ExprParser::new().parse(errors, "2147483648");
//     assert!(expr.is_err());
//     assert_eq!(
//         expr.unwrap_err(),
//         lalrpop_util::ParseError::User {
//             error: MainError::InputTooBig
//         }
//     );
// }
