<script lang="ts">
  /**
   * The online tools.
   *
   * The form is generated from the tool description the backend supplies,
   * so a tool that is added to the registry appears here without a change to
   * this component. Running a tool posts the form values as JSON and renders
   * the shaped result.
   */
  import type { Tool, ToolRegistry, ToolResult, ToolRunResponse } from '../lib/types';
  import { postJson, ApiError } from '../lib/api';
  import { language, pick, tr } from '../lib/i18n';
  import ResultView from '../lib/ResultView.svelte';

  interface Props {
    registry: ToolRegistry | null;
    id: string | null;
  }

  let { registry, id }: Props = $props();

  /** The current language, read so that the text re-renders on a switch. */
  let lang = $derived($language);

  let tool = $state<Tool | null>(null);
  let values = $state<Record<string, string>>({});
  let result = $state<ToolResult | null>(null);
  let error = $state<{ parameter: string | null; message: string } | null>(null);
  let running = $state(false);

  /**
   * Returns the form values a tool declares as its defaults.
   *
   * Building a fresh object each time means the form state is replaced
   * rather than mutated inside an effect, which is what keeps the effect
   * from writing the state it also reads.
   */
  function defaultsFor(next: Tool | null): Record<string, string> {
    const fresh: Record<string, string> = {};
    if (!next) return fresh;
    for (const param of next.params) {
      fresh[param.name] = param.default ?? '';
    }
    return fresh;
  }

  // The effect depends only on the registry and the route, and it assigns
  // the three pieces of form state as a unit. It must not read `values`,
  // because reading state that the same effect writes makes the effect
  // schedule itself again and the update depth is exceeded.
  $effect(() => {
    const currentRegistry = registry;
    const currentId = id;
    if (!currentRegistry) return;
    const found = currentId
      ? (currentRegistry.tools.find((candidate) => candidate.id === currentId) ?? null)
      : null;
    tool = found;
    result = null;
    error = null;
    values = defaultsFor(found);
  });

  /** Runs the selected tool. */
  async function run() {
    if (!tool) return;
    running = true;
    error = null;
    result = null;
    try {
      const body: Record<string, string> = {};
      for (const param of tool.params) {
        const value = values[param.name] ?? '';
        if (value !== '' || param.required) {
          body[param.name] = value;
        }
      }
      const response = await postJson<ToolRunResponse>(`/api/tools/${tool.id}/run`, body);
      if (response.ok && response.result) {
        result = response.result;
      } else if (response.error) {
        error = response.error;
      } else {
        error = { parameter: null, message: tr('tools.noResult', lang) };
      }
    } catch (caught) {
      error = {
        parameter: null,
        message:
          caught instanceof ApiError ? caught.message : tr('tools.requestFailed', lang),
      };
    } finally {
      running = false;
    }
  }

  /** Resets the form to the declared defaults. */
  function reset() {
    values = defaultsFor(tool);
    result = null;
    error = null;
  }

  /** The tools grouped by category, in the order the registry lists them. */
  const categories = $derived.by(() => {
    if (!registry) return [];
    // A category is matched by its English form, which the backend uses as
    // the stable key of the group. Comparing the objects by identity would
    // never match, because the listing carries a copy per tool.
    return registry.categories.map((category) => ({
      category,
      tools: registry.tools.filter(
        (candidate) => candidate.category.en === category.en,
      ),
    }));
  });
</script>

<div class="page-header">
  <h1>{tr('tools.title', lang)}</h1>
  <p>{tr('tools.intro', lang)}</p>
</div>

{#if !registry}
  <p class="empty">{tr('tools.loading', lang)}</p>
{:else}
  {#each categories as group}
    <section style="margin-bottom: 1.25rem;">
      <h3 style="margin-bottom: 0.5rem;">{pick(group.category, lang)}</h3>
      <div style="display: flex; flex-wrap: wrap; gap: 0.4rem;">
        {#each group.tools as candidate}
          <a
            href={`#/tools/${candidate.id}`}
            class="badge"
            style={id === candidate.id
              ? 'border-color: var(--accent); color: var(--accent); padding: 0.3rem 0.7rem;'
              : 'padding: 0.3rem 0.7rem;'}
          >
            {pick(candidate.name, lang)}
          </a>
        {/each}
      </div>
    </section>
  {/each}

  {#if !tool}
    <div class="notice muted">
      <p>{tr('tools.select', lang)}</p>
    </div>
  {:else}
    <section class="card" style="margin-top: 1.5rem;">
      <header style="display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap;">
        <h2 style="margin: 0; border: none;">{pick(tool.name, lang)}</h2>
        <span class="badge">{pick(tool.category, lang)}</span>
      </header>
      <p style="color: var(--text-muted);">{pick(tool.description, lang)}</p>

      <form
        onsubmit={(event) => {
          event.preventDefault();
          void run();
        }}
      >
        <div class="form-grid">
          {#each tool.params as param}
            {@const invalid = error?.parameter === param.name}
            <div
              class="field {param.kind === 'multiline' ? 'wide' : ''} {invalid ? 'invalid' : ''}"
            >
              <label for={`field-${param.name}`}>
                {pick(param.label, lang)}{#if param.required}<span style="color: var(--negative);"
                    > *</span>{/if}
              </label>
              {#if param.kind === 'select'}
                <select id={`field-${param.name}`} bind:value={values[param.name]}>
                  {#each param.constraints.options ?? [] as option}
                    <option value={option.value}>{pick(option.label, lang)}</option>
                  {/each}
                </select>
              {:else if param.kind === 'multiline'}
                <textarea id={`field-${param.name}`} bind:value={values[param.name]} rows="4"></textarea>
              {:else if param.kind === 'number' || param.kind === 'integer'}
                <input
                  id={`field-${param.name}`}
                  type="number"
                  step={param.kind === 'integer' ? '1' : 'any'}
                  min={param.constraints.min ?? undefined}
                  max={param.constraints.max ?? undefined}
                  bind:value={values[param.name]}
                />
              {:else}
                <input id={`field-${param.name}`} type="text" bind:value={values[param.name]} />
              {/if}
              <span class="help">{pick(param.help, lang)}</span>
              {#if param.constraints.unit}
                <span class="unit">{tr('tools.unit', lang)}: {param.constraints.unit}</span>
              {/if}
            </div>
          {/each}
        </div>

        <div style="display: flex; gap: 0.6rem; margin-top: 1rem;">
          <button type="submit" disabled={running}>
            {#if running}<span class="spinner"></span>{/if}
            {running ? tr('tools.running', lang) : tr('tools.run', lang)}
          </button>
          <button class="ghost" type="button" onclick={reset}>{tr('tools.reset', lang)}</button>
        </div>
      </form>

      {#if tool.api_items.length > 0}
        <div class="calls">
          <span class="calls-label">{tr('tools.calls', lang)}</span>
          {#each tool.api_items as name}
            <a class="chip" href={`#/api/${name}`}>{name}</a>
          {/each}
        </div>
      {/if}
    </section>

    {#if error}
      <div class="notice error" style="margin-top: 1rem;">
        <p>
          {#if error.parameter}
            <strong>{error.parameter}:</strong>
          {/if}
          {error.message}
        </p>
      </div>
    {/if}

    {#if result}
      <div style="margin-top: 1rem;">
        <ResultView {result} />
      </div>
    {/if}
  {/if}
{/if}
