use rocket::http::Status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Door {
    pub id: u32,
    pub name: String,
    pub status: String,
    pub ip_addr: String,
    pub description: String,
}

fn doors_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Doors")
}

fn check_session(
    cookies: &rocket::http::CookieJar<'_>,
) -> Result<(), Status> {
    let session = match cookies.get_private("session") {
        Some(cookie) => cookie.value().to_string(),
        None => return Err(Status::Unauthorized),
    };

    if session != "valid_session_cookie" {
        return Err(Status::Unauthorized);
    }

    Ok(())
}

#[get("/doors-data")]
pub fn doors_data(
    cookies: &rocket::http::CookieJar<'_>,
) -> Result<Json<Vec<Door>>, Status> {
    check_session(cookies)?;

    let directory = doors_directory();

    if let Err(_) = fs::create_dir_all(&directory) {
        return Err(Status::InternalServerError);
    }

    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(_) => return Err(Status::InternalServerError),
    };

    let mut doors = Vec::<Door>::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(_) => continue,
        };

        let door = match serde_json::from_str::<Door>(&contents) {
            Ok(door) => door,
            Err(_) => continue,
        };

        doors.push(door);
    }

    doors.sort_by_key(|door| door.id);

    Ok(Json(doors))
}

#[put("/add-door/<id>", data = "<door>")]
pub async fn add_door_route(
    id: u32,
    door: Json<Door>,
    cookies: &rocket::http::CookieJar<'_>,
) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    if door.id != id {
        return Status::BadRequest;
    }

    if door.name.trim().is_empty() {
        return Status::BadRequest;
    }

    let directory = doors_directory();

    if let Err(_) = fs::create_dir_all(&directory) {
        return Status::InternalServerError;
    }

    let path = directory.join(format!("{}.json", id));

    if path.exists() {
        return Status::Conflict;
    }

    let door_to_save = Door {
        id: door.id,
        name: door.name.clone(),
        status: if door.status.trim().is_empty() {
            "Closed".to_string()
        } else {
            door.status.clone()
        },
        ip_addr: door.ip_addr.clone(),
        description: door.description.clone(),
    };

    let json = match serde_json::to_string_pretty(&door_to_save) {
        Ok(json) => json,
        Err(_) => return Status::InternalServerError,
    };

    if fs::write(&path, json).is_err() {
        return Status::InternalServerError;
    }

    crate::log(
        format!("Added door: {}", id),
        crate::get_doer(cookies),
    );

    Status::Ok
}

#[delete("/delete-door/<id>")]
pub async fn delete_door_route(
    id: u32,
    cookies: &rocket::http::CookieJar<'_>,
) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    let path = doors_directory().join(format!("{}.json", id));

    match fs::remove_file(&path) {
        Ok(_) => {
            crate::log(
                format!("Deleted door: {}", id),
                crate::get_doer(cookies),
            );

            Status::Ok
        }

        Err(error) => {
            if error.kind() == std::io::ErrorKind::NotFound {
                Status::NotFound
            } else {
                Status::InternalServerError
            }
        }
    }
}

pub fn add_door(
    id: u32,
    name: String,
    ip_addr: String,
    description: String,
) {
    let directory = doors_directory();

    if fs::create_dir_all(&directory).is_err() {
        return;
    }

    let mut highest_id = 0u32;

    if let Ok(entries) = fs::read_dir(&directory) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }

            if let Some(stem) = path.file_stem().and_then(|x| x.to_str()) {
                if let Ok(id) = stem.parse::<u32>() {
                    if id > highest_id {
                        highest_id = id;
                    }
                }
            }
        }
    }

    let id = highest_id + 1;

    let door = Door {
        id,
        name,
        status: "Closed".to_string(),
        ip_addr,
        description,
    };

    let path = directory.join(format!("{}.json", id));

    if let Ok(json) = serde_json::to_string_pretty(&door) {
        let _ = fs::write(path, json);
    }
}