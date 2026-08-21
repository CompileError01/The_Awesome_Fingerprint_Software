// needed by rocket
#[macro_use]
extern crate rocket;
use cookie::prefix::Secure;
use rocket::http::Status;
use rocket::tokio::time;
use rocket::{data, launch};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::{Cookie, CookieJar, SameSite};

// CORS
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

use std::fmt::format;
// for PathBuffs
use std::{fs, path::Path, path::PathBuf};

// for Json
use serde::{Serialize, Deserialize};
use rocket::serde::json::Json;
use serde_json::{Value, json};

// for date and time
use chrono::{DateTime, Local};

use std::time::Duration;

struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

#[derive(Serialize, Deserialize, Debug)]
struct PanelUser {
    username: String,
    passwordHash: String,
}

/*#[derive(Serialize, Deserialize, Debug)]
struct Log {
    datetime: String,
    event: String,
    doer: String,
}*/

// yes I know that this struct is a duplicate, and has to be updated twice, please leave a complaint at goFrickYourself@gmail.com
// has to also be updated on the Dashboard_Page
#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    group: String,
    firstName: String,
    lastName: String,
    email: String,
}

#[get("/get-auth-state")]
fn get_auth_state(cookies: &CookieJar<'_>) -> Status {
    let session = match cookies.get_private("session") {
        Some(cookie) => cookie.value().to_string(),
        None => return Status::Unauthorized
    };

    if session == "valid_session_cookie" {
        return Status::Ok;
    }

    Status::Unauthorized
}

#[post("/verify-login", data="<data>")]
fn verify_login(data: Json<PanelUser>, cookies: &CookieJar<'_>) -> Status {
    
    let username = data.username.clone();
    let password_hash = data.passwordHash.clone();

    let mut verification_successful: bool = false;

    let panel_users: Vec<PathBuf> = get_panel_users_file_names();

    for user in panel_users {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("PanelUsers")
            .join(user);

        let file_content = fs::read_to_string(path).unwrap();

        let user: PanelUser = serde_json::from_str(&file_content).unwrap();

        if username == user.username && password_hash == user.passwordHash {
            verification_successful = true;
        }
        
    }
    
    if verification_successful {

        cookies.add_private(
                Cookie::build(("session", "valid_session_cookie"))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax)
                    .max_age(cookie::time::Duration::seconds(60 * 60)) // i know that this is complete bs
        );

        cookies.add_private(
            Cookie::build(("doer", username.clone()))
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(SameSite::Lax)
                .max_age(cookie::time::Duration::seconds(60 * 60))
        );

        log(format!("Access Granted"), username);
        return Status::Ok;
    }
    else {

        log(format!("Access Denied"), username);
        Status::Unauthorized
    }
}

#[get("/users-data")]
fn users_data(cookies: &CookieJar<'_>) -> Result<Json<Vec<User>>, Status> {

    let session = match cookies.get_private("session") {
           Some(cookie) => cookie.value().to_string(),
           None => return Err(Status::Unauthorized),
    };
    
    if session != "valid_session_cookie" {
       return Err(Status::Unauthorized);
    }

    
    let mut users = Vec::new();
    let users_file_names = get_users_file_names();

    for user in users_file_names {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("Database")
            .join("Users")
            .join(user);

        let file_content = fs::read_to_string(path).unwrap();

        let file_content_as_json: User = serde_json::from_str(&file_content).unwrap();
        
        users.push(file_content_as_json);
    }

    Ok(Json(users))
}

#[put("/add-user/<id>", data="<user>")]
async fn add_user(id: String, user: Json<User>, cookies: &CookieJar<'_>) -> Status {

    let session = match cookies.get_private("session") {
           Some(cookie) => cookie.value().to_string(),
           None => return Status::Unauthorized,
    };
    
    if session != "valid_session_cookie" {
       return Status::Unauthorized;
    }
    
    let path: String = format!("{}/../Database/Users/{}.json", env!("CARGO_MANIFEST_DIR"), id);

    let json = serde_json::to_string_pretty(&*user).unwrap();

    fs::write(path, json).unwrap();

    log(format!("Added user: {id}"), get_doer(cookies));

    return Status::Ok;
}

#[delete("/delete-user/<id>")]
async fn delete_user(id: u32, cookies: &CookieJar<'_>) -> Status{
    let session = match cookies.get_private("session") {
           Some(cookie) => cookie.value().to_string(),
           None => return Status::Unauthorized,
    };
    
    if session != "valid_session_cookie" {
       return Status::Unauthorized;
    }
    
    fs::remove_file(format!("{}/../Database/Users/{}.json", env!("CARGO_MANIFEST_DIR"), id)).unwrap();

    log(format!("Deleted user: {id}"), get_doer(cookies));

    return Status::Ok;
}

#[post("/edit-user/<id>", data="<user>")]
async fn edit_user(id: u32, user: Json<User>, cookies: &CookieJar<'_>) -> Status{

    let session = match cookies.get_private("session") {
           Some(cookie) => cookie.value().to_string(),
           None => return Status::Unauthorized,
    };
    
    if session != "valid_session_cookie" {
       return Status::Unauthorized;
    }
    
    let path: String = format!("{}/../Database/Users/{}.json", env!("CARGO_MANIFEST_DIR"), id);

    let json = serde_json::to_string_pretty(&*user).unwrap();

    fs::write(path, json).unwrap();

    log(format!("Edited user: {id}"), get_doer(cookies));

    return Status::Ok;
}

#[launch]
fn rocket() -> _ {
    log(String::from("Backend launching :)"), String::from("system"));
    
    // this handels cors autorization (it took me 5 hours to fix this, frick this shit)
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::some_exact(&[
            "https://127.0.0.1:8080",
        ]),
        allowed_headers: AllowedHeaders::some(&[
            "Accept",
            "Authorization",
            "Content-Type",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .attach(cors)
        .mount("/", routes![get_auth_state])
        .mount("/", routes![verify_login])
        .mount("/", routes![users_data])
        .mount("/", routes![add_user])
        .mount("/", routes![delete_user])
        .mount("/", routes![edit_user])
}

fn log(event: String, doer: String) {
    let format_path = format!("{}/Logs/log.json", env!("CARGO_MANIFEST_DIR"));
    let path = format_path.as_str();
    let dt_string = Local::now().to_string();
    let dt = dt_string.as_str();

    let mut data: Value = if std::path::Path::new(path).exists() {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    } else {
        json!({})
    };

    let log_name = format!("log_{}", dt);

    data[log_name.as_str()] = json!({
        "datetime": dt,
        "event": event,
        "doer": doer
    
    });

    fs::write(path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
}

fn get_doer(cookies: &CookieJar<'_>) -> String {
    let session = match cookies.get_private("doer") {
           Some(cookie) => cookie.value().to_string(),
           None => return String::from("")
    };

    session
}

fn get_users_file_names() -> Vec<PathBuf> {
    let path = format!("{}/../Database/Users/", env!("CARGO_MANIFEST_DIR"));
    let users_dir_path = Path::new(&path);
    let dir_entries = fs::read_dir(users_dir_path).expect("Failed to open directory!");
    let mut users_file_names = Vec::new();

    for entry in dir_entries {
        match entry {
            Ok(entry) => users_file_names.push(entry.file_name().into()),
            Err(error) => eprintln!("Failed to read entry: {}", error),
        }
    }

    users_file_names
}

fn get_panel_users_file_names() -> Vec<PathBuf> {
    let path = format!("{}/PanelUsers", env!("CARGO_MANIFEST_DIR"));
    let users_dir_path = Path::new(&path);
    let dir_entries = fs::read_dir(users_dir_path).expect("Failed to open directory!");
    let mut users_file_names = Vec::new();

    for entry in dir_entries {
        match entry {
            Ok(entry) => users_file_names.push(entry.file_name().into()),
            Err(error) => eprintln!("Failed to read entry: {}", error),
        }
    }

    users_file_names
}

