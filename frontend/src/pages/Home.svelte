<script lang="ts">
  /**
   * The home page.
   *
   * It states what the service is and links to the three sections, and it
   * reports the size of the embedded content so that a reader can see what
   * is available.
   */
  import type { Health, DocumentIndex } from '../lib/types';
  import { navigate } from '../lib/router';

  interface Props {
    health: Health | null;
    index: DocumentIndex | null;
  }

  let { health, index }: Props = $props();

  /** The number of documents, or a dash when the listing is absent. */
  const documentCount = $derived(index?.document_count ?? 0);
</script>

<div class="page-header">
  <h1>APSIS</h1>
  <p>
    The web service for the ApsisRS computational kernels. It explains the concepts the
    kernels implement, shows the interface of the library as it is written in its source,
    and runs the kernels on parameters you supply.
  </p>
</div>

{#if health}
  <div class="grid metrics">
    <div class="metric">
      <div class="value">{documentCount}</div>
      <div class="label">concept documents</div>
    </div>
    <div class="metric">
      <div class="value">{health.graph_nodes}</div>
      <div class="label">graph nodes</div>
    </div>
    <div class="metric">
      <div class="value">{health.api_crates}</div>
      <div class="label">crates documented</div>
    </div>
    <div class="metric">
      <div class="value">{health.api_items}</div>
      <div class="label">public items</div>
    </div>
    <div class="metric">
      <div class="value">{health.tools}</div>
      <div class="label">online tools</div>
    </div>
    <div class="metric">
      <div class="value">{health.version}</div>
      <div class="label">service version</div>
    </div>
  </div>
{/if}

<div class="grid cards" style="margin-top: 1.5rem;">
  <article class="card">
    <h3>Concepts</h3>
    <p style="color: var(--text-muted)">
      What the kernels compute: the time scales, the reference frames, the element formats,
      and the propagation models, with the units and the applicable range of each.
    </p>
    <button class="secondary" type="button" onclick={() => navigate('/docs')}>Open the documentation</button>
  </article>

  <article class="card">
    <h3>Knowledge graph</h3>
    <p style="color: var(--text-muted)">
      The concepts as a graph, with the relationships between them, and the crate structure
      of the library. Every node opens the document that explains it.
    </p>
    <button class="secondary" type="button" onclick={() => navigate('/graph')}>Open the graph</button>
  </article>

  <article class="card">
    <h3>Interface</h3>
    <p style="color: var(--text-muted)">
      The public interface of the ApsisRS orbit layer, extracted from the source doc
      comments, with the signature, the units, and the error cases of each item.
    </p>
    <button class="secondary" type="button" onclick={() => navigate('/api')}>Open the interface</button>
  </article>

  <article class="card">
    <h3>Online tools</h3>
    <p style="color: var(--text-muted)">
      Run the kernels on your own input: parse a two-line element set, propagate an orbit,
      convert a time scale, convert coordinates, and more.
    </p>
    <button class="secondary" type="button" onclick={() => navigate('/tools')}>Open the tools</button>
  </article>
</div>
