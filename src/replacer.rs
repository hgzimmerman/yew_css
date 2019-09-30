use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt, rest};
use nom::multi::many0;
use nom::sequence::{pair, tuple};
use nom::IResult;
use smallvec::smallvec;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Section<'a> {
    Id {
        hash: &'a str,
        name: &'a str,
    },
    Class {
        dot: &'a str,
        name: &'a str,
    },
    Element {
        name: &'a str,
    },
    RulesBlock {
        leading_space: &'a str,
        open_brace: &'a str,
        content: &'a str,
        close_brace: &'a str,
    },
    Other(&'a str),
}

pub fn mangle_css_string(i: &str, mangle: &str) -> (String, HashMap<String, String>) {
    map(
        insert_mangles_parser,
        |(sections, rest)| mangle_string_and_produce_lut(sections, rest, mangle)
    )(i)
        .unwrap().1
}

fn insert_mangles_parser(
    i: &str,
) -> IResult<&str, (Vec<(Section, Option<Section>)>, Option<&str>)> {
    pair(
        many0(alt((
            pair(get_id, opt(get_block)),
            pair(get_class, opt(get_block)),
            pair(get_element, opt(get_block)),
            map(get_other, |s| (s, None)), // Can only get a rule block after an id, class or element.
        ))),
        opt(rest),
    )(i)
}

fn mangle_string_and_produce_lut(sections: Vec<(Section, Option<Section>)>, rest: Option<&str>, mangle: &str) -> (String, HashMap<String, String>) {
    let hm: HashMap<String, String> = sections
        .iter()
        .filter_map(|s: &(Section, Option<Section>)| match s.0 {
            Section::Id { name, .. } => {
                Some((name.to_string(), format!("{}{}", mangle, name)))
            }
            Section::Class { name, .. } => {
                Some((name.to_string(), format!("{}{}", mangle, name)))
            }
            _ => None,
        })
        .collect();

    let modified_string: String = sections
        .into_iter()
        .map(|(lhs, block)| -> smallvec::IntoIter<[Section; 2]> {
            if let Some(block) = block {
                smallvec![lhs, block].into_iter()
            } else {
                smallvec![lhs].into_iter()
            }
        })
        .flatten()
        .map(|section: Section| -> smallvec::IntoIter<[&str; 4]> {
            // This avoids allocating by using a smallvec instead of a vec.
            // I would still like to find a better way though.
            match section {
                Section::Id { hash, name } => smallvec![hash, mangle, name].into_iter(),
                Section::Class { dot, name } => smallvec![dot, mangle, name].into_iter(),
                Section::Element { name } => smallvec![name].into_iter(),
                Section::Other(other) => smallvec![other].into_iter(),
                Section::RulesBlock {
                    leading_space,
                    open_brace,
                    content,
                    close_brace,
                } => smallvec![leading_space, open_brace, content, close_brace].into_iter(),
            }
        })
        .flatten()
        .chain(rest.into_iter())
        .collect();

    (modified_string, hm)
}

fn is_not_ident_terminator(c: char) -> bool {
    let terminator = " \t\n\r.#,$@%^&*(){}[]<>0123456789";
    !terminator.contains(c)
}

pub fn get_id(i: &str) -> IResult<&str, Section> {
    map(
        pair(tag("#"), take_while1(is_not_ident_terminator)),
        |(hash, name)| Section::Id { hash, name },
    )(i)
}
pub fn get_class(i: &str) -> IResult<&str, Section> {
    map(
        pair(tag("."), take_while1(is_not_ident_terminator)),
        |(dot, name)| Section::Class { dot, name },
    )(i)
}

pub fn get_element(i: &str) -> IResult<&str, Section> {
    map(take_while1(is_not_ident_terminator), |name| {
        Section::Element { name }
    })(i)
}

pub fn get_block(i: &str) -> IResult<&str, Section> {
    map(
        tuple((multispace0, tag("{"), take_until("}"), tag("}"))),
        |(leading_space, open_brace, content, close_brace)| Section::RulesBlock {
            leading_space,
            open_brace,
            content,
            close_brace,
        },
    )(i)
}

pub fn get_other(i: &str) -> IResult<&str, Section> {
    let end_other = ".#";
    map(take_while1(move |x| !end_other.contains(x)), Section::Other)(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_class() {
        let i = ".class";
        let (_, c) = get_class(i).unwrap();
        assert_eq!(
            c,
            Section::Class {
                dot: ".",
                name: "class"
            }
        )
    }

    #[test]
    fn captures_lut() {
        let i = ".yeet {}";
        let (_, lut) = mangle_css_string(i, "mangle-");
        let mut expected = HashMap::new();
        expected.insert("yeet".to_string(), "mangle-yeet".to_string());

        assert_eq!(expected, lut);
    }

    #[test]
    fn replaces_with_mangles() {
        let i = ".yeet {}";
        let (s, _) = mangle_css_string(i, "mangle-");
        let expected = ".mangle-yeet {}";

        assert_eq!(expected, s);
    }

    #[test]
    fn ignores_hash_inside_colon_semi_colon() {
        let i = r##".yeet {
    background-color: #FF00FF;
}"##;
        let (s, _) = mangle_css_string(i, "mangle-");
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
        let (s, _) = mangle_css_string(i, "mangle-");
        let expected = r##".mangle-yeet {
    background-color: #FF00FF
}"##;
        assert_eq!(expected, s);
    }

    #[test]
    fn pseudo_classes_arent_broken() {
        let i = ".yeet:hello {}";
        let (s, _) = mangle_css_string(i, "mangle-");
        let expected = ".mangle-yeet:hello {}";
        assert_eq!(expected, s);
    }
    #[test]
    fn pseudo_classes_arent_broken_2() {
        let i = ".yeet:hello > .other {}";
        let (s, _) = mangle_css_string(i, "mangle-");
        let expected = ".mangle-yeet:hello > .mangle-other {}";
        assert_eq!(expected, s);
    }
}
