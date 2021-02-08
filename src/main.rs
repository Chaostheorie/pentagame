// includes
mod auth;
mod config;
mod frontend;
mod graph;
mod server;
mod state;
mod ws;
mod db;

// imports
use crate::config::DEFAULT_CONFIG_NAME;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::env::{set_var, var};
use std::path::Path;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;

pub fn main() -> std::io::Result<()> {
    let matches: ArgMatches = App::new("pentagame online 2")
        .author("Cobalt <cobalt.rocks>")
        .long_about("pentagame online - Copyright (C) 2020 Cobalt under GPLv3.0")
        .version("0.0.1")
        .subcommand(SubCommand::with_name("license").about("Shows short license"))
        .subcommand(
            SubCommand::with_name("serve")
                .about("serve pentagame online server")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .default_value(DEFAULT_CONFIG_NAME)
                        .value_name("CONFIG")
                        .help(
                            "custom config file. You may set the CONFIG env variable instead too.",
                        )
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about("generate new session key")
                .arg(
                    Arg::with_name("file")
                        .short("f")
                        .default_value(config::DEFAULT_KEY_FILE)
                        .long("secret-file")
                        .value_name("FILE")
                        .help("Set a custom output file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .default_value(DEFAULT_CONFIG_NAME)
                        .value_name("CONFIG")
                        .help(
                            "custom config file. You may set the CONFIG env variable instead too.",
                        )
                        .takes_value(true),
                ),
        )
        .get_matches();

    // license
    if matches.subcommand_matches("license").is_some() {
        println!(
            "pentagame online  Copyright (C) 2020  Cobalt under GPLv3.0\nThis program comes with ABSOLUTELY NO WARRANTY\nThis is free software, and you are welcome to redistribute it\nunder certain conditions; See LICENSE for details"
        );
    }

    // read config from 'cms.toml' and evaluate host
    match matches.subcommand_matches("serve") {
        Some(_) => {
            let res = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_NAME); // double fallback can never hurt

            // this only works due to lazy static characteristics. (being evaluated on first use)
            // Since server.rs is the first one to import CONFIG the evaluation will be done after this call
            set_var("CONFIG", res);
            server::main()?
        }
        None => (),
    };

    match matches.subcommand_matches("generate") {
        Some(subcommand_matches) => {
            let config_raw_path = match matches.value_of("config") {
                Some(path) => path.to_owned(),
                None => {
                    let env_var = var("CONFIG");
                    if env_var.is_err() {
                        DEFAULT_CONFIG_NAME.to_owned()
                    } else {
                        env_var.unwrap()
                    }
                }
            };

            let config_path = Path::new(&config_raw_path);
            let mut config = config::Config::load_config(&config_raw_path);
            config.auth.file = subcommand_matches.value_of("file").unwrap().to_owned();
            config.dump_config(&config_path)?;
            auth::generate_key(&config.auth)?;
        }
        None => (),
    };

    Ok(())
}
