<script lang="ts">
  /**
   * The knowledge graph page.
   *
   * The concept graph and the module structure graph are shown as two
   * selectable views. Selecting a node shows its summary and the links to
   * its document and its interface item.
   */
  import type { Graph, GraphNode, ApiIndex } from '../lib/types';
  import { getJson } from '../lib/api';
  import { navigate } from '../lib/router';
  import { groupColor } from '../lib/format';
  import { groupLabel, language, pick, tr } from '../lib/i18n';
  import GraphView from '../lib/GraphView.svelte';

  interface Props {
    concepts: Graph | null;
    api: ApiIndex | null;
  }

  let { concepts, api }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  let view = $state<'concepts' | 'modules'>('concepts');
  let moduleGraph = $state<Graph | null>(null);
  let selected = $state<string | null>(null);
  let loadingModules = $state(false);

  /** The graph currently displayed. */
  const graph = $derived(view === 'concepts' ? concepts : moduleGraph);

  /** The selected node, when the view supplies one. */
  const selectedNode = $derived(
    graph && selected ? (graph.nodes.find((node) => node.id === selected) ?? null) : null,
  );

  /** Loads the module structure graph the first time it is shown. */
  async function showModules() {
    view = 'modules';
    selected = null;
    if (moduleGraph || loadingModules) return;
    loadingModules = true;
    try {
      moduleGraph = await getJson<Graph>('/api/graph/modules');
    } finally {
      loadingModules = false;
    }
  }

  /** Selects a node and shows its detail. */
  function select(node: GraphNode) {
    selected = node.id;
  }

  /** Opens the document or the interface item a node refers to. */
  function open(node: GraphNode) {
    if (node.document) {
      navigate(`/docs/${node.document}`);
    } else if (node.api) {
      navigate(`/api/${node.api}`);
    }
  }
</script>

<div class="page-header">
  <h1>{tr('graph.title', lang)}</h1>
  <p>{tr('graph.intro', lang)}</p>
</div>

<div class="graph-controls">
  <button
    class={view === 'concepts' ? '' : 'ghost'}
    type="button"
    onclick={() => {
      view = 'concepts';
      selected = null;
    }}>{tr('graph.concepts', lang)}</button>
  <button class={view === 'modules' ? '' : 'ghost'} type="button" onclick={showModules}>
    {tr('graph.modules', lang)}
  </button>
  {#if loadingModules && !moduleGraph}
    <span class="badge"><span class="spinner"></span> {tr('common.loading', lang)}</span>
  {/if}
</div>

{#if !graph}
  <p class="empty">{tr('graph.loading', lang)}</p>
{:else}
  <GraphView {graph} {selected} onselect={select} onopen={open} />

  <div class="legend">
    {#each graph.groups as group}
      <span><i style={`background:${groupColor(group)}`}></i>{groupLabel(group, lang)}</span>
    {/each}
  </div>

  {#if selectedNode}
    <section class="card" style="margin-top: 1rem;">
      <header style="display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap;">
        <h3 style="margin: 0;">{pick(selectedNode.label, lang)}</h3>
        <span class="badge">{selectedNode.kind}</span>
        <span class="badge" style={`color:${groupColor(selectedNode.group)}`}
          >{groupLabel(selectedNode.group, lang)}</span>
      </header>
      <p style="color: var(--text-muted);">{pick(selectedNode.summary, lang)}</p>
      <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
        {#if selectedNode.document}
          <button
            class="secondary"
            type="button"
            onclick={() => navigate(`/docs/${selectedNode.document}`)}>
            {tr('graph.openDocument', lang)}
          </button>
        {/if}
        {#if selectedNode.api}
          <button class="ghost" type="button" onclick={() => navigate(`/api/${selectedNode.api}`)}>
            {tr('graph.interface', lang)}{selectedNode.api}
          </button>
        {/if}
      </div>
    </section>
  {/if}
{/if}

{#if api && view === 'modules'}
  <div class="notice muted">
    {tr('graph.moduleNote', lang)}
    {api.modules.map((module) => module.name).join(', ')}。
  </div>
{/if}
