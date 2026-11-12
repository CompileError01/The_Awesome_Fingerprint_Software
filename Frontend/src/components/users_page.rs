use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::Callback;
use yew::TargetCast;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use web_sys::{window, HtmlInputElement, HtmlSelectElement};

use gloo_net::http::Request;

const DAY_NAMES: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DaySchedule {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_start_time")]
    start: String,
    #[serde(default = "default_end_time")]
    end: String,
}

fn default_start_time() -> String {
    "09:00".to_string()
}

fn default_end_time() -> String {
    "17:00".to_string()
}

impl Default for DaySchedule {
    fn default() -> Self {
        DaySchedule {
            enabled: false,
            start: default_start_time(),
            end: default_end_time(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DoorAccess {
    #[serde(alias = "door_id")]
    door_id: u32,
    #[serde(default, alias = "always_allowed")]
    always_allowed: bool,
    #[serde(default = "default_schedule")]
    schedule: [DaySchedule; 7],
}

fn default_schedule() -> [DaySchedule; 7] {
    std::array::from_fn(|_| DaySchedule::default())
}

impl DoorAccess {
    fn new(door_id: u32) -> Self {
        DoorAccess {
            door_id,
            always_allowed: false,
            schedule: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DoorOption {
    id: u32,
    name: String,
    #[serde(default)]
    ip_addr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct User {
    id: u32,
    #[serde(alias = "first_name")]
    first_name: String,
    #[serde(alias = "last_name")]
    last_name: String,
    #[serde(default, alias = "is_admin")]
    is_admin: bool,
    #[serde(default, alias = "is_group_leader")]
    is_group_leader: bool,
    #[serde(default, alias = "expires_on")]
    expires_on: Option<String>,
    #[serde(default, alias = "door_access")]
    door_access: Vec<DoorAccess>,
    // One ID per enrolled finger. For now always `[id]`.
    #[serde(default, alias = "finger_ids")]
    finger_ids: Vec<u32>,
    // Minutes the door stays open when a group leader presses the button.
    #[serde(default, alias = "open_duration")]
    open_duration: u32,
}

#[function_component(UsersPage)]
pub fn users_page() -> Html {
    let users = use_state(Vec::<User>::new);

    {
        let users = users.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                refresh_users(users).await;
            });

            || ()
        });
    }

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

    let latest_id: u32 = find_available_id(&users);

    let show_add_panel = use_state(|| false);

    let first_name_input_ref = use_node_ref();
    let last_name_input_ref = use_node_ref();
    let admin_input_ref = use_node_ref();
    let group_leader_input_ref = use_node_ref();
    let expiration_input_ref = use_node_ref();
    let open_hours_input_ref = use_node_ref();
    let open_minutes_input_ref = use_node_ref();

    let first_name_input_ref_cb = first_name_input_ref.clone();
    let last_name_input_ref_cb = last_name_input_ref.clone();
    let admin_input_ref_cb = admin_input_ref.clone();
    let group_leader_input_ref_cb = group_leader_input_ref.clone();
    let expiration_input_ref_cb = expiration_input_ref.clone();
    let open_hours_input_ref_cb = open_hours_input_ref.clone();
    let open_minutes_input_ref_cb = open_minutes_input_ref.clone();

    let door_access_list = use_state(Vec::<DoorAccess>::new);
    let finger_ids = use_state(Vec::<u32>::new);
    let fingerprint_user_id = use_state(|| 1u32);

    let new_door_select_ref = use_node_ref();
    let fingerprint_door_select_ref = use_node_ref();

    let open_add_panel = {
        let show_add_panel = show_add_panel.clone();
        let door_access_list = door_access_list.clone();
        let finger_ids = finger_ids.clone();
        let fingerprint_user_id = fingerprint_user_id.clone();

        Callback::from(move |_| {
            door_access_list.set(Vec::new());
            finger_ids.set(vec![latest_id]);
            fingerprint_user_id.set(latest_id);
            show_add_panel.set(true);
        })
    };

    let close_add_panel = {
        let show_add_panel = show_add_panel.clone();
        Callback::from(move |_| show_add_panel.set(false))
    };

    let show_edit_panel = use_state(|| false);
    let id_from_table = use_state(|| 0u32);

    let open_edit_panel = {
        let show_edit_panel = show_edit_panel.clone();
        let id_from_table = id_from_table.clone();
        let door_access_list = door_access_list.clone();
        let finger_ids = finger_ids.clone();
        let fingerprint_user_id = fingerprint_user_id.clone();
        let users = users.clone();

        Callback::from(move |id: u32| {
            id_from_table.set(id);
            fingerprint_user_id.set(id);

            if let Some(user) = users.iter().find(|user| user.id == id) {
                door_access_list.set(user.door_access.clone());
                finger_ids.set(if user.finger_ids.is_empty() {
                    vec![id]
                } else {
                    user.finger_ids.clone()
                });
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
    let admin_input_ref_cb_clone = admin_input_ref_cb.clone();
    let group_leader_input_ref_cb_clone = group_leader_input_ref_cb.clone();
    let expiration_input_ref_cb_clone = expiration_input_ref_cb.clone();
    let open_hours_input_ref_cb_clone = open_hours_input_ref_cb.clone();
    let open_minutes_input_ref_cb_clone = open_minutes_input_ref_cb.clone();

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
        let finger_ids = finger_ids.clone();
        let users = users.clone();

        Callback::from(move |_| {
            let first_name_input_ref = first_name_input_ref_cb.clone();
            let last_name_input_ref = last_name_input_ref_cb.clone();
            let admin_input_ref = admin_input_ref_cb.clone();
            let group_leader_input_ref = group_leader_input_ref_cb.clone();
            let expiration_input_ref = expiration_input_ref_cb.clone();
            let users = users.clone();

            let id = latest_id;

            let first_name = first_name_input_ref
                .cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();

            let last_name = last_name_input_ref
                .cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();

            let is_admin = admin_input_ref
                .cast::<HtmlInputElement>()
                .map(|input| input.checked())
                .unwrap_or(false);

            let is_group_leader = group_leader_input_ref
                .cast::<HtmlInputElement>()
                .map(|input| input.checked())
                .unwrap_or(false);

            let expires_on = expiration_input_ref
                .cast::<HtmlInputElement>()
                .map(|input| input.value())
                .filter(|value| !value.is_empty());

            let open_duration = read_open_duration(
                &open_hours_input_ref_cb,
                &open_minutes_input_ref_cb,
            );

            if !first_name.is_empty() && !last_name.is_empty() {
                let door_access = (*door_access_list).clone();
                let finger_ids = (*finger_ids).clone();

                spawn_local(async move {
                    let result = add_user(
                        id,
                        first_name,
                        last_name,
                        is_admin,
                        is_group_leader,
                        expires_on,
                        door_access,
                        finger_ids,
                        open_duration,
                    )
                    .await;

                    if result {
                        refresh_users(users).await;
                    } else {
                        window()
                            .unwrap()
                            .alert_with_message(
                                "Could not add user. The ID may already exist or the server rejected the request.",
                            )
                            .ok();
                    }
                });

                show_add_panel.set(false);
            } else {
                window()
                    .unwrap()
                    .alert_with_message("First and last name are required.")
                    .ok();
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
                    if !delete_user(id).await {
                        window()
                            .unwrap()
                            .alert_with_message("Could not delete user.")
                            .ok();
                    }

                    refresh_users(users).await;
                });
            }
        })
    };

    let edit_user_button_event = {
        let show_edit_panel = show_edit_panel.clone();
        let id = *id_from_table;
        let door_access_list = door_access_list.clone();
        let finger_ids = finger_ids.clone();
        let users = users.clone();

        Callback::from(move |_| {
            let first_name = first_name_input_ref_cb_clone
                .cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();

            let last_name = last_name_input_ref_cb_clone
                .cast::<HtmlInputElement>()
                .map(|input| input.value().trim().to_string())
                .unwrap_or_default();

            let is_admin = admin_input_ref_cb_clone
                .cast::<HtmlInputElement>()
                .map(|input| input.checked())
                .unwrap_or(false);

            let is_group_leader = group_leader_input_ref_cb_clone
                .cast::<HtmlInputElement>()
                .map(|input| input.checked())
                .unwrap_or(false);

            let expires_on = expiration_input_ref_cb_clone
                .cast::<HtmlInputElement>()
                .map(|input| input.value())
                .filter(|value| !value.is_empty());

            let open_duration = read_open_duration(
                &open_hours_input_ref_cb_clone,
                &open_minutes_input_ref_cb_clone,
            );

            if first_name.is_empty() || last_name.is_empty() {
                window()
                    .unwrap()
                    .alert_with_message("First and last name are required.")
                    .ok();

                return;
            }

            let door_access = (*door_access_list).clone();
            let finger_ids = (*finger_ids).clone();
            let users = users.clone();

            spawn_local(async move {
                let result = edit_user(
                    id,
                    first_name,
                    last_name,
                    is_admin,
                    is_group_leader,
                    expires_on,
                    door_access,
                    finger_ids,
                    open_duration,
                )
                .await;

                if result {
                    refresh_users(users).await;
                } else {
                    window()
                        .unwrap()
                        .alert_with_message("Could not save user changes.")
                        .ok();
                }
            });

            show_edit_panel.set(false);
        })
    };

    let get_new_fingerprint_button_event = {
        let fingerprint_door_select_ref = fingerprint_door_select_ref.clone();
        let doors = doors.clone();
        let user_id = *fingerprint_user_id;

        Callback::from(move |_| {
            let selected_id = fingerprint_door_select_ref
                .cast::<HtmlSelectElement>()
                .and_then(|select| select.value().parse::<u32>().ok());

            let door_ip = selected_id
                .and_then(|door_id| doors.iter().find(|door| door.id == door_id))
                .or_else(|| doors.first())
                .map(|door| door.ip_addr.clone());

            match door_ip {
                Some(door_ip) => {
                    spawn_local(async move {
                        if let Err(error) = get_new_fingerprint(&door_ip, user_id).await {
                            window()
                                .unwrap()
                                .alert_with_message(&format!(
                                    "Fingerprint enrollment request failed: {error}"
                                ))
                                .ok();
                        }
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

    let available_doors: Vec<DoorOption> = doors
        .iter()
        .filter(|door| {
            !door_access_list
                .iter()
                .any(|access| access.door_id == door.id)
        })
        .cloned()
        .collect();

    let door_access_section: Html = html! {
        <div class="door-access-editor">
            <h3>{"Door access"}</h3>

            {
                door_access_list.iter().enumerate().map(|(index, access)| {
                    let door_name = doors.iter()
                        .find(|door| door.id == access.door_id)
                        .map(|door| door.name.clone())
                        .unwrap_or_else(|| format!("Door #{}", access.door_id));

                    let always_allowed_event = {
                        let door_access_list = door_access_list.clone();

                        Callback::from(move |_: Event| {
                            let mut updated = (*door_access_list).clone();
                            updated[index].always_allowed =
                                !updated[index].always_allowed;
                            door_access_list.set(updated);
                        })
                    };

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

                            <label>
                                <input
                                    type="checkbox"
                                    checked={access.always_allowed}
                                    onchange={always_allowed_event}
                                />
                                {" Always allowed (24/7)"}
                            </label>

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
                                                        disabled={access.always_allowed}
                                                        onchange={toggle_event}
                                                    />
                                                </td>
                                                <td>
                                                    <input
                                                        type="time"
                                                        value={day.start.clone()}
                                                        oninput={start_event}
                                                        disabled={access.always_allowed || !day.enabled}
                                                    />
                                                </td>
                                                <td>
                                                    <input
                                                        type="time"
                                                        value={day.end.clone()}
                                                        oninput={end_event}
                                                        disabled={access.always_allowed || !day.enabled}
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

    let fingerprint_section: Html = html! {
        <div class="fingerprint-scan">
            <select
                ref={&fingerprint_door_select_ref}
                disabled={doors.is_empty()}
            >
                {
                    doors.iter().enumerate().map(|(index, door)| {
                        html! {
                            <option value={door.id.to_string()} selected={index == 0}>
                                { &door.name }
                            </option>
                        }
                    }).collect::<Html>()
                }
            </select>

            <button
                onclick={get_new_fingerprint_button_event}
                disabled={doors.is_empty()}
            >
                {"Scan fingerprint"}
            </button>

            <span>
                {
                    format!(
                        "Finger IDs: {}",
                        finger_ids
                            .iter()
                            .map(|finger_id| finger_id.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
            </span>
        </div>
    };

    let user = get_single_user_data(*id_from_table, &users);

    html! {
        <>
            if *show_add_panel {
                <div
                    class="modal-backdrop"
                    onclick={close_add_panel.clone()}
                >
                    <div
                        class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <h2>{"Add a user"}</h2>

                        <div>
                            <input
                                type="text"
                                ref={&first_name_input_ref}
                                required=true
                                placeholder="First Name"
                            />

                            <input
                                type="text"
                                ref={&last_name_input_ref}
                                required=true
                                placeholder="Last Name"
                            />

                            <label>
                                <input type="checkbox" ref={&admin_input_ref}/>
                                {" Administrator"}
                            </label>

                            <label>
                                <input type="checkbox" ref={&group_leader_input_ref}/>
                                {" Group leader"}
                            </label>

                            <label>
                                {"Expires on (optional)"}
                                <input type="date" ref={&expiration_input_ref}/>
                            </label>

                            <label>
                                {"Open duration (group leaders)"}
                                <input
                                    type="number"
                                    min="0"
                                    step="1"
                                    ref={&open_hours_input_ref}
                                    value="0"
                                />
                                {" h "}
                                <input
                                    type="number"
                                    min="0"
                                    max="59"
                                    step="1"
                                    ref={&open_minutes_input_ref}
                                    value="0"
                                />
                                {" min"}
                            </label>
                        </div>

                        { fingerprint_section.clone() }
                        { door_access_section.clone() }

                        <div>
                            <button onclick={add_user_button_event}>{"Add"}</button>
                            <button onclick={close_add_panel}>{"Cancel"}</button>
                        </div>
                    </div>
                </div>
            } else if *show_edit_panel {
                <div
                    class="modal-backdrop"
                    onclick={close_edit_panel.clone()}
                >
                    <div
                        class="modal"
                        onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}
                    >
                        <h2>{"Edit User"}</h2>

                        {
                            if let Some(user) = user.as_ref() {
                                html! {
                                    <>
                                        <div>
                                            <input
                                                type="text"
                                                ref={&first_name_input_ref}
                                                required=true
                                                value={user.first_name.clone()}
                                                placeholder="First Name"
                                            />

                                            <input
                                                type="text"
                                                ref={&last_name_input_ref}
                                                required=true
                                                value={user.last_name.clone()}
                                                placeholder="Last Name"
                                            />

                                            <label>
                                                <input
                                                    type="checkbox"
                                                    ref={&admin_input_ref}
                                                    checked={user.is_admin}
                                                />
                                                {" Administrator"}
                                            </label>

                                            <label>
                                                <input
                                                    type="checkbox"
                                                    ref={&group_leader_input_ref}
                                                    checked={user.is_group_leader}
                                                />
                                                {" Group leader"}
                                            </label>

                                            <label>
                                                {"Expires on (optional)"}
                                                <input
                                                    type="date"
                                                    ref={&expiration_input_ref}
                                                    value={user.expires_on.clone().unwrap_or_default()}
                                                />
                                            </label>

                                            <label>
                                                {"Open duration (group leaders)"}
                                                <input
                                                    type="number"
                                                    min="0"
                                                    step="1"
                                                    ref={&open_hours_input_ref}
                                                    value={(user.open_duration / 60).to_string()}
                                                />
                                                {" h "}
                                                <input
                                                    type="number"
                                                    min="0"
                                                    max="59"
                                                    step="1"
                                                    ref={&open_minutes_input_ref}
                                                    value={(user.open_duration % 60).to_string()}
                                                />
                                                {" min"}
                                            </label>
                                        </div>

                                        { fingerprint_section.clone() }
                                        { door_access_section.clone() }

                                        <div>
                                            <button onclick={edit_user_button_event}>{"Apply"}</button>
                                            <button onclick={close_edit_panel}>{"Cancel"}</button>
                                        </div>
                                    </>
                                }
                            } else {
                                html! {
                                    <p>{"User not found."}</p>
                                }
                            }
                        }
                    </div>
                </div>
            } else {
                <h1>{"Users"}</h1>

                <p>{format!("Loaded {} users", users.len())}</p>

                <button onclick={open_add_panel}>{"Add user"}</button>

                if users.is_empty() {
                    <p>{"No users yet. Add your first user."}</p>
                }

                <table border="1" width="80%">
                    <thead>
                        <tr>
                            <th>{"ID"}</th>
                            <th>{"First-Name"}</th>
                            <th>{"Last-Name"}</th>
                            <th>{"Roles"}</th>
                            <th>{"Expires on"}</th>
                            <th>{"Door Access"}</th>
                            <th>{"Actions"}</th>
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
                                        <td>{ user.id }</td>
                                        <td>{ &user.first_name }</td>
                                        <td>{ &user.last_name }</td>
                                        <td>{
                                            [
                                                if user.is_admin { Some("Admin") } else { None },
                                                if user.is_group_leader { Some("Group leader") } else { None },
                                            ]
                                            .into_iter()
                                            .flatten()
                                            .collect::<Vec<&str>>()
                                            .join(", ")
                                        }</td>
                                        <td>{ user.expires_on.clone().unwrap_or_else(|| "Never".to_string()) }</td>
                                        <td>{ door_names }</td>
                                        <td>
                                            <button onclick={
                                                Callback::from(move |_: MouseEvent| {
                                                    open_edit_panel.emit(id);
                                                })
                                            }>
                                                {"Edit"}
                                            </button>

                                            <button onclick={
                                                Callback::from(move |_: MouseEvent| {
                                                    delete_user_button_event.emit(id);
                                                })
                                            }>
                                                {"Delete"}
                                            </button>
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

// Combines the hours and minutes inputs into a total number of minutes.
fn read_open_duration(hours_ref: &NodeRef, minutes_ref: &NodeRef) -> u32 {
    let read = |node_ref: &NodeRef| {
        node_ref
            .cast::<HtmlInputElement>()
            .and_then(|input| input.value().trim().parse::<u32>().ok())
            .unwrap_or(0)
    };

    read(hours_ref)
        .saturating_mul(60)
        .saturating_add(read(minutes_ref))
}

async fn get_new_fingerprint(door_ip: &str, user_id: u32) -> Result<(), String> {
    let url = format!("https://{door_ip}:8000/get-fingerprint/?user_id={user_id}");

    let response = Request::get(url.as_str())
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("door returned HTTP {}", response.status()))
    }
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
    first_name: String,
    last_name: String,
    is_admin: bool,
    is_group_leader: bool,
    expires_on: Option<String>,
    door_access: Vec<DoorAccess>,
    finger_ids: Vec<u32>,
    open_duration: u32,
) -> bool {
    let user = User {
        id,
        first_name,
        last_name,
        is_admin,
        is_group_leader,
        expires_on,
        door_access,
        finger_ids,
        open_duration,
    };

    let Ok(body) = serde_json::to_string(&user) else {
        return false;
    };

    match Request::put(format!("https://127.0.0.1:8000/add-user/{id}").as_str())
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .expect("wrong formatting!")
        .send()
        .await
    {
        Ok(response) => response.ok(),
        Err(_) => false,
    }
}

async fn edit_user(
    id: u32,
    first_name: String,
    last_name: String,
    is_admin: bool,
    is_group_leader: bool,
    expires_on: Option<String>,
    door_access: Vec<DoorAccess>,
    finger_ids: Vec<u32>,
    open_duration: u32,
) -> bool {
    let user = User {
        id,
        first_name,
        last_name,
        is_admin,
        is_group_leader,
        expires_on,
        door_access,
        finger_ids,
        open_duration,
    };

    let Ok(body) = serde_json::to_string(&user) else {
        return false;
    };

    match Request::post(format!("https://127.0.0.1:8000/edit-user/{id}").as_str())
        .credentials(web_sys::RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .expect("wrong formatting!")
        .send()
        .await
    {
        Ok(response) => response.ok(),
        Err(_) => false,
    }
}

async fn delete_user(id: u32) -> bool {
    match Request::delete(format!("https://127.0.0.1:8000/delete-user/{id}").as_str())
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) => response.ok(),
        Err(_) => false,
    }
}

fn get_single_user_data(id: u32, users: &[User]) -> Option<User> {
    users.iter().find(|user| user.id == id).cloned()
}

fn find_available_id(users: &[User]) -> u32 {
    let mut candidate = 1u32;

    while users.iter().any(|user| user.id == candidate) {
        candidate = candidate.saturating_add(1);

        if candidate == u32::MAX {
            break;
        }
    }

    candidate
}