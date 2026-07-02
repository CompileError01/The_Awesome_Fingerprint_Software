use yew::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use wasm_bindgen_futures::spawn_local;

#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {

    let data = use_state(|| None::<Value>);

    {
        let data = data.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = fetch_data().await;
                data.set(Some(response));
            });

            || ()
        });
    }
    
    html! {
        <>
                    <h1>{"Dashboard"}</h1>
        
                    {
                        if let Some(response) = &*data {
                            html! {
                                <pre>{format!("{:#}", response)}</pre>
                            }
                        } else {
                            html! {
                                <p>{"Loading..."}</p>
                            }
                        }
                    }
                </>
    }
}

async fn fetch_data() -> Value{
    let api_key = "valid_api_key";
        let url = "http://127.0.0.1:8000/users-data"; 
    
        // Create a header map and insert the API key
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("x-api-key{}", api_key)).unwrap());
    
        // Send the GET request
        reqwest::Client::new()
            .get(url)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()
}