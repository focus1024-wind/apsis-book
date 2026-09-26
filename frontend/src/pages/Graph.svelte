<script lang="ts">
  /**
   * The knowledge graph page.
   *
   * The concept graph and the crate structure graph are shown side by side
   * as two selectable views. Selecting a node shows its summary and the
   * links to its document and its interface item.
   */
  import type { Graph, GraphNode, ApiIndex } from '../lib/types';
  import { getJson } from '../lib/api';
  import { navigate } from '../lib/router';
  import { groupColor } from '../lib/format';
  import GraphView from '../lib/GraphView.svelte';

  interface Props {
    concepts: Graph | null;
    api: ApiIndex | null;
  }

  let { concepts, api }: Props = $props();

  let view = $state<'concepts' | 'crates'>('concepts');
  let crateGraph = $state<Graph | null>(null);
  let selected = $state<string | null>(null);
  let loadingCrates = $state(false);

  /** The graph currently displayed. */
  const graph = $derived(view === 'concepts' ? concepts : crateGraph);

  /** The selected node, when the view supplies one. */
  const selectedNode = $derived(
    graph && selected ? graph.nodes.find((node) => node.id === selected) ?? null : null,
  );

  /** Loads the crate structure graph the first time it is shown. */
  async function showCrates() {
    view = 'crates';
    selected = null;
    if (crateGraph || loadingCrates) return;
    loadingCrates = true;
    try {
      crateGraph = await getJson<Graph>('/api/graph/crates');
    } finally {
      loadingCrates = false;
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
  <h1>Knowledge graph</h1>
  <p>
    The concepts and the relationships between them. Drag to pan, scroll to zoom, click a
    node to see its description, and double click to open the document that explains it.
  </p>
</div>

<div class="graph-controls">
  <button
    class={view === 'concepts' ? '' : 'ghost'}
    type="button"
    onclick={() => { view = 'concepts'; selected = null; }}>Concepts</button>
  <button
    class={view === 'crates' ? '' : 'ghost'}
    type="button"
    onclick={showCrates}>Crate structure</button>
  {#if loadingCrates && !crateGraph}
    <span class="badge"><span class="spinner"></span> loading</span>
  {/if}
</div>

{#if !graph}
  <p class="empty">Loading the graph…</p>
{:else}
  <GraphView {graph} {selected} onselect={select} onopen={open} />

  <div class="legend">
    {#each graph.groups as group}
      <span><i style={`background:${groupColor(group)}`}></i>{group}</span>
    {/each}
  </div>

  {#if selectedNode}
    <section class="card" style="margin-top: 1rem;">
      <header style="display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap;">
        <h3 style="margin: 0;">{selectedNode.label}</h3>
        <span class="badge">{selectedNode.kind}</span>
        <span class="badge" style={`color:${groupColor(selectedNode.group)}`}>{selectedNode.group}</span>
      </header>
      <p style="color: var(--text-muted);">{selectedNode.summary}</p>
      <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
        {#if selectedNode.document}
          <button class="secondary" type="button" onclick={() => navigate(`/docs/${selectedNode.document}`)}>
            Open the document
          </button>
        {/if}
        {#if selectedNode.api}
          <button class="ghost" type="button" onclick={() => navigate(`/api/${selectedNode.api}`)}>
            Interface: {selectedNode.api}
          </button>
        {/if}
      </div>
    </section>
  {/if}
{/if}

{#if api && view === 'crates'}
  <div class="notice muted">
    The crate graph is generated from the extracted interface catalog, so it describes the
    layer set this build enables: {api.crates.map((crate) => crate.name).join(', ')}.
  </div>
{/if}
