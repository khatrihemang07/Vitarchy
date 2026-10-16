//! Throwaway databases for integration tests, on the STAGE Postgres only.
//!
//! Tests connect through `TEST_DATABASE_URL` and nothing else: there is no
//! fallback to `DATABASE_URL` or `.env`, and a URL that points at the live
//! database is refused with a panic. See `docs/development.md` (Environments).

use std::{
    str::FromStr,
    sync::atomic::{AtomicU32, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{Connection, PgConnection, PgPool, postgres::PgConnectOptions};

/// Used only when the live URL cannot be discovered: the shipped live defaults.
const DEFAULT_LIVE_URL: &str = "postgres://vitarchy:vitarchy@127.0.0.1:5433/vitarchy";
/// Env var that `make`/`scripts/with-env.sh` export with the live `DATABASE_URL`.
const LIVE_URL_VAR: &str = "LIVE_DATABASE_URL";
/// Prefix of every database this module creates, so leftovers are recognisable.
const PREFIX: &str = "vitarchy_test_";
/// Leftovers from a crashed run older than this are dropped at the next start.
const STALE_AFTER_SECS: u64 = 3600;
/// Longest label kept in a database name (Postgres identifiers max out at 63).
const LABEL_MAX: usize = 20;

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Where a database URL points: normalised host, port and database name.
#[derive(Debug, PartialEq, Eq)]
pub struct Target {
    host: String,
    /// Postgres port (5432 when the URL names none).
    pub port: u16,
    /// Database name.
    pub db: String,
}

impl Target {
    /// Reads the target out of a `postgres://` URL.
    ///
    /// # Errors
    /// When `url` is not a URL with a host.
    pub fn parse(url: &str) -> Result<Self, String> {
        let parsed = url::Url::parse(url).map_err(|e| e.to_string())?;
        let host = parsed
            .host_str()
            .ok_or("URL has no host")?
            .to_ascii_lowercase();
        // `localhost`, 127.0.0.1 and ::1 are the same machine, so they compare equal.
        let host = match host.as_str() {
            "localhost" | "127.0.0.1" | "::1" | "[::1]" => "loopback".to_owned(),
            _ => host,
        };
        Ok(Self {
            host,
            port: parsed.port().unwrap_or(5432),
            db: parsed.path().trim_start_matches('/').to_owned(),
        })
    }
}

/// Value of `key` in dotenv-style `text`: first uncommented `KEY=value` line,
/// with spaces and one pair of quotes stripped.
fn dotenv_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (k, v) = line.trim().split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches(['"', '\'']).to_owned())
    })
}

/// The live target: `live_var` (the `LIVE_DATABASE_URL` env var) first, then
/// `DATABASE_URL` in `dotenv` (the root `.env`), then the shipped defaults.
/// An unparseable source is skipped, so the result is always conservative.
fn live_target(live_var: Option<&str>, dotenv: Option<&str>) -> Target {
    let from_dotenv = dotenv.and_then(|t| dotenv_value(t, "DATABASE_URL"));
    [live_var.map(str::to_owned), from_dotenv]
        .into_iter()
        .flatten()
        .find_map(|url| Target::parse(&url).ok())
        .unwrap_or_else(|| Target::parse(DEFAULT_LIVE_URL).expect("default live URL is valid"))
}

/// Rejects `url` if it shares a server (host and port) with `live`, or names
/// the live database anywhere: tests create databases on the server they use.
///
/// # Errors
/// A human-readable reason.
pub fn check_against(url: &str, live: &Target) -> Result<PgConnectOptions, String> {
    let opts = PgConnectOptions::from_str(url)
        .map_err(|e| format!("TEST_DATABASE_URL is not a valid Postgres URL: {e}"))?;
    let target = Target::parse(url).map_err(|e| format!("TEST_DATABASE_URL is invalid: {e}"))?;
    if target.host == live.host && target.port == live.port {
        return Err(format!(
            "TEST_DATABASE_URL uses the same server (port {}) as the LIVE database",
            live.port
        ));
    }
    if target.db == live.db {
        return Err(format!(
            "TEST_DATABASE_URL names the database `{}`, which is the LIVE database",
            live.db
        ));
    }
    Ok(opts)
}

/// Parses `url` and rejects anything that points at the live database. The
/// live URL comes from `LIVE_DATABASE_URL`, else `.env` at the workspace root,
/// else the shipped defaults (port 5433, database `vitarchy`).
///
/// # Errors
/// Returns a human-readable reason when `url` is unparseable or live.
pub fn check_not_live(url: &str) -> Result<PgConnectOptions, String> {
    let dotenv = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.env")).ok();
    let live_var = std::env::var(LIVE_URL_VAR).ok();
    check_against(url, &live_target(live_var.as_deref(), dotenv.as_deref()))
}

/// Connection options for the stage server, from `TEST_DATABASE_URL`.
///
/// # Panics
/// When the variable is unset or points at the live database: a test must
/// never run against real data.
#[must_use]
pub fn stage_options() -> PgConnectOptions {
    let url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        panic!(
            "TEST_DATABASE_URL must be set; run tests with `make test` (it uses the stage database)"
        )
    });
    check_not_live(&url).unwrap_or_else(|reason| panic!("refusing to run tests: {reason}"))
}

/// A uniquely named database that exists for the life of one test.
pub struct TestDb {
    name: String,
    /// Pool connected to the throwaway database.
    pub pool: PgPool,
}

impl TestDb {
    /// Creates an empty database named after `label` and connects to it.
    /// Also sweeps databases left behind by crashed runs.
    ///
    /// # Panics
    /// When stage is unreachable or `TEST_DATABASE_URL` is unsafe.
    pub async fn create(label: &str) -> Self {
        let base = stage_options();
        let mut admin = PgConnection::connect_with(&base)
            .await
            .expect("connect to stage Postgres (is `make stage-db` running?)");
        sweep_stale(&mut admin).await;
        let name = unique_name(label);
        exec(&mut admin, format!("CREATE DATABASE {name}")).await;
        let pool = PgPool::connect_with(base.database(&name))
            .await
            .expect("connect to test database");
        Self { name, pool }
    }

    /// Closes the pool and drops the database.
    ///
    /// # Panics
    /// When stage is unreachable.
    pub async fn drop(self) {
        self.pool.close().await;
        let mut admin = PgConnection::connect_with(&stage_options())
            .await
            .expect("connect to stage Postgres");
        exec(
            &mut admin,
            format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", self.name),
        )
        .await;
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// `vitarchy_test_<unix secs>_<pid>_<n>_<label>`; the timestamp drives the sweep.
fn unique_name(label: &str) -> String {
    let label: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .take(LABEL_MAX)
        .collect();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{PREFIX}{}_{}_{n}_{label}", now_secs(), std::process::id())
}

/// The creation time encoded in a database name this module made.
fn created_at(name: &str) -> Option<u64> {
    let mut parts = name.strip_prefix(PREFIX)?.split('_');
    let secs = parts.next()?.parse().ok()?;
    // A real name has pid and counter after the timestamp.
    parts.nth(1)?;
    Some(secs)
}

async fn sweep_stale(admin: &mut PgConnection) {
    let names: Vec<String> = sqlx::query_scalar(
        "SELECT datname FROM pg_database WHERE datname LIKE 'vitarchy\\_test\\_%'",
    )
    .fetch_all(&mut *admin)
    .await
    .expect("list test databases");
    for name in names {
        if created_at(&name).is_some_and(|t| now_secs().saturating_sub(t) > STALE_AFTER_SECS) {
            exec(
                admin,
                format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"),
            )
            .await;
        }
    }
}

/// Runs a statement whose text is built here from sanitised identifiers only.
async fn exec(conn: &mut PgConnection, sql: String) {
    sqlx::query(sqlx::AssertSqlSafe(sql))
        .execute(conn)
        .await
        .expect("database statement");
}

#[cfg(test)]
mod tests {
    use super::{Target, check_against, created_at, dotenv_value, live_target, unique_name};

    fn live(url: &str) -> Target {
        Target::parse(url).unwrap()
    }

    /// The shipped defaults, independent of whatever `.env` this machine has.
    fn check_default(url: &str) -> Result<(), String> {
        check_against(url, &live_target(None, None)).map(|_| ())
    }

    #[test]
    fn refuses_the_live_port() {
        let err = check_default("postgres://u:p@127.0.0.1:5433/anything").unwrap_err();
        assert!(err.contains("LIVE"), "{err}");
    }

    #[test]
    fn refuses_the_live_database_name_on_any_port() {
        assert!(check_default("postgres://u:p@127.0.0.1:5999/vitarchy").is_err());
    }

    #[test]
    fn accepts_the_stage_database() {
        assert!(check_default("postgres://u:p@127.0.0.1:5434/vitarchy_stage").is_ok());
    }

    #[test]
    fn rejects_garbage() {
        assert!(check_default("not a url").is_err());
    }

    #[test]
    fn localhost_and_loopback_ips_are_the_same_host() {
        let l = live("postgres://u:p@127.0.0.1:5500/real");
        for host in ["localhost", "127.0.0.1", "[::1]", "LOCALHOST"] {
            let url = format!("postgres://u:p@{host}:5500/other");
            assert!(check_against(&url, &l).is_err(), "{host}");
        }
    }

    #[test]
    fn a_custom_live_url_is_refused_by_server_and_by_database_name() {
        let l = live("postgres://u:p@localhost:5500/real");
        assert!(check_against("postgres://u:p@127.0.0.1:5500/any", &l).is_err());
        assert!(check_against("postgres://u:p@127.0.0.1:5999/real", &l).is_err());
        assert!(check_against("postgres://u:p@127.0.0.1:5434/vitarchy_stage", &l).is_ok());
    }

    #[test]
    fn a_missing_port_means_5432() {
        let l = live("postgres://u@localhost/real");
        assert!(check_against("postgres://u@127.0.0.1:5432/x", &l).is_err());
    }

    #[test]
    fn live_target_prefers_the_env_var_over_dotenv() {
        let t = live_target(
            Some("postgres://u:p@127.0.0.1:6000/a"),
            Some("DATABASE_URL=postgres://u:p@127.0.0.1:6001/b"),
        );
        assert_eq!((t.port, t.db.as_str()), (6000, "a"));
    }

    #[test]
    fn live_target_falls_back_to_dotenv_then_to_the_default() {
        let t = live_target(
            None,
            Some("A=1\nDATABASE_URL=postgres://u:p@localhost:6001/b\n"),
        );
        assert_eq!((t.port, t.db.as_str()), (6001, "b"));
        let t = live_target(None, None);
        assert_eq!((t.port, t.db.as_str()), (5433, "vitarchy"));
        let t = live_target(Some("garbage"), Some("no url here"));
        assert_eq!((t.port, t.db.as_str()), (5433, "vitarchy"));
    }

    #[test]
    fn dotenv_value_handles_comments_quotes_and_spacing() {
        let text = "# DATABASE_URL=nope\nOTHER=1\n  DATABASE_URL = \"postgres://x/y\" \n";
        assert_eq!(
            dotenv_value(text, "DATABASE_URL").as_deref(),
            Some("postgres://x/y")
        );
        assert_eq!(dotenv_value(text, "MISSING"), None);
    }

    #[test]
    fn names_encode_their_creation_time_and_stay_valid() {
        let name = unique_name("Weird Label!!");
        assert!(created_at(&name).is_some(), "{name}");
        assert!(
            name.len() <= 63 && name.ends_with("weird_label__"),
            "{name}"
        );
        assert_ne!(name, unique_name("Weird Label!!"));
    }

    #[test]
    fn foreign_names_have_no_creation_time() {
        assert_eq!(created_at("vitarchy_stage"), None);
        assert_eq!(created_at("vitarchy_test_x_1"), None);
    }
}
