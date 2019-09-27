//#![type_length_limit="1483678"]
//#![type_length_limit="2225506"]
#![type_length_limit="18451204"]

//! https://davidwalsh.name/add-rules-stylesheets
use std::cell::RefCell;
use std::rc::Rc;
use std::thread_local;
use stdweb::web::{document, Document, Element, INode};
//mod parser;

use regex::{Regex, Match, Captures};
use std::collections::HashMap;
use std::ops::Index;

thread_local! {
    /// Global counter used to keep css items distinct
    static SHARED_MANGLER_COUNT: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
}

thread_local! {
    /// Only instantiate the css mangler regex once.
    static CSS_MANGLE_REGEX: Regex = Regex::new(r##"(?P<punct>[\.#])(?P<class_or_id>[^\s{#\.\d]+)"##).unwrap();
}

fn mangle_css_string(css: &str, mangle_str: &str) -> (String, HashMap<String, String>)
{
    let lut: HashMap<String, String> = CSS_MANGLE_REGEX.with(|re|
        re
            .captures_iter(css)
            .map(|m: Captures| {
                m.iter()
                    .flatten()
                    .map(|m: Match| {
                        println!("yeet_ {}", m.as_str());

                        let mut key = m.as_str();
                        key = &key[1..];

                        let mangled = format!("{}{}", mangle_str, key);
                        (key.to_string(), mangled)
                    })
                    .next().unwrap()
            })
            .collect()
    );

    let replaced = CSS_MANGLE_REGEX.with(|re| {
        re.replace_all(css, format!("${{punct}}{}$class_or_id", mangle_str).as_str())
    });
    (replaced.to_string(), lut)
}


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
        let (mangled_css, mangle_lut) = mangle_css_string(css, &mangler );

        // create a new style item.
        let style: Element = create_style_element(&self.document);
        style.set_text_content(&mangled_css);

        Css {
            css: css.to_string(),
            mangler: self.mangler.clone(),
            mangler_id: new_id,
            style,
            mangle_lut
        }
    }
}

/// A handle to a stylesheet, that mangles its owned CSS.
pub struct Css {
    /// The unadulterated css.
    css: String,
    /// Name to prepend before class names to prevent collisions.
    mangler: String,
    /// Id which keeps cloned CSS distinct.
    mangler_id: usize,
    /// The stylesheet element in the DOM.
    style: Element,
    mangle_lut: HashMap<String, String>
}

impl Css {
    /// Replaces the text in this stylesheet with the new string.
    pub fn overwrite_css(&mut self, css: String) {
        self.style.set_text_content(&css);
        self.css = css;
    }

    pub fn inner_css(&self) -> String {
        self.style.text_content().unwrap()
    }

    pub fn plain_css(&self) -> String {
        self.css.clone()
    }

    pub fn get_mangler(&self) -> String {
        format_mangler(&self.mangler, self.mangler_id)
    }
}

impl Clone for Css {
    fn clone(&self) -> Self {
        let css = self.css.clone();

        let new_id: usize = SHARED_MANGLER_COUNT.with(|smc| {
            let mut count = smc.as_ref().borrow_mut();
            *count = 1 + *count;
            *count
        });

        let mangler = format_mangler(&self.mangler, new_id);
        let (mangled_css, mangle_lut) = mangle_css_string(&css, &mangler );

        // create a new style item.
        let style: Element = create_style_element(&document());
        style.set_text_content(&mangled_css);

        Css {
            css,
            mangler: self.mangler.clone(),
            mangler_id: new_id,
            style,
            mangle_lut
        }
    }
}


impl Index<&str> for Css {
    type Output = String;

    fn index(&self, index: &str) -> &Self::Output {
        self.mangle_lut.get(index).expect("Css class or id name does not exist in this css sheet.")
    }
}

fn format_mangler(mangle: &str, id: usize) -> String {
    if id == 0 {
        format!("{}-", mangle, )
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
        let (new_css, mapping_lut) = mangle_css_string(css, mangle_str);
        let expected_css = ".mangle-class {lorem: ipsum}";

        assert_eq!(new_css, expected_css)
    }

    #[test]
    fn mangle_css_lut() {
        let css = ".class {lorem: ipsum}";
        let mangle_str = "mangle-";
        let (new_css, mapping_lut) = mangle_css_string(css, mangle_str);

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
