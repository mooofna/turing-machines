// Every route, in the light and the dark theme, on a desktop and a phone (see the projects in
// playwright.config.ts): no console errors, exactly one h1, no serious or critical axe violation
// (WCAG 2.1 AA), no horizontal scrolling and no request to another origin. Then the theme toggle
// and the pieces of the shell: hero, chapter list, gallery, about page and skip link.
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

const CHAPTERS = [
  '/machine',
  '/programs-as-data',
  '/limits',
  '/busy-beaver',
  '/proving-bb5',
  '/frontier'
];
const ROUTES = ['/', ...CHAPTERS, '/playground', '/gallery', '/about'];

interface Watch {
  errors: string[];
  foreign: string[];
}

/** Records console errors, uncaught exceptions and requests to other origins. */
function watch(page: Page, baseURL: string): Watch {
  const origin = new URL(baseURL).origin;
  const result: Watch = { errors: [], foreign: [] };
  page.on('pageerror', (e) => result.errors.push(e.message));
  page.on('console', (m) => {
    if (m.type() === 'error') result.errors.push(m.text());
  });
  page.on('request', (r) => {
    const url = new URL(r.url());
    if (url.protocol === 'data:' || url.protocol === 'blob:') return;
    if (url.origin !== origin) result.foreign.push(r.url());
  });
  return result;
}

/** Scrolls through the page so that lazy diagrams start, then waits until nothing computes. */
async function settle(page: Page) {
  await page.waitForLoadState('networkidle');
  await page.evaluate(async () => {
    for (let y = 0; y < document.body.scrollHeight; y += window.innerHeight) {
      window.scrollTo(0, y);
      await new Promise((r) => setTimeout(r, 50));
    }
    window.scrollTo(0, 0);
  });
  await page.waitForFunction(
    () => !document.querySelector('[data-status="computing"], [aria-busy="true"]'),
    undefined,
    { timeout: 30_000 }
  );
}

/** Serious and critical violations of WCAG 2.1 A and AA; canvases are images axe cannot read. */
async function violations(page: Page): Promise<string[]> {
  const result = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .exclude('canvas')
    .analyze();
  return result.violations
    .filter((v) => v.impact === 'serious' || v.impact === 'critical')
    .map(
      (v) =>
        `${v.id}: ${v.nodes
          .map((n) => n.target.join(' '))
          .slice(0, 3)
          .join(' | ')}`
    );
}

async function horizontalOverflow(page: Page): Promise<number> {
  return page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
}

for (const route of ROUTES) {
  for (const colorScheme of ['light', 'dark'] as const) {
    test(`${route} in the ${colorScheme} theme`, async ({ page, baseURL }) => {
      await page.emulateMedia({ colorScheme });
      const seen = watch(page, baseURL!);
      await page.goto(route);
      await expect(page.locator('h1')).toHaveCount(1);
      await settle(page);
      expect(await violations(page)).toEqual([]);
      expect(await horizontalOverflow(page)).toBeLessThanOrEqual(0);
      expect(seen.errors).toEqual([]);
      expect(seen.foreign).toEqual([]);
    });
  }

  test(`${route} switches theme with the toggle`, async ({ page, baseURL }) => {
    await page.emulateMedia({ colorScheme: 'light' });
    const seen = watch(page, baseURL!);
    await page.goto(route);
    await settle(page);
    const html = page.locator('html');
    const toggle = page.getByRole('button', { name: 'Switch to the dark theme' });
    await expect(toggle).toHaveText('Dark');
    await toggle.click();
    await expect(html).toHaveAttribute('data-theme', 'dark');
    await expect(page.getByRole('button', { name: 'Switch to the light theme' })).toHaveText(
      'Light'
    );
    const background = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(await background()).toBe('rgb(14, 14, 14)');
    expect(await violations(page)).toEqual([]);

    // The choice survives a reload, against the system setting.
    await page.reload();
    await expect(html).toHaveAttribute('data-theme', 'dark');
    await settle(page);
    expect(await background()).toBe('rgb(14, 14, 14)');
    await page.getByRole('button', { name: 'Switch to the light theme' }).click();
    await expect(html).toHaveAttribute('data-theme', 'light');
    expect(await background()).toBe('rgb(255, 255, 255)');
    expect(seen.errors).toEqual([]);
  });
}

test('the landing page measures and draws the BB(5) champion', async ({ page }) => {
  await page.goto('/');
  const hero = page.locator('.hero');
  await expect(hero.locator('.spacetime[data-status="done"]')).toHaveCount(1, { timeout: 60_000 });
  const size = await hero
    .locator('canvas')
    .evaluate((c: HTMLCanvasElement) => ({ w: c.width, h: c.height }));
  expect(size.w).toBeGreaterThan(0);
  expect(size.h).toBeGreaterThan(0);
  const facts = page.locator('.facts dd');
  await expect(facts).toHaveCount(3);
  for (const dd of await facts.all()) {
    await expect(dd).not.toHaveText(/computing/);
    await expect(dd).toHaveText(/\d/);
  }
});

test('the landing page lists the six chapters in order', async ({ page }) => {
  await page.goto('/');
  const links = page.locator('#chapters nav a');
  await expect(links).toHaveCount(CHAPTERS.length);
  const hrefs = await links.evaluateAll((as) =>
    as.map((a) => new URL((a as HTMLAnchorElement).href).pathname)
  );
  expect(hrefs).toEqual(CHAPTERS);
  await page.getByRole('link', { name: 'Start with chapter 1' }).click();
  await expect(page).toHaveURL(/\/machine$/);
  await expect(
    page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Chapters' })
  ).toHaveAttribute('aria-current', 'true');
});

test('the gallery links every machine to the playground', async ({ page }) => {
  await page.goto('/gallery');
  const rows = page.locator('table.machines tbody tr');
  expect(await rows.count()).toBeGreaterThan(0);
  for (const row of await rows.all()) {
    const id = (await row.getAttribute('id'))!.replace(/^m-/, '');
    await expect(row.getByRole('link', { name: /Open in playground/ })).toHaveAttribute(
      'href',
      new RegExp(`/playground#g=${id}$`)
    );
  }
  await settle(page);
  await expect(page.locator('.thumb .spacetime[data-status="error"]')).toHaveCount(0);
});

test('the about page carries the origin, the licence and the citation', async ({ page }) => {
  await page.goto('/about');
  await expect(page.getByRole('heading', { name: 'Origin' })).toBeVisible();
  await expect(page.getByText(/Markus\s+Krötzsch/).first()).toBeVisible();
  await expect(page.locator('pre.bib')).toContainText('@software{heide2026turing');
  await expect(page.getByRole('button', { name: 'Copy BibTeX' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'MIT licence' })).toHaveAttribute(
    'href',
    /LICENSE-MIT$/
  );
});

test('the skip link moves focus to the content', async ({ page }) => {
  await page.goto('/about');
  await page.keyboard.press('Tab');
  const skip = page.getByRole('link', { name: 'Skip to content' });
  await expect(skip).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('main')).toBeFocused();
});
