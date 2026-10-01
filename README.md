# apsis-book

The web service layer of ApsisRS. It presents the ApsisRS computational
kernels for commercial spaceflight and satellite internet systems in a
browser: the concept documentation, a knowledge graph, the interface
documentation extracted from the library source, and an online tool set that
runs the kernels on parameters the user supplies.

The service builds to a single binary that embeds every asset it serves. It
does not read a content directory or a frontend build at run time, and it
does not need either to be present next to the binary.

## What it serves

| Section | Content | Route prefix |
|---------|---------|--------------|
| Concepts | Documents on the time scales, the reference frames, the element formats, and the propagation models, with the units and the applicable range of each | `/api/docs` |
| Knowledge graph | The concepts and their relationships, plus the crate structure of the library and the algorithm flow graphs | `/api/graph` |
| Interface | The public interface of the ApsisRS layers the service builds against, extracted from the source doc comments | `/api/api` |
| Online tools | TLE parsing, SGP4 propagation, track sampling, time scale conversion, coordinate conversion, frame rotation, Keplerian conversion, and Hohmann transfer planning | `/api/tools` |

## Design constraints

The service follows the constraints of the workspace it belongs to.

- **One runtime dependency.** The only entry in `[dependencies]` is
  `apsis-rs`, through a path dependency. The HTTP server, the JSON codec,
  the router, the Markdown renderer, and the graph model are all written
  against the standard library.
- **No `unsafe`.** The crate does not contain an unsafe block.
- **No panic in a request path.** A malformed request, a bad parameter, or
  a failing computation produces a response, not a panic. The worker that
  serves a request catches a panic in the handler as a final guard and
  answers with a server error.
- **Self-contained comments.** Every comment explains the code, the
  algorithm, the interface, or the project. No comment refers to a private
  document, an instruction, or a plan.

## Building

The frontend must be built before the release binary so that its output can
be embedded. A release build does this automatically.

Every built asset carries a content hash in its name, so replacing the binary
replaces those names. A browser therefore cannot be holding a stale bundle:
the new bundle is requested under a URL it has never seen. The entry point
is served with `no-cache, must-revalidate`, and hashed assets with
`public, max-age=31536000, immutable`. The service registers no service
worker.

```sh
# Development: a debug build embeds whatever is in frontend/dist and does
# not run the frontend build.
cargo build

# Release: build.rs builds the frontend and embeds the result.
cargo build --release
```

The binary is at `target/release/apsis-book`.

Environment variables:

| Variable | Effect |
|----------|--------|
| `APSIS_BOOK_HOST` | Interface to bind, default `127.0.0.1` |
| `APSIS_BOOK_PORT` | Port to bind, default `8080` |
| `APSIS_BOOK_ADDRESS` | Bind address as `HOST:PORT`, overriding the two above |
| `APSIS_BOOK_WORKERS` | Worker threads, default 4 |
| `APSIS_BOOK_SKIP_FRONTEND` | Set to `1` to skip the frontend build in a release build |
| `APSIS_BOOK_FRONTEND_DIR` | Frontend directory, default `frontend` |

Command line options override the environment:

```sh
# Bind every interface on port 8080.
apsis-book --host 0.0.0.0 --port 8080

# The combined form is equivalent.
apsis-book --address 0.0.0.0:8080
```

`--host` and `--port` may be given together. `--address` cannot be combined
with either of them, because the order in which they would apply is not
defined. A host may be a literal address, an IPv6 literal in brackets, or a
name the operating system resolves.

### Language

The interface is bilingual and opens in Chinese. The reader switches to
English with the control in the top bar; the choice is stored in the browser
and applies to every page.

The backend carries every user facing string in both languages, so the switch
takes effect without a request. A concept document is written once in English
as `content/concepts/<id>.md` and translated as
`content/concepts/<id>.zh.md`. A document without a translation is shown in
English to every reader, with a note saying so.

## Running

```sh
./target/release/apsis-book --address 127.0.0.1:8080
```

Open `http://127.0.0.1:8080`. The JSON interfaces are available without the
frontend:

- `GET /health`
- `GET /api/docs`, `GET /api/docs/<id>`, `GET /api/docs/search?q=`
- `GET /api/graph`, `GET /api/graph/crates`, `GET /api/graph/flows`, `GET /api/graph/<flow>`
- `GET /api/api`, `GET /api/api/crates/<name>`, `GET /api/api/items/<name>`
- `GET /api/tools`, `GET /api/tools/<id>`, `POST /api/tools/<id>/run`

A tool call takes a JSON object of parameters and returns a shaped result:

```sh
curl -s http://127.0.0.1:8080/api/tools/hohmann-transfer/run \
  -H 'content-type: application/json' \
  -d '{"from_radius": 6778, "to_radius": 42164}'
```

## Regenerating the interface catalog

The interface documentation is generated from the ApsisRS sources rather than
written by hand. Run the extractor from this directory when the sources
change, then rebuild:

```sh
scripts/extract_api.sh
cargo build --release
```

The script reads the workspace at `../apsis-rs` by default; set
`APSIS_RS_WORKSPACE` to point at a checkout elsewhere. It requires Python 3.8
or later. The Rust build does not run it and does not depend on Python.

## Editing the content

| Path | Contents |
|------|----------|
| `content/concepts/chapters.json` | Chapter list and the reading order of documents, with bilingual titles |
| `content/concepts/*.md` | Concept documents, with a `key: value` metadata header |
| `content/concepts/*.zh.md` | Chinese translations; the identifier comes from the file name |
| `content/graph/concepts.json` | The concept graph |
| `content/flows/*.json` | The algorithm flow graphs |
| `content/api.json` | The generated interface catalog; do not edit by hand |
| `content/api.zh.json` | Chinese translations of the crate summaries and item doc comments |

A document header declares `id`, `title`, `chapter`, `summary`, `tags`, and
`prerequisites`. A cross reference is written `[[id]]` or `[[id|label]]` and
renders as a link that the frontend routes without a page load.

## Layout

```text
apsis-book/
├── build.rs                  embed the frontend build and the text content
├── Cargo.toml                one runtime dependency: apsis-rs
├── content/                  authored documents, graphs, and generated catalog
├── frontend/                 Svelte and TypeScript, built by Vite
│   ├── src/lib/              API client, router, components, formatting
│   ├── src/pages/            one component per route
│   └── dist/                 build output, embedded by build.rs
├── scripts/extract_api.sh    interface catalog generator
├── src/
│   ├── api/                  the router and the JSON payloads
│   ├── docs/                 content, Markdown, documents, interface catalog
│   ├── embedded/             access to the embedded frontend assets
│   ├── graph/                the knowledge graph and the flow graphs
│   ├── http/                 request, response, and server
│   ├── tools/                the tool registry, the handlers, the result shape
│   ├── json.rs               the JSON codec
│   ├── main.rs               the process entry point
│   └── state.rs              the parsed content, shared by every worker
└── tests/service.rs          end to end tests against the built binary
```

## Testing

```sh
cargo test          # unit tests and end to end tests
cargo clippy --all-targets
cargo fmt --check
cd frontend && npm run check
```

The end to end tests start the built binary on an ephemeral port and make
plain TCP requests against it.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
