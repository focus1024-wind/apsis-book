# apsis-book design notes

These notes explain the decisions behind the service. They are for someone
reading the code, and they describe the design rather than a plan.

## Why the standard library only

The ApsisRS workspace is written so that its numerical kernels can be audited
without following a dependency tree. A web service that pulled in an
asynchronous runtime, a web framework, and a serialization framework would
break that property at the layer where the kernels are exposed, which is the
layer a reader is most likely to look at first.

The cost is real and worth stating:

- The HTTP server is synchronous and handles one request per connection.
  That is enough for a documentation and tool service and it is not enough
  for a streaming or long-polling workload.
- The JSON codec is written here. It covers the subset of RFC 8259 the
  service exchanges and it is tested against malformed input, but it is not a
  general purpose library.
- The Markdown renderer covers the constructs the content uses. It is not a
  CommonMark implementation.

Each of these is bounded work rather than an open ended one, and each is
small enough to read in one sitting, which is the point.

## Request handling

A connection is accepted, one request is read, one response is written, and
the connection closes. The server sends `Connection: close` on every
response, which removes the need for a connection state machine and for
keep-alive bookkeeping.

The head is read one byte at a time. That is slower than a buffered read and
it is what makes the boundary between the head and the body exact without a
pushback buffer: a byte that is not part of the head is never consumed. A
read timeout bounds how long a slow peer can hold a worker, and a byte limit
on the head and on the body bounds what a hostile peer can make the process
allocate.

Requests are served by a fixed pool of worker threads fed from a bounded
queue. When every worker is busy, an accepted connection waits in the queue,
so memory use is bounded by the pool size rather than by the connection
count. The `Connection: close` policy means a worker never waits for a
second request on the same connection.

A panic in a handler is caught and turned into a server error response. The
handlers do not panic deliberately, and the guard is there because a panic
in a worker thread would otherwise end that thread and, with a fixed pool,
quietly reduce the service's capacity.

## JSON

`Value` holds `Null`, `Bool`, `Number(f64)`, `String`, `Array`, and `Object`.
Objects preserve insertion order, which makes the encoded output stable and
readable.

Two properties matter for safety:

- **Encoding never produces unparsable output.** A non-finite number has no
  JSON representation, so it is written as `null` rather than as `NaN` or
  `inf`. A computation that produced a not-a-number value therefore cannot
  produce a document a client fails to parse.
- **Decoding is bounded.** Nesting depth is limited, so a hostile document
  cannot exhaust the stack through recursion.

Strings are escaped per RFC 8259 and surrogate pairs are combined. Text that
is not valid UTF-8 is rejected rather than replaced, so a client cannot
smuggle a byte sequence through the decoder.

## Documents and content

Content is embedded as text by `build.rs`, which scans `content/` and
generates a table of `include_str!` entries. The document layer parses that
table once at startup.

A document is Markdown with a small metadata header. The header is parsed for
the identifier, the title, the chapter, the summary, the tags, and the
prerequisites; the body is rendered to HTML once and both forms are kept.
Serving a document therefore costs a string copy.

The Markdown renderer escapes every text node and emits only the markup it
generates, so a document cannot inject markup. The `{@html}` in the frontend
is the one place the service hands markup to the browser, and the backend
guarantees what can be in it.

Cross references are collected while parsing and checked against the set of
documents. A reference that does not resolve is reported by
`Tree::missing_links`, which the state test asserts is empty for the embedded
content. A mistyped reference therefore fails a test rather than becoming a
dead link in a page.

## Search

The index maps a lowercased token to the documents that contain it. A query
is tokenised the same way. A token is split on any non-alphanumeric
character, so `mean_motion`, `mean-motion`, and `mean motion` all match a
query for `mean motion`.

Scoring is the sum over the query tokens of a weight times a frequency term,
with a bonus for a token that appears in the title. The weight decreases with
the number of documents a token appears in, so a common word contributes
less than a rare one. The index is built once at startup and never changes,
so the postings are read only and need no lock.



A query is handled in two ways. A query that yields tokens is matched
against the postings; a query that yields no token, such as a single
character, is matched as a substring, because a one character query cannot
form a token and would otherwise always return nothing.

Chinese text needs more than word splitting. Latin text is split on
whitespace and punctuation, but Chinese is written without spaces, so
splitting a Chinese sentence on non-alphanumeric characters leaves one long
token that a two character query can never match. A run of Chinese
characters therefore yields one token per adjacent pair, which is the
smallest unit specific enough to be a word. A query of 坐标 matches a
sentence that contains 坐标, which a whole-run token would not. A single
Chinese character is left to the substring fallback rather than becoming a
token, because a token of one character would match nearly every document.

## Graphs

One graph shape serves the knowledge graph, the crate structure, and the
algorithm flows: nodes with a kind and a group, and labelled edges. The
backend supplies the structure; the layout is computed in the browser.

The layout is a force directed simulation with an initial placement derived
from a hash of the node identifier, so the same graph is drawn the same way
on every load. The simulation is run to convergence when the data changes
rather than animated, which keeps the component a pure function of its props
and makes the picture reproducible in a test or a screenshot.

## Tools

A tool is a declaration, not code that the frontend knows about. The registry
declares each parameter with its type, its unit, its bounds, its default, and
its help text. The frontend renders a form from that declaration.

Parameter values arrive as a JSON object. `Params` performs the typed reads
and validates against the declaration, so a handler deals in `f64`, `i64`,
`String`, and enumerated choices. A value may arrive as a JSON number or as a
string, because a form control sends text; both are accepted. A value outside
its declared bounds is rejected with the parameter name, which is what lets
the form highlight the field.

A handler shapes its result through `out`, which fixes the result to scalars,
a table, and numeric series. The frontend has one result renderer, so adding
a tool means adding a declaration and a handler and changing nothing in the
frontend.

An error carries the parameter name when the failure is a bad input and a
message written for the user of the form. The message names the condition and
the accepted range. It does not quote internal state, so a failure cannot
leak the shape of the implementation.

## Interface catalog

The catalog is generated from the ApsisRS sources by
`scripts/extract_api.sh`, which reads the crate directories, collects the
module documentation and the public items with their doc comments and
signatures, and writes `content/api.json`. The file is committed, so building
the Rust binary does not depend on the ApsisRS checkout or on Python.

Extraction rather than hand maintenance is what keeps the interface
documentation honest: a change to a signature in the library is a change to
the generated catalog after the script is rerun, and a hand written
description would drift silently.

The script reports the crates it read in `generated_from`, which the service
returns in `/api/api`, so the provenance of what is displayed is checkable.

The translation is not a separate file. `apsis-rs` writes each doc comment in
English and then in Chinese, in the one comment beside the code, and the
extractor splits the two halves as it reads. Both languages therefore come
from the same place as the code they describe, and neither can fall behind
it: a crate that has not been translated shows up as an item with no Chinese
rather than as an item quietly reading as English, and a test asserts that no
such item is left in the committed catalog.

An earlier revision kept the Chinese in `content/api.zh.json`, keyed by item
name. That is the arrangement this replaced. It was wrong for a reason worth
recording: the key was a name, and a name is not unique. `new` is defined by
fifty-nine crates and they do not all mean the same thing by it, so a
translation filed under the bare name could be shown against the wrong
documentation. Splitting the comment removes the question, because the
translation is attached to the item rather than looked up by a name.

A doc comment refers to another item with a Rust path, written as
`[Epoch](crate::Epoch)` or as a shortcut `[`Epoch::mjd`]`. The renderer turns
such a reference into a link to the interface page of that item. The name is
resolved by walking the path from the right and taking the first segment that
is a known item, so a link to a variant or a method points at the enum or the
trait that declares it. A reference whose segments are all unknown stays a
code span, which is what keeps a label such as `[`0, 1`]` from becoming a
link to nothing.

The documentation comment is rendered to HTML by the backend, with the same
renderer the concept documents use, so a comment that contains a list, a
table, or a code block is drawn as one.

## Frontend and embedding

The frontend is Svelte and TypeScript, built by Vite. The build output is a
stable set of file names rather than content hashed ones, so an unchanged
source produces an unchanged embedded table and an incremental Rust build
does not relink.

`build.rs` runs the frontend build in a release profile and skips it in a
debug profile. The split is deliberate: a debug build iterates on the backend
and should not pay for a frontend build, while a release build should produce
one binary with no further steps. `APSIS_BOOK_SKIP_FRONTEND=1` forces the skip
in a release build. A frontend failure is reported as a warning and the last
successful output is embedded, so a release build without network access
still produces a binary.

When nothing is embedded the service serves a built-in page that explains the
build step and lists the JSON routes, so a fresh checkout with no frontend
build still answers something useful.

## Language

The interface is bilingual and the backend does not know which language the
reader chose, because the choice is made in the browser and can change without
a request. Every user facing string is therefore carried in both languages.

Three mechanisms carry a translation:

- A [`Text`] value holds a Chinese and an English string and is encoded as
  `{"zh": ..., "en": ...}`. The tool registry uses it for every label, the
  graph data uses it for every node and edge caption, and the chapter list
  uses it for every chapter title.
- A document is written once in English and translated as a second file whose
  name carries the `.zh.md` suffix. The reader's language selects the form;
  a document without a translation is shown in English, with a note in the
  page that says so.
- A tool result label is written in English in the handler and translated by
  lookup in `src/tools/labels.rs`. A label that is absent from the table is a
  symbol, such as `position.x` or `BSTAR`, and is shown as it is in both
  languages, which is what a reader expects for a name.

The default language is Chinese. The choice is stored in `localStorage` and
reflected in the `lang` attribute of the document element.

Two things stay in English in both languages. Units are written with the
symbols the standards use, and interface items keep the names they have in
the library source, because a translated identifier would not match the code
it documents.

## Command line

The listener is bound from `--host` and `--port`, or from the combined
`--address`. The host may be a literal, an IPv6 literal, or a name the
operating system resolves; the name is resolved once, before the listener is
bound, so that a resolution failure is reported as a configuration error
rather than as a bind failure.

`--address` cannot be combined with `--host` or `--port`. Accepting both
would require an order of application that is not defined by the command
line, and the resulting behaviour would be surprising.

## Interface layout

These layout decisions keep the pages coherent at any window width.

A page is a **reading column**. Most pages are text, a card grid, or a form,
and all three read better in a centred column than across a whole wide
screen, so the content is capped at a measure and centred. A document is the
one page that is wider than the column, because it carries a table of
contents beside the text; the table of contents is part of the cap, and a
document with no headings drops the column and uses the reading measure on
its own. A graph breaks the rule the other way: it uses the whole width,
because a wide screen should show more of the graph rather than more margin.

The **interface index** shows one line per crate rather than the full module
documentation. A crate summary is several paragraphs, and a grid of them is
unreadable; the index shows the first sentence and the crate page shows the
rest. A doc comment is rendered as Markdown by the backend, so a comment
that contains a list, a table, or a code block is drawn as one, and the
frontend does not have to parse it.

The **graph** is drawn on a fixed layout space that is scaled to the
viewport. A double click opens the document or the interface item a node
names, so a graph is also a way into the documentation.



A grid of cards has **one card size on the whole page**, not one per row.

A grid makes the cards of a single row equal, which is not enough: each row is
measured on its own, so a page built from several grids steps from one row
height to the next and the column of cards looks ragged. The tallest card
across every grid is therefore measured in the browser and applied to all of
them, with a `ResizeObserver` so that the measurement follows the column
count and the language.

The columns are **a fixed count rather than as many as fit**. Sizing columns
with `auto-fill` makes the column width depend on how many cards are in the
row, so a section of three and a section of four produce different card
widths on the same page. A fixed count keeps one rhythm down the page; a
section that does not fill a row leaves the gap at the end instead of
stretching its cards.

A card is a column, so its **action sits on the bottom line of every card**
whatever the length of the text above it. The summary is not truncated: a
uniform height means a card with a longer summary simply carries more lines.

The space between cards comes from the grid `gap` **only**. The sibling
margin that stacks cards outside a grid is cancelled inside one, because a
card with a preceding sibling would otherwise carry both a row gap and a
sibling margin. The first card of a row has no preceding sibling, so it
would sit flush while the rest of the row sank by one gap, which reads as a
misalignment even though every card is the same size.

The sidebar is **a sticky element inside a frame**. The frame carries the
surface and the rule so the column is drawn for the whole page; the sidebar
itself is `sticky` and scrolls. Without the frame the column would end one
viewport down and read as cut off on a long page.

The graph canvas takes the **aspect of the layout space**, so the drawing
fills it instead of being letterboxed inside it. The force simulation
settles into a roughly round cluster while the canvas is wide, so the layout
is widened to the canvas shape before it is fitted; the change is a uniform
scale, which leaves relative distances readable.

## Anchors and outlines

A heading anchor is reduced from the heading text, and a table of contents
carries that anchor. Two rules make the link land where the reader expects.

The reduction **keeps the letters of every script**. Keeping only the ASCII
ones meant every Chinese heading reduced to the same fallback, so the whole
table of contents pointed at the first section and clicking an entry always
scrolled to the top of the document. The document text and the outline are
reduced by the same rule, so an entry names an anchor the document carries.

A repeated heading is **numbered on both sides**. Two headings with the same
text would otherwise share one anchor, and the second entry would scroll to
the first. The document and the outline number the repeat the same way, so
the mapping stays exact.

An outline carries the **level** of each heading, so a nested heading is
indented under its parent rather than listed beside it. A fold control is
shown only for a heading that has a child, because a control that hides
nothing is worse than no control.

## Language coverage

A user facing string falls into three classes, and each is handled
differently.

Interface chrome — navigation, headings, buttons, help text — is a
hand written dictionary in `frontend/src/lib/i18n.ts`, keyed by a stable
name. It covers the graph group captions and the interface item kinds, so a
Chinese reader sees a Chinese legend and Chinese group headings.

Authored content — concept documents, crate summaries, item documentation —
is a separate file per language, so the backend serves both and the reader
switches without another request. A document declares its own tags in each
language file under the same key, so the tag list follows the language too.

Derived text — a one line summary used in a card — is produced by the
backend from the document, and the renderer resolves the rustdoc forms a doc
comment uses. A shortcut reference such as `[`Vec3`]` is reduced to the item
name; without that, the brackets and backticks appear verbatim in the
summary.

## Caching

The service serves one binary that is replaced wholesale on an upgrade, so
the only question caching has to answer is: can a browser be holding a copy
that is no longer the one this binary embeds?

The build answers it by putting a content hash in every asset name. A build
that changes a file also changes that file's name, so the URL of the new
bundle is one the browser has never requested and therefore cannot have
cached. The service then states the policy that follows from the name alone:

- A file whose name carries a hash is `public, max-age=31536000,
  immutable`. The file is kept for a year and never revalidated, because
  nothing will ask for that name again.
- The entry point is `no-cache, must-revalidate`. It names the current
  bundle, so it is never served from a cache without asking, and the
  requirement is stated to intermediaries as well as to the browser.
- A file whose name carries no hash is `no-cache, must-revalidate`, because
  a later build reuses that name.

The decision reads the name only, so it is correct for a file embedded by an
earlier build as well as for one embedded by the current build.

Two alternatives were rejected. `no-store` on every file removes the
question at the cost of re-downloading the bundle on every page load.
`no-cache` on every file is correct but, without a conditional request
handler on the service, makes the browser re-download content it could have
reused. Neither is as fast as naming the content.

The service registers no service worker and emits no manifest, so there is
no second cache layer to reason about.

## Tool routes

The tool route has two states. Without a tool identifier the page is the
index: one card per tool, grouped by category, each card opening the tool.
With an identifier the page is that tool on its own.

The two states are one route rather than two pages. A single route means the
back button, a shared link, and a reload all behave the same way whether the
reader arrived from the index or from a link elsewhere. The tool page carries
a link back to the index, because the index is not on it.

The tool page holds no list. That is the point of the split: a reader who
opens a link to a tool sees the form rather than the list it was reached
from, and the form gets the whole width. The cost is that comparing two
tools takes two round trips through the index.

## Testing

No test code lives in `src/`. Every test is an integration test in `tests/`,
as it is in `apsis-rs`.

The service is a library with a thin binary over it, which is what makes that
possible: an integration test is a separate crate and reaches only what the
library makes public, so the modules are in `src/lib.rs` and `main.rs` is
just the entry point. The tests cover the parser, the codec, the Markdown
renderer, the formula renderer, the document model, the search tokenizer, the
graph parsing, the router, and every tool handler.

A handful of items are internal to a module and a test reaches them
nonetheless. Those stay `pub(crate)` where they are defined and are offered
through `src/testkit.rs`, which is the one place a caller is expected to look
for them. A wrapper there is also the only way to offer a private thing
outward at all: Rust does not let visibility be widened twice, so a
`pub(crate)` item cannot be re-exported as `pub`. That keeps the seams in one
file instead of scattering them through the modules being tested, and it
leaves the internals free to change shape behind the wrapper.

The end to end tests in `tests/service.rs` start the built binary on an
ephemeral port and make plain TCP requests against it. They exercise the
request parser, the router, the encoded payloads, the embedded content, and
the tool handlers together, which no unit test can do.

One assertion is worth calling out: the state test checks that
`Tree::missing_links` is empty and that the graph has no dangling edge. Those
two checks turn a content mistake into a test failure.
