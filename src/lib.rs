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
        parse_statement_expect("22 * 44 + 66", "((22 * 44) + 66)");
    }

    #[test]
    fn parse_test_simple_addition() {
        parse_statement_expect("22 + 33 - -12", "((22 + 33) - -12)");
    }

    #[test]
    fn number_too_big() {
        let mut errors = vec![];
        let expr = main_parser::StatementParser::new().parse(&mut errors, "9223372036854775808");
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
        parse_statement_expect_same("if something() { \ndo_other()\n }");
    }

    #[test]
    fn simple_if_else_test() {
        parse_statement_expect_same("if something { \ndo_if()\n } else { \nother()\n }");
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

    /// `parse_statement_expect(l, l);`
    fn parse_statement_expect_same(l: &str) {
        parse_statement_expect(l, l);
    }

    /// Convenience func, this parses l and makes sure it's string representation equals r
    fn parse_statement_expect(l: &str, r: &str) {
        let mut e = vec![];
        let expr = main_parser::StatementParser::new().parse(&mut e, l);
        println!("{:?} with error vec {:?}", expr, e);
        assert!(expr.is_ok());
        assert_eq!(&format!("{:?}", expr.unwrap()), r);
    }
}
