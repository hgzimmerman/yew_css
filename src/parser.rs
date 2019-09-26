use crate::parser::selector::Selector;

pub mod attribute;
pub mod pseudo_class;
pub mod pseudo_element;
pub mod selector;
pub mod util;

pub struct CssRule {
    selector: Selector,
    declarations: Vec<Declaration>,
}

pub struct Declaration {
    property: String,
    value: String,
}

pub struct Comment(String);

pub struct Universal;
pub struct Value(String);
