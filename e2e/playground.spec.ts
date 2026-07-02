import { expect, test } from '@playwright/test';

test.describe('playground', () => {
  test('loads the WASM engine in a worker and shows its version', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByTestId('engine-version')).toHaveText(/^engine v\d+\.\d+\.\d+$/);
    await expect(page.getByTestId('run')).toBeEnabled();
  });

  test('page is cross-origin isolated so blocking stdin can work', async ({ page }) => {
    await page.goto('/');
    expect(await page.evaluate(() => globalThis.crossOriginIsolated)).toBe(true);
  });

  test('compiles and runs Hello World end to end', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('run').click();
    const consoleOutput = page.getByTestId('console');
    await expect(consoleOutput).toContainText('$ javac Main.java');
    await expect(consoleOutput).toContainText('$ java Main');
    await expect(consoleOutput).toContainText('Hello, World!');
  });

  test('reports source errors with locations', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('source').fill('class Main { String s = "oops; }');
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText(
      'Main.java:1:25: error: unterminated string literal',
    );
  });

  test('gives friendly messages for future Java features', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        for (int i = 0; i < 3; i++) {',
          '            System.out.println(i);',
          '        }',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText(
      'for loops are not yet supported by jvmjs',
    );
  });
});
