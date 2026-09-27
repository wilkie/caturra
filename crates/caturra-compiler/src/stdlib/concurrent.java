// `java.util.concurrent`, `.atomic` and `.locks` — phase 2 of
// specs/CONCURRENCY.md. Written as bundled Java over what phase 1 built
// (threads, `synchronized`, `wait`/`notify`), which is how a JDK writes them
// too; nothing here is an intrinsic. Every class that stands for a JDK one
// carries its binary name (lib.rs, `jdk_binary_name`), and every value and
// message was captured from a JDK 11 first.

// ---------------------------------------------------------------- the types

interface Callable<V> {
  V call() throws Exception;
}

interface Executor {
  void execute(Runnable command);
}

interface ThreadFactory {
  Thread newThread(Runnable r);
}

interface Future<V> {
  boolean cancel(boolean mayInterruptIfRunning);

  boolean isCancelled();

  boolean isDone();

  V get() throws InterruptedException, ExecutionException;

  V get(long timeout, TimeUnit unit)
      throws InterruptedException, ExecutionException, TimeoutException;
}

interface RunnableFuture<V> extends Runnable, Future<V> {}

// `invokeAll`/`invokeAny` take `Collection<Callable<T>>` where a JDK writes
// `Collection<? extends Callable<T>>`: a wildcard keeps only its bound's name
// here, and then nothing could say what `T` is. A collection of a program's
// own Callable subclass is the one argument this narrows away.
interface ExecutorService extends Executor {
  void shutdown();

  List<Runnable> shutdownNow();

  boolean isShutdown();

  boolean isTerminated();

  boolean awaitTermination(long timeout, TimeUnit unit) throws InterruptedException;

  <T> Future<T> submit(Callable<T> task);

  <T> Future<T> submit(Runnable task, T result);

  Future<?> submit(Runnable task);

  <T> List<Future<T>> invokeAll(Collection<Callable<T>> tasks)
      throws InterruptedException;

  <T> T invokeAny(Collection<Callable<T>> tasks)
      throws InterruptedException, ExecutionException;
}

// ---------------------------------------------------------------- TimeUnit

enum TimeUnit {
  NANOSECONDS(1L),
  MICROSECONDS(1000L),
  MILLISECONDS(1000000L),
  SECONDS(1000000000L),
  MINUTES(60000000000L),
  HOURS(3600000000000L),
  DAYS(86400000000000L);

  private final long __scale;

  TimeUnit(long scale) {
    __scale = scale;
  }

  // `d` in a unit of `from` nanoseconds, as a unit of `to`: dividing
  // truncates, and multiplying saturates at the long range — the JDK's rule.
  private static long __cvt(long d, long to, long from) {
    if (from == to) return d;
    if (from < to) return d / (to / from);
    long ratio = from / to;
    long max = Long.MAX_VALUE / ratio;
    if (d > max) return Long.MAX_VALUE;
    if (d < -max) return Long.MIN_VALUE;
    return d * ratio;
  }

  public long convert(long sourceDuration, TimeUnit sourceUnit) {
    return __cvt(sourceDuration, __scale, sourceUnit.__scale);
  }

  public long toNanos(long duration) { return __cvt(duration, 1L, __scale); }

  public long toMicros(long duration) { return __cvt(duration, 1000L, __scale); }

  public long toMillis(long duration) { return __cvt(duration, 1000000L, __scale); }

  public long toSeconds(long duration) { return __cvt(duration, 1000000000L, __scale); }

  public long toMinutes(long duration) { return __cvt(duration, 60000000000L, __scale); }

  public long toHours(long duration) { return __cvt(duration, 3600000000000L, __scale); }

  public long toDays(long duration) { return __cvt(duration, 86400000000000L, __scale); }

  private int __excessNanos(long duration, long millis) {
    long nanos = toNanos(duration) - millis * 1000000L;
    return (int) Math.max(0L, Math.min(nanos, 999999L));
  }

  public ChronoUnit toChronoUnit() {
    switch (this) {
      case NANOSECONDS: return ChronoUnit.NANOS;
      case MICROSECONDS: return ChronoUnit.MICROS;
      case MILLISECONDS: return ChronoUnit.MILLIS;
      case SECONDS: return ChronoUnit.SECONDS;
      case MINUTES: return ChronoUnit.MINUTES;
      case HOURS: return ChronoUnit.HOURS;
      default: return ChronoUnit.DAYS;
    }
  }

  public static TimeUnit of(ChronoUnit chronoUnit) {
    if (chronoUnit == null) throw new NullPointerException("chronoUnit");
    for (TimeUnit unit : values()) {
      if (unit.toChronoUnit() == chronoUnit) return unit;
    }
    throw new IllegalArgumentException("No TimeUnit equivalent for " + chronoUnit);
  }

  public void sleep(long timeout) throws InterruptedException {
    if (timeout > 0) {
      long ms = toMillis(timeout);
      Thread.sleep(ms, __excessNanos(timeout, ms));
    }
  }

  public void timedJoin(Thread thread, long timeout) throws InterruptedException {
    if (timeout > 0) {
      long ms = toMillis(timeout);
      thread.join(ms, __excessNanos(timeout, ms));
    }
  }

  public void timedWait(Object obj, long timeout) throws InterruptedException {
    if (timeout > 0) {
      long ms = toMillis(timeout);
      obj.wait(ms, __excessNanos(timeout, ms));
    }
  }
}

// ---------------------------------------------------------------- futures

class FutureTask<V> implements RunnableFuture<V> {
  // NEW, then one of NORMAL, EXCEPTIONAL, CANCELLED, INTERRUPTED.
  private static final int __NEW = 0;
  private static final int __NORMAL = 1;
  private static final int __EXCEPTIONAL = 2;
  private static final int __CANCELLED = 3;
  private static final int __INTERRUPTED = 4;

  private Callable<V> __callable;
  private int __state;
  private Object __outcome;
  private Thread __runner;

  public FutureTask(Callable<V> callable) {
    if (callable == null) throw new NullPointerException();
    __callable = callable;
  }

  public FutureTask(Runnable runnable, V result) {
    this(Executors.callable(runnable, result));
  }

  public void run() {
    synchronized (this) {
      if (__state != __NEW || __runner != null) return;
      __runner = Thread.currentThread();
    }
    V result = null;
    Throwable failure = null;
    boolean ran = false;
    try {
      result = __callable.call();
      ran = true;
    } catch (Throwable thrown) {
      failure = thrown;
    }
    boolean finished = false;
    synchronized (this) {
      __runner = null;
      if (__state == __NEW) {
        if (ran) {
          __state = __NORMAL;
          __outcome = result;
        } else {
          __state = __EXCEPTIONAL;
          __outcome = failure;
        }
        __callable = null;
        finished = true;
      }
      notifyAll();
    }
    if (finished) done();
  }

  // Called once the task is over, however it ended; a subclass's hook.
  protected void done() {}

  public boolean cancel(boolean mayInterruptIfRunning) {
    synchronized (this) {
      if (__state != __NEW) return false;
      __state = mayInterruptIfRunning ? __INTERRUPTED : __CANCELLED;
      if (mayInterruptIfRunning && __runner != null) __runner.interrupt();
      __callable = null;
      notifyAll();
    }
    done();
    return true;
  }

  public synchronized boolean isCancelled() { return __state >= __CANCELLED; }

  public synchronized boolean isDone() { return __state != __NEW; }

  public synchronized V get() throws InterruptedException, ExecutionException {
    while (__state == __NEW) wait();
    return __report();
  }

  public synchronized V get(long timeout, TimeUnit unit)
      throws InterruptedException, ExecutionException, TimeoutException {
    if (unit == null) throw new NullPointerException();
    long end = System.currentTimeMillis() + unit.toMillis(timeout);
    while (__state == __NEW) {
      long left = end - System.currentTimeMillis();
      if (left <= 0) throw new TimeoutException();
      wait(left);
    }
    return __report();
  }

  @SuppressWarnings("unchecked")
  private V __report() throws ExecutionException {
    if (__state == __NORMAL) return (V) __outcome;
    if (__state >= __CANCELLED) throw new CancellationException();
    throw new ExecutionException((Throwable) __outcome);
  }

  public String toString() {
    String status;
    synchronized (this) {
      if (__state == __NORMAL) {
        status = "[Completed normally]";
      } else if (__state == __EXCEPTIONAL) {
        status = "[Completed exceptionally: " + __outcome + "]";
      } else if (__state >= __CANCELLED) {
        status = "[Cancelled]";
      } else {
        status = __callable == null ? "[Not completed]" : "[Not completed, task = " + __callable + "]";
      }
    }
    return super.toString() + status;
  }
}

// `Executors.callable(task, result)`: a Runnable as a Callable.
class __RunnableAdapter<T> implements Callable<T> {
  private final Runnable __task;
  private final T __result;

  __RunnableAdapter(Runnable task, T result) {
    __task = task;
    __result = result;
  }

  public T call() {
    __task.run();
    return __result;
  }
}

// ---------------------------------------------------------------- executors

class Executors {
  private Executors() {}

  public static ExecutorService newFixedThreadPool(int nThreads) {
    return new ThreadPoolExecutor(nThreads, nThreads, 0L, false, defaultThreadFactory());
  }

  public static ExecutorService newFixedThreadPool(int nThreads, ThreadFactory threadFactory) {
    if (threadFactory == null) throw new NullPointerException();
    return new ThreadPoolExecutor(nThreads, nThreads, 0L, false, threadFactory);
  }

  public static ExecutorService newSingleThreadExecutor() {
    return new __FinalizableDelegatedExecutorService(
        new ThreadPoolExecutor(1, 1, 0L, false, defaultThreadFactory()));
  }

  public static ExecutorService newSingleThreadExecutor(ThreadFactory threadFactory) {
    if (threadFactory == null) throw new NullPointerException();
    return new __FinalizableDelegatedExecutorService(
        new ThreadPoolExecutor(1, 1, 0L, false, threadFactory));
  }

  // Threads made as tasks arrive, and retired after a minute idle.
  public static ExecutorService newCachedThreadPool() {
    return new ThreadPoolExecutor(0, Integer.MAX_VALUE, 60000L, true, defaultThreadFactory());
  }

  public static ExecutorService newCachedThreadPool(ThreadFactory threadFactory) {
    if (threadFactory == null) throw new NullPointerException();
    return new ThreadPoolExecutor(0, Integer.MAX_VALUE, 60000L, true, threadFactory);
  }

  public static ExecutorService unconfigurableExecutorService(ExecutorService executor) {
    if (executor == null) throw new NullPointerException();
    return new __DelegatedExecutorService(executor);
  }

  public static ThreadFactory defaultThreadFactory() {
    return new __DefaultThreadFactory();
  }

  public static <T> Callable<T> callable(Runnable task, T result) {
    if (task == null) throw new NullPointerException();
    return new __RunnableAdapter<T>(task, result);
  }

  public static Callable<Object> callable(Runnable task) {
    if (task == null) throw new NullPointerException();
    return new __RunnableAdapter<Object>(task, null);
  }
}

// `pool-N-thread-M`: N counts the factories made, M the threads each makes.
class __DefaultThreadFactory implements ThreadFactory {
  private static int __poolNumber = 1;
  private final String __prefix;
  private int __threadNumber = 1;

  __DefaultThreadFactory() {
    __prefix = "pool-" + __poolNumber++ + "-thread-";
  }

  public Thread newThread(Runnable r) {
    Thread t = new Thread(r, __prefix + __threadNumber++);
    if (t.isDaemon()) t.setDaemon(false);
    if (t.getPriority() != 5) t.setPriority(5);
    return t;
  }
}

// One pool thread: its first task, then whatever the queue hands it.
class __Worker implements Runnable {
  final ThreadPoolExecutor __pool;
  Runnable __first;
  Thread __thread;

  __Worker(ThreadPoolExecutor pool, Runnable first) {
    __pool = pool;
    __first = first;
  }

  public void run() {
    __pool.__runWorker(this);
  }
}

class ThreadPoolExecutor implements ExecutorService {
  private static final int __RUNNING = 0;
  private static final int __SHUTDOWN = 1;
  private static final int __STOP = 2;
  private static final int __TERMINATED = 3;

  private final int __core;
  private final int __max;
  private final long __keepAliveMillis;
  // A cached pool hands a task to an idle thread or makes one; it queues
  // nothing (the JDK's `SynchronousQueue`).
  private final boolean __handoff;
  private final ThreadFactory __factory;
  private final ArrayDeque<Runnable> __queue = new ArrayDeque<>();
  private final ArrayList<__Worker> __workers = new ArrayList<>();
  private int __state = __RUNNING;
  private int __idle;
  private int __active;
  private long __completed;
  private int __largest;

  ThreadPoolExecutor(int core, int max, long keepAliveMillis, boolean handoff, ThreadFactory factory) {
    if (core < 0 || max <= 0 || max < core || keepAliveMillis < 0) {
      throw new IllegalArgumentException();
    }
    __core = core;
    __max = max;
    __keepAliveMillis = keepAliveMillis;
    __handoff = handoff;
    __factory = factory;
  }

  public void execute(Runnable command) {
    if (command == null) throw new NullPointerException();
    synchronized (this) {
      if (__state == __RUNNING) {
        if (__workers.size() < __core) {
          __addWorker(command);
          return;
        }
        if (!__handoff) {
          __queue.add(command);
          notifyAll();
          return;
        }
        if (__idle > __queue.size()) {
          __queue.add(command);
          notifyAll();
          return;
        }
        if (__workers.size() < __max) {
          __addWorker(command);
          return;
        }
      }
    }
    throw new RejectedExecutionException(
        "Task " + command + " rejected from " + this);
  }

  private void __addWorker(Runnable first) {
    __Worker worker = new __Worker(this, first);
    Thread t = __factory.newThread(worker);
    worker.__thread = t;
    __workers.add(worker);
    if (__workers.size() > __largest) __largest = __workers.size();
    t.start();
  }

  void __runWorker(__Worker worker) {
    Runnable task = worker.__first;
    worker.__first = null;
    boolean abrupt = true;
    try {
      while (task != null || (task = __take()) != null) {
        synchronized (this) {
          __active++;
        }
        try {
          task.run();
        } finally {
          synchronized (this) {
            __active--;
            __completed++;
          }
        }
        task = null;
      }
      abrupt = false;
    } finally {
      __workerExit(worker, abrupt);
    }
  }

  // The next task, or null when this thread should end: the pool is
  // stopping, shut down with nothing left, or this thread was idle past its
  // keep-alive.
  private synchronized Runnable __take() {
    long deadline = 0;
    while (true) {
      if (__state >= __STOP) return null;
      if (!__queue.isEmpty()) return __queue.poll();
      if (__state != __RUNNING) return null;
      boolean timed = __workers.size() > __core;
      long now = System.currentTimeMillis();
      if (timed && deadline == 0) deadline = now + __keepAliveMillis;
      if (timed && now >= deadline) return null;
      __idle++;
      try {
        if (timed) {
          wait(deadline - now);
        } else {
          wait();
        }
      } catch (InterruptedException e) {
        // `shutdownNow` interrupts idle threads; the loop re-reads the state.
      } finally {
        __idle--;
      }
    }
  }

  private void __workerExit(__Worker worker, boolean abrupt) {
    synchronized (this) {
      __workers.remove(worker);
      if (__state < __STOP) {
        int min = __core;
        if (min == 0 && !__queue.isEmpty()) min = 1;
        if (abrupt || __workers.size() < min) {
          if (__state == __RUNNING || !__queue.isEmpty()) __addWorker(null);
        }
      }
      __tryTerminate();
    }
  }

  private void __tryTerminate() {
    if (__state == __RUNNING || __state == __TERMINATED) return;
    if (__workers.isEmpty() && (__state == __STOP || __queue.isEmpty())) {
      __state = __TERMINATED;
      notifyAll();
    }
  }

  public synchronized void shutdown() {
    if (__state == __RUNNING) __state = __SHUTDOWN;
    notifyAll();
    __tryTerminate();
  }

  public synchronized List<Runnable> shutdownNow() {
    if (__state < __STOP) __state = __STOP;
    List<Runnable> drained = new ArrayList<>(__queue);
    __queue.clear();
    for (__Worker worker : __workers) worker.__thread.interrupt();
    notifyAll();
    __tryTerminate();
    return drained;
  }

  public synchronized boolean isShutdown() { return __state != __RUNNING; }

  public synchronized boolean isTerminated() { return __state == __TERMINATED; }

  public synchronized boolean isTerminating() {
    return __state != __RUNNING && __state != __TERMINATED;
  }

  public synchronized boolean awaitTermination(long timeout, TimeUnit unit)
      throws InterruptedException {
    long end = System.currentTimeMillis() + unit.toMillis(timeout);
    while (__state != __TERMINATED) {
      long left = end - System.currentTimeMillis();
      if (left <= 0) return false;
      wait(left);
    }
    return true;
  }

  public <T> Future<T> submit(Callable<T> task) {
    if (task == null) throw new NullPointerException();
    FutureTask<T> future = new FutureTask<T>(task);
    execute(future);
    return future;
  }

  public <T> Future<T> submit(Runnable task, T result) {
    if (task == null) throw new NullPointerException();
    FutureTask<T> future = new FutureTask<T>(task, result);
    execute(future);
    return future;
  }

  public Future<?> submit(Runnable task) {
    if (task == null) throw new NullPointerException();
    FutureTask<Object> future = new FutureTask<Object>(task, null);
    execute(future);
    return future;
  }

  public <T> List<Future<T>> invokeAll(Collection<Callable<T>> tasks)
      throws InterruptedException {
    if (tasks == null) throw new NullPointerException();
    List<Future<T>> futures = new ArrayList<>();
    for (Callable<T> task : tasks) {
      FutureTask<T> future = new FutureTask<T>(task);
      futures.add(future);
      execute(future);
    }
    for (Future<T> future : futures) {
      if (!future.isDone()) {
        try {
          future.get();
        } catch (CancellationException | ExecutionException ignored) {
          // Its outcome is the caller's to read.
        }
      }
    }
    return futures;
  }

  // The first task to complete NORMALLY answers; the rest are cancelled. If
  // none does, the last failure's cause is reported.
  public <T> T invokeAny(Collection<Callable<T>> tasks)
      throws InterruptedException, ExecutionException {
    if (tasks == null) throw new NullPointerException();
    if (tasks.isEmpty()) throw new IllegalArgumentException();
    Object signal = new Object();
    List<FutureTask<T>> futures = new ArrayList<>();
    synchronized (signal) {
      for (Callable<T> task : tasks) {
        FutureTask<T> future = new __SignallingTask<T>(task, signal);
        futures.add(future);
        execute(future);
      }
      while (true) {
        ExecutionException last = null;
        boolean pending = false;
        for (FutureTask<T> future : futures) {
          if (!future.isDone()) {
            pending = true;
            continue;
          }
          try {
            T value = future.get();
            for (FutureTask<T> other : futures) other.cancel(true);
            return value;
          } catch (ExecutionException e) {
            last = e;
          } catch (CancellationException e) {
            // cancelled from outside: not an answer
          }
        }
        if (!pending) {
          if (last != null) throw last;
          throw new ExecutionException((Throwable) null);
        }
        signal.wait();
      }
    }
  }

  public int getPoolSize() { synchronized (this) { return __workers.size(); } }

  public int getActiveCount() { synchronized (this) { return __active; } }

  public int getCorePoolSize() { return __core; }

  public int getMaximumPoolSize() { return __max; }

  public int getLargestPoolSize() { synchronized (this) { return __largest; } }

  public long getCompletedTaskCount() { synchronized (this) { return __completed; } }

  public long getTaskCount() {
    synchronized (this) { return __completed + __active + __queue.size(); }
  }

  public long getKeepAliveTime(TimeUnit unit) {
    return unit.convert(__keepAliveMillis, TimeUnit.MILLISECONDS);
  }

  public ThreadFactory getThreadFactory() { return __factory; }

  public String toString() {
    String run;
    int size;
    int active;
    int queued;
    long completed;
    synchronized (this) {
      run = __state == __RUNNING ? "Running" : __state == __TERMINATED ? "Terminated" : "Shutting down";
      size = __workers.size();
      active = __active;
      queued = __queue.size();
      completed = __completed;
    }
    return super.toString() + "[" + run + ", pool size = " + size + ", active threads = "
        + active + ", queued tasks = " + queued + ", completed tasks = " + completed + "]";
  }
}

// A task that also tells `invokeAny` it is over.
class __SignallingTask<T> extends FutureTask<T> {
  private final Object __signal;

  __SignallingTask(Callable<T> task, Object signal) {
    super(task);
    __signal = signal;
  }

  protected void done() {
    synchronized (__signal) {
      __signal.notifyAll();
    }
  }
}

// `unconfigurableExecutorService` (and `newSingleThreadExecutor`) wrap a pool,
// so that nothing can reconfigure it — and a program sees the wrapper's class.
class __DelegatedExecutorService implements ExecutorService {
  private final ExecutorService __e;

  __DelegatedExecutorService(ExecutorService executor) { __e = executor; }

  public void execute(Runnable command) { __e.execute(command); }

  public void shutdown() { __e.shutdown(); }

  public List<Runnable> shutdownNow() { return __e.shutdownNow(); }

  public boolean isShutdown() { return __e.isShutdown(); }

  public boolean isTerminated() { return __e.isTerminated(); }

  public boolean awaitTermination(long timeout, TimeUnit unit) throws InterruptedException {
    return __e.awaitTermination(timeout, unit);
  }

  public <T> Future<T> submit(Callable<T> task) { return __e.submit(task); }

  public <T> Future<T> submit(Runnable task, T result) { return __e.submit(task, result); }

  public Future<?> submit(Runnable task) { return __e.submit(task); }

  public <T> List<Future<T>> invokeAll(Collection<Callable<T>> tasks)
      throws InterruptedException {
    return __e.invokeAll(tasks);
  }

  public <T> T invokeAny(Collection<Callable<T>> tasks)
      throws InterruptedException, ExecutionException {
    return __e.invokeAny(tasks);
  }
}

// The single-thread executor's wrapper — the same, under the JDK's own
// class name for it.
class __FinalizableDelegatedExecutorService extends __DelegatedExecutorService {
  __FinalizableDelegatedExecutorService(ExecutorService executor) { super(executor); }
}

// ---------------------------------------------------------------- synchronizers

class CountDownLatch {
  private long __count;

  public CountDownLatch(int count) {
    if (count < 0) throw new IllegalArgumentException("count < 0");
    __count = count;
  }

  public synchronized void await() throws InterruptedException {
    if (Thread.interrupted()) throw new InterruptedException();
    while (__count > 0) wait();
  }

  public synchronized boolean await(long timeout, TimeUnit unit) throws InterruptedException {
    if (Thread.interrupted()) throw new InterruptedException();
    long end = System.currentTimeMillis() + unit.toMillis(timeout);
    while (__count > 0) {
      long left = end - System.currentTimeMillis();
      if (left <= 0) return false;
      wait(left);
    }
    return true;
  }

  public synchronized void countDown() {
    if (__count > 0 && --__count == 0) notifyAll();
  }

  public synchronized long getCount() { return __count; }

  public String toString() {
    return super.toString() + "[Count = " + getCount() + "]";
  }
}

class Semaphore {
  private int __permits;
  private final boolean __fair;
  private int __waiting;

  public Semaphore(int permits) { this(permits, false); }

  public Semaphore(int permits, boolean fair) {
    __permits = permits;
    __fair = fair;
  }

  public void acquire() throws InterruptedException { acquire(1); }

  public synchronized void acquire(int permits) throws InterruptedException {
    if (permits < 0) throw new IllegalArgumentException();
    if (Thread.interrupted()) throw new InterruptedException();
    __waiting++;
    try {
      while (__permits < permits) wait();
    } finally {
      __waiting--;
    }
    __permits -= permits;
  }

  public void acquireUninterruptibly() { acquireUninterruptibly(1); }

  public synchronized void acquireUninterruptibly(int permits) {
    if (permits < 0) throw new IllegalArgumentException();
    boolean interrupted = false;
    __waiting++;
    while (__permits < permits) {
      try {
        wait();
      } catch (InterruptedException e) {
        interrupted = true;
      }
    }
    __waiting--;
    __permits -= permits;
    if (interrupted) Thread.currentThread().interrupt();
  }

  public boolean tryAcquire() { return tryAcquire(1); }

  public synchronized boolean tryAcquire(int permits) {
    if (permits < 0) throw new IllegalArgumentException();
    if (__permits < permits) return false;
    __permits -= permits;
    return true;
  }

  public boolean tryAcquire(long timeout, TimeUnit unit) throws InterruptedException {
    return tryAcquire(1, timeout, unit);
  }

  public synchronized boolean tryAcquire(int permits, long timeout, TimeUnit unit)
      throws InterruptedException {
    if (permits < 0) throw new IllegalArgumentException();
    if (Thread.interrupted()) throw new InterruptedException();
    long end = System.currentTimeMillis() + unit.toMillis(timeout);
    __waiting++;
    try {
      while (__permits < permits) {
        long left = end - System.currentTimeMillis();
        if (left <= 0) return false;
        wait(left);
      }
    } finally {
      __waiting--;
    }
    __permits -= permits;
    return true;
  }

  public void release() { release(1); }

  public synchronized void release(int permits) {
    if (permits < 0) throw new IllegalArgumentException();
    __permits += permits;
    notifyAll();
  }

  public synchronized int availablePermits() { return __permits; }

  public synchronized int drainPermits() {
    int all = __permits;
    __permits = 0;
    return all;
  }

  public boolean isFair() { return __fair; }

  public final synchronized boolean hasQueuedThreads() { return __waiting > 0; }

  public final synchronized int getQueueLength() { return __waiting; }

  public String toString() {
    return super.toString() + "[Permits = " + availablePermits() + "]";
  }
}

// ---------------------------------------------------------------- locks

interface Lock {
  void lock();

  void lockInterruptibly() throws InterruptedException;

  boolean tryLock();

  boolean tryLock(long time, TimeUnit unit) throws InterruptedException;

  void unlock();

  Condition newCondition();
}

interface Condition {
  void await() throws InterruptedException;

  void awaitUninterruptibly();

  boolean await(long time, TimeUnit unit) throws InterruptedException;

  long awaitNanos(long nanosTimeout) throws InterruptedException;

  void signal();

  void signalAll();
}

class ReentrantLock implements Lock, Serializable {
  private final boolean __fair;
  Thread __owner;
  int __holds;
  private int __waiting;
  private final ArrayList<Thread> __queued = new ArrayList<>();

  public ReentrantLock() { this(false); }

  public ReentrantLock(boolean fair) { __fair = fair; }

  public synchronized void lock() {
    Thread me = Thread.currentThread();
    boolean interrupted = false;
    __waiting++;
    __queued.add(me);
    while (__owner != null && __owner != me) {
      try {
        wait();
      } catch (InterruptedException e) {
        interrupted = true;
      }
    }
    __queued.remove(me);
    __waiting--;
    __owner = me;
    __holds++;
    if (interrupted) me.interrupt();
  }

  public synchronized void lockInterruptibly() throws InterruptedException {
    if (Thread.interrupted()) throw new InterruptedException();
    Thread me = Thread.currentThread();
    __waiting++;
    try {
      while (__owner != null && __owner != me) wait();
    } finally {
      __waiting--;
    }
    __owner = me;
    __holds++;
  }

  public synchronized boolean tryLock() {
    Thread me = Thread.currentThread();
    if (__owner != null && __owner != me) return false;
    __owner = me;
    __holds++;
    return true;
  }

  public synchronized boolean tryLock(long timeout, TimeUnit unit) throws InterruptedException {
    if (Thread.interrupted()) throw new InterruptedException();
    Thread me = Thread.currentThread();
    long end = System.currentTimeMillis() + unit.toMillis(timeout);
    __waiting++;
    try {
      while (__owner != null && __owner != me) {
        long left = end - System.currentTimeMillis();
        if (left <= 0) return false;
        wait(left);
      }
    } finally {
      __waiting--;
    }
    __owner = me;
    __holds++;
    return true;
  }

  public synchronized void unlock() {
    if (__owner != Thread.currentThread()) throw new IllegalMonitorStateException();
    if (--__holds == 0) {
      __owner = null;
      notifyAll();
    }
  }

  public Condition newCondition() { return new __ConditionObject(this); }

  public synchronized int getHoldCount() {
    return __owner == Thread.currentThread() ? __holds : 0;
  }

  public synchronized boolean isHeldByCurrentThread() {
    return __owner == Thread.currentThread();
  }

  public synchronized boolean isLocked() { return __owner != null; }

  public final boolean isFair() { return __fair; }

  public synchronized boolean hasQueuedThreads() { return __waiting > 0; }

  public synchronized int getQueueLength() { return __waiting; }

  public final synchronized boolean hasQueuedThread(Thread thread) {
    if (thread == null) throw new NullPointerException();
    return __queued.contains(thread);
  }

  private __ConditionObject __owned(Condition condition) {
    if (condition == null) throw new NullPointerException();
    if (!(condition instanceof __ConditionObject) || !((__ConditionObject) condition).__of(this)) {
      throw new IllegalArgumentException("Not owner");
    }
    if (__owner != Thread.currentThread()) throw new IllegalMonitorStateException();
    return (__ConditionObject) condition;
  }

  public synchronized boolean hasWaiters(Condition condition) {
    return __owned(condition).__waiting() > 0;
  }

  public synchronized int getWaitQueueLength(Condition condition) {
    return __owned(condition).__waiting();
  }

  public String toString() {
    Thread owner;
    synchronized (this) {
      owner = __owner;
    }
    return super.toString()
        + (owner == null ? "[Unlocked]" : "[Locked by thread " + owner.getName() + "]");
  }
}

// A lock's condition: waiters queue in order, and every wait and signal is
// made holding the LOCK's own monitor, so a signal cannot slip between a
// waiter releasing the lock and starting to wait.
class __ConditionObject implements Condition {
  private final ReentrantLock __lock;
  private final ArrayDeque<boolean[]> __waiters = new ArrayDeque<>();

  __ConditionObject(ReentrantLock lock) { __lock = lock; }

  boolean __of(ReentrantLock lock) { return __lock == lock; }

  int __waiting() { return __waiters.size(); }

  public long awaitNanos(long nanosTimeout) throws InterruptedException {
    long end = System.nanoTime() + nanosTimeout;
    long ms = Math.max(1, (nanosTimeout + 999999) / 1000000);
    if (nanosTimeout <= 0) ms = 1;
    __awaitUntil(System.currentTimeMillis() + ms, true);
    return end - System.nanoTime();
  }

  private boolean __awaitUntil(long end, boolean interruptible) throws InterruptedException {
    Thread me = Thread.currentThread();
    synchronized (__lock) {
      if (__lock.__owner != me) throw new IllegalMonitorStateException();
      if (interruptible && Thread.interrupted()) throw new InterruptedException();
      boolean[] signalled = new boolean[1];
      __waiters.add(signalled);
      int holds = __lock.__holds;
      __lock.__owner = null;
      __lock.__holds = 0;
      __lock.notifyAll();
      boolean interrupted = false;
      boolean timedOut = false;
      while (!signalled[0]) {
        long left = end - System.currentTimeMillis();
        if (end != 0 && left <= 0) {
          timedOut = true;
          break;
        }
        try {
          if (end == 0) {
            __lock.wait();
          } else {
            __lock.wait(left);
          }
        } catch (InterruptedException e) {
          if (interruptible) {
            interrupted = true;
            break;
          }
          me.interrupt();
        }
      }
      __waiters.remove(signalled);
      while (__lock.__owner != null) {
        try {
          __lock.wait();
        } catch (InterruptedException e) {
          interrupted = interrupted || interruptible;
        }
      }
      __lock.__owner = me;
      __lock.__holds = holds;
      if (interrupted) throw new InterruptedException();
      return !timedOut;
    }
  }

  public void await() throws InterruptedException { __awaitUntil(0, true); }

  public void awaitUninterruptibly() {
    try {
      __awaitUntil(0, false);
    } catch (InterruptedException e) {
      Thread.currentThread().interrupt();
    }
  }

  public boolean await(long time, TimeUnit unit) throws InterruptedException {
    long ms = unit.toMillis(time);
    return __awaitUntil(System.currentTimeMillis() + Math.max(ms, 1), true);
  }

  public void signal() {
    synchronized (__lock) {
      if (__lock.__owner != Thread.currentThread()) throw new IllegalMonitorStateException();
      boolean[] first = __waiters.poll();
      if (first != null) {
        first[0] = true;
        __lock.notifyAll();
      }
    }
  }

  public void signalAll() {
    synchronized (__lock) {
      if (__lock.__owner != Thread.currentThread()) throw new IllegalMonitorStateException();
      while (!__waiters.isEmpty()) __waiters.poll()[0] = true;
      __lock.notifyAll();
    }
  }
}

// ---------------------------------------------------------------- atomics

class AtomicInteger extends Number implements Serializable {
  private int __value;

  public AtomicInteger() {}

  public AtomicInteger(int initialValue) { __value = initialValue; }

  public final synchronized int get() { return __value; }

  public final synchronized void set(int newValue) { __value = newValue; }

  public final void lazySet(int newValue) { set(newValue); }

  public final int getPlain() { return get(); }

  public final void setPlain(int newValue) { set(newValue); }

  public final int getOpaque() { return get(); }

  public final void setOpaque(int newValue) { set(newValue); }

  public final int getAcquire() { return get(); }

  public final void setRelease(int newValue) { set(newValue); }

  public final synchronized int getAndSet(int newValue) {
    int old = __value;
    __value = newValue;
    return old;
  }

  public final synchronized boolean compareAndSet(int expectedValue, int newValue) {
    if (__value != expectedValue) return false;
    __value = newValue;
    return true;
  }

  public final boolean weakCompareAndSet(int expectedValue, int newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetPlain(int expectedValue, int newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetVolatile(int expectedValue, int newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetAcquire(int expectedValue, int newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetRelease(int expectedValue, int newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final synchronized int compareAndExchange(int expectedValue, int newValue) {
    int old = __value;
    if (old == expectedValue) __value = newValue;
    return old;
  }

  public final int compareAndExchangeAcquire(int expectedValue, int newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final int compareAndExchangeRelease(int expectedValue, int newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final synchronized int getAndIncrement() { return __value++; }

  public final synchronized int getAndDecrement() { return __value--; }

  public final synchronized int getAndAdd(int delta) {
    int old = __value;
    __value += delta;
    return old;
  }

  public final synchronized int incrementAndGet() { return ++__value; }

  public final synchronized int decrementAndGet() { return --__value; }

  public final synchronized int addAndGet(int delta) { return __value += delta; }

  public final synchronized int getAndUpdate(IntUnaryOperator updateFunction) {
    int old = __value;
    __value = updateFunction.applyAsInt(old);
    return old;
  }

  public final synchronized int updateAndGet(IntUnaryOperator updateFunction) {
    return __value = updateFunction.applyAsInt(__value);
  }

  public final synchronized int getAndAccumulate(int x, IntBinaryOperator accumulatorFunction) {
    int old = __value;
    __value = accumulatorFunction.applyAsInt(old, x);
    return old;
  }

  public final synchronized int accumulateAndGet(int x, IntBinaryOperator accumulatorFunction) {
    return __value = accumulatorFunction.applyAsInt(__value, x);
  }

  public String toString() { return Integer.toString(get()); }

  public int intValue() { return get(); }

  public long longValue() { return (long) get(); }

  public float floatValue() { return (float) get(); }

  public double doubleValue() { return (double) get(); }
}

class AtomicLong extends Number implements Serializable {
  private long __value;

  public AtomicLong() {}

  public AtomicLong(long initialValue) { __value = initialValue; }

  public final synchronized long get() { return __value; }

  public final synchronized void set(long newValue) { __value = newValue; }

  public final void lazySet(long newValue) { set(newValue); }

  public final long getPlain() { return get(); }

  public final void setPlain(long newValue) { set(newValue); }

  public final long getOpaque() { return get(); }

  public final void setOpaque(long newValue) { set(newValue); }

  public final long getAcquire() { return get(); }

  public final void setRelease(long newValue) { set(newValue); }

  public final synchronized long getAndSet(long newValue) {
    long old = __value;
    __value = newValue;
    return old;
  }

  public final synchronized boolean compareAndSet(long expectedValue, long newValue) {
    if (__value != expectedValue) return false;
    __value = newValue;
    return true;
  }

  public final boolean weakCompareAndSet(long expectedValue, long newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetPlain(long expectedValue, long newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetVolatile(long expectedValue, long newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetAcquire(long expectedValue, long newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetRelease(long expectedValue, long newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final synchronized long compareAndExchange(long expectedValue, long newValue) {
    long old = __value;
    if (old == expectedValue) __value = newValue;
    return old;
  }

  public final long compareAndExchangeAcquire(long expectedValue, long newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final long compareAndExchangeRelease(long expectedValue, long newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final synchronized long getAndIncrement() { return __value++; }

  public final synchronized long getAndDecrement() { return __value--; }

  public final synchronized long getAndAdd(long delta) {
    long old = __value;
    __value += delta;
    return old;
  }

  public final synchronized long incrementAndGet() { return ++__value; }

  public final synchronized long decrementAndGet() { return --__value; }

  public final synchronized long addAndGet(long delta) { return __value += delta; }

  public final synchronized long getAndUpdate(LongUnaryOperator updateFunction) {
    long old = __value;
    __value = updateFunction.applyAsLong(old);
    return old;
  }

  public final synchronized long updateAndGet(LongUnaryOperator updateFunction) {
    return __value = updateFunction.applyAsLong(__value);
  }

  public final synchronized long getAndAccumulate(long x, LongBinaryOperator accumulatorFunction) {
    long old = __value;
    __value = accumulatorFunction.applyAsLong(old, x);
    return old;
  }

  public final synchronized long accumulateAndGet(long x, LongBinaryOperator accumulatorFunction) {
    return __value = accumulatorFunction.applyAsLong(__value, x);
  }

  public String toString() { return Long.toString(get()); }

  public int intValue() { return (int) get(); }

  public long longValue() { return get(); }

  public float floatValue() { return (float) get(); }

  public double doubleValue() { return (double) get(); }
}

class AtomicBoolean implements Serializable {
  private boolean __value;

  public AtomicBoolean() {}

  public AtomicBoolean(boolean initialValue) { __value = initialValue; }

  public final synchronized boolean get() { return __value; }

  public final synchronized void set(boolean newValue) { __value = newValue; }

  public final void lazySet(boolean newValue) { set(newValue); }

  public final boolean getPlain() { return get(); }

  public final void setPlain(boolean newValue) { set(newValue); }

  public final boolean getOpaque() { return get(); }

  public final void setOpaque(boolean newValue) { set(newValue); }

  public final boolean getAcquire() { return get(); }

  public final void setRelease(boolean newValue) { set(newValue); }

  public final synchronized boolean getAndSet(boolean newValue) {
    boolean old = __value;
    __value = newValue;
    return old;
  }

  public final synchronized boolean compareAndSet(boolean expectedValue, boolean newValue) {
    if (__value != expectedValue) return false;
    __value = newValue;
    return true;
  }

  public boolean weakCompareAndSet(boolean expectedValue, boolean newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public boolean weakCompareAndSetPlain(boolean expectedValue, boolean newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public boolean weakCompareAndSetVolatile(boolean expectedValue, boolean newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public boolean weakCompareAndSetAcquire(boolean expectedValue, boolean newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public boolean weakCompareAndSetRelease(boolean expectedValue, boolean newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final synchronized boolean compareAndExchange(boolean expectedValue, boolean newValue) {
    boolean old = __value;
    if (old == expectedValue) __value = newValue;
    return old;
  }

  public final boolean compareAndExchangeAcquire(boolean expectedValue, boolean newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final boolean compareAndExchangeRelease(boolean expectedValue, boolean newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public String toString() { return Boolean.toString(get()); }
}

class AtomicReference<V> implements Serializable {
  private V __value;

  public AtomicReference() {}

  public AtomicReference(V initialValue) { __value = initialValue; }

  public final synchronized V get() { return __value; }

  public final synchronized void set(V newValue) { __value = newValue; }

  public final void lazySet(V newValue) { set(newValue); }

  public final V getPlain() { return get(); }

  public final void setPlain(V newValue) { set(newValue); }

  public final V getOpaque() { return get(); }

  public final void setOpaque(V newValue) { set(newValue); }

  public final V getAcquire() { return get(); }

  public final void setRelease(V newValue) { set(newValue); }

  public final synchronized V getAndSet(V newValue) {
    V old = __value;
    __value = newValue;
    return old;
  }

  // By IDENTITY, as the JDK compares.
  public final synchronized boolean compareAndSet(V expectedValue, V newValue) {
    if (__value != expectedValue) return false;
    __value = newValue;
    return true;
  }

  public final boolean weakCompareAndSet(V expectedValue, V newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetPlain(V expectedValue, V newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetVolatile(V expectedValue, V newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetAcquire(V expectedValue, V newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final boolean weakCompareAndSetRelease(V expectedValue, V newValue) {
    return compareAndSet(expectedValue, newValue);
  }

  public final synchronized V compareAndExchange(V expectedValue, V newValue) {
    V old = __value;
    if (old == expectedValue) __value = newValue;
    return old;
  }

  public final V compareAndExchangeAcquire(V expectedValue, V newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final V compareAndExchangeRelease(V expectedValue, V newValue) {
    return compareAndExchange(expectedValue, newValue);
  }

  public final synchronized V getAndUpdate(UnaryOperator<V> updateFunction) {
    V old = __value;
    __value = updateFunction.apply(old);
    return old;
  }

  public final synchronized V updateAndGet(UnaryOperator<V> updateFunction) {
    return __value = updateFunction.apply(__value);
  }

  public final synchronized V getAndAccumulate(V x, BinaryOperator<V> accumulatorFunction) {
    V old = __value;
    __value = accumulatorFunction.apply(old, x);
    return old;
  }

  public final synchronized V accumulateAndGet(V x, BinaryOperator<V> accumulatorFunction) {
    return __value = accumulatorFunction.apply(__value, x);
  }

  public String toString() { return String.valueOf(get()); }
}
