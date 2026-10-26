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
  api_modules: number;
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

/** One subject within a layer. */
export interface DocumentSubsection {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  documents: DocumentSummary[];
}

/**
 * A layer inside a group.
 *
 * A layer that covers two unrelated subjects, such as the physics layer,
 * carries a subsection for each so that the two are not presented as one
 * list. `documents` is then empty and the documents live in the
 * subsections.
 */
export interface DocumentSection {
  id: string;
  title: Bilingual;
  summary: Bilingual;
  documents: DocumentSummary[];
  subsections: DocumentSubsection[];
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

/** A module of the interface catalog. */
export interface ApiModuleSummary {
  name: string;
  layer: string;
  summary: BilingualSummary;
  items: number;
}

/** A module summary in both languages, with the plain and rendered forms. */
export interface BilingualSummary {
  zh: string;
  en: string;
  plain: Bilingual;
  html: Bilingual;
  /**
   * The opening sentence, for a card that links onward. The rest of the
   * summary belongs on the page the card leads to.
   */
  brief?: Bilingual;
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

/** One layer of the library, used to organize the interface index. */
export interface ApiLayer {
  id: string;
  title: Bilingual;
  summary: Bilingual;
}

/** The interface catalog index. */
export interface ApiIndex {
  generated_from: string[];
  module_count: number;
  item_count: number;
  /** The layers, in reading order, that the modules are grouped by. */
  layers: ApiLayer[];
  modules: ApiModuleSummary[];
  items: { name: string; kind: string; module: string }[];
}

/** One module with its items. */
export interface ApiModule {
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
  /** The help rendered from Markdown, so that a formula in it is set. */
  help_html?: Bilingual;
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
  /** The description rendered from Markdown, so that a formula in it is set. */
  description_html?: Bilingual;
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
  /** The unit rendered, so that a power in it reads as a power. */
  unit_html?: string;
  monospace: boolean;
}

/** A column of a result table. */
export interface ResultColumn {
  key: string;
  label: Bilingual;
  unit: string;
  /** The unit rendered, so that a power in it reads as a power. */
  unit_html?: string;
  numeric: boolean;
}

/**
 * One cell of a result table.
 *
 * A cell is usually a number. A text cell may carry a translation, because a
 * cell holding a fixed phrase, such as the meaning of a field, is written
 * once and shown in the reader's language. Such a cell may also carry the
 * text rendered, which is how a unit written with a power in it, such as
 * `rev/day^2`, is shown as a power.
 */
export type ResultCell = string | number | (Bilingual & { html?: string }) | null;

/** A result table. */
export interface ResultTable {
  columns: ResultColumn[];
  rows: ResultCell[][];
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
  /** The notes rendered from Markdown, which is how a formula in one is set. */
  notes_html?: Bilingual[];
  axis?: { name: Bilingual; unit: string; values: number[] };
}

/** The envelope of a tool call. */
export interface ToolRunResponse {
  ok: boolean;
  tool: string;
  result?: ToolResult;
  error?: { parameter: string | null; message: string };
}
