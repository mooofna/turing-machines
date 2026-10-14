// The playground: running, the three editors, keyboard control, share links, accessibility and
// the phone layout.

import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

/** Collects console errors and uncaught exceptions of a page. */
function watchErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(error.message));
  return errors;
}

async function open(page: Page, hash = '') {
  await page.goto(`playground${hash}`);
  await expect(page.locator('.status')).toContainText(/Step\s+0/);
}

const status = (page: Page) => page.locator('.status');

test('loads an example machine without errors', async ({ page }) => {
  const errors = watchErrors(page);
  await open(page);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Playground');
  await expect(page.getByLabel('Machine name')).toHaveValue('Binary increment');
  await expect(page.getByRole('img', { name: /head at position 0/ })).toBeVisible();
  expect(errors).toEqual([]);
});

test('steps forward and back, then runs to the halt', async ({ page }) => {
  await open(page);
  const step = page.getByRole('button', { name: 'Step', exact: true });
  await step.click();
  await expect(status(page)).toContainText(/Step\s+1/);
  await step.click();
  await expect(status(page)).toContainText(/Step\s+2/);
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  await expect(status(page)).toContainText(/Step\s+1/);
  await page.getByRole('button', { name: 'Reset', exact: true }).click();
  await expect(status(page)).toContainText(/Step\s+0/);

  await page.getByRole('button', { name: 'Run', exact: true }).click();
  await expect(status(page)).toContainText('Halted', { timeout: 10_000 });
  await expect(page.getByRole('button', { name: 'Back', exact: true })).toBeEnabled();
  await expect(page.getByRole('button', { name: 'Step', exact: true })).toBeDisabled();
});

test('runs at full speed', async ({ page }) => {
  await open(page, '#g=bb4');
  await page.getByLabel('Speed').fill('100');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  await expect(status(page)).toContainText('Halted');
  await expect(status(page)).toContainText(/Step\s+107/);
});

test('keyboard shortcuts step, run and reset', async ({ page }) => {
  await open(page);
  await page.locator('main').click({ position: { x: 5, y: 5 } });
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  await expect(status(page)).toContainText(/Step\s+2/);
  await page.keyboard.press('ArrowLeft');
  await expect(status(page)).toContainText(/Step\s+1/);
  await page.keyboard.press('Home');
  await expect(status(page)).toContainText(/Step\s+0/);
  await page.keyboard.press('Space');
  await expect(status(page)).toContainText('Halted', { timeout: 10_000 });
  // Shortcuts stay out of text fields.
  await page.getByLabel('Input', { exact: true }).focus();
  await page.keyboard.press('Home');
  await expect(status(page)).toContainText('Halted');
});

test('opens each view of the machine', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Diagram' }).click();
  await expect(page.getByRole('button', { name: /^State q0, start/ })).toBeVisible();
  await page.getByRole('tab', { name: 'Table' }).click();
  await expect(page.getByRole('grid', { name: 'Transition table' })).toBeVisible();
  await page.getByRole('tab', { name: 'Text' }).click();
  await expect(page.locator('.cm-content')).toContainText('δ(q0, 0) = (q0, 0, R)');
});

test('a table edit rewrites the text', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Table' }).click();
  const cell = page.getByRole('textbox', { name: 'δ(q1, 0)' });
  await cell.click();
  await cell.fill('q2, 1, R');
  await cell.press('Enter');
  await expect(cell).toHaveValue('q2, 1, R');
  await page.getByRole('tab', { name: 'Text' }).click();
  await expect(page.locator('.cm-content')).toContainText('δ(q1, 0) = (q2, 1, R)');
  // Comments survive the rewrite.
  await expect(page.locator('.cm-content')).toContainText('// Add one, carrying to the left.');
});

test('a table edit can create a state, and undo removes it', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Table' }).click();
  const cell = page.getByRole('textbox', { name: 'δ(q2, 0)' });
  await cell.click();
  await cell.fill('q7, 0, L');
  await cell.press('Enter');
  await expect(page.getByText('The state q7 does not exist yet.')).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button', { name: /^State q7/ })).toBeVisible();
  await page.getByRole('button', { name: 'Undo', exact: true }).first().click();
  await expect(page.getByRole('button', { name: /^State q7/ })).toHaveCount(0);
});

test('keyboard navigation in the table', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Table' }).click();
  const first = page.getByRole('textbox', { name: 'δ(q0, 0)' });
  await first.focus();
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('textbox', { name: 'δ(q0, 1)' })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(page.getByRole('textbox', { name: 'δ(q1, 1)' })).toBeFocused();
  // Typing replaces the cell, Escape restores it.
  await page.keyboard.type('nonsense');
  await page.keyboard.press('Escape');
  await expect(page.getByRole('textbox', { name: 'δ(q1, 1)' })).toHaveValue('q1, 0, L');
  // An invalid entry is explained and kept.
  await page.keyboard.type('q1, 0');
  await page.keyboard.press('Enter');
  await expect(page.locator('#table-message')).toContainText('Give the next state');
});

test('a text edit updates the table', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Text' }).click();
  const editor = page.locator('.cm-content');
  await expect(editor).toContainText('δ(q1, ␣) = (q2, 1, L)');
  await editor.click();
  await page.keyboard.press('ControlOrMeta+End');
  await page.keyboard.type('\nδ(q2, 0) = (q2, 0, R)');
  await page.getByRole('tab', { name: 'Table' }).click();
  await expect(page.getByRole('textbox', { name: 'δ(q2, 0)' })).toHaveValue('q2, 0, R');
});

test('text errors keep the last valid machine', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Text' }).click();
  const editor = page.locator('.cm-content');
  await editor.click();
  await page.keyboard.press('ControlOrMeta+End');
  await page.keyboard.type('\nδ(q2, 0) = (q2, 0, X)');
  await expect(page.locator('.problems')).toContainText('Error');
  await page.getByRole('tab', { name: 'Table' }).click();
  await expect(page.getByText('The text has errors')).toBeVisible();
  await page.getByRole('button', { name: 'Discard the text changes' }).click();
  await expect(page.getByText('The text has errors')).toHaveCount(0);
});

test('the diagram adds a state on double-click', async ({ page }) => {
  await open(page);
  await page.getByRole('tab', { name: 'Diagram' }).click();
  const svg = page.getByRole('group', { name: 'State diagram' });
  const box = (await svg.boundingBox())!;
  await svg.dblclick({ position: { x: 40, y: box.height - 60 } });
  await expect(page.getByRole('button', { name: /^State q3/ })).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('menu', { name: 'State q3' })).toBeVisible();
  await page.getByRole('menuitem', { name: 'Add transition' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: 'Create' }).click();
  await page.getByRole('tab', { name: 'Text' }).click();
  await expect(page.locator('.cm-content')).toContainText('δ(q3, 0) = (q3, 0, R)');
});

test('a share link reproduces the machine and the input', async ({ page, context, browser }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await open(page);
  await page.getByRole('tab', { name: 'Table' }).click();
  const cell = page.getByRole('textbox', { name: 'δ(q2, 0)' });
  await cell.click();
  await cell.fill('q2, 0, R');
  await cell.press('Enter');
  const input = page.getByLabel('Input', { exact: true });
  await input.fill('111');
  await page.getByRole('button', { name: 'Share' }).click();
  await expect(page).toHaveURL(/#src=.+&in=111$/);
  const url = page.url();

  const other = await browser.newPage();
  await other.goto(url);
  await expect(other.getByLabel('Input', { exact: true })).toHaveValue('111');
  await other.getByRole('tab', { name: 'Table' }).click();
  await expect(other.getByRole('textbox', { name: 'δ(q2, 0)' })).toHaveValue('q2, 0, R');
  await other.close();
});

test('gallery links load an example', async ({ page }) => {
  await open(page, '#g=unary-multiplication&in=%23xx%23xxx%23');
  await expect(page.getByLabel('Machine name')).toHaveValue('Unary multiplication');
  await expect(page.getByLabel('Input', { exact: true })).toHaveValue('#xx#xxx#');
});

test('the input is checked against the input alphabet', async ({ page }) => {
  await open(page);
  await page.getByLabel('Input', { exact: true }).fill('10a1');
  await expect(page.locator('#pg-input-help')).toContainText('a is not in the input alphabet');
  await expect(page.getByRole('button', { name: 'Run', exact: true })).toBeDisabled();
});

for (const scheme of ['light', 'dark'] as const) {
  test(`has no accessibility violations (${scheme})`, async ({ page }) => {
    await page.emulateMedia({ colorScheme: scheme });
    await open(page);
    for (const tab of ['Diagram', 'Table', 'Text']) {
      await page.getByRole('tab', { name: tab }).click();
      if (tab === 'Text') await expect(page.locator('.cm-content')).toBeVisible();
      const result = await new AxeBuilder({ page }).include('main').analyze();
      expect(result.violations, `${tab}: ${JSON.stringify(result.violations, null, 2)}`).toEqual(
        []
      );
    }
  });
}

test.describe('on a phone', () => {
  test.use({ viewport: { width: 375, height: 812 }, hasTouch: true });

  test('opens the table and runs the machine', async ({ page }) => {
    const errors = watchErrors(page);
    await open(page);
    await expect(page.getByRole('tab', { name: 'Table' })).toHaveAttribute('aria-selected', 'true');
    const overflow = await page
      .locator('main')
      .evaluate((main) => main.scrollWidth - main.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    await page.getByRole('button', { name: 'Run', exact: true }).tap();
    await expect(status(page)).toContainText('Halted', { timeout: 10_000 });
    await page.getByRole('tab', { name: 'Diagram' }).tap();
    await expect(page.getByText('Edit the machine in the table.')).toBeVisible();
    expect(errors).toEqual([]);
  });
});
