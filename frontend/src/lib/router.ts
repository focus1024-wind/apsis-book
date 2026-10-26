/**
 * A hash based router.
 *
 * The service serves one HTML entry point for every non-asset path, so the
 * route lives in the fragment and never reaches the server. A fragment
 * change is handled without a page load, and a reload of any route returns
 * the entry point from the server and re-renders from the fragment.
 *
 * Routes:
 *
 * - `#/` or empty: the home page
 * - `#/docs` and `#/docs/<id>`: the concept documentation
 * - `#/graph` and `#/graph/<id>`: the knowledge graph and the flow graphs
 * - `#/api` and `#/api/<module>`: the interface documentation
 * - `#/tools` and `#/tools/<id>`: the online tools
 * - `#/search?q=...`: the search results
 */

/** The parsed route. */
export interface Route {
  /** The first segment, or an empty string for the home page. */
  name: string;
  /** The second segment, when present. */
  id: string | null;
  /** The query parameters of the fragment. */
  query: URLSearchParams;
}

/** Parses the current fragment into a route. */
export function currentRoute(): Route {
  const raw = window.location.hash.replace(/^#/, '');
  const [path, search] = raw.split('?');
  const segments = path.split('/').filter((segment) => segment.length > 0);
  return {
    name: segments[0] ?? '',
    id: segments[1] ? decodeURIComponent(segments[1]) : null,
    query: new URLSearchParams(search ?? ''),
  };
}

/** Navigates to a route, pushing a history entry. */
export function navigate(path: string): void {
  const target = path.startsWith('#') ? path : `#${path}`;
  if (window.location.hash === target) {
    return;
  }
  window.location.hash = target;
}

/** Replaces the current route without adding a history entry. */
export function replace(path: string): void {
  const target = path.startsWith('#') ? path : `#${path}`;
  const url = `${window.location.pathname}${window.location.search}${target}`;
  window.history.replaceState(null, '', url);
}

/** Subscribes to route changes. */
export function onRouteChange(listener: () => void): () => void {
  window.addEventListener('hashchange', listener);
  return () => window.removeEventListener('hashchange', listener);
}

/**
 * Returns the page to the top when the route changes.
 *
 * A hash change does not move the scroll position, so a reader who has
 * scrolled to the bottom of a long document and follows a link lands in the
 * middle of the next page, with nothing to show that it is a different one.
 *
 * The scroll follows the path rather than the fragment, so that a search or a
 * filter applied to the page a reader is already on leaves them where they
 * are. Only a move to another page starts at the top.
 *
 * Going back also starts at the top rather than where the reader left. The
 * browser would restore that position itself, but it cannot be told apart
 * from an ordinary link: a fragment navigation creates a history entry, and
 * the browser fires `popstate` for it as well as for a traversal, so there is
 * no event that means "this was a traversal" on its own. Restoring the
 * position by watching the path would mean keeping a history of positions,
 * which is more machinery than the gain is worth here.
 */
export function scrollToTopOnNavigation(): () => void {
  let previous: string | null = null;

  const onChange = () => {
    const path = window.location.hash.split('?')[0];
    if (path === previous) return;
    previous = path;
    window.scrollTo(0, 0);
  };

  // The first page is already at the top, but the position is recorded so
  // that moving away and back is treated as two changes rather than one.
  previous = window.location.hash.split('?')[0];

  window.addEventListener('hashchange', onChange);
  return () => window.removeEventListener('hashchange', onChange);
}

/**
 * Intercepts clicks on internal links so that the router handles them.
 *
 * A link with a `data-concept` attribute names a concept, which is turned
 * into a documentation route. Any other internal link is followed as
 * written.
 */
export function handleLinkClick(event: MouseEvent): boolean {
  if (event.defaultPrevented || event.button !== 0) return false;
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return false;

  const target = event.target as HTMLElement | null;
  const anchor = target?.closest('a');
  if (!anchor) return false;

  const concept = anchor.getAttribute('data-concept');
  if (concept) {
    event.preventDefault();
    navigate(`/docs/${concept}`);
    return true;
  }

  const href = anchor.getAttribute('href');
  if (href && href.startsWith('#/')) {
    return false;
  }
  return false;
}
