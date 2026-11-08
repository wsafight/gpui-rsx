import { expect, test, type Page } from '@playwright/test';

async function fitsViewport(page: Page) {
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
  expect(overflow).toBeLessThanOrEqual(1);
}

async function openMobileNavigation(page: Page) {
  const menu = page.locator('button[popovertarget="starlight__sidebar"]');
  if (await menu.isVisible()) {
    await menu.click();
    await expect(page.locator('#starlight__sidebar')).toBeVisible();
  }
}

for (const locale of ['', 'zh-cn/']) {
  for (const theme of ['light', 'dark']) {
    test.describe(`${locale || 'en/'} ${theme}`, () => {
      test.beforeEach(async ({ page }) => {
        await page.addInitScript(value => localStorage.setItem('starlight-theme', value), theme);
      });

      test('home, navigation and tables stay usable', async ({ page }, info) => {
        await page.goto(locale);
        await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
        await expect(page.locator('h1')).toContainText('GPUI');
        await fitsViewport(page);
        await page.screenshot({ path: info.outputPath('home.png'), fullPage: true });

        await page.goto(`${locale}getting-started/`);
        await openMobileNavigation(page);
        const demoLink = page.locator('#starlight__sidebar a').filter({ hasText: locale ? '可运行示例' : 'Runnable Demos' });
        await expect(demoLink).toHaveCount(1);
        await demoLink.click();
        await expect(page).toHaveURL(new RegExp(`/${locale}examples/demos/$`));
        await fitsViewport(page);

        await page.goto(`${locale}reference/api/`);
        const tables = page.locator('.sl-markdown-content table');
        expect(await tables.count()).toBeGreaterThan(0);
        await fitsViewport(page);
        const scrollers = await tables.evaluateAll(elements => elements
          .filter(table => table.scrollWidth > table.clientWidth)
          .map(table => {
            table.scrollLeft = table.scrollWidth;
            return table.scrollLeft;
          }));
        if (page.viewportSize()!.width < 600) {
          expect(scrollers.length).toBeGreaterThan(0);
          expect(scrollers.every(offset => offset > 0)).toBe(true);
        }
      });

      test('Playground edits, keyboard diagnostics and sharing work', async ({ page }) => {
        await page.goto(`${locale}playground/`);
        const source = page.locator('#rsx-source');
        const status = page.locator('#rsx-status');
        await expect(status).toHaveAttribute('data-state', 'ready');
        const invalid = '<div>{"中文😀"}<span class="flex-colum" /></div>';
        await source.fill(invalid);
        await expect(status).toHaveAttribute('data-state', 'error');
        const diagnostic = page.locator('#rsx-diagnostics button').filter({ hasText: 'flex-colum' }).first();
        await diagnostic.focus();
        await page.keyboard.press('Enter');
        await expect(source).toBeFocused();
        expect(await source.evaluate((element: HTMLTextAreaElement) => element.value.slice(element.selectionStart, element.selectionEnd))).toBe('flex-colum');

        const valid = '<div class="p-4 bg-brand">{"分享中文😀"}</div>';
        await source.fill(valid);
        await page.locator('.theme-panel summary').click();
        await expect(page.locator('#rsx-theme')).toBeVisible();
        await page.locator('#rsx-theme').fill('[colors]\nbrand = "#0f766e"');
        await page.locator('#rsx-tailwind').check();
        await expect(status).toHaveAttribute('data-state', 'ready');
        await expect(page.locator('#rsx-expansion')).toContainText('16f32');
        await expect.poll(async () => page.frameLocator('#rsx-preview').locator('body').innerText()).toContain('分享中文😀');
        await page.locator('#rsx-share').click();
        await expect(page).toHaveURL(/#.+/);
        await page.reload();
        await expect(source).toHaveValue(valid);
        await expect(page.locator('#rsx-theme')).toHaveValue('[colors]\nbrand = "#0f766e"');
        await expect(page.locator('#rsx-tailwind')).toBeChecked();
        await expect(status).toHaveAttribute('data-state', 'ready');
        await fitsViewport(page);
      });
    });
  }

  test(`${locale || 'en/'} theme choice survives navigation`, async ({ page }) => {
    await page.goto(`${locale}getting-started/`);
    await openMobileNavigation(page);
    await page.locator('starlight-theme-select select:visible').selectOption('dark');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await page.goto(`${locale}usage/syntax/`);
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await openMobileNavigation(page);
    await page.locator('starlight-theme-select select:visible').selectOption('light');
    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  });
}
