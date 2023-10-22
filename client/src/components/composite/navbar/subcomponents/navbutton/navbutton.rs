use crate::components::base::button::Button;
use yew::prelude::*;
use yew_icons::{Icon, IconId};
use web_sys::MouseEvent;
use super::style::{menubutton, navbutton, dropdown_navbutton};

pub fn get_navbutton_style(is_dropdown: bool) -> String {
    let dropdown_navbutton = dropdown_navbutton();
    let navbutton = navbutton();
    if is_dropdown {
        dropdown_navbutton
    } else {
        navbutton
    }
}

pub fn menu_button(toggle_dropdown: Callback<MouseEvent>) -> Html {
    let menubutton_style = menubutton();
    html! {
        <Button title={"Menu"} class={menubutton_style} onclick={toggle_dropdown.clone()}>
            <Icon icon_id={IconId::LucideMenu}/>
        </Button>
    }
}

pub fn home_button(is_dropdown: bool, home_onclick_redirect: Callback<MouseEvent>) -> Html {
    let style = get_navbutton_style(is_dropdown);
    html! {
        <Button title={"Home"} class={style} onclick={home_onclick_redirect.clone()}>
            <Icon icon_id={IconId::HeroiconsSolidHome}/>
        </Button>
    }
}

pub fn login_button(is_dropdown: bool, login_onclick: Callback<MouseEvent>) -> Html {
    let style = get_navbutton_style(is_dropdown);
    html! {
        <Button title={"Login"} class={style} onclick={login_onclick.clone()}>
            <Icon icon_id={IconId::BootstrapDoorOpenFill}/>
        </Button>
    }
}

pub fn logout_button(is_dropdown: bool, home_onclick: Callback<MouseEvent>) -> Html {
    let style = get_navbutton_style(is_dropdown);
    html! {
        <Button title={"Logout"} class={style} onclick={home_onclick.clone()} >
            <Icon icon_id={IconId::BootstrapDoorClosedFill}/>
        </Button>
    }
}

pub fn registration_button(is_dropdown: bool, registration_onclick: Callback<MouseEvent>) -> Html {
    let style = get_navbutton_style(is_dropdown);
    html! {
        <Button title={"Register"} class={style} onclick={registration_onclick.clone()}>
            <Icon icon_id={IconId::HeroiconsSolidClipboardDocumentCheck}/>
        </Button>
    }
}

pub fn dashboard_button(is_dropdown: bool, dashboard_onclick: Callback<MouseEvent>) -> Html {
    let style = get_navbutton_style(is_dropdown);
    html! {
        <Button title={"Dashboard"} class={style} onclick={dashboard_onclick.clone()}>
            <Icon icon_id={IconId::LucideLayoutDashboard}/>
        </Button>
    }
}
