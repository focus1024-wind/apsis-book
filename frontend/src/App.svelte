<script lang="ts">
  /**
   * The application shell.
   *
   * It loads the content that every page shares, tracks the route, and
   * renders the page for the current route. Content is fetched once and
   * reused, because it is embedded in the binary and does not change while
   * the service runs.
   */
  import type {
    Health,
    DocumentIndex,
    Document,
    Graph,
    ApiIndex,
    ToolRegistry,
    SearchResponse,
  } from './lib/types';
  import { getJson, queryString } from './lib/api';
  import { currentRoute, onRouteChange, navigate } from './lib/router';
  import { language, pick, setLanguage, tr, type Language } from './lib/i18n';
  import Home from './pages/Home.svelte';
  import Docs from './pages/Docs.svelte';
  import DocumentPage from './pages/Document.svelte';
  import Search from './pages/Search.svelte';
  import GraphPage from './pages/Graph.svelte';
  import FlowsPage from './pages/Flows.svelte';
  import ApiPage from './pages/Api.svelte';
  import ToolsPage from './pages/Tools.svelte';

  let route = $state(currentRoute());
  /** The current language, read so that the shell re-renders on a switch. */
  let lang = $derived($language);

  let health = $state<Health | null>(null);
  let index = $state<DocumentIndex | null>(null);
  let concepts = $state<Graph | null>(null);
  let api = $state<ApiIndex | null>(null);
  let tools = $state<ToolRegistry | null>(null);

  let document = $state<Document | null>(null);
  let documentId = $state<string | null>(null);
  let search = $state<SearchResponse | null>(null);
  let searchQuery = $state('');
  let searchLoading = $state(false);
  let status = $state<string | null>(null);

  /** The chapters whose documents are hidden in the sidebar. */
  let collapsed = $state<string[]>([]);

  /** Folds a chapter, or unfolds it when it is already folded. */
  function toggleChapter(id: string) {
    collapsed = collapsed.includes(id)
      ? collapsed.filter((entry) => entry !== id)
      : [...collapsed, id];
  }

  /** Loads the content every page shares. */
  async function loadShared() {
    try {
      const [healthResponse, indexResponse, graphResponse, apiResponse, toolsResponse] =
        await Promise.all([
          getJson<Health>('/health'),
          getJson<DocumentIndex>('/api/docs'),
          getJson<Graph>('/api/graph'),
          getJson<ApiIndex>('/api/api'),
          getJson<ToolRegistry>('/api/tools'),
        ]);
      health = healthResponse;
      index = indexResponse;
      concepts = graphResponse;
      api = apiResponse;
      tools = toolsResponse;
    } catch {
      status = 'The service content could not be loaded. Reload the page to try again.';
    }
  }

  /** Loads the document named by the route. */
  async function loadDocument(id: string | null) {
    if (!id) {
      document = null;
      documentId = null;
      return;
    }
    if (id === documentId) return;
    try {
      document = await getJson<Document>(`/api/docs/${encodeURIComponent(id)}`);
      documentId = id;
    } catch {
      document = null;
      documentId = id;
    }
  }

  /** Runs the search named by the route. */
  async function loadSearch(query: string) {
    if (!query) {
      search = null;
      searchQuery = '';
      return;
    }
    searchLoading = true;
    try {
      search = await getJson<SearchResponse>(`/api/docs/search${queryString({ q: query, limit: 40 })}`);
      searchQuery = query;
    } catch {
      search = null;
    } finally {
      searchLoading = false;
    }
  }

  $effect(() => {
    void loadShared();
    const stop = onRouteChange(() => {
      route = currentRoute();
    });
    return stop;
  });

  $effect(() => {
    const current = route;
    void loadDocument(current.name === 'docs' ? current.id : null);
    void loadSearch(current.name === 'search' ? current.query.get('q') ?? '' : '');
  });

  /** Submits the search box. */
  function submitSearch(event: SubmitEvent) {
    event.preventDefault();
    const input = (event.currentTarget as HTMLFormElement).querySelector('input');
    const value = input?.value.trim() ?? '';
    if (value === '') return;
    navigate(`/search?q=${encodeURIComponent(value)}`);
  }

  /** The page title, used for the document title element. */
  const title = $derived.by(() => {
    if (route.name === 'docs' && document) {
      const form = document.forms.find((entry) => entry.language === lang) ?? document.forms[0];
      if (form) return `${form.title} · ApsisRS`;
    }
    if (route.name === 'api') return `${tr('nav.api', lang)} · ApsisRS`;
    if (route.name === 'graph') return `${tr('nav.graph', lang)} · ApsisRS`;
    if (route.name === 'flows') return `${tr('nav.flows', lang)} · ApsisRS`;
    if (route.name === 'tools') return `${tr('nav.tools', lang)} · ApsisRS`;
    if (route.name === 'search') return `${tr('search.title', lang)} · ApsisRS`;
    return 'ApsisRS';
  });

  /** Switches the interface language. */
  function toggleLanguage() {
    const next: Language = lang === 'zh' ? 'en' : 'zh';
    setLanguage(next);
  }

  $effect(() => {
    window.document.title = title;
  });
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<div class="app">
  <div class="sidebar-frame">
  <aside class="sidebar">
    <div class="sidebar-brand">
      <a href="#/">APSIS</a>
      <p>{tr('app.subtitle', lang)}</p>
    </div>

    <nav>
      <div class="sidebar-section">
        <h2>{tr('nav.explore', lang)}</h2>
        <a class="item" class:active={route.name === ''} href="#/">{tr('nav.overview', lang)}</a>
        <a class="item" class:active={route.name === 'docs'} href="#/docs">{tr('nav.docs', lang)}</a>
        <a class="item" class:active={route.name === 'graph'} href="#/graph"
          >{tr('nav.graph', lang)}</a>
        <a class="item" class:active={route.name === 'flows'} href="#/flows"
          >{tr('nav.flows', lang)}</a>
        <a class="item" class:active={route.name === 'api'} href="#/api">{tr('nav.api', lang)}</a>
        <a class="item" class:active={route.name === 'tools'} href="#/tools"
          >{tr('nav.tools', lang)}</a>
      </div>

      {#if index}
        {#each index.groups as group (group.id)}
          <div class="sidebar-section">
            <h2 class="sidebar-group-title">{pick(group.title, lang)}</h2>
            {#each group.sections as section (section.id)}
              <div class="sidebar-chapter" class:collapsed={collapsed.includes(section.id)}>
                <div class="sidebar-chapter-head">
                  <a class="sidebar-chapter-title" href="#/docs" title={pick(section.title, lang)}>
                    {pick(section.title, lang)}
                  </a>
                  <button
                    class="sidebar-chapter-toggle"
                    aria-expanded={!collapsed.includes(section.id)}
                    aria-label="{tr('nav.toggleChapter', lang)}: {pick(section.title, lang)}"
                    onclick={() => toggleChapter(section.id)}
                  >▾</button>
                </div>
                {#if !collapsed.includes(section.id)}
                  {#each section.documents as entry (entry.id)}
                    <a
                      class="item"
                      class:active={route.name === 'docs' && route.id === entry.id}
                      href={`#/docs/${entry.id}`}>{pick(entry.title, lang)}</a>
                  {/each}
                {/if}
              </div>
            {/each}
          </div>
        {/each}
      {/if}
    </nav>
  </aside>
  </div>

  <div class="main">
    <header class="topbar">
      <form class="search-box" onsubmit={submitSearch}>
        <input
          type="search"
          placeholder={tr('search.placeholder', lang)}
          value={searchQuery}
          aria-label={tr('search.placeholder', lang)}
        />
        <button type="submit">{tr('search.button', lang)}</button>
      </form>
      {#if health}
        <span class="badge">{health.documents} {tr('nav.documents', lang)}</span>
        <span class="badge">{health.tools} {tr('tool.plural', lang)}</span>
        <span class="badge">v{health.version}</span>
      {/if}
      <button
        class="ghost"
        type="button"
        title={tr('docs.otherLanguageTitle', lang)}
        onclick={toggleLanguage}
      >{tr('docs.otherLanguage', lang)}</button>
    </header>

    <main class="content" class:doc={route.name === 'docs' && route.id !== null} class:wide={route.name === 'graph' || route.name === 'flows'}>
      {#if status}
        <div class="notice error"><p>{status}</p></div>
      {/if}

      {#if route.name === ''}
        <Home {health} {index} />
      {:else if route.name === 'docs'}
        {#if route.id}
          <DocumentPage {document} id={route.id} />
        {:else}
          <Docs {index} />
        {/if}
      {:else if route.name === 'search'}
        <Search response={search} query={searchQuery} loading={searchLoading} />
      {:else if route.name === 'graph'}
        <GraphPage {concepts} {api} />
      {:else if route.name === 'flows'}
        <FlowsPage id={route.id} />
      {:else if route.name === 'api'}
        <ApiPage index={api} id={route.id} />
      {:else if route.name === 'tools'}
        <ToolsPage registry={tools} id={route.id} />
      {:else}
        <div class="notice error">
          <p>{tr('common.notFound', lang)} <code>{route.name}</code>。</p>
          <p><a href="#/">{tr('common.goHome', lang)}</a></p>
        </div>
      {/if}
    </main>
  </div>
</div>
