import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [
    sveltekit({
      adapter: adapter({ fallback: '404.html' }),
      // GitHub Pages serves the site below /<repository>; CI sets BASE_PATH accordingly.
      paths: { base: (process.env.BASE_PATH ?? '') as '' | `/${string}` },
      // Links into the playground carry the machine in the hash (#g=, #tm=, #src=), which is
      // read in the browser, so prerendering must not look for elements with those ids.
      prerender: { handleHttpError: 'warn', handleMissingId: 'ignore' },
      csp: {
        mode: 'hash',
        directives: {
          'default-src': ['self'],
          'script-src': ['self', 'wasm-unsafe-eval'],
          'worker-src': ['self', 'blob:'],
          'connect-src': ['self'],
          'img-src': ['self', 'data:', 'blob:'],
          'style-src': ['self', 'unsafe-inline'],
          'font-src': ['self'],
          'object-src': ['none'],
          'base-uri': ['self'],
          'form-action': ['self']
        }
      }
    })
  ],
  server: { fs: { allow: ['..'] } },
  worker: { format: 'es' },
  test: { include: ['src/**/*.test.ts'], environment: 'node' }
});
