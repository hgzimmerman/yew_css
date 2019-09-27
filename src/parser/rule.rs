use crate::parser::selector::{Selector, parse_selector};
use crate::parser::declaration::{Declaration, parse_declarations};
use nom::combinator::map;
use nom::sequence::pair;
use crate::parser::util::wsd;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct CssRule {
    selector: Selector,
    declarations: Vec<Declaration>,
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
    use crate::parser::rule::{parse_css_rule, CssRule};
    use crate::parser::selector::Selector;
    use crate::parser::declaration::Declaration;

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
