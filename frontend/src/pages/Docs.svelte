<script lang="ts">
  /**
   * The documentation listing.
   *
   * Documents are grouped by chapter, in the reading order the backend
   * supplies. Titles and summaries are carried in both languages.
   */
  import type { DocumentIndex } from '../lib/types';
  import { navigate } from '../lib/router';
  import { language, pick, pickList, tr } from '../lib/i18n';

  interface Props {
    index: DocumentIndex | null;
  }

  let { index }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);
</script>

<div class="page-header">
  <h1>{tr('docs.title', lang)}</h1>
  <p>{tr('docs.intro', lang)}</p>
</div>

{#if !index}
  <p class="empty">{tr('docs.loading', lang)}</p>
{:else}
  {#each index.chapters as chapter}
    <section style="margin-bottom: 2rem;">
      <h2 style="margin-top: 1.5rem;">{pick(chapter.title, lang)}</h2>
      {#if pick(chapter.summary, lang)}
        <p style="color: var(--text-muted); margin-top: 0">{pick(chapter.summary, lang)}</p>
      {/if}
      <div class="grid cards">
        {#each chapter.documents as document}
          <article class="card">
            <h3><a href={`#/docs/${document.id}`}>{pick(document.title, lang)}</a></h3>
            <p class="card-clamp" style="color: var(--text-muted); margin: 0.4rem 0 0.6rem">
              {pick(document.summary, lang)}
            </p>
            <div class="card-tags">
              {#each pickList(document.tags, lang) as tag}
                <span class="tag">{tag}</span>
              {/each}
            </div>
            <button
              class="ghost card-action"
              type="button"
              onclick={() => navigate(`/docs/${document.id}`)}>{tr('docs.read', lang)}</button>
          </article>
        {/each}
      </div>
    </section>
  {/each}
{/if}
