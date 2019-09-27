use nom::branch::{alt, permutation};
use nom::bytes::complete::{tag, tag_no_case, take_while};
use nom::character::complete::alpha1;
use nom::character::complete::char;
use nom::character::{is_alphanumeric, is_digit};
use nom::combinator::{map, not, opt, verify, peek};
use nom::error::ErrorKind;
use nom::error::ParseError;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, delimitedc, pair, preceded, tuple};
use nom::IResult;
use nom::character::complete::multispace1;

use crate::parser::attribute::{parse_attribute, Attribute};
use crate::parser::pseudo_class::parse_pseudo_classes;
use crate::parser::pseudo_class::PseudoClass;
use crate::parser::pseudo_element::parse_pseudo_element;
use crate::parser::pseudo_element::PseudoElement;
use crate::parser::util::{is_valid_identifier, wsd, take_valid_ident_string};

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

pub fn parse_selector<'a>(i: &'a str) -> IResult<&'a str, Selector> {
    let class_parser = preceded(char('.'), |i: &str| {
        let (i, name): (&str, String) = take_valid_ident_string()(i)?;
        let (i, pseudo_classes) = opt(parse_pseudo_classes)(i)?;
        let (i, attribute) = opt(parse_attribute())(i)?;
        let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) =
            map(opt(parse_combinator), Box::new)(i)?;
        Ok((
            i,
            Selector::Class {
                name,
                combinator,
                pseudo_classes,
                attribute,
            },
        ))
    });

    let element_parser = verify(
        |i: &str| {
            // wrap in a verify to give type hints about the error type.
//            let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;

            let (i, name): (&str, String) = take_valid_ident_string()(i)?;
            let (i, (pseudo_classes, pseudo_element)) = pseudo_elements_and_pseudo_classes(i)?;
            let (i, attribute) = opt(parse_attribute())(i)?;

            let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) =
                map(opt(parse_combinator), Box::new)(i)?;
            Ok((
                i,
                Selector::Element {
                    name,
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
        let (i, name): (&str, String) = take_valid_ident_string()(i)?;
        let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) =
            map(opt(parse_combinator), Box::new)(i)?;
        let (i, pseudo_classes) = opt(parse_pseudo_classes)(i)?;
        let (i, attribute) = opt(parse_attribute())(i)?;
        Ok((
            i,
            Selector::Id {
                name,
                combinator,
                pseudo_classes,
                attribute,
            },
        ))
    });

    let universal_parser = map(tag("*"), |_| Selector::Universal); // This is likely incomplete.
    alt((class_parser, id_parser, universal_parser, element_parser))(i)
}

fn parse_combinator(i: &str) -> IResult<&str, (Combinator, Selector)> {
    alt((
        map(pair(wsd(char('>')), parse_selector), |(_, s)| (Combinator::Child, s)),
        map(pair(wsd(char('+')), parse_selector), |(_, s)| (Combinator::Adjacent, s)),
        map(pair(wsd(char('~')), parse_selector), |(_, s)| (Combinator::GeneralSibling, s)),
        map(pair(multispace1, parse_selector), |(_, s)| (Combinator::Descendent, s)), // TODO this broke (a little)?
    ))(i)
}

fn switch_pair<T,U>((t, u): (T,U)) -> (U,T) {
    (u,t)
}

fn pseudo_elements_and_pseudo_classes(i: &str) ->  IResult<&str, (Option<Vec<PseudoClass>>, Option<PseudoElement>)> {
    alt((
        map(map(pair(parse_pseudo_element, parse_pseudo_classes), |(x,y)| (Some(x), Some(y))), switch_pair),
        map(pair( parse_pseudo_classes, parse_pseudo_element), |(x,y)| (Some(x), Some(y))),

        map(pair(opt(parse_pseudo_classes), parse_pseudo_element), |(x, y)| (x, Some(y))),
        map(map(pair(opt(parse_pseudo_element), parse_pseudo_classes), |(x,y)| (x, Some(y))), switch_pair),

        pair( opt(parse_pseudo_classes), opt(parse_pseudo_element)),
    ))(i)
}





#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::attribute::CaseSensitivity;

    #[test]
    fn both_pseudo() {
        let i = "::after:active";
        let (parsed_classes, parsed_element) =  pseudo_elements_and_pseudo_classes(i).unwrap().1;
        assert_eq!(parsed_classes, Some(vec![PseudoClass::Active]));
        assert_eq!(parsed_element, Some(PseudoElement::After));
    }

    #[test]
    fn both_pseudo2() {
        let i = ":active::after";
        let (parsed_classes, parsed_element) =  pseudo_elements_and_pseudo_classes(i).unwrap().1;
        assert_eq!(parsed_classes, Some(vec![PseudoClass::Active]));
        assert_eq!(parsed_element, Some(PseudoElement::After));
    }

    #[test]
    fn class_parse() {
        let i = ".class";
        let parsed = parse_selector(i).expect("should parse").1;
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
        let parsed = parse_selector(i).expect("should parse").1;
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
        let parsed = parse_selector(i).expect("should parse").1;
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
        let parsed = parse_selector(i).expect("should parse").1;
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
            pseudo_classes: Some(vec![PseudoClass::Active]),
            attribute: Some(Attribute {
                name: "attr".to_string(),
                target: None,
                case_sensitivity: CaseSensitivity::Default
            }),
        };
        assert_eq!(parsed, expected)
    }

    #[test]
    fn order_doesnt_matter_for_pseudo_classes() {
        let i = "div:active::after";
        let parsed1 = parse_selector(i).expect("should parse").1;

        let i = "div::after:active";
        let (rest, parsed2) = parse_selector(i).expect("should parse");
        dbg!(rest);
        assert_eq!(parsed1, parsed2)
    }


    #[test]
    fn order_doesnt_matter_for_pseudo_classes_with_spaces() {
        let i = "div :active ::after";
        let parsed1 = parse_selector(i).expect("should parse").1;

        let i = "div::after:active";
        let (rest, parsed2) = parse_selector(i).expect("should parse");
        dbg!(rest);
        assert_eq!(parsed1, parsed2)
    }

    #[test]
    fn element_rejects_invalid_element_name_open_brace() {
        let i = "{";
        parse_selector(i).expect_err("should not parse");
    }

}
