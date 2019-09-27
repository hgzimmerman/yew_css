use crate::parser::util::{take_until_encountered, wsd};
use crate::parser::{
    selector::parse_selector, selector::Selector, util::take_not_close_paren_string,
    util::take_valid_ident_string,
};
use nom::branch::alt;
use nom::bytes::complete::{tag_no_case, take_while};
use nom::character::complete::char;
use nom::combinator::{map, opt, not};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, pair};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum PseudoClass {
    Active,
    Checked,
    Disabled,
    Empty,
    Enabled,
    FirstChild,
    FirstOfType,
    Focus,
    Hover,
    Indeterminate,
    InRange,
    Invalid,
    Lang(String),
    LastChild,
    LastOfType,
    Left,
    Link,
    Not(Selector),
    NthChild(String), // Todo, these could be more thoroughly parsed.
    NthLastChild(String),
    NthLastOfType(String),
    OnlyOfType,
    OnlyChild,
    Optional,
    OutOfRange,
    ReadOnly,
    ReadWrite,
    Required,
    Right,
    Root,
    Target,
    Valid,
    Visited,
    Other(String),
}

fn parse_pseudo_classes_impl(i: &str) -> IResult<&str, Vec<PseudoClass>> {
    many1(preceded(
        char(':'),
        alt((
            // Only a certain number of branches ber alt.
            alt((
                map(tag_no_case("active"), |_| PseudoClass::Active),
                map(tag_no_case("checked"), |_| PseudoClass::Checked),
                map(tag_no_case("disabled"), |_| PseudoClass::Disabled),
                map(tag_no_case("empty"), |_| PseudoClass::Empty),
                map(tag_no_case("enabled"), |_| PseudoClass::Enabled),
                map(tag_no_case("first-child"), |_| PseudoClass::FirstChild),
                map(tag_no_case("first-of-type"), |_| PseudoClass::FirstOfType),
                map(tag_no_case("focus"), |_| PseudoClass::Focus),
                map(tag_no_case("hover"), |_| PseudoClass::Hover),
                map(tag_no_case("indeterminate"), |_| PseudoClass::Indeterminate),
                map(tag_no_case("in-range"), |_| PseudoClass::InRange),
                map(tag_no_case("invalid"), |_| PseudoClass::Invalid),
                map(
                    preceded(
                        tag_no_case("lang"),
                        delimited(char('('), take_valid_ident_string(), char(')')),
                    ),
                    PseudoClass::Lang,
                ),
                map(tag_no_case("last-child"), |_| PseudoClass::LastChild),
                map(tag_no_case("last-of-type"), |_| PseudoClass::LastOfType),
                map(tag_no_case("left"), |_| PseudoClass::Left),
                map(tag_no_case("link"), |_| PseudoClass::Link),
                map(
                    preceded(tag_no_case("not"), parse_selector),
                    |s: Selector| PseudoClass::Not(s),
                ),
                map(
                    preceded(
                        tag_no_case("nth-child"),
                        delimited(char('('), take_not_close_paren_string(), char(')')),
                    ),
                    PseudoClass::NthChild,
                ),
                map(
                    preceded(
                        tag_no_case("nth-last-child"),
                        delimited(char('('), take_not_close_paren_string(), char(')')),
                    ),
                    PseudoClass::NthLastChild,
                ),
                map(
                    preceded(
                        tag_no_case("nth-last-of-type"),
                        delimited(char('('), take_not_close_paren_string(), char(')')),
                    ),
                    PseudoClass::NthLastOfType,
                ),
            )),
            alt((
                map(tag_no_case("only-child"), |_| PseudoClass::OnlyChild),
                map(tag_no_case("only-of-type"), |_| PseudoClass::OnlyOfType),
                map(tag_no_case("optional"), |_| PseudoClass::Optional),
                map(tag_no_case("out-of-range"), |_| PseudoClass::OutOfRange),
                map(tag_no_case("read-only"), |_| PseudoClass::ReadOnly),
                map(tag_no_case("read-write"), |_| PseudoClass::ReadWrite),
                map(tag_no_case("required"), |_| PseudoClass::Required),
                map(tag_no_case("right"), |_| PseudoClass::Right),
                map(tag_no_case("root"), |_| PseudoClass::Root),
                map(tag_no_case("target"), |_| PseudoClass::Target),
                map(tag_no_case("valid"), |_| PseudoClass::Valid),
                map(tag_no_case("visited"), |_| PseudoClass::Visited),
                map(pair(not(char(':')), take_until_encountered(" )", ":")), |(_, s)| PseudoClass::Other(s)),
            )),
        )),
    ))(i)
}

pub(crate) fn parse_pseudo_classes(
    i: &str
) -> IResult<&str, Vec<PseudoClass>> {
    wsd(parse_pseudo_classes_impl)(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn impl_active() {
        let i = ":active";
        let parsed = parse_pseudo_classes_impl(i).expect("Should parse").1;
        assert_eq!(parsed, vec![PseudoClass::Active])
    }
    #[test]
    fn active() {
        let i = ":active";
        let parsed = parse_pseudo_classes(i).expect("Should parse").1;
        assert_eq!(parsed, vec![PseudoClass::Active])
    }

    #[test]
    fn other_variant() {
        let i = ":other";
        let parsed = parse_pseudo_classes(i).expect("Should parse").1;
        assert_eq!(parsed, vec![PseudoClass::Other("other".to_string())])
    }

    #[test]
    fn lang() {
        let i = ":lang(fr)";
        let parsed = parse_pseudo_classes(i).expect("Should parse").1;
        assert_eq!(parsed, vec![PseudoClass::Lang("fr".to_string())])
    }

    #[test]
    fn nth_child() {
        let i = ":nth-child(some_garbage)";
        let parsed = parse_pseudo_classes(i).expect("Should parse").1;
        assert_eq!(
            parsed,
            vec![PseudoClass::NthChild("some_garbage".to_string())]
        )
    }

    #[test]
    fn garbage_with_paren() {
        let i = ":aaaah(some_garbage) other";
        let parsed = parse_pseudo_classes(i).expect("Should parse").1;
        assert_eq!(
            parsed,
            vec![PseudoClass::Other("aaaah(some_garbage)".to_string())]
        )
    }

    #[test]
    fn garbage_multiple() {
        let i = ":aaaah:active";
        let parsed = parse_pseudo_classes(i).expect("Should parse").1;
        assert_eq!(
            parsed,
            vec![
                PseudoClass::Other("aaaah".to_string()),
                PseudoClass::Active
            ]
        )
    }
}
