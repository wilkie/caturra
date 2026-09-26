// `java.lang.Thread`, phase 0 of specs/CONCURRENCY.md: the thread a program
// is already running on (`main`), and `Thread` as a type and a value. What it
// does NOT do yet is run a second thread — `start` and `join` are refused by
// name until the scheduler (phase 1) exists; `run()` is ordinary Java and
// runs the target on the calling thread, exactly as it does on a JDK.
//
// Every value and message here was captured from a JDK 11 first.

// What `getState()` answers. A JDK nests it (`Thread.State`); a nested class
// is hoisted under its simple name here, and programs have classes called
// `State`, so it lives at the top level under a reserved name and carries
// the JDK's binary name — `Thread.State` resolves to it
// (`NESTED_LIBRARY_CLASSES`).
enum __ThreadState { NEW, RUNNABLE, BLOCKED, WAITING, TIMED_WAITING, TERMINATED }

class Thread implements Runnable {
  // A JDK's own threads take ids 2 to 22 before `main` runs a line (measured
  // on JDK 11), so the first thread a program makes is 23.
  private static long __nextId = 23;
  // `Thread-N` counts only the threads that were NOT given a name.
  private static int __nextNumber = 0;
  private static Thread __main;

  private final long __id;
  private final boolean __isMain;
  private String __name;
  private Runnable __target;
  private int __priority = 5;
  private boolean __daemon;
  private boolean __interrupted;

  // `main` itself: id 1, and never counted.
  private Thread(boolean isMain) {
    __id = 1;
    __isMain = isMain;
    __name = "main";
  }

  public Thread() {
    this(null, "Thread-" + __nextNumber++);
  }

  public Thread(Runnable target) {
    this(target, "Thread-" + __nextNumber++);
  }

  public Thread(String name) {
    this(null, name);
  }

  public Thread(Runnable target, String name) {
    if (name == null) throw new NullPointerException("name cannot be null");
    __target = target;
    __name = name;
    __id = __nextId++;
    __isMain = false;
  }

  public static Thread currentThread() {
    if (__main == null) __main = new Thread(true);
    return __main;
  }

  // Calling `run()` directly is ordinary Java: the target runs HERE, on the
  // calling thread, and `currentThread()` inside it is still `main`.
  public void run() {
    if (__target != null) __target.run();
  }

  public String getName() { return __name; }

  public void setName(String name) {
    if (name == null) throw new NullPointerException("name cannot be null");
    __name = name;
  }

  public long getId() { return __id; }

  public int getPriority() { return __priority; }

  // 1 to 10; a JDK's refusal carries no message.
  public void setPriority(int priority) {
    if (priority > 10 || priority < 1) throw new IllegalArgumentException();
    __priority = priority;
  }

  public boolean isDaemon() { return __daemon; }

  public void setDaemon(boolean on) {
    if (isAlive()) throw new IllegalThreadStateException();
    __daemon = on;
  }

  // Only `main` has been started: every other thread is still NEW.
  public boolean isAlive() { return __isMain; }

  public __ThreadState getState() {
    return __isMain ? __ThreadState.RUNNABLE : __ThreadState.NEW;
  }

  public void interrupt() { __interrupted = true; }

  public boolean isInterrupted() { return __interrupted; }

  // Answers AND CLEARS the current thread's flag.
  public static boolean interrupted() {
    Thread current = currentThread();
    boolean was = current.__interrupted;
    current.__interrupted = false;
    return was;
  }

  // A real wait: the host parks (the browser worker) or moves its virtual
  // clock (tests). An interrupt that is already pending ends the sleep before
  // it starts, and is consumed by doing so — the JDK's order.
  public static void sleep(long millis) throws InterruptedException {
    if (millis < 0) throw new IllegalArgumentException("timeout value is negative");
    Thread current = currentThread();
    if (current.__interrupted) {
      current.__interrupted = false;
      throw new InterruptedException("sleep interrupted");
    }
    __System.__sleep(millis);
  }

  // JDK 11 rounds the nanoseconds to a whole millisecond: up at half a
  // millisecond or more, and up from zero for any nanos at all.
  public static void sleep(long millis, int nanos) throws InterruptedException {
    if (millis < 0) throw new IllegalArgumentException("timeout value is negative");
    if (nanos < 0 || nanos > 999999) {
      throw new IllegalArgumentException("nanosecond timeout value out of range");
    }
    if (nanos >= 500000 || (nanos != 0 && millis == 0)) millis++;
    sleep(millis);
  }

  // On one thread there is nothing to yield to, and a JDK promises nothing
  // about what a yield does in any case.
  public static void yield() {}

  public static void onSpinWait() {}

  public String toString() {
    return "Thread[" + __name + "," + __priority + ",main]";
  }
}
