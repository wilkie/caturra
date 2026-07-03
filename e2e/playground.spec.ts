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

  test('runs a stage-1 program with locals, operators, and concat', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        int a = 6, b = 7;',
          '        String answer = "a * b = " + a * b;',
          '        System.out.println(answer);',
          '        System.out.println(a * b == 42 && a < b);',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    const consoleOutput = page.getByTestId('console');
    await expect(consoleOutput).toContainText('a * b = 42');
    await expect(consoleOutput).toContainText('true');
  });

  test('reports runtime exceptions like java does', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        int zero = 0;',
          '        System.out.println(1 / zero);',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText(
      'java.lang.ArithmeticException: / by zero',
    );
  });

  test('runs loops with control flow in the browser', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        String out = "";',
          '        for (int i = 1; i <= 5; i++) {',
          '            if (i == 3) continue;',
          '            out += i;',
          '        }',
          '        System.out.println(out);',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('1245');
  });

  test('gives friendly messages for future Java features', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        int[] nums = new int[3];',
          '        System.out.println(nums.length);',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText(
      'arrays are not yet supported by jvmjs',
    );
  });
});
