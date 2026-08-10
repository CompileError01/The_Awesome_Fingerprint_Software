use yew::prelude::*;

mod components;
use components::login_page::LoginPage;
use components::dashboard_page::DashboardPage;
use components::doors_page::DoorsPage;
use components::log_page::LogPage;
use components::users_page::UsersPage;

struct PanelUser {
    username: String,
    password: String,
}

#[derive(Clone, PartialEq)]
enum Pages {
    DashboardPage,
    UsersPage,
    DoorsPage,
    LogPage,
}

pub enum AuthState {
    Loading,
    LoggedOut,
    LoggedIn,
    
}

#[component]
fn App() -> Html {
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

    let auth_state = use_state(|| AuthState::LoggedIn);
    let logged_in: bool = false;
    
    html! {
        <>
        <hr/>
        {
            match *auth_state {
                AuthState::LoggedOut => html! { <LoginPage/> },
                AuthState::Loading => html! { <LoginPage/> },
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
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
