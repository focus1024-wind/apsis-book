<script lang="ts">
  /**
   * The interface documentation.
   *
   * The catalog is extracted from the source of the library, so what is
   * shown here is the interface as it is written rather than a description
   * of it. Selecting a crate lists its items; searching filters the item
   * names across every crate.
   */
  import type { ApiIndex, ApiCrate, ApiItem } from '../lib/types';
  import { getJson } from '../lib/api';
  import { language, tr } from '../lib/i18n';
  import Signature from '../lib/Signature.svelte';

  interface Props {
    index: ApiIndex | null;
    id: string | null;
  }

  let { index, id }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  let selectedCrate = $state<string | null>(null);
  let crate = $state<ApiCrate | null>(null);
  let item = $state<(ApiItem & { crate: string; layer: string }) | null>(null);
  let itemError = $state<string | null>(null);
  let filter = $state('');
  let loading = $state(false);

  /** Selects a route segment: either a crate or an item name. */
  async function resolve(segment: string | null) {
    selectedCrate = null;
    crate = null;
    item = null;
    itemError = null;
    if (!segment || !index) return;

    const knownCrate = index.crates.find((candidate) => candidate.name === segment);
    if (knownCrate) {
      loading = true;
      try {
        crate = await getJson<ApiCrate>(`/api/api/crates/${segment}`);
        selectedCrate = segment;
      } finally {
        loading = false;
      }
      return;
    }

    const knownItem = index.items.find((candidate) => candidate.name === segment);
    if (!knownItem) {
      itemError = `There is no crate or interface item named ${segment}.`;
      return;
    }
    loading = true;
    try {
      const response = await getJson<ApiItem & { crate: string; layer: string }>(
        `/api/api/items/${segment}`,
      );
      item = response;
      selectedCrate = response.crate;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void resolve(id);
  });

  /** The item names that match the filter. */
  const filtered = $derived.by(() => {
    if (!index) return [];
    const needle = filter.trim().toLowerCase();
    if (needle === '') return [];
    return index.items
      .filter(
        (candidate) =>
          candidate.name.toLowerCase().includes(needle) ||
          candidate.crate.toLowerCase().includes(needle),
      )
      .slice(0, 60);
  });

  /** The items of the crate grouped by kind. */
  const groups = $derived.by(() => {
    if (!crate) return [];
    const order = ['struct', 'enum', 'trait', 'fn', 'const', 'type'];
    const result: { kind: string; items: ApiItem[] }[] = [];
    for (const kind of order) {
      const items = crate.items.filter((candidate) => candidate.kind === kind);
      if (items.length > 0) result.push({ kind, items });
    }
    return result;
  });
</script>

<div class="page-header">
  <h1>Interface documentation</h1>
  <p>
    The public interface of the ApsisRS layers this service builds against, extracted from
    the source doc comments. Signatures are shown as they are written, with the units and
    the error cases stated in the documentation of each item.
  </p>
</div>

{#if !index}
  <p class="empty">Loading the interface catalog…</p>
{:else}
  <div class="grid cards">
    {#each index.crates as candidate}
      <article class="card" style={selectedCrate === candidate.name ? 'border-color: var(--accent-strong);' : ''}>
        <header style="display: flex; align-items: baseline; gap: 0.6rem; flex-wrap: wrap;">
          <h3 style="margin: 0;">
            <a href={`#/api/${candidate.name}`}>{candidate.name}</a>
          </h3>
          <span class="badge">{candidate.layer}</span>
          <span class="badge">{candidate.items} items</span>
        </header>
        <p style="color: var(--text-muted); font-size: 0.88rem;">{candidate.summary}</p>
      </article>
    {/each}
  </div>

  <section class="card" style="margin-top: 1.5rem;">
    <h3>Find an item</h3>
    <input
      type="search"
      placeholder="Filter the {index.item_count} public items by name or crate"
      bind:value={filter}
    />
    {#if filter.trim() !== ''}
      {#if filtered.length === 0}
        <p class="empty">No item matches that filter.</p>
      {:else}
        <div style="display: flex; flex-wrap: wrap; gap: 0.4rem; margin-top: 0.75rem;">
          {#each filtered as candidate}
            <a class="badge" href={`#/api/${candidate.name}`} style="font-size: 0.78rem;">
              {candidate.name}
              <span style="color: var(--text-faint);"> · {candidate.crate}</span>
            </a>
          {/each}
        </div>
      {/if}
    {/if}
  </section>

  {#if loading}
    <p class="empty"><span class="spinner"></span> Loading…</p>
  {/if}

  {#if itemError}
    <div class="notice error"><p>{itemError}</p></div>
  {/if}

  {#if item}
    <section class="card" style="margin-top: 1.5rem;">
      <p class="breadcrumb">
        <a href="#/api">{tr('nav.api', lang)}</a> /
        <a href={`#/api/${item.crate}`}>{item.crate}</a> / {item.name}
      </p>
      <header style="display: flex; align-items: baseline; gap: 0.6rem; flex-wrap: wrap;">
        <span class="kind">{item.kind}</span>
        <span class="name" style="font-family: var(--mono); color: var(--accent); font-size: 1.05rem;">
          {item.name}
        </span>
      </header>
      <div style="margin: 0.75rem 0;">
        <Signature signature={item.signature} name={item.name} />
      </div>
      {#if item.doc}
        <p class="doc" style="white-space: pre-wrap; color: var(--text-muted);">{item.doc}</p>
      {:else}
        <p class="empty">This item has no documentation comment.</p>
      {/if}
    </section>
  {/if}

  {#if crate}
    <section style="margin-top: 1.5rem;">
      <p class="breadcrumb"><a href="#/api">{tr('nav.api', lang)}</a> / {crate.name}</p>
      <h2 style="margin-top: 0.5rem;">{crate.name}</h2>
      <p style="color: var(--text-muted);">{crate.summary}</p>

      {#each groups as group}
        <h3>{group.kind === 'fn' ? 'functions' : `${group.kind}s`}</h3>
        {#each group.items as candidate}
          <article class="api-item">
            <header>
              <span class="name">{candidate.name}</span>
              {#if candidate.associated}
                <span class="badge">associated</span>
              {/if}
            </header>
            <div style="margin: 0.5rem 0;">
              <Signature signature={candidate.signature} name={candidate.name} />
            </div>
            {#if candidate.doc}
              <p class="doc">{candidate.doc}</p>
            {/if}
          </article>
        {/each}
      {/each}
    </section>
  {/if}
{/if}
