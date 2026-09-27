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

/** One language form of a document. */
export interface DocumentForm {
  language: 'zh' | 'en';
  title: string;
  summary: string;
  html: string;
  markdown: string;
  headings: { anchor: string; text: string }[];
}

/** One document, with a rendered form per embedded language. */
export interface Document {
  id: string;
  chapter: string;
  tags: string[];
  prerequisites: string[];
  references: string[];
  forms: DocumentForm[];
}

/** The document listing, grouped by chapter. */
export interface DocumentIndex {
  document_count: number;
  chapters: {
    id: string;
    title: Bilingual;
    summary: Bilingual;
    documents: { id: string; title: Bilingual; summary: Bilingual; tags: string[] }[];
  }[];
}

/** One search result. */
export interface SearchHit {
  id: string;
  language: 'zh' | 'en';
  title: string;
  summary: string;
  chapter: string;
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

/** A graph. */
export interface Graph {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  groups: string[];
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/** A crate of the interface catalog. */
export interface ApiCrateSummary {
  name: string;
  layer: string;
  summary: string;
  /** The module documentation, with the Markdown markup removed. */
  summary_plain: string;
  items: number;
}

/** One interface item. */
export interface ApiItem {
  kind: string;
  name: string;
  signature: string;
  /** The documentation comment as it is written in the source. */
  doc: string;
  /** The documentation comment rendered as HTML by the backend. */
  doc_html: string;
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
  summary: string;
  summary_plain: string;
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
