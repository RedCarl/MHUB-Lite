import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 编辑器的「原子写入」会在源目录临时建 .xxx.tmpdir/ 再改名；
      // chokidar 跟上去会撞 EBUSY，直接不看这些目录。
      ignored: ['**/src-tauri/**', '**/core/**', '**/.*.tmpdir/**', '**/*.tmp'],
    },
  },
  build: {
    target: 'safari15',
    minify: 'esbuild',
    sourcemap: false,
  },
});
