// Bundled subset of java.util helpers used by the CSA corpus, injected when a
// source references them: Random (backed by Math.random, deterministic under
// the VM's seeded RNG) and Collections.

class Random {
  public Random() {}
  public Random(long seed) {}
  public int nextInt(int bound) { return (int) (Math.random() * bound); }
  public int nextInt(int origin, int bound) { return origin + (int) (Math.random() * (bound - origin)); }
  public int nextInt() { return (int) (Math.random() * 2147483647.0); }
  public long nextLong() { return (long) (Math.random() * 9.223372036854776E18); }
  public double nextDouble() { return Math.random(); }
  public boolean nextBoolean() { return Math.random() < 0.5; }
}

class Collections {
  public static void reverse(java.util.ArrayList<Object> list) {
    int n = list.size();
    for (int i = 0; i < n / 2; i++) {
      Object tmp = list.get(i);
      list.set(i, list.get(n - 1 - i));
      list.set(n - 1 - i, tmp);
    }
  }

  public static void swap(java.util.ArrayList<Object> list, int i, int j) {
    Object tmp = list.get(i);
    list.set(i, list.get(j));
    list.set(j, tmp);
  }
}
