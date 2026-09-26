import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// The build output is embedded in the Rust binary by build.rs, so the
// configuration is deliberately plain. The entry point and the asset names
// are fixed rather than hashed so that the embedded table is stable between
// builds of unchanged sources, which keeps incremental Rust builds cheap.
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
        entryFileNames: 'assets/app.js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name][extname]',
      },
    },
  },
});
