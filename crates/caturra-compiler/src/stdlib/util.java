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

  public Random() { setSeed((long) (__Math.random() * 281474976710656.0)); }
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

  // The STREAM factories (Java 8). Each is lazy, as the JDK's are, so the
  // generator advances once per element PULLED — a `limit`ed or short-circuited
  // pipeline leaves the seed exactly where a real one would.
  // The two complaints a JDK makes before it hands back a stream, in its own
  // words. Unchecked, a negative size reached `limit`, which reports the
  // number alone, and a bound at or below the origin drew from an empty range.
  private void __checkSize(long streamSize) {
    if (streamSize < 0L) throw new IllegalArgumentException("size must be non-negative");
  }

  void __checkRange(boolean ordered) {
    if (!ordered) throw new IllegalArgumentException("bound must be greater than origin");
  }

  public java.util.stream.IntStream ints() {
    return java.util.stream.IntStream.generate(() -> nextInt());
  }

  public java.util.stream.IntStream ints(long streamSize) {
    __checkSize(streamSize);
    return java.util.stream.IntStream.generate(() -> nextInt()).limit(streamSize);
  }

  public java.util.stream.IntStream ints(int origin, int bound) {
    __checkRange(origin < bound);
    return java.util.stream.IntStream.generate(() -> __boundedInt(origin, bound));
  }

  public java.util.stream.IntStream ints(long streamSize, int origin, int bound) {
    __checkSize(streamSize);
    __checkRange(origin < bound);
    return java.util.stream.IntStream.generate(() -> __boundedInt(origin, bound))
        .limit(streamSize);
  }

  public java.util.stream.LongStream longs() {
    return java.util.stream.LongStream.generate(() -> nextLong());
  }

  public java.util.stream.LongStream longs(long streamSize) {
    __checkSize(streamSize);
    return java.util.stream.LongStream.generate(() -> nextLong()).limit(streamSize);
  }

  public java.util.stream.LongStream longs(long origin, long bound) {
    __checkRange(origin < bound);
    return java.util.stream.LongStream.generate(() -> __boundedLong(origin, bound));
  }

  public java.util.stream.LongStream longs(long streamSize, long origin, long bound) {
    __checkSize(streamSize);
    __checkRange(origin < bound);
    return java.util.stream.LongStream.generate(() -> __boundedLong(origin, bound))
        .limit(streamSize);
  }

  public java.util.stream.DoubleStream doubles() {
    return java.util.stream.DoubleStream.generate(() -> nextDouble());
  }

  public java.util.stream.DoubleStream doubles(long streamSize) {
    __checkSize(streamSize);
    return java.util.stream.DoubleStream.generate(() -> nextDouble()).limit(streamSize);
  }

  public java.util.stream.DoubleStream doubles(double origin, double bound) {
    __checkRange(origin < bound);
    return java.util.stream.DoubleStream.generate(() -> __boundedDouble(origin, bound));
  }

  public java.util.stream.DoubleStream doubles(long streamSize, double origin, double bound) {
    __checkSize(streamSize);
    __checkRange(origin < bound);
    return java.util.stream.DoubleStream.generate(() -> __boundedDouble(origin, bound))
        .limit(streamSize);
  }

  // `internalNextDouble`: scale one draw into the range, and pull the result
  // back under the bound if rounding pushed it over.
  double __boundedDouble(double origin, double bound) {
    double drawn = nextDouble() * (bound - origin) + origin;
    if (drawn >= bound) {
      drawn = Double.longBitsToDouble(Double.doubleToLongBits(bound) - 1L);
    }
    return drawn;
  }

  // `internalNextLong`: the JDK draws a whole long, then folds it into the
  // range by repeated modulus — rejecting a draw that would bias the result,
  // which is why the loop is there rather than a single remainder.
  long __boundedLong(long origin, long bound) {
    long drawn = nextLong();
    long span = bound - origin;
    long limit = span - 1;
    if (span > 0) {
      if ((span & limit) == 0L) {
        drawn = (drawn & limit) + origin;
      } else {
        long candidate = drawn >>> 1;
        long value = candidate % span;
        while (candidate + limit - value < 0L) {
          candidate = nextLong() >>> 1;
          value = candidate % span;
        }
        drawn = value + origin;
      }
    } else {
      while (drawn < origin || drawn >= bound) {
        drawn = nextLong();
      }
    }
    return drawn;
  }

  // `internalNextInt`: a positive span is one bounded draw shifted, and a span
  // that OVERFLOWS an int is drawn whole and rejected until it lands.
  int __boundedInt(int origin, int bound) {
    int span = bound - origin;
    if (span > 0) {
      return nextInt(span) + origin;
    }
    int drawn = nextInt();
    while (drawn < origin || drawn >= bound) {
      drawn = nextInt();
    }
    return drawn;
  }

  public double nextDouble() {
    return (((long) next(26) << 27) + next(27)) / 9007199254740992.0; // 2^53
  }

  // The polar (Marsaglia) method, which is what java.util.Random uses: it makes
  // two values at a time, so the second is cached.
  // Fills the array with random bytes, four per next(32) draw, low byte first
  // — so a seeded Random replays the JDK's exact bytes.
  public void nextBytes(byte[] bytes) {
    int i = 0;
    int len = bytes.length;
    while (i < len) {
      int rnd = nextInt();
      int n = __Math.min(len - i, 4);
      while (n > 0) {
        bytes[i] = (byte) rnd;
        i++;
        rnd >>= 8;
        n--;
      }
    }
  }

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
    double factor = __Math.sqrt(-2 * __Math.log(s) / s);
    __nextGaussian = v2 * factor;
    __haveNextGaussian = true;
    return v1 * factor;
  }
}

// `java.util.concurrent.ThreadLocalRandom` — the random source a program reaches
// for with `ThreadLocalRandom.current().nextInt(lo, hi)`, because `Random` has
// no two-argument `nextInt` until Java 17. One thread here, so one instance,
// seeded the way an unseeded `Random` is; a JDK's is seeded per thread and
// exposes no seed, so its NUMBERS can never be compared — its ranges, its
// refusals and its class can.
class ThreadLocalRandom extends Random {
  private static ThreadLocalRandom __instance;
  private boolean __initialized;

  private ThreadLocalRandom() {
    super();
    __initialized = true;
  }

  public static ThreadLocalRandom current() {
    if (__instance == null) __instance = new ThreadLocalRandom();
    return __instance;
  }

  // A JDK refuses to be reseeded once it exists (the constructor's own call,
  // before the flag is set, is the one that is allowed).
  public void setSeed(long seed) {
    if (__initialized) throw new UnsupportedOperationException();
    super.setSeed(seed);
  }

  public int nextInt(int origin, int bound) {
    __checkRange(origin < bound);
    return __boundedInt(origin, bound);
  }

  public long nextLong(long bound) {
    if (bound <= 0L) throw new IllegalArgumentException("bound must be positive");
    return __boundedLong(0L, bound);
  }

  public long nextLong(long origin, long bound) {
    __checkRange(origin < bound);
    return __boundedLong(origin, bound);
  }

  // `!(bound > 0.0)` rather than `bound <= 0.0`: a NaN bound is refused too.
  public double nextDouble(double bound) {
    if (!(bound > 0.0)) throw new IllegalArgumentException("bound must be positive");
    return __boundedDouble(0.0, bound);
  }

  public double nextDouble(double origin, double bound) {
    __checkRange(origin < bound);
    return __boundedDouble(origin, bound);
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

  // Written as a JDK writes it — `set(i, set(j, get(i)))` — because the ORDER
  // is observable: on a `List.of` a swap past the end reaches the refused
  // `set` before the bad index, so it is an UnsupportedOperationException and
  // not an IndexOutOfBounds.
  public static void swap(java.util.ArrayList<Object> list, int i, int j) {
    list.set(i, list.set(j, list.get(i)));
  }

  // Fisher-Yates from the end, exactly as java.util.Collections does it, so a
  // seeded Random replays the JDK's permutation.
  public static void shuffle(java.util.ArrayList<Object> list, Random rnd) {
    for (int i = list.size(); i > 1; i--) swap(list, i - 1, rnd.nextInt(i));
  }

  public static void shuffle(java.util.ArrayList<Object> list) {
    shuffle(list, new Random());
  }
}

// `java.util.StringJoiner` — the JDK's own shape, because the details are
// observable: the builder holds the PREFIX and the elements (never the
// suffix), which is what makes `merge` splice another joiner's contents
// without its prefix, and what makes `length()` answer before `toString()`
// ever runs. An empty joiner prints `setEmptyValue`'s text if one was set and
// prefix+suffix otherwise.
class StringJoiner {
  private final String __delimiter;
  private final String __prefix;
  private final String __suffix;
  private StringBuilder __value = null;
  private String __empty;

  public StringJoiner(CharSequence delimiter) {
    this(delimiter, "", "");
  }

  public StringJoiner(CharSequence delimiter, CharSequence prefix, CharSequence suffix) {
    if (prefix == null) {
      throw new NullPointerException("The prefix must not be null");
    }
    if (delimiter == null) {
      throw new NullPointerException("The delimiter must not be null");
    }
    if (suffix == null) {
      throw new NullPointerException("The suffix must not be null");
    }
    __prefix = prefix.toString();
    __delimiter = delimiter.toString();
    __suffix = suffix.toString();
    __empty = __prefix + __suffix;
  }

  public StringJoiner setEmptyValue(CharSequence emptyValue) {
    if (emptyValue == null) {
      throw new NullPointerException("The empty value must not be null");
    }
    __empty = emptyValue.toString();
    return this;
  }

  // The builder is created on the FIRST add, holding the prefix; every later
  // one appends the delimiter first. That is why an empty joiner can still
  // answer a different text.
  private StringBuilder __prepare() {
    if (__value != null) {
      __value.append(__delimiter);
    } else {
      __value = new StringBuilder();
      __value.append(__prefix);
    }
    return __value;
  }

  public StringJoiner add(CharSequence newElement) {
    StringBuilder builder = __prepare();
    if (newElement == null) {
      builder.append("null");
    } else {
      builder.append(newElement.toString());
    }
    return this;
  }

  // `merge` takes the other joiner's ELEMENTS, not its prefix — and a merged
  // joiner counts as one element, so the delimiter goes in once.
  public StringJoiner merge(StringJoiner other) {
    if (other == null) {
      throw new NullPointerException();
    }
    if (other.__value != null) {
      String theirs = other.__value.toString().substring(other.__prefix.length());
      __prepare().append(theirs);
    }
    return this;
  }

  public int length() {
    if (__value == null) {
      return __empty.length();
    }
    return __value.length() + __suffix.length();
  }

  @Override
  public String toString() {
    if (__value == null) {
      return __empty;
    }
    return __value.toString() + __suffix;
  }
}
