// Captures every route in the light and the dark theme, on a desktop and a phone, as full-page
// PNG files in web/screenshots/<route>-<theme>-<device>.png, for the visual review.
//
// Run against a preview server (npm run build && npm run preview; restart the preview after every
// build, it does not serve files that appeared after it started), then:
//   node scripts/screenshots.ts [route ...]
// BASE_URL overrides the address of the server (default http://localhost:4173).

import { mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium, type Page } from '@playwright/test';

const ROUTES = [
  '/',
  '/machine',
  '/programs-as-data',
  '/limits',
  '/busy-beaver',
  '/proving-bb5',
  '/frontier',
  '/playground',
  '/gallery',
  '/about'
];

const DEVICES = {
  desktop: { viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1, isMobile: false },
  phone: { viewport: { width: 375, height: 812 }, deviceScaleFactor: 2, isMobile: true }
} as const;

const THEMES = ['light', 'dark'] as const;

const base = (process.env.BASE_URL ?? 'http://localhost:4173').replace(/\/$/, '');
const out = join(dirname(fileURLToPath(import.meta.url)), '..', 'screenshots');
const routes = process.argv.length > 2 ? process.argv.slice(2) : ROUTES;

const fileName = (route: string) => (route === '/' ? 'home' : route.slice(1).replaceAll('/', '-'));

/** Waits until fonts are loaded and no space-time diagram is still computing. */
async function settle(page: Page) {
  await page.waitForLoadState('networkidle');
  await page.evaluate(() => document.fonts.ready);
  await page
    .waitForFunction(
      () => !document.querySelector('[data-status="computing"], [aria-busy="true"]'),
      {
        timeout: 15_000
      }
    )
    .catch(() => undefined);
  await page.waitForTimeout(300);
}

mkdirSync(out, { recursive: true });
const browser = await chromium.launch();
try {
  for (const [device, options] of Object.entries(DEVICES)) {
    for (const theme of THEMES) {
      const context = await browser.newContext({
        ...options,
        hasTouch: options.isMobile,
        colorScheme: theme,
        reducedMotion: 'reduce'
      });
      const page = await context.newPage();
      for (const route of routes) {
        await page.goto(base + route);
        await settle(page);
        // Scroll through the page so that lazy diagrams start, then return to the top.
        await page.evaluate(async () => {
          for (let y = 0; y < document.body.scrollHeight; y += window.innerHeight) {
            window.scrollTo(0, y);
            await new Promise((r) => setTimeout(r, 60));
          }
          window.scrollTo(0, 0);
        });
        await settle(page);
        const path = join(out, `${fileName(route)}-${theme}-${device}.png`);
        await page.screenshot({ path, fullPage: true });
        console.log(path);
      }
      await context.close();
    }
  }
} finally {
  await browser.close();
}
