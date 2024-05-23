use serde_valid::Validate;
use gloo_net::http::Method;
use yew::prelude::*;

use shared_types::namespace_dtos::{CreateNamespaceDto, NamespaceDto};
use crate::hooks::use_mutation::use_mutation;

#[function_component(CreateNamespaceForm)]
fn create_namespace_form() -> Html {
    let service_name = use_state(|| "".to_string());
    let environment_type = use_state(|| "".to_string());
    let validation_error = use_state(|| None);

    let (loading, error, namespace_handler) = use_mutation::<NamespaceDto, CreateNamespaceDto>(Method::POST);

    let onsubmit = {
        let service_name = service_name.clone();
        let environment_type = environment_type.clone();

        let namespace_handler = namespace_handler.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let request = CreateNamespaceDto {
                service_name: (*service_name).clone(),
                environment_type: (*environment_type).clone(),
            };

            match request.validate() {
                Ok(_) => {
                    validation_error.set(None);
                    namespace_handler("/api/namespace".to_string(), Some(request));
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
                    <label for="service_name">{"Service Name:"}</label>
                    <input
                        id="service_name"
                        type="text"
                        value={(*service_name).clone()}
                        oninput={Callback::from(move |e: InputEvent| service_name.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <div>
                    <label for="environment_type">{"Environment Type:"}</label>
                    <input
                        id="environment_type"
                        type="text"
                        value={(*environment_type).clone()}
                        oninput={Callback::from(move |e: InputEvent| environment_type.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value()))}
                    />
                </div>
                <button type="submit" disabled={**loading}>{"Create Namespace"}</button>
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