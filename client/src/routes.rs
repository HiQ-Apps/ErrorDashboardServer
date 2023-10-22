use yew::prelude::*;
use yew_router::prelude::*;

use crate::views::dashboard::DashboardPage;
use crate::views::home::HomePage;
use crate::views::login::LoginPage;
use crate::views::register::RegistrationPage;

#[derive(Routable, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[at("/")]
    HomePage,
    #[at("/login")]
    LoginPage,
    #[at("/register")]
    RegistrationPage,
    #[at("/dashboard")]
    DashboardPage,
}

pub fn switch(routes: AppRoute) -> Html {
    match routes {
        AppRoute::HomePage => html! {<HomePage/>},
        AppRoute::LoginPage => html! {<LoginPage/>},
        AppRoute::RegistrationPage => html! {<RegistrationPage/>},
        AppRoute::DashboardPage => html! {<DashboardPage/>},
    }
}
