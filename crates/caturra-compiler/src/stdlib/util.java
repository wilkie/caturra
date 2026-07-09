// Bundled subset of java.util helpers used by the CSA corpus, injected when a
// source references them: Random and Collections.

// Java's exact 48-bit linear congruential generator, so a given seed replays
// the same sequence a real JVM produces — `new Random(42).nextInt()` is
// -1170105035 here as it is there. An unseeded Random draws its seed from
// Math.random(), which the VM seeds deterministically for tests and from host
// entropy in the browser.
class Random {
  private long __seed;
  private double __nextGaussian = 0.0;
  private boolean __haveNextGaussian = false;

  public Random() { setSeed((long) (Math.random() * 281474976710656.0)); }
  public Random(long seed) { setSeed(seed); }

  public void setSeed(long seed) {
    __seed = (seed ^ 0x5DEECE66DL) & 281474976710655L; // (1L << 48) - 1
    __haveNextGaussian = false;
  }

  // The generator's core: advance the state and take the top `bits`.
  protected int next(int bits) {
    __seed = (__seed * 0x5DEECE66DL + 0xBL) & 281474976710655L;
    return (int) (__seed >>> (48 - bits));
  }

  public int nextInt() { return next(32); }

  public int nextInt(int bound) {
    if (bound <= 0) throw new IllegalArgumentException("bound must be positive");
    int r = next(31);
    int m = bound - 1;
    if ((bound & m) == 0) {
      // A power of two takes the high bits directly.
      r = (int) ((bound * (long) r) >> 31);
    } else {
      // Otherwise reject the values that would bias the modulo.
      int u = r;
      r = u % bound;
      while (u - r + m < 0) {
        u = next(31);
        r = u % bound;
      }
    }
    return r;
  }

  public long nextLong() { return ((long) next(32) << 32) + next(32); }

  public boolean nextBoolean() { return next(1) != 0; }

  public float nextFloat() { return next(24) / ((float) (1 << 24)); }

  public double nextDouble() {
    return (((long) next(26) << 27) + next(27)) / 9007199254740992.0; // 2^53
  }

  // The polar (Marsaglia) method, which is what java.util.Random uses: it makes
  // two values at a time, so the second is cached.
  public double nextGaussian() {
    if (__haveNextGaussian) {
      __haveNextGaussian = false;
      return __nextGaussian;
    }
    double v1 = 0.0;
    double v2 = 0.0;
    double s = 0.0;
    do {
      v1 = 2 * nextDouble() - 1;
      v2 = 2 * nextDouble() - 1;
      s = v1 * v1 + v2 * v2;
    } while (s >= 1 || s == 0);
    double factor = Math.sqrt(-2 * Math.log(s) / s);
    __nextGaussian = v2 * factor;
    __haveNextGaussian = true;
    return v1 * factor;
  }
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
