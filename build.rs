//! Build script for `apsis-book`.
//!
//! The script has one job: turn the frontend build output into a Rust source
//! file that embeds every asset in the binary. It never introduces a crate
//! dependency, because it uses only `std`.
//!
//! Behaviour:
//!
//! - In a debug build the frontend is not rebuilt. The script scans
//!   whatever is already in `frontend/dist/` and embeds it.
//! - In a release build the script runs the frontend build first, unless
//!   `APSIS_BOOK_SKIP_FRONTEND=1` is set. The frontend directory can be
//!   changed with `APSIS_BOOK_FRONTEND_DIR`.
//! - When `frontend/dist/` is absent the script emits an empty asset table.
//!   The service then serves a built-in placeholder page, so a checkout
//!   without a frontend build still compiles and runs.
//!
//! The generated files are `$OUT_DIR/embedded_assets.rs`, which defines
//! `pub static ASSETS: &[(&str, &[u8])]`, and `$OUT_DIR/embedded_content.rs`,
//! which defines `pub static CONTENT: &[(&str, &str)]` for the text content
//! under `content/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env("OUT_DIR"));
    let frontend_dir = std::env::var("APSIS_BOOK_FRONTEND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("frontend"));
    let dist_dir = frontend_dir.join("dist");

    // A rerun is needed when the frontend sources or the configuration
    // change. The output directory is not watched, otherwise the script
    // would trigger itself.
    println!("cargo:rerun-if-changed=build.rs");
    for name in [
        "package.json",
        "package-lock.json",
        "vite.config.ts",
        "vite.config.js",
        "svelte.config.js",
        "tsconfig.json",
        "index.html",
    ] {
        println!(
            "cargo:rerun-if-changed={}",
            frontend_dir.join(name).display()
        );
    }
    println!(
        "cargo:rerun-if-changed={}",
        frontend_dir.join("src").display()
    );
    // The output directory is registered unconditionally, so creating it
    // after a build that embedded nothing causes the next build to embed it.
    // Cargo accepts a path that does not exist yet.
    println!("cargo:rerun-if-changed={}", dist_dir.display());
    println!("cargo:rerun-if-env-changed=APSIS_BOOK_SKIP_FRONTEND");
    println!("cargo:rerun-if-env-changed=APSIS_BOOK_FRONTEND_DIR");

    let is_release = std::env::var("DEBUG")
        .map(|value| value == "false")
        .unwrap_or(false);
    let skip = std::env::var("APSIS_BOOK_SKIP_FRONTEND")
        .map(|value| value == "1")
        .unwrap_or(false);

    if is_release && !skip {
        if frontend_dir.join("package.json").exists() {
            build_frontend(&frontend_dir);
        } else {
            println!(
                "cargo:warning=apsis-book: no frontend/package.json, embedding whatever is in {}",
                dist_dir.display()
            );
        }
    }

    let assets = if dist_dir.is_dir() {
        collect(&dist_dir).unwrap_or_else(|error| {
            println!(
                "cargo:warning=apsis-book: cannot scan {}: {error}",
                dist_dir.display()
            );
            Vec::new()
        })
    } else {
        Vec::new()
    };

    if assets.is_empty() {
        println!(
            "cargo:warning=apsis-book: no frontend assets found in {}; the placeholder page will be served",
            dist_dir.display()
        );
    }

    let source = render(&assets);
    fs::write(out_dir.join("embedded_assets.rs"), source)
        .expect("the generated asset table must be writable");

    // The authored text content is embedded as strings so that the document
    // layer can parse it without touching the filesystem at run time.
    let content_dir = manifest_dir.join("content");
    println!("cargo:rerun-if-changed={}", content_dir.display());
    let content = if content_dir.is_dir() {
        collect(&content_dir).unwrap_or_else(|error| {
            println!(
                "cargo:warning=apsis-book: cannot scan {}: {error}",
                content_dir.display()
            );
            Vec::new()
        })
    } else {
        Vec::new()
    };
    let content_source = render_content(&content);
    fs::write(out_dir.join("embedded_content.rs"), content_source)
        .expect("the generated content table must be writable");
}

/// Returns an environment variable that cargo always sets for a build
/// script.
fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set by cargo"))
}

/// Installs the frontend dependencies when needed and builds the frontend.
///
/// A failure is reported as a warning rather than an error so that a release
/// build without network access still produces a binary, embedding the last
/// successful build output when one exists.
fn build_frontend(frontend_dir: &Path) {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    if !frontend_dir.join("node_modules").is_dir() {
        println!("cargo:warning=apsis-book: installing frontend dependencies");
        match Command::new(npm)
            .args(["install", "--no-audit", "--no-fund"])
            .current_dir(frontend_dir)
            .status()
        {
            Ok(status) if status.success() => {}
            Ok(status) => println!(
                "cargo:warning=apsis-book: npm install exited with {status}; embedding the existing dist if any"
            ),
            Err(error) => {
                println!(
                    "cargo:warning=apsis-book: cannot run npm ({error}); embedding the existing dist if any"
                );
                return;
            }
        }
    }

    println!("cargo:warning=apsis-book: building the frontend");
    match Command::new(npm)
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(status) => println!(
            "cargo:warning=apsis-book: npm run build exited with {status}; embedding the existing dist if any"
        ),
        Err(error) => println!(
            "cargo:warning=apsis-book: cannot run npm run build ({error}); embedding the existing dist if any"
        ),
    }
}

/// Walks `root` and returns every regular file as a pair of the path
/// relative to `root`, with forward slashes, and its absolute path.
fn collect(root: &Path) -> std::io::Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let mut entries: Vec<_> = fs::read_dir(&directory)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                files.push((relative, path));
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Renders the asset table as Rust source.
fn render(assets: &[(String, PathBuf)]) -> String {
    let mut source = String::new();
    source.push_str("// Generated by build.rs. Do not edit.\n");
    source.push_str("pub static ASSETS: &[(&str, &[u8])] = &[\n");
    for (name, path) in assets {
        source.push_str("    (");
        source.push_str(&rust_string(name));
        source.push_str(", include_bytes!(");
        source.push_str(&rust_string(&path.to_string_lossy()));
        source.push_str(")),\n");
    }
    source.push_str("];\n");
    source
}

/// Renders the text content table as Rust source.
///
/// Only files with a text extension are included, so that a stray binary in
/// the content directory cannot break the build.
fn render_content(files: &[(String, PathBuf)]) -> String {
    let mut source = String::new();
    source.push_str("// Generated by build.rs. Do not edit.\n");
    source.push_str("pub static CONTENT: &[(&str, &str)] = &[\n");
    for (name, path) in files {
        if !is_text_file(name) {
            continue;
        }
        source.push_str("    (");
        source.push_str(&rust_string(name));
        source.push_str(", include_str!(");
        source.push_str(&rust_string(&path.to_string_lossy()));
        source.push_str(")),\n");
    }
    source.push_str("];\n");
    source
}

/// Returns `true` when the file extension names a text format.
fn is_text_file(name: &str) -> bool {
    matches!(
        name.rsplit('.').next().unwrap_or_default(),
        "md" | "markdown" | "json" | "txt" | "graph"
    )
}

/// Escapes a string as a Rust string literal.
fn rust_string(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for character in text.chars() {
        match character {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            other => literal.push(other),
        }
    }
    literal.push('"');
    literal
}
