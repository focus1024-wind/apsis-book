import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// The build output is embedded in the Rust binary by build.rs, so the
// configuration is deliberately plain.
//
// The asset names carry a content hash. A new build therefore changes the
// name of every file whose contents changed, which is what lets the service
// mark them immutable: a browser that has an old copy cannot be asked for it
// again, because the URL of the new bundle is a different one. The entry
// point keeps a fixed name so that opening the site always fetches the
// document that names the current bundle.
export default defineConfig({
  plugins: [svelte()],
  build: {
    target: 'es2020',
    outDir: 'dist',
    emptyOutDir: true,
    assetsDir: 'assets',
    sourcemap: false,
    cssCodeSplit: false,
    rollupOptions: {
      output: {
        entryFileNames: 'assets/[name]-[hash].js',
        chunkFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]',
      },
    },
  },
});
