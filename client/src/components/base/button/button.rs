use yew::prelude::*;
use yew_icons::Icon;

use web_sys::MouseEvent;

use crate::components::base::button::style::{button_container, button_text};

#[derive(Properties, Clone, PartialEq)]
pub struct ButtonProps {
    pub title: Option<String>,
    pub label: Option<String>,
    pub onclick: Callback<MouseEvent>,
    pub class: Option<String>,
    pub children: Option<ChildrenWithProps<Icon>>,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    // Styles
    let button_container = button_container();
    let button_text = button_text();
    let combined_style = format!(
        "{} {} {}",
        button_container,
        button_text,
        props.class.as_deref().unwrap_or("")
    );
    // Utils
    let onclick = props.onclick.clone();

    html! {
        <button
            class={ combined_style }
            onclick={ onclick }>
            {
                if let Some(ref children) = props.children {
                    html! { <div class="children-container"> { for children.iter() } </div> }
                } else {
                    html! {}
                }
            }
            {
                if let Some(ref label) = props.label {
                    html! { <span class="label-container">{ label }</span> }
                } else {
                    html! {}
                }
            }
        </button>
    }
}
