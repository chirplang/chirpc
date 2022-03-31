use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use lalrpop_util::{lexer::Token, ErrorRecovery, ParseError};

use crate::text;

#[derive(Debug)]
pub enum ChipError<'a> {
    Multiple(Vec<Box<dyn Error + 'a>>),
    InputTooBig,
    TagClosedIncorrectly,
    PreFormatted(String),
    ParserError(
        Rc<text::CodeText>,
        Vec<ErrorRecovery<usize, Token<'a>, ChipError<'a>>>,
        Box<dyn Error + 'a>,
    ),
}

impl Display for ChipError<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            ChipError::InputTooBig => write!(f, "[MainError InputTooBig]"),
            ChipError::TagClosedIncorrectly => write!(f, "[MainError TagClosedIncorrectly]"),
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
            ChipError::PreFormatted(string) => write!(f, "{}", string),
        }
    }
}

impl Error for ChipError<'_> {}

/// FIXME: This formatting is bad, there's not enough information in here.
/// FIXME: So if you wanna, please fix this. Ideally this should create the same quality errors as rustc does.
pub fn format_parse_error<'a>(
    in_file: Rc<text::CodeText>,
    err: ParseError<usize, Token, ChipError>,
) -> ChipError<'a> {
    let out = match err {
        ParseError::InvalidToken { location } => (
            Some(in_file.line_col(location)),
            format!("Encountered invalid token"),
        ),
        ParseError::UnrecognizedEOF { location, expected } => (
            Some(in_file.line_col(location)),
            format!("Encountered unexpected EOF, expected one of {:?}", expected),
        ),
        ParseError::UnrecognizedToken { token, expected } => {
            let loc = in_file.line_col(token.0);
            (
                Some(loc),
                format!(
                    "Encountered unrecognised token {} at {} - {}, expected one of {:?}",
                    token.1, token.0, token.2, expected
                ),
            )
        }
        ParseError::ExtraToken { token } => {
            let loc = in_file.line_col(token.0);
            (
                Some(loc),
                format!(
                    "Encountered extra token {} at {} - {}",
                    token.1, token.0, token.2
                ),
            )
        }
        ParseError::User { error } => (None, format!("{:?}", error)),
    };

    let out = if let Some(loc) = out.0 {
        format!(
            "{:?}:{} {}\n{}\n",
            in_file.path(),
            loc,
            out.1,
            in_file.line_text(loc.0)
        )
    } else {
        format!("{:?} {}\n", in_file.path(), out.1)
    };
    ChipError::PreFormatted(out)
}
