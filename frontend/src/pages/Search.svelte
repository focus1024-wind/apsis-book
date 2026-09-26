<script lang="ts">
  /**
   * The search results page.
   *
   * The query is read from the route fragment. Results link to the document
   * that matched.
   */
  import type { SearchResponse } from '../lib/types';

  interface Props {
    response: SearchResponse | null;
    query: string;
    loading: boolean;
  }

  let { response, query, loading }: Props = $props();
</script>

<div class="page-header">
  <h1>Search</h1>
  <p>
    {#if query}
      Results for <strong>{query}</strong>{#if response} — {response.count} of {response.indexed} documents{/if}.
    {:else}
      Enter a term in the search box above.
    {/if}
  </p>
</div>

{#if loading}
  <p class="empty"><span class="spinner"></span> Searching…</p>
{:else if !query}
  <p class="empty">The index covers the titles and bodies of every concept document.</p>
{:else if !response || response.hits.length === 0}
  <p class="empty">No document contains that term.</p>
{:else}
  <div class="grid cards">
    {#each response.hits as hit}
      <article class="card">
        <h3><a href={`#/docs/${hit.id}`}>{hit.title}</a></h3>
        <p style="color: var(--text-muted); margin: 0.4rem 0;">{hit.summary}</p>
        <span class="badge">{hit.chapter}</span>
        <span class="badge">score {hit.score.toFixed(2)}</span>
      </article>
    {/each}
  </div>
{/if}
