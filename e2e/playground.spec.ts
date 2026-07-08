import { expect, test } from '@playwright/test';
import type { Locator, Page } from '@playwright/test';

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

  test('a breakpoint inside a lambda in a second file pauses at the right file', async ({
    page,
  }) => {
    await page.goto('/');
    await setSource(
      page,
      'public class Main { public static void main(String[] a) { Helper.run(); } }',
    );
    // The lambda lives in Helper.java (a non-first unit). Its synthesized
    // class must advertise Helper.java, or the breakpoint would never match.
    await setFile(
      page,
      'Helper.java',
      [
        'interface Task { void go(); }',
        'public class Helper {',
        '  static int result = 0;',
        '  static void run() {',
        '    Task t = () -> {',
        '      Helper.result = 42;',
        '      System.out.println("ran " + Helper.result);',
        '    };',
        '    t.go();',
        '  }',
        '}',
      ].join('\n'),
    );
    await toggleBreakpoint(page, 6); // Helper.result = 42; — inside the lambda
    await selectFile(page, 'Main.java');
    await page.getByTestId('debug').click();

    // It pauses inside the lambda, at Helper.java:6 (not Main.java).
    await expect(page.getByTestId('frames')).toContainText('Helper.java:6');
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('ran 42');
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

test.describe('swing (interactive)', () => {
  test('a button click runs its listener in the VM and re-renders', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Click counter' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const button = root.getByRole('button', { name: 'Click me' });
    await expect(button).toBeVisible();
    await expect(root).toContainText('Clicks: 0');

    // Each click round-trips to the parked VM event loop, runs the Java
    // ActionListener (count++, label.setText), and re-renders the tree.
    await button.click();
    await expect(root).toContainText('Clicks: 1');
    await button.click();
    await expect(root).toContainText('Clicks: 2');

    // The program is still live (an interactive UI never "completes"): Stop
    // is offered and Run stays disabled until it is pressed.
    await expect(page.getByTestId('stop-run')).toBeVisible();
  });

  test('the button is operable from the keyboard (Enter and Space)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Click counter' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const button = root.getByRole('button', { name: 'Click me' });
    await expect(button).toBeVisible();

    // Focus the button and activate it with the keyboard; focus is restored
    // across the re-render, so repeated key presses keep hitting it.
    await button.focus();
    await expect(button).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(root).toContainText('Clicks: 1');
    await expect(button).toBeFocused();
    await page.keyboard.press('Space');
    await expect(root).toContainText('Clicks: 2');
  });

  test('combo box, slider, and checkbox dispatch their listeners', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Controls demo' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    await expect(root).toContainText('1 Small pizza');

    // Each control exposes the right ARIA role and an accessible name (via
    // setLabelFor / its own label), and firing it updates the summary in the
    // VM. A combo box selection fires its ActionListener.
    await root.getByRole('combobox', { name: 'Size:' }).selectOption({ label: 'Large' });
    await expect(root).toContainText('Large');

    // A slider fires its ChangeListener.
    await root.getByRole('slider', { name: 'Quantity:' }).fill('3');
    await expect(root).toContainText('3 Large pizza');

    // A checkbox toggle fires its ItemListener.
    await root.getByRole('checkbox', { name: 'Extra cheese' }).check();
    await expect(root).toContainText('with extra cheese');
  });

  test('radio buttons in a ButtonGroup are single-select and dispatch', async ({ page }) => {
    await page.goto('/');
    await setSource(
      page,
      [
        'import javax.swing.*;',
        'import java.awt.*;',
        'public class Main {',
        '  public static void main(String[] args) {',
        '    JFrame frame = new JFrame("Pick");',
        '    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);',
        '    JLabel out = new JLabel("none");',
        '    JRadioButton a = new JRadioButton("Cats");',
        '    JRadioButton b = new JRadioButton("Dogs");',
        '    ButtonGroup g = new ButtonGroup();',
        '    g.add(a);',
        '    g.add(b);',
        '    a.addActionListener(e -> out.setText("Cats"));',
        '    b.addActionListener(e -> out.setText("Dogs"));',
        '    frame.add(a);',
        '    frame.add(b);',
        '    frame.add(out);',
        '    frame.setVisible(true);',
        '  }',
        '}',
      ].join('\n'),
    );
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const cats = root.getByRole('radio', { name: 'Cats' });
    const dogs = root.getByRole('radio', { name: 'Dogs' });
    await expect(cats).toBeVisible();

    await cats.check();
    await expect(root).toContainText('Cats');
    await expect(cats).toBeChecked();

    // Selecting the other radio deselects the first (shared ButtonGroup name).
    await dogs.check();
    await expect(root).toContainText('Dogs');
    await expect(dogs).toBeChecked();
    await expect(cats).not.toBeChecked();
  });

  test('a custom-painted panel renders to an accessible canvas and repaints', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Custom drawing' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    // The bitmap drawing is exposed as a named image for assistive tech.
    await expect(root.getByRole('img', { name: 'An orange circle you can grow' })).toBeVisible();

    // Sample the canvas: the orange circle sits at the centre; a point above
    // it is background until the circle grows into it.
    const sample = (x: number, y: number): Promise<number[]> =>
      page.evaluate(
        ([px, py]) => {
          const canvas = document.querySelector<HTMLCanvasElement>(
            '[data-testid="swing-root"] canvas',
          );
          const ctx = canvas?.getContext('2d');
          if (!ctx) {
            return [-1, -1, -1];
          }
          const d = ctx.getImageData(px, py, 1, 1).data;
          return [d[0] ?? 0, d[1] ?? 0, d[2] ?? 0];
        },
        [x, y],
      );

    await expect.poll(() => sample(120, 100)).toEqual([255, 140, 0]); // centre is orange
    expect(await sample(120, 70)).not.toEqual([255, 140, 0]); // 30px up is background

    // Grow twice (radius 20 -> 50): the circle now covers the upper point.
    await root.getByRole('button', { name: 'Grow' }).click();
    await root.getByRole('button', { name: 'Grow' }).click();
    await expect.poll(() => sample(120, 70)).toEqual([255, 140, 0]);
  });

  test('a MouseListener fires with canvas coordinates (click to draw)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Click to draw' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const canvas = root.getByRole('img', { name: 'Click to add a dot' });
    await expect(canvas).toBeVisible();

    const sample = (x: number, y: number): Promise<number[]> =>
      page.evaluate(
        ([px, py]) => {
          const c = document.querySelector<HTMLCanvasElement>('[data-testid="swing-root"] canvas');
          const ctx = c?.getContext('2d');
          if (!ctx) {
            return [-1, -1, -1];
          }
          const d = ctx.getImageData(px, py, 1, 1).data;
          return [d[0] ?? 0, d[1] ?? 0, d[2] ?? 0];
        },
        [x, y],
      );

    // Nothing drawn yet: the spot is background, not a blue dot.
    expect(await sample(100, 80)).not.toEqual([30, 90, 200]);

    // Clicking the canvas dispatches mousePressed with those coordinates; the
    // listener adds a dot there and the panel repaints.
    await canvas.click({ position: { x: 100, y: 80 } });
    await expect.poll(() => sample(100, 80)).toEqual([30, 90, 200]);
  });

  test('a Timer drives an animation (ticks advance on their own)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Bouncing ball' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    await expect(root).toContainText('ticks:');

    // No interaction: the Timer keeps firing and the tick count climbs.
    await expect
      .poll(
        async () => {
          const text = (await root.textContent()) ?? '';
          const match = /ticks: (\d+)/.exec(text);
          return match ? Number(match[1]) : 0;
        },
        { timeout: 5000 },
      )
      .toBeGreaterThan(5);

    // An animation never completes on its own: it stays running until Stop.
    await expect(page.getByTestId('stop-run')).toBeVisible();
  });

  test('a MouseMotionListener draws while dragging (coalesced)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Sketch pad' });
    await page.getByTestId('run').click();
    const canvas = page.getByTestId('swing-root').getByRole('img', { name: 'Drag to draw' });
    await expect(canvas).toBeVisible();
    const box = await canvas.boundingBox();
    if (!box) {
      throw new Error('no canvas box');
    }

    // Drag a horizontal stroke across the middle. Pauses between moves let the
    // event loop capture several coalesced positions (not just the last one).
    const y = box.y + box.height / 2;
    await page.mouse.move(box.x + 40, y);
    await page.mouse.down();
    for (const dx of [90, 140, 190, 240]) {
      await page.mouse.move(box.x + dx, y);
      await page.waitForTimeout(70);
    }
    await page.mouse.up();

    // Somewhere along the mid-line at x≈140 there is a stroke pixel. The
    // stroke is a 1px anti-aliased line, so check for the stroke's strong
    // green drop from the near-white background rather than an exact colour.
    const strokeNear = (px: number, y0: number, y1: number): Promise<boolean> =>
      page.evaluate(
        ([x, from, to]) => {
          const c = document.querySelector<HTMLCanvasElement>('[data-testid="swing-root"] canvas');
          const ctx = c?.getContext('2d');
          if (!ctx) {
            return false;
          }
          for (let yy = from; yy <= to; yy++) {
            const d = ctx.getImageData(x, yy, 1, 1).data;
            if ((d[1] ?? 255) < 200) {
              return true; // green pulled down toward the stroke's 30
            }
          }
          return false;
        },
        [px, y0, y1],
      );
    await expect.poll(() => strokeNear(140, 95, 125)).toBe(true);
  });

  test('a KeyListener moves a square with the arrow keys', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Keyboard mover' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const canvas = root.getByRole('img', { name: 'Use the arrow keys or WASD to move the square' });
    await expect(canvas).toBeVisible();
    // Starts centred; the board is focusable so the keys work immediately.
    await expect(root).toContainText('at (140, 90)');

    const sample = (x: number, y: number): Promise<number[]> =>
      page.evaluate(
        ([px, py]) => {
          const c = document.querySelector<HTMLCanvasElement>('[data-testid="swing-root"] canvas');
          const ctx = c?.getContext('2d');
          if (!ctx) {
            return [-1, -1, -1];
          }
          const d = ctx.getImageData(px, py, 1, 1).data;
          return [d[0] ?? 0, d[1] ?? 0, d[2] ?? 0];
        },
        [x, y],
      );

    // Each keydown nudges the square 10px right; poll the label between presses
    // so each event registers (the loop parks before the next press).
    for (const x of [150, 160, 170]) {
      await canvas.press('ArrowRight');
      await expect(root).toContainText(`at (${String(x)}, 90)`);
    }
    await canvas.press('ArrowDown');
    await expect(root).toContainText('at (170, 100)');

    // The blue square (fillRect, exact colour) now sits at its new centre, and
    // the old centre is background again.
    await expect.poll(() => sample(170, 100)).toEqual([80, 200, 255]);
    expect(await sample(140, 90)).not.toEqual([80, 200, 255]);
  });

  test('Graphics draws a filled polygon and arc with the fill colour', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Shapes and fonts' });
    await page.getByTestId('run').click();
    const canvas = page
      .getByTestId('swing-root')
      .getByRole('img', { name: 'A drawing with a polygon, an arc, and sized text' });
    await expect(canvas).toBeVisible();

    const sample = (x: number, y: number): Promise<number[]> =>
      page.evaluate(
        ([px, py]) => {
          const c = document.querySelector<HTMLCanvasElement>('[data-testid="swing-root"] canvas');
          const ctx = c?.getContext('2d');
          if (!ctx) {
            return [-1, -1, -1];
          }
          const d = ctx.getImageData(px, py, 1, 1).data;
          return [d[0] ?? 0, d[1] ?? 0, d[2] ?? 0];
        },
        [x, y],
      );

    // Inside the blue triangle (fillPolygon, exact fill colour).
    await expect.poll(() => sample(60, 90)).toEqual([80, 200, 255]);
    // Inside the orange pie slice (fillArc), above its centre (205, 75).
    await expect.poll(() => sample(205, 55)).toEqual([255, 180, 60]);
  });

  test('setFont styles a widget and Graphics2D.setStroke widens a line', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Shapes and fonts' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // The title JLabel's text is styled by setFont(SansSerif, BOLD, 20).
    // Scope to the label element so it isn't confused with the window titlebar.
    const title = root.locator('span.swing-label', { hasText: 'Shapes and Fonts' });
    await expect(title).toBeVisible();
    const style = await title.evaluate((el) => {
      const s = getComputedStyle(el);
      return { size: s.fontSize, weight: s.fontWeight };
    });
    expect(style.size).toBe('20px');
    expect(Number(style.weight)).toBeGreaterThanOrEqual(700);

    // The 6px red line is thick: a pixel 2px off its centre line (y=205) is
    // still on the stroke, which a 1px pen would not cover.
    const red = (x: number, y: number): Promise<boolean> =>
      page.evaluate(
        ([px, py]) => {
          const c = document.querySelector<HTMLCanvasElement>('[data-testid="swing-root"] canvas');
          const ctx = c?.getContext('2d');
          if (!ctx) {
            return false;
          }
          const d = ctx.getImageData(px, py, 1, 1).data;
          return (d[0] ?? 0) > 200 && (d[1] ?? 0) < 140 && (d[2] ?? 0) < 140;
        },
        [x, y],
      );
    await expect.poll(() => red(160, 205)).toBe(true); // centre of the line
    expect(await red(160, 203)).toBe(true); // 2px off — only a thick pen reaches here
  });

  test('the Look and Feel picker re-skins every Swing widget via the token layer', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Click counter' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const button = root.getByRole('button', { name: 'Click me' });
    await expect(button).toBeVisible();

    const radius = (): Promise<string> => button.evaluate((el) => getComputedStyle(el).borderTopLeftRadius);
    // Default (system) look: 4px radius.
    expect(await radius()).toBe('4px');

    // Metal: nearly square (2px) — no re-run needed, just the token swap.
    await page.getByTestId('swing-laf').selectOption('metal');
    await expect.poll(radius).toBe('2px');

    // High Contrast: square with a black border.
    await page.getByTestId('swing-laf').selectOption('contrast');
    await expect.poll(radius).toBe('0px');
    const border = await button.evaluate((el) => getComputedStyle(el).borderTopColor);
    expect(border).toBe('rgb(0, 0, 0)');
  });

  test('a menu accelerator fires its item from the keyboard', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Menus (JMenuBar)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    await expect(root.getByRole('menubar')).toBeVisible();

    // Ctrl+N triggers File > New without the menu ever being opened.
    await page.keyboard.press('Control+n');
    await expect(root).toContainText('New file');

    // The accelerator hint shows alongside the item. getByRole matching the
    // exact name "Open" proves the aria-hidden hint stays out of the name.
    await root.getByRole('button', { name: 'File' }).click();
    const open = root.getByRole('menuitem', { name: 'Open', exact: true });
    await expect(open).toContainText('Ctrl+O');
  });

  test('check and radio menu items toggle state and stay mutually exclusive', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Menu options' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // Open the View menu: a checkbox item (unchecked) toggles on click.
    await root.getByRole('button', { name: 'View' }).click();
    const wrap = root.getByRole('menuitemcheckbox', { name: 'Word Wrap' });
    await expect(wrap).toHaveAttribute('aria-checked', 'false');
    await wrap.click();
    await expect(root).toContainText('Wrap: on');

    // Reopen: Light is the selected radio; picking Dark switches the selection.
    await root.getByRole('button', { name: 'View' }).click();
    await expect(root.getByRole('menuitemradio', { name: 'Light' })).toHaveAttribute('aria-checked', 'true');
    const dark = root.getByRole('menuitemradio', { name: 'Dark' });
    await expect(dark).toHaveAttribute('aria-checked', 'false');
    await dark.click();
    await expect(root).toContainText('Theme: Dark');

    // Reopen: the group is now Dark-selected, Light cleared, and Wrap still on.
    await root.getByRole('button', { name: 'View' }).click();
    await expect(root.getByRole('menuitemradio', { name: 'Dark' })).toHaveAttribute('aria-checked', 'true');
    await expect(root.getByRole('menuitemradio', { name: 'Light' })).toHaveAttribute('aria-checked', 'false');
    await expect(root.getByRole('menuitemcheckbox', { name: 'Word Wrap' })).toHaveAttribute('aria-checked', 'true');
  });

  test('a GridBagLayout places labelled fields in a grid', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'GridBag form' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // Two labelled rows of fields.
    await expect(root.getByText('Name:')).toBeVisible();
    await expect(root.getByText('Email:')).toBeVisible();
    const fields = root.locator('input[type="text"]');
    await expect(fields).toHaveCount(2);

    // The submit button spans both columns (gridwidth = 2).
    const submit = root.getByRole('button', { name: 'Submit' });
    const gridColumn = await submit.evaluate((el) => el.style.gridColumn);
    expect(gridColumn).toBe('1 / span 2');

    // The name field sits in column 2 (gridx = 1), its label in column 1.
    const nameCol = await fields.first().evaluate((el) => el.style.gridColumn);
    expect(nameCol).toBe('2 / span 1');

    // It still works: submitting reads the field and updates the status.
    await fields.first().fill('Ada');
    await submit.click();
    await expect(root).toContainText('Submitted: Ada');
  });

  test('a JToggleButton stays pressed and reports its state', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Toggle buttons' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // A toggle button exposes aria-pressed; it starts unpressed.
    const bold = root.getByRole('button', { name: 'Bold', pressed: false });
    await expect(bold).toBeVisible();

    // Clicking presses it and fires the listener (the preview updates).
    await bold.click();
    await expect(root.getByRole('button', { name: 'Bold' })).toHaveAttribute('aria-pressed', 'true');
    await expect(root).toContainText('Style: bold');

    // Clicking again releases it.
    await root.getByRole('button', { name: 'Bold' }).click();
    await expect(root.getByRole('button', { name: 'Bold' })).toHaveAttribute('aria-pressed', 'false');
    await expect(root).toContainText('Style: plain');
  });

  test('a JPasswordField masks input and getPassword() reads it back', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Login form' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // The password field is a masked input (no textbox role); the user field
    // is the one text box.
    const pw = root.locator('input[type="password"]');
    await expect(pw).toBeVisible();
    const user = root.locator('input[type="text"]');
    await expect(user).toBeVisible();

    // Fill both; logging in reads user.getText() and pass.getPassword().
    await user.fill('ada');
    await pw.fill('secret');
    await root.getByRole('button', { name: 'Log in' }).click();
    await expect(root).toContainText('Welcome, ada (6 chars)');
  });

  test('preferred size, Color.darker, and horizontal alignment style widgets', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Styled widgets' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // setPreferredSize(200, 60) sizes the button; darker() shades (60,120,220).
    const big = root.getByRole('button', { name: 'Big button' });
    const style = await big.evaluate((el) => {
      const s = getComputedStyle(el);
      return { w: s.width, h: s.height, bg: s.backgroundColor };
    });
    expect(style.w).toBe('200px');
    expect(style.h).toBe('60px');
    expect(style.bg).toBe('rgb(42, 84, 154)'); // (60,120,220) * 0.7

    // JTextField.RIGHT right-aligns the field's text.
    const amount = root.getByRole('textbox');
    await expect(amount).toHaveCSS('text-align', 'right');

    // The button still dispatches; the label updates.
    await big.click();
    await expect(root.getByText('Clicked!')).toBeVisible();
  });

  test('SwingUtilities.invokeLater bootstraps and setVisible toggles a component', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Show / hide' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // The UI was built inside invokeLater and added via getContentPane().
    const secret = root.getByText('Now you see me!');
    await expect(secret).toBeVisible();

    // Toggling calls setVisible(false)/(true), hiding and showing the label.
    const toggle = root.getByRole('button', { name: 'Toggle' });
    await toggle.click();
    await expect(secret).toBeHidden();
    await toggle.click();
    await expect(secret).toBeVisible();
  });

  test('setLayout(null) positions children absolutely with setBounds', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Absolute layout' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // The button is placed by setBounds(90, 100, 180, 40): absolute at 90,100.
    const button = root.getByRole('button', { name: 'Hello, World!' });
    await expect(button).toBeVisible();
    const box = await button.evaluate((el) => {
      const s = getComputedStyle(el);
      return { position: s.position, left: s.left, top: s.top, width: s.width, height: s.height };
    });
    expect(box.position).toBe('absolute');
    expect(box.left).toBe('90px');
    expect(box.top).toBe('100px');
    expect(box.width).toBe('180px');
    expect(box.height).toBe('40px');

    // It still dispatches under the null layout: clicking updates the label.
    await button.click();
    await expect(root.getByText('Clicked the button!')).toBeVisible();
  });

  test('a JToolBar is an accessible strip with roving arrow-key focus', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Toolbar' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // An accessible toolbar named by setName, holding three buttons.
    const toolbar = root.getByRole('toolbar');
    await expect(toolbar).toBeVisible();
    await expect(toolbar).toHaveAttribute('aria-label', 'Formatting');
    await expect(toolbar).toHaveAttribute('aria-orientation', 'horizontal');
    await expect(toolbar.getByRole('button')).toHaveCount(3);

    // Roving focus: the toolbar is one Tab stop; arrows move between buttons.
    const bold = root.getByRole('button', { name: 'Bold' });
    await bold.focus();
    await page.keyboard.press('ArrowRight');
    const italic = root.getByRole('button', { name: 'Italic' });
    await expect(italic).toBeFocused();

    // A toolbar button dispatches its ActionListener.
    await italic.click();
    await expect(root).toContainText('Italic clicked');
  });

  test('a JSplitPane separates two panes with a resizable divider', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Split pane' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // A horizontal split has a vertical, keyboard-operable divider.
    const divider = root.getByRole('separator');
    await expect(divider).toBeVisible();
    await expect(divider).toHaveAttribute('aria-orientation', 'vertical');

    // Both panes render; a nav button on the left updates the content on the
    // right (proving each pane's components dispatch).
    await expect(root.getByText('Showing: Inbox')).toBeVisible();
    await root.getByRole('button', { name: 'Sent' }).click();
    await expect(root.getByText('Showing: Sent')).toBeVisible();

    // Arrow keys resize the first pane (client-side, no VM round trip).
    const before = Number(await divider.getAttribute('aria-valuenow'));
    await divider.press('ArrowRight');
    await divider.press('ArrowRight');
    const after = Number(await divider.getAttribute('aria-valuenow'));
    expect(after).toBeGreaterThan(before);
  });

  test('a JTabbedPane switches panels and fires its ChangeListener', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Tabbed pane' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // An accessible tablist of three tabs; the first is selected and its panel
    // (Welcome home!) is the only one shown.
    const tablist = root.getByRole('tablist');
    await expect(tablist).toBeVisible();
    await expect(tablist.getByRole('tab')).toHaveCount(3);
    const home = root.getByRole('tab', { name: 'Home' });
    await expect(home).toHaveAttribute('aria-selected', 'true');
    await expect(root.getByText('Welcome home!')).toBeVisible();
    await expect(root.getByText('Version 1.0')).toBeHidden();

    // Selecting the About tab switches the visible panel and fires the
    // ChangeListener, which updates the status label.
    await root.getByRole('tab', { name: 'About' }).click();
    await expect(root.getByText('Version 1.0')).toBeVisible();
    await expect(root.getByText('Welcome home!')).toBeHidden();
    await expect(root.getByRole('tab', { name: 'About' })).toHaveAttribute('aria-selected', 'true');
    await expect(root).toContainText('On tab: About');
  });

  test('BorderFactory renders titled, matte, and compound borders', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Bordered form' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    // A titled border is an accessible group box named by its caption, holding
    // the three checkboxes.
    const group = root.getByRole('group', { name: 'Preferences' });
    await expect(group).toBeVisible();
    await expect(group.getByRole('checkbox')).toHaveCount(3);
    // The caption is drawn via the ::before rule from the data attribute.
    await expect(group).toHaveAttribute('data-border-title', 'Preferences');

    // A matte border draws only the top and bottom sides, in the given colour.
    const banner = root.locator('span.swing-label', { hasText: /^Settings$/ });
    const matte = await banner.evaluate((el) => {
      const s = getComputedStyle(el);
      return { top: s.borderTopWidth, left: s.borderLeftWidth, color: s.borderTopColor };
    });
    expect(matte.top).toBe('4px');
    expect(matte.left).toBe('0px');
    expect(matte.color).toBe('rgb(220, 140, 40)');

    // A compound border: the outer line frame (2px, coloured) plus the inner
    // empty border's padding (6px vertical, 10px horizontal).
    const note = root.locator('span.swing-label', { hasText: 'Pick your settings above.' });
    const compound = await note.evaluate((el) => {
      const s = getComputedStyle(el);
      return {
        width: s.borderTopWidth,
        style: s.borderTopStyle,
        color: s.borderTopColor,
        padTop: s.paddingTop,
        padLeft: s.paddingLeft,
      };
    });
    expect(compound.width).toBe('2px');
    expect(compound.style).toBe('solid');
    expect(compound.color).toBe('rgb(60, 120, 220)');
    expect(compound.padTop).toBe('6px');
    expect(compound.padLeft).toBe('10px');
  });

  test('JOptionPane shows accessible modal dialogs (input, confirm, message)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Dialogs (JOptionPane)' });
    await page.getByTestId('run').click();

    // Input dialog: a focus-trapped modal with a text field. Each show*
    // blocks the program until answered.
    const dialog = page.getByRole('dialog');
    await expect(dialog).toBeVisible();
    await expect(dialog).toContainText("What's your name?");
    await page.getByTestId('dialog-input').fill('Ada');
    await dialog.getByRole('button', { name: 'OK' }).click();

    // Confirm dialog: Yes/No/Cancel; the entered name flowed back to the VM.
    await expect(dialog).toContainText('Nice to meet you, Ada.');
    await dialog.getByRole('button', { name: 'Yes' }).click();

    // Message dialog: the YES branch's greeting.
    await expect(dialog).toContainText('Hello, Ada!');
    await dialog.getByRole('button', { name: 'OK' }).click();

    // The program finishes and the controls reset.
    await expect(dialog).toBeHidden();
    await expect(page.getByTestId('run')).toBeEnabled();
  });

  test('a JMenuBar opens accessible menus; items dispatch and open dialogs', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Menus (JMenuBar)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    await expect(root.getByRole('menubar')).toBeVisible();

    // Keyboard: open the File menu, arrow to Open, and activate it — the menu
    // is a proper ARIA menu (menubar / menu / menuitem) with arrow navigation.
    await root.getByRole('button', { name: 'File' }).focus();
    await page.keyboard.press('Enter');
    // The popup opens as a top-layer popover, so it escapes the window's
    // `overflow: hidden` and is never clipped by a small frame.
    const asPopover = await page.evaluate(
      () => document.querySelector('[data-testid="swing-root"] .swing-menu-popup:popover-open') !== null,
    );
    expect(asPopover).toBe(true);
    await expect(root.getByRole('menuitem', { name: 'New' })).toBeFocused();
    await page.keyboard.press('ArrowDown');
    await expect(root.getByRole('menuitem', { name: 'Open' })).toBeFocused();
    await page.keyboard.press('Enter');
    await expect(root).toContainText('Opened a file');

    // Mouse: Help > About runs its listener, which opens a JOptionPane dialog.
    await root.getByRole('button', { name: 'Help' }).click();
    await root.getByRole('menu', { name: 'Help' }).getByRole('menuitem', { name: 'About' }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog).toContainText('Notes 1.0');
    await dialog.getByRole('button', { name: 'OK' }).click();
  });

  test('BorderLayout places children north/south/east/west/center; they dispatch', async ({
    page,
  }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'BorderLayout regions' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');

    const refresh = root.getByRole('button', { name: 'Refresh' }); // NORTH
    const sidebar = root.getByRole('button', { name: 'Sidebar' }); // WEST
    const details = root.getByText('Details'); // EAST
    const center = root.getByRole('textbox'); // CENTER
    const statusBar = root.getByText('Ready.'); // SOUTH
    await expect(refresh).toBeVisible();

    const centerOf = async (loc: Locator): Promise<{ x: number; y: number }> => {
      const b = await loc.boundingBox();
      if (!b) throw new Error('element has no bounding box');
      return { x: b.x + b.width / 2, y: b.y + b.height / 2 };
    };
    const north = await centerOf(refresh);
    const south = await centerOf(statusBar);
    const west = await centerOf(sidebar);
    const east = await centerOf(details);
    const middle = await centerOf(center);

    // The five regions sit where BorderLayout puts them, relative to CENTER.
    expect(north.y).toBeLessThan(middle.y);
    expect(south.y).toBeGreaterThan(middle.y);
    expect(west.x).toBeLessThan(middle.x);
    expect(east.x).toBeGreaterThan(middle.x);

    // The NORTH toolbar button dispatches to the SOUTH status bar.
    await refresh.click();
    await expect(root).toContainText('Refreshed the view');
  });

  test('a nested submenu cascades via keyboard; click-away collapses the bar', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Menus (JMenuBar)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const fileButton = root.getByRole('button', { name: 'File' });

    // Click-away: open File, then click outside the bar — it all collapses.
    // Click low in the window, clear of the popup (top-left) and close button.
    await fileButton.click();
    await expect(fileButton).toHaveAttribute('aria-expanded', 'true');
    await expect(root.getByRole('menuitem', { name: 'Export' })).toBeVisible();
    const box = await root.boundingBox();
    if (!box) throw new Error('swing-root has no bounding box');
    await page.mouse.click(box.x + box.width - 10, box.y + box.height - 10);
    await expect(fileButton).toHaveAttribute('aria-expanded', 'false');
    await expect(root.getByRole('menuitem', { name: 'Export' })).toBeHidden();

    // Keyboard: open File, arrow down to the Export submenu, open it with
    // ArrowRight, and activate a nested item.
    await fileButton.click();
    await page.keyboard.press('ArrowDown'); // New -> Open
    await page.keyboard.press('ArrowDown'); // Open -> Export
    const exportItem = root.getByRole('menuitem', { name: 'Export' });
    await expect(exportItem).toBeFocused();
    await expect(exportItem).toHaveAttribute('aria-haspopup', 'true');
    await page.keyboard.press('ArrowRight'); // open submenu, focus PNG
    await expect(root.getByRole('menu', { name: 'Export' })).toBeVisible();
    await expect(root.getByRole('menuitem', { name: 'PNG' })).toBeFocused();
    await page.keyboard.press('ArrowDown'); // PNG -> JPEG
    await page.keyboard.press('Enter');
    await expect(root).toContainText('Exported JPEG');
  });

  test('a JTextArea holds multi-line text that a listener reads back', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Notepad (JTextArea)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    // A <textarea> is a multiline textbox; setLabelFor gives it a name.
    const area = root.getByRole('textbox', { name: 'Notes:' });
    await expect(area).toBeVisible();

    // The typed value (with newlines) round-trips to the VM intact.
    await area.fill('first line\nsecond line\nthird');
    await root.getByRole('button', { name: 'Count' }).click();
    await expect(root).toContainText('3 line(s)');
  });

  test('a JScrollPane scrolls a large content region; items still dispatch', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Scroll pane (JScrollPane)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const pane = root.locator('.swing-scrollpane');
    await expect(pane).toBeVisible();

    // The wrapped list overflows the fixed viewport, so the pane scrolls.
    const overflows = await pane.evaluate((el) => el.scrollHeight > el.clientHeight + 4);
    expect(overflows).toBe(true);

    // A button near the bottom needs scrolling; Playwright scrolls it into the
    // pane and its listener still dispatches.
    await root.getByRole('button', { name: 'Item 20', exact: true }).click();
    await expect(root).toContainText('Picked Item 20');
  });

  test('a JList is a native list box; selecting an item fires its listener', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'List (JList)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    // A sized <select> exposes role "listbox"; setLabelFor gives it a name.
    const list = root.getByRole('listbox', { name: 'Pick a fruit:' });
    await expect(list).toBeVisible();

    // Selecting an option fires the ListSelectionListener; getSelectedValue
    // reads the picked item.
    await list.selectOption({ label: 'Cherry' });
    await expect(root).toContainText('You picked Cherry');

    // Keyboard: the native list box moves selection with the arrow keys.
    await list.focus();
    await page.keyboard.press('ArrowUp');
    await expect(root).toContainText('You picked Banana');
  });

  test('a multi-select JList reports every chosen value', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Multi-select list (JList)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const list = root.getByRole('listbox', { name: /Choose toppings/ });
    await expect(list).toBeVisible();
    // A native multi-select list box (the browser maps this into the
    // accessibility tree as multiselectable for screen readers).
    await expect(list).toHaveJSProperty('multiple', true);

    // Choosing several options reports all of them (via getSelectedValues).
    await list.selectOption([{ label: 'Cheese' }, { label: 'Pepperoni' }, { label: 'Olive' }]);
    await expect(root).toContainText('3 topping(s): Cheese, Pepperoni, Olive');
  });

  test('a DefaultListModel-backed JList adds and removes items live', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'To-do list (DefaultListModel)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const list = root.getByRole('listbox');
    await expect(list.getByRole('option')).toHaveCount(2);

    // Adding to the model re-renders the list with the new item.
    await root.getByRole('textbox').fill('Water plants');
    await root.getByRole('button', { name: 'Add' }).click();
    await expect(list.getByRole('option')).toHaveCount(3);
    await expect(list.getByRole('option', { name: 'Water plants' })).toBeVisible();

    // Removing the selected item shrinks the model-backed list.
    await list.selectOption({ label: 'Buy milk' });
    await root.getByRole('button', { name: 'Remove selected' }).click();
    await expect(list.getByRole('option')).toHaveCount(2);
    await expect(list.getByRole('option', { name: 'Buy milk' })).toHaveCount(0);
  });

  test('a JTable shows a grid, selects a row, and reads/writes cells', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Table (JTable)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const table = root.getByRole('grid');
    await expect(table).toBeVisible();

    // Column headers and one row per data entry (plus the header row).
    await expect(table.getByRole('columnheader', { name: 'Player' })).toBeVisible();
    await expect(table.getByRole('row')).toHaveCount(4);

    // This model overrides isCellEditable, so the cells are read-only.
    await expect(table.locator('td.swing-cell-editable')).toHaveCount(0);

    // Selecting a row fires the selection listener.
    await table.getByRole('row', { name: /Bo/ }).click();
    await expect(root).toContainText('Bo has 7 points');

    // The Award button reads and writes the selected cell (getValueAt/setValueAt).
    await root.getByRole('button', { name: 'Award point' }).click();
    await expect(root).toContainText('Bo now has 8 points');
    await expect(table.getByRole('row', { name: /Bo/ })).toContainText('8');
  });

  test('a DefaultTableModel-backed JTable adds and removes rows live', async ({ page }) => {
    await page.goto('/');
    await page
      .getByTestId('swing-level')
      .selectOption({ label: 'Editable table (DefaultTableModel)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const table = root.getByRole('grid');
    await expect(table.getByRole('row')).toHaveCount(3); // header + 2 rows

    // Append a row through the form; the model update re-renders the grid.
    await root.getByRole('textbox', { name: 'Item:' }).fill('Cherries');
    await root.getByRole('textbox', { name: 'Qty:' }).fill('5');
    await root.getByRole('button', { name: 'Add' }).click();
    await expect(table.getByRole('row')).toHaveCount(4);
    await expect(table.getByRole('row', { name: /Cherries/ })).toContainText('5');

    // Select a row and remove it.
    await table.getByRole('row', { name: /Apples/ }).click();
    await root.getByRole('button', { name: 'Remove selected' }).click();
    await expect(table.getByRole('row')).toHaveCount(3);
    await expect(table.getByRole('row', { name: /Apples/ })).toHaveCount(0);
  });

  test('a JSpinner drives a JProgressBar (spinbutton + progressbar roles)', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Progress + spinner' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const bar = root.getByRole('progressbar');
    const spin = root.getByRole('spinbutton', { name: 'Level (0-100):' });
    await expect(bar).toHaveAttribute('aria-valuenow', '40');
    await expect(bar).toContainText('40%');
    await expect(spin).toHaveValue('40');

    // Changing the spinner fires its ChangeListener, which updates the bar.
    await spin.fill('75');
    await spin.blur();
    await expect(bar).toHaveAttribute('aria-valuenow', '75');
    await expect(bar).toContainText('75%');
    await expect(root).toContainText('Level is 75.');
  });

  test('a BoxLayout stacks components vertically; a button still dispatches', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Box layout' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const one = root.getByRole('button', { name: 'One' });
    const two = root.getByRole('button', { name: 'Two' });
    const three = root.getByRole('button', { name: 'Three' });
    await expect(one).toBeVisible();

    const box = async (loc: Locator): Promise<{ x: number; y: number; bottom: number }> => {
      const b = await loc.boundingBox();
      if (!b) throw new Error('element has no bounding box');
      return { x: b.x, y: b.y, bottom: b.y + b.height };
    };
    const b1 = await box(one);
    const b2 = await box(two);
    const b3 = await box(three);

    // Y_AXIS: stacked top to bottom, left-aligned in a column (same x).
    expect(b2.y).toBeGreaterThan(b1.y);
    expect(b3.y).toBeGreaterThan(b2.y);
    expect(Math.abs(b2.x - b1.x)).toBeLessThan(2);
    // The 8px strut leaves a gap between consecutive buttons.
    expect(b2.y - b1.bottom).toBeGreaterThanOrEqual(6);

    await two.click();
    await expect(root).toContainText('Picked Two');
  });

  test('a JTable edits a cell inline and the value reaches the model', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Edit cells (JTable)' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const grid = root.getByRole('grid');
    await expect(grid).toBeVisible();

    // Baseline total: 3 + 1 + 2 = 6.
    await root.getByRole('button', { name: 'Total qty' }).click();
    await expect(root).toContainText('Total quantity: 6');

    // Double-click the Apples qty cell, change 3 -> 10, commit with Enter.
    await grid.getByRole('gridcell', { name: '3', exact: true }).dblclick();
    const editor = root.locator('.swing-cell-editor');
    await editor.fill('10');
    await editor.press('Enter');
    await expect(grid.getByRole('gridcell', { name: '10', exact: true })).toBeVisible();

    // The edit reached the model: total is now 10 + 1 + 2 = 13.
    await root.getByRole('button', { name: 'Total qty' }).click();
    await expect(root).toContainText('Total quantity: 13');
  });

  test('the window close button ends an interactive run cleanly', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Click counter' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    await expect(root.getByRole('button', { name: 'Click me' })).toBeVisible();
    // A live UI keeps the run going: Run is disabled, Stop is offered.
    await expect(page.getByTestId('run')).toBeDisabled();
    await expect(page.getByTestId('stop-run')).toBeVisible();

    // The accessible close button (the demo sets EXIT_ON_CLOSE) ends the
    // program on its own — the run completes without terminating the worker.
    const close = root.getByRole('button', { name: 'Close window' });
    await expect(close).toBeVisible();
    await close.click();
    await expect(page.getByTestId('run')).toBeEnabled();
    await expect(page.getByTestId('stop-run')).toBeHidden();
  });

  test('a listener reads the current value typed into a text field', async ({ page }) => {
    await page.goto('/');
    await page.getByTestId('swing-level').selectOption({ label: 'Greeter form' });
    await page.getByTestId('run').click();
    const root = page.getByTestId('swing-root');
    const field = root.getByRole('textbox', { name: 'Name:' });
    await expect(field).toBeVisible();

    // The field value is sent with the click, synced into the JTextField,
    // and read by the listener via getText().
    await field.fill('Ada');
    await root.getByRole('button', { name: 'Greet' }).click();
    await expect(root).toContainText('Hello, Ada!');

    // Typing a new value and clicking again reflects the latest input.
    await field.fill('Grace');
    await root.getByRole('button', { name: 'Greet' }).click();
    await expect(root).toContainText('Hello, Grace!');
  });
});

test.describe('swing (debugger)', () => {
  test('a breakpoint inside an event listener pauses when the button is clicked', async ({
    page,
  }) => {
    await page.goto('/');
    // Explicit source so the listener body lines are known (10 and 11).
    await setSource(
      page,
      [
        'import javax.swing.*;', // 1
        'import java.awt.*;', // 2
        '', // 3
        'public class Main {', // 4
        '  static int count = 0;', // 5
        '  public static void main(String[] args) {', // 6
        '    JFrame frame = new JFrame("Counter");', // 7
        '    JButton button = new JButton("Bump");', // 8
        '    button.addActionListener(e -> {', // 9
        '      Main.count++;', // 10
        '      System.out.println("count is " + Main.count);', // 11
        '    });', // 12
        '    frame.add(button);', // 13
        '    frame.setVisible(true);', // 14
        '  }', // 15
        '}', // 16
      ].join('\n'),
    );
    // Breakpoint on the first statement inside the listener body.
    await toggleBreakpoint(page, 10);
    await page.getByTestId('debug').click();

    // The UI renders live under the debugger; clicking the button runs the
    // listener in the VM, which hits the breakpoint and pauses.
    const button = page.getByTestId('swing-root').getByRole('button', { name: 'Bump' });
    await expect(button).toBeVisible();
    await button.click();

    // Paused inside the listener, at the right source line, with the loop's
    // ActionEvent local in scope.
    const frames = page.getByTestId('frames');
    await expect(frames).toContainText('Main.java:10');
    await expect(page.locator('.cm-paused-line')).toContainText('Main.count++');

    // A single resume runs the rest of the listener body (the println fires)
    // — it does not spuriously re-pause on the same source line.
    await page.getByTestId('resume').click();
    await expect(page.getByTestId('console')).toContainText('count is 1');
    await expect(page.getByTestId('paused-view')).toBeHidden();

    // The interactive debug session is still live; Stop aborts it and the
    // controls reset (the escape hatch when idling for the next event).
    await expect(page.getByTestId('stop-run')).toBeVisible();
    await page.getByTestId('stop-run').click();
    await expect(page.getByTestId('run')).toBeEnabled();
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
