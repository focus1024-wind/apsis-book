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
  import ResultView from '../lib/ResultView.svelte';

  interface Props {
    registry: ToolRegistry | null;
    id: string | null;
  }

  let { registry, id }: Props = $props();

  let tool = $state<Tool | null>(null);
  let values = $state<Record<string, string>>({});
  let result = $state<ToolResult | null>(null);
  let error = $state<{ parameter: string | null; message: string } | null>(null);
  let running = $state(false);

  /** Selects a tool and fills the form with its defaults. */
  function select(next: Tool | null) {
    tool = next;
    result = null;
    error = null;
    values = {};
    if (!next) return;
    for (const param of next.params) {
      values[param.name] = param.default ?? '';
    }
  }

  $effect(() => {
    if (!registry) return;
    if (!id) {
      select(null);
      return;
    }
    const found = registry.tools.find((candidate) => candidate.id === id) ?? null;
    select(found);
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
        error = { parameter: null, message: 'the tool did not return a result' };
      }
    } catch (caught) {
      error = {
        parameter: null,
        message:
          caught instanceof ApiError
            ? caught.message
            : 'the request could not be completed',
      };
    } finally {
      running = false;
    }
  }

  /** Resets the form to the declared defaults. */
  function reset() {
    if (!tool) return;
    values = {};
    for (const param of tool.params) {
      values[param.name] = param.default ?? '';
    }
    result = null;
    error = null;
  }

  /** The tools grouped by category. */
  const categories = $derived.by(() => {
    if (!registry) return [];
    return registry.categories.map((category) => ({
      category,
      tools: registry.tools.filter((candidate) => candidate.category === category),
    }));
  });
</script>

<div class="page-header">
  <h1>Online tools</h1>
  <p>
    Run the ApsisRS kernels on your own input. Every tool calls the same library the
    documentation describes, and every result reports the units of each quantity.
  </p>
</div>

{#if !registry}
  <p class="empty">Loading the tool registry…</p>
{:else}
  {#each categories as group}
    <section style="margin-bottom: 1.25rem;">
      <h3 style="margin-bottom: 0.5rem;">{group.category}</h3>
      <div style="display: flex; flex-wrap: wrap; gap: 0.4rem;">
        {#each group.tools as candidate}
          <a
            href={`#/tools/${candidate.id}`}
            class="badge"
            style={id === candidate.id
              ? 'border-color: var(--accent); color: var(--accent); padding: 0.3rem 0.7rem;'
              : 'padding: 0.3rem 0.7rem;'}
          >
            {candidate.name}
          </a>
        {/each}
      </div>
    </section>
  {/each}

  {#if !tool}
    <div class="notice muted">
      <p>Select a tool above to open its form.</p>
    </div>
  {:else}
    <section class="card" style="margin-top: 1.5rem;">
      <header style="display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap;">
        <h2 style="margin: 0; border: none;">{tool.name}</h2>
        <span class="badge">{tool.category}</span>
      </header>
      <p style="color: var(--text-muted);">{tool.description}</p>

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
                {param.label}{#if param.required}<span style="color: var(--negative);"> *</span>{/if}
              </label>
              {#if param.kind === 'select'}
                <select id={`field-${param.name}`} bind:value={values[param.name]}>
                  {#each param.constraints.options ?? [] as option}
                    <option value={option.value}>{option.label}</option>
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
              <span class="help">{param.help}</span>
              {#if param.constraints.unit}
                <span class="unit">unit: {param.constraints.unit}</span>
              {/if}
            </div>
          {/each}
        </div>

        <div style="display: flex; gap: 0.6rem; margin-top: 1rem;">
          <button type="submit" disabled={running}>
            {#if running}<span class="spinner"></span>{/if}
            {running ? 'Running…' : 'Run'}
          </button>
          <button class="ghost" type="button" onclick={reset}>Reset</button>
        </div>
      </form>

      {#if tool.api_items.length > 0}
        <p style="color: var(--text-faint); font-size: 0.82rem; margin-top: 0.75rem;">
          Calls:
          {#each tool.api_items as name}
            <a href={`#/api/${name}`} style="font-family: var(--mono);">{name}</a>
          {/each}
        </p>
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
