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
  import { language, tr } from '../lib/i18n';

  interface Props {
    health: Health | null;
    index: DocumentIndex | null;
  }

  let { health, index }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  /** The number of documents, or zero when the listing is absent. */
  const documentCount = $derived(index?.document_count ?? 0);

  /** The sections, with the route each one opens. */
  const sections = $derived([
    { title: tr('home.docs', lang), body: tr('home.docs.body', lang), action: tr('home.docs.action', lang), route: '/docs' },
    { title: tr('home.graph', lang), body: tr('home.graph.body', lang), action: tr('home.graph.action', lang), route: '/graph' },
    { title: tr('home.api', lang), body: tr('home.api.body', lang), action: tr('home.api.action', lang), route: '/api' },
    { title: tr('home.tools', lang), body: tr('home.tools.body', lang), action: tr('home.tools.action', lang), route: '/tools' },
  ]);

  /** The metric tiles shown above the sections. */
  const metrics = $derived([
    { value: String(documentCount), label: tr('home.metric.documents', lang) },
    { value: String(health?.graph_nodes ?? 0), label: tr('home.metric.graph', lang) },
    { value: String(health?.api_crates ?? 0), label: tr('home.metric.crates', lang) },
    { value: String(health?.api_items ?? 0), label: tr('home.metric.items', lang) },
    { value: String(health?.tools ?? 0), label: tr('home.metric.tools', lang) },
    { value: health?.version ?? '—', label: tr('home.metric.version', lang) },
  ]);
</script>

<div class="page-header">
  <h1>APSIS</h1>
  <p>{tr('home.intro', lang)}</p>
</div>

{#if health}
  <div class="grid metrics">
    {#each metrics as metric}
      <div class="metric">
        <div class="value">{metric.value}</div>
        <div class="label">{metric.label}</div>
      </div>
    {/each}
  </div>
{/if}

<div class="grid cards" style="margin-top: 1.5rem;">
  {#each sections as section}
    <article class="card">
      <h3>{section.title}</h3>
      <p style="color: var(--text-muted)">{section.body}</p>
      <button class="secondary" type="button" onclick={() => navigate(section.route)}>
        {section.action}
      </button>
    </article>
  {/each}
</div>
