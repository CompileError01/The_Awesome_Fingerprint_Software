// needed by rocket
#[macro_use]
extern crate rocket;
use rocket::http::Status;
use rocket::launch;
use rocket::request::{FromRequest, Outcome, Request};

// CORS
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

// for PathBuffs
use std::{fs, path::Path, path::PathBuf};

// for Json
use serde::{Serialize, Deserialize};
use rocket::serde::json::Json;

struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

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

#[get("/users-data")]
fn users_data(_key: ApiKey<'_>) -> Json<Vec<User>> {
    let mut users = Vec::new();

    let users_file_names = get_users_file_names();

    for user in users_file_names {
        let path = PathBuf::from("/run/media/sleuth/128Gb/FingerprintSystem/Database/Users/")
            .join(user);

        let file_content = fs::read_to_string(path).unwrap();

        let file_content_as_json: User = serde_json::from_str(&file_content).unwrap();
        
        users.push(file_content_as_json);
    }

    Json(users)
}

#[put("/add-user/<name>", data="<user>")]
async fn add_user(name: String, user: Json<User>) {
    let path: String = format!("/run/media/sleuth/128Gb/FingerprintSystem/Database/Users/{name}.json");

    let json = serde_json::to_string_pretty(&*user).unwrap();

    fs::write(path, json).unwrap();
}

#[delete("/delete-user/<id>")]
async fn delete_user(id: u32)
{
    fs::remove_file(format!("/run/media/sleuth/128Gb/FingerprintSystem/Database/Users/{id}.json")).unwrap();
}

#[launch]
fn rocket() -> _ {

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
        .mount("/", routes![users_data])
        .mount("/", routes![add_user])
        .mount("/", routes![delete_user])
}

fn get_users_file_names() -> Vec<PathBuf> {
    let users_dir_path =
        Path::new("/run/media/sleuth/128Gb/FingerprintSystem/Database/Users");
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