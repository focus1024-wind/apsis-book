<script lang="ts">
  /**
   * A hand written SVG line chart.
   *
   * Each series is drawn on a shared pair of axes. The chart is a pure
   * function of its props: given the same data it produces the same picture,
   * which keeps it testable and free of hidden animation state.
   */
  import type { ResultSeries } from '../lib/types';

  interface Props {
    series: ResultSeries[];
    axis?: { name: string; unit: string; values: number[] } | null;
    height?: number;
  }

  let { series, axis = null, height = 220 }: Props = $props();

  const WIDTH = 720;
  const PADDING = { left: 68, right: 18, top: 16, bottom: 34 };

  /** A series with the colour it is drawn in. */
  const palette = ['var(--accent)', 'var(--positive)', 'var(--warning)', 'var(--negative)'];

  interface Prepared {
    name: string;
    unit: string;
    color: string;
    points: { x: number; y: number }[];
  }

  /** Builds the drawn geometry. */
  const prepared = $derived.by((): Prepared[] => {
    const usable = series.filter((item) => item.values.length > 1);
    if (usable.length === 0) return [];

    const plotWidth = WIDTH - PADDING.left - PADDING.right;
    const plotHeight = height - PADDING.top - PADDING.bottom;

    // The horizontal axis is the sample index, or the supplied axis, which
    // has the same length as every series.
    const xValues =
      axis && axis.values.length > 0
        ? axis.values
        : usable[0].values.map((_, index) => index);
    let xMin = Math.min(...xValues);
    let xMax = Math.max(...xValues);

    let yMin = Number.POSITIVE_INFINITY;
    let yMax = Number.NEGATIVE_INFINITY;
    for (const item of usable) {
      for (const value of item.values) {
        if (!Number.isFinite(value)) continue;
        yMin = Math.min(yMin, value);
        yMax = Math.max(yMax, value);
      }
    }
    if (!Number.isFinite(yMin) || !Number.isFinite(yMax)) return [];
    if (yMin === yMax) {
      yMin -= 1;
      yMax += 1;
    }
    if (xMin === xMax) {
      xMin -= 1;
      xMax += 1;
    }
    // A small vertical margin so that a flat series is not drawn on the
    // frame itself.
    const yPad = (yMax - yMin) * 0.06;
    yMin -= yPad;
    yMax += yPad;

    return usable.map((item, index) => ({
      name: item.name,
      unit: item.unit,
      color: palette[index % palette.length],
      points: item.values.map((value, position) => {
        const xValue = xValues[position] ?? position;
        return {
          x: PADDING.left + ((xValue - xMin) / (xMax - xMin)) * plotWidth,
          y: PADDING.top + plotHeight - ((value - yMin) / (yMax - yMin)) * plotHeight,
        };
      }),
    }));
  });

  /** A polyline path for one series. */
  function path(points: { x: number; y: number }[]): string {
    return points.map((point, index) => `${index === 0 ? 'M' : 'L'} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`).join(' ');
  }

  /** The vertical grid lines and labels. */
  const yTicks = $derived.by((): { y: number; label: string }[] => {
    const plotHeight = height - PADDING.top - PADDING.bottom;
    const values: number[] = [];
    for (const item of series) {
      for (const value of item.values) if (Number.isFinite(value)) values.push(value);
    }
    if (values.length === 0) return [];
    let min = Math.min(...values);
    let max = Math.max(...values);
    if (min === max) {
      min -= 1;
      max += 1;
    }
    const pad = (max - min) * 0.06;
    min -= pad;
    max += pad;
    const ticks: { y: number; label: string }[] = [];
    for (let index = 0; index <= 4; index += 1) {
      const fraction = index / 4;
      const value = min + (max - min) * fraction;
      ticks.push({
        y: PADDING.top + plotHeight - fraction * plotHeight,
        label: Math.abs(value) >= 1e4 || (Math.abs(value) < 1e-3 && value !== 0)
          ? value.toExponential(2)
          : value.toFixed(Math.abs(value) >= 100 ? 1 : 4),
      });
    }
    return ticks;
  });
</script>

{#if prepared.length === 0}
  <p class="empty">No series to plot.</p>
{:else}
  <svg class="chart" viewBox={`0 0 ${WIDTH} ${height}`} preserveAspectRatio="none" role="img">
    {#each yTicks as tick}
      <line
        x1={PADDING.left}
        y1={tick.y}
        x2={WIDTH - PADDING.right}
        y2={tick.y}
        stroke="var(--border)"
        stroke-width="1"
        stroke-dasharray="3 4"
      />
      <text
        x={PADDING.left - 8}
        y={tick.y + 4}
        text-anchor="end"
        fill="var(--text-faint)"
        font-size="10"
        font-family="var(--mono)">{tick.label}</text>
    {/each}

    <line
      x1={PADDING.left}
      y1={PADDING.top}
      x2={PADDING.left}
      y2={height - PADDING.bottom}
      stroke="var(--border-strong)"
    />
    <line
      x1={PADDING.left}
      y1={height - PADDING.bottom}
      x2={WIDTH - PADDING.right}
      y2={height - PADDING.bottom}
      stroke="var(--border-strong)"
    />

    {#each prepared as item}
      <path d={path(item.points)} fill="none" stroke={item.color} stroke-width="1.8" />
    {/each}

    {#if axis}
      <text
        x={WIDTH - PADDING.right}
        y={height - 8}
        text-anchor="end"
        fill="var(--text-faint)"
        font-size="10"
        font-family="var(--mono)">{axis.name} [{axis.unit}]</text>
    {/if}
  </svg>

  <div class="legend">
    {#each prepared as item}
      <span><i style={`background:${item.color}`}></i>{item.name} [{item.unit}]</span>
    {/each}
  </div>
{/if}
