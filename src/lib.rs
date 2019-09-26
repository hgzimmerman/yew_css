//! https://davidwalsh.name/add-rules-stylesheets
use stdweb::web::{Document, Element, INode, document};
use std::rc::Rc;
use std::cell::RefCell;
use std::thread_local;

thread_local! {
    /// Global counter used to keep css items distinct
    static SHARED_MANGLER_COUNT: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
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
            document: document()
        }
    }

    pub fn attach_css(&mut self, css: &str) -> Css {
        let style: Element = create_style_element(&self.document);

        let new_id: usize = SHARED_MANGLER_COUNT.with(|smc| {
            let mut count = smc.as_ref().borrow_mut();
            *count = 1 + *count;
            *count
        });

        let mangled_css = mangle_css(&self.mangler, new_id, css);
        style.set_text_content(&mangled_css);

        Css {
            css: String::from(css),
            mangler: self.mangler.clone(),
            mangler_id: new_id,
            shared_mangler_count: SHARED_MANGLER_COUNT.with(|smc| smc.clone()),
            style
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
    /// Shared counter of manglers with this name.
    shared_mangler_count: Rc<RefCell<usize>>,
    /// The stylesheet element in the DOM.
    style: Element
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

    /// Gets the mangled version of the class
    pub fn class(&self, class_name: &str) -> String {
        mangle_class(&self.mangler, self.mangler_id, class_name)
    }
    pub fn c(&self, class_name: &str) -> String {
        self.class(class_name)
    }
}


impl Clone for Css {
    fn clone(&self) -> Self {
        let mut count = self.shared_mangler_count.as_ref().borrow_mut();
        *count = 1 + *count;
        let new_id: usize = *count;
        let css = self.css.clone();

        // create a new style item.
        let style: Element = create_style_element(&document());

        let mangled_css = mangle_css(&self.mangler, new_id, &css);
        style.set_text_content(&mangled_css);

        Css {
            css,
            mangler: self.mangler.clone(),
            mangler_id: new_id,
            shared_mangler_count: self.shared_mangler_count.clone(),
            style
        }
    }
}

fn mangle_class(mangle: &str, id: usize, name: &str) -> String {
    if id == 0 {
        format!(".{}--{}", mangle, name)
    } else {
        format!(".{}__{}--{}", mangle, id, name)
    }
}

fn mangle_css(_mangle: &str, _id: usize, input: &str) -> String {
    // TODO implement me
    input.to_string()
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

