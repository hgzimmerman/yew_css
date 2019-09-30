use std::collections::HashMap;
use nom::IResult;
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::combinator::{map, rest, not, opt};
use nom::bytes::complete::{take_until, take_while, tag};
use nom::character::complete::char;
use nom::sequence::{preceded, pair, delimited};

#[derive(Debug, PartialEq)]
pub enum Section<'a> {
    Id(&'a str),
    Class(&'a str),
    ValueSemiColonTerminated(&'a str),
    ValueCloseBracketTerminated(&'a str),
    Other(&'a str)
}

pub fn mangle_css_string(i: &str, mangle: &str) -> (String, HashMap<String, String>) {
    insert_mangles_impl(i, mangle).unwrap().1
}


pub fn insert_mangles_impl<'a>(i: &'a str, mangle: &str) -> IResult<&'a str, (String, HashMap<String, String>)> {
    // TODO there's probably a few ways to improve performance here, any PR is welcome.
    map(
        pair(
            many0(
                alt((
                    get_id,
                    get_class,
                    get_value_semi_colon_terminated,
                    get_other,
                ))
            ),
            opt(rest)
        ),
        |(sections, rest): (Vec<Section>, Option<&str>)| {
            let hm: HashMap<String, String> = sections.iter().filter_map(|s| {
                match s {
                    Section::Id(id) => {
                        Some((id.to_string(), format!("{}{}", mangle, id)))
                    }
                    Section::Class(class) => {
                        Some((class.to_string(), format!("{}{}", mangle, class)))
                    }
                    _ => None
                }
            }).collect();

            let mut modified_string = sections.into_iter()
                .fold(String::new(), |acc, section |{ // TODO consider joining instead of folding.
                    match section {
                        Section::Id(id) => acc + &format!("#{}{}", mangle, id),
                        Section::Class(class) => acc + &format!(".{}{}", mangle, class),
                        Section::ValueSemiColonTerminated(any) => acc + &format!(":{};", any),
                        Section::ValueCloseBracketTerminated(any) => acc + &format!(":{}}}", any),
                        Section::Other(other) => acc + other
                    }
                });
            if let Some(rest) = rest {
                modified_string = modified_string + rest // This may force another allocation
            }

            (modified_string, hm)
        }
    )(i)
}

fn is_not_ident_terminator(c: char) -> bool {
    let terminator = " \t\n\r.#,$@%^&*(){}[]<>{0123456789";
    !terminator.contains(c)
}

pub fn get_id(i: &str) -> IResult<&str, Section> {
    map(
        preceded(char('#'), take_while(is_not_ident_terminator)),
            Section::Id
    )(i)
}
pub fn get_class(i: &str) -> IResult<&str, Section> {
    map(
    preceded(char('.'), take_while(is_not_ident_terminator)),
    Section::Class
    )(i)
}

pub fn get_other(i: &str) -> IResult<&str, Section> {
    map(
    take_until(".#{"), // TODO nneeds a variant that is context-aware.
        Section::Other
    )(i)
}

// TODO this will break on pseudo-classes
// to fix, add structure to the parser, where it can detect a {} block for ignoring #s only after a class, id, or element.
// This requires that classes can have valid data after them
// so a datatype for class would be classname, opt(other|class|id), brackets

// better would be classname, opt(bracketed)
pub fn get_value_semi_colon_terminated(i: &str) -> IResult<&str, Section> {
    map(
        delimited(char(':'), take_until(";"), char(';')),
        Section::ValueSemiColonTerminated
    )(i)
}

// TODO this will break on pseudo-classes
pub fn get_value_close_bracket_terminated(i: &str) -> IResult<&str, Section> {
    map(
        delimited(char(':'), take_until("}"), char('}')),
        Section::ValueCloseBracketTerminated
    )(i)
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_class() {
        let i = ".class";
        let (_, c) = get_class(i).unwrap();
        assert_eq!(c, Section::Class("class"))
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
        let (_, (s, _)) = insert_mangles_impl(i, "mangle-").unwrap();
        let expected = ".mangle-yeet:hello > .mangle-other {}";
        assert_eq!(expected, s);
    }
}