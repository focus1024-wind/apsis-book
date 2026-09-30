<script lang="ts">
  /**
   * The algorithm flow graphs.
   *
   * Each flow describes the steps of one algorithm. Selecting a step shows
   * the interface item that implements it. The list of flows comes from the
   * backend, so adding a flow to the content is enough for it to appear
   * here.
   */
  import type { FlowIndex, Graph, GraphNode } from '../lib/types';
  import { getJson } from '../lib/api';
  import { groupColor } from '../lib/format';
  import { language, pick, tr } from '../lib/i18n';
  import GraphView from '../lib/GraphView.svelte';

  interface Props {
    id: string | null;
  }

  let { id }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  let flows = $state<Record<string, Graph>>({});
  let index = $state<FlowIndex['flows']>([]);
  let loading = $state(false);
  let selected = $state<string | null>(null);

  /** The identifier currently shown. */
  const active = $derived(id ?? index[0]?.id ?? '');
  /** The graph currently shown. */
  const graph = $derived(flows[active] ?? null);

  /** The selected node. */
  const selectedNode = $derived(
    graph && selected ? (graph.nodes.find((node) => node.id === selected) ?? null) : null,
  );

  /** Loads the flow index and any flow graph that has not been fetched. */
  async function load() {
    loading = true;
    try {
      if (index.length === 0) {
        index = (await getJson<FlowIndex>('/api/graph/flows')).flows;
      }
      for (const flow of index) {
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
  <h1>{tr('flows.title', lang)}</h1>
  <p>{tr('flows.intro', lang)}</p>
</div>

<div class="graph-controls">
  {#each index as flow}
    <a
      href={`#/flows/${flow.id}`}
      class="badge"
      style={active === flow.id ? 'border-color: var(--accent); color: var(--accent);' : ''}
      >{pick(flow.title, lang)}</a>
  {/each}
  {#if loading && !graph}
    <span class="badge"><span class="spinner"></span> {tr('common.loading', lang)}</span>
  {/if}
</div>

{#if !graph}
  <p class="empty">{tr('flows.loading', lang)}</p>
{:else}
  <p style="color: var(--text-muted);">{pick(graph.summary, lang)}</p>
  <GraphView {graph} {selected} onselect={select} onopen={open} />

  <div class="legend">
    {#each graph.groups as group}
      <span><i style={`background:${groupColor(group)}`}></i>{group}</span>
    {/each}
  </div>

  {#if selectedNode}
    <section class="card" style="margin-top: 1rem;">
      <header style="display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap;">
        <h3 style="margin: 0;">{pick(selectedNode.label, lang)}</h3>
        <span class="badge">{selectedNode.kind}</span>
      </header>
      <p style="color: var(--text-muted);">{pick(selectedNode.summary, lang)}</p>
      {#if selectedNode.api}
        <a href={`#/api/${selectedNode.api}`}>{tr('graph.interface', lang)}<code
            >{selectedNode.api}</code></a>
      {/if}
    </section>
  {/if}
{/if}
