use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::composite::navbar::style::{navbar_container, navbar_logo_container, menu_container, navlinks};
use crate::components::composite::navbar::subcomponents::navbutton::{
    dashboard_button, home_button, login_button, logout_button, menu_button, registration_button,
};
use crate::routes::AppRoute;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    // Hooks
    let navigator = use_navigator().unwrap();
    let dropdown_handler = use_state(|| false);

    // Styles
    let navbar_container = navbar_container();
    let navbar_logo_container = navbar_logo_container();
    let menu_container = menu_container();
    let navlinks = navlinks();

    // Utils
    let toggle_dropdown = {
        let dropdown_visible_setter = dropdown_handler.setter().clone();
        let dropdown_handler_for_toggle = dropdown_handler.clone();
    
        Callback::from(move |_: MouseEvent| {
            let current_state = *dropdown_handler_for_toggle;
            dropdown_visible_setter.set(!current_state);
        })
    };
    
    let home_onclick_redirect = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&AppRoute::HomePage))
    };

    let login_onclick = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&AppRoute::LoginPage))
    };

    let registration_onclick = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&AppRoute::RegistrationPage))
    };

    let dashboard_onclick = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&AppRoute::DashboardPage))
    };

    // Components
    let home_button = home_button(*dropdown_handler, home_onclick_redirect.clone());
    let menu_button = menu_button(toggle_dropdown.clone());
    let login_button = login_button(*dropdown_handler, login_onclick.clone());
    let logout_button = logout_button(*dropdown_handler, home_onclick_redirect.clone());
    let registration_button = registration_button(*dropdown_handler, registration_onclick.clone());
    let dashboard_button = dashboard_button(*dropdown_handler, dashboard_onclick.clone());

    html! {
        <div class={navbar_container}>
            <div class={navbar_logo_container} onclick={home_onclick_redirect.clone()}>
                <img src={"./images/error_icon.png"} />
                <h1>{"Error Dashboard"}</h1>
            </div>
            <div class={menu_container}>
                {menu_button}
            </div>
            <ul class={navlinks}>
                {home_button}
                {login_button}
                {logout_button}
                {registration_button}
                {dashboard_button}
            </ul>
        </div>
    }
}
