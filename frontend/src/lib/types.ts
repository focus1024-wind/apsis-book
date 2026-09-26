/**
 * The types of the payloads the service returns.
 *
 * They mirror the JSON the Rust layer encodes. The service is the authority
 * on these shapes; a change there is a change here.
 */

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

/** One document, with its rendered HTML. */
export interface Document {
  id: string;
  title: string;
  chapter: string;
  summary: string;
  tags: string[];
  prerequisites: string[];
  references: string[];
  headings: { anchor: string; text: string }[];
  html: string;
  markdown: string;
}

/** The document listing, grouped by chapter. */
export interface DocumentIndex {
  document_count: number;
  chapters: {
    id: string;
    title: string;
    summary: string;
    documents: { id: string; title: string; summary: string; tags: string[] }[];
  }[];
}

/** One search result. */
export interface SearchHit {
  id: string;
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
  label: string;
  kind: string;
  group: string;
  summary: string;
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
  title: string;
  summary: string;
  groups: string[];
  nodes: GraphNode[];
  edges: GraphEdge[];
}

/** A crate of the interface catalog. */
export interface ApiCrateSummary {
  name: string;
  layer: string;
  summary: string;
  items: number;
}

/** One interface item. */
export interface ApiItem {
  kind: string;
  name: string;
  signature: string;
  doc: string;
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
  items: ApiItem[];
}

/** A tool parameter constraint. */
export interface ParamConstraints {
  unit?: string;
  min?: number | null;
  max?: number | null;
  options?: { value: string; label: string }[];
}

/** One tool parameter. */
export interface ToolParam {
  name: string;
  label: string;
  help: string;
  kind: 'number' | 'integer' | 'text' | 'multiline' | 'select';
  required: boolean;
  default: string | null;
  constraints: ParamConstraints;
}

/** One tool. */
export interface Tool {
  id: string;
  name: string;
  summary: string;
  description: string;
  category: string;
  params: ToolParam[];
  api_items: string[];
}

/** The tool registry. */
export interface ToolRegistry {
  categories: string[];
  tools: Tool[];
}

/** A labelled scalar in a tool result. */
export interface ResultScalar {
  label: string;
  value: string;
  unit: string;
  monospace: boolean;
}

/** A result table. */
export interface ResultTable {
  columns: { key: string; label: string; unit: string; numeric: boolean }[];
  rows: (string | number | null)[][];
}

/** A numeric series. */
export interface ResultSeries {
  name: string;
  unit: string;
  values: number[];
}

/** The result of a tool. */
export interface ToolResult {
  kind: string;
  title: string;
  scalars: ResultScalar[];
  table: ResultTable;
  series: ResultSeries[];
  notes: string[];
  axis?: { name: string; unit: string; values: number[] };
}

/** The envelope of a tool call. */
export interface ToolRunResponse {
  ok: boolean;
  tool: string;
  result?: ToolResult;
  error?: { parameter: string | null; message: string };
}
