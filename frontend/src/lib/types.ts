/**
 * The types of the payloads the service returns.
 *
 * They mirror the JSON the Rust layer encodes. The service is the authority
 * on these shapes; a change there is a change here.
 *
 * A user facing string is a {@link Bilingual}, which carries both languages
 * so that the reader can switch without another request.
 */

/** A string carried in both languages. */
export interface Bilingual {
  zh: string;
  en: string;
}

/** A list of strings carried in both languages. */
export interface BilingualList {
  zh: string[];
  en: string[];
}

/** Service health. */
export interface Health {
  status: string;
  service: string;
  version: string;
  documents: number;
  chapters: number;
  graph_nodes: number;
  api_crates: number;
  api_items: number;
  tools: number;
  frontend_embedded: boolean;
}

/** One section heading of a document. */
export interface DocumentHeading {
  anchor: string;
  text: string;
  level: number;
}

/** One language form of a document. */
export interface DocumentForm {
  language: 'zh' | 'en';
  title: string;
  summary: string;
  html: string;
  markdown: string;
  /** Section headings, each with the anchor, text, and level. */
  headings: { anchor: string; text: string; level: number }[];
}

/** One document, with a rendered form per embedded language. */
export interface Document {
  id: string;
  chapter: string;
  prerequisites: string[];
  references: string[];
  forms: DocumentForm[];
  /** Tags in both languages. */
  tags: BilingualList;
}

/** One document in the listing. */
export interface DocumentSummary {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  tags: BilingualList;
}

/** A layer, or section, inside a group. */
export interface DocumentSection {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  documents: DocumentSummary[];
}

/**
 * The document listing.
 *
 * The top level is a group rather than a chapter: the concepts and the
 * implementation are described in different ways and a reader who needs one
 * should not have to read past the other.
 */
export interface DocumentIndex {
  document_count: number;
  groups: {
    id: string;
    title: Bilingual;
    sections: DocumentSection[];
  }[];
}

/** One search result. */
export interface SearchHit {
  id: string;
  language: 'zh' | 'en';
  title: string;
  summary: string;
  chapter: string;
  /** The chapter title in both languages, so a result is readable. */
  chapter_title: Bilingual;
  score: number;
}

/** A search response. */
export interface SearchResponse {
  query: string;
  count: number;
  indexed: number;
  hits: SearchHit[];
}

/** A graph node. */
export interface GraphNode {
  id: string;
  label: Bilingual;
  kind: string;
  group: string;
  summary: Bilingual;
  document: string | null;
  api: string | null;
}

/** A graph edge. */
export interface GraphEdge {
  source: string;
  target: string;
  kind: string;
  label: string | null;
}

/** An entry of the flow index. */
export interface FlowEntry {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  nodes: number;
  edges: number;
}

/** The flow index. */
export interface FlowIndex {
  flows: FlowEntry[];
}

/** A graph. */
export interface Graph {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  groups: string[];
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/**
 * A documentation text in both languages.
 *
 * The `plain` members carry the text with the Markdown markup removed, and
 * the `html` members carry it rendered. A language without a translation
 * falls back to English.
 */
export interface ApiText {
  zh: string;
  en: string;
  plain: Bilingual;
  html: Bilingual;
}

/** A crate of the interface catalog. */
export interface ApiCrateSummary {
  name: string;
  layer: string;
  summary: BilingualSummary;
  items: number;
}

/** A crate summary in both languages, with the plain and rendered forms. */
export interface BilingualSummary {
  zh: string;
  en: string;
  plain: Bilingual;
  html: Bilingual;
}

/** One interface item. */
export interface ApiItem {
  kind: string;
  name: string;
  signature: string;
  /** The documentation comment in both languages. */
  doc: ApiText;
  associated: boolean;
}

/** The interface catalog index. */
export interface ApiIndex {
  generated_from: string[];
  crate_count: number;
  item_count: number;
  crates: ApiCrateSummary[];
  items: { name: string; kind: string; crate: string }[];
}

/** One crate with its items. */
export interface ApiCrate {
  name: string;
  layer: string;
  summary: BilingualSummary;
  items: ApiItem[];
}

/** A tool parameter constraint. */
export interface ParamConstraints {
  unit?: string;
  min?: number | null;
  max?: number | null;
  options?: { value: string; label: Bilingual }[];
}

/** One tool parameter. */
export interface ToolParam {
  name: string;
  label: Bilingual;
  help: Bilingual;
  kind: 'number' | 'integer' | 'text' | 'multiline' | 'select';
  required: boolean;
  default: string | null;
  constraints: ParamConstraints;
}

/** One tool. */
export interface Tool {
  id: string;
  name: Bilingual;
  summary: Bilingual;
  description: Bilingual;
  category: Bilingual;
  params: ToolParam[];
  api_items: string[];
}

/** The tool registry. */
export interface ToolRegistry {
  categories: Bilingual[];
  tools: Tool[];
}

/** A labelled scalar in a tool result. */
export interface ResultScalar {
  label: Bilingual;
  value: string;
  unit: string;
  monospace: boolean;
}

/** A result table. */
export interface ResultTable {
  columns: { key: string; label: Bilingual; unit: string; numeric: boolean }[];
  rows: (string | number | null)[][];
}

/** A numeric series. */
export interface ResultSeries {
  name: Bilingual;
  unit: string;
  values: number[];
}

/** The result of a tool. */
export interface ToolResult {
  kind: string;
  title: Bilingual;
  scalars: ResultScalar[];
  table: ResultTable;
  series: ResultSeries[];
  notes: Bilingual[];
  axis?: { name: Bilingual; unit: string; values: number[] };
}

/** The envelope of a tool call. */
export interface ToolRunResponse {
  ok: boolean;
  tool: string;
  result?: ToolResult;
  error?: { parameter: string | null; message: string };
}
