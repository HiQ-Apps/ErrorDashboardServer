use std::collections::HashMap;
use std::convert::From;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;
use yew::Callback;

#[hook]
pub fn use_form<T>(
    default_values: HashMap<String, T>,
) -> (UseStateHandle<HashMap<String, T>>, Callback<Event>)
where
    T: 'static + Clone + PartialEq + From<String>,
{
    let form_handle: UseStateHandle<HashMap<String, T>> = use_state(|| default_values.clone());

    let handle_change: Callback<Event> = Callback::from({
        let form: UseStateHandle<HashMap<String, T>> = form_handle.clone();
        move |e: Event| {
            if let Some(target) = e.target() {
                let input: Result<HtmlInputElement, _> = target.dyn_into();
                if let Ok(input) = input {
                    let field_name: String = input.name();
                    let value: String = input.value();

                    let mut current_state: HashMap<String, T> = (*form).clone();

                    current_state.insert(field_name, T::from(value));
                    form.set(current_state);
                }
            }
        }
    });

    (form_handle, handle_change)
}
