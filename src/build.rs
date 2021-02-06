use actix_web_static_files::NpmBuild;
use std::env;

fn main() {
    let production = env::var("ASSET_ENVIRONMENT")
        .map(|v| v == "PRODUCTION")
        .unwrap_or(true); // run by default

    // NOTE: logging to static/compile.log

    if production {
        NpmBuild::new("./static")
            .install()
            .unwrap()
            .run("compile-production")
            .unwrap()
            .target("./static/dist")
            .to_resource_dir()
            .build()
            .unwrap();
    } else {
        NpmBuild::new("./static")
            .install()
            .unwrap()
            .run("compile")
            .unwrap()
            .target("./static/dist")
            .to_resource_dir()
            .build()
            .unwrap();
    }
}
