#!/usr/bin/env python3
"""Extract the public interface of apsis-rs into content/api.json.

The service presents the public interface of the `apsis-rs` workspace as it
is written in the source, so the catalog is generated from the source rather
than maintained by hand. This script reads the crate directories, collects
the crate level documentation and every public item with its doc comment and
signature, and writes the result to `content/api.json`. The file is embedded
in the binary by `build.rs`.

Each doc comment in `apsis-rs` is written in English and then in Chinese, in
the one comment beside the code. The two halves are split here, on the first
line that contains a Chinese character, and both are written to the catalog.
The service shows one of them, so no separate translation file is needed and
a translation cannot fall behind the code it describes. This replaced an
earlier arrangement that kept the Chinese in `content/api.zh.json` keyed by
item name, which was unsound because a name such as `new` is defined by many
crates and does not mean the same thing in each.

Run it from the `apsis-book` directory when the `apsis-rs` sources change:

    scripts/extract_api.sh

It requires Python 3.8 or later. It is a development tool: the Rust build
does not run it and does not depend on Python.
"""

import json
import os
import re
import sys

# The layer of a crate is the directory it sits under, so the catalog is
# grouped the same way the workspace is laid out.
LAYER_ORDER = [
    "foundation",
    "physics",
    "orbit",
    "network",
    "rf",
    "ops",
    "sim",
]


def manifest_crates(workspace):
    """Return (layer, name, directory) for every crate of the workspace.

    The crates come from the workspace manifest rather than from a list kept
    here. A list goes stale the moment a crate is added, and a stale list is
    silent: the catalog is simply short of the interface and every page
    built from it is quietly incomplete. The manifest is the one place the
    workspace states which crates there are, and each member path carries its
    layer in the directory above the crate name.

    The order is the one the workspace defines, which is also dependency
    order within each layer.
    """
    manifest = manifest_path(workspace)
    members = []
    inside = False
    with open(manifest, "r", encoding="utf-8") as handle:
        for line in handle:
            stripped = line.strip()
            if stripped.startswith("members"):
                inside = True
                continue
            if inside:
                if stripped.startswith("]"):
                    break
                # A member is a quoted path, possibly with a trailing comma.
                entry = stripped.split("#")[0].strip().strip(",").strip('"')
                if entry:
                    members.append(entry)

    crates = []
    for directory in members:
        parts = directory.split("/")
        if len(parts) < 2:
            continue
        name = parts[-1]
        # A member that sits directly in `crates` is the facade, which
        # re-exports every layer and so belongs to none of them. It is left
        # without a layer rather than given the wrong one, and the page that
        # lists crates by layer shows it apart from the layers.
        layer = parts[-2] if len(parts) >= 3 else None
        crates.append((layer, name, directory))
    return crates


def manifest_path(workspace):
    """The workspace manifest path."""
    return os.path.join(workspace, "Cargo.toml")


def ordered(crates):
    """Order the crates by layer, keeping the workspace order within a layer."""
    rank = {layer: index for index, layer in enumerate(LAYER_ORDER)}
    return sorted(
        enumerate(crates),
        # The facade goes last: it re-exports what the layers define, so a
        # reader reaches it after the crate that defines an item.
        key=lambda pair: (
            rank.get(pair[1][0], len(rank) + 1) if pair[1][0] else len(rank),
            pair[0],
        ),
    )


ITEM = re.compile(
    r"^(?P<indent>\s*)pub (?:(?P<qualifier>const|unsafe) )?"
    r"(?P<kind>fn|struct|enum|trait|const|type|union) "
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)

DOC = re.compile(r"^\s*/// ?(.*)$")
MODULE_DOC = re.compile(r"^\s*//! ?(.*)$")

# A doc comment carries the English text first and the Chinese text after it,
# in one comment. The boundary is the first line that contains a Chinese
# character. The two halves are returned separately so that the service can
# show one of them rather than both at once.
CJK = re.compile(r"[\u4e00-\u9fff]")
# A line that is only a code fence or an example belongs to whichever half it
# is in; it is not a boundary on its own.
def split_bilingual_lines(lines):
    """Split doc comment lines into their English and Chinese parts."""
    boundary = None
    for position, line in enumerate(lines):
        if CJK.search(line):
            boundary = position
            break
    if boundary is None:
        return "\n".join(lines).strip(), None
    english = "\n".join(lines[:boundary]).strip()
    chinese = "\n".join(lines[boundary:]).strip()
    # A comment that is Chinese throughout has no English half; the Chinese
    # is then the only text there is.
    return english, chinese or None


def split_bilingual(comment):
    """Split one doc comment, given as text, into its two languages."""
    return split_bilingual_lines(comment.split("\n"))


def module_doc_lines(lines):
    """Return the leading //! block as its lines, keeping their structure."""
    collected = []
    for line in lines:
        match = MODULE_DOC.match(line)
        if match:
            collected.append(match.group(1))
        elif collected:
            break
        elif line.strip() and not line.startswith("//!"):
            break
    return collected


def module_doc_pair(lines):
    """Return the leading //! block in English and in Chinese.

    The module documentation is the whole block, written in English and then
    in Chinese. Both halves are returned whole, with their paragraphs and
    lists intact: the crate page shows all of it, and the one line a card
    needs is derived from it later.

    The split is made on the lines rather than on the joined text, because a
    paragraph boundary and a language boundary are both line boundaries, and
    joining first would lose both.
    """
    return split_bilingual_lines(module_doc_lines(lines))


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
            doc_all = doc_comment(buffer)
            doc, doc_zh = split_bilingual(doc_all)
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
                    "doc_zh": doc_zh,
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

    return module_doc_pair(lines), items


def collect_crate(workspace, name, directory):
    """Return the catalog entry of one crate."""
    root = os.path.join(workspace, directory, "src")
    summary = ""
    summary_zh = None
    items = []
    lib = os.path.join(root, "lib.rs")
    if os.path.exists(lib):
        (summary, summary_zh), _ = collect_file(lib)
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
    return {
        "name": name,
        "layer": None,
        "summary": summary,
        "summary_zh": summary_zh,
        "items": unique,
    }


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

    discovered = manifest_crates(workspace)
    if not discovered:
        sys.stderr.write("no crates found in %s\n" % manifest_path(workspace))
        return 1

    crates = []
    sources = []
    for _, (layer, name, directory) in ordered(discovered):
        if not os.path.isdir(os.path.join(workspace, directory)):
            sys.stderr.write("skipping %s: no such directory\n" % directory)
            continue
        if layer is None:
            # The facade sits directly in `crates` and re-exports every layer.
            # It defines nothing of its own, so listing it would repeat every
            # item of the layers under a second name without adding anything.
            sys.stderr.write("skipping %s: the workspace facade\n" % name)
            continue
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
