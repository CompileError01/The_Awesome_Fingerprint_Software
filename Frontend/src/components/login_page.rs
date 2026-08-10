use yew::prelude::*;
use yew::Callback;
use web_sys::{HtmlInputElement};
use wasm_bindgen_futures::spawn_local;



#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let username_input_ref = use_node_ref();
    let password_input_ref = use_node_ref();

    let login_button_event = {
        let username_input_ref = username_input_ref.clone();
        let password_input_ref = password_input_ref.clone();

        Callback::from(move |_| {
            
            let mut username: String = String::new();
            let mut password: String = String::new();
    
            if let Some(input) = &username_input_ref.cast::<HtmlInputElement>() {
                username = input.value();
            }
            if let Some(input) = &password_input_ref.cast::<HtmlInputElement>() {
                password = input.value();
            }
    
            spawn_local(async move {
            verify_login(username, password).await;
            });

        })
    };

    html! {
        <>
        
        <h1>{"Login"}</h1>

        <input type="text" ref={&username_input_ref} required=true placeholder="Username"/>
        <input type="password" ref={&password_input_ref} required=true placeholder="Password"/>

        <button onclick={login_button_event}>{"Login"}</button>

        </>
    }
}

async fn verify_login(username: String, password: String) {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("http://127.0.0.1:8000/validate-login/"))
        .header("x-api-key", "valid_api_key")
        .header("Content-Type", "application/json")
        .body(format!(r#"{{
            "username": {username},
            "password": {password}
            }}
            "#))
        .send()
        .await
        .unwrap();
    
    panic!("Yoo this is the big status code: {}", response.status());
}