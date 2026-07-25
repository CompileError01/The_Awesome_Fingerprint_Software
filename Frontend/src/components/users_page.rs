use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;

// yes I know that this struct is a duplicate, and has to be updated twice,
// please leave a complaint at goFrickYourself@gmail.com
// also has to be updated in the backend
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    id: u32,
    group: String,
    firstName: String,
    lastName: String,
    email: String,
}

#[function_component(UsersPage)]
pub fn users_page() -> Html {
    // Holds all users returned by the API
    let users = use_state(Vec::<User>::new);

    {
        let users = users.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let data = fetch_data().await;
                users.set(data);
            });

            || ()
        });
    }

    html! {
        <>
            <div>
                <h1>{"Users"}</h1>
                <button>{"add user"}</button>
            </div>
            
            if users.is_empty() {
                <p>{"Loading..."}</p>
            } else {
                
                <p>{format!("Loaded {} users at {}", users.len(), Local::now())}</p>
            }

            <table border="1" width="80%">
                <thead>
                    <tr>
                        <th>{"ID"}</th>
                        <th>{"First-Name"}</th>
                        <th>{"Last-Name"}</th>
                        <th>{"Email"}</th>
                        <th>{"Group"}</th>
                    </tr>
                </thead>

                <tbody>
                    {
                        users.iter().map(|user| {
                            html! {
                                <tr>
                                    <td>{&user.id}</td>
                                    <td>{&user.firstName}</td>
                                    <td>{&user.lastName}</td>
                                    <td>{&user.email}</td>
                                    <td>{&user.group}</td>
                                </tr>
                            }
                        }).collect::<Html>()
                    }
                </tbody>
            </table>
        </>
    }
}


async fn fetch_data() -> Vec<User> {
    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8000/users-data")
        .header("x-api-key", "valid_api_key")
        .send()
        .await
        .unwrap();

    println!("Status: {}", response.status());

    response.json::<Vec<User>>().await.unwrap()
}