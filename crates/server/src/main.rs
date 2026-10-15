use std::process::ExitCode;

use sqlx::postgres::PgPoolOptions;
use vitarchy_core::{AppState, telemetry};
use vitarchy_server::{app, config::Config, migrate, openapi, registry};

const USAGE: &str = "usage: vitarchy-server [openapi]\n  (no argument)  run the HTTP server\n  openapi        print the OpenAPI document to stdout";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["openapi"] => print_openapi(),
        [] => match serve().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn print_openapi() -> ExitCode {
    match openapi::spec(registry::all()).to_pretty_json() {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    telemetry::init();
    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy(&config.database_url)?;
    let modules = registry::all();
    migrate::run(&pool, &modules).await?;

    let router = app::build(AppState::new(pool), modules);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
