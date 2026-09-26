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
  import Home from './pages/Home.svelte';
  import Docs from './pages/Docs.svelte';
  import DocumentPage from './pages/Document.svelte';
  import Search from './pages/Search.svelte';
  import GraphPage from './pages/Graph.svelte';
  import FlowsPage from './pages/Flows.svelte';
  import ApiPage from './pages/Api.svelte';
  import ToolsPage from './pages/Tools.svelte';

  let route = $state(currentRoute());

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
    if (route.name === 'docs' && document) return `${document.title} · ApsisRS`;
    if (route.name === 'api') return 'Interface · ApsisRS';
    if (route.name === 'graph') return 'Knowledge graph · ApsisRS';
    if (route.name === 'flows') return 'Algorithm flows · ApsisRS';
    if (route.name === 'tools') return 'Online tools · ApsisRS';
    if (route.name === 'search') return 'Search · ApsisRS';
    return 'ApsisRS web service';
  });

  $effect(() => {
    window.document.title = title;
  });
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<div class="app">
  <aside class="sidebar">
    <div class="sidebar-brand">
      <a href="#/">APSIS</a>
      <p>ApsisRS computational kernels</p>
    </div>

    <nav>
      <div class="sidebar-section">
        <h2>Explore</h2>
        <a class="item" class:active={route.name === ''} href="#/">Overview</a>
        <a class="item" class:active={route.name === 'docs'} href="#/docs">Concepts</a>
        <a class="item" class:active={route.name === 'graph'} href="#/graph">Knowledge graph</a>
        <a class="item" class:active={route.name === 'flows'} href="#/flows">Algorithm flows</a>
        <a class="item" class:active={route.name === 'api'} href="#/api">Interface</a>
        <a class="item" class:active={route.name === 'tools'} href="#/tools">Online tools</a>
      </div>

      {#if index}
        <div class="sidebar-section">
          <h2>Documents</h2>
          {#each index.chapters as chapter}
            <div class="sidebar-chapter">
              <div class="sidebar-chapter-title">{chapter.title}</div>
              {#each chapter.documents as entry}
                <a
                  class="item"
                  class:active={route.name === 'docs' && route.id === entry.id}
                  href={`#/docs/${entry.id}`}>{entry.title}</a>
              {/each}
            </div>
          {/each}
        </div>
      {/if}
    </nav>
  </aside>

  <div class="main">
    <header class="topbar">
      <form class="search-box" onsubmit={submitSearch}>
        <input
          type="search"
          placeholder="Search the documentation"
          value={searchQuery}
          aria-label="Search the documentation"
        />
        <button type="submit">Search</button>
      </form>
      {#if health}
        <span class="badge">{health.documents} docs</span>
        <span class="badge">{health.tools} tools</span>
        <span class="badge">v{health.version}</span>
      {/if}
    </header>

    <main class="content" class:wide={route.name === 'graph' || route.name === 'flows'} class:narrow={route.name === 'docs' && route.id !== null}>
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
          <p>There is no page at <code>{route.name}</code>.</p>
          <p><a href="#/">Go to the overview</a></p>
        </div>
      {/if}
    </main>
  </div>
</div>
