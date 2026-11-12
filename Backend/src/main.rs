#[macro_use]
extern crate rocket;

mod doors;

use rocket::fs::TempFile;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::serde::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PanelUser {
    username: String,
    password_hash: String,
}

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

fn default_schedule() -> [DaySchedule; 7] {
    std::array::from_fn(|_| DaySchedule {
        enabled: false,
        start: default_start_time(),
        end: default_end_time(),
    })
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    // ISO date: YYYY-MM-DD. None means the account does not expire.
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

impl User {
    // Until multiple fingers can be enrolled, the only finger ID is the user ID.
    fn normalized(mut self) -> Self {
        if self.finger_ids.is_empty() {
            self.finger_ids = vec![self.id];
        }
        self
    }
}

#[get("/get-auth-state")]
fn get_auth_state(cookies: &CookieJar<'_>) -> Status {
    if check_session(cookies).is_ok() {
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[post("/verify-login", data = "<data>")]
fn verify_login(data: Json<PanelUser>, cookies: &CookieJar<'_>) -> Status {
    let username = data.username.clone();
    let password_hash = data.password_hash.clone();

    for user_file in get_panel_users_file_names() {
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

        if username == user.username && password_hash == user.password_hash {
            let cookie = Cookie::build(("session", "valid_session_cookie"))
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .max_age(cookie::time::Duration::seconds(60 * 60));
            cookies.add_private(cookie);

            let doer_cookie = Cookie::build(("doer", username.clone()))
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .max_age(cookie::time::Duration::seconds(60 * 60));
            cookies.add_private(doer_cookie);

            log("Access Granted".to_string(), username);
            return Status::Ok;
        }
    }

    log("Access Denied".to_string(), username);
    Status::Unauthorized
}

#[get("/users-data")]
fn users_data(cookies: &CookieJar<'_>) -> Result<Json<Vec<User>>, Status> {
    check_session(cookies)?;

    let directory = users_directory();
    let entries = fs::read_dir(&directory).map_err(|_| Status::InternalServerError)?;
    let mut users = Vec::<User>::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if let Ok(user) = serde_json::from_str::<User>(&content) {
            users.push(user.normalized());
        }
    }

    users.sort_by_key(|user| user.id);
    Ok(Json(users))
}

#[put("/add-user/<id>", data = "<user>")]
fn add_user(id: u32, user: Json<User>, cookies: &CookieJar<'_>) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    if id == 0 || user.id != id {
        return Status::BadRequest;
    }

    let directory = users_directory();

    if fs::create_dir_all(&directory).is_err() {
        return Status::InternalServerError;
    }

    let path = directory.join(format!("{id}.json"));

    if path.exists() {
        return Status::Conflict;
    }

    let json = match serde_json::to_string_pretty(&(*user).clone().normalized()) {
        Ok(value) => value,
        Err(_) => return Status::InternalServerError,
    };

    if fs::write(path, json).is_err() {
        return Status::InternalServerError;
    }

    log(format!("Added user: {id}"), get_doer(cookies));
    Status::Ok
}

#[delete("/delete-user/<id>")]
fn delete_user(id: u32, cookies: &CookieJar<'_>) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    let path = users_directory().join(format!("{id}.json"));

    match fs::remove_file(path) {
        Ok(_) => {
            let _ = fs::remove_file(users_directory().join(format!("{id}.bin")));
            log(format!("Deleted user: {id}"), get_doer(cookies));
            Status::Ok
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Status::NotFound,
        Err(_) => Status::InternalServerError,
    }
}

#[post("/edit-user/<id>", data = "<user>")]
fn edit_user(id: u32, user: Json<User>, cookies: &CookieJar<'_>) -> Status {
    if let Err(status) = check_session(cookies) {
        return status;
    }

    if user.id != id || id == 0 {
        return Status::BadRequest;
    }

    let directory = users_directory();
    let path = directory.join(format!("{id}.json"));

    if !path.exists() {
        return Status::NotFound;
    }

    let json = match serde_json::to_string_pretty(&(*user).clone().normalized()) {
        Ok(value) => value,
        Err(_) => return Status::InternalServerError,
    };

    if fs::write(path, json).is_err() {
        return Status::InternalServerError;
    }

    log(format!("Edited user: {id}"), get_doer(cookies));
    Status::Ok
}

// The controller uploads the captured fingerprint template here.
#[post("/enroll-finished/<id>", data = "<fingerprint_file>")]
async fn enroll_finished(mut fingerprint_file: TempFile<'_>, id: u32) -> Status {
    if !users_directory().join(format!("{id}.json")).exists() {
        return Status::NotFound;
    }

    let path = users_directory().join(format!("{id}.bin"));

    match fingerprint_file.copy_to(path).await {
        Ok(_) => {
            log(
                format!("Fingerprint enrolled for user: {id}"),
                "Door controller".to_string(),
            );
            Status::Ok
        }
        Err(_) => Status::InternalServerError,
    }
}

fn users_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("Database")
        .join("Users")
}

fn check_session(cookies: &CookieJar<'_>) -> Result<(), Status> {
    match cookies.get_private("session") {
        Some(cookie) if cookie.value() == "valid_session_cookie" => Ok(()),
        _ => Err(Status::Unauthorized),
    }
}

fn get_doer(cookies: &CookieJar<'_>) -> String {
    cookies
        .get_private("doer")
        .map(|cookie| cookie.value().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn log(event: String, doer: String) {
    println!("[{}] {} - {}", chrono::Local::now(), doer, event);
}

fn get_panel_users_file_names() -> Vec<PathBuf> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("PanelUsers");

    match fs::read_dir(directory) {
        Ok(entries) => entries
            .flatten()
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
                enroll_finished,
            ],
        )
}
