use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::{window, HtmlInputElement};
use gloo_net::http::Request;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Door {
    id: u32,
    name: String,
    status: String,
    ip_addr: String,
    description: String,
}


#[function_component(DoorsPage)]
pub fn doors_page() -> Html {

    // Holds all doors returned by the API
    let doors = use_state(Vec::<Door>::new);

    {
        let doors = doors.clone();

        use_effect_with((), move |_| {

            spawn_local(async move {
                refresh_doors(doors).await;
            });

            || ()
        });
    }

    let show_add_panel = use_state(|| false);

    let door_name_input_ref = use_node_ref();
    let door_ip_input_ref = use_node_ref();
    let door_description_input_ref = use_node_ref();


    let open_add_panel = {
        let show_add_panel = show_add_panel.clone();

        Callback::from(move |_| {
            show_add_panel.set(true);
        })
    };


    let close_add_panel = {
        let show_add_panel = show_add_panel.clone();

        Callback::from(move |_| {
            show_add_panel.set(false);
        })
    };


    let latest_id = find_highest_door_id(&doors);

    let add_door_button_event = {

        let show_add_panel = show_add_panel.clone();
        let door_name_input_ref = door_name_input_ref.clone();
        let door_ip_input_ref = door_ip_input_ref.clone();
        let door_description_input_ref = door_description_input_ref.clone();
        let doors = doors.clone();

        Callback::from(move |_| {

            let door_name_input_ref = door_name_input_ref.clone();
            let door_ip_input_ref = door_ip_input_ref.clone();
            let door_description_input_ref = door_description_input_ref.clone();
            let doors = doors.clone();

            let id = latest_id + 1;

            let mut name = String::new();
            let mut ip_addr = String::new();
            let mut description = String::new();

            if let Some(input) =
                door_name_input_ref.cast::<HtmlInputElement>()
            {
                name = input.value();
            }

            if let Some(input) =
                door_ip_input_ref.cast::<HtmlInputElement>()
            {
                ip_addr = input.value();
            }

            if let Some(input) =
                door_description_input_ref.cast::<HtmlInputElement>()
            {
                description = input.value();
            }


            if !name.is_empty() {

                spawn_local(async move {

                    add_door(
                        id,
                        name,
                        ip_addr,
                        description,
                    ).await;

                    refresh_doors(doors).await;

                });

                show_add_panel.set(false);

            } else {

                window()
                    .unwrap()
                    .alert_with_message(
                        "Please enter a door name!"
                    )
                    .unwrap();
            }
        })
    };

    let delete_door_button_event = {

        let doors = doors.clone();

        Callback::from(move |id: u32| {

            let doors = doors.clone();

            let confirmed = window()
                .unwrap()
                .confirm_with_message(
                    "Are you sure you want to delete this door?"
                )
                .unwrap();


            if confirmed {

                spawn_local(async move {

                    delete_door(id).await;

                    refresh_doors(doors).await;

                });
            }
        })
    };

    html! {
        <>

            if *show_add_panel {

                <div
                    class="modal-backdrop"
                    onclick={close_add_panel.clone()}
                >

                    <div
                        class="modal"
                        onclick={
                            Callback::from(
                                |e: MouseEvent| {
                                    e.stop_propagation()
                                }
                            )
                        }
                    >

                        <h2>{"Add a door"}</h2>


                        <div>

                            <input
                                type="text"
                                ref={&door_name_input_ref}
                                required=true
                                placeholder="Door Name"
                            />

                        </div>


                        <div>

                            <input
                                type="text"
                                ref={&door_ip_input_ref}
                                placeholder="IP Address"
                            />

                        </div>


                        <div>

                            <input
                                type="text"
                                ref={&door_description_input_ref}
                                placeholder="Description"
                            />

                        </div>


                        <div>

                            <button
                                onclick={add_door_button_event}
                            >
                                {"Add"}
                            </button>


                            <button
                                onclick={close_add_panel}
                            >
                                {"Cancel"}
                            </button>

                        </div>

                    </div>

                </div>
            }


            else {

                <h1>{"Doors"}</h1>


                if doors.is_empty() {

                    <p>{"Loading..."}</p>

                } else {

                    <p>
                        {format!(
                            "Loaded {} doors",
                            doors.len()
                        )}
                    </p>

                }


                
                <button
                    onclick={open_add_panel}
                >
                    {"Add Door"}
                </button>


                <table border="1" width="80%">

                    <thead>

                        <tr>

                            <th>{"ID"}</th>

                            <th>{"Door Name"}</th>

                            <th>{"Status"}</th>

                            <th>{"Actions"}</th>

                        </tr>

                    </thead>


                    <tbody>

                        {
                            doors.iter().map(|door| {

                                let delete_door_button_event =
                                    delete_door_button_event.clone();

                                let id = door.id;


                                html! {

                                    <tr>


                                        <td>
                                            {door.id}
                                        </td>


                                        <td>
                                            {&door.name}
                                        </td>


                                        <td>

                                            {
                                                match door.status.as_str() {

                                                    "Open" => html! {
                                                        <span class="door-status open">
                                                            {"Open"}
                                                        </span>
                                                    },

                                                    "Closed" => html! {
                                                        <span class="door-status closed">
                                                            {"Closed"}
                                                        </span>
                                                    },

                                                    "Offline" => html! {
                                                        <span class="door-status offline">
                                                            {"Offline"}
                                                        </span>
                                                    },

                                                    _ => html! {
                                                        <span class="door-status">
                                                            {&door.status}
                                                        </span>
                                                    },
                                                }
                                            }

                                        </td>


                                        <td>

                                            <button
                                                onclick={
                                                    Callback::from(
                                                        move |_: MouseEvent| {

                                                            delete_door_button_event
                                                                .emit(id);

                                                        }
                                                    )
                                                }
                                            >
                                                {"Delete"}
                                            </button>

                                        </td>

                                    </tr>

                                }

                            })
                            .collect::<Html>()
                        }

                    </tbody>

                </table>

            }

        </>
    }
}


async fn refresh_doors(doors: UseStateHandle<Vec<Door>>) {

    let mut data = fetch_door_data().await;

    // Sort doors by ID
    data.sort_by_key(|door| door.id);

    doors.set(data);
}


async fn fetch_door_data() -> Vec<Door> {

    let response = Request::get(
        "https://127.0.0.1:8000/doors-data"
    )
    .credentials(
        web_sys::RequestCredentials::Include
    )
    .send()
    .await
    .unwrap();


    response
        .json::<Vec<Door>>()
        .await
        .unwrap()
}


async fn add_door(
    id: u32,
    name: String,
    ip_addr: String,
    description: String,
) {

    let response = Request::put(
        format!(
            "https://127.0.0.1:8000/add-door/{}",
            id
        )
        .as_str()
    )
    .credentials(
        web_sys::RequestCredentials::Include
    )
    .header(
        "Content-Type",
        "application/json"
    )
    .body(
        serde_json::to_string(
            &Door {
                id,
                name,
                status: "Closed".to_string(),
                ip_addr,
                description,
            }
        )
        .unwrap()
    )
    .expect("Wrong formatting!")
    .send()
    .await
    .unwrap();
}

async fn delete_door(
    id: u32
) {

    let response = Request::delete(
        format!(
            "https://127.0.0.1:8000/delete-door/{}",
            id
        )
        .as_str()
    )
    .credentials(
        web_sys::RequestCredentials::Include
    )
    .send()
    .await
    .unwrap();
}

fn find_highest_door_id(
    doors: &Vec<Door>
) -> u32 {

    let mut highest_id = 0;


    for door in doors {

        if door.id > highest_id {
            highest_id = door.id;
        }

    }


    highest_id
}