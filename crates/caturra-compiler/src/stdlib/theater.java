// Bundled clean-room org.code.theater + org.code.media (Code.org CSA).
// Auto-injected when a source imports org.code.theater or org.code.media.
// Phase 1 (headless): Scene records draw commands; Theater.playScenes writes
// them to the VFS file "theater.log" for inspection. Canvas is 400x400 and
// Color values match the real library; the pixel/GIF renderer is Phase 2.
// Only Scene/Theater/Color/Image/Pixel/Font/FontStyle/Instrument are public.

enum Font { MONO, SANS }
enum FontStyle { NORMAL, BOLD, ITALIC }
enum Instrument { PIANO, BASS }

class Color {
  int red, green, blue;
  // The real library routes every constructor through sanitizeValue, which
  // clamps each channel to 0..255 ("Values below 0 will be set to 0, and
  // values above 255 will be set to 255"). The filter lessons rely on it:
  // U5L8's sharpen() caps at 255 but happily hands setRed() a negative.
  public Color(int red, int green, int blue) { this.red = __clamp(red); this.green = __clamp(green); this.blue = __clamp(blue); }
  public Color(Color c) { this.red = c.red; this.green = c.green; this.blue = c.blue; }
  public Color(String name) {
    int[] rgb = __resolve(name);
    if (rgb == null) throw new IllegalArgumentException("Invalid color " + name);
    this.red = rgb[0]; this.green = rgb[1]; this.blue = rgb[2];
  }
  public int getRed() { return red; }
  public int getGreen() { return green; }
  public int getBlue() { return blue; }
  public static Color copyWithRed(Color c, int v) { return new Color(v, c.green, c.blue); }
  public static Color copyWithGreen(Color c, int v) { return new Color(c.red, v, c.blue); }
  public static Color copyWithBlue(Color c, int v) { return new Color(c.red, c.green, v); }
  public String toString() { return red + " " + green + " " + blue; }
  static int __clamp(int v) { return v < 0 ? 0 : (v > 255 ? 255 : v); }
  // Names only, exactly the 27 the real ColorConstantMap holds. NOT hex: the
  // real Color(String) looks the name up in that map and throws for anything
  // else, so `new Color("#40E0D0")` is an IllegalArgumentException on Code.org.
  // We used to accept it, which is the dangerous direction — a program that
  // runs here and dies there. (org.code.neighborhood's painter colours are a
  // different set and DO take hex; see __NbhdColors.)
  static int[] __resolve(String s) {
    String n = s.toUpperCase();
    if (n.equals("WHITE")) return new int[]{255,255,255};
    if (n.equals("SILVER")) return new int[]{192,192,192};
    if (n.equals("GRAY")) return new int[]{128,128,128};
    if (n.equals("BLACK")) return new int[]{0,0,0};
    if (n.equals("RED")) return new int[]{255,0,0};
    if (n.equals("MAROON")) return new int[]{128,0,0};
    if (n.equals("YELLOW")) return new int[]{255,255,0};
    if (n.equals("OLIVE")) return new int[]{128,128,0};
    // The upstream source literally writes `new Color(0, 256, 0)` for LIME;
    // sanitizeValue clamps it, so the observable green is 255.
    if (n.equals("LIME")) return new int[]{0,255,0};
    if (n.equals("GREEN")) return new int[]{0,128,0};
    if (n.equals("AQUA")) return new int[]{0,255,255};
    if (n.equals("TEAL")) return new int[]{0,128,128};
    if (n.equals("BLUE")) return new int[]{0,0,255};
    if (n.equals("NAVY")) return new int[]{0,0,128};
    if (n.equals("FUCHSIA")) return new int[]{255,0,255};
    if (n.equals("PURPLE")) return new int[]{128,0,128};
    if (n.equals("PINK")) return new int[]{255,192,203};
    if (n.equals("ORANGE")) return new int[]{255,165,0};
    if (n.equals("GOLD")) return new int[]{255,215,0};
    if (n.equals("BROWN")) return new int[]{165,42,42};
    if (n.equals("CHOCOLATE")) return new int[]{210,105,30};
    if (n.equals("TAN")) return new int[]{210,180,140};
    if (n.equals("TURQUOISE")) return new int[]{64,224,208};
    if (n.equals("INDIGO")) return new int[]{75,0,130};
    if (n.equals("VIOLET")) return new int[]{238,130,238};
    if (n.equals("BEIGE")) return new int[]{245,245,220};
    if (n.equals("IVORY")) return new int[]{255,255,240};
    return null;
  }
  public static final Color WHITE = new Color(255,255,255);
  public static final Color SILVER = new Color(192,192,192);
  public static final Color GRAY = new Color(128,128,128);
  public static final Color BLACK = new Color(0,0,0);
  public static final Color RED = new Color(255,0,0);
  public static final Color MAROON = new Color(128,0,0);
  public static final Color YELLOW = new Color(255,255,0);
  public static final Color OLIVE = new Color(128,128,0);
  public static final Color LIME = new Color(0,256,0);
  public static final Color GREEN = new Color(0,128,0);
  public static final Color AQUA = new Color(0,255,255);
  public static final Color TEAL = new Color(0,128,128);
  public static final Color BLUE = new Color(0,0,255);
  public static final Color NAVY = new Color(0,0,128);
  public static final Color FUCHSIA = new Color(255,0,255);
  public static final Color PURPLE = new Color(128,0,128);
  public static final Color PINK = new Color(255,192,203);
  public static final Color ORANGE = new Color(255,165,0);
  public static final Color GOLD = new Color(255,215,0);
  public static final Color BROWN = new Color(165,42,42);
  public static final Color CHOCOLATE = new Color(210,105,30);
  public static final Color TAN = new Color(210,180,140);
  public static final Color TURQUOISE = new Color(64,224,208);
  public static final Color INDIGO = new Color(75,0,130);
  public static final Color VIOLET = new Color(238,130,238);
  public static final Color BEIGE = new Color(245,245,220);
  public static final Color IVORY = new Color(255,255,240);
}

class Pixel {
  Image image; int x, y;
  // The image's packed-pixel array and this pixel's index into it, cached at
  // construction. The filter lessons build a Pixel for every pixel of a 400x400
  // image and then touch all three channels, so resolving `image.px[y * width +
  // x]` on each access — five field reads plus a call into Image — dominated the
  // run. `px` is only ever mutated in place, never replaced, so this stays valid.
  int[] __px; int __i;
  Pixel(Image image, int x, int y) {
    this.image = image; this.x = x; this.y = y;
    this.__px = image.px; this.__i = y * image.width + x;
  }
  public int getX() { return x; }
  public int getY() { return y; }
  public Color getColor() { return image.__get(x, y); }
  public void setColor(Color c) { image.__set(x, y, c); }
  // Read/write a single channel straight through the image's packed pixel, with
  // no Color object in between. The filter lessons walk every pixel of a 400x400
  // image (160k) and touch all three channels, so going via getColor()/new
  // Color() allocated ~1.4M objects on a heap that never collects.
  public int getRed() { return (__px[__i] >> 16) & 255; }
  public int getGreen() { return (__px[__i] >> 8) & 255; }
  public int getBlue() { return __px[__i] & 255; }
  // The real setters go through `Color.copyWithRed(...)`, whose constructor
  // clamps each channel to 0..255 — so an out-of-range value must clamp, not
  // wrap: U5L8's sharpen() caps at 255 but hands setRed() negatives, and a
  // negative shifted into the packed pixel would smear across the other
  // channels (red/yellow streaks over the whole image). The clamp is spelled
  // out as a ternary instead of calling Color.__clamp: a call would knock
  // these hottest-of-all methods off the VM's frameless fast path.
  public void setRed(int v) { v = v < 0 ? 0 : (v > 255 ? 255 : v); __px[__i] = (v << 16) | (__px[__i] & 0x0000FFFF); }
  public void setGreen(int v) { v = v < 0 ? 0 : (v > 255 ? 255 : v); int p = __px[__i]; __px[__i] = (p & 0x00FF0000) | (v << 8) | (p & 0x000000FF); }
  public void setBlue(int v) { v = v < 0 ? 0 : (v > 255 ? 255 : v); __px[__i] = (__px[__i] & 0x00FFFF00) | v; }
}

class Image {
  // Pixels are packed 0xRRGGBB, row-major. The host preloads a named asset's
  // pixels into the VFS and the VM hands us the whole buffer in one native call
  // (System.__imagePixels): a 400x400 image is 160k pixels, so a per-pixel
  // decode loop in the interpreter would take a minute. See specs/EXECUTION.md.
  int width, height;
  int[] px;
  // The image's Pixel objects, built on first use. The real Image holds a
  // Pixel[][] and hands the SAME object back from every getPixel(x, y) — so
  // `img.getPixel(0, 0) == img.getPixel(0, 0)` is true, and an out-of-range
  // coordinate is an ArrayIndexOutOfBoundsException off this array rather
  // than a Pixel aliasing some other row. Built lazily, as the real one is
  // for a loaded image: a 400x400 image is 160k Pixels, and a program that
  // never asks for one should not pay for them.
  Pixel[][] __pixels;
  public Image(int width, int height) {
    this.width = width; this.height = height;
    px = new int[width * height];
    // The real Image(int, int) fills with DEFAULT_BACKGROUND_COLOR, which is
    // Color.WHITE. Java's own zero-fill would leave it black.
    __fill(0x00FFFFFF);
  }
  public Image(String filename) {
    int[] dims = System.__imageDims("__caturra_image_" + filename);
    if (dims.length == 2 && dims[0] > 0 && dims[1] > 0) {
      width = dims[0]; height = dims[1];
      px = System.__imagePixels("__caturra_image_" + filename);
    } else {
      // The real Image(String) throws FILE_NOT_FOUND for an asset it cannot
      // load. We used to hand back a blank 100x100 canvas instead — which is
      // how three lessons came to draw an empty box for months: the manifest
      // was missing their picture and the fallback swallowed it. A missing
      // asset is a real failure and now says so.
      throw new RuntimeException("FILE_NOT_FOUND");
    }
  }
  public Image(Image src) {
    this.width = src.width; this.height = src.height;
    px = new int[width * height];
    System.arraycopy(src.px, 0, px, 0, px.length);
  }
  void __fill(int packed) {
    for (int i = 0; i < px.length; i++) { px[i] = packed; }
  }
  public int getWidth() { return width; }
  public int getHeight() { return height; }
  public Pixel getPixel(int x, int y) {
    if (__pixels == null) { __makePixels(); }
    return __pixels[x][y];
  }
  void __makePixels() {
    __pixels = new Pixel[width][height];
    for (int i = 0; i < width; i++) {
      for (int j = 0; j < height; j++) { __pixels[i][j] = new Pixel(this, i, j); }
    }
  }
  public void setPixel(int x, int y, Color c) { __set(x, y, c); }
  public void clear(Color c) { __fill((c.red << 16) | (c.green << 8) | c.blue); }
  Color __get(int x, int y) { int p = px[y * width + x]; return new Color((p >> 16) & 255, (p >> 8) & 255, p & 255); }
  void __set(int x, int y, Color c) { __setRgb(x, y, c.red, c.green, c.blue); }
  // Packed-pixel accessors, so Pixel can touch one channel without a Color.
  int __raw(int x, int y) { return px[y * width + x]; }
  void __setRgb(int x, int y, int r, int g, int b) { px[y * width + x] = (r << 16) | (g << 8) | b; }
}

class Scene {
  java.util.ArrayList<String> __cmds = new java.util.ArrayList<String>();
  static int __soundSeq = 0;
  static int __imgSeq = 0;
  public Scene() {}
  public final int getWidth() { return 400; }
  public final int getHeight() { return 400; }
  java.util.ArrayList<String> __commands() { return __cmds; }
  private String __c(Color c) { return c.getRed() + " " + c.getGreen() + " " + c.getBlue(); }
  public final void clear(String color) { __cmds.add("clear " + __c(new Color(color))); }
  public final void clear(Color color) { __cmds.add("clear " + __c(color)); }
  public final void pause(double seconds) { __cmds.add("pause " + Math.max(seconds, 0.1)); }
  public final void playNote(int note, double seconds) { __cmds.add("note PIANO " + note + " " + seconds); }
  public final void playNote(Instrument inst, int note, double seconds) { __cmds.add("note " + inst + " " + note + " " + seconds); }
  public final void playNoteAndPause(int note, double seconds) { playNote(note, seconds); pause(seconds); }
  public final void playNoteAndPause(Instrument inst, int note, double seconds) { playNote(inst, note, seconds); pause(seconds); }
  public final void playSound(String filename) { __cmds.add("sound file " + filename); }
  // Serialize the samples to a VFS file the host reads back and plays; the log
  // just references it by id. Format (shared with the host): the sample count,
  // then that many space-separated signed 16-bit ints. See specs/EXECUTION.md.
  public final void playSound(double[] sound) {
    int id = __soundSeq;
    __soundSeq = __soundSeq + 1;
    try {
      java.io.PrintWriter w = new java.io.PrintWriter(new java.io.File("__caturra_pcm_" + id));
      StringBuilder sb = new StringBuilder();
      sb.append(sound.length);
      for (int i = 0; i < sound.length; i++) {
        double x = sound[i];
        if (x > 1.0) x = 1.0;
        if (x < -1.0) x = -1.0;
        sb.append(' ').append((int) Math.round(x * 32767.0));
      }
      w.print(sb.toString());
      w.close();
    } catch (Exception e) {}
    __cmds.add("sound pcm " + id + " " + sound.length);
  }
  public final void setStrokeWidth(double width) { __cmds.add("strokeWidth " + width); }
  public final void setStrokeColor(String color) { __cmds.add("strokeColor " + __c(new Color(color))); }
  public final void setStrokeColor(Color color) { __cmds.add("strokeColor " + __c(color)); }
  public final void setFillColor(String color) { __cmds.add("fillColor " + __c(new Color(color))); }
  public final void setFillColor(Color color) { __cmds.add("fillColor " + __c(color)); }
  public final void removeStrokeColor() { __cmds.add("strokeColor none"); }
  public final void removeFillColor() { __cmds.add("fillColor none"); }
  public final void setTextColor(String color) { __cmds.add("textColor " + __c(new Color(color))); }
  public final void setTextColor(Color color) { __cmds.add("textColor " + __c(color)); }
  public final void setTextHeight(int height) { __cmds.add("textHeight " + height); }
  public final void setTextStyle(Font font, FontStyle style) { __cmds.add("textStyle " + font + " " + style); }
  public final void drawText(String text, int x, int y) { __cmds.add("text \"" + text + "\" " + x + " " + y + " 0.0"); }
  public final void drawText(String text, int x, int y, double rotation) { __cmds.add("text \"" + text + "\" " + x + " " + y + " " + rotation); }
  public final void drawLine(int sx, int sy, int ex, int ey) { __cmds.add("line " + sx + " " + sy + " " + ex + " " + ey); }
  public final void drawRegularPolygon(int x, int y, int sides, int radius) { __cmds.add("polygon " + x + " " + y + " " + sides + " " + radius); }
  public final void drawEllipse(int x, int y, int width, int height) { __cmds.add("ellipse " + x + " " + y + " " + width + " " + height); }
  public final void drawRectangle(int x, int y, int width, int height) { __cmds.add("rectangle " + x + " " + y + " " + width + " " + height); }
  public final void drawShape(int[] points, boolean close) {
    String s = "shape";
    for (int p : points) s += " " + p;
    __cmds.add(s + " " + close);
  }
  // An in-engine Image (possibly pixel-edited) is handed to the host as a VFS
  // pixel buffer; the log references it by id so the host can draw the real thing.
  private int __writeImage(Image image) {
    int id = __imgSeq;
    __imgSeq = __imgSeq + 1;
    System.__writeImage("__caturra_img_" + id, image.width, image.height, image.px);
    return id;
  }
  public final void drawImage(Image image, int x, int y, int size) { __cmds.add("image obj " + __writeImage(image) + " " + x + " " + y + " " + size); }
  public final void drawImage(Image image, int x, int y, int size, double rotation) { __cmds.add("image obj " + __writeImage(image) + " " + x + " " + y + " " + size + " " + rotation); }
  public final void drawImage(Image image, int x, int y, int width, int height, double rotation) { __cmds.add("image obj " + __writeImage(image) + " " + x + " " + y + " " + width + " " + height + " " + rotation); }
  // Drawing an asset by name hands the file to the host renderer rather than
  // decoding 160k pixels into the VM — but the real Scene routes these through
  // `new Image(filename)`, so a name it cannot load throws. Check that the host
  // preloaded it (dimensions only, not the pixels) so a missing asset fails
  // here too instead of silently drawing nothing.
  private void __requireAsset(String filename) {
    int[] dims = System.__imageDims("__caturra_image_" + filename);
    if (dims.length != 2 || dims[0] <= 0 || dims[1] <= 0) throw new RuntimeException("FILE_NOT_FOUND");
  }
  public final void drawImage(String filename, int x, int y, int size) { __requireAsset(filename); __cmds.add("image " + filename + " " + x + " " + y + " " + size); }
  public final void drawImage(String filename, int x, int y, int size, double rotation) { __requireAsset(filename); __cmds.add("image " + filename + " " + x + " " + y + " " + size + " " + rotation); }
  public final void drawImage(String filename, int x, int y, int width, int height, double rotation) { __requireAsset(filename); __cmds.add("image " + filename + " " + x + " " + y + " " + width + " " + height + " " + rotation); }
}

class SoundLoader {
  // The host decodes bundled audio and preloads its samples into the VFS as
  // `__caturra_sound_<filename>` (same text format as playSound's output) before
  // the run, so real assets come back as real samples. When no asset was
  // provided (unknown name, or a headless run), fall back to indexable silence:
  // the lessons index by time (`sound[start * 44100]`, e.g. createClip(sound, 2,
  // 5)), so an empty buffer would throw ArrayIndexOutOfBounds. See EXECUTION.md.
  public static double[] read(String filename) {
    try {
      java.util.Scanner sc = new java.util.Scanner(new java.io.File("__caturra_sound_" + filename));
      int n = sc.nextInt();
      double[] a = new double[n];
      for (int i = 0; i < n; i++) a[i] = sc.nextInt() / 32768.0;
      sc.close();
      return a;
    } catch (Exception e) {
      return new double[44100 * 10];
    }
  }
}

class Theater {
  static void __write(Scene[] scenes) {
    try {
      java.io.PrintWriter w = new java.io.PrintWriter(new java.io.File("theater.log"));
      for (Scene s : scenes) for (String c : s.__commands()) w.println(c);
      w.close();
    } catch (Exception e) {}
  }
  public static void playScenes(Scene a) { __write(new Scene[]{a}); }
  public static void playScenes(Scene a, Scene b) { __write(new Scene[]{a, b}); }
  public static void playScenes(Scene a, Scene b, Scene c) { __write(new Scene[]{a, b, c}); }
  public static void playScenes(Scene a, Scene b, Scene c, Scene d) { __write(new Scene[]{a, b, c, d}); }
  public static void play(Scene[] scenes) { __write(scenes); }
}
