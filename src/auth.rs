use crate::config::AuthenticationConfig;
use dashmap::DashMap;
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs::File;
use std::io::{Error as IOError, Write};
use std::path::Path;
use uuid::Uuid;
use serde::Deserialize;

#[derive(Clone)]
pub struct AuthCache(pub DashMap<Uuid, i32>);

impl AuthCache {
    pub fn build() -> AuthCache {
        AuthCache(DashMap::new())
    }
}

pub fn generate_key(config: &AuthenticationConfig) -> Result<[u8; 4096], IOError> {
    // create buffer and fill with random data
    let mut key_buffer: [u8; 4096] = [0; 4096];
    OsRng.fill_bytes(&mut key_buffer);

    // create key file according to ApplicationConfig Policy
    let path = Path::new(&config.file);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `key` string to `file`, returns `io::Result<()>`
    match file.write_all(&key_buffer) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }

    Ok(key_buffer)
}

#[derive(Clone, Deserialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub game: i32,
    pub name: String,
}

impl User {
    pub fn with_name(game: &i32, name: &String) -> User {
        User {
            id: Uuid::new_v4(),
            game: *game,
            name: name.clone(),
        }
    }
}
