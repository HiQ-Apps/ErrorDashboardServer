use gloo_net::http::Method;
use serde_valid::Validate;
use yew::prelude::*;

use crate::hooks::use_mutation::use_mutation;
use shared_types::user_dtos::{UserLoginDTO, UserLoginResponseDTO};

#[function_component(LoginForm)]
fn login_form() -> Html {
    let email = use_state(|| "".to_string());
    let password = use_state(|| "".to_string());
    let validation_error = use_state(|| None);

    let (loading, error, login_handler) = use_mutation::<UserLoginResponseDTO, UserLoginDTO>(Method::POST);

    let onsubmit = {
        let email = email.clone();
        let password = password.clone();
        let login_handler = login_handler.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let request = UserLoginDTO {
                email: (*email).clone(),
                password: (*password).clone(),
            };
            
            match request.validate() {
                Ok(_) => {
                    validation_error.set(None);
                    login_handler("/auth/login".to_string(), Some(request));
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
                        value={(*email).clone()}
                        oninput={Callback::from(move |e: InputEvent| email.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
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
                <button type="submit" disabled={**loading}>{"Login"}</button>
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