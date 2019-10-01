//! https://davidwalsh.name/add-rules-stylesheets
use std::cell::RefCell;
use std::rc::Rc;
use std::thread_local;
use stdweb::{js, web::{document, Document, Element, INode}};
mod replacer;
use replacer::mangle_css_string;

use std::collections::HashMap;
use std::ops::Index;
use stdweb::unstable::TryFrom;

use stdweb::web::error::{InvalidStateError, IndexSizeError, SyntaxError, HierarchyRequestError};
use stdweb::Value;

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

fn create_style_element(document: &Document) -> Element {
    let style: Element = document.create_element("style").unwrap();
    style.append_child(&document.create_text_node("")); // Hack for webkit.
    document.head().unwrap().append_child(&style);
    style
}

impl CssService {
    pub fn new() -> Self {
        CssService {
            mangler: "".to_string(),
            document: document(),
        }
    }

    pub fn with_mangler(mangler: String) -> Self {
        CssService {
            mangler,
            document: document(),
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
        let style: Element = create_style_element(&self.document);
        style.set_text_content(&mangled_css);

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
#[derive(Debug, PartialEq)]
pub struct Css {
    /// Name to prepend before class names to prevent collisions.
    mangler: String,
    /// Id which keeps cloned CSS distinct.
    mangler_id: usize,
    /// The stylesheet element in the DOM.
    style: Element,
    /// Look up table for classes.
    mangle_lut: HashMap<String, String>,
}

impl Css {
    /// Replaces the text in this stylesheet with the new string.
    /// Also replaces the unaltered copy stored in this object.
    pub fn overwrite(&mut self, css: String) {
        self.style.set_text_content(&css);
    }

    /// Gets the mangled css from the stylesheet itself.
    pub fn inner_css(&self) -> String {
        self.style.text_content().unwrap()
    }

    /// Gets the mangler associated with this css sheet.
    pub fn get_mangler(&self) -> String {
        format_mangler(&self.mangler, self.mangler_id)
    }

    /// Adds a style to this sheet
    /// https://developer.mozilla.org/en-US/docs/Web/API/CSSStyleSheet/insertRule
    pub fn insert_rule(&mut self, css: &str) -> Result<(), CssError>{
        let css = mangle_css_string(css, &self.get_mangler()).0;
        let sheet = &self.style;
        let error: Value = js! {
            try {
                @{sheet}.insert_rule(@{css});
            } catch (e) {
                return e;
            }
        };
        if let Ok(_) = InvalidStateError::try_from(error.clone()) {
            return Err(CssError::InvalidStateError)
        }
        if let Ok(_) = IndexSizeError::try_from(error.clone()) {
            return Err(CssError::IndexSizeError)
        }
        if let Ok(_) = HierarchyRequestError::try_from(error.clone()) {
            return Err(CssError::HierarchyRequestError)
        }
        if let Ok(_) = SyntaxError::try_from(error) {
            return Err(CssError::HierarchyRequestError)
        }
        return Ok(())
    }
    pub fn insert_rule_with_index(&mut self, css: &str, index: i32) -> Result<(), CssError> {
        let css = mangle_css_string(css, &self.get_mangler()).0;
        let sheet = &self.style;
        let error: Value = js! {
            try {
                @{sheet}.insert_rule(@{css}, @{index});
            } catch (e) {
                return e;
            }
        };
        if let Ok(_) = InvalidStateError::try_from(error.clone()) {
            return Err(CssError::InvalidStateError)
        }
        if let Ok(_) = IndexSizeError::try_from(error.clone()) {
            return Err(CssError::IndexSizeError)
        }
        if let Ok(_) = HierarchyRequestError::try_from(error.clone()) {
            return Err(CssError::HierarchyRequestError)
        }
        if let Ok(_) = SyntaxError::try_from(error) {
            return Err(CssError::HierarchyRequestError)
        }
        return Ok(());
    }

    pub fn remove_rule(&mut self, index: u32) {
        let sheet = &self.style;
        js! {
            @{sheet}.remove_rule(@{index});
        }
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
        document()
            .head()
            .expect("could not get head")
            .remove_child(&self.style)
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
