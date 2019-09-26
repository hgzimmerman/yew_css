
//pub enum CssTokens {
//    Class{
//        attribute: Option<String>,
//        selector: Selector,
//        rules: Vec<Rule>
//    }
//}

use nom::IResult;
use nom::error::ErrorKind;
use nom::combinator::{not, opt, map, verify};
use nom::branch::alt;
use nom::character::complete::char;
use nom::character::complete::alpha1;
use nom::sequence::{preceded, tuple, delimited, pair, delimitedc};
use nom::bytes::complete::{tag, take_while, tag_no_case};
use nom::character::{is_alphanumeric, is_digit};
use nom::multi::{many1, many0};
use nom::error::ParseError;


pub mod pseudo_element;
pub mod pseudo_class;
pub mod attribute;
pub mod util;

pub use pseudo_class::PseudoClass;
use pseudo_class::parse_pseudo_classes;
use pseudo_element::PseudoElement;
use pseudo_element::parse_pseudo_element;
use crate::parser::attribute::{parse_attribute, Attribute};
use crate::parser::util::is_valid_identifier;


pub struct CssRule {
    selector: Selector,
    declarations: Vec<Declaration>
}

pub struct Declaration {
    property: String,
    value: String
}

pub struct Comment(String);

pub struct Universal;
pub struct Value(String);





#[derive(Debug, PartialEq)]
pub enum Combinator {
    Descendent, // Space
    Child, // >
    Adjacent, // +
    GeneralSibling // ~
}

#[derive(Debug, PartialEq)]
pub enum Selector {
    Element {
        name: String,
        combinator: Box<Option<(Combinator, Selector)>>, // The inner selector can't be the Selector::Selectors variant.
        pseudo_classes: Option<Vec<PseudoClass>>,
        pseudo_element: Option<PseudoElement>,
        attribute: Option<Attribute>
    },
    Class {
        name: String,
        combinator: Box<Option<(Combinator, Selector)>>, // The inner selector can't be the Selector::Selectors variant.
        pseudo_classes: Option<Vec<PseudoClass>>,
        attribute: Option<Attribute>
    },
    Id {
        name: String,
        combinator: Box<Option<(Combinator, Selector)>>, // The inner selector can't be the Selector::Selectors variant.
        pseudo_classes: Option<Vec<PseudoClass>>,
        attribute: Option<Attribute>
    },
    //Selectors(Vec<Selector>), // TODO, is this needed at this level of enum? Probably not.
    Universal,
}

pub fn parse_selector<'a>() -> impl Fn(&'a str) -> IResult<&'a str, Selector> {
    let class_parser = preceded(
        char('.'),
        |i: &str| {
            let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;
            let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) = map(opt(parse_combinator), Box::new)(i)?;
            let (i, pseudo_classes) = parse_pseudo_classes()(i)?;
            let (i, attribute) = opt(parse_attribute())(i)?;
            Ok((
               i,
                Selector::Class {
                    name: name.to_string(),
                    combinator,
                    pseudo_classes,
                    attribute
                }
            ))
        }
    );

    let element_parser = verify(|i: &str | { // wrap in a verify to give type hints about the error type.
        let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;
        let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) = map(opt(parse_combinator), Box::new)(i)?;
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
                attribute
            }
        ))
    },
    |_| true
    );

    let id_parser = preceded(
        char('#'),
        |i: &str| {
            let (i, name): (&str, &str) = take_while(is_valid_identifier)(i)?;
            let (i, combinator): (&str, Box<Option<(Combinator, Selector)>>) = map(opt(parse_combinator), Box::new)(i)?;
            let (i, pseudo_classes) = parse_pseudo_classes()(i)?;
            let (i, attribute) = opt(parse_attribute())(i)?;
            Ok((
                i,
                Selector::Id {
                    name: name.to_string(),
                    combinator,
                    pseudo_classes,
                    attribute
                }
            ))
        }
    );

    let universal_parser = map(tag("*"), |_| Selector::Universal); // This is likely incomplete.
    alt((class_parser, id_parser, universal_parser, element_parser))
}





fn parse_combinator(i: &str) -> IResult<&str, (Combinator, Selector)>  {
    pair(
        alt((
                map(char(' '), |_| Combinator::Descendent),
                map(char('>'), |_| Combinator::Child),
                map(char('+'), |_| Combinator::Adjacent),
                map(char('~'), |_| Combinator::GeneralSibling),
            )),
        parse_selector()
    )(i)
}






