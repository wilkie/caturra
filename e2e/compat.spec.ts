import { expect, test } from '@playwright/test';

/**
 * The compatibility page claims things about Java, and every claim is a program it
 * can run. This drives the page in a real browser and makes it prove all of them —
 * the same engine, the same WASM, that a visitor gets.
 *
 * `tests/compat_manifest.rs` already checks the manifest against a real
 * `javac`/`java`. What that CANNOT check is whether the page's own machinery works:
 * that it loads the engine, compiles in the browser, and judges the result honestly.
 * That is what this is for. A page that reports "verified" while running nothing
 * would sail through the Rust test and fail here.
 */
test.describe('java compatibility page', () => {
  test('every feature the page claims, it proves in the browser', async ({ page }) => {
    const failures: string[] = [];
    page.on('pageerror', (error) => failures.push(error.message));

    await page.goto('/compat.html');
    await expect(page.getByRole('heading', { name: 'Java compatibility', level: 1 })).toBeVisible();

    const runAll = page.locator('[data-run-all]');
    await expect(runAll).toBeEnabled({ timeout: 30_000 }); // the engine has to load
    await runAll.click();

    // Every card ends up with a verdict, and every verdict is a pass: a supported
    // feature reproduced the JDK's output, and a gap was declined for the documented
    // reason. Generous timeout — this compiles and runs ~33 Java programs.
    const cards = page.locator('[data-feature]');
    const total = await cards.count();
    expect(total).toBeGreaterThan(20);

    const verdicts = page.locator('[data-verdict]');
    await expect(verdicts).toHaveCount(total, { timeout: 180_000 });
    await expect(page.locator('.verdict-fail')).toHaveCount(0);
    await expect(page.locator('.verdict-pass')).toHaveCount(total + 1); // +1: the summary

    await expect(page.getByText(`${String(total)} verified in your browser`)).toBeVisible();
    expect(failures).toEqual([]);
  });

  test('a supported feature shows its program, its output, and the JDK it matched', async ({
    page,
  }) => {
    await page.goto('/compat.html');
    const card = page.locator('[data-feature="hashmap"]');
    await expect(card).toBeVisible();

    // The claim is inspectable: the actual Java is right there.
    await card.getByRole('button', { name: 'read the program' }).click();
    await expect(card.locator('.source')).toContainText(
      'Map<String, Integer> votes = new HashMap<>()',
    );

    await expect(page.locator('[data-run-all]')).toBeEnabled({ timeout: 30_000 });
    await card.locator('[data-run="hashmap"]').click();

    await expect(card.locator('[data-verdict="hashmap"]')).toHaveText('matches the JDK', {
      timeout: 60_000,
    });
    // Both columns: what this browser just printed, and what a real JDK printed.
    // Note the order — `{alan=2, grace=5, ada=4}`, the JDK's own HashMap bucket
    // order, which is not alphabetical and not insertion order. Reproducing THAT
    // is the claim.
    const outputs = card.locator('.out');
    await expect(outputs.first()).toContainText('{alan=2, grace=5, ada=4}');
    await expect(outputs.last()).toContainText('{alan=2, grace=5, ada=4}');
  });

  test('a gap is honest: it is refused, and the page prints the reason', async ({ page }) => {
    await page.goto('/compat.html');
    const card = page.locator('[data-feature="iterator"]');
    await expect(card).toBeVisible();
    await expect(card).toContainText('caturra says:');

    await expect(page.locator('[data-run-all]')).toBeEnabled({ timeout: 30_000 });
    await card.locator('[data-run="iterator"]').click();

    await expect(card.locator('[data-verdict="iterator"]')).toHaveText(
      'declined, exactly as documented',
      { timeout: 60_000 },
    );
    await expect(card.locator('.out')).toContainText('not supported by caturra');
  });
});
