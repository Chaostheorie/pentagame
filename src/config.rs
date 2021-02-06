/*
Borrowed from https://gitlab.com/C0balt/oxidized-cms
*/

use crate::auth::generate_key;
use crate::frontend::helper::{log_error, log_info, log_success};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::fs::{read_to_string, File};
use std::io::{Error, Read, Write};
use std::path::Path;
use std::process::exit;

// Types
pub type SecretKey = [u8; 4096];

// Constants for default names
pub const DEFAULT_CONFIG_NAME: &str = "pentagame.toml";
pub const DEFAULT_KEY_FILE: &str = "secret.key";
pub const DEFAULT_RATE_LIMIT: usize = 100;

#[derive(Deserialize, Clone, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthenticationConfig,
    pub admin: AdminConfig,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct AuthenticationConfig {
    pub file: String,
    pub salt: String,
    pub session: i64,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct AdminConfig {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct ServerConfig {
    pub ip: String,
    pub port: Option<u32>,
}

impl Config {
    pub fn load_config(config_raw_path: &str) -> Config {
        let config_path = Path::new(config_raw_path);
        if !config_path.exists() {
            log_error(
                "Config Error",
                format!("Config not found at {}", config_raw_path),
            );
            exit(1)
        } else {
            let config_file = match read_to_string(config_path) {
                Ok(content) => content,
                Err(why) => {
                    log_error("Config Error", why.to_string());
                    exit(1)
                }
            };

            match toml::from_str::<Config>(&config_file) {
                Ok(config) => {
                    log_success(
                        "CONFIG",
                        format!("Successfully loaded config from {}", config_raw_path),
                    );
                    config
                }
                Err(why) => {
                    log_error("Config Error", why.to_string());
                    exit(1)
                }
            }
        }
    }

    pub fn dump_config(&self, config_path: &Path) -> Result<(), Error> {
        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(config_path) {
            Err(why) => {
                log_error("Config Error", why.to_string());
                exit(3)
            }
            Ok(file) => file,
        };

        // Write the config (TOML) string to `file`, returns `io::Result<()>`
        match file.write_all(toml::to_string_pretty(self).unwrap().as_bytes()) {
            Err(why) => {
                log_error("Config Error", why.to_string());
                exit(3)
            }
            Ok(_) => log_success(
                "CONFIG",
                format!("successfully wrote config to {}", config_path.display()),
            ),
        }

        Ok(())
    }

    pub fn create_key(&mut self, config_path: &Path) -> Result<SecretKey, Error> {
        let key_path = Path::new(DEFAULT_KEY_FILE);
        let key = generate_key(&self.auth)?;
        self.auth.file = DEFAULT_KEY_FILE.to_owned();
        self.dump_config(config_path)?;

        if key_path.exists() {
            log_info(
                "CONFIG",
                format!(
                    "Default key file '{}' exists. Overwriting key.",
                    DEFAULT_KEY_FILE
                ),
            );
        }

        // creating file
        let mut key_file = match File::create(key_path) {
            Err(why) => {
                log_error("Config Error", why.to_string());
                exit(3)
            }
            Ok(file) => file,
        };

        // Write the key (bytes) to `key_file`, returns `io::Result<()>`
        match key_file.write_all(&key) {
            Err(why) => {
                log_error("Config Error", why.to_string());
                exit(3)
            }
            Ok(_) => log_success(
                "CONFIG",
                format!("successfully wrote key to {}", key_path.display()),
            ),
        }

        Ok(key)
    }

    pub fn load_key(&mut self, config_path: &Path) -> SecretKey {
        // check if new key should be generated
        if self.auth.file == "NEW" {
            log_info(
                "[CONFIG",
                "Auth.file was set to 'NEW' -> Generating new key".to_owned(),
            );
            return self
                .create_key(config_path)
                .expect("Failed to create new secret key");
        }

        // evaluate and check path
        let key_path = Path::new(&self.auth.file);

        if !key_path.exists() {
            {
                log_error("Config Error", "Key file doesn't exist".to_owned());
                exit(1)
            }
        } else {
            // create empty buff
            let mut key_buffer: SecretKey = [0; 4096];

            // read bytes from file
            let mut key_file = match File::open(key_path) {
                Ok(file) => file,
                Err(why) => {
                    log_error("Config Error", why.to_string());
                    exit(1);
                }
            };

            match key_file.read_exact(&mut key_buffer) {
                Ok(_) => (),
                Err(why) => {
                    log_error("Config Error", why.to_string());
                    exit(1);
                }
            }

            key_buffer
        }
    }
}

// Throw the Config struct into a CONFIG lazy_static to avoid multiple processing
// Do the same for SECRET_KEY
// WARNING: This is a workaround for now and the config and key structure should be seperated
//          at some point. I will do this when I'm having too much time or getting money for this
lazy_static! {
    pub static ref CONFIG: Config =
        Config::load_config(&var("CONFIG").unwrap_or(DEFAULT_CONFIG_NAME.to_owned()));
    pub static ref SECRET_KEY: SecretKey = CONFIG.clone().load_key(Path::new(DEFAULT_KEY_FILE));
}
