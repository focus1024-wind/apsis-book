<script lang="ts">
  /**
   * The search results page.
   *
   * The query is read from the route fragment. Results link to the document
   * that matched.
   */
  import type { SearchResponse } from '../lib/types';
  import { language, tr } from '../lib/i18n';

  interface Props {
    response: SearchResponse | null;
    query: string;
    loading: boolean;
  }

  let { response, query, loading }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);
</script>

<div class="page-header">
  <h1>{tr('search.title', lang)}</h1>
  <p>
    {#if query}
      {tr('search.results', lang)} <strong>{query}</strong>{#if response} — {response.count}
        {tr('search.of', lang)} {response.indexed} {tr('search.documents', lang)}{/if}。
    {:else}
      {tr('search.enter', lang)}
    {/if}
  </p>
</div>

{#if loading}
  <p class="empty"><span class="spinner"></span> {tr('search.searching', lang)}</p>
{:else if !query}
  <p class="empty">{tr('search.hint', lang)}</p>
{:else if !response || response.hits.length === 0}
  <p class="empty">{tr('search.none', lang)}</p>
{:else}
  <div class="grid cards">
    {#each response.hits as hit}
      <article class="card">
        <h3><a href={`#/docs/${hit.id}`}>{hit.title}</a></h3>
        <p style="color: var(--text-muted); margin: 0.4rem 0;">{hit.summary}</p>
        <span class="badge">{hit.chapter}</span>
        <span class="badge">{hit.language}</span>
        <span class="badge">{tr('search.score', lang)} {hit.score.toFixed(2)}</span>
      </article>
    {/each}
  </div>
{/if}
