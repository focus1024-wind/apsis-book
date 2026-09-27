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

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// A running service that is stopped when it is dropped.
struct Service {
    child: Child,
    address: String,
}

impl Service {
    /// Starts the binary and waits for it to report its address.
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
    fn get(&self, path: &str) -> String {
        self.request(&format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n"))
    }

    /// Sends a POST request with a JSON body.
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
fn body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

/// Returns the status line of a response.
fn status(response: &str) -> &str {
    response.lines().next().unwrap_or("")
}

/// Waits for a condition, polling until it holds or the deadline passes.
///
/// The accept loop starts a moment after the address is printed, so the
/// first connection can be refused.
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
    assert!(body(&index).contains("\"chapters\""), "{}", body(&index));

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
    assert!(body.contains("传播"), "the Chinese doc should be served");
    assert!(body.contains("<p>"), "the doc should be rendered as HTML");
    // A doc comment link to another item becomes an internal route.
    assert!(
        body.contains("data-api=") && body.contains("#/api/"),
        "the doc link should be resolved: {body}"
    );
    // The English documentation is still present, so the language can switch
    // without another request.
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
