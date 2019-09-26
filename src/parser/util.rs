use nom::bytes::complete::{take, take_until, take_while};
use nom::character::complete::multispace0;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, preceded};
use nom::IResult;

pub(crate) fn is_valid_identifier(c: char) -> bool {
    let invalid = " \n\t:[](){}<>&*$#.,;";
    !invalid.contains(c)
}

pub(crate) fn is_not_close_paren(c: char) -> bool {
    c != ')'
}

fn is_not_space(c: char) -> bool {
    c != ' '
}

pub(crate) fn take_valid_ident<'a>() -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    take_while(is_valid_identifier)
}
pub(crate) fn take_valid_ident_string<'a>() -> impl Fn(&'a str) -> IResult<&'a str, String> {
    map(take_valid_ident(), String::from)
}

pub(crate) fn take_not_close_paren_string<'a>() -> impl Fn(&'a str) -> IResult<&'a str, String> {
    map(take_while(is_not_close_paren), String::from)
}

pub(crate) fn take_not_space_string<'a>() -> impl Fn(&'a str) -> IResult<&'a str, String> {
    map(take_while(is_not_space), String::from)
}

/// Take items until a char is encountered from either of the two lists.
/// If the next char in the sequence is consumable, the parser will advance one char to consume that char.
/// Otherwise, the char will not be consumed.
pub(crate) fn take_until_encountered<'a>(
    consumable: &'a str,
    dont_consume: &'a str,
) -> impl Fn(&'a str) -> IResult<&'a str, String> {
    let mut last_matched = false;
    debug_assert!(!consumable.chars().any(|x: char| dont_consume.contains(x)));

    move |i: &'a str| {
        let (i, captured) =
            take_while(|c| !consumable.contains(c) && !dont_consume.contains(c))(i)?;
        if let Some(next_char) = i.chars().next() {
            if consumable.contains(next_char) {
                let (i, last) = take(1usize)(i)?;
                return Ok((i, [captured, last].join("")));
            }
        }

        Ok((i, captured.to_string()))
    }
}

// preceding whitespace consumer
pub(crate) fn ws<'a, T>(
    f: impl Fn(&'a str) -> IResult<&'a str, T>,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    preceded(multispace0, f)
}
// removes surrounding whitespace.
pub(crate) fn wsd<'a, T>(
    f: impl Fn(&'a str) -> IResult<&'a str, T>,
) -> impl Fn(&'a str) -> IResult<&'a str, T> {
    delimited(multispace0, f, multispace0)
}
