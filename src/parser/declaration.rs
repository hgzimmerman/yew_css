
use crate::parser::selector::{Selector, parse_selector};
use nom::IResult;
use crate::parser::util::{take_until_encountered, wsd};
use nom::combinator::{map, opt};
use nom::sequence::{pair, separated_pair, delimited};
use nom::character::complete::char;
use nom::multi::separated_list;

#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub(crate) property: String,
    pub(crate) value: String,
}


pub (crate) fn parse_declaration(i: &str) -> IResult<&str, Declaration> {
    map(
        separated_pair(
            wsd(take_until_encountered("", " :")),
            char(':'),
            wsd(take_until_encountered("", " ;}\n\t"))
        ),
        |(property, value)| {
            Declaration {
                property,
                value
            }
        }
    )(i)
}

pub (crate) fn parse_declarations(i: &str) -> IResult<&str, Vec<Declaration>>  {
    let declarations_with_optional_terminating_semicolon = map(
        pair(
            wsd(separated_list(char(';'), wsd(parse_declaration))),
            opt(wsd(char(';')))
        ),
        |(declarations, _)| declarations
    );

    delimited(
        char('{'),
        declarations_with_optional_terminating_semicolon,
        char('}')
    )(i)
}