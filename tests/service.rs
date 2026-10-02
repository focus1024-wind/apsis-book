//! End to end tests for the service binary.
//!
//! The tests start the real binary on an ephemeral port, wait for the
//! address it reports, and then make plain TCP requests against it. This
//! exercises the request parser, the router, the JSON codec, the embedded
//! content, and the tool handlers together, which the unit tests cannot do.
//!
//! The binary is located through the `CARGO_BIN_EXE_apsis-book` variable that
//! cargo sets for an integration test, so the test always runs against the
//! binary built from the same sources.
//!
//! 针对服务二进制的端到端测试。
//!
//! 测试在一个临时端口上启动真实的二进制，等待它报告自己的地址，然后以普通 TCP
//! 请求与之通信。这样把请求解析器、路由器、JSON 编解码器、内嵌内容与工具处理
//! 函数放在一起检验，这是单元测试做不到的。
//!
//! 二进制的位置取自 cargo 为集成测试设置的 `CARGO_BIN_EXE_apsis-book`，因此测试
//! 总是针对由同一份源码构建出的二进制运行。

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// A running service that is stopped when it is dropped.
/// 一个运行中的服务，被丢弃时即停止。
struct Service {
    child: Child,
    address: String,
}

impl Service {
    /// Starts the binary and waits for it to report its address.
    /// 启动二进制并等待它报告自己的地址。
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_apsis-book"))
            .args(["--address", "127.0.0.1:0", "--workers", "2"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the service binary must start");

        let stdout = child.stdout.take().expect("stdout is piped");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(rest) = line.split("http://").nth(1) {
                    let _ = sender.send(rest.trim().to_string());
                    break;
                }
            }
        });

        let address = receiver
            .recv_timeout(Duration::from_secs(15))
            .expect("the service must report the address it bound");
        Self { child, address }
    }

    /// Sends a raw request and returns the whole response as text.
    /// 发送一个原始请求，并把整个响应作为文本返回。
    fn request(&self, request: &str) -> String {
        let mut stream =
            TcpStream::connect(&self.address).expect("the service must accept a connection");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .expect("a read timeout must be settable");
        stream
            .write_all(request.as_bytes())
            .expect("the request must be writable");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("the response must be readable");
        response
    }

    /// Sends a GET request.
    /// 发送一个 GET 请求。
    fn get(&self, path: &str) -> String {
        self.request(&format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n"))
    }

    /// Sends a POST request with a JSON body.
    /// 发送一个带 JSON 请求体的 POST 请求。
    fn post(&self, path: &str, body: &str) -> String {
        self.request(&format!(
            "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        ))
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Returns the body of a response.
/// 返回某个响应的请求体。
fn body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

/// Returns the status line of a response.
/// 返回某个响应的状态行。
fn status(response: &str) -> &str {
    response.lines().next().unwrap_or("")
}

/// Waits for a condition, polling until it holds or the deadline passes.
///
/// The accept loop starts a moment after the address is printed, so the
/// first connection can be refused.
///
/// 等待某个条件，轮询直到它成立或超过时限。
///
/// 接受循环在地址打印之后片刻才启动，因此第一次连接可能被拒绝。
fn wait_ready(service: &Service) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect(&service.address).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("the service did not become ready");
}

#[test]
fn health_endpoint_reports_the_embedded_content() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/health");
    assert!(status(&response).contains("200"), "{response}");
    let body = body(&response);
    assert!(body.contains("\"status\":\"ok\""), "{body}");
    assert!(body.contains("\"documents\":"), "{body}");
    assert!(body.contains("\"tools\":"), "{body}");
}

#[test]
fn serves_the_frontend_entry_point() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/");
    assert!(status(&response).contains("200"), "{response}");
    assert!(response.contains("text/html"), "{response}");
    let body = body(&response);
    assert!(
        body.contains("<!doctype html>") || body.contains("<!DOCTYPE html>"),
        "the entry point must be an HTML document"
    );
}

#[test]
fn serves_the_document_index_and_a_document() {
    let service = Service::start();
    wait_ready(&service);

    let index = service.get("/api/docs");
    assert!(status(&index).contains("200"), "{index}");
    let listing = body(&index);
    assert!(listing.contains("\"groups\""), "{listing}");
    // The concepts and the implementation are separate groups.
    // 概念与实现是两个分开的分组。
    assert!(listing.contains("\"concepts\""), "{listing}");
    assert!(listing.contains("\"implementation\""), "{listing}");

    let document = service.get("/api/docs/sgp4");
    assert!(status(&document).contains("200"), "{document}");
    let document_body = body(&document);
    assert!(document_body.contains("\"html\""), "{document_body}");
    assert!(document_body.contains("SGP4"), "{document_body}");
}

#[test]
fn serves_the_concept_and_crate_graphs() {
    let service = Service::start();
    wait_ready(&service);

    let concepts = service.get("/api/graph");
    assert!(body(&concepts).contains("\"nodes\""), "{}", body(&concepts));

    let crates = service.get("/api/graph/crates");
    assert!(body(&crates).contains("apsis-sgp4"), "{}", body(&crates));

    let flow = service.get("/api/graph/sgp4-propagation");
    assert!(body(&flow).contains("\"edges\""), "{}", body(&flow));
}

#[test]
fn interface_documentation_is_translated() {
    let service = Service::start();
    wait_ready(&service);

    let item = service.get("/api/api/items/propagate_minutes");
    let body = body(&item);
    // The Chinese documentation is served and is rendered to HTML.
    // 中文文档会被提供，并渲染为 HTML。
    assert!(body.contains("传播"), "the Chinese doc should be served");
    assert!(body.contains("<p>"), "the doc should be rendered as HTML");
    // A doc comment link to another item becomes an internal route.
    // 指向另一个条目的文档注释链接会成为一个内部路由。
    assert!(
        body.contains("data-api=") && body.contains("#/api/"),
        "the doc link should be resolved: {body}"
    );
    // The English documentation is still present, so the language can switch
    // without another request.

    // 英文文档仍然存在，因此切换语言无需再一次请求。
    assert!(
        body.contains("Returns the state of the satellite"),
        "the English doc should still be present"
    );
}

#[test]
fn crate_summaries_are_translated() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/api/crates/apsis-sgp4");
    let body = body(&response);
    // The Chinese summary and its plain form are both served.
    // 中文概要与它的纯文本形式都会被提供。
    assert!(
        body.contains("传播器"),
        "the Chinese summary should be served"
    );
    assert!(
        body.contains("\"plain\""),
        "the plain form should be served"
    );
}

#[test]
fn serves_the_interface_catalog() {
    let service = Service::start();
    wait_ready(&service);

    let index = service.get("/api/api");
    assert!(body(&index).contains("\"crate_count\""), "{}", body(&index));

    let item = service.get("/api/api/items/propagate_minutes");
    let item_body = body(&item);
    assert!(item_body.contains("\"signature\""), "{item_body}");
    assert!(item_body.contains("propagate_minutes"), "{item_body}");
}

#[test]
fn searches_the_documentation() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/docs/search?q=propagation");
    let body = body(&response);
    assert!(body.contains("\"hits\""), "{body}");
    assert!(!body.contains("\"hits\":[]"), "{body}");
}

#[test]
fn runs_the_tle_parser_tool() {
    let service = Service::start();
    wait_ready(&service);
    let tle = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\\n2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
    let response = service.post(
        "/api/tools/parse-tle/run",
        &format!("{{\"tle\":\"{tle}\"}}"),
    );
    let body = body(&response);
    assert!(status(&response).contains("200"), "{response}");
    assert!(body.contains("\"ok\":true"), "{body}");
    assert!(body.contains("25544"), "{body}");
}

#[test]
fn runs_the_sgp4_propagation_tool() {
    let service = Service::start();
    wait_ready(&service);
    let tle = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\\n2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
    let response = service.post(
        "/api/tools/sgp4-propagate/run",
        &format!("{{\"tle\":\"{tle}\",\"minutes\":10}}"),
    );
    let body = body(&response);
    assert!(body.contains("\"ok\":true"), "{body}");
    assert!(body.contains("\"kind\":\"state-vector\""), "{body}");
}

#[test]
fn runs_the_time_conversion_tool() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.post(
        "/api/tools/time-convert/run",
        "{\"input\":\"calendar\",\"year\":2024,\"month\":6,\"day\":15}",
    );
    let body = body(&response);
    assert!(body.contains("\"ok\":true"), "{body}");
    assert!(body.contains("\"kind\":\"time-scales\""), "{body}");
}

#[test]
fn reports_a_bad_parameter_with_its_name() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.post("/api/tools/sgp4-propagate/run", "{\"minutes\":99999999}");
    let body = body(&response);
    assert!(body.contains("\"ok\":false"), "{body}");
    assert!(body.contains("\"parameter\":\"minutes\""), "{body}");
}

#[test]
fn rejects_a_malformed_body_without_failing() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.post("/api/tools/sgp4-propagate/run", "{");
    let body = body(&response);
    assert!(body.contains("\"ok\":false"), "{body}");
    // The service must still answer a subsequent request.
    // 服务必须仍能回应后续请求。
    assert!(status(&service.get("/health")).contains("200"));
}

#[test]
fn unknown_api_resource_is_a_not_found() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/docs/not-a-document");
    assert!(status(&response).contains("404"), "{response}");
}

#[test]
fn documents_are_served_in_both_languages() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/docs/sgp4");
    let body = body(&response);
    assert!(body.contains("\"language\":\"zh\""), "{body}");
    assert!(body.contains("\"language\":\"en\""), "{body}");
    // The Chinese form of the document carries Chinese text.
    // 文档的中文形式携带中文文本。
    assert!(body.contains("传播"), "the Chinese form should be embedded");
}

#[test]
fn document_listing_carries_bilingual_titles() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/docs");
    let body = body(&response);
    assert!(body.contains("\"zh\""), "{body}");
    assert!(body.contains("\"en\""), "{body}");
}

#[test]
fn tool_registry_carries_bilingual_labels() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/tools");
    let body = body(&response);
    assert!(
        body.contains("解析两行元素集"),
        "the tool name should be Chinese too"
    );
    assert!(body.contains("Parse a two-line element set"), "{body}");
}

#[test]
fn tool_result_labels_are_bilingual() {
    let service = Service::start();
    wait_ready(&service);
    let tle = "1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927\\n2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
    let response = service.post(
        "/api/tools/sgp4-propagate/run",
        &format!("{{\"tle\":\"{tle}\"}}"),
    );
    let body = body(&response);
    assert!(body.contains("\"zh\":\"半长轴\""), "{body}");
    assert!(body.contains("\"en\":\"Semi-major axis\""), "{body}");
}

#[test]
fn the_flow_index_lists_every_flow() {
    let service = Service::start();
    wait_ready(&service);
    let index = service.get("/api/graph/flows");
    let body = body(&index);
    assert!(body.contains("\"flows\""), "{body}");
    assert!(body.contains("sgp4-propagation"), "{body}");
    assert!(body.contains("tle-parsing"), "{body}");
    // The index carries a bilingual title, so the frontend needs no copy of
    // the flow names.

    // 索引携带双语标题，因此前端无须另存一份流程图名。
    assert!(body.contains("\"title\""), "{body}");
    assert!(body.contains("\"zh\""), "{body}");
}

#[test]
fn static_assets_are_revalidated() {
    let service = Service::start();
    wait_ready(&service);
    // The build writes stable file names, so an immutable policy would pin a
    // reader to the bundle current at first load.

    // 构建写出的是稳定的文件名，因此不可变策略会把读者钉在首次加载时的那个包上。
    let asset = service.get("/assets/app.js");
    assert!(asset.contains("no-cache"), "{asset}");
    assert!(!asset.contains("immutable"), "{asset}");
}

#[test]
fn a_traversal_attempt_does_not_reach_the_filesystem() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/../../etc/passwd");
    let body = body(&response);
    assert!(
        !body.contains("root:x:"),
        "a traversal must not read a file"
    );
}

#[test]
fn graph_labels_are_bilingual() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.get("/api/graph");
    let body = body(&response);
    assert!(body.contains("概念图谱"), "{body}");
}

#[test]
fn head_request_omits_the_body_but_keeps_the_length() {
    let service = Service::start();
    wait_ready(&service);
    let response = service.request("HEAD /health HTTP/1.1\r\nHost: localhost\r\n\r\n");
    assert!(status(&response).contains("200"), "{response}");
    assert!(response.contains("Content-Length:"), "{response}");
    assert!(
        body(&response).is_empty(),
        "a HEAD response must have no body"
    );
}
