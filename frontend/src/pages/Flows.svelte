<script lang="ts">
  /**
   * The algorithm flow graphs.
   *
   * Each flow describes the steps of one algorithm. Selecting a step shows
   * the interface item that implements it.
   */
  import type { Graph, GraphNode } from '../lib/types';
  import { getJson } from '../lib/api';
  import { groupColor } from '../lib/format';
  import GraphView from '../lib/GraphView.svelte';

  /** The flow identifiers, in display order. */
  const FLOWS = [
    { id: 'sgp4-propagation', label: 'SGP4 propagation' },
    { id: 'tle-parsing', label: 'Two-line element parsing' },
    { id: 'time-scale-conversion', label: 'Time scale conversion' },
    { id: 'frame-rotation', label: 'Frame rotation and geodetic conversion' },
  ];

  interface Props {
    id: string | null;
  }

  let { id }: Props = $props();

  let flows = $state<Record<string, Graph>>({});
  let loading = $state(false);
  let selected = $state<string | null>(null);

  /** The identifier currently shown. */
  const active = $derived(id ?? FLOWS[0].id);
  /** The graph currently shown. */
  const graph = $derived(flows[active] ?? null);

  /** The selected node. */
  const selectedNode = $derived(
    graph && selected ? graph.nodes.find((node) => node.id === selected) ?? null : null,
  );

  /** Loads the flow graphs that have not been fetched yet. */
  async function load() {
    loading = true;
    try {
      for (const flow of FLOWS) {
        if (flows[flow.id]) continue;
        flows[flow.id] = await getJson<Graph>(`/api/graph/${flow.id}`);
      }
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void load();
  });

  /** Selects a node. */
  function select(node: GraphNode) {
    selected = node.id;
  }

  /** Opens the interface item a step refers to. */
  function open(node: GraphNode) {
    if (node.api) {
      window.location.hash = `#/api/${node.api}`;
    }
  }
</script>

<div class="page-header">
  <h1>Algorithm flows</h1>
  <p>
    The steps of the algorithms the kernels implement. Each step names the interface item
    that performs it; click a step to see its description and double click to open the
    interface documentation.
  </p>
</div>

<div class="graph-controls">
  {#each FLOWS as flow}
    <a
      href={`#/flows/${flow.id}`}
      class={active === flow.id ? 'badge' : 'badge'}
      style={active === flow.id ? 'border-color: var(--accent); color: var(--accent);' : ''}
    >{flow.label}</a>
  {/each}
  {#if loading && !graph}
    <span class="badge"><span class="spinner"></span> loading</span>
  {/if}
</div>

{#if !graph}
  <p class="empty">Loading the flow…</p>
{:else}
  <p style="color: var(--text-muted);">{graph.summary}</p>
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
      </header>
      <p style="color: var(--text-muted);">{selectedNode.summary}</p>
      {#if selectedNode.api}
        <a href={`#/api/${selectedNode.api}`}>Interface: <code>{selectedNode.api}</code></a>
      {/if}
    </section>
  {/if}
{/if}
