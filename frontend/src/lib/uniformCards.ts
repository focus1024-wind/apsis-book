/**
 * Gives every card in a container the height of the tallest one.
 *
 * A grid makes the cards of one row equal, but each row is measured on its
 * own, so a page built from several grids steps from one row height to the
 * next and the column of cards looks ragged. Measuring the tallest card
 * across every grid and applying it to all of them removes the step.
 *
 * The measurement runs when the content or the language changes, and on a
 * resize, because the number of columns and therefore the natural height of
 * a card both depend on the width. Nothing is measured on the server: the
 * height is a property of the rendering, not of the content.
 */

/**
 * The cards the measurement covers.
 *
 * The interface page groups its items under a kind and lays them out in its
 * own grid, so both grids take part; a card in either is made the height of
 * the tallest.
 */
const CARD_SELECTOR = '.grid.cards > .card, .item-grid > .api-item';

/**
 * Returns the height of the tallest card under `root`, in pixels, or `0` when
 * there is no card.
 */
export function tallestCardHeight(root: ParentNode): number {
  let tallest = 0;
  // The card is laid out at its natural height here, because the uniform
  // height is not applied yet.
  for (const card of root.querySelectorAll<HTMLElement>(CARD_SELECTOR)) {
    tallest = Math.max(tallest, card.getBoundingClientRect().height);
  }
  return Math.ceil(tallest);
}

/**
 * Applies the tallest card height to every card under `root`, and removes the
 * inline height when there is no card.
 */
export function applyUniformHeight(root: ParentNode): void {
  const cards = [...root.querySelectorAll<HTMLElement>(CARD_SELECTOR)];
  if (cards.length === 0) {
    return;
  }
  const tallest = tallestCardHeight(root);
  for (const card of cards) {
    card.dataset.uniformHeight = '1';
    card.style.height = `${tallest}px`;
  }
}

/**
 * Measures the cards and keeps them equal, returning a stop function.
 *
 * The returned function detaches the listener.
 */
export function uniformCards(node: HTMLElement | null): () => void {
  if (!node) return () => {};
  let frame = 0;
  const update = () => {
    frame = 0;
    applyUniformHeight(node);
  };
  const schedule = () => {
    if (frame !== 0) return;
    frame = requestAnimationFrame(update);
  };
  schedule();
  const observer = new ResizeObserver(schedule);
  observer.observe(node);
  window.addEventListener('resize', schedule);
  return () => {
    if (frame !== 0) cancelAnimationFrame(frame);
    observer.disconnect();
    window.removeEventListener('resize', schedule);
  };
}
