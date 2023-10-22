use crate::components::composite::navbar::Navbar;
use crate::routes::{switch, AppRoute};
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::router::BrowserRouter;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div>
            <BrowserRouter>
                <Navbar/>
                <Switch<AppRoute> render={switch} />
            </BrowserRouter>
        </div>
    }
}
