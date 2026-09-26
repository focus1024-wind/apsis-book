<script lang="ts">
  /**
   * The documentation listing.
   *
   * Documents are grouped by chapter, in the reading order the backend
   * supplies.
   */
  import type { DocumentIndex } from '../lib/types';
  import { navigate } from '../lib/router';

  interface Props {
    index: DocumentIndex | null;
  }

  let { index }: Props = $props();
</script>

<div class="page-header">
  <h1>Concepts</h1>
  <p>
    A systematic description of the concepts the ApsisRS kernels implement, together with
    the methods they do not implement and how the alternatives compare.
  </p>
</div>

{#if !index}
  <p class="empty">Loading the documentation listing…</p>
{:else}
  {#each index.chapters as chapter}
    <section style="margin-bottom: 2rem;">
      <h2 style="margin-top: 1.5rem;">{chapter.title}</h2>
      {#if chapter.summary}
        <p style="color: var(--text-muted); margin-top: 0;">{chapter.summary}</p>
      {/if}
      <div class="grid cards">
        {#each chapter.documents as document}
          <article class="card">
            <h3><a href={`#/docs/${document.id}`}>{document.title}</a></h3>
            <p style="color: var(--text-muted); margin: 0.4rem 0 0.6rem;">{document.summary}</p>
            <div>
              {#each document.tags as tag}
                <span class="tag">{tag}</span>
              {/each}
            </div>
            <button
              class="ghost"
              type="button"
              style="margin-top: 0.75rem;"
              onclick={() => navigate(`/docs/${document.id}`)}>Read</button>
          </article>
        {/each}
      </div>
    </section>
  {/each}
{/if}
