// imports
use crate::config::{CONFIG, SECRET_KEY};
use crate::frontend::helper::{log_info, log_success};
use crate::frontend::routes;
use crate::state::{AppState, GameServerState};
use crate::ws::{actor::GameServer, routes as ws_routes};
use actix::Actor;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    http::ContentEncoding, middleware::Compress, middleware::DefaultHeaders, web, App, HttpServer,
};
use actix_web_static_files;
use futures::executor;
use std::env::{set_var, var};
use std::io::Result;
use std::{sync::mpsc, sync::Arc, thread};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[actix_web::main]
pub async fn main() -> Result<()> {
    // initial log
    log_info("SERVER", "Starting Server Initialization".to_owned());

    // logging
    if var("RUST_LOG").is_err() {
        set_var("RUST_LOG", "actix_web=info");
        log_info("SERVER", "actix logging set to info".to_owned());
    }

    // create stopper channel
    let (tx, rx) = mpsc::channel::<()>();

    // evaluate host
    let host = match CONFIG.server.port {
        Some(port) => format!("{}:{}", CONFIG.server.ip, port),
        None => format!("{}:8080", CONFIG.server.ip),
    };

    // get user session length
    let session_length = CONFIG.auth.session.clone();

    // clone host for server bind
    let server_bind = CONFIG.server.ip.clone();

    // initialize actix-web server
    log_success("SERVER", format!("Binding server to http://{}", host));

    // build app state
    let app_state = AppState::build();
    let game_state = GameServerState::build(Arc::clone(&app_state.auth_cache));
    let state = web::Data::new(app_state);

    let server = HttpServer::new(move || {
        {
            App::new()
                .app_data(state.clone())
                .data(tx.clone())
                .wrap(Compress::new(ContentEncoding::Gzip))
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&SECRET_KEY.clone())
                        .name("auth")
                        .path("/")
                        .domain(server_bind.clone())
                        .max_age(3600 * session_length) // config Value from seconds to hours
                        .secure(false), // this can only be true if you have https enabled
                ))
                .service(actix_web_static_files::ResourceFiles::new(
                    "/static",
                    generate(),
                ))
                .wrap(DefaultHeaders::new().header("Cache-Control", "max-age=86400")) // 1 Day
                .service(
                    web::scope("/content")
                        .route("/rules", web::get().to(routes::get_rules))
                        .route("/cookies", web::get().to(routes::get_cookies)),
                )
                .service(
                    web::scope("/games")
                        .service(
                            web::resource("/ws/")
                                .data(GameServer::build().start())
                                .to(ws_routes::game_route),
                        )
                        .route("/join/{id}", web::get().to(routes::get_game_join))
                        .route("/join/{id}", web::post().to(routes::post_game_join))
                        .route("/leave", web::get().to(ws_routes::get_game_leave_route))
                        .route("/", web::get().to(routes::get_game_overview))
                        .route("/create", web::get().to(routes::get_create_game))
                        .route("/create", web::post().to(routes::post_create_game))
                        .route("/view/{id}", web::get().to(routes::get_view_game)),
                )
                .route("/robots.txt", web::get().to(api_routes::get_robots_txt))
                .route("/", web::get().to(routes::get_index))
                .default_service(web::route().to(routes::get_error_404))
        }
    })
    .bind(host)?
    .run();

    // clone the Server handle
    let srv = server.clone();
    thread::spawn(move || {
        // wait for shutdown signal
        rx.recv().unwrap();

        // tear down games
        unimplemented!();

        // stop server gracefully
        executor::block_on(srv.stop(true))
    });

    // run server
    server.await
}
