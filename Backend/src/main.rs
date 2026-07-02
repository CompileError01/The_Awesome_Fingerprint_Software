// needed by rocket
use rocket::http::Status;
use rocket::launch;
use rocket::request::{Outcome, Request, FromRequest};
#[macro_use] extern crate rocket;
// for PathBuffs
use std::{fs, path::Path, path::PathBuf};

// for json
use serde::{Deserialize, Serialize};

struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {

        fn is_valid(key: &str) -> bool {
            key == "valid_api_key" // REPLACE WITH PROPER API KEY!!!
        }

        match req.headers().get_one("x-api-key") {
            None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Error((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

#[get("/users-data")]
fn users_data(key: ApiKey<'_>) -> String {
    let mut return_string: String = String::new();
    
    let users_file_names = get_users_file_names();

    for user in users_file_names {
        let path = PathBuf::from("/run/media/sleuth/128Gb/FingerprintSystem/Database/Users/").join(user);
        
        let file_content = fs::read_to_string(path).unwrap(); 
        return_string.push_str(file_content.as_str());
    }

    return_string
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![users_data])
}


fn get_users_file_names() -> Vec<PathBuf> {
let users_dir_path = Path::new("/run/media/sleuth/128Gb/FingerprintSystem/Database/Users");
let dir_entries = fs::read_dir(users_dir_path).expect("Failed to open directory!");
let mut users_file_names: Vec<PathBuf> = Vec::new();

for entry in dir_entries {
    match entry {
        Ok(entry) => {
            users_file_names.push(entry.file_name().into());
        }

        Err(error) => {
            eprintln!("Failed to read entry: {}", error);
        }
    }
}
users_file_names
}