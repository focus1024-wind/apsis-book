#!/usr/bin/env python3
"""Extract the public interface of `apsis` into content/api.json.

The service presents the public interface of the `apsis` crate as it is
written in the source, so the catalog is generated from the source rather than
maintained by hand. This script reads the module directories under
`crates/apsis/src`, collects the module documentation and every public item
with its doc comment and signature, and writes the result to
`content/api.json`. The file is embedded in the binary by `build.rs`.

A layer is a directory directly under `src`, and a module is a directory
directly under it, named `apsis_<subject>` — the path the crate re-exports as
`apsis::apsis_<subject>`. The directories are read rather than listed here, so
a module appears in the catalog as soon as it exists and no page built from
the catalog can be quietly short of a layer.

Each doc comment in `apsis` is written in English and then in Chinese, in the
one comment beside the code. The two halves are split here, on the first line
that contains a Chinese character, and both are written to the catalog. The
service shows one of them, so no separate translation file is needed and a
translation cannot fall behind the code it describes.

Run it from the `apsis-book` directory when the `apsis` sources change:

    scripts/extract_api.sh

It requires Python 3.8 or later. It is a development tool: the Rust build does
not run it and does not depend on Python.
"""

import json
import os
import re
import sys

# The layer of a module is the directory it sits under, so the catalog is
# grouped the same way the crate is laid out. The order is the dependency
# order: a layer may use the layers to its left and no layer to its right.
LAYER_ORDER = [
    "foundation",
    "physics",
    "orbit",
    "rf",
    "network",
    "ops",
    "sim",
]


def crate_modules(crate_dir):
    """Return (layer, name, directory) for every module of the crate.

    A layer is a directory directly under `src`, and a module is a directory
    directly under it whose name begins with `apsis_`, which is the path the
    crate re-exports it under. The directories are read rather than listed
    here: a list kept in this script would go stale in silence the moment a
    module were added, and every page built from the catalog would then be
    quietly incomplete.
    """
    source = os.path.join(crate_dir, "src")
    modules = []
    for layer in LAYER_ORDER:
        layer_dir = os.path.join(source, layer)
        if not os.path.isdir(layer_dir):
            continue
        for name in sorted(os.listdir(layer_dir)):
            directory = os.path.join(layer_dir, name)
            if name.startswith("apsis_") and os.path.isdir(directory):
                modules.append((layer, name, directory))
    return modules


def ordered(modules):
    """Order the modules by layer, with the layer's own module last.

    Within a layer the module named after the layer, such as `apsis_orbit`, is
    the reader's entry point: it re-exports the items the other modules of the
    layer define, so it comes after them. The remaining modules keep the name
    order they are discovered in, which is stable without being a list this
    script has to maintain.
    """
    rank = {layer: index for index, layer in enumerate(LAYER_ORDER)}
    return sorted(
        modules,
        key=lambda module: (
            rank.get(module[0], len(rank)),
            module[1] == "apsis_" + module[0],
            module[1],
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


def collect_module(name, directory):
    """Return the catalog entry of one module.

    The module's own documentation comes from its `mod.rs`, and the items are
    every public declaration under the directory, in file name order so that
    the page reads the same way every time it is generated.
    """
    summary = ""
    summary_zh = None
    items = []
    entry_point = os.path.join(directory, "mod.rs")
    if os.path.exists(entry_point):
        (summary, summary_zh), _ = collect_file(entry_point)
    for current, _, files in os.walk(directory):
        for file_name in sorted(files):
            if not file_name.endswith(".rs"):
                continue
            path = os.path.join(current, file_name)
            _, file_items = collect_file(path)
            items.extend(file_items)
    # A module re-exports the items of the modules below it, so a name that
    # occurs more than once keeps its first occurrence, which is the
    # definition.
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
    crate_dir = os.path.join(workspace, "crates", "apsis")
    if not os.path.isdir(crate_dir):
        sys.stderr.write("the apsis crate was not found at %s\n" % crate_dir)
        return 1

    discovered = crate_modules(crate_dir)
    if not discovered:
        sys.stderr.write(
            "no modules found in %s\n" % os.path.join(crate_dir, "src")
        )
        return 1

    modules = []
    sources = []
    for layer, name, directory in ordered(discovered):
        entry = collect_module(name, directory)
        entry["layer"] = layer
        modules.append(entry)
        sources.append(os.path.relpath(directory, workspace))

    document = {
        "generated_from": sources,
        "modules": modules,
    }
    output = os.path.join(web_dir, "content", "api.json")
    os.makedirs(os.path.dirname(output), exist_ok=True)
    with open(output, "w", encoding="utf-8") as handle:
        json.dump(document, handle, ensure_ascii=False, indent=1, sort_keys=False)
        handle.write("\n")

    total = sum(len(module["items"]) for module in modules)
    print("wrote %s: %d modules, %d items" % (output, len(modules), total))
    return 0


if __name__ == "__main__":
    sys.exit(main())
