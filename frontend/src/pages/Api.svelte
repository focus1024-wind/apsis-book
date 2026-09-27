<script lang="ts">
  /**
   * The interface documentation.
   *
   * The page has two views. The index lists the crates with a short
   * description and lets a reader filter the public items by name. A crate
   * view lists the items of one crate, grouped by kind, with the signature
   * and the documentation of each. An item view shows one item on its own so
   * that a link from a tool or a graph node lands on a readable page.
   *
   * The catalog is extracted from the source of the library, so what is
   * shown here is the interface as it is written rather than a description
   * of it.
   */
  import type { ApiIndex, ApiCrate, ApiItem } from '../lib/types';
  import { getJson } from '../lib/api';
  import { navigate } from '../lib/router';
  import { language, pick, tr } from '../lib/i18n';
  import Signature from '../lib/Signature.svelte';

  interface Props {
    index: ApiIndex | null;
    id: string | null;
  }

  let { index, id }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  let crate = $state<ApiCrate | null>(null);
  let item = $state<(ApiItem & { crate: string; layer: string }) | null>(null);
  let notFound = $state<string | null>(null);
  let filter = $state('');
  let loading = $state(false);

  /** The kind of a member, for the heading of a group. */
  const KIND_ORDER = ['struct', 'enum', 'trait', 'fn', 'const', 'type'];

  /** Resolves a route segment: either a crate name or an item name. */
  async function resolve(segment: string | null) {
    crate = null;
    item = null;
    notFound = null;
    if (!segment || !index) return;

    if (index.crates.some((candidate) => candidate.name === segment)) {
      loading = true;
      try {
        crate = await getJson<ApiCrate>(`/api/api/crates/${segment}`);
      } finally {
        loading = false;
      }
      return;
    }

    if (!index.items.some((candidate) => candidate.name === segment)) {
      notFound = segment;
      return;
    }
    loading = true;
    try {
      item = await getJson<ApiItem & { crate: string; layer: string }>(
        `/api/api/items/${segment}`,
      );
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
      .slice(0, 80);
  });

  /** The items of the crate grouped by kind. */
  const groups = $derived.by(() => {
    const current = crate;
    if (!current) return [];
    return KIND_ORDER.map((kind) => ({
      kind,
      items: current.items.filter((candidate) => candidate.kind === kind),
    })).filter((group) => group.items.length > 0);
  });

  /** Opens an item route. */
  function openItem(name: string) {
    navigate(`/api/${name}`);
  }

  /**
   * Handles a click inside a rendered doc comment.
   *
   * A doc comment that links to another item is rendered as an internal
   * link, which the router handles; a link to a web address is left to the
   * browser.
   */
  function onDocClick(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    const anchor = target?.closest('a');
    if (!anchor) return;
    const item = anchor.getAttribute('data-api');
    if (item) {
      event.preventDefault();
      navigate(`/api/${item}`);
    }
  }

  /**
   * Returns the first sentence of a crate summary.
   *
   * A crate summary is the module documentation, which is several
   * paragraphs. The index shows one sentence per crate so that the grid
   * stays readable; the full text is on the crate page.
   */
  function firstSentence(summary: string): string {
    const text = summary.replace(/\s+/g, ' ').trim();
    if (text === '') return '';
    const end = text.search(/[.!?](\s|$)/);
    if (end === -1) return text.length > 160 ? `${text.slice(0, 157)}...` : text;
    return text.slice(0, end + 1);
  }
</script>

<div class="page-header">
  <h1>{tr('api.title', lang)}</h1>
  <p>{tr('api.intro', lang)}</p>
</div>

{#if !index}
  <p class="empty">{tr('api.loading', lang)}</p>
{:else if crate}
  <!-- One crate, with its items grouped by kind. -->
  <p class="breadcrumb">
    <a href="#/api">{tr('nav.api', lang)}</a> / {crate.name}
    <span class="badge" style="margin-left: 0.5rem">{crate.layer}</span>
  </p>
  <h2 style="margin-top: 0.5rem">{crate.name}</h2>
  <!-- The HTML is rendered by the backend from the module documentation,
       and every text node is escaped there. -->
  <div class="doc-body crate-summary">{@html pick(crate.summary.html, lang)}</div>

  {#each groups as group}
    <h3 id={`kind-${group.kind}`}>
      {group.kind === 'fn' ? 'functions' : `${group.kind}s`}
      <span class="badge">{group.items.length}</span>
    </h3>
    {#each group.items as candidate (candidate.name + candidate.signature)}
      <article class="api-item">
        <header>
          <a class="name" href={`#/api/${candidate.name}`}>{candidate.name}</a>
          {#if candidate.associated}
            <span class="badge">{tr('api.associated', lang)}</span>
          {/if}
          <button
            class="ghost"
            type="button"
            style="margin-left: auto; padding: 0.05rem 0.5rem; font-size: 0.75rem;"
            onclick={() => openItem(candidate.name)}>{tr('api.detail', lang)}</button>
        </header>
        <div style="margin: 0.5rem 0">
          <Signature signature={candidate.signature} name={candidate.name} />
        </div>
        {#if candidate.doc.en}
          <!-- The HTML is rendered by the backend from the doc comment, and
               every text node is escaped there. -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div class="doc-body api-doc" role="document" onclick={onDocClick}>
            {@html pick(candidate.doc.html, lang)}
          </div>
        {/if}
      </article>
    {/each}
  {/each}
{:else if item}
  <!-- One item on its own. -->
  <section class="card">
    <p class="breadcrumb">
      <a href="#/api">{tr('nav.api', lang)}</a> /
      <a href={`#/api/${item.crate}`}>{item.crate}</a> / {item.name}
    </p>
    <header style="display: flex; align-items: baseline; gap: 0.6rem; flex-wrap: wrap">
      <span class="kind">{item.kind}</span>
      <span style="font-family: var(--mono); color: var(--accent); font-size: 1.05rem">
        {item.name}
      </span>
    </header>
    <div style="margin: 0.75rem 0">
      <Signature signature={item.signature} name={item.name} />
    </div>
    {#if item.doc.en}
      <!-- The HTML is rendered by the backend from the doc comment, and
           every text node is escaped there. -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div class="doc-body api-doc" role="document" onclick={onDocClick}>
        {@html pick(item.doc.html, lang)}
      </div>
    {:else}
      <p class="empty">{tr('api.noDoc', lang)}</p>
    {/if}
  </section>
{:else}
  <!-- The index. -->
  {#if notFound}
    <div class="notice error">
      <p>{tr('api.noItem', lang)} <code>{notFound}</code>。</p>
    </div>
  {/if}
  {#if loading}
    <p class="empty"><span class="spinner"></span> {tr('api.loadingItem', lang)}</p>
  {/if}

  <section class="card">
    <h3 style="margin-top: 0">{tr('api.find', lang)}</h3>
    <input
      type="search"
      placeholder={`${tr('api.filter', lang)} (${index.item_count})`}
      bind:value={filter}
    />
    {#if filter.trim() !== ''}
      {#if filtered.length === 0}
        <p class="empty">{tr('api.noMatch', lang)}</p>
      {:else}
        <div class="chip-row">
          {#each filtered as candidate}
            <a class="chip" href={`#/api/${candidate.name}`}>
              {candidate.name}
              <span class="chip-note">{candidate.crate}</span>
            </a>
          {/each}
        </div>
      {/if}
    {/if}
  </section>

  <div class="crate-grid">
    {#each index.crates as candidate}
      <a class="crate-card" href={`#/api/${candidate.name}`}>
        <header>
          <span class="name">{candidate.name}</span>
          <span class="badge">{candidate.layer}</span>
          <span class="badge">{candidate.items}</span>
        </header>
        <p>{firstSentence(pick(candidate.summary.plain, lang))}</p>
      </a>
    {/each}
  </div>
{/if}
