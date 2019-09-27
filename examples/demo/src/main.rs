use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew_css::{Css, CssService};

struct Model {
    css: Option<Css>,
    other_css: Css
}

enum Msg {
    DropCss,
    ChangeRed,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let css = CssService::with_mangler("lorem".to_string()).attach_css("body { background-color: blue }");
        let other_css = CssService::with_mangler("ipsum".to_string()).attach_css(".class {background-color: cyan}");
        Model { css: Some(css), other_css }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DropCss => {
                self.css = None;
                true
            }
            Msg::ChangeRed => {
                if let Some(css) = &mut self.css {
                    css.overwrite_css("body { background-color: red } ".to_string())
                }
                true
            }
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {

        html! {
            <>
                <button class=&self.other_css["class"] onclick=|_| Msg::DropCss>{ "Drop Css!" }</button>
                <button onclick=|_| Msg::ChangeRed>{ "red!" }</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
