<script lang="ts">
  /**
   * One concept document.
   *
   * The body is HTML rendered by the backend, which escapes every text node
   * and emits only the markup it generated, so rendering it here does not
   * introduce an injection path. Internal links are intercepted and handled
   * by the router.
   *
   * The document carries a form per embedded language. The form matching
   * the reader's language is shown, with the English form as the fallback
   * when a translation is absent.
   */
  import type { Document, DocumentForm, DocumentHeading } from '../lib/types';
  import { navigate } from '../lib/router';
  import { language, pickList, setLanguage, tr, type Language } from '../lib/i18n';

  interface Props {
    document: Document | null;
    id: string | null;
  }

  let { document, id }: Props = $props();

  /** The current language, read so that the form re-renders on a switch. */
  let lang = $derived($language);

  /** The form for the reader's language, falling back to English. */
  const form = $derived.by((): DocumentForm | null => {
    if (!document) return null;
    const preferred = document.forms.find((entry) => entry.language === lang);
    if (preferred) return preferred;
    return document.forms.find((entry) => entry.language === 'en') ?? document.forms[0] ?? null;
  });

  /** True when the reader's language has no form of its own. */
  const translationMissing = $derived(
    !!document && !!form && form.language !== lang,
  );

  /** Which top level sections of the table of contents are expanded. */
  let openSections = $state<Record<string, boolean>>({});

  /**
   * Returns `true` when a heading has a child under it.
   *
   * A fold control is only shown for a heading that actually has children,
   * so a flat document does not carry controls that do nothing.
   */
  function hasChildren(heading: DocumentHeading, index: number): boolean {
    const next = form?.headings[index + 1];
    return next !== undefined && next.level > heading.level;
  }

  /**
   * Returns `true` when a heading is visible in the table of contents.
   *
   * A heading is visible when it is a top level entry, when its section is
   * expanded, or when it is the first child of a section so that a section
   * never looks empty while it is collapsed.
   */
  function isVisible(heading: DocumentHeading, index: number): boolean {
    if (heading.level <= 2) return true;
    const parent = form?.headings
      .slice(0, index)
      .reverse()
      .find((candidate) => candidate.level <= 2);
    if (!parent) return true;
    const open = openSections[parent.anchor];
    if (open !== false) return true;
    // Keep the first child visible so a collapsed section still shows where
    // it starts.
    const firstChild = form?.headings.findIndex((candidate) => candidate.level > 2);
    return firstChild === index;
  }

  /** Flips a section open or closed. */
  function toggleSection(anchor: string) {
    openSections = { ...openSections, [anchor]: openSections[anchor] === false };
  }

  /** True when a form in the other language is available to switch to. */
  const otherLanguage = $derived.by((): Language | null => {
    if (!document || !form) return null;
    const other: Language = form.language === 'zh' ? 'en' : 'zh';
    return document.forms.some((entry) => entry.language === other) ? other : null;
  });

  /**
   * Scrolls to a section of the document.
   *
   * The application routes on the fragment, so a plain `#anchor` link would
   * be read as a route and would leave the page. The fragment is left alone
   * and the element is scrolled to directly instead.
   */
  function scrollToSection(anchor: string) {
    const element = window.document.getElementById(anchor);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  /** Handles a click inside the body. */
  function onClick(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    const anchor = target?.closest('a');
    if (!anchor) return;
    const concept = anchor.getAttribute('data-concept');
    if (concept) {
      event.preventDefault();
      navigate(`/docs/${concept}`);
      return;
    }
    const href = anchor.getAttribute('href');
    if (href && href.startsWith('#') && !href.startsWith('#/')) {
      event.preventDefault();
      scrollToSection(href.slice(1));
    }
  }

  /** Sets the language; the router keeps the reader on the same document. */
  function switchLanguage(next: Language) {
    setLanguage(next);
  }
</script>

{#if !id}
  <p class="empty">{tr('docs.noSelection', lang)}</p>
{:else if !document || !form}
  <div class="notice error">
    <p>{tr('docs.notFound', lang)} <code>{id}</code>。</p>
    <p><a href="#/docs">{tr('docs.back', lang)}</a></p>
  </div>
{:else}
  <div class="doc-layout" class:no-toc={form.headings.length === 0}>
    <article class="doc-body">
      <p class="breadcrumb">
        <a href="#/docs">{tr('docs.breadcrumb', lang)}</a> / {document.chapter}
      </p>
      <h1>{form.title}</h1>
      {#if form.summary}
        <p style="color: var(--text-muted); font-size: 1.05rem;">{form.summary}</p>
      {/if}
      <div>
        {#each pickList(document.tags, lang) as tag}
          <span class="tag">{tag}</span>
        {/each}
        {#if otherLanguage}
          <button
            class="ghost"
            type="button"
            style="padding: 0.05rem 0.6rem; font-size: 0.75rem;"
            onclick={() => switchLanguage(otherLanguage)}>{tr('docs.otherLanguage', lang)}</button>
        {/if}
      </div>

      {#if translationMissing}
        <div class="notice muted">
          <p>{tr('docs.translationMissing', lang)}</p>
        </div>
      {/if}

      {#if document.prerequisites.length > 0}
        <div class="notice">
          <strong>{tr('docs.readFirst', lang)}</strong>
          {#each document.prerequisites as prerequisite, index}
            {#if index > 0}, {/if}
            <a href={`#/docs/${prerequisite}`}>{prerequisite}</a>
          {/each}
        </div>
      {/if}

      <!-- The HTML is generated by the backend from authored Markdown and
           every text node is escaped there. The handler only intercepts
           clicks on the generated links. -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div class="doc-content" role="document" onclick={onClick}>
        {@html form.html}
      </div>
    </article>

    {#if form.headings.length > 0}
      <aside class="doc-toc">
        <h2 style="font-size: 0.72rem;">{tr('docs.onThisPage', lang)}</h2>
        <ul class="toc-list">
          {#each form.headings as heading, index (heading.anchor)}
            {#if isVisible(heading, index)}
              <li class="toc-item level-{heading.level}">
                {#if hasChildren(heading, index)}
                  <button
                    class="toc-toggle"
                    class:collapsed={openSections[heading.anchor] === false}
                    aria-expanded={openSections[heading.anchor] !== false}
                    aria-label="{tr('docs.toggle', lang)}: {heading.text}"
                    onclick={() => toggleSection(heading.anchor)}>▾</button>
                {/if}
                <a
                  class="toc-link"
                  href="#{heading.anchor}"
                  onclick={(event) => {
                    event.preventDefault();
                    scrollToSection(heading.anchor);
                  }}>{heading.text}</a>
              </li>
            {/if}
          {/each}
        </ul>
      </aside>
    {/if}
  </div>
{/if}
