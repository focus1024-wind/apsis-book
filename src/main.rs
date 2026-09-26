//! The `apsis-book` service binary.
//!
//! The process binds an HTTP/1.1 listener and answers requests from three
//! sources of content that are all embedded in the binary:
//!
//! - the concept documentation and the knowledge graph data, read from the
//!   `content/` directory at compile time;
//! - the interface documentation of the `apsis-rs` orbit layer, extracted
//!   from its source doc comments and embedded as `content/api.json`;
//! - the frontend build output under `frontend/dist/`, embedded by
//!   `build.rs`.
//!
//! Because every asset is embedded, the binary is self-contained: it serves
//! the same content from a copy that has no source tree next to it.

mod api;
mod docs;
mod embedded;
mod graph;
mod http;
mod json;
mod state;
mod tools;

use std::process::ExitCode;
use std::sync::Arc;

use crate::api::Router;
use crate::http::{Server, ServerConfig};
use crate::state::AppState;

/// Default bind address.
const DEFAULT_ADDRESS: &str = "127.0.0.1:8080";

/// Entry point.
fn main() -> ExitCode {
    let options = match Options::from_environment() {
        Ok(options) => options,
        Err(message) => {
            eprintln!("apsis-book: {message}");
            return ExitCode::FAILURE;
        }
    };

    let state = Arc::new(AppState::new());
    let router = Arc::new(Router::new(state));
    let config = ServerConfig {
        workers: options.workers,
        ..ServerConfig::default()
    };

    let server = match Server::bind(&options.address, config) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("apsis-book: cannot bind {}: {error}", options.address);
            return ExitCode::FAILURE;
        }
    };

    let bound = server
        .local_address()
        .map(|address| address.to_string())
        .unwrap_or_else(|_| options.address.clone());
    println!("apsis-book listening on http://{bound}");
    if !embedded::is_populated() {
        println!("apsis-book: no frontend build is embedded; serving the placeholder page");
    }

    match server.run(router) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("apsis-book: server stopped: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Command line options, read from arguments and then from the environment.
struct Options {
    /// Address to bind, in `host:port` form.
    address: String,
    /// Number of worker threads.
    workers: usize,
}

impl Options {
    /// Builds the options from the process arguments.
    ///
    /// Arguments override the environment variables `APSIS_BOOK_ADDRESS` and
    /// `APSIS_BOOK_WORKERS`. `--help` and `--version` print and exit.
    fn from_environment() -> Result<Self, String> {
        let mut options = Self {
            address: std::env::var("APSIS_BOOK_ADDRESS")
                .unwrap_or_else(|_| DEFAULT_ADDRESS.to_string()),
            workers: match std::env::var("APSIS_BOOK_WORKERS") {
                Ok(value) => value
                    .parse()
                    .map_err(|_| "APSIS_BOOK_WORKERS must be a positive integer".to_string())?,
                Err(_) => 4,
            },
        };
        if options.workers == 0 {
            return Err("APSIS_BOOK_WORKERS must be greater than zero".to_string());
        }

        let mut arguments = std::env::args().skip(1);
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--address" | "-a" => {
                    options.address = arguments
                        .next()
                        .ok_or_else(|| "--address needs a value".to_string())?;
                }
                "--workers" | "-w" => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "--workers needs a value".to_string())?;
                    options.workers = value
                        .parse()
                        .map_err(|_| "--workers needs a positive integer".to_string())?;
                }
                "--help" | "-h" => {
                    println!("{}", USAGE);
                    std::process::exit(0);
                }
                "--version" | "-V" => {
                    println!("apsis-book {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument {other:?}; try --help")),
            }
        }
        if options.workers == 0 {
            return Err("--workers must be greater than zero".to_string());
        }
        Ok(options)
    }
}

/// Usage text.
const USAGE: &str = "\
apsis-book: web service for the ApsisRS computational kernels

USAGE:
    apsis-book [OPTIONS]

OPTIONS:
    -a, --address <HOST:PORT>    address to bind (default 127.0.0.1:8080)
    -w, --workers <N>            worker threads (default 4)
    -h, --help                   print this help
    -V, --version                print the version

ENVIRONMENT:
    APSIS_BOOK_ADDRESS            bind address
    APSIS_BOOK_WORKERS            worker threads";
