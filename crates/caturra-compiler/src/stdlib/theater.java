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
  public Color(int red, int green, int blue) { this.red = red; this.green = green; this.blue = blue; }
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
  static int[] __resolve(String s) {
    String n = s.toUpperCase();
    if (n.length() == 7 && n.charAt(0) == '#') {
      return new int[]{ __hex(n.substring(1, 3)), __hex(n.substring(3, 5)), __hex(n.substring(5, 7)) };
    }
    if (n.equals("WHITE")) return new int[]{255,255,255};
    if (n.equals("SILVER")) return new int[]{192,192,192};
    if (n.equals("GRAY")) return new int[]{128,128,128};
    if (n.equals("BLACK")) return new int[]{0,0,0};
    if (n.equals("RED")) return new int[]{255,0,0};
    if (n.equals("MAROON")) return new int[]{128,0,0};
    if (n.equals("YELLOW")) return new int[]{255,255,0};
    if (n.equals("OLIVE")) return new int[]{128,128,0};
    if (n.equals("LIME")) return new int[]{0,256,0};
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
  static int __hex(String h) {
    return Integer.parseInt(h, 16);
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
  Pixel(Image image, int x, int y) { this.image = image; this.x = x; this.y = y; }
  public int getX() { return x; }
  public int getY() { return y; }
  public Color getColor() { return image.__get(x, y); }
  public void setColor(Color c) { image.__set(x, y, c); }
  public int getRed() { return getColor().getRed(); }
  public int getGreen() { return getColor().getGreen(); }
  public int getBlue() { return getColor().getBlue(); }
  public void setRed(int v) { Color c = getColor(); image.__set(x, y, new Color(v, c.green, c.blue)); }
  public void setGreen(int v) { Color c = getColor(); image.__set(x, y, new Color(c.red, v, c.blue)); }
  public void setBlue(int v) { Color c = getColor(); image.__set(x, y, new Color(c.red, c.green, v)); }
}

class Image {
  int width, height;
  int[][] r; int[][] g; int[][] b;
  public Image(int width, int height) {
    this.width = width; this.height = height;
    r = new int[width][height]; g = new int[width][height]; b = new int[width][height];
  }
  public Image(String filename) { this(100, 100); }
  public Image(Image src) {
    this.width = src.width; this.height = src.height;
    r = new int[width][height]; g = new int[width][height]; b = new int[width][height];
    for (int i = 0; i < width; i++) for (int j = 0; j < height; j++) { r[i][j]=src.r[i][j]; g[i][j]=src.g[i][j]; b[i][j]=src.b[i][j]; }
  }
  public int getWidth() { return width; }
  public int getHeight() { return height; }
  public Pixel getPixel(int x, int y) { return new Pixel(this, x, y); }
  public void setPixel(int x, int y, Color c) { __set(x, y, c); }
  public void clear(Color c) { for (int i=0;i<width;i++) for(int j=0;j<height;j++) __set(i,j,c); }
  Color __get(int x, int y) { return new Color(r[x][y], g[x][y], b[x][y]); }
  void __set(int x, int y, Color c) { r[x][y]=c.red; g[x][y]=c.green; b[x][y]=c.blue; }
}

class Scene {
  java.util.ArrayList<String> __cmds = new java.util.ArrayList<String>();
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
  public final void playSound(String filename) { __cmds.add("sound " + filename); }
  public final void playSound(double[] sound) { __cmds.add("sound " + sound.length + " samples"); }
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
  public final void drawImage(Image image, int x, int y, int size) { __cmds.add("image " + image.getWidth() + "x" + image.getHeight() + " " + x + " " + y + " " + size); }
  public final void drawImage(Image image, int x, int y, int size, double rotation) { __cmds.add("image " + image.getWidth() + "x" + image.getHeight() + " " + x + " " + y + " " + size + " " + rotation); }
  public final void drawImage(Image image, int x, int y, int width, int height, double rotation) { __cmds.add("image " + image.getWidth() + "x" + image.getHeight() + " " + x + " " + y + " " + width + " " + height + " " + rotation); }
  public final void drawImage(String filename, int x, int y, int size) { __cmds.add("image " + filename + " " + x + " " + y + " " + size); }
  public final void drawImage(String filename, int x, int y, int size, double rotation) { __cmds.add("image " + filename + " " + x + " " + y + " " + size + " " + rotation); }
  public final void drawImage(String filename, int x, int y, int width, int height, double rotation) { __cmds.add("image " + filename + " " + x + " " + y + " " + width + " " + height + " " + rotation); }
}

class SoundLoader {
  // Audio decoding needs real assets (Phase 2); return silence so
  // sound levels compile and run headless.
  public static double[] read(String filename) { return new double[0]; }
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
