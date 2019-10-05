//! https://davidwalsh.name/add-rules-stylesheets
//! https://github.com/andrewwakeling/create-stylesheet
use std::cell::RefCell;
use std::rc::Rc;
use std::thread_local;
//use stdweb::{js, web::{document, Document, Element, INode}};
mod replacer;
use replacer::mangle_css_string;

use std::collections::HashMap;
use std::ops::Index;
//use stdweb::unstable::TryFrom;

//use stdweb::web::error::{InvalidStateError, IndexSizeError, SyntaxError, HierarchyRequestError};

use web_sys::{Document, Node, CssStyleSheet, StyleSheet, window, Element};

use wasm_bindgen::JsValue;


thread_local! {
    /// Global counter used to keep css items distinct
    static SHARED_MANGLER_COUNT: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
}

#[macro_export]
macro_rules! css {
    ($mangle_string: expr, $css: expr) => {
        $crate::CssService::with_mangler($mangle_string.to_string()).attach_css($css)
    };
    ($css: expr) => {
        $crate::CssService::new().attach_css($css)
    };
}

#[macro_export]
macro_rules! css_file {
    ($mangle_string: expr, $file: expr) => {
        $crate::CssService::with_mangler($mangle_string.to_string()).attach_css(include_str!($file))
    };
    ($file: expr) => {
        $crate::CssService::new().attach_css(include_str!($file))
    };
}

#[derive(Debug)]
pub struct CssService {
    /// Identifying string for the css styles
    mangler: String,
    /// Reference to the document.
    document: Document,
}

//fn create_style_element(document: &Document) -> Element {
//    let style: Element = document.create_element("style").unwrap();
//    style.append_child(&document.create_text_node("")); // Hack for webkit.
//    document.head().unwrap().append_child(&style);
//    style
//}

fn create_style_element(document: &Document) -> CssStyleSheet {
    let style = document.create_element("style").expect("Could not create style");
    style.set_attribute("type", "text/css");
    style.append_child(&document.create_text_node(""));
    document.head().expect("Could not get head").append_child(&style);
    let js_value: JsValue = style.into();
    CssStyleSheet::from(js_value)
}

impl CssService {
    pub fn new() -> Self {
        CssService {
            mangler: "".to_string(),
            document: web_sys::window().unwrap().document().unwrap(),
        }
    }

    pub fn with_mangler(mangler: String) -> Self {
        CssService {
            mangler,
            document: web_sys::window().unwrap().document().unwrap(),
        }
    }

    pub fn attach_css(&mut self, css: &str) -> Css {
        let new_id: usize = SHARED_MANGLER_COUNT.with(|smc| {
            let mut count = smc.as_ref().borrow_mut();
            *count = 1 + *count;
            *count
        });

        let mangler = format_mangler(&self.mangler, new_id);
        let (mangled_css, mangle_lut) = mangle_css_string(css, &mangler);

        // create a new style item.
        let style: CssStyleSheet = create_style_element(&self.document);

        let style_element = Element::from(JsValue::from(style.clone()));
        style_element.set_text_content(Some(&mangled_css));

        Css {
            mangler: self.mangler.clone(),
            mangler_id: new_id,
            style,
            mangle_lut,
        }
    }
}
// https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/insertRule#Restrictions
pub enum CssError {
    IndexSizeError,
    HierarchyRequestError,
    SyntaxError,
    InvalidStateError
}

/// A handle to a stylesheet, that mangles its owned CSS.
#[derive(Debug)]
pub struct Css {
    /// Name to prepend before class names to prevent collisions.
    mangler: String,
    /// Id which keeps cloned CSS distinct.
    mangler_id: usize,
    /// The stylesheet element in the DOM.
    style: CssStyleSheet,
    /// Look up table for classes.
    mangle_lut: HashMap<String, String>,
}

impl Css {
    /// Replaces the text in this stylesheet with the new string.
    pub fn overwrite(&mut self, css: String) {
        let (css, lut) = mangle_css_string(&css, &self.mangler);
        let style_element = Element::from(JsValue::from(self.style.clone()));
        style_element.set_text_content(Some(&css));
        self.mangle_lut = lut
    }

    /// Gets the mangled css from the stylesheet itself.
    pub fn inner_css(&self) -> String {
        let style_element = Element::from(JsValue::from(self.style.clone()));
        style_element.text_content().unwrap()
    }

    /// Gets the mangler associated with this css sheet.
    pub fn get_mangler(&self) -> String {
        format_mangler(&self.mangler, self.mangler_id)
    }

//    /// Adds a style to this sheet
//    /// https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/insertRule
//    pub fn insert_rule(&mut self, css: &str) -> Result<u32, JsValue>{
//        let (css, lut) = mangle_css_string(css, &self.get_mangler());
//        log::info!("inserting rule: {}", css);
//        let retval = self.style.insert_rule(&css);
////        log::info!("{:?}", self.style.css_rules().unwrap().length());  // TODO adding doesn't work
//        self.mangle_lut.extend(lut);
//        retval
//    }
//    pub fn insert_rule_with_index(&mut self, css: &str, index: u32) -> Result<(), CssError> {
//        let (css, lut) = mangle_css_string(css, &self.get_mangler());
//
//        self.style.insert_rule_with_index(&css, index);
//        self.mangle_lut.extend(lut);
//        return Ok(());
//    }

//    pub fn remove_rule(&mut self, index: u32) {
//        self.style.delete_rule(index);
//    }

    pub fn get(&self, index: &str) -> Option<String> {
        self.mangle_lut.get(index).cloned()
    }
}



// TODO make a SafeCss that wraps the Css struct and will insert mangled queries into an R<HashMap<String, String>> if it can't find the index.
impl Index<&str> for Css {
    type Output = String;

    fn index(&self, index: &str) -> &Self::Output {
        self.mangle_lut.get(index).unwrap_or_else(|| {
            panic!(format!(
                "CSS class or id name: '{}' does not exist in this css sheet with the mangler: {}",
                index,
                self.get_mangler()
            ))
        })
    }
}

fn format_mangler(mangle: &str, id: usize) -> String {
    if id == 0 {
        format!("{}-", mangle)
    } else {
        format!("{}__{}-", mangle, id)
    }
}

impl Drop for Css {
    /// On drop, remove the stylesheet from the DOM.
    fn drop(&mut self) {
        let style_node = Node::from(JsValue::from(self.style.clone()));

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .head()
            .map(Node::from)
            .expect("could not get head")
            .remove_child(&style_node)
            .expect("could not remove style sheet for css");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn mangle_css() {
        let css = ".class {lorem: ipsum}";
        let mangle_str = "mangle-";
        let (new_css, _mapping_lut) = mangle_css_string(css, mangle_str);
        let expected_css = ".mangle-class {lorem: ipsum}";

        assert_eq!(new_css, expected_css)
    }

    #[test]
    fn mangle_css_lut() {
        let css = ".class {lorem: ipsum}";
        let mangle_str = "mangle-";
        let (_new_css, mapping_lut) = mangle_css_string(css, mangle_str);

        let mut expected_lut = HashMap::new();
        expected_lut.insert("class".to_string(), "mangle-class".to_string());

        assert_eq!(expected_lut, mapping_lut);
    }

    // cant run on x86 target :(
    //    #[test]
    //    fn css_index() {
    //        let css = ".class {lorem: ipsum}";
    //        let css = CssService::with_mangler("mangle_me".to_string()).attach_css(css);
    //        let x = &css["class"];
    //    }
}
