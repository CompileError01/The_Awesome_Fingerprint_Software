use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;

#[function_component(DashboardPage)]
pub fn dashboard_page() -> Html {
    let data = use_state(|| "Loading...".to_string());
    
        {
            let data = data.clone();
    
            use_effect_with((), move |_| {
                spawn_local(async move {
                    let text = fetch_data().await;
                    data.set(text);
                });
    
                || ()
            });
        }
    
        html! {
            <>
                <h1>{"Dashboard"}</h1>
                <p>{(*data).clone()}</p>
            </>
        }
}

async fn fetch_data() -> String {
    let client = reqwest::Client::new();

    client
        .get("http://127.0.0.1:8000/users-data")
        .header("x-api-key", "valid_api_key")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}