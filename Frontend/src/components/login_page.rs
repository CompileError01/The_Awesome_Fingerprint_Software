use reqwest::StatusCode;
use yew::prelude::*;
use yew::Callback;
use web_sys::{HtmlInputElement};
use wasm_bindgen_futures::spawn_local;

use gloo_net::http::Request;
use web_sys::RequestCredentials;

// hashing
use argon2::{Argon2, PasswordHash, PasswordHasher, password_hash::Salt};

use crate::AuthState;

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    
    let auth_state = use_context::<UseStateHandle<AuthState>>()
            .expect("AuthState context not found");
    
    let username_input_ref = use_node_ref();
    let password_input_ref = use_node_ref();

    let login_button_event = {
        let auth_state = auth_state.clone();
        
        let username_input_ref = username_input_ref.clone();
        let password_input_ref = password_input_ref.clone();
        
        Callback::from(move |_| {

            let auth_state = auth_state.clone();
            
            let mut username: String = String::new();
            let mut password: String = String::new();

            let status_code: StatusCode = StatusCode::default(); 
            
            if let Some(input) = &username_input_ref.cast::<HtmlInputElement>() {
                username = input.value();
            }
            if let Some(input) = &password_input_ref.cast::<HtmlInputElement>() {
                password = input.value();
            }
    
            spawn_local(async move {
            let status_code = verify_login(username, password).await;

            if status_code ==  StatusCode::OK {
                auth_state.set(AuthState::LoggedIn);
            }
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

async fn verify_login(username: String, password: String) -> u16 {
    let hash: PasswordHash<'static> = hash_password(password);
   
    let response = Request::post("https://127.0.0.1:8000/verify-login")
            .credentials(web_sys::RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .body(format!(r#"{{
                "username": "{username}",
                "passwordHash": "{hash}"
                }}
                "#))
            .expect("wrong formatting!")
            .send()
            .await
            .unwrap();
    
        response.status()
}

fn hash_password(password: String) -> PasswordHash<'static> {
    let salt_str = "testtesttest"; // reminder for myself do not use bad salt
    let salt: Salt = salt_str.try_into().unwrap();
    
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), salt).unwrap()

}