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
//!
//! The `--host` and `--port` options bind the listener, and the language of
//! the user interface is selected in the browser and defaults to Chinese.
//!
//! `apsis-book` 服务的二进制。
//!
//! 该进程绑定一个 HTTP/1.1 监听器，并以内嵌在二进制中的三种内容来源回应请求：
//!
//! - 概念文档与知识图谱数据，在编译期从 `content/` 目录读入；
//! - `apsis-rs` 各层的接口文档，从其源码文档注释提取，并作为
//!   `content/api.json` 内嵌；
//! - `frontend/dist/` 下的前端构建产物，由 `build.rs` 内嵌。
//!
//! 由于每一项资源都已内嵌，二进制是自足的：把它复制到旁边没有源码树的地方，它
//! 提供的内容完全相同。
//!
//! `--host` 与 `--port` 选项用于绑定监听器；用户界面的语言在浏览器中选择，
//! 默认为中文。

use apsis_book::api::Router;
use apsis_book::embedded;
use apsis_book::http::{Server, ServerConfig};
use apsis_book::state::AppState;

use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;

/// Default bind host, which accepts connections from the local machine
/// only.
///
/// 默认绑定的主机，只接受来自本机的连接。
const DEFAULT_HOST: &str = "127.0.0.1";

/// Default bind port.
/// 默认绑定的端口。
const DEFAULT_PORT: u16 = 8080;

/// Entry point.
/// 入口。
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

    let server = match Server::bind_address(options.address, config) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("apsis-book: cannot bind {}: {error}", options.address);
            return ExitCode::FAILURE;
        }
    };

    let bound = server
        .local_address()
        .map(|address| address.to_string())
        .unwrap_or_else(|_| options.address.to_string());
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
///
/// 命令行选项，先读参数，再读环境变量。
struct Options {
    /// Address to bind.
    /// 要绑定的地址。
    address: SocketAddr,
    /// Number of worker threads.
    /// 工作线程数。
    workers: usize,
}

impl Options {
    /// Builds the options from the process arguments.
    ///
    /// Arguments override the environment variables `APSIS_BOOK_HOST`,
    /// `APSIS_BOOK_PORT`, `APSIS_BOOK_ADDRESS`, and `APSIS_BOOK_WORKERS`.
    /// `--help` and `--version` print and exit.
    ///
    /// The host and the port may be given separately with `--host` and
    /// `--port`, or together with `--address`. Giving both `--address` and
    /// either of the two separate options is an error, because the order in
    /// which they would apply is not defined.
    ///
    /// 从进程参数构造选项。
    ///
    /// 参数优先于环境变量 `APSIS_BOOK_HOST`、`APSIS_BOOK_PORT`、
    /// `APSIS_BOOK_ADDRESS` 与 `APSIS_BOOK_WORKERS`。`--help` 与 `--version`
    /// 打印后退出。
    ///
    /// 主机与端口可以用 `--host` 与 `--port` 分别给出，也可以用 `--address`
    /// 一并给出。同时给出 `--address` 与那两个选项中的任何一个都是错误，因为
    /// 二者生效的先后顺序没有定义。
    fn from_environment() -> Result<Self, String> {
        let mut host =
            std::env::var("APSIS_BOOK_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
        let mut port = match std::env::var("APSIS_BOOK_PORT") {
            Ok(value) => parse_port(&value).map_err(|error| format!("APSIS_BOOK_PORT: {error}"))?,
            Err(_) => DEFAULT_PORT,
        };
        let mut combined: Option<SocketAddr> = match std::env::var("APSIS_BOOK_ADDRESS") {
            Ok(value) => Some(
                parse_address(&value).map_err(|error| format!("APSIS_BOOK_ADDRESS: {error}"))?,
            ),
            Err(_) => None,
        };
        let mut workers = match std::env::var("APSIS_BOOK_WORKERS") {
            Ok(value) => value
                .parse()
                .map_err(|_| "APSIS_BOOK_WORKERS must be a positive integer".to_string())?,
            Err(_) => 4,
        };
        if workers == 0 {
            return Err("APSIS_BOOK_WORKERS must be greater than zero".to_string());
        }

        let mut host_given = false;
        let mut port_given = false;

        let mut arguments = std::env::args().skip(1);
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--host" => {
                    host = arguments
                        .next()
                        .ok_or_else(|| "--host needs a value".to_string())?;
                    host_given = true;
                }
                "--port" | "-p" => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "--port needs a value".to_string())?;
                    port = parse_port(&value).map_err(|error| format!("--port: {error}"))?;
                    port_given = true;
                }
                "--address" | "-a" => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "--address needs a value".to_string())?;
                    combined =
                        Some(parse_address(&value).map_err(|error| format!("--address: {error}"))?);
                }
                "--workers" | "-w" => {
                    let value = arguments
                        .next()
                        .ok_or_else(|| "--workers needs a value".to_string())?;
                    workers = value
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

        if workers == 0 {
            return Err("--workers must be greater than zero".to_string());
        }

        let address = if host_given || port_given {
            if combined.is_some() {
                return Err(
                    "--address cannot be combined with --host or --port; use one form".to_string(),
                );
            }
            resolve(&host, port).map_err(|error| format!("--host: {error}"))?
        } else if let Some(address) = combined {
            address
        } else {
            resolve(&host, port).map_err(|error| format!("APSIS_BOOK_HOST: {error}"))?
        };

        Ok(Self { address, workers })
    }
}

/// Parses a port number.
/// 解析一个端口号。
fn parse_port(value: &str) -> Result<u16, String> {
    value
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("{value:?} is not a port number"))
}

/// Parses an address written as `host:port`.
/// 解析写成 `host:port` 的地址。
fn parse_address(value: &str) -> Result<SocketAddr, String> {
    value
        .trim()
        .parse::<SocketAddr>()
        .map_err(|_| format!("{value:?} is not a host and port"))
}

/// Resolves a host and a port into a socket address.
///
/// A host may be a literal address, a bracketed or bare IPv6 literal, or a
/// name that the operating system resolves. A name that resolves to several
/// addresses uses the first one, which is the order the resolver reports and
/// therefore the preferred one.
///
/// 把主机与端口解析为一个套接字地址。
///
/// 主机可以是字面地址、带方括号或不带方括号的 IPv6 字面量，或由操作系统解析的
/// 名字。解析出多个地址的名字使用第一个，那是解析器报告的顺序，因而也是首选的
/// 一个。
fn resolve(host: &str, port: u16) -> Result<SocketAddr, String> {
    let host = host.trim();
    if host.is_empty() {
        return Err("the host is empty".to_string());
    }

    // A literal address, including a bracketed IPv6 literal, needs no name
    // resolution.

    // 字面地址——包括带方括号的 IPv6 字面量——无需名字解析。
    let literal = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = literal.parse::<std::net::IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    // A name that already carries a port, such as `localhost:9000`, is
    // accepted so that a value written in the combined form still works.

    // 已经带端口的名，例如 `localhost:9000`，也被接受，使以合并形式写下的值仍然
    // 可用。
    if let Ok(address) = host.parse::<SocketAddr>() {
        return Ok(address);
    }

    use std::net::ToSocketAddrs;
    match (host, port).to_socket_addrs() {
        Ok(mut addresses) => addresses
            .next()
            .ok_or_else(|| format!("{host:?} does not resolve to an address")),
        Err(error) => Err(format!("{host:?} does not resolve: {error}")),
    }
}

/// Usage text.
/// 用法文本。
const USAGE: &str = "\
apsis-book: web service for the ApsisRS computational kernels

USAGE:
    apsis-book [OPTIONS]

OPTIONS:
    -a, --address <HOST:PORT>    address to bind (default 127.0.0.1:8080)
        --host <HOST>            interface to bind (default 127.0.0.1)
    -p, --port <PORT>            port to bind (default 8080)
    -w, --workers <N>            worker threads (default 4)
    -h, --help                   print this help
    -V, --version                print the version

ENVIRONMENT:
    APSIS_BOOK_ADDRESS            bind address, as HOST:PORT
    APSIS_BOOK_HOST               interface to bind
    APSIS_BOOK_PORT               port to bind
    APSIS_BOOK_WORKERS            worker threads

EXAMPLES:
    apsis-book --host 0.0.0.0 --port 8080
    apsis-book --address 127.0.0.1:9000";
