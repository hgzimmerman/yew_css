use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::alpha1;
use nom::character::complete::char;
use nom::character::{is_alphanumeric, is_digit};
use nom::combinator::{map, not, opt, verify};
use nom::error::ErrorKind;
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, delimitedc, pair, preceded, tuple};
use nom::IResult;

use crate::parser::attribute::{parse_attribute, Attribute};
use crate::parser::pseudo_class::parse_pseudo_classes;
use crate::parser::pseudo_class::PseudoClass;
use crate::parser::pseudo_element::parse_pseudo_element;
use crate::parser::pseudo_element::PseudoElement;
use crate::parser::util::is_valid_identifier;

#[derive(Debug, PartialEq)]
pub enum Combinator {
    Descendent,     // Space
    Child,          // >
    Adjacent,       // +
    GeneralSibling, // ~
}

#[derive(Debug, PartialEq)]
pub enum Selector {
    Element {
        name: String,
        combinator: Box<Option<(Combinator, Selector)>>, // The inner selector can't be the Selector::Selectors variant.
        pseudo_classes: Option<Vec<PseudoClass>>,
        pseudo_element: Option<PseudoElement>,
        attribute: Option<Attribute>,
    },
    Class {
        name: String,
        combinator: Box<Option<(Combinator, Selector)>>, // The inner selector can't be the Selector::Selectors variant.
        pseudo_classes: Option<Vec<PseudoClass>>,
        attribute: Option<Attribute>,
    },
    Id {
        name: String,
        combinator: Box<Option<(Combinator, Selector)>>, // The inner selector can't be the Selector::Selectors variant.
        pseudo_classes: Option<Vec<PseudoClass>>,
        attribute: Option<Attribute>,
    },
    //Selectors(Vec<Selector>), // TODO, is this needed at this level of enum? Probably not.
    Universal,
}

pub fn parse_selector<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Selector> {
    let class_parser = preceded(char('.'), |i: &str| {
        let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;
        let (i, pseudo_classes) = parse_pseudo_classes()(i)?;
        let (i, attribute) = opt(parse_attribute())(i)?;
        let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) =
            map(opt(parse_combinator), Box::new)(i)?;
        Ok((
            i,
            Selector::Class {
                name: name.to_string(),
                combinator,
                pseudo_classes,
                attribute,
            },
        ))
    });

    let element_parser = verify(
        |i: &str| {
            // wrap in a verify to give type hints about the error type.
            let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;
            let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) =
                map(opt(parse_combinator), Box::new)(i)?;
            let (i, pseudo_classes) = parse_pseudo_classes()(i)?;
            let (i, pseudo_element) = parse_pseudo_element(i)?;
            let (i, attribute) = opt(parse_attribute())(i)?;
            Ok((
                i,
                Selector::Element {
                    name: name.to_string(),
                    combinator,
                    pseudo_classes,
                    pseudo_element,
                    attribute,
                },
            ))
        },
        |_| true,
    );

    let id_parser = preceded(char('#'), |i: &str| {
        let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;
        let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) =
            map(opt(parse_combinator), Box::new)(i)?;
        let (i, pseudo_classes) = parse_pseudo_classes()(i)?;
        let (i, attribute) = opt(parse_attribute())(i)?;
        Ok((
            i,
            Selector::Id {
                name: name.to_string(),
                combinator,
                pseudo_classes,
                attribute,
            },
        ))
    });

    let universal_parser = map(tag("*"), |_| Selector::Universal); // This is likely incomplete.
    alt((class_parser, id_parser, universal_parser, element_parser))
}

fn parse_combinator(i: &str) -> IResult<&str, (Combinator, Selector)> {
    pair(
        alt((
            map(char('>'), |_| Combinator::Child),
            map(char('+'), |_| Combinator::Adjacent),
            map(char('~'), |_| Combinator::GeneralSibling),
            map(char(' '), |_| Combinator::Descendent), // TODO this broke
        )),
        parse_selector(),
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::attribute::CaseSensitivity;

    #[test]
    fn class_parse() {
        let i = ".class";
        let parsed = parse_selector()(i).expect("should parse").1;
        let expected = Selector::Class {
            name: "class".to_string(),
            combinator: Box::new(None),
            pseudo_classes: None,
            attribute: None,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn element_parse() {
        let i = "div";
        let parsed = parse_selector()(i).expect("should parse").1;
        let expected = Selector::Element {
            name: "div".to_string(),
            combinator: Box::new(None),
            pseudo_classes: None,
            pseudo_element: None,
            attribute: None,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn id_parse() {
        let i = "#unique";
        let parsed = parse_selector()(i).expect("should parse").1;
        let expected = Selector::Id {
            name: "unique".to_string(),
            combinator: Box::new(None),
            pseudo_classes: None,
            attribute: None,
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn full_class() {
        let i = ".class:active[attr] > div";
        let parsed = parse_selector()(i).expect("should parse").1;
        let expected = Selector::Class {
            name: "class".to_string(),
            combinator: Box::new(Some((
                Combinator::Child,
                Selector::Element {
                    name: "div".to_string(),
                    combinator: Box::new(None),
                    pseudo_classes: None,
                    pseudo_element: None,
                    attribute: None,
                },
            ))),
            pseudo_classes: None,
            attribute: Some(Attribute {
                name: "attr".to_string(),
                target: None,
                case_sensitivity: CaseSensitivity::Default
            }),
        };
        assert_eq!(parsed, expected)
    }

}
