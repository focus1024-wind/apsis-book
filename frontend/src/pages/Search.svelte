<script lang="ts">
  /**
   * The search results page.
   *
   * The query is read from the route fragment. A result links to the
   * document that matched, and carries a short excerpt around the match so
   * that a reader can tell why it was returned.
   */
  import type { SearchResponse } from '../lib/types';
  import { language, pick, tr } from '../lib/i18n';

  interface Props {
    response: SearchResponse | null;
    query: string;
    loading: boolean;
  }

  let { response, query, loading }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  /**
   * Returns a short excerpt of a summary around the query.
   *
   * When the query is not in the summary, the beginning of the summary is
   * returned, because the summary is still the most useful line to show.
   */
  function excerpt(summary: string, needle: string, length = 200): string {
    const text = summary.replace(/\s+/g, ' ').trim();
    if (text.length <= length) return text;
    const at = text.toLowerCase().indexOf(needle.toLowerCase());
    if (at <= 0) return `${text.slice(0, length - 1)}…`;
    const start = Math.max(0, at - Math.floor(length / 3));
    const end = Math.min(text.length, start + length);
    return `${start > 0 ? '…' : ''}${text.slice(start, end)}${end < text.length ? '…' : ''}`;
  }
</script>

<div class="page-header">
  <h1>{tr('search.title', lang)}</h1>
  <p>
    {#if query}
      {tr('search.results', lang)} <strong>{query}</strong>{#if response} — {response.count}
        {tr('search.of', lang)} {response.indexed} {tr('search.documents', lang)}{/if}
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
  <ul class="result-list">
    {#each response.hits as hit}
      <li class="result">
        <a class="result-title" href={`#/docs/${hit.id}`}>{hit.title}</a>
        <p class="result-summary">{excerpt(hit.summary, query)}</p>
        <div class="result-meta">
          <span class="badge">{pick(hit.chapter_title, lang)}</span>
          <span class="badge">{tr('search.score', lang)} {hit.score.toFixed(2)}</span>
        </div>
      </li>
    {/each}
  </ul>
{/if}
