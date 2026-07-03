/**
 * Blocking stdin over a SharedArrayBuffer (see `specs/EXECUTION.md`).
 *
 * The engine runs synchronously in a worker; when the program reads a
 * line, the worker parks on `Atomics.wait` while the main thread fills
 * the buffer and notifies. Layout:
 *
 * ```
 * Int32[0]  state: 0 = waiting for the host, 1 = line ready, 2 = EOF
 * Int32[1]  payload length in bytes
 * bytes 8.. UTF-8 payload
 * ```
 */

const STATE_INDEX = 0;
const LENGTH_INDEX = 1;
const HEADER_BYTES = 8;

const STATE_WAITING = 0;
const STATE_READY = 1;
const STATE_EOF = 2;

/** Default line capacity: generous for console input. */
export const DEFAULT_STDIN_CAPACITY = 64 * 1024;

/** Allocate the shared stdin channel (main thread). */
export function createStdinBuffer(
  capacityBytes: number = DEFAULT_STDIN_CAPACITY,
): SharedArrayBuffer {
  return new SharedArrayBuffer(HEADER_BYTES + capacityBytes);
}

/**
 * Main thread: publish one line (or EOF for `null`) and wake the
 * worker. Lines longer than the buffer capacity are truncated at a
 * UTF-8 boundary.
 */
export function supplyLine(buffer: SharedArrayBuffer, line: string | null): void {
  const state = new Int32Array(buffer, 0, 2);
  if (line === null) {
    Atomics.store(state, STATE_INDEX, STATE_EOF);
  } else {
    // TextEncoder refuses SharedArrayBuffer-backed views, so encode
    // into regular memory and copy across (truncating to capacity).
    const capacity = buffer.byteLength - HEADER_BYTES;
    let encoded = new TextEncoder().encode(line);
    if (encoded.length > capacity) {
      encoded = encoded.subarray(0, capacity);
    }
    new Uint8Array(buffer, HEADER_BYTES, encoded.length).set(encoded);
    Atomics.store(state, LENGTH_INDEX, encoded.length);
    Atomics.store(state, STATE_INDEX, STATE_READY);
  }
  Atomics.notify(state, STATE_INDEX);
}

/**
 * Worker thread: block until the main thread supplies a line. Call
 * `requestLine` after arming the channel — it should post the
 * stdin-request message that prompts the main thread to respond.
 * Returns `null` on EOF.
 */
export function readLineBlocking(
  buffer: SharedArrayBuffer,
  requestLine: () => void,
): string | null {
  const state = new Int32Array(buffer, 0, 2);
  Atomics.store(state, STATE_INDEX, STATE_WAITING);
  requestLine();
  // If the main thread already responded, wait() sees a non-WAITING
  // value and returns immediately ('not-equal') — no missed wakeups.
  Atomics.wait(state, STATE_INDEX, STATE_WAITING);
  if (Atomics.load(state, STATE_INDEX) === STATE_EOF) {
    return null;
  }
  const length = Atomics.load(state, LENGTH_INDEX);
  // Copy out of shared memory before decoding: TextDecoder rejects
  // SharedArrayBuffer-backed views.
  const bytes = new Uint8Array(length);
  bytes.set(new Uint8Array(buffer, HEADER_BYTES, length));
  return new TextDecoder().decode(bytes);
}

// ----- Interrupt flag (debugger pause button) -----
//
// A single shared Int32 the main thread sets and the engine polls
// between instructions. Consuming resets it, so one click = one pause.

/** Allocate the shared interrupt flag (main thread). */
export function createInterruptFlag(): SharedArrayBuffer {
  return new SharedArrayBuffer(4);
}

/** Main thread: request a pause at the next opportunity. */
export function requestInterrupt(flag: SharedArrayBuffer): void {
  Atomics.store(new Int32Array(flag, 0, 1), 0, 1);
}

/** Worker: check-and-clear the pause request. */
export function consumeInterrupt(flag: SharedArrayBuffer): boolean {
  return Atomics.compareExchange(new Int32Array(flag, 0, 1), 0, 1, 0) === 1;
}
