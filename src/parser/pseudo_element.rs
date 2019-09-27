use crate::parser::util::{take_valid_ident_string, wsd};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::tag_no_case;
use nom::combinator::{map, opt};
use nom::sequence::preceded;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum PseudoElement {
    After,
    Before,
    FirstLetter,
    FirstLine,
    Selection,
    Other(String),
}

pub(crate) fn parse_pseudo_element(i: &str) -> IResult<&str, PseudoElement> {
    wsd(preceded(
        tag("::"),
        alt((
            map(tag_no_case("after"), |_| PseudoElement::After),
            map(tag_no_case("before"), |_| PseudoElement::Before),
            map(tag_no_case("first-letter"), |_| PseudoElement::FirstLetter),
            map(tag_no_case("first-line"), |_| PseudoElement::FirstLine),
            map(tag_no_case("selection"), |_| PseudoElement::Selection),
            map(take_valid_ident_string(), PseudoElement::Other),
        )),
    ))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_after() {
        let i = "::after";
        let parsed = parse_pseudo_element(i).expect("Should parse").1;
        assert_eq!(parsed, PseudoElement::After)
    }
}
