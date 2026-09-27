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
            {scalar.value}{#if scalar.unit}<span class="unit">{scalar.unit}</span>{/if}
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
                {pick(column.label, lang)}{#if column.unit}<span class="unit"> [{column.unit}]</span>{/if}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each result.table.rows as row}
            <tr>
              {#each row as cell, index}
                <td class={result.table.columns[index]?.numeric ? 'numeric' : ''}>
                  {formatCell(cell, result.table.columns[index]?.numeric ?? false)}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if result.notes.length > 0}
    <div class="notice muted">
      {#each result.notes as note}
        <p>{pick(note, lang)}</p>
      {/each}
    </div>
  {/if}
</section>
