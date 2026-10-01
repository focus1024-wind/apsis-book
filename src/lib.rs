//! The `apsis-book` service, as a library.
//!
//! The service is a process, but its substance is a set of modules: the
//! request router and the payloads it encodes, the document and content
//! layer, the graph model, the tool registry and its handlers, and the HTTP
//! server underneath them. Those are gathered here so that the end to end
//! tests can drive the service as a library rather than only through a
//! socket, and so that the module tests can live beside this file in
//! `tests/` rather than inside the modules themselves.
//!
//! The process entry point is in `main.rs`, which is a thin layer over this.
//!
//! ## Visibility
//!
//! Almost everything here is `pub` because the service is the product and
//! the modules are its parts. The few items that exist only so that a test
//! can reach them are not `pub` directly: they are `pub(crate)` and are
//! re-exported through [`testkit`], which is the one place a caller is
//! expected to look for them. That keeps the seams in one module instead of
//! scattering them through the ones being tested.

pub mod api;
pub mod docs;
pub mod embedded;
pub mod graph;
pub mod http;
pub mod i18n;
pub mod json;
pub mod state;
pub mod tools;

pub mod testkit;
