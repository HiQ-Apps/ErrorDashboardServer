use serde_valid::Validate;
use gloo_net::http::Method;
use yew::prelude::*;

use shared_types::user_dtos::{ShortUserDTO, UserCreateDTO};
use crate::hooks::use_mutation::use_mutation;

#[function_component(Registration)]
fn registration() -> Html {
    let username = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let email = use_state(|| "".to_string());
    let validation_error = use_state(|| None);

    let (loading, error, registration_handler) = use_mutation::<ShortUserDTO, UserCreateDTO>(Method::POST);

    let onsubmit = {
        let username = username.clone();
        let password = password.clone();
        let email = email.clone();
        let registration_handler = registration_handler.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let request = UserCreateDTO {
                username: (*username).clone(),
                password: (*password).clone(),
                email: (*email).clone(),
            };

            match request.validate() {
                Ok(_) => {
                    validation_error.set(None);
                    registration_handler("/api/register".to_string(), Some(request));
                },
                Err(errors) => {
                    validation_error.set(Some(errors));
                },
            }
        })
    };

    html! {
        <div>
            <form onsubmit={onsubmit}>
                <div>
                    <label for="username">{"Username:"}</label>
                    <input
                        id="username"
                        type="text"
                        value={(*username).clone()}
                        oninput={Callback::from(move |e: InputEvent| username.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <div>
                    <label for="password">{"Password:"}</label>
                    <input
                        id="password"
                        type="password"
                        value={(*password).clone()}
                        oninput={Callback::from(move |e: InputEvent| password.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <div>
                    <label for="email">{"Email:"}</label>
                    <input
                        id="email"
                        type="email"
                        value={(*email).clone()}
                        oninput={Callback::from(move |e: InputEvent| email.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <button type="submit" disabled={**loading}>{"Register"}</button>
            </form>
            if **loading {
                <p>{"Loading..."}</p>
            }
            if let Some(error) = &**error {
                <p>{format!("Error: {}", error)}</p>
            }
        </div>
    }
}