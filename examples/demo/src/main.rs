use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew_css::{css_file, Css, CssService};

struct Model {
    droppable_css: Option<Css>,
    css: Css,
}

enum Msg {
    DropCss,
    ChangeRed,
}

std::thread_local! {
    pub static GLOBAL_CSS: Css = css_file!("global", "../assets/styles.css");
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let droppable_css = CssService::with_mangler("lorem".to_string())
            .attach_css("body { background-color: blue }");

        let css = CssService::with_mangler("ipsum".to_string())
            .attach_css(".class {background-color: cyan}");
        Model {
            droppable_css: Some(droppable_css),
            css,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DropCss => {
                self.droppable_css = None;
                true
            }
            Msg::ChangeRed => {
                if let Some(css) = &mut self.droppable_css {
                    css.overwrite_css("body { background-color: red } ".to_string())
                }
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        GLOBAL_CSS.with(|css| {
            return html! {
                <>
                    <button class=&self.css["class"] onclick=|_| Msg::DropCss>{ "Drop Css!" }</button>
                    <button class=&css["fancy"] onclick=|_| Msg::ChangeRed>{ "red!" }</button>
                </>
            };
        })
    }
}

fn main() {
    yew::start_app::<Model>();
}
