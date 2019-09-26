use crate::parser::util::{take_until_encountered, take_valid_ident_string, ws};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum AttributeOperator {
    Eq,
    TildeEq,
    PipeEq,
    UpEq,
    DollarEq,
    StarEq,
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub(crate) name: String,
    pub(crate) target: Option<(AttributeOperator, String)>,
    pub(crate) case_sensitivity: CaseSensitivity,
}

#[derive(Debug, PartialEq)]
pub enum CaseSensitivity {
    Insensitive, // i
    Sensitive,   // s
    Default,     // <nothing>
}

pub(crate) fn parse_attribute_operator<'a>(
) -> impl Fn(&'a str) -> IResult<&'a str, AttributeOperator> {
    alt((
        map(char('='), |_| AttributeOperator::Eq),
        map(tag("~="), |_| AttributeOperator::TildeEq),
        map(tag("|="), |_| AttributeOperator::PipeEq),
        map(tag("^="), |_| AttributeOperator::UpEq),
        map(tag("$="), |_| AttributeOperator::DollarEq),
        map(tag("*="), |_| AttributeOperator::StarEq),
    ))
}

pub(crate) fn parse_attribute<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Attribute> {
    let attribute_parser = map(
        tuple((
            ws(take_until_encountered("", " ]=~|^$*")),
            opt(pair(
                ws(parse_attribute_operator()),
                ws(take_valid_ident_string()),
            )),
            map(
                opt(ws(alt((char('s'), char('i'))))),
                |c: Option<char>| match c {
                    Some('s') => CaseSensitivity::Sensitive,
                    Some('i') => CaseSensitivity::Insensitive,
                    Some(_) => unreachable!(),
                    None => CaseSensitivity::Default,
                },
            ),
        )),
        |(name, target, case_sensitivity)| Attribute {
            name,
            target,
            case_sensitivity,
        },
    );

    delimited(char('['), attribute_parser, ws(char(']')))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_attribute() {
        let i = "[yeet]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "yeet".to_string(),
            target: None,
            case_sensitivity: CaseSensitivity::Default,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn assign_attribute() {
        let i = "[lorem=ipsum]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "lorem".to_string(),
            target: Some((AttributeOperator::Eq, "ipsum".to_string())),
            case_sensitivity: CaseSensitivity::Default,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn pipe_eq_attribute() {
        let i = "[lorem|=ipsum]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "lorem".to_string(),
            target: Some((AttributeOperator::PipeEq, "ipsum".to_string())),
            case_sensitivity: CaseSensitivity::Default,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn eq_attribute_insensitive() {
        let i = "[lorem=ipsum i]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "lorem".to_string(),
            target: Some((AttributeOperator::Eq, "ipsum".to_string())),
            case_sensitivity: CaseSensitivity::Insensitive,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn attribute_sensitive() {
        let i = "[lorem i]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "lorem".to_string(),
            target: None,
            case_sensitivity: CaseSensitivity::Insensitive,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn pipe_eq_attribute_with_spaces() {
        let i = "[lorem |= ipsum]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "lorem".to_string(),
            target: Some((AttributeOperator::PipeEq, "ipsum".to_string())),
            case_sensitivity: CaseSensitivity::Default,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn pipe_eq_attribute_with_maximum_whitespace() {
        let i = "[ lorem |= ipsum s]";
        let parsed = parse_attribute()(i).expect("Should parse").1;
        let expected = Attribute {
            name: "lorem".to_string(),
            target: Some((AttributeOperator::PipeEq, "ipsum".to_string())),
            case_sensitivity: CaseSensitivity::Sensitive,
        };
        assert_eq!(parsed, expected)
    }

}
