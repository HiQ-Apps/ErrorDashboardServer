mod app;
mod components {
    pub mod composite {
        pub mod navbar;
    }
    pub mod base {
        pub mod button;
    }
}
mod globals;
mod hooks;
mod routes;
mod views;

use crate::app::App;

pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
