use std::collections::HashMap;
use nom::IResult;
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::combinator::{map, rest, not, opt};
use nom::bytes::complete::{take_until, take_while, tag, take_while1};
use nom::character::complete::{char, multispace0};
use nom::sequence::{preceded, pair, delimited, tuple};

#[derive(Debug, PartialEq)]
pub enum Section<'a> {
    Id {
        hash: &'a str,
        name: &'a str
    },
    Class {
        dot: &'a str,
        name: &'a str
    },
    Element {
        name: &'a str
    },
    RulesBlock {
        leading_space: &'a str,
        open_brace: &'a str,
        content: &'a str,
        close_brace: &'a str
    },
    Other(&'a str)
}

pub fn mangle_css_string(i: &str, mangle: &str) -> (String, HashMap<String, String>) {
    insert_mangles_impl(i, mangle).unwrap().1
}

fn insert_mangles_parser<'a>(i: &'a str) -> IResult<&'a str, (Vec<(Section<'a>, Option<Section<'a>>)>, Option<&'a str>)> {
    pair(
        many0(
            alt((
                pair(get_id, opt(get_block)),
                pair(get_class, opt(get_block)),
                pair(get_element, opt(get_block)),
                map (get_other, |s| (s, None)), // Can only get a rule block after an id, class or element.
            ))
        ),
        opt(rest)
    )(i)
}


pub fn insert_mangles_impl<'a>(i: &'a str, mangle: &str) -> IResult<&'a str, (String, HashMap<String, String>)> {
    // TODO there's probably a few ways to improve performance here, any PR is welcome.
    map(
        insert_mangles_parser,
        |(sections, rest): (Vec<(Section, Option<Section>)>, Option<&str>)| {
            let hm: HashMap<String, String> = sections.iter().filter_map(|s: &(Section, Option<Section>)| {
                match s.0 {
                    Section::Id{ name, ..} => {
                        Some((name.to_string(), format!("{}{}", mangle, name)))
                    }
                    Section::Class{name, ..} => {
                        Some((name.to_string(), format!("{}{}", mangle, name)))
                    }
                    _ => None
                }
            }).collect();

            let mut modified_string: String = sections
                .into_iter()
                .map(|(lhs, block)| {
                    if let Some(block) = block {
                        vec![lhs, block].into_iter()
                    } else {
                        vec![lhs].into_iter()
                    }
                })
                .flatten()
                .map(|section: Section| {
                    match section {
                        Section::Id {hash, name} => vec![hash, mangle, name].into_iter(), // TODO a whole bunch of allocation here, If there's a better way to collect heterogeneous collections of strs into strings, I need to find it out.
                        Section::Class{dot, name} => vec![dot, mangle, name].into_iter(),
                        Section::Element{name} => vec![name].into_iter(),
                        Section::Other(other) => vec![other].into_iter(),
                        Section::RulesBlock { leading_space, open_brace, content, close_brace } => vec![leading_space, open_brace, content, close_brace].into_iter()
                    }
                })
                .flatten()
                .collect();
            if let Some(rest) = rest {
                modified_string = modified_string + rest // This may force another allocation
            }

            (modified_string, hm)
        }
    )(i)
}

fn is_not_ident_terminator(c: char) -> bool {
    let terminator = " \t\n\r.#,$@%^&*(){}[]<>0123456789";
    !terminator.contains(c)
}

pub fn get_id(i: &str) -> IResult<&str, Section> {
    map(
        pair(tag("#"), take_while1(is_not_ident_terminator)),
        |(hash, name)| Section::Id {hash, name}
    )(i)
}
pub fn get_class(i: &str) -> IResult<&str, Section> {
    println!("Getting class with input: '{}'", i);
    map(
    pair(tag("."), take_while1(is_not_ident_terminator)),
    |(dot, name)| Section::Class {dot, name}
    )(i)
}

pub fn get_element(i: &str) -> IResult<&str, Section> {
    println!("Getting element with input: '{}'", i);
    map(
        take_while1(is_not_ident_terminator),
        |name| Section::Element{name}
    )(i)
}

pub fn get_block(i: &str) -> IResult<&str, Section> {
    map(
        tuple((
            multispace0,
            tag("{"),
            take_until("}"),
            tag("}"),
        )),
        |(leading_space, open_brace, content, close_brace)| Section::RulesBlock{ leading_space, open_brace, content, close_brace}
    )(i)
}

pub fn get_other(i: &str) -> IResult<&str, Section> {
    println!("attempting to get other with input: '{}'", i);
    let end_other = ".#";
    map(
        take_while1(move |x| !end_other.contains(x)),
//    take_until(alt((".#{"))), // TODO needs a variant that is context-aware.
        Section::Other
    )(i)
}

// TODO this will break on pseudo-classes
// to fix, add structure to the parser, where it can detect a {} block for ignoring #s only after a class, id, or element.
// This requires that classes can have valid data after them
// so a datatype for class would be classname, opt(other|class|id), brackets

// better would be classname, opt(bracketed)
//pub fn get_value_semi_colon_terminated(i: &str) -> IResult<&str, Section> {
//    map(
//        delimited(char(':'), take_until(";"), char(';')),
//        Section::ValueSemiColonTerminated
//    )(i)
//}
//
//// TODO this will break on pseudo-classes
//pub fn get_value_close_bracket_terminated(i: &str) -> IResult<&str, Section> {
//    map(
//        delimited(char(':'), take_until("}"), char('}')),
//        Section::ValueCloseBracketTerminated
//    )(i)
//}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_class() {
        let i = ".class";
        let (_, c) = get_class(i).unwrap();
        assert_eq!(c, Section::Class{dot: ".", name: "class"})
    }


    #[test]
    fn captures_lut() {
        let i = ".yeet {}";
        let (_, (_, lut)) = insert_mangles_impl(i, "mangle-").unwrap();
        let mut expected = HashMap::new();
        expected.insert("yeet".to_string(), "mangle-yeet".to_string());

        assert_eq!(expected, lut);
    }

    #[test]
    fn replaces_with_mangles() {
        let i = ".yeet {}";
        let (_, (s, _)) = insert_mangles_impl(i, "mangle-").unwrap();
        let expected = ".mangle-yeet {}";

        assert_eq!(expected, s);
    }

    #[test]
    fn ignores_hash_inside_colon_semi_colon() {
        let i = r##".yeet {
    background-color: #FF00FF;
}"##;
        let (_, (s, _)) = insert_mangles_impl(i, "mangle-").unwrap();
        let expected = r##".mangle-yeet {
    background-color: #FF00FF;
}"##;
        assert_eq!(expected, s);
    }

    #[test]
    fn ignores_hash_inside_colon_bracket() {
        let i = r##".yeet {
    background-color: #FF00FF
}"##;
        let (_, (s, _)) = insert_mangles_impl(i, "mangle-").unwrap();
        let expected = r##".mangle-yeet {
    background-color: #FF00FF
}"##;
        assert_eq!(expected, s);
    }

    #[test]
    fn pseudo_classes_arent_broken() {
        let i = ".yeet:hello {}";
        let (_, (s, _)) = insert_mangles_impl(i, "mangle-").unwrap();
        let expected = ".mangle-yeet:hello {}";
        assert_eq!(expected, s);
    }
    #[test]
    fn pseudo_classes_arent_broken_2() {
        let i = ".yeet:hello > .other {}";
//        let ast = insert_mangles_parser(i).unwrap();
//        dbg!(ast);
//        panic!("yaoeuaoeu");

        let (_, (s, _)) = insert_mangles_impl(i, "mangle-").unwrap();
        let expected = ".mangle-yeet:hello > .mangle-other {}";
        assert_eq!(expected, s);
    }
}