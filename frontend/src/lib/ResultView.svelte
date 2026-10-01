<script lang="ts">
  /**
   * Renders a tool result.
   *
   * The backend shapes every result as scalars, a table, and series, so one
   * component renders every tool and adding a tool needs no frontend
   * change.
   */
  import type { ToolResult } from '../lib/types';
  import { formatCell } from '../lib/format';
  import { language, pick } from '../lib/i18n';
  import Chart from './Chart.svelte';

  interface Props {
    result: ToolResult;
  }

  let { result }: Props = $props();

  /** The current language, read so that the title re-renders on a switch. */
  let lang = $derived($language);
</script>

<section class="card">
  <h3>{pick(result.title, lang)}</h3>

  {#if result.scalars.length > 0}
    <div class="scalars">
      {#each result.scalars as scalar}
        <div class="scalar">
          <span class="label">{pick(scalar.label, lang)}</span>
          <span class="value" style={scalar.monospace ? '' : 'font-family: var(--sans)'}>
            {scalar.value}{#if scalar.unit}<span class="unit"
              >{#if scalar.unit_html}{@html scalar.unit_html}{:else}{scalar.unit}{/if}</span
            >{/if}
          </span>
        </div>
      {/each}
    </div>
  {/if}

  {#if result.series.length > 0}
    <Chart series={result.series} axis={result.axis ?? null} />
  {/if}

  {#if result.table.rows.length > 0}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            {#each result.table.columns as column}
              <th class={column.numeric ? 'numeric' : ''}>
                {pick(column.label, lang)}{#if column.unit}<span class="unit"
                  > [{#if column.unit_html}{@html column.unit_html}{:else}{column.unit}{/if}]</span
                >{/if}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each result.table.rows as row}
            <tr>
              {#each row as cell, index}
                <td class={result.table.columns[index]?.numeric ? 'numeric' : ''}>
                  {#if typeof cell === 'object' && cell !== null && cell.html}
                    <!-- A cell written as notation, such as a unit with a
                         power in it. The HTML is rendered by the backend and
                         every text node is escaped there. -->
                    {@html cell.html}
                  {:else}
                    {formatCell(
                      cell,
                      result.table.columns[index]?.numeric ?? false,
                      lang,
                    )}
                  {/if}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if result.notes.length > 0}
    <div class="notice muted doc-body">
      {#each result.notes as note, index (index)}
        <!-- The HTML is rendered by the backend from the note, and every text
             node is escaped there. -->
        {@html pick(result.notes_html?.[index] ?? { zh: note.zh, en: note.en }, lang)}
      {/each}
    </div>
  {/if}
</section>
