use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::Callback;
use yew::TargetCast;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use web_sys::{window, HtmlInputElement, HtmlSelectElement, HtmlDocument};

use wasm_bindgen::JsValue;

use gloo_net::http::Request;
use web_sys::RequestCredentials;

const DAY_NAMES: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

// Schedule for a single weekday. Lives inside DoorAccess::schedule,
// index 0 = Monday ... index 6 = Sunday.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DaySchedule {
    enabled: bool,
    start: String,
    end: String,
}

impl Default for DaySchedule {
    fn default() -> Self {
        DaySchedule {
            enabled: false,
            start: "09:00".to_string(),
            end: "17:00".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DoorAccess {
    door_id: u32,
    schedule: [DaySchedule; 7],
}

impl DoorAccess {
    fn new(door_id: u32) -> Self {
        DoorAccess {
            door_id,
            schedule: Default::default(),
        }
    }
}

// A trimmed-down view of a Door, just enough to list doors and reach
// their fingerprint reader. Kept separate from doors_page.rs's own
// Door struct (same duplication situation as User, see below).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DoorOption {
    id: u32,
    name: String,
    #[serde(default)]
    ip_addr: String,
}

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
    #[serde(default)]
    door_access: Vec<DoorAccess>,
    #[serde(default)]
    fingerprint: String,
}

#[function_component(UsersPage)]
pub fn users_page() -> Html {

    // Holds all users returned by the API
    let users = use_state(Vec::<User>::new);

    // gets all the user data from the api and sorts it by id
    {
        let users = users.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                refresh_users(users).await;
            });

            || ()
        });
    }

    // Holds all doors, so we can list them for door-access assignment
    // and for picking which door's reader to scan a fingerprint from.
    let doors = use_state(Vec::<DoorOption>::new);

    {
        let doors = doors.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let data = fetch_doors().await;
                doors.set(data);
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

    // Door access being built/edited in whichever panel is open, plus
    // the fingerprint captured for that panel. Both get reset/pre-filled
    // by open_add_panel / open_edit_panel below.
    let door_access_list = use_state(Vec::<DoorAccess>::new);
    let fingerprint_data = use_state(String::new);

    // For picking which door to add access for
    let new_door_select_ref = use_node_ref();

    // For picking which door's reader to scan a fingerprint from
    let fingerprint_door_select_ref = use_node_ref();

    let mut id_for_fingerprint_request: u32 = 0;

    let open_add_panel = {
        let show_add_panel = show_add_panel.clone();
        let door_access_list = door_access_list.clone();
        let fingerprint_data = fingerprint_data.clone();

        id_for_fingerprint_request.set = latest_id + 1;
        
        Callback::from(move |_| {
            door_access_list.set(Vec::new());
            fingerprint_data.set(String::new());
            show_add_panel.set(true);
        })
    };

    let close_add_panel = {
        let show_add_panel = show_add_panel.clone();
        Callback::from(move |_| show_add_panel.set(false))
    };


    // For editing users
    let show_edit_panel = use_state(|| false);

    let id_from_table = use_state(|| 0u32);

    let open_edit_panel = {
        let show_edit_panel = show_edit_panel.clone();
        let id_from_table = id_from_table.clone();
        let door_access_list = door_access_list.clone();
        let fingerprint_data = fingerprint_data.clone();
        let users = users.clone();

        Callback::from(move |id: u32| {
            id_from_table.set(id);

            if let Some(user) = users.iter().find(|user| user.id == id) {
                door_access_list.set(user.door_access.clone());
                fingerprint_data.set(user.fingerprint.clone());
            }

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

    // Adds a DoorAccess entry for whichever door is picked in the
    // "add door access" select.
    let add_door_access_event = {
        let door_access_list = door_access_list.clone();
        let new_door_select_ref = new_door_select_ref.clone();

        Callback::from(move |_| {
            if let Some(select) = new_door_select_ref.cast::<HtmlSelectElement>() {
                if let Ok(door_id) = select.value().parse::<u32>() {
                    let mut updated = (*door_access_list).clone();

                    if !updated.iter().any(|access| access.door_id == door_id) {
                        updated.push(DoorAccess::new(door_id));
                        door_access_list.set(updated);
                    }
                }
            }
        })
    };

    let add_user_button_event = {
        let show_add_panel = show_add_panel.clone();
        let door_access_list = door_access_list.clone();
        let fingerprint_data = fingerprint_data.clone();
        let users = users.clone();

        Callback::from(move |_| {

            let first_name_input_ref = first_name_input_ref_cb.clone();
            let last_name_input_ref = last_name_input_ref_cb.clone();
            let email_input_ref = email_input_ref_cb.clone();
            let group_input_ref = group_input_ref_cb.clone();
            let users = users.clone();

            let id = latest_id + 1;

            let mut first_name: String = String::new();
            let mut last_name: String = String::new();
            let mut email: String = String::new();
            let mut group: String = String::new();

            if let Some(input) = first_name_input_ref.cast::<HtmlInputElement>() {
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
                let door_access = (*door_access_list).clone();
                let fingerprint = (*fingerprint_data).clone();

                spawn_local(async move {
                    add_user(id, group, first_name, last_name, email, door_access, fingerprint).await;
                    refresh_users(users).await;
                });

                show_add_panel.set(false)
            }
            else {
                window().unwrap().alert_with_message("An invalid input has been given!").unwrap();
            }
        })

    };

    let delete_user_button_event = {
        let users = users.clone();

        Callback::from(move |id: u32| {
            let users = users.clone();

            let confirmed = window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this user?")
                .unwrap();

            if confirmed {
                spawn_local(async move {
                    delete_user(id).await;
                    refresh_users(users).await;
                });
            }
        })
    };

    let edit_user_button_event = {
        let show_edit_panel = show_edit_panel.clone();
        let id = *id_from_table;
        let door_access_list = door_access_list.clone();
        let fingerprint_data = fingerprint_data.clone();
        let users = users.clone();

        Callback::from(move |_| {
            let first_name_input_ref = first_name_input_ref_cb_clone.clone();
            let last_name_input_ref = last_name_input_ref_cb_clone.clone();
            let email_input_ref = email_input_ref_cb_clone.clone();
            let group_input_ref = group_input_ref_cb_clone.clone();
            let users = users.clone();

            let mut first_name: String = String::new();
            let mut last_name: String = String::new();
            let mut email: String = String::new();
            let mut group: String = String::new();

            if let Some(input) = first_name_input_ref.cast::<HtmlInputElement>() {
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

            let door_access = (*door_access_list).clone();
            let fingerprint = (*fingerprint_data).clone();

            spawn_local(async move {
                edit_user(id, group, first_name, last_name, email, door_access, fingerprint).await;
                refresh_users(users).await;
            });

            show_edit_panel.set(false);

        })
    };

    let get_new_fingerprint_button_event = {
        let fingerprint_data = fingerprint_data.clone();
        let fingerprint_door_select_ref = fingerprint_door_select_ref.clone();
        let doors = doors.clone();

        let ip = id_for

        Callback::from(move |_| {
            let fingerprint_data = fingerprint_data.clone();

            let door_ip = fingerprint_door_select_ref
                .cast::<HtmlSelectElement>()
                .and_then(|select| select.value().parse::<u32>().ok())
                .and_then(|door_id| {
                    doors.iter().find(|door| door.id == door_id).map(|door| door.ip_addr.clone())
                });

            match door_ip {
                Some(door_ip) => {
                    spawn_local(async move {
                        get_new_fingerprint(&door_ip, ip).await.unwrap();
                    });
                }
                None => {
                    window()
                        .unwrap()
                        .alert_with_message("Please select a door to scan from.")
                        .unwrap();
                }
            }
        })
    };

    // Doors that don't already have a DoorAccess entry, for the
    // "add door access" select.
    let available_doors: Vec<DoorOption> = doors
        .iter()
        .filter(|door| !door_access_list.iter().any(|access| access.door_id == door.id))
        .cloned()
        .collect();

    // The door-access editor: one block per door already assigned, each
    // with a Monday-Sunday enabled/start/end row, plus a picker to add
    // another door. Built once and reused in both the add and edit panels.
    let door_access_section: Html = html! {
        <div class="door-access-editor">
            <h3>{"Door access"}</h3>

            {
                door_access_list.iter().enumerate().map(|(index, access)| {
                    let door_name = doors.iter()
                        .find(|door| door.id == access.door_id)
                        .map(|door| door.name.clone())
                        .unwrap_or_else(|| format!("Door #{}", access.door_id));

                    let remove_event = {
                        let door_access_list = door_access_list.clone();
                        Callback::from(move |_| {
                            let mut updated = (*door_access_list).clone();
                            updated.remove(index);
                            door_access_list.set(updated);
                        })
                    };

                    html! {
                        <div class="door-access-entry">
                            <div>
                                <strong>{ door_name }</strong>
                                <button onclick={remove_event}>{"Remove"}</button>
                            </div>

                            <table>
                                <thead>
                                    <tr>
                                        <th>{"Day"}</th>
                                        <th>{"Allowed"}</th>
                                        <th>{"From"}</th>
                                        <th>{"Until"}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                {
                                    DAY_NAMES.iter().enumerate().map(|(day_index, day_name)| {
                                        let day = access.schedule[day_index].clone();

                                        let toggle_event = {
                                            let door_access_list = door_access_list.clone();
                                            Callback::from(move |_: Event| {
                                                let mut updated = (*door_access_list).clone();
                                                updated[index].schedule[day_index].enabled =
                                                    !updated[index].schedule[day_index].enabled;
                                                door_access_list.set(updated);
                                            })
                                        };

                                        let start_event = {
                                            let door_access_list = door_access_list.clone();
                                            Callback::from(move |e: InputEvent| {
                                                if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                                                    let mut updated = (*door_access_list).clone();
                                                    updated[index].schedule[day_index].start = input.value();
                                                    door_access_list.set(updated);
                                                }
                                            })
                                        };

                                        let end_event = {
                                            let door_access_list = door_access_list.clone();
                                            Callback::from(move |e: InputEvent| {
                                                if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                                                    let mut updated = (*door_access_list).clone();
                                                    updated[index].schedule[day_index].end = input.value();
                                                    door_access_list.set(updated);
                                                }
                                            })
                                        };

                                        html! {
                                            <tr>
                                                <td>{ *day_name }</td>
                                                <td>
                                                    <input
                                                        type="checkbox"
                                                        checked={day.enabled}
                                                        onchange={toggle_event}
                                                    />
                                                </td>
                                                <td>
                                                    <input
                                                        type="time"
                                                        value={day.start.clone()}
                                                        oninput={start_event}
                                                        disabled={!day.enabled}
                                                    />
                                                </td>
                                                <td>
                                                    <input
                                                        type="time"
                                                        value={day.end.clone()}
                                                        oninput={end_event}
                                                        disabled={!day.enabled}
                                                    />
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Html>()
                                }
                                </tbody>
                            </table>
                        </div>
                    }
                }).collect::<Html>()
            }

            {
                if available_doors.is_empty() {
                    if doors.is_empty() {
                        html! { <p>{"No doors have been added yet."}</p> }
                    } else {
                        html! { <p>{"Every door already has an access entry for this user."}</p> }
                    }
                } else {
                    html! {
                        <div>
                            <select ref={&new_door_select_ref}>
                                {
                                    available_doors.iter().map(|door| {
                                        html! {
                                            <option value={door.id.to_string()}>{ &door.name }</option>
                                        }
                                    }).collect::<Html>()
                                }
                            </select>
                            <button onclick={add_door_access_event}>{"Add door access"}</button>
                        </div>
                    }
                }
            }
        </div>
    };

    // The fingerprint scan block: pick a door, scan, see whether a
    // fingerprint has been captured. Reused in both panels.
    let fingerprint_section: Html = html! {
        <div class="fingerprint-scan">
            <select ref={&fingerprint_door_select_ref} disabled={doors.is_empty()}>
                {
                    doors.iter().map(|door| {
                        html! {
                            <option value={door.id.to_string()}>{ &door.name }</option>
                        }
                    }).collect::<Html>()
                }
            </select>
            <button onclick={get_new_fingerprint_button_event} disabled={doors.is_empty()}>
                {"Scan fingerprint"}
            </button>
            {
                if fingerprint_data.is_empty() {
                    html! { <span>{"No fingerprint captured yet"}</span> }
                } else {
                    html! { <span>{"Fingerprint captured ✓"}</span> }
                }
            }
        </div>
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

                        { fingerprint_section.clone() }

                        { door_access_section.clone() }

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

                        { fingerprint_section.clone() }

                        { door_access_section.clone() }

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
                            <th>{"Door Access"}</th>
                        </tr>
                    </thead>

                    <tbody>
                        {
                            users.iter().map(|user| {
                                let delete_user_button_event = delete_user_button_event.clone();
                                let id = user.id;
                                let open_edit_panel = open_edit_panel.clone();

                                let door_names = if user.door_access.is_empty() {
                                    "-".to_string()
                                } else {
                                    user.door_access.iter()
                                        .map(|access| {
                                            doors.iter()
                                                .find(|door| door.id == access.door_id)
                                                .map(|door| door.name.clone())
                                                .unwrap_or_else(|| format!("Door #{}", access.door_id))
                                        })
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                };

                                html! {
                                    <tr>
                                        <td>{&user.id}</td>
                                        <td>{&user.firstName}</td>
                                        <td>{&user.lastName}</td>
                                        <td>{&user.email}</td>
                                        <td>{&user.group}</td>
                                        <td>{door_names}</td>
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

async fn get_new_fingerprint(door_ip: &str, ip: u32) {
    let url = format!("https://{door_ip}:8000/get-fingerprint/");

    let response = Request::get(url.as_str())
        .send()
        .await
        .unwrap();
}

async fn refresh_users(users: UseStateHandle<Vec<User>>) {
    let mut data = fetch_data().await;

    data.sort_by_key(|user| user.id);

    users.set(data);
}

async fn fetch_data() -> Vec<User> {

    let response = Request::get("https://127.0.0.1:8000/users-data")
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await
            .unwrap();

    response.json::<Vec<User>>().await.unwrap()
}

async fn fetch_doors() -> Vec<DoorOption> {

    let response = Request::get("https://127.0.0.1:8000/doors-data")
            .credentials(web_sys::RequestCredentials::Include)
            .send()
            .await
            .unwrap();

    response.json::<Vec<DoorOption>>().await.unwrap()
}

async fn add_user(
    id: u32,
    group: String,
    first_name: String,
    last_name: String,
    email: String,
    door_access: Vec<DoorAccess>,
    fingerprint: String,
) {

    let user = User {
        id,
        group,
        firstName: first_name,
        lastName: last_name,
        email,
        door_access,
        fingerprint,
    };

    let body = serde_json::to_string(&user).expect("failed to serialize user");

    let response = Request::put(format!("https://127.0.0.1:8000/add-user/{id}").as_str())
            .credentials(web_sys::RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("wrong formatting!")
            .send()
            .await
            .unwrap();

}

async fn edit_user(
    id: u32,
    group: String,
    first_name: String,
    last_name: String,
    email: String,
    door_access: Vec<DoorAccess>,
    fingerprint: String,
) {

    let user = User {
        id,
        group,
        firstName: first_name,
        lastName: last_name,
        email,
        door_access,
        fingerprint,
    };

    let body = serde_json::to_string(&user).expect("failed to serialize user");

    let response = Request::post(format!("https://127.0.0.1:8000/edit-user/{id}").as_str())
            .credentials(web_sys::RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .body(body)
            .expect("wrong formatting!")
            .send()
            .await
            .unwrap();
}

async fn delete_user(id: u32) {

    let response = Request::delete(format!("https://127.0.0.1:8000/delete-user/{id}").as_str())
            .credentials(web_sys::RequestCredentials::Include)
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