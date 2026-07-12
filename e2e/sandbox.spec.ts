/**
 * Cross-origin sandbox mode (see specs/EXECUTION.md). The editor is
 * served on http://localhost:4173 and the engine runs inside a hidden
 * iframe on the *different-origin* http://127.0.0.1:4174 sandbox, driven
 * over a MessagePort. These tests prove the full path works in a real
 * browser — compile/run and blocking stdin through the iframe — and that
 * the sandbox is genuinely isolated from the editor origin's storage.
 *
 * The `?sandbox=` query parameter (honored only for loopback origins)
 * flips the same playground build into iframe mode; the rest of the e2e
 * suite runs the default same-origin worker path.
 */
import { expect, test } from '@playwright/test';
import type { Frame, Page } from '@playwright/test';

const EDITOR = 'http://localhost:4173';
const SANDBOX = 'http://127.0.0.1:4174';
const EDITOR_WITH_SANDBOX = `${EDITOR}/?sandbox=${SANDBOX}`;

interface PlaygroundHooks {
  setSource: (text: string) => void;
}

async function setSource(page: Page, text: string): Promise<void> {
  await page.waitForFunction(
    () => (window as unknown as { playground?: PlaygroundHooks }).playground !== undefined,
  );
  await page.evaluate((t) => {
    (window as unknown as { playground: PlaygroundHooks }).playground.setSource(t);
  }, text);
}

/** The hidden engine iframe, once the handshake has navigated it to the sandbox origin. */
function sandboxFrame(page: Page): Frame {
  const frame = page.frames().find((candidate) => candidate.url().startsWith(SANDBOX));
  if (!frame) {
    throw new Error('sandbox iframe not found');
  }
  return frame;
}

test.describe('cross-origin sandbox mode', () => {
  test('boots the engine inside a cross-origin iframe', async ({ page }) => {
    await page.goto(EDITOR_WITH_SANDBOX);
    // The version only appears once the iframe loaded, the port handshake
    // completed, and the engine answered across the origin boundary.
    await expect(page.getByTestId('engine-version')).toHaveText(/^engine v\d+\.\d+\.\d+$/);

    const frame = sandboxFrame(page);
    expect(new URL(frame.url()).origin).toBe(SANDBOX);
    // The editor origin is not the sandbox origin.
    expect(new URL(page.url()).origin).not.toBe(SANDBOX);
  });

  test('compiles and runs Hello World through the iframe', async ({ page }) => {
    await page.goto(EDITOR_WITH_SANDBOX);
    await expect(page.getByTestId('run')).toBeEnabled();
    await page.getByTestId('run').click();
    const consoleOutput = page.getByTestId('console');
    await expect(consoleOutput).toContainText('$ javac Main.java');
    await expect(consoleOutput).toContainText('Hello, World!');

    // Exactly one engine iframe — the session is opened once and reused, not
    // re-created on every React render (which would pile up orphaned engines).
    const sandboxFrames = page.frames().filter((candidate) => candidate.url().startsWith(SANDBOX));
    expect(sandboxFrames).toHaveLength(1);
    expect(await page.locator('iframe').count()).toBe(1);
  });

  test('blocking stdin round-trips across the origin boundary', async ({ page }) => {
    await page.goto(EDITOR_WITH_SANDBOX);
    await expect(page.getByTestId('engine-version')).toHaveText(/^engine v\d+\.\d+\.\d+$/);
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
    // The engine parked on its SharedArrayBuffer inside the sandbox, asked
    // the editor for a line over the port, and resumed — all cross-origin.
    await expect(page.getByTestId('console')).toContainText('Hello Ada, age 37 next year!');
  });

  test('the sandbox is cross-origin isolated but cannot see editor storage', async ({ page }) => {
    await page.goto(EDITOR_WITH_SANDBOX);
    await expect(page.getByTestId('engine-version')).toHaveText(/^engine v\d+\.\d+\.\d+$/);

    // A secret in the editor origin's storage.
    await page.evaluate(() => {
      localStorage.setItem('caturra-secret', 'do-not-leak');
      document.cookie = 'caturra_session=do-not-leak';
    });

    const frame = sandboxFrame(page);
    const view = await frame.evaluate(() => ({
      origin: location.origin,
      isolated: globalThis.crossOriginIsolated,
      hasSab: typeof SharedArrayBuffer !== 'undefined',
      leakedLocalStorage: localStorage.getItem('caturra-secret'),
      cookies: document.cookie,
    }));

    expect(view.origin).toBe(SANDBOX);
    // Isolated (so its worker keeps SharedArrayBuffer for blocking stdin)...
    expect(view.isolated).toBe(true);
    expect(view.hasSab).toBe(true);
    // ...yet the editor origin's session data is invisible here.
    expect(view.leakedLocalStorage).toBeNull();
    expect(view.cookies).not.toContain('do-not-leak');
  });
});
