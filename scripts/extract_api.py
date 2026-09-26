#!/usr/bin/env python3
"""Extract the public interface of apsis-rs into content/api.json.

The service presents the public interface of the `apsis-rs` workspace as it
is written in the source, so the catalog is generated from the source rather
than maintained by hand. This script reads the crate directories of the
layers that `apsis-book` enables, collects the crate level documentation and
every public item with its doc comment and signature, and writes the result
to `content/api.json`. The file is embedded in the binary by `build.rs`.

Run it from the `apsis-book` directory when the `apsis-rs` sources change:

    scripts/extract_api.sh

It requires Python 3.8 or later. It is a development tool: the Rust build
does not run it and does not depend on Python.
"""

import json
import os
import re
import sys

# Crates of the layers that the enabled `apsis-rs` features pull in, in
# dependency order. The order is the one the workspace layer rule defines.
LAYERS = [
    ("foundation", ["apsis-core", "apsis-units", "apsis-math", "apsis-macros"]),
    ("physics", ["apsis-time", "apsis-leap", "apsis-coord", "apsis-geodesy", "apsis-frames"]),
    ("orbit", [
        "apsis-tle",
        "apsis-sgp4",
        "apsis-kepler",
        "apsis-ephemeris",
        "apsis-maneuver",
        "apsis-orbit",
    ]),
]

# Directories of the workspace, relative to the workspace root, keyed by
# crate name.
CRATE_DIRS = {
    "apsis-core": "crates/foundation/apsis-core",
    "apsis-units": "crates/foundation/apsis-units",
    "apsis-math": "crates/foundation/apsis-math",
    "apsis-macros": "crates/foundation/apsis-macros",
    "apsis-time": "crates/physics/apsis-time",
    "apsis-leap": "crates/physics/apsis-leap",
    "apsis-coord": "crates/physics/apsis-coord",
    "apsis-geodesy": "crates/physics/apsis-geodesy",
    "apsis-frames": "crates/physics/apsis-frames",
    "apsis-tle": "crates/orbit/apsis-tle",
    "apsis-sgp4": "crates/orbit/apsis-sgp4",
    "apsis-kepler": "crates/orbit/apsis-kepler",
    "apsis-ephemeris": "crates/orbit/apsis-ephemeris",
    "apsis-maneuver": "crates/orbit/apsis-maneuver",
    "apsis-orbit": "crates/orbit/apsis-orbit",
}

ITEM = re.compile(
    r"^(?P<indent>\s*)pub (?:(?P<qualifier>const|unsafe) )?"
    r"(?P<kind>fn|struct|enum|trait|const|type|union) "
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)

DOC = re.compile(r"^\s*/// ?(.*)$")
MODULE_DOC = re.compile(r"^\s*//! ?(.*)$")


def module_doc(lines):
    """Return the first paragraph of the leading //! block."""
    paragraph = []
    for line in lines:
        match = MODULE_DOC.match(line)
        if match:
            paragraph.append(match.group(1))
        elif paragraph:
            break
        elif line.strip() and not line.startswith("//!"):
            break
    text = " ".join(part.strip() for part in paragraph if part.strip())
    text = re.sub(r"\s+", " ", text).strip()
    # Drop a trailing colon that introduces a list, which reads poorly when
    # the paragraph is shown on its own.
    return text.rstrip(":")


def doc_comment(buffer):
    """Return the doc comment at the end of `buffer` as one string."""
    collected = []
    for line in reversed(buffer):
        match = DOC.match(line)
        if match:
            collected.append(match.group(1))
            continue
        stripped = line.strip()
        if stripped.startswith("#[") or stripped.startswith("#!["):
            continue
        break
    collected.reverse()
    return "\n".join(collected).strip()


def is_test_attribute(line):
    """Return true for the attribute that introduces the test module."""
    return line.strip() in ("#[cfg(test)]", "#[cfg(test)] #[cfg(test)]")


def collect_file(path):
    """Return the module documentation and public items of one source file."""
    with open(path, "r", encoding="utf-8") as handle:
        lines = handle.read().split("\n")

    items = []
    buffer = []
    index = 0
    # Depth of the test module: zero means not inside one.
    test_depth = 0
    brace_depth = 0

    while index < len(lines):
        line = lines[index]

        if test_depth:
            brace_depth += line.count("{") - line.count("}")
            if brace_depth <= 0:
                test_depth = 0
                brace_depth = 0
            index += 1
            continue
        if is_test_attribute(line) and index + 1 < len(lines) and "mod tests" in lines[index + 1]:
            test_depth = 1
            brace_depth = 0
            index += 2
            continue

        match = ITEM.match(line)
        if match:
            doc = doc_comment(buffer)
            signature_lines = [line.strip()]
            # A declaration continues until its parentheses balance and the
            # line ends with `{`, `;`, or a where clause that is closed.
            depth = line.count("(") - line.count(")")
            cursor = index
            while not signature_lines[-1].rstrip().endswith(("{", ";")) or depth > 0:
                cursor += 1
                if cursor >= len(lines):
                    break
                continuation = lines[cursor].strip()
                signature_lines.append(continuation)
                depth += continuation.count("(") - continuation.count(")")
            signature = " ".join(signature_lines)
            signature = re.sub(r"\s+", " ", signature).strip()
            kind = match.group("kind")
            if kind == "union":
                kind = "struct"
            items.append(
                {
                    "kind": kind,
                    "name": match.group("name"),
                    "signature": signature,
                    "doc": doc,
                    "associated": bool(match.group("indent")),
                }
            )
            buffer = []
            index = cursor + 1
            continue

        buffer.append(line)
        if len(buffer) > 200:
            buffer = buffer[-200:]
        index += 1

    return module_doc(lines), items


def collect_crate(workspace, name, directory):
    """Return the catalog entry of one crate."""
    root = os.path.join(workspace, directory, "src")
    summary = ""
    items = []
    lib = os.path.join(root, "lib.rs")
    if os.path.exists(lib):
        summary, _ = collect_file(lib)
    for current, _, files in os.walk(root):
        for file_name in sorted(files):
            if not file_name.endswith(".rs"):
                continue
            path = os.path.join(current, file_name)
            _, file_items = collect_file(path)
            items.extend(file_items)
    # The facade crate re-exports items that are documented on the crates
    # below it, so a name that occurs more than once keeps its first
    # occurrence, which is the definition.
    seen = set()
    unique = []
    for item in items:
        key = (item["kind"], item["name"], item["signature"])
        if key in seen:
            continue
        seen.add(key)
        unique.append(item)
    return {"name": name, "layer": None, "summary": summary, "items": unique}


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    web_dir = os.path.dirname(here)
    workspace = os.environ.get(
        "APSIS_RS_WORKSPACE",
        os.path.normpath(os.path.join(web_dir, "..", "apsis-rs")),
    )
    if not os.path.isdir(workspace):
        sys.stderr.write("apsis-rs workspace not found at %s\n" % workspace)
        return 1

    crates = []
    sources = []
    for layer, names in LAYERS:
        for name in names:
            directory = CRATE_DIRS[name]
            entry = collect_crate(workspace, name, directory)
            entry["layer"] = layer
            crates.append(entry)
            sources.append(os.path.join(directory, "src"))

    document = {
        "generated_from": sources,
        "crates": crates,
    }
    output = os.path.join(web_dir, "content", "api.json")
    os.makedirs(os.path.dirname(output), exist_ok=True)
    with open(output, "w", encoding="utf-8") as handle:
        json.dump(document, handle, ensure_ascii=False, indent=1, sort_keys=False)
        handle.write("\n")

    total = sum(len(crate["items"]) for crate in crates)
    print("wrote %s: %d crates, %d items" % (output, len(crates), total))
    return 0


if __name__ == "__main__":
    sys.exit(main())
