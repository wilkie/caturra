// javax.swing.SwingWorker (specs/CONCURRENCY.md) — work off the event-dispatch
// thread, as a JDK does it: a `FutureTask` over `doInBackground`, run by a
// pool of up to ten DAEMON threads named `SwingWorker-pool-N-thread-M`;
// `publish`ed chunks reach `process` on the dispatch thread, and so does
// `done`; the "state" and "progress" properties are reported to listeners on
// the dispatch thread too. Captured from a JDK 11 first.

// `SwingWorker.StateValue`, at the top level under a reserved name (a nested
// class would be hoisted as `StateValue`), carrying the JDK's binary name.
enum __SwingWorkerStateValue { PENDING, STARTED, DONE }

abstract class SwingWorker<T, V> implements RunnableFuture<T> {
  private static ExecutorService __executor = null;

  private final FutureTask<T> __future;
  private volatile __SwingWorkerStateValue __state = __SwingWorkerStateValue.PENDING;
  private int __progress = 0;
  private final PropertyChangeSupport __support;
  // Chunks published and not yet handed to `process`.
  private final java.util.ArrayList<V> __chunks = new java.util.ArrayList<V>();
  private boolean __processQueued = false;

  public SwingWorker() {
    __future = new __SwingWorkerTask<T>(new __SwingWorkerCall<T>(this), this);
    __support = new __SwingWorkerSupport(this);
  }

  protected abstract T doInBackground() throws Exception;

  protected void process(java.util.List<V> chunks) {}

  protected void done() {}

  // Runs the task HERE, on the calling thread (a JDK's `run` is the same).
  public final void run() { __future.run(); }

  public final void execute() { __workers().execute(this); }

  @SafeVarargs
  protected final void publish(V... chunks) {
    synchronized (__chunks) {
      for (int i = 0; i < chunks.length; i++) __chunks.add(chunks[i]);
      if (__processQueued) return;
      __processQueued = true;
    }
    SwingUtilities.invokeLater(new __SwingWorkerProcess<V>(this));
  }

  // On the dispatch thread: everything published since the last delivery.
  void __deliverChunks() {
    java.util.ArrayList<V> batch;
    synchronized (__chunks) {
      batch = new java.util.ArrayList<V>(__chunks);
      __chunks.clear();
      __processQueued = false;
    }
    if (!batch.isEmpty()) process(batch);
  }

  protected final void setProgress(int progress) {
    if (progress < 0 || progress > 100) {
      throw new IllegalArgumentException("the value should be from 0 to 100");
    }
    if (__progress == progress) return;
    int old = __progress;
    __progress = progress;
    if (!__support.hasListeners("progress")) return;
    __support.firePropertyChange("progress", old, progress);
  }

  public final int getProgress() { return __progress; }

  public final boolean cancel(boolean mayInterruptIfRunning) {
    return __future.cancel(mayInterruptIfRunning);
  }

  public final boolean isCancelled() { return __future.isCancelled(); }

  public final boolean isDone() { return __future.isDone(); }

  public final T get() throws InterruptedException, ExecutionException { return __future.get(); }

  public final T get(long timeout, TimeUnit unit)
      throws InterruptedException, ExecutionException, TimeoutException {
    return __future.get(timeout, unit);
  }

  public final void addPropertyChangeListener(PropertyChangeListener listener) {
    __support.addPropertyChangeListener(listener);
  }

  public final void removePropertyChangeListener(PropertyChangeListener listener) {
    __support.removePropertyChangeListener(listener);
  }

  public final void firePropertyChange(String propertyName, Object oldValue, Object newValue) {
    __support.firePropertyChange(propertyName, oldValue, newValue);
  }

  public final PropertyChangeSupport getPropertyChangeSupport() { return __support; }

  // A finished task is DONE whatever the field says: `done` may not have run.
  public final __SwingWorkerStateValue getState() {
    if (isDone()) return __SwingWorkerStateValue.DONE;
    return __state;
  }

  void __setState(__SwingWorkerStateValue state) {
    __SwingWorkerStateValue old = __state;
    __state = state;
    firePropertyChange("state", old, state);
  }

  // The task is over, however it ended: `done` on the dispatch thread, then
  // the DONE state.
  void __finished() {
    if (SwingUtilities.isEventDispatchThread()) {
      done();
    } else {
      SwingUtilities.invokeLater(new __SwingWorkerDone<T, V>(this));
    }
    __setState(__SwingWorkerStateValue.DONE);
  }

  private static synchronized ExecutorService __workers() {
    if (__executor == null) {
      __executor = Executors.newFixedThreadPool(10, new __SwingWorkerThreads());
    }
    return __executor;
  }
}

// `SwingWorker-` before a default factory's name, and a daemon, as a JDK's.
class __SwingWorkerThreads implements ThreadFactory {
  private final ThreadFactory __base = Executors.defaultThreadFactory();
  public Thread newThread(Runnable r) {
    Thread thread = __base.newThread(r);
    thread.setName("SwingWorker-" + thread.getName());
    thread.setDaemon(true);
    return thread;
  }
}

class __SwingWorkerCall<T> implements Callable<T> {
  private final SwingWorker<T, ?> __worker;
  __SwingWorkerCall(SwingWorker<T, ?> worker) { __worker = worker; }
  public T call() throws Exception {
    __worker.__setState(__SwingWorkerStateValue.STARTED);
    return __worker.doInBackground();
  }
}

class __SwingWorkerTask<T> extends FutureTask<T> {
  private final SwingWorker<T, ?> __worker;
  __SwingWorkerTask(Callable<T> callable, SwingWorker<T, ?> worker) {
    super(callable);
    __worker = worker;
  }
  protected void done() { __worker.__finished(); }
}

class __SwingWorkerProcess<V> implements Runnable {
  private final SwingWorker<?, V> __worker;
  __SwingWorkerProcess(SwingWorker<?, V> worker) { __worker = worker; }
  public void run() { __worker.__deliverChunks(); }
}

class __SwingWorkerDone<T, V> implements Runnable {
  private final SwingWorker<T, V> __worker;
  __SwingWorkerDone(SwingWorker<T, V> worker) { __worker = worker; }
  public void run() { __worker.done(); }
}

// A worker's listeners hear of a change on the dispatch thread, wherever it
// was made.
class __SwingWorkerSupport extends PropertyChangeSupport {
  __SwingWorkerSupport(Object source) { super(source); }
  public void firePropertyChange(PropertyChangeEvent event) {
    if (SwingUtilities.isEventDispatchThread()) {
      super.firePropertyChange(event);
    } else {
      SwingUtilities.invokeLater(new __SwingWorkerFire(this, event));
    }
  }
  void __fireHere(PropertyChangeEvent event) { super.firePropertyChange(event); }
}

class __SwingWorkerFire implements Runnable {
  private final __SwingWorkerSupport __support;
  private final PropertyChangeEvent __event;
  __SwingWorkerFire(__SwingWorkerSupport support, PropertyChangeEvent event) {
    __support = support;
    __event = event;
  }
  public void run() { __support.__fireHere(__event); }
}
