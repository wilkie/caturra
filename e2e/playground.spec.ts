import { expect, test } from '@playwright/test';
import type { Page } from '@playwright/test';

/** The editor's automation hooks installed by the playground page. */
interface PlaygroundHooks {
  setSource: (text: string) => void;
  getSource: () => string;
  toggleBreakpoint: (line: number) => void;
  breakpointLines: () => number[];
  setFile: (name: string, text: string) => void;
  selectFile: (name: string) => void;
  activeFile: () => string;
  levelReady: () => Promise<void>;
  neighborhoodState: () => {
    colors: (string | null)[][];
    painters: { id: string; x: number; y: number; dir: string }[];
  };
}

/** Replace the CodeMirror document (typing via contenteditable is flaky). */
async function setSource(page: Page, text: string): Promise<void> {
  await page.waitForFunction(
    () => (window as unknown as { playground?: PlaygroundHooks }).playground !== undefined,
  );
  await page.evaluate((t) => {
    (window as unknown as { playground: PlaygroundHooks }).playground.setSource(t);
  }, text);
}

/** WCAG relative luminance of a computed `rgb(...)` color. */
function luminance(rgb: string): number {
  const match = /rgba?\((\d+),\s*(\d+),\s*(\d+)/.exec(rgb);
  if (!match) {
    throw new Error(`unparseable color: ${rgb}`);
  }
  const [r, g, b] = [match[1], match[2], match[3]].map((part) => {
    const channel = Number(part) / 255;
    return channel <= 0.039_28 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrastRatio(a: string, b: string): number {
  const first = luminance(a);
  const second = luminance(b);
  return (Math.max(first, second) + 0.05) / (Math.min(first, second) + 0.05);
}

/** Computed text color and effective (nearest opaque) background. */
async function colorPair(page: Page, selector: string): Promise<{ color: string; bg: string }> {
  return page.evaluate((sel) => {
    const element = document.querySelector(sel);
    if (!element) {
      throw new Error(`missing ${sel}`);
    }
    const color = getComputedStyle(element).color;
    let node: Element | null = element;
    while (node) {
      const bg = getComputedStyle(node).backgroundColor;
      if (bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent') {
        return { color, bg };
      }
      node = node.parentElement;
    }
    return { color, bg: 'rgb(255, 255, 255)' };
  }, selector);
}

async function setFile(page: Page, name: string, text: string): Promise<void> {
  await page.evaluate(
    ([n, t]) => {
      (window as unknown as { playground: PlaygroundHooks }).playground.setFile(n ?? '', t ?? '');
    },
    [name, text],
  );
}

async function selectFile(page: Page, name: string): Promise<void> {
  await page.evaluate((n) => {
    (window as unknown as { playground: PlaygroundHooks }).playground.selectFile(n);
  }, name);
}

/** Wait for the selected level's on-demand content chunk to finish loading. */
async function waitLevel(page: Page): Promise<void> {
  await page.evaluate(() =>
    (window as unknown as { playground: PlaygroundHooks }).playground.levelReady(),
  );
}

async function toggleBreakpoint(page: Page, line: number): Promise<void> {
  await page.evaluate((l) => {
    (window as unknown as { playground: PlaygroundHooks }).playground.toggleBreakpoint(l);
  }, line);
}

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

  test('loads a Unit 1 neighborhood level and animates it on the canvas', async ({ page }) => {
    await page.goto('/');
    // Choose the unit, then a real multi-file lesson level from it.
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 1' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Investigate and Modify: Debugging' });
    await waitLevel(page);
    await expect(page.getByTestId('viz')).toBeVisible();
    await expect(page.getByTestId('file-tabs')).toContainText('PainterPlus.java');

    await page.getByTestId('run').click();
    // The starting program animates the painter off its origin (0, 0).
    const handle = await page.waitForFunction(() => {
      const state = (
        window as unknown as { playground: PlaygroundHooks }
      ).playground.neighborhoodState();
      const painter = state.painters[0];
      return painter && (painter.x !== 0 || painter.y !== 0) ? state : null;
    });
    const state = (await handle.jsonValue()) as { painters: unknown[] };
    expect(state.painters.length).toBeGreaterThan(0);
  });

  test('loads a Unit 2 console level and runs it', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 2' });
    // A console unit: the level dropdown fills, the neighborhood canvas stays hidden.
    const levelSelect = page.getByTestId('level-select');
    await expect(levelSelect.locator('option')).not.toHaveCount(1);
    // Pick the first real level (skip the placeholder) and run it.
    const firstLevel = await levelSelect.locator('optgroup option').first().getAttribute('value');
    await levelSelect.selectOption(firstLevel);
    await waitLevel(page);
    await expect(page.getByTestId('viz')).toBeHidden();
    await page.getByTestId('run').click();
    // A console level compiles through javac (some Investigate levels start
    // with an intentional error to fix), and never drives the canvas.
    await expect(page.getByTestId('console')).toContainText('$ javac Main.java');
    await expect(page.getByTestId('viz')).toBeHidden();
  });
  test('renders a theater scene on the stage', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('theater-level').selectOption({ label: 'Shapes' });
    await expect(page.getByTestId('theater-viz')).toBeVisible();
    await page.getByTestId('run').click();
    // Wait until the aqua background is drawn, then sample the yellow circle.
    const handle = await page.waitForFunction(() => {
      const canvas = document.querySelector<HTMLCanvasElement>('[data-testid="theater-canvas"]');
      const ctx = canvas?.getContext('2d');
      if (!ctx) {
        return null;
      }
      const bg = ctx.getImageData(5, 5, 1, 1).data;
      if (!(bg[0] === 0 && bg[1] === 255 && bg[2] === 255)) {
        return null;
      }
      const circle = ctx.getImageData(200, 90, 1, 1).data;
      return { bg: [bg[0], bg[1], bg[2]], circle: [circle[0], circle[1], circle[2]] };
    });
    const value = (await handle.jsonValue()) as { bg: number[]; circle: number[] };
    expect(value.bg).toEqual([0, 255, 255]); // aqua clear()
    expect(value.circle).toEqual([255, 255, 0]); // yellow ellipse
  });

  test('Stop button interrupts a running program and the session recovers', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      'public class Main { public static void main(String[] a) { while (true) {} } }',
    );
    await page.getByTestId('run').click();
    // Stop is offered while running; Run is disabled.
    await expect(page.getByTestId('stop-run')).toBeVisible();
    await expect(page.getByTestId('run')).toBeDisabled();
    await page.getByTestId('stop-run').click();
    // The loop is interrupted and the controls reset.
    await expect(page.getByTestId('console')).toContainText('program stopped');
    await expect(page.getByTestId('stop-run')).toBeHidden();
    await expect(page.getByTestId('run')).toBeEnabled();
    // The respawned session still works.
    await setSource(
      page,
      'public class Main { public static void main(String[] a) { System.out.println("alive"); } }',
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('alive');
  });

  test('switching levels replaces a same-named file that is the active tab', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 3' });
    const level = page.getByTestId('level-select');
    await level.selectOption({ label: 'Practice: Writing Algorithms with 1D Arrays (a)' });
    await waitLevel(page);
    // Make the shared MusicSurvey.java the active tab — the bug trigger:
    // an active non-Main file used to survive the level switch with stale
    // content (here a 2-arg constructor) shadowing the new level's version.
    await selectFile(page, 'MusicSurvey.java');
    await level.selectOption({ label: 'Practice: Writing Algorithms with 1D Arrays (d)' });
    await waitLevel(page);
    const source = await page.evaluate(() => {
      const pg = (window as unknown as { playground: PlaygroundHooks }).playground;
      pg.selectFile('MusicSurvey.java');
      return pg.getSource();
    });
    expect(source).toContain('String timesFile, String agesFile, String hoursFile');
    expect(source).not.toContain('String agesFile, String streamingFile');
    // (d)'s 3-arg Main now matches its 3-arg constructor: compiles cleanly.
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('$ java Main');
    await expect(page.getByTestId('console')).not.toContainText('error:');
  });

  test('Solve then Test passes a corpus validator (with the local overlay)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 3' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: Writing Algorithms with 1D Arrays (d)' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    // The MusicSurvey validator (Arrays.toString of Respondent[]) passes
    // against the loaded solution.
    await expect(page.getByTestId('test-results').locator('.test-pass')).toContainText(
      'Reverse the elements',
    );
    await expect(page.getByTestId('console')).toContainText('1 / 1 tests passed');
  });

  test('Solve then Test passes a Unit 6 String validator', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 6' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: The substring() Method (a) #1' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 5 2D-array validator', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 5' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: 2D Array Elements (a)' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 4 reflection validator', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 4' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: Using Static Variables and Methods (a)' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 3 array validator', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 3' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: Creating 1D Arrays (a)' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 2 System.out validator (SystemOutTestRunner)', async ({
    page,
  }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 2' });
    await page.getByTestId('level-select').selectOption({ label: 'Practice: The Food Truck' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    // Captures the student program's System.out output and checks the lines.
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 2 reflection validator (Constructor.newInstance)', async ({
    page,
  }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 2' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Skill Building: Writing a Constructor (a)' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    // Uses ConstructorsHelper (getConstructor + newInstance + class literals).
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 1 EasyMock validator (partial mocks)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 1' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: PainterPlus Methods (a)' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    // The validator partial-mocks the Painter and verifies call counts.
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 1 reflection validator (Class.forName)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 1' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: Creating PainterPlus #1' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    // This validator uses Class.forName + isAssignableFrom + getClassNames.
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('Solve then Test passes a Unit 1 neighborhood validator (org.code.validation)', async ({
    page,
  }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 1' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Skill Building: Painter Objects' });
    await waitLevel(page);
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local overlay (validators/solutions not generated)');
    }
    await solve.click();
    await page.getByTestId('test').click();
    // The neighborhood harness runs the solution and asserts on the painter
    // action log; every test passes and none fail.
    await expect(page.getByTestId('test-results').locator('.test-fail')).toHaveCount(0);
    await expect(page.getByTestId('test-results').locator('.test-pass').first()).toBeVisible();
  });

  test('the Solve button loads the level solution (when the overlay is present)', async ({
    page,
  }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 3' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: Writing Algorithms with 1D Arrays (d)' });
    await waitLevel(page);
    // The Solve button only appears when the (dev-only) solution overlay is present.
    const solve = page.getByTestId('solve');
    if ((await solve.count()) === 0) {
      test.skip(true, 'no local solution overlay');
    }
    await solve.click();
    // The solution's completed reverseResponses replaces the start stub.
    await selectFile(page, 'MusicSurvey.java');
    const source = await page.evaluate(() =>
      (window as unknown as { playground: PlaygroundHooks }).playground.getSource(),
    );
    expect(source).toContain('responses[index] = responses[');
  });

  test('a level reads its .txt data files from the VFS (and shows them as tabs)', async ({
    page,
  }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 3' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Practice: Writing Algorithms with 1D Arrays (d)' });
    await waitLevel(page);
    // Data files load as editable tabs alongside the Java files.
    await expect(page.getByTestId('file-tabs')).toContainText('times.txt');
    await expect(page.getByTestId('file-tabs')).toContainText('ages.txt');
    await page.getByTestId('run').click();
    // The program reads the .txt files from the VFS and prints real rows.
    await expect(page.getByTestId('console')).toContainText('years old');
    await expect(page.getByTestId('console')).toContainText('hours per day');
  });

  test('loads and runs a Unit 3 (arrays) level', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 3' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Predict and Run: Storing Multiple Values' });
    await waitLevel(page);
    await page.getByTestId('run').click();
    // Reaching "$ java Main" means the array program compiled cleanly.
    await expect(page.getByTestId('console')).toContainText('$ java Main');
    await expect(page.getByTestId('console')).not.toContainText('error:');
  });

  test('runs a Constructors level using reflection (ConstructorsHelper)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 2' });
    await page
      .getByTestId('level-select')
      .selectOption({ label: 'Predict and Run: No-Argument Constructors' });
    await waitLevel(page);
    await expect(page.getByTestId('file-tabs')).toContainText('ConstructorsHelper.java');
    await page.getByTestId('run').click();
    const console = page.getByTestId('console');
    await expect(console).toContainText('Vlogger Class Constructors');
    await expect(console).toContainText('public Vlogger()');
  });

  test('runs an Attributes level using reflection (AttributesHelper)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('unit-select').selectOption({ label: 'CSA 2025 Unit 2' });
    await page.getByTestId('level-select').selectOption({ label: 'Practice: The Food Truck (a)' });
    await waitLevel(page);
    // The level bundles the reflection helper and the classes to complete.
    await expect(page.getByTestId('file-tabs')).toContainText('AttributesHelper.java');
    await page.evaluate(() => {
      const pg = (window as unknown as { playground: PlaygroundHooks }).playground;
      // Complete the exercise: declare a field, then print attributes.
      pg.setFile(
        'Dessert.java',
        'public class Dessert { private String flavor; private double price; }',
      );
      pg.setFile(
        'Main.java',
        'public class Main { public static void main(String[] args) { AttributesHelper.printAttributes(new Dessert()); } }',
      );
    });
    await page.getByTestId('run').click();
    const console = page.getByTestId('console');
    await expect(console).toContainText('Dessert Class Attributes');
    await expect(console).toContainText('private String flavor');
    await expect(console).toContainText('private double price');
  });

  test('loads an FRQ level and tests it against the corpus validator', async ({ page }) => {
    await page.goto('/');
    // Pick the AP FRQ unit and its first BoxOfCandy level.
    const unit = page.getByTestId('unit-select');
    await unit.selectOption({ label: 'AP FRQ Practice' });
    const level = page.getByTestId('level-select');
    await level.selectOption({ index: 1 });
    await waitLevel(page);
    // The level's student classes are loaded as tabs; its validator is hidden.
    await expect(page.getByTestId('file-tabs')).toContainText('BoxOfCandy.java');
    await expect(page.getByTestId('file-tabs')).toContainText('Candy.java');
    // Provide a minimal (unimplemented) BoxOfCandy so it compiles; the
    // teacher validator then runs and reports the test as failing.
    await page.evaluate(() => {
      (window as unknown as { playground: PlaygroundHooks }).playground.setFile(
        'BoxOfCandy.java',
        [
          'public class BoxOfCandy {',
          '  private Candy[][] box;',
          '  public boolean moveCandyToFirstRow(int col) { return false; }',
          '  public Candy removeNextByFlavor(String flavor) { return null; }',
          '}',
        ].join('\n'),
      );
    });
    await page.getByTestId('test').click();
    const results = page.getByTestId('test-results');
    await expect(results).toBeVisible();
    await expect(results.locator('.test-fail')).toContainText('MoveCandyToFirstRow');
  });

  test('runs a JUnit validator via the Test button', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      'public class Counter { private int count = 0; ' +
        'public void increment() { count++; } ' +
        'public int getCount() { return count; } }',
    );
    // A validator: one passing test, one deliberately failing.
    await page.evaluate(() => {
      (window as unknown as { playground: PlaygroundHooks }).playground.setFile(
        'CounterTest.java',
        [
          'import static org.junit.jupiter.api.Assertions.*;',
          'import org.junit.jupiter.api.Test;',
          'import org.junit.jupiter.api.Order;',
          'import org.junit.jupiter.api.DisplayName;',
          'public class CounterTest {',
          '  @Test @Order(1) @DisplayName("starts at zero")',
          '  public void a() { assertEquals(0, new Counter().getCount()); }',
          '  @Test @Order(2) @DisplayName("wrong on purpose")',
          '  public void b() { assertEquals(99, new Counter().getCount()); }',
          '}',
        ].join('\n'),
      );
    });
    await page.getByTestId('test').click();
    const results = page.getByTestId('test-results');
    await expect(results).toBeVisible();
    await expect(results.locator('.test-pass')).toContainText('starts at zero');
    await expect(results.locator('.test-fail')).toContainText('wrong on purpose');
    await expect(page.getByTestId('console')).toContainText('1 / 2 tests passed');
  });

  test('reports source errors with locations', async ({ page }) => {
    await page.goto('/');
    await setSource(page, 'class Main { String s = "oops; }');
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText(
      'Main.java:1:25: error: unterminated string literal',
    );
  });

  test('runs a stage-1 program with locals, operators, and concat', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
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
    await setSource(
      page,
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
    await setSource(
      page,
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
    await setSource(
      page,
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
    await setSource(
      page,
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
    await setSource(
      page,
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
    await setSource(
      page,
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

  test('Scanner reads the stdin box through SharedArrayBuffer', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'import java.util.Scanner;',
        '',
        'public class Main {',
        '    public static void main(String[] args) {',
        '        Scanner in = new Scanner(System.in);',
        '        String name = in.nextLine();',
        '        int age = in.nextInt();',
        '        System.out.println("Hello " + name + ", age " + (age + 1) + " next year!");',
        '    }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('stdin').fill('Ada\n36');
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('Hello Ada, age 37 next year!');
  });

  test('runs ArrayList programs in the browser', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'import java.util.ArrayList;',
        '',
        'public class Main {',
        '    public static void main(String[] args) {',
        '        ArrayList<Integer> squares = new ArrayList<>();',
        '        for (int i = 1; i <= 4; i++) squares.add(i * i);',
        '        System.out.println(squares);',
        '    }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('[1, 4, 9, 16]');
  });

  test('reads and writes virtual files in the browser', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'import java.io.File;',
        'import java.io.PrintWriter;',
        'import java.util.Scanner;',
        '',
        'public class Main {',
        '    public static void main(String[] args) throws Exception {',
        '        PrintWriter out = new PrintWriter("notes.txt");',
        '        out.println("virtual filesystem");',
        '        out.close();',
        '        Scanner in = new Scanner(new File("notes.txt"));',
        '        System.out.println("read back: " + in.nextLine());',
        '    }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('read back: virtual filesystem');
  });

  test('debugger pauses at a breakpoint, shows locals, steps, resumes', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    static int twice(int n) {',
        '        return n * 2;',
        '    }',
        '',
        '    public static void main(String[] args) {',
        '        int total = 0;',
        '        for (int i = 1; i <= 2; i++) {',
        '            total += twice(i);',
        '        }',
        '        System.out.println("total " + total);',
        '    }',
        '}',
      ].join('\n'),
    );
    // Flag line 9 (`total += twice(i);`) the traditional way.
    await toggleBreakpoint(page, 9);
    await page.getByTestId('debug').click();

    // First arrival: paused with locals i=1, total=0.
    const frames = page.getByTestId('frames');
    await expect(frames).toContainText('Main.main (Main.java:9)');
    await expect(frames).toContainText('i = 1');
    await expect(frames).toContainText('total = 0');

    // Step into twice(): the callee frame appears with n = 1.
    await page.getByTestId('step-into').click();
    await expect(frames).toContainText('Main.twice (Main.java:3)');
    await expect(frames).toContainText('n = 1');

    // Continue: second breakpoint arrival, loop state advanced.
    await page.getByTestId('resume').click();
    await expect(frames).toContainText('i = 2');
    await expect(frames).toContainText('total = 2');

    // Continue to completion.
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('total 6');
    await expect(page.getByTestId('debug-bar')).toBeHidden();
  });

  test('debugger stop button terminates the program', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        System.out.println("before");',
        '        System.out.println("after");',
        '    }',
        '}',
      ].join('\n'),
    );
    await toggleBreakpoint(page, 4);
    await page.getByTestId('debug').click();
    await expect(page.getByTestId('frames')).toContainText('Main.main (Main.java:4)');
    await page.getByTestId('stop').click();
    await expect(page.getByTestId('console')).toContainText('(stopped by the debugger)');
    const consoleText = await page.getByTestId('console').textContent();
    expect(consoleText).toContain('before');
    expect(consoleText).not.toContain('after');
  });

  test('clicking a line number toggles a breakpoint dot and the debugger honors it', async ({
    page,
  }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        int a = 1;',
        '        int b = 2;',
        '        System.out.println(a + b);',
        '    }',
        '}',
      ].join('\n'),
    );

    // A real gutter interaction: click the "4" line number.
    await page.locator('.cm-lineNumbers .cm-gutterElement', { hasText: /^4$/ }).click();
    await expect(page.locator('.cm-breakpoint-dot')).toHaveCount(1);

    // Clicking again clears it; once more re-sets it for the run.
    await page.locator('.cm-lineNumbers .cm-gutterElement', { hasText: /^4$/ }).click();
    await expect(page.locator('.cm-breakpoint-dot')).toHaveCount(0);
    await page.locator('.cm-lineNumbers .cm-gutterElement', { hasText: /^4$/ }).click();

    await page.getByTestId('debug').click();
    await expect(page.getByTestId('frames')).toContainText('Main.main (Main.java:4)');
    // The paused line is highlighted in the editor.
    await expect(page.locator('.cm-paused-line')).toHaveCount(1);
    await expect(page.locator('.cm-paused-line')).toContainText('int b = 2;');

    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('3');
    await expect(page.locator('.cm-paused-line')).toHaveCount(0);
  });

  test('breakpoints toggled while paused take effect on resume', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        System.out.println("one");',
        '        System.out.println("two");',
        '        System.out.println("three");',
        '    }',
        '}',
      ].join('\n'),
    );
    await toggleBreakpoint(page, 3);
    await page.getByTestId('debug').click();
    await expect(page.getByTestId('frames')).toContainText('Main.java:3');

    // While paused, add a breakpoint on line 5 and drop the one on 3.
    await toggleBreakpoint(page, 5);
    await toggleBreakpoint(page, 3);
    await page.getByTestId('resume').click();

    // It pauses again — at the new line, after printing one and two.
    await expect(page.getByTestId('frames')).toContainText('Main.java:5');
    const consoleText = await page.getByTestId('console').textContent();
    expect(consoleText).toContain('two');
    expect(consoleText).not.toContain('three');
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('three');
  });

  test('compiler errors appear as squiggles with hover messages', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        int x = yy + 1;',
        '        System.out.println(x);',
        '    }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText("cannot find variable 'yy'");

    // The squiggle sits on the offending token.
    const squiggle = page.locator('.cm-lintRange-error');
    await expect(squiggle).toHaveCount(1);
    await expect(squiggle).toContainText('yy');

    // Hovering shows the compiler message.
    await squiggle.hover();
    await expect(page.locator('.cm-tooltip-lint')).toContainText("cannot find variable 'yy'");

    // Fixing the code and re-running clears the squiggle.
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        int yy = 2;',
        '        int x = yy + 1;',
        '        System.out.println(x);',
        '    }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('3');
    await expect(page.locator('.cm-lintRange-error')).toHaveCount(0);
  });

  test('watch expressions evaluate at pauses and update while stepping', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'import java.util.ArrayList;',
        '',
        'public class Main {',
        '    public static void main(String[] args) {',
        '        ArrayList<Integer> nums = new ArrayList<>();',
        '        int total = 0;',
        '        for (int i = 1; i <= 3; i++) {',
        '            nums.add(i * i);',
        '            total += i;',
        '        }',
        '        System.out.println(total);',
        '    }',
        '}',
      ].join('\n'),
    );
    await toggleBreakpoint(page, 9); // total += i;
    await page.getByTestId('debug').click();
    await expect(page.getByTestId('frames')).toContainText('Main.java:9');

    // Add a watch while paused: evaluates immediately, still paused.
    await page.getByTestId('watch-input').fill('nums.size() * 10 + total');
    await page.getByTestId('watch-add').click();
    await expect(page.getByTestId('watches')).toContainText('nums.size() * 10 + total = 10');

    // Continue to the next arrival: the value tracks live state.
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('watches')).toContainText('nums.size() * 10 + total = 21');

    // A bad watch shows the compiler's message inline.
    await page.getByTestId('watch-input').fill('nosuch + 1');
    await page.getByTestId('watch-add').click();
    await expect(page.getByTestId('watches')).toContainText("cannot find variable 'nosuch'");

    await page.getByTestId('resume').click();
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('6');
  });

  test('multiple files compile together and debug across tabs', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        Greeter g = new Greeter("Ada");',
        '        System.out.println(g.greet());',
        '    }',
        '}',
      ].join('\n'),
    );
    await setFile(
      page,
      'Greeter.java',
      [
        'public class Greeter {',
        '    private String name;',
        '',
        '    public Greeter(String name) {',
        '        this.name = name;',
        '    }',
        '',
        '    public String greet() {',
        '        String message = "Hello, " + name + "!";',
        '        return message;',
        '    }',
        '}',
      ].join('\n'),
    );

    // A breakpoint inside the second file.
    await toggleBreakpoint(page, 9); // String message = ...;
    await selectFile(page, 'Main.java');
    await page.getByTestId('debug').click();

    // Paused in Greeter.java: the tab auto-switches, the frame shows
    // the right file, and the field-expanded `this` is visible.
    await expect(page.getByTestId('frames')).toContainText('Greeter.greet (Greeter.java:9)');
    await expect(page.getByTestId('frames')).toContainText('this = Greeter@');
    await expect(page.getByTestId('frames')).toContainText('name="Ada"');
    const active = await page.evaluate(() =>
      (window as unknown as { playground: PlaygroundHooks }).playground.activeFile(),
    );
    expect(active).toBe('Greeter.java');
    await expect(page.locator('.cm-paused-line')).toContainText('String message');

    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('Hello, Ada!');
  });

  test('diagnostics land in the right file tab', async ({ page }) => {
    await page.goto('/');
    await setFile(page, 'Broken.java', 'public class Broken { int x = nope; }');
    await selectFile(page, 'Main.java');
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('Broken.java:1');
    // No squiggle on Main.java (the active tab)...
    await expect(page.locator('.cm-lintRange-error')).toHaveCount(0);
    // ...but switching to Broken.java shows it.
    await selectFile(page, 'Broken.java');
    await expect(page.locator('.cm-lintRange-error')).toHaveCount(1);
  });

  test('try/catch runs in the browser and the debugger pauses in handlers', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        try {',
        '            int[] data = new int[2];',
        '            data[7] = 1;',
        '        } catch (ArrayIndexOutOfBoundsException e) {',
        '            System.out.println("caught: " + e.getMessage());',
        '        }',
        '        System.out.println("recovered");',
        '    }',
        '}',
      ].join('\n'),
    );
    // Breakpoint inside the catch handler.
    await toggleBreakpoint(page, 7);
    await page.getByTestId('debug').click();
    await expect(page.getByTestId('frames')).toContainText('Main.java:7');
    // The caught exception is a named, inspectable local.
    await expect(page.getByTestId('frames')).toContainText(
      'e = java.lang.ArrayIndexOutOfBoundsException: Index 7 out of bounds for length 2',
    );
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText(
      'caught: Index 7 out of bounds for length 2',
    );
    await expect(page.getByTestId('console')).toContainText('recovered');
  });

  test('gives friendly messages for future Java features', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        synchronized (args) {',
        '            System.out.println("hi");',
        '        }',
        '    }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('console')).toContainText('not yet supported by caturra');
  });
});

test.describe('swing (accessible DOM)', () => {
  test('renders the component tree as native, named, accessible controls', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Sign-up form' });
    await expect(page.getByTestId('swing-viz')).toBeVisible();
    await page.getByTestId('run').click();

    // The rendered UI is real HTML, so we can assert on the accessibility
    // tree a screen reader would see — roles and accessible names, not pixels.
    const root = page.getByTestId('swing-root');
    await expect(root.getByRole('button', { name: 'Submit' })).toBeVisible();

    // The window is a named region landmark (JFrame title).
    await expect(page.getByRole('region', { name: 'Sign Up' })).toBeVisible();

    // setLabelFor gives each text field an accessible name via <label for>.
    await expect(root.getByRole('textbox', { name: 'Name:' })).toBeVisible();
    await expect(root.getByRole('textbox', { name: 'Email:' })).toBeVisible();

    // The checkbox takes its name from its wrapping label and starts checked.
    await expect(root.getByRole('checkbox', { name: 'Email me updates' })).toBeChecked();
  });

  test('controls are keyboard navigable in the order components were added', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Sign-up form' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // Start in the first field, then Tab through: labels are not focusable,
    // so focus lands on field → field → checkbox → button, matching add order.
    const nameField = root.getByRole('textbox', { name: 'Name:' });
    await nameField.focus();
    await expect(nameField).toBeFocused();

    await page.keyboard.press('Tab');
    await expect(root.getByRole('textbox', { name: 'Email:' })).toBeFocused();

    await page.keyboard.press('Tab');
    await expect(root.getByRole('checkbox', { name: 'Email me updates' })).toBeFocused();

    await page.keyboard.press('Tab');
    await expect(root.getByRole('button', { name: 'Submit' })).toBeFocused();
  });

  test('a disabled button renders as a disabled control (not focusable)', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'import javax.swing.*;',
        'public class Main {',
        '  public static void main(String[] args) {',
        '    JFrame frame = new JFrame("Demo");',
        '    JButton go = new JButton("Go");',
        '    go.setEnabled(false);',
        '    frame.add(go);',
        '    frame.setVisible(true);',
        '  }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    await expect(page.getByTestId('swing-viz')).toBeVisible();
    await expect(page.getByTestId('swing-root').getByRole('button', { name: 'Go' })).toBeDisabled();
  });

  test('Swing widget text has sufficient contrast in dark mode', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Sign-up form' });
    await page.getByTestId('run').click();
    await expect(
      page.getByTestId('swing-root').getByRole('button', { name: 'Submit' }),
    ).toBeVisible();
    const button = await colorPair(page, '.swing-button');
    expect(contrastRatio(button.color, button.bg)).toBeGreaterThan(4);
  });
});

test.describe('playground in dark mode', () => {
  test.use({ colorScheme: 'dark' });

  test('editor, tooltip, and paused view stay legible', async ({ page }) => {
    await page.goto('/');

    // The editor follows the scheme: dark background, not the light
    // default sitting on a dark page.
    const editor = await colorPair(page, '#source .cm-content');
    expect(luminance(editor.bg)).toBeLessThan(0.5);
    expect(contrastRatio(editor.color, editor.bg)).toBeGreaterThan(4);

    // Syntax tokens too: the light theme's dark keyword colors on a
    // dark background is exactly the bug this guards against.
    const keyword = await page.evaluate(() => {
      const spans = [...document.querySelectorAll('#source .cm-line span')];
      const target = spans.find((span) => span.textContent === 'public');
      if (!target) {
        throw new Error('no highlighted keyword token found');
      }
      return getComputedStyle(target).color;
    });
    expect(contrastRatio(keyword, editor.bg)).toBeGreaterThan(3);

    // Squiggle hover tooltip: readable text on its background.
    await setSource(page, 'public class Main { int x = yy; }');
    await page.getByTestId('run').click();
    await expect(page.locator('.cm-lintRange-error')).toHaveCount(1);
    await page.locator('.cm-lintRange-error').hover();
    await expect(page.locator('.cm-tooltip-lint')).toBeVisible();
    const tooltip = await colorPair(page, '.cm-tooltip-lint');
    expect(contrastRatio(tooltip.color, tooltip.bg)).toBeGreaterThan(4);

    // Paused debugger view: the frames/locals text is readable.
    await setSource(
      page,
      [
        'public class Main {',
        '    public static void main(String[] args) {',
        '        int a = 41;',
        '        System.out.println(a + 1);',
        '    }',
        '}',
      ].join('\n'),
    );
    await toggleBreakpoint(page, 4);
    await page.getByTestId('debug').click();
    await expect(page.getByTestId('frames')).toContainText('a = 41');
    const frames = await colorPair(page, '#frames');
    expect(contrastRatio(frames.color, frames.bg)).toBeGreaterThan(4);
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('42');
  });
});
