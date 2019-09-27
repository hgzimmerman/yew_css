
pub mod attribute;
pub mod pseudo_class;
pub mod pseudo_element;
pub mod selector;
pub mod util;

#[derive(Debug, PartialEq)]
pub struct CssRule {
    selector: Selector,
    declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq)]
pub struct Declaration {
    property: String,
    value: String,
}

pub struct Comment(String);

pub struct Universal;
pub struct Value(String);


use crate::parser::selector::{Selector, parse_selector};
use nom::IResult;
use crate::parser::util::{take_until_encountered, wsd};
use nom::combinator::{map, opt};
use nom::sequence::{pair, separated_pair, delimited};
use nom::character::complete::char;
use nom::multi::separated_list;

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
//    let safe_declaration = wsd(map(pair(parse_declaration, opt(take_until_encountered("",";")), |(declaration, _)|))
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


pub (crate) fn parse_css_rule(i: &str) -> IResult<&str, CssRule> {
    map(
        pair(wsd(parse_selector), wsd(parse_declarations)),
        |(selector, declarations)| {
            CssRule {
                selector,
                declarations
            }
        }
    )(i)
}

#[cfg(test)]
mod test {
    use crate::parser::{parse_css_rule, CssRule, Declaration};
    use crate::parser::selector::Selector;

    #[test]
    fn parse_rule_simple() {
        let i = "body { background-color: blue; }";
        let parsed = parse_css_rule(i).expect("should_parse").1;
        let expected = CssRule {
            selector: Selector::Element {
                name: "body".to_string(),
                combinator: Box::new(None),
                pseudo_classes: None,
                pseudo_element: None,
                attribute: None
            },
            declarations: vec![
                Declaration {
                    property: "background-color".to_string(),
                    value: "blue".to_string()
                }
            ]
        };
        assert_eq!(parsed, expected);
    }


    #[test]
    fn parse_rule_multi_line() {
        let i = "body { \
            background-color: blue; \
        }";
        let parsed = parse_css_rule(i).expect("should_parse").1;
        let expected = CssRule {
            selector: Selector::Element {
                name: "body".to_string(),
                combinator: Box::new(None),
                pseudo_classes: None,
                pseudo_element: None,
                attribute: None
            },
            declarations: vec![
                Declaration {
                    property: "background-color".to_string(),
                    value: "blue".to_string()
                }
            ]
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_rule_multi_line_multiple_declarations() {
        let i = "body { \
            background-color: blue; \
            width: 32px;
        }";
        let parsed = parse_css_rule(i).expect("should_parse").1;
        let expected = CssRule {
            selector: Selector::Element {
                name: "body".to_string(),
                combinator: Box::new(None),
                pseudo_classes: None,
                pseudo_element: None,
                attribute: None
            },
            declarations: vec![
                Declaration {
                    property: "background-color".to_string(),
                    value: "blue".to_string()
                },
                Declaration {
                    property: "width".to_string(),
                    value: "32px".to_string()
                }
            ]
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn parse_rule_multi_line_multiple_declarations_missing_last_semicolon() {
        let i = "body { \
            background-color: blue; \
            width: 32px
        }";
        let (rest, parsed) = parse_css_rule(i).expect("should_parse");
        assert_eq!(rest, "");
        let expected = CssRule {
            selector: Selector::Element {
                name: "body".to_string(),
                combinator: Box::new(None),
                pseudo_classes: None,
                pseudo_element: None,
                attribute: None
            },
            declarations: vec![
                Declaration {
                    property: "background-color".to_string(),
                    value: "blue".to_string()
                },
                Declaration {
                    property: "width".to_string(),
                    value: "32px".to_string()
                }
            ]
        };
        assert_eq!(parsed, expected);
    }
}