use nom::combinator::{map, opt};
use nom::branch::alt;
use nom::bytes::complete::tag;
use crate::parser::util::take_valid_ident_string;
use nom::sequence::{tuple, pair, delimited};
use nom::character::complete::char;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum AttributeOperator {
    Eq,
    TildeEq,
    PipeEq,
    UpEq,
    DollarEq,
    StarEq
}

#[derive(Debug, PartialEq)]
pub struct Attribute{
    name: String,
    target: Option<(AttributeOperator, String)>,
    case_sensitivity: CaseSensitivity
}

#[derive(Debug, PartialEq)]
pub enum CaseSensitivity {
    Insensitive, // i
    Sensitive, // s
    Default // <nothing>
}

pub(crate) fn parse_attribute_operator<'a>() -> impl Fn(&'a str) -> IResult<&'a str, AttributeOperator> {
    alt((
        map(char('='), |_|AttributeOperator::Eq),
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
            take_valid_ident_string(),
            opt(pair(parse_attribute_operator(), take_valid_ident_string())),
            map(opt(alt((char('s'), char('i')))), |c: Option<char>| {
                match c {
                    Some('s') => CaseSensitivity::Sensitive,
                    Some('i') => CaseSensitivity::Insensitive,
                    Some(_) => unreachable!(),
                    None => CaseSensitivity::Default
                }
            })
        )),
        |(name, target, case_sensitivity)| {
            Attribute {
                name,
                target,
                case_sensitivity
            }
        }
    );

    delimited(
        char('['),
        attribute_parser,
        char(']'),
    )
}
