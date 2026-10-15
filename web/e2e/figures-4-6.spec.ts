// The figures of chapters 4 to 6, run in the browser against the production build: the
// champions, the enumeration, the towers and Antihydra produce the known values, the pages stay
// free of console errors and of serious axe violations in both themes after every figure ran.
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

/** Records console errors and uncaught exceptions. */
function watch(page: Page): string[] {
  const errors: string[] = [];
  page.on('pageerror', (e) => errors.push(e.message));
  page.on('console', (m) => {
    if (m.type() === 'error') errors.push(m.text());
  });
  return errors;
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

/** Checks the option `value` of the segmented control of a figure. */
async function choose(page: Page, figure: string, value: string) {
  await page.locator(`${figure} input[type="radio"][value="${value}"]`).check();
}

test('Fig. 4.1 runs the five-state champion to its halt', async ({ page }) => {
  const errors = watch(page);
  await page.goto('/busy-beaver');
  const figure = page.locator('#fig-4-1');
  await figure.scrollIntoViewIfNeeded();
  await choose(page, '#fig-4-1', 's5');
  const readout = figure.locator('.readout');
  await expect(readout).toContainText('1RB1LC_1RC1RB_1RD0LE_1LA1LD_1RZ0LA');
  await expect(readout).toContainText('47,176,870', { timeout: 30_000 });
  await expect(readout).toContainText('4,098');
  await expect(figure.locator('.plate')).toHaveAttribute('data-status', 'done', {
    timeout: 30_000
  });
  await expect(figure.locator('.reading')).toContainText('step 47,176,870');
  // The smaller champions are drawn one row per step.
  await choose(page, '#fig-4-1', 's4');
  await expect(readout).toContainText('107');
  await expect(readout).toContainText('13');
  await expect(figure.locator('canvas')).toHaveJSProperty('height', 108);
  expect(errors).toEqual([]);
});

test('Fig. 4.2 lists the milestones with their sources', async ({ page }) => {
  await page.goto('/busy-beaver');
  const rows = page.locator('#fig-4-2 tbody tr');
  await expect(rows).toHaveCount(8);
  await expect(rows.first()).toContainText('1962');
  await expect(page.locator('#fig-4-2')).toContainText('47,176,870');
  for (const link of await page.locator('#fig-4-2 a.source').all()) {
    await expect(link).toHaveAttribute('href', /\/about#source-/);
  }
});

test('Fig. 5.1 re-derives S(3) and S(4) with no holdouts', async ({ page }) => {
  test.setTimeout(180_000);
  const errors = watch(page);
  await page.goto('/proving-bb5');
  const figure = page.locator('#fig-5-1');
  await figure.scrollIntoViewIfNeeded();
  const start = figure.getByRole('button', { name: 'Start' });
  const cell = (row: string | RegExp, column: number) =>
    figure
      .locator('tbody tr', { has: page.getByRole('rowheader', { name: row }) })
      .locator('td')
      .nth(column);

  await choose(page, '#fig-5-1', '3');
  await start.click();
  await expect(figure.getByRole('status')).toContainText('Finished', { timeout: 30_000 });
  await expect(cell('Nodes in tree normal form', 0)).toHaveText('5,417');
  await expect(cell('Nodes in tree normal form', 3)).toHaveText('agrees');
  await expect(cell('Holdouts', 0)).toHaveText('0');
  await expect(cell(/the most steps/, 0)).toHaveText('21');
  await expect(cell(/the most steps/, 3)).toHaveText('agrees');
  await expect(cell(/the most ones/, 0)).toHaveText('6');

  await choose(page, '#fig-5-1', '4');
  await expect(start).toBeEnabled();
  await start.click();
  await expect(figure.getByRole('status')).toContainText('Finished', { timeout: 120_000 });
  await expect(cell('Nodes in tree normal form', 0)).toHaveText('858,909');
  await expect(cell('Halting', 0)).toHaveText('249,693');
  await expect(cell('Never halt, all deciders', 0)).toHaveText('609,216');
  await expect(cell('Holdouts', 0)).toHaveText('0');
  await expect(cell(/the most steps/, 0)).toHaveText('107');
  await expect(cell(/the most ones/, 0)).toHaveText('13');
  // The records are the classic champions, also when the enumeration ran in shards.
  await expect(figure.locator('.record').first()).toContainText('1RB1LB_1LA0LC_1RZ1LD_1RD0RA');
  await expect(figure.locator('.record').last()).toContainText('1RB1LB_1LA0LC_1RZ1LD_1RD0RA');
  await expect(figure.locator('td.agree', { hasText: 'agrees' })).toHaveCount(4);
  await expect(figure.locator('td.agree', { hasText: 'differs' })).toHaveCount(0);
  expect(errors).toEqual([]);
});

test('Fig. 5.2 shows one decided machine per decider', async ({ page }) => {
  const errors = watch(page);
  await page.goto('/proving-bb5');
  const figure = page.locator('#fig-5-2');
  await figure.scrollIntoViewIfNeeded();
  const readout = figure.locator('.readout');
  const expected = {
    cycler: 'Cyclers and translated cyclers',
    translated: 'Cyclers and translated cyclers',
    ngram: 'n-gram closed position sets',
    repwl: 'Repeated word lists'
  };
  for (const [key, decider] of Object.entries(expected)) {
    await choose(page, '#fig-5-2', key);
    await expect(readout).toContainText('Never halts', { timeout: 30_000 });
    await expect(readout).toContainText(decider);
    await expect(figure.locator('.plate')).toHaveAttribute('data-status', 'done', {
      timeout: 30_000
    });
  }
  await choose(page, '#fig-5-2', 'cycler');
  await expect(readout).toContainText('repeats every 14 steps');
  expect(errors).toEqual([]);
});

test('Fig. 6.1 computes the towers and fills the text', async ({ page }) => {
  const errors = watch(page);
  await page.goto('/frontier');
  const figure = page.locator('#fig-6-1');
  await figure.scrollIntoViewIfNeeded();
  await expect(figure.locator('.count')).toContainText('19,729 decimal digits');
  const digits = await figure
    .locator('.digits .group')
    .evaluateAll((groups) => groups.map((g) => g.textContent).join(''));
  expect(digits).toHaveLength(19_729);
  expect(digits.startsWith('2003529930')).toBe(true);
  expect(digits.endsWith('6')).toBe(true);
  await expect(page.locator('[data-live="tower-4"]')).toHaveText('65,536');
  await expect(page.locator('[data-live="tower-5-digits"]')).toHaveText('19,729');
  await choose(page, '#fig-6-1', '4');
  await expect(figure.locator('.value')).toHaveText('65,536');
  await choose(page, '#fig-6-1', '6');
  await expect(figure.locator('.count')).toContainText('6.031');
  expect(errors).toEqual([]);
});

test('Fig. 6.2 checks and iterates Antihydra on its own and continues the check', async ({
  page
}) => {
  test.setTimeout(240_000);
  const errors = watch(page);
  await page.goto('/frontier');
  const figure = page.locator('#fig-6-2');
  const checked = page.locator('[data-live="antihydra-checked"]');
  // Nothing runs before the figure is near the screen.
  await expect(checked.first()).toHaveAttribute('data-state', 'pending');
  await figure.scrollIntoViewIfNeeded();
  const status = figure.getByRole('status');
  const rows = figure.locator('table.rows tbody tr');

  // Without any action: the walk and a first part of the check, published into the text.
  await expect(status.last()).toContainText(
    'did not reach the halting condition within 100,000 iterations',
    { timeout: 30_000 }
  );
  await expect(status.first()).toContainText('rule applications agree with the rule', {
    timeout: 30_000
  });
  await expect(page.locator('[data-live="antihydra-iterations"]').first()).toHaveText('100,000');
  await expect(page.locator('[data-live="antihydra-balance"]')).toHaveText(/^\d{1,3}(,\d{3})*$/);
  await figure.getByRole('button', { name: /Show all \d+ rows/ }).click();
  const first = await rows.count();
  expect(first).toBeGreaterThan(20);
  expect(first).toBeLessThan(41);
  await expect(checked.first()).toHaveText(String(first - 1));
  await expect(figure.locator('td.agree', { hasText: 'differs' })).toHaveCount(0);

  // The primary action continues the check to the end of the exact range.
  const more = figure.getByRole('button', {
    name: 'Continue the check to 2 to the power of 53 steps'
  });
  await expect(more).toHaveAccessibleDescription(/again from the blank tape/);
  await more.click();
  await expect(status.first()).toContainText('All 40 rule applications agree with the rule', {
    timeout: 180_000
  });
  await expect(more).toHaveCount(0);
  await expect(rows).toHaveCount(41);
  await expect(rows.first()).toContainText('start');
  await expect(figure.locator('td.agree', { hasText: 'agrees' })).toHaveCount(41);
  await expect(figure.locator('td.agree', { hasText: 'differs' })).toHaveCount(0);
  await expect(checked.first()).toHaveText('40');
  expect(errors).toEqual([]);
});

for (const colorScheme of ['light', 'dark'] as const) {
  test(`chapters 4 to 6 pass axe in the ${colorScheme} theme after the figures ran`, async ({
    page
  }) => {
    test.setTimeout(240_000);
    await page.emulateMedia({ colorScheme });
    const errors = watch(page);

    await page.goto('/busy-beaver');
    await page.locator('#fig-4-1').scrollIntoViewIfNeeded();
    await expect(page.locator('#fig-4-1 .plate')).toHaveAttribute('data-status', 'done', {
      timeout: 30_000
    });
    expect(await violations(page)).toEqual([]);

    await page.goto('/proving-bb5');
    await page.locator('#fig-5-1').scrollIntoViewIfNeeded();
    await page.locator('#fig-5-1').getByRole('button', { name: 'Start' }).click();
    await expect(page.locator('#fig-5-1').getByRole('status')).toContainText('Finished', {
      timeout: 30_000
    });
    await page.locator('#fig-5-2').scrollIntoViewIfNeeded();
    await expect(page.locator('#fig-5-2 .plate')).toHaveAttribute('data-status', 'done', {
      timeout: 30_000
    });
    expect(await violations(page)).toEqual([]);

    await page.goto('/frontier');
    await page.locator('#fig-6-1').scrollIntoViewIfNeeded();
    await expect(page.locator('#fig-6-1 .count')).toContainText('decimal digits');
    await page.locator('#fig-6-2').scrollIntoViewIfNeeded();
    await expect(page.locator('#fig-6-2').getByRole('status').last()).toContainText(
      'did not reach the halting condition',
      { timeout: 60_000 }
    );
    await expect(page.locator('#fig-6-2').getByRole('button', { name: /Continue/ })).toBeVisible({
      timeout: 30_000
    });
    expect(await violations(page)).toEqual([]);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)
    ).toBeLessThanOrEqual(0);
    expect(errors).toEqual([]);
  });
}
