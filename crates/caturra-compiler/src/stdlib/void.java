// java.lang.Void — the class of no values, written only as a TYPE ARGUMENT:
// `Callable<Void>`, `SwingWorker<String, Void>`, where the only thing to return
// is `null`. It cannot be instantiated, as on a JDK.
final class Void {
  // The Class object for the pseudo-type `void`, as on a JDK.
  public static final Class<Void> TYPE = void.class;
  private Void() {}
}
