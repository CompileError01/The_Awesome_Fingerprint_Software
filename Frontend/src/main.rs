use std::thread::spawn;

use yew::platform::spawn_local;
use yew::prelude::*;

mod components;
use components::login_page::LoginPage;
use components::dashboard_page::DashboardPage;
use components::doors_page::DoorsPage;
use components::log_page::LogPage;
use components::users_page::UsersPage;

use reqwest::StatusCode;

use gloo_net::http::Request;
use web_sys::RequestCredentials;

#[derive(Clone, PartialEq)]
enum Pages {
    DashboardPage,
    UsersPage,
    DoorsPage,
    LogPage,
}

#[derive(PartialEq)]
pub enum AuthState {
    LoggedOut,
    LoggedIn,
    
}



#[component]
fn App() -> Html {
    let auth_state = use_state(|| AuthState::LoggedOut);

    let mut backend_auth_state: u16 = 401;

    let value = auth_state.clone();
    spawn_local(async move {
        backend_auth_state = get_auth_state().await; 

        if backend_auth_state == 200 {
            value.set(AuthState::LoggedIn);
        }
    });

    let current_page = use_state(|| Pages::DashboardPage);

    let dashboard_page = {
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| current_page.set(Pages::DashboardPage))
    };
    let users_page = {
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| current_page.set(Pages::UsersPage))
    };
    let doors_page = {
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| current_page.set(Pages::DoorsPage))
    };
    let log_page = {
        let current_page = current_page.clone();
        Callback::from(move |_: MouseEvent| current_page.set(Pages::LogPage))
    };
    
    html! {
        <ContextProvider<UseStateHandle<AuthState>> context={auth_state.clone()}>
        <>
        <hr/>
        {
            match *auth_state {
                AuthState::LoggedOut => html! { <LoginPage/> },
                AuthState::LoggedIn => html! {
                    <>
                    <div>
                        <button onclick={dashboard_page}>{ "Dashboard" }</button>
                        <button onclick={users_page}>{ "Users" }</button>
                        <button onclick={doors_page}>{ "Doors" }</button>
                        <button onclick={log_page}>{ "Logs" }</button>
                    </div>
                    
                    <hr/>
                    {
                        match *current_page {
                            Pages::DashboardPage => html! { <DashboardPage/> },
                            Pages::UsersPage => html! { <UsersPage/> },
                            Pages::DoorsPage => html! { <DoorsPage/> },
                            Pages::LogPage => html! { <LogPage/> },
                        }
                    }
                    </>
                    
                }
            }
        }
    
        </>
        </ContextProvider<UseStateHandle<AuthState>>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

async fn get_auth_state() -> u16 {
    let response = Request::get("https://127.0.0.1:8000/get-auth-state")
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await
            .unwrap();
    
        response.status()
}
