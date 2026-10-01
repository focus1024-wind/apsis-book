<script lang="ts">
  /**
   * A hand written SVG graph renderer.
   *
   * The layout is a force directed simulation with a deterministic initial
   * placement derived from the node identifier, so the same graph is drawn
   * the same way on every load. The simulation is run to convergence when
   * the data changes rather than animated, which keeps the component a pure
   * function of its props.
   *
   * Interaction: dragging pans, the wheel zooms about the pointer, a click
   * on a node selects it, and a double click opens its document.
   */
  import type { Graph, GraphNode } from '../lib/types';
  import { groupColor, truncate } from '../lib/format';
  import { language, pick, tr } from '../lib/i18n';

  interface Props {
    graph: Graph;
    selected?: string | null;
    onselect?: (node: GraphNode) => void;
    onopen?: (node: GraphNode) => void;
  }

  let { graph, selected = null, onselect, onopen }: Props = $props();

  /** The current language, read so that the labels re-render on a switch. */
  let lang = $derived($language);

  /** Position of a node in layout space. */
  interface Placement {
    x: number;
    y: number;
    vx: number;
    vy: number;
  }

  /** The radius of a node marker. */
  const NODE_RADIUS = 9;
  /**
   * Width and height of the layout space.
   *
   * The canvas takes this aspect, so the drawing fills it rather than being
   * letterboxed inside it.
   */
  const WIDTH = 1400;
  const HEIGHT = 760;

  let placements = $state<Map<string, Placement>>(new Map());
  let span = $state(1);
  let offsetX = $state(0);
  let offsetY = $state(0);
  let hovered = $state<string | null>(null);
  let dragging = $state(false);
  let dragStart = { x: 0, y: 0, offsetX: 0, offsetY: 0 };

  /** A deterministic pseudo random value in `[0, 1)` from a string. */
  function hash(text: string): number {
    let value = 2166136261;
    for (let index = 0; index < text.length; index += 1) {
      value ^= text.charCodeAt(index);
      value = Math.imul(value, 16777619);
    }
    return ((value >>> 0) % 100000) / 100000;
  }

  /** Runs the layout for a graph. */
  function layout(data: Graph): Map<string, Placement> {
    const result = new Map<string, Placement>();
    const count = Math.max(1, data.nodes.length);
    // Initial placement: nodes are spread on a circle in a deterministic
    // order, with a small jitter so that symmetric graphs do not collapse.
    data.nodes.forEach((node, index) => {
      // The seeds are placed on two rings so that a large graph does not
      // start as a single tight circle.
      const ring = index % 3;
      const angle =
        (index / Math.ceil(count / 3)) * Math.PI * 2 + hash(node.id) * 0.8 + ring * 0.7;
      const radius = 220 + ring * 165 + hash(`${node.id}-r`) * 120;
      result.set(node.id, {
        x: WIDTH / 2 + Math.cos(angle) * radius,
        y: HEIGHT / 2 + Math.sin(angle) * radius,
        vx: 0,
        vy: 0,
      });
    });

    const links = data.edges
      .map((edge) => ({ source: result.get(edge.source), target: result.get(edge.target) }))
      .filter((link) => link.source !== undefined && link.target !== undefined);

    // A small force simulation: repulsion between every pair, attraction
    // along every edge, and a weak pull toward the centre.
    // The spacing grows with the node count: a graph of forty nodes laid out
    // for a graph of twenty is legible, and the same spacing is crowded.
    const iterations = 420;
    const idealLength = 130 + Math.sqrt(count) * 16;
    const repulsion = 9000 + count * 260;
    for (let step = 0; step < iterations; step += 1) {
      for (const first of result.values()) {
        for (const second of result.values()) {
          if (first === second) continue;
          const dx = second.x - first.x;
          const dy = second.y - first.y;
          const distanceSquared = Math.max(64, dx * dx + dy * dy);
          const force = repulsion / distanceSquared;
          const distance = Math.sqrt(distanceSquared);
          first.vx -= (dx / distance) * force;
          first.vy -= (dy / distance) * force;
        }
      }
      for (const link of links) {
        const source = link.source as Placement;
        const target = link.target as Placement;
        const dx = target.x - source.x;
        const dy = target.y - source.y;
        const distance = Math.max(1, Math.sqrt(dx * dx + dy * dy));
        const force = (distance - idealLength) * 0.012;
        source.vx += (dx / distance) * force;
        source.vy += (dy / distance) * force;
        target.vx -= (dx / distance) * force;
        target.vy -= (dy / distance) * force;
      }
      for (const placement of result.values()) {
        placement.vx += (WIDTH / 2 - placement.x) * 0.0016;
        placement.vy += (HEIGHT / 2 - placement.y) * 0.0016;
        placement.x += placement.vx;
        placement.y += placement.vy;
        placement.vx *= 0.72;
        placement.vy *= 0.72;
        placement.x = Math.min(WIDTH - 90, Math.max(90, placement.x));
        placement.y = Math.min(HEIGHT - 60, Math.max(60, placement.y));
      }
    }
    return spreadToCanvas(result);
  }

  /**
   * Widens a layout until it has the shape of the canvas.
   *
   * The simulation settles into a roughly round cluster, while the canvas is
   * wide. Left alone the fit then scales by the short axis and leaves a large
   * empty margin on the long one. Stretching along the long axis to match the
   * canvas shape uses the whole area; the distortion is a uniform scale, so
   * relative distances and the layout stay readable.
   */
  function spreadToCanvas(points: Map<string, Placement>): Map<string, Placement> {
    const values = [...points.values()];
    if (values.length < 2) return points;
    let minX = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (const point of values) {
      minX = Math.min(minX, point.x);
      maxX = Math.max(maxX, point.x);
      minY = Math.min(minY, point.y);
      maxY = Math.max(maxY, point.y);
    }
    const width = maxX - minX;
    const height = maxY - minY;
    if (width < 1 || height < 1) return points;
    const wanted = WIDTH / HEIGHT;
    const current = width / height;
    // Only widen a narrow layout; a layout that is already wider than the
    // canvas is left alone, since stretching further would overflow it.
    if (current >= wanted * 0.9) return points;
    const scale = (wanted * height) / width;
    if (scale > 3) return points;
    const centreX = (minX + maxX) / 2;
    for (const point of values) {
      point.x = centreX + (point.x - centreX) * scale;
    }
    return points;
  }

  /**
   * Scales and centres the view so that every node is visible with a margin.
   *
   * The layout occupies a fixed design space, which the force simulation
   * fills only in its centre. Fitting the view to the nodes removes the
   * empty margin the reader would otherwise have to pan across.
   */
  function fitToNodes() {
    if (placements.size === 0) {
      span = 1;
      offsetX = 0;
      offsetY = 0;
      return;
    }
    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let maxX = Number.NEGATIVE_INFINITY;
    let maxY = Number.NEGATIVE_INFINITY;
    for (const placement of placements.values()) {
      minX = Math.min(minX, placement.x);
      maxX = Math.max(maxX, placement.x);
      minY = Math.min(minY, placement.y);
      maxY = Math.max(maxY, placement.y);
    }
    const margin = 70;
    const width = Math.max(1, maxX - minX + margin * 2);
    const height = Math.max(1, maxY - minY + margin * 2);
    const scale = Math.min(WIDTH / width, HEIGHT / height);
    span = Math.max(0.3, Math.min(2.5, scale));
    offsetX = (WIDTH - (maxX - minX) * span) / 2 - minX * span;
    offsetY = (HEIGHT - (maxY - minY) * span) / 2 - minY * span;
  }

  $effect(() => {
    placements = layout(graph);
    fitToNodes();
  });

  /** The transform of the layout viewport. */
  const transform = $derived(`translate(${offsetX} ${offsetY}) scale(${span})`);

  /** Handles a pointer press on the canvas, starting a pan. */
  function onPointerDown(event: PointerEvent) {
    dragging = true;
    dragStart = { x: event.clientX, y: event.clientY, offsetX, offsetY };
    (event.currentTarget as Element).setPointerCapture(event.pointerId);
  }

  /** Handles a pointer move, panning while pressed. */
  function onPointerMove(event: PointerEvent) {
    if (!dragging) return;
    offsetX = dragStart.offsetX + (event.clientX - dragStart.x);
    offsetY = dragStart.offsetY + (event.clientY - dragStart.y);
  }

  /** Ends a pan. */
  function onPointerUp(event: PointerEvent) {
    dragging = false;
    (event.currentTarget as Element).releasePointerCapture(event.pointerId);
  }

  /** Zooms about the pointer. */
  function onWheel(event: WheelEvent) {
    event.preventDefault();
    const factor = event.deltaY < 0 ? 1.12 : 1 / 1.12;
    const next = Math.min(4, Math.max(0.3, span * factor));
    const rect = (event.currentTarget as Element).getBoundingClientRect();
    const pointerX = event.clientX - rect.left;
    const pointerY = event.clientY - rect.top;
    offsetX = pointerX - ((pointerX - offsetX) * next) / span;
    offsetY = pointerY - ((pointerY - offsetY) * next) / span;
    span = next;
  }

  /** Returns the viewport to the fitted view. */
  function reset() {
    fitToNodes();
  }
</script>

<div class="graph-controls">
  <button class="ghost" type="button" onclick={reset}>{tr('graph.fit', lang)}</button>
  <span class="badge">{graph.nodes.length} {tr('graph.nodes', lang)}</span>
  <span class="badge">{graph.edges.length} {tr('graph.edges', lang)}</span>
  <span class="badge">{tr('graph.zoom', lang)} {span.toFixed(2)}×</span>
</div>

<svg
  class="graph-canvas"
  viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
  preserveAspectRatio="xMidYMid meet"
  role="img"
  aria-label={pick(graph.title, lang)}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
>
  <defs>
    <marker
      id="arrow"
      viewBox="0 0 10 10"
      refX="9"
      refY="5"
      markerWidth="6"
      markerHeight="6"
      orient="auto-start-reverse"
    >
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--border-strong)" />
    </marker>
    <marker
      id="arrow-active"
      viewBox="0 0 10 10"
      refX="9"
      refY="5"
      markerWidth="6"
      markerHeight="6"
      orient="auto-start-reverse"
    >
      <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--accent)" />
    </marker>
  </defs>

  <g {transform}>
    {#each graph.edges as edge (edge.source + '->' + edge.target)}
      {@const from = placements.get(edge.source)}
      {@const to = placements.get(edge.target)}
      {#if from && to}
        {@const active =
          selected === edge.source || selected === edge.target || hovered === edge.source || hovered === edge.target}
        <line
          x1={from.x}
          y1={from.y}
          x2={to.x}
          y2={to.y}
          stroke={active ? 'var(--accent-strong)' : 'var(--border-strong)'}
          stroke-width={active ? 2 : 1.2}
          marker-end={active ? 'url(#arrow-active)' : 'url(#arrow)'}
        />
      {/if}
    {/each}

    {#each graph.nodes as item (item.id)}
      {@const placement = placements.get(item.id)}
      {#if placement}
        {@const isSelected = selected === item.id}
        {@const isHovered = hovered === item.id}
        <g
          class="node"
          role="button"
          tabindex="0"
          aria-label={pick(item.label, lang)}
          onmouseenter={() => (hovered = item.id)}
          onmouseleave={() => (hovered = null)}
          onclick={() => onselect?.(item)}
          ondblclick={() => onopen?.(item)}
          onkeydown={(event) => {
            if (event.key === 'Enter') onselect?.(item);
          }}
          style="cursor: pointer;"
        >
          {#if isSelected || isHovered}
            <circle
              cx={placement.x}
              cy={placement.y}
              r={NODE_RADIUS + 8}
              fill={groupColor(item.group)}
              opacity="0.18"
            />
          {/if}
          {#if item.kind === 'decision'}
            <rect
              x={placement.x - NODE_RADIUS}
              y={placement.y - NODE_RADIUS}
              width={NODE_RADIUS * 2}
              height={NODE_RADIUS * 2}
              rx="2"
              fill="var(--surface-raised)"
              stroke={groupColor(item.group)}
              stroke-width="2"
            />
          {:else if item.kind === 'terminal'}
            <rect
              x={placement.x - NODE_RADIUS * 1.2}
              y={placement.y - NODE_RADIUS * 0.8}
              width={NODE_RADIUS * 2.4}
              height={NODE_RADIUS * 1.6}
              rx={NODE_RADIUS * 0.8}
              fill="var(--surface-raised)"
              stroke={groupColor(item.group)}
              stroke-width="2"
            />
          {:else if item.kind === 'data'}
            <path
              d={`M ${placement.x - NODE_RADIUS} ${placement.y} L ${placement.x - NODE_RADIUS * 0.4} ${placement.y - NODE_RADIUS} L ${placement.x + NODE_RADIUS} ${placement.y} L ${placement.x - NODE_RADIUS * 0.4} ${placement.y + NODE_RADIUS} Z`}
              fill="var(--surface-raised)"
              stroke={groupColor(item.group)}
              stroke-width="2"
            />
          {:else}
            <circle
              cx={placement.x}
              cy={placement.y}
              r={NODE_RADIUS}
              fill="var(--surface-raised)"
              stroke={groupColor(item.group)}
              stroke-width="2"
            />
          {/if}
          <text
            x={placement.x}
            y={placement.y + NODE_RADIUS + 15}
            text-anchor="middle"
            fill={isSelected ? 'var(--accent)' : 'var(--text)'}
            font-size="12"
            font-family="var(--mono)"
          >
            {truncate(pick(item.label, lang), 26)}
          </text>
        </g>
      {/if}
    {/each}
  </g>
</svg>
