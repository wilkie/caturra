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

  test('runs recursive methods in the browser', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    static int fib(int n) {',
          '        if (n < 2) return n;',
          '        return fib(n - 1) + fib(n - 2);',
          '    }',
          '',
          '    public static void main(String[] args) {',
          '        System.out.println("fib(12) = " + fib(12));',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('fib(12) = 144');
  });

  test('runs array programs in the browser', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        int[] scores = {90, 85, 77, 100};',
          '        int best = scores[0];',
          '        for (int s : scores) {',
          '            if (s > best) best = s;',
          '        }',
          '        System.out.println("best of " + scores.length + ": " + best);',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('best of 4: 100');
  });

  test('runs user-defined classes in the browser', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'class Dice {',
          '    private int sides;',
          '    Dice(int sides) { this.sides = sides; }',
          '    int max() { return sides; }',
          '}',
          '',
          'public class Main {',
          '    public static void main(String[] args) {',
          '        Dice d = new Dice(20);',
          '        System.out.println("d" + d.max());',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('d20');
  });

  test('runs polymorphic hierarchies in the browser', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'interface Noise { String sound(); }',
          'class Cat implements Noise {',
          '    public String sound() { return "meow"; }',
          '}',
          'class Cow implements Noise {',
          '    public String sound() { return "moo"; }',
          '}',
          '',
          'public class Main {',
          '    public static void main(String[] args) {',
          '        Noise[] farm = { new Cat(), new Cow() };',
          '        for (Noise n : farm) System.out.print(n.sound() + " ");',
          '        System.out.println();',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('meow moo');
  });

  test('gives friendly messages for future Java features', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('source')
      .fill(
        [
          'public class Main {',
          '    public static void main(String[] args) {',
          '        Scanner in = new Scanner(System.in);',
          '        System.out.println(in.nextLine());',
          '    }',
          '}',
        ].join('\n'),
      );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('unknown type');
  });
});
