// `java.lang.Thread` (specs/CONCURRENCY.md, phases 0 and 1). The object is
// ordinary Java; what a thread DOES — run, park, wake — is the interpreter's
// scheduler, reached through the reserved `__System.__…` calls. A thread's
// run state lives there, not in a field here, because other threads change it
// (a `join` ends when the SCHEDULER sees the target finish).
//
// Every value and message here was captured from a JDK 11 first.

// What `getState()` answers. A JDK nests it (`Thread.State`); a nested class
// is hoisted under its simple name here, and programs have classes called
// `State`, so it lives at the top level under a reserved name and carries
// the JDK's binary name — `Thread.State` resolves to it
// (`NESTED_LIBRARY_CLASSES`). The scheduler answers with an ORDINAL.
enum __ThreadState { NEW, RUNNABLE, BLOCKED, WAITING, TIMED_WAITING, TERMINATED }

class Thread implements Runnable {
  // A JDK's own threads take ids 2 to 22 before `main` runs a line (measured
  // on JDK 11), so the first thread a program makes is 23.
  private static long __nextId = 23;
  // `Thread-N` counts only the threads that were NOT given a name.
  private static int __nextNumber = 0;

  private final long __id;
  private String __name;
  private Runnable __target;
  private int __priority = 5;
  private boolean __daemon;
  private boolean __started;

  // `main` itself: id 1, never counted, and made the first time a program
  // asks for it — the scheduler holds on to it from then on.
  private Thread(boolean isMain) {
    __id = 1;
    __name = "main";
    __started = true;
    __System.__setMain(this);
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
  }

  public static Thread currentThread() {
    Thread current = (Thread) __System.__currentThread();
    return current != null ? current : new Thread(true);
  }

  // The new thread runs when the scheduler next picks it — never inside
  // `start` itself, which returns to the caller at once, as a JDK's does.
  public void start() {
    if (__started) throw new IllegalThreadStateException();
    __started = true;
    __System.__start(this, __daemon);
  }

  // Calling `run()` directly is ordinary Java: the target runs HERE, on the
  // calling thread, and `currentThread()` inside it is the caller.
  public void run() {
    if (__target != null) __target.run();
  }

  // A started thread's first frame. What escapes `run` is the default
  // uncaught handler's to report, on standard error, and then the thread is
  // over; the others carry on.
  private void __entry() {
    try {
      run();
    } catch (Throwable thrown) {
      System.err.print("Exception in thread \"" + __name + "\" ");
      thrown.printStackTrace();
    }
  }

  public final void join() throws InterruptedException {
    join(0);
  }

  // Zero waits for as long as it takes. A thread that is not alive — never
  // started, or finished — is joined at once, even with an interrupt
  // pending (the JDK's loop asks `isAlive()` first).
  public final void join(long millis) throws InterruptedException {
    if (millis < 0) throw new IllegalArgumentException("timeout value is negative");
    if (!isAlive()) return;
    if (__System.__join(this, millis)) throw new InterruptedException();
  }

  public final void join(long millis, int nanos) throws InterruptedException {
    if (millis < 0) throw new IllegalArgumentException("timeout value is negative");
    if (nanos < 0 || nanos > 999999) {
      throw new IllegalArgumentException("nanosecond timeout value out of range");
    }
    if (nanos >= 500000 || (nanos != 0 && millis == 0)) millis++;
    join(millis);
  }

  public String getName() { return __name; }

  public void setName(String name) {
    if (name == null) throw new NullPointerException("name cannot be null");
    __name = name;
  }

  public long getId() { return __id; }

  public int getPriority() { return __priority; }

  // 1 to 10; a JDK's refusal carries no message. The scheduler ignores it,
  // as a JDK is free to.
  public void setPriority(int priority) {
    if (priority > 10 || priority < 1) throw new IllegalArgumentException();
    __priority = priority;
  }

  public boolean isDaemon() { return __daemon; }

  public void setDaemon(boolean on) {
    if (isAlive()) throw new IllegalThreadStateException();
    __daemon = on;
  }

  public boolean isAlive() {
    int status = __System.__threadStatus(this);
    return status != 0 && status != 5;
  }

  public __ThreadState getState() {
    return __ThreadState.values()[__System.__threadStatus(this)];
  }

  // Wakes a sleep, join or wait with InterruptedException; otherwise sets
  // the flag. A thread that is not alive keeps no flag (measured).
  public void interrupt() { __System.__interrupt(this); }

  public boolean isInterrupted() { return __System.__isInterrupted(this, false); }

  // Answers AND CLEARS the current thread's flag.
  public static boolean interrupted() {
    return __System.__isInterrupted(currentThread(), true);
  }

  public static boolean holdsLock(Object lock) {
    if (lock == null) throw new NullPointerException();
    return __System.__holdsLock(lock);
  }

  // Parks this thread and lets the others run. An interrupt that is already
  // pending ends the sleep before it starts, and is consumed by doing so —
  // the JDK's order.
  public static void sleep(long millis) throws InterruptedException {
    if (millis < 0) throw new IllegalArgumentException("timeout value is negative");
    if (__System.__sleep(millis)) throw new InterruptedException("sleep interrupted");
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

  // Ends the running thread's turn.
  public static void yield() { __System.__yield(); }

  public static void onSpinWait() {}

  // A finished thread has left its group, and says so with an empty name.
  public String toString() {
    String group = getState() == __ThreadState.TERMINATED ? "" : "main";
    return "Thread[" + __name + "," + __priority + "," + group + "]";
  }
}
