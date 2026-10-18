use std::{env, process::ExitCode};

use sqlx::postgres::PgPoolOptions;
use vitarchy_core::{AppState, telemetry};
use vitarchy_server::{app, config::Config, migrate, openapi, registry, repair, spa, startup_log};

const USAGE: &str = "usage: vitarchy-server [openapi | finance-merge-duplicates]\n  (no argument)             run the HTTP server\n  openapi                   print the OpenAPI document to stdout\n  finance-merge-duplicates  merge Finance Accounts with the same issuer, kind and digits";

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
        ["finance-merge-duplicates"] => match merge_duplicates().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
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

/// Loads `.env` (live runs only) and starts logging.
fn init_environment() {
    // Stage runs (scripts/with-env.sh sets VITARCHY_STAGE) must never pick up the
    // live `.env`, even for variables the stage file leaves out.
    if env::var_os("VITARCHY_STAGE").is_none() {
        dotenvy::dotenv().ok();
    }
    telemetry::init();
}

async fn merge_duplicates() -> Result<(), Box<dyn std::error::Error>> {
    init_environment();
    let report = repair::finance_merge_duplicates(&Config::from_env()?).await?;
    println!(
        "merged {} Accounts in {} groups: {} Transactions moved, {} folded into others ({} Printings moved)",
        report.accounts_merged, report.groups, report.moved, report.folded, report.printings
    );
    Ok(())
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    init_environment();
    let config = Config::from_env()?;
    let (core, modules) = (registry::core(), registry::modules());
    startup_log::log(&config, &core, &modules);
    if let Some(dist) = &config.web_dist {
        spa::ensure_built(dist)?;
    }

    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_lazy(&config.database_url)?;
    let modules: Vec<_> = core.into_iter().chain(modules).collect();
    migrate::run(&pool, &modules).await?;

    let router = app::build(AppState::new(pool), modules, config.web_dist.as_deref());
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
