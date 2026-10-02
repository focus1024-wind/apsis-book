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
//!
//! `apsis-book` 服务，以库的形式。
//!
//! 服务是一个进程，但它的实质是一组模块：请求路由器及其编码的载荷、文档与内容层、
//! 图模型、工具注册表及其处理函数，以及它们之下的 HTTP 服务器。把它们汇集于此，
//! 是为了让端到端测试能以库的形式驱动服务而不只是通过套接字，也让模块测试能与
//! 本文件并列放在 `tests/`，而不是待在模块内部。
//!
//! 进程入口在 `main.rs`，它只是覆盖在此之上的一薄层。
//!
//! ## 可见性
//!
//! 这里几乎一切都是 `pub`，因为服务本身即产品，而各模块是它的部件。少数条目
//! 仅为让测试能够触达而存在，它们不直接是 `pub`，而是 `pub(crate)` 并经
//! [`testkit`] 转出——那是调用方应当寻找它们的唯一位置。这样接缝集中在一个模块里，
//! 而不必散布到被测的各个模块中。

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
