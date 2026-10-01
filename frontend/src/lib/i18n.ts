/**
 * Language selection for the interface.
 *
 * The service is bilingual. The backend sends every user facing string in
 * both languages, so switching the language needs no request. The choice
 * lives in a store that persists to `localStorage` and is reflected in the
 * `lang` attribute of the document element.
 *
 * The default is Chinese. A stored choice wins over the default; a stored
 * value that is not a supported language is ignored.
 */

import { writable, get } from 'svelte/store';

/** A language tag. */
export type Language = 'zh' | 'en';

/** The language the interface opens in. */
export const DEFAULT_LANGUAGE: Language = 'zh';

/** The key the choice is stored under. */
const STORAGE_KEY = 'apsis-book-language';

/** A string carried in both languages. */
export interface Text {
  zh: string;
  en: string;
}

/** A list of strings carried in both languages. */
export interface BilingualList {
  zh: string[];
  en: string[];
}

/** Returns `true` when the value is a supported language tag. */
export function isLanguage(value: unknown): value is Language {
  return value === 'zh' || value === 'en';
}

/** Reads the stored language, or the default. */
function initialLanguage(): Language {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (isLanguage(stored)) return stored;
  } catch {
    // A blocked or absent store is not an error; the default applies.
  }
  return DEFAULT_LANGUAGE;
}

/** The current language. */
export const language = writable<Language>(initialLanguage());

/** Applies a language to the store, the storage, and the document. */
export function setLanguage(next: Language): void {
  language.set(next);
  try {
    window.localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // Persisting is best effort.
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = next === 'zh' ? 'zh-CN' : 'en';
  }
}

language.subscribe(() => {
  // The subscription keeps the document attribute in step with the store
  // even when the store is set directly.
});

/** Returns the current language. */
export function currentLanguage(): Language {
  return get(language);
}

/** Picks a string from a bilingual value sent by the backend. */
export function pick(value: Text | null | undefined, target?: Language): string {
  if (!value) return '';
  const selected = target ?? get(language);
  if (selected === 'zh') return value.zh || value.en;
  return value.en || value.zh;
}

/** Picks a list from a bilingual value that may be empty in one language. */
export function pickList(
  value: BilingualList | null | undefined,
  target?: Language,
): string[] {
  if (!value) return [];
  const selected = target ?? get(language);
  // A list can be empty in one language and filled in the other, so the
  // length is checked rather than the truthiness of the array.
  if (selected === 'zh') return value.zh?.length ? value.zh : (value.en ?? []);
  return value.en?.length ? value.en : (value.zh ?? []);
}

/** The interface strings, keyed by a stable name. */
const STRINGS: Record<string, Text> = {
  'app.title': { zh: 'APSIS', en: 'APSIS' },
  'app.subtitle': { zh: 'ApsisRS 计算内核', en: 'ApsisRS computational kernels' },
  'nav.explore': { zh: '浏览', en: 'Explore' },
  'nav.overview': { zh: '总览', en: 'Overview' },
  'nav.docs': { zh: '概念文档', en: 'Concepts' },
  'nav.graph': { zh: '知识图谱', en: 'Knowledge graph' },
  'nav.flows': { zh: '算法流程', en: 'Algorithm flows' },
  'nav.api': { zh: '接口文档', en: 'Interface' },
  'nav.tools': { zh: '在线工具', en: 'Online tools' },
  'nav.documents': { zh: '文档', en: 'Documents' },
  'nav.toggleChapter': { zh: '展开或折叠该章', en: 'Expand or collapse the chapter' },
  'search.placeholder': { zh: '搜索文档', en: 'Search the documentation' },
  'search.button': { zh: '搜索', en: 'Search' },
  'search.title': { zh: '搜索', en: 'Search' },
  'search.results': { zh: '结果', en: 'Results' },
  'search.of': { zh: '共', en: 'of' },
  'search.documents': { zh: '篇文档', en: 'documents' },
  'search.enter': { zh: '请在上方搜索框输入关键词。', en: 'Enter a term in the search box above.' },
  'search.none': { zh: '没有文档包含该关键词。', en: 'No document contains that term.' },
  'search.searching': { zh: '搜索中…', en: 'Searching…' },
  'search.hint': {
    zh: '索引覆盖每篇概念文档的标题与正文。',
    en: 'The index covers the titles and bodies of every concept document.',
  },
  'search.score': { zh: '相关度', en: 'score' },

  'home.intro': {
    zh: 'ApsisRS 计算内核的 Web 服务。它解释这些内核实现的概念，按源码展示库的接口，并在你给定的参数上运行这些内核。',
    en: 'The web service for the ApsisRS computational kernels. It explains the concepts the kernels implement, shows the interface of the library as it is written in its source, and runs the kernels on parameters you supply.',
  },
  'home.docs': { zh: '概念文档', en: 'Concepts' },
  'home.docs.body': {
    zh: '内核计算的是什么：时间尺度、参考系、元素格式与传播模型，并给出各自的单位与适用区间。',
    en: 'What the kernels compute: the time scales, the reference frames, the element formats, and the propagation models, with the units and the applicable range of each.',
  },
  'home.docs.action': { zh: '打开文档', en: 'Open the documentation' },
  'home.graph': { zh: '知识图谱', en: 'Knowledge graph' },
  'home.graph.body': {
    zh: '以图谱呈现概念及其关系，以及库的 crate 结构。每个节点可打开解释它的文档。',
    en: 'The concepts as a graph, with the relationships between them, and the crate structure of the library. Every node opens the document that explains it.',
  },
  'home.graph.action': { zh: '打开图谱', en: 'Open the graph' },
  'home.api': { zh: '接口文档', en: 'Interface' },
  'home.api.body': {
    zh: '从源码文档注释中提取的 ApsisRS 轨道层公共接口，含签名、单位与错误情形。',
    en: 'The public interface of the ApsisRS orbit layer, extracted from the source doc comments, with the signature, the units, and the error cases of each item.',
  },
  'home.api.action': { zh: '打开接口文档', en: 'Open the interface' },
  'home.tools': { zh: '在线工具', en: 'Online tools' },
  'home.tools.body': {
    zh: '用你自己的输入运行内核：解析两行元素集、传播轨道、转换时间尺度、转换坐标等。',
    en: 'Run the kernels on your own input: parse a two-line element set, propagate an orbit, convert a time scale, convert coordinates, and more.',
  },
  'home.tools.action': { zh: '打开工具', en: 'Open the tools' },
  'home.metric.documents': { zh: '概念文档', en: 'concept documents' },
  'home.metric.graph': { zh: '图谱节点', en: 'graph nodes' },
  'home.metric.crates': { zh: '文档化的 crate', en: 'crates documented' },
  'home.metric.items': { zh: '公共项', en: 'public items' },
  'home.metric.tools': { zh: '在线工具', en: 'online tools' },
  'home.metric.version': { zh: '服务版本', en: 'service version' },

  'docs.title': { zh: '概念文档', en: 'Concepts' },
  'docs.intro': {
    zh: '系统性地描述 ApsisRS 内核所实现的概念，同时列出未实现的其他方法及其对比。',
    en: 'A systematic description of the concepts the ApsisRS kernels implement, together with the methods they do not implement and how the alternatives compare.',
  },
  'docs.read': { zh: '阅读', en: 'Read' },
  'docs.loading': { zh: '正在加载文档列表…', en: 'Loading the documentation listing…' },
  'docs.breadcrumb': { zh: '概念文档', en: 'Concepts' },
  'docs.noSelection': { zh: '未选择文档。', en: 'No document selected.' },
  'docs.notFound': { zh: '不存在标识为', en: 'There is no document with the identifier' },
  'docs.back': { zh: '返回列表', en: 'Back to the listing' },
  'docs.readFirst': { zh: '先读：', en: 'Read first:' },
  'docs.onThisPage': { zh: '本页目录', en: 'On this page' },
  'docs.toggle': { zh: '展开或折叠该节', en: 'Expand or collapse the section' },
  'docs.noSections': { zh: '无小节。', en: 'No sections.' },
  'docs.translationMissing': {
    zh: '本文档暂无中文版本，显示英文原文。',
    en: 'This document has no Chinese translation; the English text is shown.',
  },
  'docs.otherLanguage': { zh: 'English', en: '中文' },
  'docs.otherLanguageTitle': { zh: 'Switch to English', en: '切换到中文' },

  'graph.title': { zh: '知识图谱', en: 'Knowledge graph' },
  'graph.intro': {
    zh: '概念及其相互关系。拖动平移，滚轮缩放，单击节点查看说明，双击打开解释它的文档。',
    en: 'The concepts and the relationships between them. Drag to pan, scroll to zoom, click a node to see its description, and double click to open the document that explains it.',
  },
  'graph.concepts': { zh: '概念图谱', en: 'Concepts' },
  'graph.crates': { zh: 'crate 结构', en: 'Crate structure' },
  'graph.loading': { zh: '正在加载图谱…', en: 'Loading the graph…' },
  'graph.nodes': { zh: '节点', en: 'nodes' },
  'graph.edges': { zh: '边', en: 'edges' },
  'graph.zoom': { zh: '缩放', en: 'zoom' },
  'graph.reset': { zh: '复位视图', en: 'Reset view' },
  'graph.fit': { zh: '适应窗口', en: 'Fit to view' },
  'graph.openDocument': { zh: '打开文档', en: 'Open the document' },
  'graph.interface': { zh: '接口：', en: 'Interface:' },
  'graph.crateNote': {
    zh: 'crate 结构图由提取出的接口目录生成，因此描述的是本次构建启用的层集合：',
    en: 'The crate graph is generated from the extracted interface catalog, so it describes the layer set this build enables:',
  },

  'flows.title': { zh: '算法流程', en: 'Algorithm flows' },
  'flows.intro': {
    zh: '内核所实现算法的各个步骤。每一步标出执行它的接口项；单击查看说明，双击打开接口文档。',
    en: 'The steps of the algorithms the kernels implement. Each step names the interface item that performs it; click a step to see its description and double click to open the interface documentation.',
  },
  'flows.loading': { zh: '正在加载流程…', en: 'Loading the flow…' },

  'api.title': { zh: '接口文档', en: 'Interface documentation' },
  'api.intro': {
    zh: '本服务所基于的 ApsisRS 各层的公共接口，从源码文档注释中提取。签名按原样展示，各单位与错误情形见各项的文档说明。',
    en: 'The public interface of the ApsisRS layers this service builds against, extracted from the source doc comments. Signatures are shown as they are written, with the units and the error cases stated in the documentation of each item.',
  },
  'api.loading': { zh: '正在加载接口目录…', en: 'Loading the interface catalog…' },
  'api.items': { zh: '项', en: 'items' },
  'api.find': { zh: '查找接口项', en: 'Find an item' },
  'api.filter': { zh: '按名称或 crate 过滤公共项', en: 'Filter the public items by name or crate' },
  'api.noMatch': { zh: '没有匹配的项。', en: 'No item matches that filter.' },
  'api.associated': { zh: '关联项', en: 'associated' },
  'api.noDoc': { zh: '该项没有文档注释。', en: 'This item has no documentation comment.' },
  'api.detail': { zh: '详情', en: 'Detail' },
  'api.noItem': { zh: '不存在接口项', en: 'There is no interface item named' },
  'api.crates': { zh: 'crate', en: 'Crates' },
  'api.loadingItem': { zh: '正在加载…', en: 'Loading…' },

  'tools.title': { zh: '在线工具', en: 'Online tools' },
  'tool.plural': { zh: '工具', en: 'tools' },
  'tools.intro': {
    zh: '用你自己的输入运行 ApsisRS 内核。每个工具都调用文档所述的同一个库，每个结果都给出各量的单位。',
    en: 'Run the ApsisRS kernels on your own input. Every tool calls the same library the documentation describes, and every result reports the units of each quantity.',
  },
  'tools.loading': { zh: '正在加载工具列表…', en: 'Loading the tool registry…' },
  'tools.select': { zh: '在上方选择一个工具以打开其表单。', en: 'Select a tool above to open its form.' },
  'tools.run': { zh: '运行', en: 'Run' },
  'tools.running': { zh: '运行中…', en: 'Running…' },
  'tools.reset': { zh: '重置', en: 'Reset' },
  'tools.calls': { zh: '调用：', en: 'Calls:' },
  'tools.unit': { zh: '单位', en: 'unit' },
  'tools.required': { zh: '必填', en: 'required' },
  'tools.error': { zh: '错误', en: 'Error' },
  'tools.noResult': { zh: '工具未返回结果。', en: 'the tool did not return a result' },
  'tools.requestFailed': { zh: '请求无法完成。', en: 'the request could not be completed' },

  'result.noSeries': { zh: '没有可绘制的数据序列。', en: 'No series to plot.' },
  'result.empty': { zh: '空', en: 'Empty' },

  'common.loading': { zh: '加载中…', en: 'Loading…' },
  'common.failed': {
    zh: '服务内容无法加载。刷新页面重试。',
    en: 'The service content could not be loaded. Reload the page to try again.',
  },
  'common.notFound': { zh: '不存在页面', en: 'There is no page at' },
  'common.goHome': { zh: '前往总览', en: 'Go to the overview' },
  'common.language': { zh: '语言', en: 'Language' },

  // Group captions of the knowledge graph and the flow graphs. The keys are
  // the group identifiers used by the content data.
  'group.overview': { zh: '总览', en: 'Overview' },
  'group.time': { zh: '时间', en: 'Time' },
  'group.geometry': { zh: '坐标与几何', en: 'Geometry' },
  'group.elements': { zh: '元素集', en: 'Elements' },
  'group.propagation': { zh: '轨道传播', en: 'Propagation' },
  'group.methods': { zh: '方法对比', en: 'Methods' },
  'group.foundation': { zh: '基础层', en: 'Foundation' },
  'group.physics': { zh: '物理层', en: 'Physics' },
  'group.orbit': { zh: '轨道层', en: 'Orbit' },
  'group.rf': { zh: '射频层', en: 'Radio' },
  'group.network': { zh: '网络层', en: 'Network' },
  'group.ops': { zh: '运行层', en: 'Operations' },
  'group.sim': { zh: '仿真层', en: 'Simulation' },
  'group.input': { zh: '输入', en: 'Input' },
  'group.parse': { zh: '解析', en: 'Parse' },
  'group.prepare': { zh: '准备', en: 'Prepare' },
  'group.branch': { zh: '分支', en: 'Branch' },
  'group.propagate': { zh: '传播', en: 'Propagate' },
  'group.verify': { zh: '校验', en: 'Verify' },
  'group.fields': { zh: '字段', en: 'Fields' },
  'group.crosscheck': { zh: '交叉检查', en: 'Cross-check' },
  'group.error': { zh: '错误', en: 'Error' },
  'group.output': { zh: '输出', en: 'Output' },
  'group.validate': { zh: '校验', en: 'Validate' },
  'group.convert': { zh: '换算', en: 'Convert' },
  'group.store': { zh: '存储', en: 'Store' },
  'group.frames': { zh: '参考系', en: 'Frames' },
  'group.rotate': { zh: '旋转', en: 'Rotate' },
  'group.geodetic': { zh: '大地坐标', en: 'Geodetic' },
  'group.general': { zh: '其他', en: 'General' },

  // Item kinds of the interface documentation.
  'kind.struct': { zh: '结构体', en: 'struct' },
  'kind.enum': { zh: '枚举', en: 'enum' },
  'kind.trait': { zh: 'trait', en: 'trait' },
  'kind.fn': { zh: '函数', en: 'fn' },
  'kind.const': { zh: '常量', en: 'const' },
  'kind.type': { zh: '类型别名', en: 'type' },

  'api.itemCount': { zh: '个接口项', en: 'items' },
  'tools.open': { zh: '打开', en: 'Open' },
  'tools.choose': { zh: '共 {count} 个工具，按类别分组。', en: '{count} tools, grouped by category.' },
};

/** Returns an interface string in the given language. */
export function tr(key: string, target?: Language): string {
  const entry = STRINGS[key];
  if (!entry) return key;
  return pick(entry, target);
}

/** Returns an interface string, reactive to the language store. */
export function t(key: string): string {
  const entry = STRINGS[key];
  if (!entry) return key;
  return get(language) === 'zh' ? entry.zh : entry.en;
}

/**
 * Returns the caption of a graph group.
 *
 * A group that is not in the dictionary is shown as written, which keeps a
 * new group in the content readable rather than blank.
 */
export function groupLabel(group: string, target?: Language): string {
  const entry = STRINGS[`group.${group}`];
  if (!entry) return group;
  return pick(entry, target);
}

/**
 * Returns the caption of an interface item kind.
 */
export function kindLabel(kind: string, target?: Language): string {
  const entry = STRINGS[`kind.${kind}`];
  if (!entry) return kind;
  return pick(entry, target);
}

/**
 * The headings that introduce a group of interface items, in the plural.
 */
export const KIND_GROUPS: Record<string, Text> = {
  struct: { zh: '结构体', en: 'Structs' },
  enum: { zh: '枚举', en: 'Enums' },
  trait: { zh: 'trait', en: 'Traits' },
  fn: { zh: '函数', en: 'Functions' },
  const: { zh: '常量', en: 'Constants' },
  type: { zh: '类型别名', en: 'Type aliases' },
};

export default t;
