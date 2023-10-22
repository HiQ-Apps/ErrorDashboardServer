use gloo_net::http::{Headers, RequestBuilder};
use serde::de::DeserializeOwned;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[hook]
pub fn use_query<T>(
    url: &str,
) -> (
    UseStateHandle<Rc<T>>,
    UseStateHandle<Rc<bool>>,
    UseStateHandle<Rc<Option<String>>>,
)
where
    T: 'static + DeserializeOwned + Default,
{
    let data_handler = use_state(|| Rc::new(T::default()));
    let loading_handler = use_state(|| Rc::new(true));
    let error_handler = use_state(|| Rc::new(None));

    let url_rc = Rc::new(url.to_string());

    let _ = use_effect_with_deps(
        {
            let url_rc = url_rc.clone();
            let data_handler = data_handler.clone();
            let loading_handler = loading_handler.clone();
            let error_handler = error_handler.clone();

            move |_| {
                let url = url_rc.clone();
                spawn_local(async move {
                    loading_handler.set(Rc::new(true));
                    let headers = Headers::new();
                    headers.set("Content-Type", "application/json");
                    match RequestBuilder::new(&url).headers(headers).send().await {
                        Ok(response) => match response.json::<T>().await {
                            Ok(data) => {
                                data_handler.set(Rc::new(data));
                            }
                            Err(err) => {
                                error_handler.set(Rc::new(Some(format!("Error: {:?}", err))));
                            }
                        },
                        Err(err) => {
                            error_handler.set(Rc::new(Some(format!("Network Error: {:?}", err))));
                        }
                    };
                    loading_handler.set(Rc::new(false));
                });

                || {}
            }
        },
        vec![url_rc],
    );

    (data_handler, loading_handler, error_handler)
}
