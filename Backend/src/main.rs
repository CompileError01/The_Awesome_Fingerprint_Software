// needed by rocket
#[macro_use]
extern crate rocket;
use rocket::http::Status;
use rocket::{data, launch};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::http::{Cookie, CookieJar};

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


#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(key: &str) -> bool {
            key == "valid_api_key"
        }

        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid)),
        }
    }
}

#[post("/verify-login", data="<data>")]
fn verify_login(data: Json<PanelUser>, cookies: &CookieJar<'_>) -> Status {

    cookies.add(
        Cookie::build(("session", "abc123"))
            .path("/")
            .http_only(true)
    );
    
    let username = data.username.clone();
    let password_hash = data.passwordHash.clone();

    let mut verification_successfull: bool = false;

    let panel_users: Vec<PathBuf> = get_panel_users_file_names();

    for user in panel_users {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("PanelUsers")
            .join(user);

        let file_content = fs::read_to_string(path).unwrap();

        let user: PanelUser = serde_json::from_str(&file_content).unwrap();

        if username == user.username && password_hash == user.passwordHash {
            verification_successfull = true;
        }
        
    }
    
    if verification_successfull {
        
        return Status::Ok;
    }
    else {
        Status::Unauthorized
    }
}

#[get("/users-data")]
fn users_data(_key: ApiKey<'_>) -> Json<Vec<User>> {
    
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

    Json(users)
}

#[put("/add-user/<id>", data="<user>")]
async fn add_user(_key: ApiKey<'_>, id: String, user: Json<User>) {
    let path: String = format!("{}/../Database/Users/{}.json", env!("CARGO_MANIFEST_DIR"), id);

    let json = serde_json::to_string_pretty(&*user).unwrap();

    fs::write(path, json).unwrap();
}

#[delete("/delete-user/<id>")]
async fn delete_user(_key: ApiKey<'_>, id: u32)
{
    fs::remove_file(format!("{}/../Database/Users/{}.json", env!("CARGO_MANIFEST_DIR"), id)).unwrap();
}

#[post("/edit-user/<id>", data="<user>")]
async fn edit_user(_key: ApiKey<'_>, id: u32, user: Json<User>)
{
    let path: String = format!("{}/../Database/Users/{}.json", env!("CARGO_MANIFEST_DIR"), id);

    let json = serde_json::to_string_pretty(&*user).unwrap();

    fs::write(path, json).unwrap();
}

#[launch]
fn rocket() -> _ {
    log("Backend launched", "nA");
    
    // this handels cors autorization (it took me 5 hours to fix this, frick this shit)
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_headers: AllowedHeaders::some(&[
            "Accept",
            "Authorization",
            "Content-Type",
            "x-api-key",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .unwrap();

    rocket::build()
        .attach(cors)
        .mount("/", routes![verify_login])
        .mount("/", routes![users_data])
        .mount("/", routes![add_user])
        .mount("/", routes![delete_user])
        .mount("/", routes![edit_user])
}

fn log(event: &str, doer: &str) {
    let format_path = format!("{}/Logs/log.json", env!("CARGO_MANIFEST_DIR"));
    let path = format_path.as_str();
    let dt_string = Local::now().to_string();
    let dt = dt_string.as_str();

    let mut data: Value = if std::path::Path::new(path).exists() {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    } else {
        json!({})
    };

    data["log01"] = json!({
        "datetime": dt,
        "event": event,
        "doer": doer
    
    });

    fs::write(path, serde_json::to_string_pretty(&data).unwrap()).unwrap();
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

