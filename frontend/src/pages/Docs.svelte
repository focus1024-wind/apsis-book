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
  import { uniformCards } from '../lib/uniformCards';

  interface Props {
    index: DocumentIndex | null;
  }

  let { index }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  /**
   * The page root, measured so that every card shares one height. The cards
   * of a page are spread over several grids, so the measurement has to start
   * from the element that contains all of them.
   */
  let page = $state<HTMLElement | null>(null);
  $effect(() => uniformCards(page));
</script>

<div bind:this={page} class="page-root">
<div class="page-header">
  <h1>{tr('docs.title', lang)}</h1>
  <p>{tr('docs.intro', lang)}</p>
</div>

{#if !index}
  <p class="empty">{tr('docs.loading', lang)}</p>
{:else}
  {#each index.groups as group (group.id)}
    <section class="docs-group" style="margin-bottom: 2.5rem;">
      <h2 class="docs-group-title">{pick(group.title, lang)}</h2>
      {#each group.sections as section (section.id)}
        <section style="margin-bottom: 1.75rem;">
          <h3 style="margin-top: 1.25rem;">{pick(section.title, lang)}</h3>
          {#if pick(section.summary, lang)}
            <p style="color: var(--text-muted); margin-top: 0">{pick(section.summary, lang)}</p>
          {/if}
          {#each section.subsections as subject (subject.id)}
            <h4 class="docs-subject">{pick(subject.title, lang)}</h4>
            {#if pick(subject.summary, lang)}
              <p style="color: var(--text-muted); margin-top: 0">{pick(subject.summary, lang)}</p>
            {/if}
            <div class="grid cards">
              {#each subject.documents as document (document.id)}
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
          {/each}
          <div class="grid cards">
            {#each section.documents as document (document.id)}
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
    </section>
  {/each}
{/if}
</div>
