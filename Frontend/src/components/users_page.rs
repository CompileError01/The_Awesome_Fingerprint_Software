use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::Callback;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use web_sys::{HtmlInputElement};
use web_sys::window;

// yes I know that this struct is a duplicate, and has to be updated twice,
// please leave a complaint at goFrickYourself@gmail.com
// also has to be UPDATED in the BACKEND
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
    let mut users = use_state(Vec::<User>::new);

    // gets all the user data from the api and sorts it by id
    {
        let users = users.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let mut data = fetch_data().await; 

                data.sort_by_key(|user| user.id);
                
                users.set(data);
            });

            || ()
        });
    }


    
    // For adding users
    let latest_id: u32 = find_highest_id(&users);

    let show_add_panel = use_state(|| false); 

    let first_name_input_ref = use_node_ref();
    let last_name_input_ref = use_node_ref();
    let email_input_ref = use_node_ref();
    let group_input_ref = use_node_ref();

    let first_name_input_ref_cb = first_name_input_ref.clone();
    let last_name_input_ref_cb = last_name_input_ref.clone();
    let email_input_ref_cb = email_input_ref.clone();
    let group_input_ref_cb = group_input_ref.clone();
    
    let open_add_panel =  {
        let show_add_panel = show_add_panel.clone();
        Callback::from(move |_| show_add_panel.set(true))
    };

    let close_add_panel = {
        let show_add_panel = show_add_panel.clone();
        Callback::from(move |_| show_add_panel.set(false))
    };
    

    // For editing users
    let show_edit_panel = use_state(|| false);

    let mut id_from_table = use_state(|| 0u32);

    let open_edit_panel = {
        let show_edit_panel = show_edit_panel.clone();
        let id_from_table = id_from_table.clone();

        Callback::from(move |id: u32| {
            id_from_table.set(id);
            show_edit_panel.set(true)
        })
    };

    let close_edit_panel = {
        let show_edit_panel = show_edit_panel.clone();
        Callback::from(move |_| show_edit_panel.set(false))
    };

    let first_name_input_ref_cb_clone = first_name_input_ref_cb.clone();
    let last_name_input_ref_cb_clone = last_name_input_ref_cb.clone();
    let email_input_ref_cb_clone = email_input_ref_cb.clone();
    let group_input_ref_cb_clone = group_input_ref_cb.clone(); 
    
    let add_user_button_event = {
        let show_add_panel = show_add_panel.clone();
        
        Callback::from(move |_| {
            
            let first_name_input_ref = first_name_input_ref_cb.clone();
            let last_name_input_ref = last_name_input_ref_cb.clone();
            let email_input_ref = email_input_ref_cb.clone();
            let group_input_ref = group_input_ref_cb.clone();
    
            let id = latest_id + 1;
            
            let mut first_name: String = String::new();
            let mut last_name: String = String::new();
            let mut email: String = String::new();
            let mut group: String = String::new();
    
            if let Some(input) = &first_name_input_ref.cast::<HtmlInputElement>() {
                first_name = input.value();
            }
            if let Some(input) = last_name_input_ref.cast::<HtmlInputElement>() {
                last_name = input.value();
            }
            if let Some(input) = email_input_ref.cast::<HtmlInputElement>() {
                email = input.value();
            }
            if let Some(input) = group_input_ref.cast::<HtmlInputElement>() {
                group = input.value();
            }

            if first_name != String::new() 
                && last_name != String::new()
                && email != String::new()
                && group != String::new()
            {
                    
                spawn_local(async move {
                    add_user(id, group, first_name, last_name, email).await;
                });
    
                show_add_panel.set(false)
            }
            else {
                window().unwrap().alert_with_message("An invalid input has been given!").unwrap();
            }
        })
        
    };

    let delete_user_button_event = {
        Callback::from(move |id: u32| {
            let confirmed = window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this user?")
                .unwrap();

            if confirmed {
                spawn_local(async move {
                    delete_user(id).await;
                });
            }
        })
    };

    let edit_user_button_event = {
        let show_edit_panel = show_edit_panel.clone();
        let id = *id_from_table;

        Callback::from(move |_| {
            let first_name_input_ref = first_name_input_ref_cb_clone.clone();
            let last_name_input_ref = last_name_input_ref_cb_clone.clone();
            let email_input_ref = email_input_ref_cb_clone.clone();
            let group_input_ref = group_input_ref_cb_clone.clone();
    
            let mut first_name: String = String::new();
            let mut last_name: String = String::new();
            let mut email: String = String::new();
            let mut group: String = String::new();
    
            if let Some(input) = &first_name_input_ref.cast::<HtmlInputElement>() {
                first_name = input.value();
            }
            if let Some(input) = last_name_input_ref.cast::<HtmlInputElement>() {
                last_name = input.value();
            }
            if let Some(input) = email_input_ref.cast::<HtmlInputElement>() {
                email = input.value();
            }
            if let Some(input) = group_input_ref.cast::<HtmlInputElement>() {
                group = input.value();
            }

            spawn_local(async move {
                edit_user(id, group, first_name, last_name, email).await;
            });

            show_edit_panel.set(false);

        })
    };

    let user = get_single_user_data(*id_from_table, &users); // get user data for editing panel
    
    html! {
        <>
            if *show_add_panel {
                <div class="modal-backdrop" onclick={close_add_panel.clone()}>
                    <div
                        class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <h2>{ "Add a user" }</h2>

                        <div>
                            <input type="text" ref={&first_name_input_ref} required=true placeholder="First Name"/>
                            <input type="text" ref={&last_name_input_ref} required=true placeholder="Last Name"/>
                            <input type="email" ref={&email_input_ref} required=true placeholder="Email"/>
                            <input type="text" ref={&group_input_ref} required=true placeholder="Group"/>
                        </div>
                        
                        <div>
                            <button onclick={add_user_button_event}>{"Add"}</button>
                            <button onclick={close_add_panel}>{"Cancel"}</button>
                        </div>

                    </div>
                </div>
            }

            else if *show_edit_panel {
                
                
                <div class="modal-backdrop" onclick={close_edit_panel.clone()}>
                    <div
                        class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <h2>{ "Edit User" }</h2>

                        <div>
                            <input type="text" ref={&first_name_input_ref} required=true 
                                value={user.as_ref().unwrap().firstName.clone()} placeholder="First Name"/>

                            <input type="text" ref={&last_name_input_ref} required=true
                                value={user.as_ref().unwrap().lastName.clone()} placeholder="Last Name"/>
                        
                            <input type="email" ref={&email_input_ref} required=true
                                value={user.as_ref().unwrap().email.clone()} placeholder="Email"/>
                        
                            <input type="text" ref={&group_input_ref} required=true
                                value={user.as_ref().unwrap().group.clone()} placeholder="Group"/>
                        </div>
                        
                        <div>
                            <button onclick={edit_user_button_event}>{"Apply"}</button>
                            <button onclick={close_edit_panel}>{"Cancel"}</button>
                        </div>

                    </div>
                </div>
            }
            
            
            else {
    
                <h1>{"Users"}</h1>
                
                if users.is_empty() {
                    <p>{"Loading..."}</p>
                } else {
                    
                    <p>{format!("Loaded {} users at {}", users.len(), Local::now())}</p>
                    <button onclick={open_add_panel} >{"add user"}</button> 
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
                                let delete_user_button_event = delete_user_button_event.clone();
                                let id = user.id;
                                let open_edit_panel = open_edit_panel.clone();
                                
                                html! {
                                    <tr>
                                        <td>{&user.id}</td>
                                        <td>{&user.firstName}</td>
                                        <td>{&user.lastName}</td>
                                        <td>{&user.email}</td>
                                        <td>{&user.group}</td>
                                        <td> 
                                            <button onclick={                                                
                                                Callback::from(move |_: MouseEvent| {
                                                    open_edit_panel.emit(id);
                                                })
                                            }>{"edit"}</button>
                                            
                                            <button onclick={
                                                Callback::from(move |_: MouseEvent| {
                                                    delete_user_button_event.emit(id);
                                                })   
                                            }> {"delete"}</button> 
                                        </td>
                                    </tr>
                                }
                            }).collect::<Html>()
                        }
                    </tbody>
                </table>

            }
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

async fn add_user(id: u32, group: String, first_name: String, last_name: String, email: String) {
    let client = reqwest::Client::new();
    
    client
        .put(format!("http://127.0.0.1:8000/add-user/{id}"))
        .header("x-api-key", "valid_api_key")
        .header("Content-Type", "application/json")
        .body(format!(r#"{{
            "id": {id}, 
            "group": "{group}", 
            "firstName": "{first_name}", 
            "lastName": "{last_name}", 
            "email": "{email}"
            }}
            "#))
        .send()
        .await
        .unwrap();
}

async fn edit_user(id: u32, group: String, first_name: String, last_name: String, email: String) {
    let client = reqwest::Client::new();
    
    client
        .post(format!("http://127.0.0.1:8000/edit-user/{id}"))
        .header("x-api-key", "valid_api_key")
        .header("Content-Type", "application/json")
        .body(format!(r#"{{
            "id": {id}, 
            "group": "{group}", 
            "firstName": "{first_name}", 
            "lastName": "{last_name}", 
            "email": "{email}"
            }}
            "#))
        .send()
        .await
        .unwrap();
}

async fn delete_user(id: u32) {
    let client = reqwest::Client::new();
    
    client
        .delete(format!("http://127.0.0.1:8000/delete-user/{id}"))
        .header("x-api-key", "valid_api_key")
        .send()
        .await
        .unwrap();
}

fn get_single_user_data(id: u32, users: &Vec<User>) -> Option<User> {
    for user in users {
        if user.id == id {
            return Some(user.clone());
        }
    }
    None
     // I know that this is very stupid
     // i know i have to implement error handling for this, but I do not have the patience to do it right now
}

fn find_highest_id(users: &Vec<User>) -> u32 {
    let mut highest_id: u32 = 0;

    for user in users {
        if user.id > highest_id { highest_id = user.id }
    }

    highest_id
}
