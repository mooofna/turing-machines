// The interactive figures of chapters 1 to 3: each renders, its primary interaction works and
// gives the number the engine computes, nothing logs an error, and the chapter pages stay free of
// serious accessibility violations in both themes with the figures loaded and used.

import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Locator, type Page } from '@playwright/test';

/** Collects console errors and uncaught exceptions of a page. */
function watchErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(error.message));
  return errors;
}

/** Opens a chapter, scrolls the figure into view and waits until it has computed. */
async function figure(page: Page, route: string, id: string): Promise<Locator> {
  await page.goto(route);
  const plate = page.locator(`[data-figure="${id}"]`);
  await plate.scrollIntoViewIfNeeded();
  await expect(plate).toHaveAttribute('aria-busy', 'false', { timeout: 30_000 });
  return plate;
}

async function violations(page: Page): Promise<string[]> {
  const result = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
    .exclude('canvas')
    .analyze();
  return result.violations
    .filter((v) => v.impact === 'serious' || v.impact === 'critical')
    .map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(' | ')}`);
}

const steps = (n: number) => `${n.toLocaleString('en-US')}`;

test.describe('Fig. 1.1, binary increment', () => {
  test('steps 1101 to 0011, which is twelve, and back', async ({ page }) => {
    const errors = watchErrors(page);
    const fig = await figure(page, '/machine', 'fig1.1');
    const step = fig.getByRole('button', { name: 'Step', exact: true });
    await expect(fig.getByLabel('Binary number, least significant bit first')).toHaveValue('1101');
    await expect(fig.locator('.derivation')).toHaveText('q0 1101');

    for (let i = 0; i < 3; i++) await step.click();
    await expect(fig.locator('.derivation')).toContainText('00 q0 01');
    await expect(fig.locator('.derivation')).toContainText('001 q1 1');
    await expect(fig.locator('.outcome')).toContainText('Halted after 3 steps');
    await expect(fig.locator('.outcome')).toContainText('Output 0011, which is 12.');
    await expect(step).toBeDisabled();
    // The undefined entry δ(q1, 1) is outlined in the table.
    await expect(fig.locator('td.active')).toHaveCount(1);

    await fig.getByRole('button', { name: 'Back', exact: true }).click();
    await expect(fig.locator('.outcome')).toContainText('Next: δ(q0, 0) = (q1, 1, R)');
    await fig.getByRole('button', { name: 'Reset', exact: true }).click();
    await expect(fig.locator('.derivation')).toHaveText('q0 1101');
    expect(errors).toEqual([]);
  });

  test('a carry past the last digit writes a new one', async ({ page }) => {
    const fig = await figure(page, '/machine', 'fig1.1');
    const input = fig.getByLabel('Binary number, least significant bit first');
    await input.fill('11');
    await expect(fig.locator('.derivation')).toHaveText('q0 11');
    for (let i = 0; i < 3; i++)
      await fig.getByRole('button', { name: 'Step', exact: true }).click();
    await expect(fig.locator('.outcome')).toContainText('Output 001, which is 4.');

    await input.fill('12');
    await expect(fig.getByText('Use only the digits 0 and 1.')).toBeVisible();
    await expect(fig.getByRole('button', { name: 'Step', exact: true })).toBeDisabled();
  });
});

test.describe('Fig. 1.2, A_mul', () => {
  test('multiplies 3 by 4 and replays the run', async ({ page }) => {
    const errors = watchErrors(page);
    const fig = await figure(page, '/machine', 'fig1.2');
    await expect(fig.getByText('#xxx#xxxx#')).toBeVisible();
    await fig.getByRole('button', { name: 'Run', exact: true }).click();
    await expect(fig.locator('.summary')).toHaveText(/3 × 4 = 12 in 276 steps of A\s*mul\./);
    await expect(fig.locator('.product')).toContainText('3 × 4 = 12');
    await expect(fig.getByText('xxxxxxxxxxxx', { exact: true })).toBeVisible();
    // The replay ends at the last step.
    await expect(fig.locator('.position')).toHaveText('276 of 276', { timeout: 15_000 });
    await expect(fig.getByRole('button', { name: 'Run', exact: true })).toBeVisible();
    expect(errors).toEqual([]);
  });

  test('shows the final tape at once under reduced motion', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const fig = await figure(page, '/machine', 'fig1.2');
    await fig.getByLabel('First factor, 0 to 12').fill('12');
    await fig.getByLabel('Second factor, 0 to 12').fill('12');
    await fig.getByRole('button', { name: 'Run', exact: true }).click();
    await expect(fig.locator('.position')).toHaveText('23,273 of 23,273');
    await expect(fig.locator('.product')).toContainText('12 × 12 = 144');

    await fig.getByLabel('Second factor, 0 to 12').fill('13');
    await expect(fig.getByText('Choose whole numbers from 0 to 12.')).toBeVisible();
    await expect(fig.getByRole('button', { name: 'Run', exact: true })).toBeDisabled();
  });
});

test.describe('Fig. 1.3, M*2 on one and two tapes', () => {
  test('measures 2(n + 1)² and 2n + 1 steps for n = 0 to 20', async ({ page }) => {
    const errors = watchErrors(page);
    const fig = await figure(page, '/machine', 'fig1.3');
    await fig.getByText('Measured steps as a table').click();
    const rows = fig.locator('details tbody tr');
    await expect(rows).toHaveCount(21);
    for (let n = 0; n <= 20; n++) {
      const cells = rows.nth(n).locator('td');
      await expect(cells.nth(0)).toHaveText(steps(2 * (n + 1) ** 2));
      await expect(cells.nth(1)).toHaveText(steps(2 * n + 1));
    }
    await expect(fig.getByRole('img', { name: /from 2 to 882.*from 1 to 41/ })).toBeVisible();
    expect(errors).toEqual([]);
  });

  test('plays both runs for the selected n', async ({ page }) => {
    const fig = await figure(page, '/machine', 'fig1.3');
    await fig.getByLabel('Input xn').fill('3');
    await expect(fig.locator('.value')).toHaveText('n = 3');
    await fig.getByRole('button', { name: 'Play', exact: true }).click();
    const one = fig.getByRole('group', { name: 'One tape, n = 3' });
    const two = fig.getByRole('group', { name: 'Two tapes, n = 3' });
    await expect(two).toContainText('halted after 7 steps', { timeout: 10_000 });
    await expect(one).toContainText('halted after 32 steps', { timeout: 10_000 });
  });

  test('jumps to the end under reduced motion', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const fig = await figure(page, '/machine', 'fig1.3');
    await fig.getByRole('button', { name: 'Play', exact: true }).click();
    await expect(fig.getByRole('group', { name: 'One tape, n = 6' })).toContainText(
      'halted after 98 steps'
    );
    await expect(fig.getByRole('group', { name: 'Two tapes, n = 6' })).toContainText(
      'halted after 13 steps'
    );
  });
});

test.describe('Fig. 2.1, linked views', () => {
  test('a table edit rewrites the line, and the line rewrites the table', async ({ page }) => {
    const errors = watchErrors(page);
    const fig = await figure(page, '/programs-as-data', 'fig2.1');
    const line = fig.getByLabel('Standard format');
    await expect(line).toHaveValue('1RB1LB_1LA1RZ');
    await expect(fig.getByRole('img', { name: /State diagram with start state A/ })).toBeVisible();

    const cell = fig.getByLabel('Entry for state B reading 1');
    await cell.focus();
    await expect(fig.getByText('δ(B, 1) = (Z, 1, R)', { exact: false })).toBeVisible();
    await expect(fig.locator('.edge.on')).toHaveCount(1);
    await cell.fill('0LA');
    await cell.press('Enter');
    await expect(line).toHaveValue('1RB1LB_1LA0LA');

    await line.fill('1RB1LB_1LA1R');
    await expect(
      fig.getByText('The table and the diagram show the last valid machine.')
    ).toBeVisible();
    await expect(line).toHaveAttribute('aria-invalid', 'true');

    await line.fill('1RB1RZ_1LB0RC_1LC1LA');
    await expect(fig.locator('tbody tr')).toHaveCount(3);
    await expect(fig.getByLabel('Entry for state C reading 1')).toHaveValue('1LA');
    await expect(fig.getByRole('link', { name: 'Open in the playground' })).toHaveAttribute(
      'href',
      /\/playground#tm=1RB1RZ_1LB0RC_1LC1LA$/
    );

    await fig.getByRole('button', { name: 'Reset', exact: true }).click();
    await expect(line).toHaveValue('1RB1LB_1LA1RZ');
    expect(errors).toEqual([]);
  });

  test('the link opens the machine in the playground', async ({ page }) => {
    const fig = await figure(page, '/programs-as-data', 'fig2.1');
    await fig.getByRole('link', { name: 'Open in the playground' }).click();
    await expect(page).toHaveURL(/\/playground#tm=1RB1LB_1LA1RZ$/);
    await expect(page.locator('.status')).toContainText(/Step\s+0/);
  });
});

test.describe('Fig. 2.2, number of machines', () => {
  test('writes out (4(n + 1))^2n for n = 1 to 6', async ({ page }) => {
    const errors = watchErrors(page);
    const fig = await figure(page, '/programs-as-data', 'fig2.2');
    for (let n = 1; n <= 6; n++) {
      await fig.getByRole('radio', { name: String(n), exact: true }).check({ force: true });
      const value = (BigInt(4 * (n + 1)) ** BigInt(2 * n)).toString();
      const other = (BigInt(4 * n + 1) ** BigInt(2 * n)).toLocaleString('en-US');
      await expect(fig.locator('.number')).toHaveText(BigInt(value).toLocaleString('en-US'));
      await expect(fig.locator('.digits')).toHaveText(`A number with ${value.length} digits.`);
      await expect(fig.locator('.other')).toContainText(other);
    }
    // Five states: the number in the chapter text.
    await fig.getByRole('radio', { name: '5', exact: true }).check({ force: true });
    await expect(fig.locator('.number')).toHaveText('63,403,380,965,376');
    expect(errors).toEqual([]);
  });

  test('the arrow keys move between the choices', async ({ page }) => {
    const fig = await figure(page, '/programs-as-data', 'fig2.2');
    await fig.getByRole('radio', { name: '5', exact: true }).focus();
    await page.keyboard.press('ArrowLeft');
    await expect(fig.getByRole('radio', { name: '4', exact: true })).toBeChecked();
    await expect(fig.locator('.number')).toHaveText('25,600,000,000');
  });
});

test.describe('Fig. 3.1, halting table', () => {
  test('cells follow the budget and claim non-halting only with a proof', async ({ page }) => {
    const errors = watchErrors(page);
    const fig = await figure(page, '/limits', 'fig3.1');
    const slider = fig.getByLabel('Step budget');
    const row = (name: string) => fig.locator('tbody tr', { hasText: name });

    // 20 steps.
    await expect(fig.locator('output')).toHaveText('20 steps');
    await expect(row('doubling machine').locator('td').nth(1)).toHaveText('halted at 18');
    await expect(row('doubling machine').locator('td').nth(2)).toHaveText('still running');
    await expect(row('Parity').locator('td').nth(0)).toContainText('never halts');
    await expect(row('Parity').locator('td').nth(1)).toHaveText('halted at 2');
    await expect(fig.locator('.notes')).toContainText(
      'walks off to the right in state odd on blank cells'
    );

    // 10 steps, the smallest budget.
    await slider.fill('0');
    await expect(fig.locator('output')).toHaveText('10 steps');
    await expect(row('doubling machine').locator('td').nth(0)).toHaveText('halted at 8');
    await expect(row('doubling machine').locator('td').nth(1)).toHaveText('still running');

    // 1,000,000 steps, the largest budget: every counter cell has halted, the unbounded counter
    // is still running and is never said to run forever.
    await slider.fill('15');
    await expect(fig.locator('output')).toHaveText('1,000,000 steps');
    const counter = row('halts when the count overflows').locator('td');
    const expected = ['19', '218', '2,217', '22,216', '222,215'];
    for (const [i, t] of expected.entries()) {
      await expect(counter.nth(i)).toHaveText(`halted at ${t}`);
    }
    const unbounded = row('Unbounded counter').locator('td');
    for (let i = 0; i < 5; i++) await expect(unbounded.nth(i)).toHaveText('still running');
    await expect(fig.locator('.summary')).toContainText(
      '12 runs halted, 3 runs were proved never to halt and 5 runs are still running'
    );
    expect(errors).toEqual([]);
  });
});

for (const route of ['/machine', '/programs-as-data', '/limits']) {
  for (const colorScheme of ['light', 'dark'] as const) {
    test(`${route} with its figures in use, ${colorScheme} theme, has no serious violations`, async ({
      page
    }) => {
      await page.emulateMedia({ colorScheme });
      const errors = watchErrors(page);
      await page.goto(route);
      const plates = page.locator('[data-figure]');
      const count = await plates.count();
      expect(count).toBeGreaterThan(0);
      for (let i = 0; i < count; i++) {
        const plate = plates.nth(i);
        await plate.scrollIntoViewIfNeeded();
        await expect(plate).toHaveAttribute('aria-busy', 'false', { timeout: 30_000 });
      }
      // Use the primary control of each figure once.
      if (route === '/machine') {
        await page.locator('[data-figure="fig1.1"]').getByRole('button', { name: 'Step' }).click();
      }
      if (route === '/programs-as-data') {
        await page.getByLabel('Entry for state A reading 0').focus();
      }
      if (route === '/limits') await page.getByRole('slider', { name: 'Step budget' }).fill('9');
      expect(await violations(page)).toEqual([]);
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth)
      ).toBeLessThanOrEqual(0);
      expect(errors).toEqual([]);
    });
  }
}
