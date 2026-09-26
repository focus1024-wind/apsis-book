import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

// TypeScript in the script blocks is handled by the Vite plugin's
// preprocessor, so no additional dependency is needed.
export default {
  preprocess: vitePreprocess(),
};
