#[macro_use]
extern crate lalrpop_util;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MainError {
    InputTooBig,
}

lalrpop_mod!(pub main_parser);
pub mod ast;

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;

    #[test]
    fn parse_test() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new()
            .parse(&mut errors, "22 * 44 + 66")
            .unwrap();
        assert_eq!(&format!("{:?}", expr), "((22 * 44) + 66)");
    }

    #[test]
    fn parse_test_simple_addition() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new()
            .parse(&mut errors, "22 + 33 - -12")
            .unwrap();
        assert_eq!(&format!("{:?}", expr), "((22 + 33) - -12)");
    }

    #[test]
    fn number_too_big() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "9223372036854775808");
        assert!(expr.is_err());
        assert_eq!(
            expr.unwrap_err(),
            lalrpop_util::ParseError::User {
                error: MainError::InputTooBig
            }
        );
    }

    #[test]
    fn error() {
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

    #[test]
    fn float() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "-13.37");
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), "-13.37")
    }

    #[test]
    fn float_exponent() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "-20.6e20");
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), "-2060000000000000000000");
    }

    #[test]
    fn float_exponent_without_dot() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "-2e5");
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), "-200000");
    }

    #[test]
    fn hex_int() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "0x70Aa");
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), "28842");
    }

    #[test]
    fn function_call() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "test_function(one, two)");
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), "test_function(one, two)");
    }

    #[test]
    fn function_call_no_args() {
        let mut errors = vec![];
        let expr = main_parser::ExprParser::new().parse(&mut errors, "test_function()");
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), "test_function()");
    }
}
