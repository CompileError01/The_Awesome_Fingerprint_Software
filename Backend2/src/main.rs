#[macro_use]
extern crate rocket;

mod doors;

use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket::tokio;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use rocket::fs::TempFile;

use serde::{Deserialize, Serialize};

use std::fmt::format;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct PanelUser {
    username: String,
    passwordHash: String,
}

// Schedule for a single weekday. Used inside DoorAccess::schedule,
// index 0 = Monday ... index 6 = Sunday.
// yes I know that this struct is a duplicate, and has to be updated twice,
// please leave a complaint at goFrickYourself@gmail.com
// also has to be UPDATED in the FRONTEND
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DaySchedule {
    enabled: bool,
    start: String,
    end: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DoorAccess {
    door_id: u32,
    schedule: [DaySchedule; 7],
}

// yes I know that this struct is a duplicate, and has to be updated twice,
// please leave a complaint at goFrickYourself@gmail.com
// also has to be UPDATED in the FRONTEND
#[derive(Serialize, Deserialize, Debug)]
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

#[get("/get-auth-state")]
fn get_auth_state(cookies: &CookieJar<'_>) -> Status {
    let session = match cookies.get_private("session") {
        Some(cookie) => cookie.value().to_string(),
        None => return Status::Unauthorized,
    };

    if session == "valid_session_cookie" {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[post("/verify-login", data = "<data>")]
fn verify_login(
    data: Json<PanelUser>,
    cookies: &CookieJar<'_>,
) -> Status {
    let username = data.username.clone();
    let password_hash = data.passwordHash.clone();

    let panel_users = get_panel_users_file_names();

    for user_file in panel_users {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("PanelUsers")
            .join(user_file);

        let file_content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let user = match serde_json::from_str::<PanelUser>(&file_content) {
            Ok(user) => user,
            Err(_) => continue,
        };

        if username == user.username && password_hash == user.passwordHash {
            cookies.add_private(
                Cookie::build(("session", "valid_session_cookie"))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax)
                    .max_age(cookie::time::Duration::seconds(60 * 60)),
            );

            cookies.add_private(
                Cookie::build(("doer", username.clone()))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax)
                    .max_age(cookie::time::Duration::seconds(60 * 60)),
            );

            log("Access Granted".to_string(), username);

            return Status::Ok;
        }
    }

    log("Access Denied".to_string(), username);

    Status::Unauthorized
}

#[get("/users-data")]
fn users_data(
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<User>>, Status> {
    check_session(cookies)?;

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Users");

    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(_) => return Err(Status::InternalServerError),
    };

    let mut users = Vec::<User>::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();

        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let file_content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let user = match serde_json::from_str::<User>(&file_content) {
            Ok(user) => user,
            Err(_) => continue,
        };

        users.push(user);
    }

    users.sort_by_key(|user| user.id);

    Ok(Json(users))
}

#[put("/add-user/<id>", data = "<user>")]
async fn add_user(
    id: u32,
    user: Json<User>,
    cookies: &CookieJar<'_>,
) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    if user.id != id {
        return Status::BadRequest;
    }

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Users");

    if fs::create_dir_all(&directory).is_err() {
        return Status::InternalServerError;
    }

    let path = directory.join(format!("{}.json", id));

    if path.exists() {
        return Status::Conflict;
    }

    let json = match serde_json::to_string_pretty(&*user) {
        Ok(json) => json,
        Err(_) => return Status::InternalServerError,
    };

    if fs::write(path, json).is_err() {
        return Status::InternalServerError;
    }

    log(
        format!("Added user: {}", id),
        get_doer(cookies),
    );

    Status::Ok
}

#[delete("/delete-user/<id>")]
async fn delete_user(
    id: u32,
    cookies: &CookieJar<'_>,
) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Users")
        .join(format!("{}.json", id));

    match fs::remove_file(path) {
        Ok(_) => {
            log(
                format!("Deleted user: {}", id),
                get_doer(cookies),
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

#[post("/edit-user/<id>", data = "<user>")]
async fn edit_user(
    id: u32,
    user: Json<User>,
    cookies: &CookieJar<'_>,
) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    if user.id != id {
        return Status::BadRequest;
    }

    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Users");

    if fs::create_dir_all(&directory).is_err() {
        return Status::InternalServerError;
    }

    let path = directory.join(format!("{}.json", id));

    if !path.exists() {
        return Status::NotFound;
    }

    let json = match serde_json::to_string_pretty(&*user) {
        Ok(json) => json,
        Err(_) => return Status::InternalServerError,
    };

    if fs::write(path, json).is_err() {
        return Status::InternalServerError;
    }

    log(
        format!("Edited user: {}", id),
        get_doer(cookies),
    );

    Status::Ok
}

#[post("/enroll-finished/<id>", data = "<fingerprint_file>")]
async fn enroll_finished(mut fingerprint_file: TempFile<'_>, id: u32) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Users")
        .join(format!("{}.bin", id));
    
    fingerprint_file.copy_to(path).await.unwrap();
}

fn check_session(
    cookies: &CookieJar<'_>,
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

pub fn get_doer(
    cookies: &CookieJar<'_>,
) -> String {
    match cookies.get_private("doer") {
        Some(cookie) => cookie.value().to_string(),
        None => "Unknown".to_string(),
    }
}

pub fn log(
    event: String,
    doer: String,
) {
    println!("[{}] {} - {}", chrono::Local::now(), doer, event);
}

fn get_panel_users_file_names() -> Vec<PathBuf> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("PanelUsers");

    match fs::read_dir(directory) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|x| x.to_str()) == Some("json"))
            .filter_map(|path| path.file_name().map(PathBuf::from))
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration failed");

    rocket::build()
        .attach(cors)
        .mount(
            "/",
            routes![
                get_auth_state,
                verify_login,
                users_data,
                add_user,
                delete_user,
                edit_user,
                doors::doors_data,
                doors::add_door_route,
                doors::delete_door_route,
                enroll_finished
            ],
        )
}