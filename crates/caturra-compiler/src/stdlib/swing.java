// Bundled clean-room javax.swing + java.awt subset (accessible DOM Swing).
// Auto-injected when a source imports javax.swing or java.awt.
//
// Phase 1 (declarative render): components build a retained tree; calling
// frame.setVisible(true) serializes that tree to the VFS file "swing.json".
// The playground replays it into real, accessible HTML (buttons, inputs,
// labels) so keyboard navigation and screen-reader semantics work without a
// canvas. Event listeners / interactivity are Phase 2 — see docs. Only the
// javax.swing / java.awt classes registered in imports.rs are public.

// ----- java.awt -----

class Color {
  int __r, __g, __b;
  public Color(int r, int g, int b) { this.__r = r; this.__g = g; this.__b = b; }
  // A packed 0xRRGGBB int (alpha byte ignored).
  public Color(int rgb) { __r = (rgb >> 16) & 255; __g = (rgb >> 8) & 255; __b = rgb & 255; }
  public int getRed() { return __r; }
  public int getGreen() { return __g; }
  public int getBlue() { return __b; }
  // Opaque ARGB, as java.awt.Color.getRGB (top byte 0xFF).
  public int getRGB() { return (255 << 24) | (__r << 16) | (__g << 8) | __b; }
  // A darker/brighter shade, following java.awt.Color's factor-0.7 algorithm.
  public Color darker() {
    return new Color(Math.max((int) (__r * 0.7), 0), Math.max((int) (__g * 0.7), 0),
        Math.max((int) (__b * 0.7), 0));
  }
  public Color brighter() {
    int r = __r, g = __g, b = __b;
    int i = 3; // (int)(1/(1-0.7))
    if (r == 0 && g == 0 && b == 0) return new Color(i, i, i);
    if (r > 0 && r < i) r = i;
    if (g > 0 && g < i) g = i;
    if (b > 0 && b < i) b = i;
    return new Color(Math.min((int) (r / 0.7), 255), Math.min((int) (g / 0.7), 255),
        Math.min((int) (b / 0.7), 255));
  }
  public static final Color WHITE = new Color(255, 255, 255);
  public static final Color LIGHT_GRAY = new Color(192, 192, 192);
  public static final Color GRAY = new Color(128, 128, 128);
  public static final Color DARK_GRAY = new Color(64, 64, 64);
  public static final Color BLACK = new Color(0, 0, 0);
  public static final Color RED = new Color(255, 0, 0);
  public static final Color PINK = new Color(255, 175, 175);
  public static final Color ORANGE = new Color(255, 200, 0);
  public static final Color YELLOW = new Color(255, 255, 0);
  public static final Color GREEN = new Color(0, 255, 0);
  public static final Color MAGENTA = new Color(255, 0, 255);
  public static final Color CYAN = new Color(0, 255, 255);
  public static final Color BLUE = new Color(0, 0, 255);
}

class Dimension {
  public int width;
  public int height;
  public Dimension(int width, int height) { this.width = width; this.height = height; }
}

// javax.swing.SwingConstants: shared alignment/position constants (the values
// components like JLabel and JTextField also expose, e.g. JLabel.CENTER).
class SwingConstants {
  public static final int CENTER = 0;
  public static final int TOP = 1;
  public static final int LEFT = 2;
  public static final int BOTTOM = 3;
  public static final int RIGHT = 4;
  public static final int NORTH = 1;
  public static final int EAST = 3;
  public static final int SOUTH = 5;
  public static final int WEST = 7;
  public static final int LEADING = 10;
  public static final int TRAILING = 11;
}

// java.awt.Font: a logical font for custom painting (Graphics.setFont). The
// logical family names (SansSerif / Serif / Monospaced / Dialog) map to CSS
// families in the renderer; the style is PLAIN / BOLD / ITALIC (bit flags, so
// BOLD | ITALIC is allowed, matching real java.awt.Font).
class Font {
  public static final int PLAIN = 0;
  public static final int BOLD = 1;
  public static final int ITALIC = 2;

  String __name;
  int __style;
  int __size;

  public Font(String name, int style, int size) { __name = name; __style = style; __size = size; }
  public String getName() { return __name; }
  public String getFontName() { return __name; }
  public String getFamily() { return __name; }
  public int getStyle() { return __style; }
  public int getSize() { return __size; }
  public boolean isPlain() { return __style == PLAIN; }
  public boolean isBold() { return (__style & BOLD) != 0; }
  public boolean isItalic() { return (__style & ITALIC) != 0; }
}

// A component border (javax.swing.border.Border), built by BorderFactory. A
// single class tagged by __type carries every variant's fields; the renderer
// styles the element from the serialized descriptor.
class Border {
  // "line" | "empty" | "titled" | "etched" | "matte" | "bevel" | "compound"
  String __type = "line";
  Color __color = null;
  int __thickness = 1;
  String __title = null;
  int __top = 0, __left = 0, __bottom = 0, __right = 0;
  boolean __raised = false;   // bevel: raised vs lowered
  Border __inner = null;      // compound inside, or a titled border's frame
  Border __outer = null;      // compound outside

  String __json() {
    StringBuilder s = new StringBuilder("{\"type\":\"");
    s.append(__type).append("\",\"thickness\":").append(__thickness);
    if (__color != null) {
      s.append(",\"color\":\"").append(__color.__r).append(",").append(__color.__g)
          .append(",").append(__color.__b).append("\"");
    }
    if (__title != null) s.append(",\"title\":\"").append(Component.__esc(__title)).append("\"");
    s.append(",\"insets\":\"").append(__top).append(",").append(__left)
        .append(",").append(__bottom).append(",").append(__right).append("\"");
    if (__type.equals("bevel")) s.append(",\"raised\":").append(__raised);
    if (__outer != null) s.append(",\"outer\":").append(__outer.__json());
    if (__inner != null) s.append(",\"inner\":").append(__inner.__json());
    return s.append("}").toString();
  }
}

// The concrete border subtypes are also constructible directly (some textbooks
// use `new LineBorder(...)` rather than the factory); each sets its tag + fields
// on the inherited Border state.
class LineBorder extends Border {
  public LineBorder(Color color) { __type = "line"; __color = color; __thickness = 1; }
  public LineBorder(Color color, int thickness) { __type = "line"; __color = color; __thickness = thickness; }
}

class EmptyBorder extends Border {
  public EmptyBorder(int top, int left, int bottom, int right) {
    __type = "empty"; __top = top; __left = left; __bottom = bottom; __right = right;
  }
}

// A per-side coloured border (asymmetric thickness).
class MatteBorder extends Border {
  public MatteBorder(int top, int left, int bottom, int right, Color color) {
    __type = "matte"; __top = top; __left = left; __bottom = bottom; __right = right; __color = color;
  }
}

class EtchedBorder extends Border {
  public EtchedBorder() { __type = "etched"; __thickness = 2; }
}

// A raised or lowered 3D bevel.
class BevelBorder extends Border {
  public static final int RAISED = 0;
  public static final int LOWERED = 1;
  public BevelBorder(int type) { __type = "bevel"; __thickness = 2; __raised = (type == RAISED); }
}

class TitledBorder extends Border {
  public TitledBorder(String title) { __type = "titled"; __title = title; }
  // Frames with `border` (instead of the default etched line) under the caption.
  public TitledBorder(Border border, String title) { __type = "titled"; __title = title; __inner = border; }
}

// Nests one border inside another (classically a line outside + empty padding
// inside).
class CompoundBorder extends Border {
  public CompoundBorder(Border outside, Border inside) {
    __type = "compound"; __outer = outside; __inner = inside;
  }
}

// javax.swing.BorderFactory: the shared factory delegating to the subtypes.
class BorderFactory {
  public static Border createLineBorder(Color color) { return new LineBorder(color); }
  public static Border createLineBorder(Color color, int thickness) { return new LineBorder(color, thickness); }
  public static Border createEmptyBorder(int top, int left, int bottom, int right) {
    return new EmptyBorder(top, left, bottom, right);
  }
  public static Border createMatteBorder(int top, int left, int bottom, int right, Color color) {
    return new MatteBorder(top, left, bottom, right, color);
  }
  public static Border createTitledBorder(String title) { return new TitledBorder(title); }
  public static Border createTitledBorder(Border border, String title) { return new TitledBorder(border, title); }
  public static Border createEtchedBorder() { return new EtchedBorder(); }
  public static Border createBevelBorder(int type) { return new BevelBorder(type); }
  public static Border createRaisedBevelBorder() { return new BevelBorder(BevelBorder.RAISED); }
  public static Border createLoweredBevelBorder() { return new BevelBorder(BevelBorder.LOWERED); }
  public static Border createCompoundBorder(Border outside, Border inside) {
    return new CompoundBorder(outside, inside);
  }
}

interface LayoutManager {
  String __desc();
}

// java.awt.Insets: the margin around a component in its cell.
class Insets {
  public int top, left, bottom, right;
  public Insets(int top, int left, int bottom, int right) {
    this.top = top; this.left = left; this.bottom = bottom; this.right = right;
  }
}

// java.awt.GridBagConstraints: where and how a component sits in a GridBagLayout
// grid. Fields are set directly (gbc.gridx = 0; gbc.fill = ...), so they're
// public. Copied on add() (real Swing does too) so a reused constraints object
// doesn't retroactively change earlier components.
class GridBagConstraints {
  public static final int RELATIVE = -1;
  public static final int REMAINDER = 0;
  public static final int NONE = 0;
  public static final int BOTH = 1;
  public static final int HORIZONTAL = 2;
  public static final int VERTICAL = 3;
  public static final int CENTER = 10;
  public static final int NORTH = 11;
  public static final int NORTHEAST = 12;
  public static final int EAST = 13;
  public static final int SOUTHEAST = 14;
  public static final int SOUTH = 15;
  public static final int SOUTHWEST = 16;
  public static final int WEST = 17;
  public static final int NORTHWEST = 18;

  public int gridx = RELATIVE;
  public int gridy = RELATIVE;
  public int gridwidth = 1;
  public int gridheight = 1;
  public double weightx = 0;
  public double weighty = 0;
  public int anchor = CENTER;
  public int fill = NONE;
  public Insets insets = new Insets(0, 0, 0, 0);
  public int ipadx = 0;
  public int ipady = 0;

  public GridBagConstraints() {}
  public GridBagConstraints(int gridx, int gridy, int gridwidth, int gridheight,
      double weightx, double weighty, int anchor, int fill, Insets insets, int ipadx, int ipady) {
    this.gridx = gridx; this.gridy = gridy; this.gridwidth = gridwidth; this.gridheight = gridheight;
    this.weightx = weightx; this.weighty = weighty; this.anchor = anchor; this.fill = fill;
    this.insets = insets; this.ipadx = ipadx; this.ipady = ipady;
  }

  GridBagConstraints __copy() {
    Insets i = new Insets(insets.top, insets.left, insets.bottom, insets.right);
    return new GridBagConstraints(gridx, gridy, gridwidth, gridheight, weightx, weighty,
        anchor, fill, i, ipadx, ipady);
  }
  String __json() {
    return "{\"gridx\":" + gridx + ",\"gridy\":" + gridy + ",\"gridwidth\":" + gridwidth
        + ",\"gridheight\":" + gridheight + ",\"weightx\":" + weightx + ",\"weighty\":" + weighty
        + ",\"anchor\":" + anchor + ",\"fill\":" + fill
        + ",\"insets\":\"" + insets.top + "," + insets.left + "," + insets.bottom + "," + insets.right + "\"}";
  }
}

class GridBagLayout implements LayoutManager {
  public GridBagLayout() {}
  public String __desc() { return "gridbag"; }
}

class FlowLayout implements LayoutManager {
  public FlowLayout() {}
  public String __desc() { return "flow"; }
}

class GridLayout implements LayoutManager {
  int __rows, __cols;
  public GridLayout(int rows, int cols) { this.__rows = rows; this.__cols = cols; }
  public String __desc() { return "grid " + __rows + " " + __cols; }
}

class BorderLayout implements LayoutManager {
  public BorderLayout() {}
  public String __desc() { return "border"; }
  public static final String NORTH = "North";
  public static final String SOUTH = "South";
  public static final String EAST = "East";
  public static final String WEST = "West";
  public static final String CENTER = "Center";
}

// Stacks components along one axis (X row, Y column). The target container is
// accepted (real API) but the axis is all the renderer needs; spacing comes
// from Box struts/glue, not a default gap.
class BoxLayout implements LayoutManager {
  int __axis;
  public static final int X_AXIS = 0;
  public static final int Y_AXIS = 1;
  public static final int LINE_AXIS = 2;
  public static final int PAGE_AXIS = 3;
  public BoxLayout(Container target, int axis) { __axis = axis; }
  public String __desc() { return "box " + __axis; }
}

// An invisible spacer produced by the Box factory methods (a strut is fixed
// size; glue stretches to push its neighbors apart).
class __Strut extends Component {
  int __w = 0;
  int __h = 0;
  boolean __glue = false;
  String __json() {
    return "{\"type\":\"strut\",\"w\":" + __w + ",\"h\":" + __h + ",\"glue\":" + __glue
        + "," + __commonJson() + "}";
  }
}

// Box: factory methods for BoxLayout spacers.
class Box {
  static __Strut __make(int w, int h, boolean glue) {
    __Strut s = new __Strut();
    s.__w = w;
    s.__h = h;
    s.__glue = glue;
    return s;
  }
  public static Component createVerticalStrut(int height) { return __make(0, height, false); }
  public static Component createHorizontalStrut(int width) { return __make(width, 0, false); }
  public static Component createRigidArea(Dimension d) { return __make(d.width, d.height, false); }
  public static Component createVerticalGlue() { return __make(0, 0, true); }
  public static Component createHorizontalGlue() { return __make(0, 0, true); }
}

// ----- Component model -----

class Component {
  static int __counter = 0;
  static int __nextId() { int n = __counter; __counter = __counter + 1; return n; }

  String __cid;
  boolean __enabled = true;
  boolean __visible = true;
  String __ttip = null;
  Color __bg = null;
  Color __fg = null;
  Font __font = null;
  Border __border = null;
  // The BorderLayout region ("North".."Center") this component was added with,
  // or null when it carries no layout constraint.
  String __region = null;
  // The GridBagConstraints this component was added with (a copy), when its
  // parent uses a GridBagLayout.
  GridBagConstraints __gbc = null;
  // Explicit bounds (setBounds/setSize/setLocation). Honored only under a null
  // (absolute) layout — a managed layout ignores them, as in real Swing.
  int __bx = 0, __by = 0, __bw = 0, __bh = 0;
  boolean __hasBounds = false;
  // Size hints (setPreferredSize / setMinimumSize / setMaximumSize) and the
  // horizontal text alignment (SwingConstants); -1 = unset.
  Dimension __prefSize = null, __minSize = null, __maxSize = null;
  int __halign = -1;
  // Accessibility (via getAccessibleContext): an explicit name/description for
  // assistive tech, overriding a widget's default (text-content) name.
  String __accName = null, __accDesc = null;
  AccessibleContext __accCtx = null;
  // A text component's Document (getDocument) carries its DocumentListener,
  // which fires on every edit. __lastText tracks the prior value so __onDoc can
  // tell an insert from a remove.
  Document __document = null;
  String __lastText = "";
  // When built from an Action, the component's enabled state tracks the action
  // LIVE (re-read each serialization), so action.setEnabled propagates even
  // after the component exists — real Swing does this via a PropertyChangeListener.
  Action __action = null;

  Component() { __cid = "c" + __nextId(); }

  // The text Document — the model behind a text field/area. Lazily created.
  public Document getDocument() {
    if (__document == null) __document = new Document(this);
    return __document;
  }
  // Fires the DocumentListener after the text synced from the host; overridden
  // by the text components (which hold the text to diff).
  void __onDoc() {}

  // The component's AccessibleContext — the Swing accessibility API. Lazily
  // created; setAccessibleName / setAccessibleDescription flow to the renderer
  // as aria-label / aria-description.
  public AccessibleContext getAccessibleContext() {
    if (__accCtx == null) __accCtx = new AccessibleContext(this);
    return __accCtx;
  }

  public void setEnabled(boolean enabled) { __enabled = enabled; }
  public boolean isEnabled() { return __action != null ? __action.isEnabled() : __enabled; }
  public void setVisible(boolean visible) { __visible = visible; }
  public boolean isVisible() { return __visible; }
  public void setFocusable(boolean focusable) {}
  // Move keyboard focus here. Records a one-shot request the renderer honors on
  // the next paint (a sequence lets the same component be re-requested).
  public void requestFocus() { __SwingRuntime.__requestFocus(__cid); }
  public boolean requestFocusInWindow() { __SwingRuntime.__requestFocus(__cid); return true; }
  public void setToolTipText(String text) { __ttip = text; }
  public void setBackground(Color c) { __bg = c; }
  public void setForeground(Color c) { __fg = c; }
  public void setFont(Font f) { __font = f; }
  public Font getFont() { return __font; }
  public void setBorder(Border b) { __border = b; }
  public Border getBorder() { return __border; }
  public void setBounds(int x, int y, int width, int height) {
    __bx = x; __by = y; __bw = width; __bh = height; __hasBounds = true;
  }
  public void setLocation(int x, int y) { __bx = x; __by = y; __hasBounds = true; }
  public void setSize(int width, int height) { __bw = width; __bh = height; __hasBounds = true; }
  public int getX() { return __bx; }
  public int getY() { return __by; }
  public int getWidth() { return __bw; }
  public int getHeight() { return __bh; }
  public void setPreferredSize(Dimension d) { __prefSize = d; }
  public Dimension getPreferredSize() { return __prefSize == null ? new Dimension(0, 0) : __prefSize; }
  public void setMinimumSize(Dimension d) { __minSize = d; }
  public Dimension getMinimumSize() { return __minSize == null ? new Dimension(0, 0) : __minSize; }
  public void setMaximumSize(Dimension d) { __maxSize = d; }
  public Dimension getMaximumSize() { return __maxSize == null ? new Dimension(0, 0) : __maxSize; }

  // JSON escaping. Uses a StringBuilder (not `out += ...`): appending to a
  // String in a loop is O(n^2), and this runs over the whole serialized paint
  // string every frame, so it dominated the cost of a growing drawing.
  static String __esc(String s) {
    StringBuilder out = new StringBuilder();
    for (int i = 0; i < s.length(); i++) {
      String ch = s.substring(i, i + 1);
      if (ch.equals("\"")) out.append("\\\"");
      else if (ch.equals("\\")) out.append("\\\\");
      else if (ch.equals("\n")) out.append("\\n");
      else if (ch.equals("\t")) out.append("\\t");
      else out.append(ch);
    }
    return out.toString();
  }

  // The shared trailing fields every node carries (no braces, no leading comma).
  String __commonJson() {
    // An action-bound component's enabled tracks its Action live.
    boolean enabled = __action != null ? __action.isEnabled() : __enabled;
    String s = "\"id\":\"" + __cid + "\",\"enabled\":" + enabled;
    // Only widgets with a listener dispatch on interaction; the renderer
    // wires a control's native event to the VM only when this is set, so an
    // input read at submit-time (no listener) doesn't round-trip on every key.
    if (__listens()) s += ",\"listens\":true";
    if (__document != null && __document.__listener != null) s += ",\"docListen\":true";
    if (!__visible) s += ",\"hidden\":true";
    if (__accName != null) s += ",\"accName\":\"" + Component.__esc(__accName) + "\"";
    if (__accDesc != null) s += ",\"accDesc\":\"" + Component.__esc(__accDesc) + "\"";
    if (__ttip != null) s += ",\"tooltip\":\"" + Component.__esc(__ttip) + "\"";
    if (__bg != null) s += ",\"bg\":\"" + __bg.__r + "," + __bg.__g + "," + __bg.__b + "\"";
    if (__fg != null) s += ",\"fg\":\"" + __fg.__r + "," + __fg.__g + "," + __fg.__b + "\"";
    // Font: "<style> <size> <family>" (family last so a spaced name survives).
    if (__font != null) {
      s += ",\"font\":\"" + __font.__style + " " + __font.__size + " " + Component.__esc(__font.__name) + "\"";
    }
    if (__border != null) s += ",\"border\":" + __border.__json();
    if (__region != null) s += ",\"region\":\"" + Component.__esc(__region) + "\"";
    if (__gbc != null) s += ",\"gbc\":" + __gbc.__json();
    // Absolute bounds "x,y,w,h" (honored by the renderer under a null layout).
    if (__hasBounds) s += ",\"bounds\":\"" + __bx + "," + __by + "," + __bw + "," + __bh + "\"";
    if (__prefSize != null) s += ",\"psize\":\"" + __prefSize.width + "," + __prefSize.height + "\"";
    if (__minSize != null) s += ",\"minsize\":\"" + __minSize.width + "," + __minSize.height + "\"";
    if (__maxSize != null) s += ",\"maxsize\":\"" + __maxSize.width + "," + __maxSize.height + "\"";
    if (__halign >= 0) s += ",\"halign\":" + __halign;
    return s;
  }

  // Overridden by every concrete component; returns one JSON object.
  String __json() { return "{\"type\":\"component\"," + __commonJson() + "}"; }

  // Phase 2 event hooks. __setFromHost syncs a value the user changed in
  // the DOM (text typed, box toggled) before dispatch; __onEvent fires the
  // component's listener when it is the activated control; __listens reports
  // whether it has one. All no-op/false by default; widgets override them.
  void __setFromHost(String value) {}
  void __onEvent() {}
  void __onMouse(int x, int y) {}
  void __onDrag(int x, int y) {}
  void __onKey(int type, int code, char ch) {}
  boolean __listens() { return false; }
}

// javax.accessibility.AccessibleContext: the Swing accessibility handle. Setting
// the accessible name/description on any component surfaces it to assistive tech
// (the renderer maps them to aria-label / aria-description).
class AccessibleContext {
  Component __owner;
  AccessibleContext(Component owner) { __owner = owner; }
  public void setAccessibleName(String name) { __owner.__accName = name; }
  public String getAccessibleName() { return __owner.__accName; }
  public void setAccessibleDescription(String description) { __owner.__accDesc = description; }
  public String getAccessibleDescription() { return __owner.__accDesc; }
}

class Container extends Component {
  java.util.ArrayList<Component> __kids = new java.util.ArrayList<Component>();
  LayoutManager __layout = null;
  // setLayout(null) requests absolute positioning (children placed by setBounds).
  // Distinct from never calling setLayout (which defaults to flow).
  boolean __absolute = false;

  public void add(Component c) { __kids.add(c); __SwingRuntime.__register(c); }
  // A layout constraint: a GridBagConstraints (copied, so a reused one doesn't
  // retroactively change earlier children) for a GridBagLayout, else a
  // BorderLayout region string (BorderLayout.NORTH).
  public void add(Component c, Object constraints) {
    if (constraints instanceof GridBagConstraints) c.__gbc = ((GridBagConstraints) constraints).__copy();
    else if (constraints != null) c.__region = "" + constraints;
    add(c);
  }
  public void remove(Component c) { __kids.remove(c); }
  public void remove(int index) { __kids.remove(index); }
  public void removeAll() { __kids.clear(); }
  public int getComponentCount() { return __kids.size(); }
  public Component getComponent(int index) { return __kids.get(index); }
  // The layout re-runs every event-loop tick, so these need do nothing.
  public void revalidate() {}
  public void validate() {}
  public void repaint() {}
  public void setLayout(LayoutManager m) { __layout = m; __absolute = (m == null); }

  String __layoutJson() {
    if (__absolute) return "\"none\"";
    if (__layout == null) return "\"flow\"";
    return "\"" + __layout.__desc() + "\"";
  }

  String __kidsJson() {
    StringBuilder s = new StringBuilder("[");
    for (int i = 0; i < __kids.size(); i++) {
      if (i > 0) s.append(",");
      s.append(__kids.get(i).__json());
    }
    return s.append("]").toString();
  }
}

// ----- javax.swing widgets -----

// java.awt.Graphics: a recorder. A custom JPanel's paintComponent draws into
// it; the commands are replayed onto a <canvas> by the renderer. Coordinates
// and the current color follow java.awt.Graphics (draw* strokes, fill* fills).
class Graphics {
  java.util.ArrayList<String> __cmds = new java.util.ArrayList<String>();
  Font __font = null;
  public void setColor(Color c) { __cmds.add("setColor " + c.__r + " " + c.__g + " " + c.__b); }
  public void fillRect(int x, int y, int w, int h) { __cmds.add("fillRect " + x + " " + y + " " + w + " " + h); }
  public void drawRect(int x, int y, int w, int h) { __cmds.add("drawRect " + x + " " + y + " " + w + " " + h); }
  public void fillOval(int x, int y, int w, int h) { __cmds.add("fillOval " + x + " " + y + " " + w + " " + h); }
  public void drawOval(int x, int y, int w, int h) { __cmds.add("drawOval " + x + " " + y + " " + w + " " + h); }
  public void drawLine(int x1, int y1, int x2, int y2) { __cmds.add("drawLine " + x1 + " " + y1 + " " + x2 + " " + y2); }
  public void drawString(String s, int x, int y) { __cmds.add("drawString \"" + s + "\" " + x + " " + y); }
  // The current font applies to subsequent drawString calls. The family name is
  // the last field so the renderer can take the rest of the line (a name may
  // contain spaces).
  public void setFont(Font f) { __font = f; __cmds.add("setFont " + f.__style + " " + f.__size + " " + f.__name); }
  public Font getFont() { return __font; }
  // Polygons: nPoints (x,y) pairs, serialized flat after the op.
  public void fillPolygon(int[] xs, int[] ys, int n) { __cmds.add(__poly("fillPolygon", xs, ys, n)); }
  public void drawPolygon(int[] xs, int[] ys, int n) { __cmds.add(__poly("drawPolygon", xs, ys, n)); }
  String __poly(String op, int[] xs, int[] ys, int n) {
    StringBuilder b = new StringBuilder(op);
    for (int i = 0; i < n; i++) b.append(" ").append(xs[i]).append(" ").append(ys[i]);
    return b.toString();
  }
  // Arcs bounded by (x,y,w,h); angles in degrees, 0 at 3 o'clock, CCW positive
  // (java.awt.Graphics). fillArc is a pie slice; drawArc strokes the open arc.
  public void fillArc(int x, int y, int w, int h, int start, int arc) {
    __cmds.add("fillArc " + x + " " + y + " " + w + " " + h + " " + start + " " + arc);
  }
  public void drawArc(int x, int y, int w, int h, int start, int arc) {
    __cmds.add("drawArc " + x + " " + y + " " + w + " " + h + " " + start + " " + arc);
  }
  boolean __empty() { return __cmds.size() == 0; }
  String __joined() {
    // StringBuilder, not `out += ...`: appending to a String in a loop is
    // O(n^2) (each += copies the whole accumulator), which made repainting a
    // growing drawing quadratic per frame.
    StringBuilder out = new StringBuilder();
    for (int i = 0; i < __cmds.size(); i++) {
      if (i > 0) out.append("\n");
      out.append(__cmds.get(i));
    }
    return out.toString();
  }
}

// java.awt.BasicStroke: a pen width and end/join style for Graphics2D drawing.
// The width is a float (real API), so `new BasicStroke(3)` and `2.5f` both work.
class BasicStroke {
  public static final int CAP_BUTT = 0;
  public static final int CAP_ROUND = 1;
  public static final int CAP_SQUARE = 2;
  public static final int JOIN_MITER = 0;
  public static final int JOIN_ROUND = 1;
  public static final int JOIN_BEVEL = 2;

  float __width;
  int __cap;
  int __join;

  public BasicStroke() { __width = 1; __cap = CAP_SQUARE; __join = JOIN_MITER; }
  public BasicStroke(float width) { __width = width; __cap = CAP_SQUARE; __join = JOIN_MITER; }
  public BasicStroke(float width, int cap, int join) { __width = width; __cap = cap; __join = join; }
  public float getLineWidth() { return __width; }
  public int getEndCap() { return __cap; }
  public int getLineJoin() { return __join; }
}

// java.awt.Graphics2D: the concrete Graphics that paintComponent receives (a
// cast `(Graphics2D) g` succeeds — the panel builds one). Adds stroke control;
// the width/cap/join apply to subsequent draw* (stroke) operations.
class Graphics2D extends Graphics {
  public void setStroke(BasicStroke s) {
    // Round the float width to an int for the command stream (sub-pixel widths
    // aren't meaningful on the integer canvas grid these drawings use).
    int w = (int) (s.__width + 0.5);
    __cmds.add("setStroke " + w + " " + s.__cap + " " + s.__join);
  }
}

class JPanel extends Container {
  int __pw = 200;
  int __ph = 150;
  MouseListener __mouseListener = null;
  MouseMotionListener __motionListener = null;
  KeyListener __keyListener = null;
  public JPanel() {}
  public JPanel(LayoutManager m) { __layout = m; }
  public void setPreferredSize(Dimension d) { __pw = d.width; __ph = d.height; }
  // Explicit bounds (setBounds under a null layout) win over the preferred size.
  public int getWidth() { return __hasBounds ? __bw : __pw; }
  public int getHeight() { return __hasBounds ? __bh : __ph; }
  public void addMouseListener(MouseListener l) { __mouseListener = l; __SwingRuntime.__interactive = true; }
  public void addMouseMotionListener(MouseMotionListener l) { __motionListener = l; __SwingRuntime.__interactive = true; }
  public void addKeyListener(KeyListener l) { __keyListener = l; __SwingRuntime.__interactive = true; }
  boolean __listens() {
    return __mouseListener != null || __motionListener != null || __keyListener != null;
  }
  void __onMouse(int x, int y) {
    if (__mouseListener != null) {
      // A real click fires press, then release, then clicked.
      MouseEvent e = new MouseEvent(this, x, y);
      __mouseListener.mousePressed(e);
      __mouseListener.mouseReleased(e);
      __mouseListener.mouseClicked(e);
    }
  }
  void __onDrag(int x, int y) {
    if (__motionListener != null) __motionListener.mouseDragged(new MouseEvent(this, x, y));
  }
  void __onKey(int type, int code, char ch) {
    if (__keyListener == null) return;
    KeyEvent e = new KeyEvent(this, code, ch);
    if (type == 0) {
      // keydown: keyPressed, then keyTyped for a printable character (matching
      // real Java's ordering) — both delivered in one host round trip.
      __keyListener.keyPressed(e);
      if (ch != KeyEvent.CHAR_UNDEFINED && ch >= 32) __keyListener.keyTyped(e);
    } else {
      __keyListener.keyReleased(e);
    }
  }
  // The event loop re-renders (and so re-paints) after every event, so a
  // repaint request is implicit; this exists so student code compiles.
  public void repaint() {}
  // Overridden by a custom drawing panel; the base draws nothing. Called
  // during serialization so the recorded commands go to the canvas.
  public void paintComponent(Graphics g) {}
  String __json() {
    // A Graphics2D (the real paintComponent argument type) so a subclass can
    // cast `(Graphics2D) g` to set the stroke; it upcasts to the Graphics param.
    Graphics g = new Graphics2D();
    paintComponent(g); // virtual dispatch: a subclass draws into g
    String paint = "";
    if (!g.__empty()) {
      paint = ",\"paint\":\"" + Component.__esc(g.__joined()) + "\",\"pw\":" + __pw
          + ",\"ph\":" + __ph;
    }
    // Which input events to wire: mouse (click), drag (motion), key (keyboard).
    String pointer = "";
    if (__mouseListener != null) pointer += ",\"mouse\":true";
    if (__motionListener != null) pointer += ",\"drag\":true";
    if (__keyListener != null) pointer += ",\"key\":true";
    return "{\"type\":\"panel\",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson() + paint + pointer + "," + __commonJson() + "}";
  }
}

// Wraps one component in a fixed-size scrolling viewport. The wrapped view is
// registered here (it is not added to a Container), so its events dispatch and
// its value syncs.
class JScrollPane extends Component {
  Component __view = null;
  int __pw = 200;
  int __ph = 150;
  int __hpolicy = 30; // HORIZONTAL_SCROLLBAR_AS_NEEDED
  int __vpolicy = 20; // VERTICAL_SCROLLBAR_AS_NEEDED
  public static final int HORIZONTAL_SCROLLBAR_AS_NEEDED = 30;
  public static final int HORIZONTAL_SCROLLBAR_NEVER = 31;
  public static final int HORIZONTAL_SCROLLBAR_ALWAYS = 32;
  public static final int VERTICAL_SCROLLBAR_AS_NEEDED = 20;
  public static final int VERTICAL_SCROLLBAR_NEVER = 21;
  public static final int VERTICAL_SCROLLBAR_ALWAYS = 22;
  public JScrollPane() {}
  public JScrollPane(Component view) { setViewportView(view); }
  public void setViewportView(Component view) {
    __view = view;
    if (view != null) __SwingRuntime.__register(view);
  }
  public void setPreferredSize(Dimension d) { __pw = d.width; __ph = d.height; }
  public void setHorizontalScrollBarPolicy(int policy) { __hpolicy = policy; }
  public void setVerticalScrollBarPolicy(int policy) { __vpolicy = policy; }
  String __json() {
    String view = __view == null ? "null" : __view.__json();
    return "{\"type\":\"scrollpane\",\"view\":" + view + ",\"pw\":" + __pw + ",\"ph\":" + __ph
        + ",\"hpolicy\":" + __hpolicy + ",\"vpolicy\":" + __vpolicy + "," + __commonJson() + "}";
  }
}

// A tabbed container: each tab has a title and a component; only the selected
// one shows. Switching tabs is always interactive (it round-trips to update the
// selection and fire any ChangeListener), so a JTabbedPane makes the app live.
class JTabbedPane extends Component {
  public static final int TOP = 1;
  public static final int LEFT = 2;
  public static final int BOTTOM = 3;
  public static final int RIGHT = 4;

  int __placement = TOP;
  java.util.ArrayList<String> __titles = new java.util.ArrayList<String>();
  java.util.ArrayList<Component> __tabs = new java.util.ArrayList<Component>();
  int __selected = 0;
  ChangeListener __listener = null;

  public JTabbedPane() { __SwingRuntime.__interactive = true; }
  public JTabbedPane(int placement) { __placement = placement; __SwingRuntime.__interactive = true; }

  public void addTab(String title, Component c) {
    __titles.add(title);
    __tabs.add(c);
    if (c != null) __SwingRuntime.__register(c);
  }
  public void add(String title, Component c) { addTab(title, c); }
  public int getTabCount() { return __tabs.size(); }
  public int getSelectedIndex() { return __selected; }
  public void setSelectedIndex(int i) { __selected = i; }
  public String getTitleAt(int i) { return __titles.get(i); }
  public void setTitleAt(int i, String title) { __titles.set(i, title); }
  public Component getComponentAt(int i) { return __tabs.get(i); }
  public void setTabPlacement(int placement) { __placement = placement; }
  public void addChangeListener(ChangeListener l) { __listener = l; __SwingRuntime.__interactive = true; }

  boolean __listens() { return __listener != null; }
  void __setFromHost(String value) { __selected = Integer.parseInt(value); }
  void __onEvent() { if (__listener != null) __listener.stateChanged(new ChangeEvent(this)); }

  String __json() {
    StringBuilder tabs = new StringBuilder("[");
    for (int i = 0; i < __tabs.size(); i++) {
      if (i > 0) tabs.append(",");
      tabs.append("{\"title\":\"").append(Component.__esc(__titles.get(i)))
          .append("\",\"component\":").append(__tabs.get(i).__json()).append("}");
    }
    tabs.append("]");
    return "{\"type\":\"tabbedpane\",\"placement\":" + __placement
        + ",\"selectedIndex\":" + __selected
        + ",\"tabs\":" + tabs.toString() + "," + __commonJson() + "}";
  }
}

// Two components separated by a draggable divider. HORIZONTAL_SPLIT places them
// side by side (left/right) with a vertical divider; VERTICAL_SPLIT stacks them
// (top/bottom). Dragging the divider is client-side (no VM round trip); a set
// divider location seeds the initial split.
class JSplitPane extends Component {
  public static final int HORIZONTAL_SPLIT = 1;
  public static final int VERTICAL_SPLIT = 0;

  int __orientation = HORIZONTAL_SPLIT;
  Component __left = null;   // left (horizontal) / top (vertical)
  Component __right = null;  // right (horizontal) / bottom (vertical)
  int __divider = -1;        // divider location in px; <=0 = even split

  public JSplitPane() {}
  public JSplitPane(int orientation) { __orientation = orientation; }
  public JSplitPane(int orientation, Component left, Component right) {
    __orientation = orientation;
    setLeftComponent(left);
    setRightComponent(right);
  }

  public void setLeftComponent(Component c) { __left = c; if (c != null) __SwingRuntime.__register(c); }
  public void setRightComponent(Component c) { __right = c; if (c != null) __SwingRuntime.__register(c); }
  public void setTopComponent(Component c) { setLeftComponent(c); }
  public void setBottomComponent(Component c) { setRightComponent(c); }
  public Component getLeftComponent() { return __left; }
  public Component getRightComponent() { return __right; }
  public Component getTopComponent() { return __left; }
  public Component getBottomComponent() { return __right; }
  public void setOrientation(int o) { __orientation = o; }
  public int getOrientation() { return __orientation; }
  public void setDividerLocation(int loc) { __divider = loc; }
  public int getDividerLocation() { return __divider; }
  // Accepted for source compatibility; they don't change the flex layout.
  public void setResizeWeight(double w) {}
  public void setContinuousLayout(boolean b) {}
  public void setOneTouchExpandable(boolean b) {}
  public void setDividerSize(int s) {}

  String __json() {
    String left = __left == null ? "null" : __left.__json();
    String right = __right == null ? "null" : __right.__json();
    return "{\"type\":\"splitpane\",\"orientation\":" + __orientation
        + ",\"divider\":" + __divider
        + ",\"left\":" + left + ",\"right\":" + right + "," + __commonJson() + "}";
  }
}

// A decorative gap between groups of toolbar buttons (JToolBar.addSeparator()).
class __ToolBarSeparator extends Component {
  String __json() { return "{\"type\":\"toolbarsep\"," + __commonJson() + "}"; }
}

// A strip of buttons (and other controls). Renders as an ARIA toolbar with
// roving-focus arrow-key navigation. Floatable/rollover are accepted no-ops.
class JToolBar extends Container {
  public static final int HORIZONTAL = 0;
  public static final int VERTICAL = 1;

  int __orientation = HORIZONTAL;
  String __name = null;

  public JToolBar() {}
  public JToolBar(String name) { __name = name; }
  public JToolBar(int orientation) { __orientation = orientation; }
  public JToolBar(String name, int orientation) { __name = name; __orientation = orientation; }

  public void setOrientation(int o) { __orientation = o; }
  public int getOrientation() { return __orientation; }
  public void setName(String n) { __name = n; }
  public String getName() { return __name; }
  public void setFloatable(boolean b) {}
  public void setRollover(boolean b) {}
  public void addSeparator() { add(new __ToolBarSeparator()); }
  // toolbar.add(action) creates a button for the action (real Swing).
  public JButton add(Action a) { JButton b = new JButton(a); add(b); return b; }

  String __json() {
    String name = __name == null ? "null" : "\"" + Component.__esc(__name) + "\"";
    return "{\"type\":\"toolbar\",\"orientation\":" + __orientation
        + ",\"name\":" + name
        + ",\"children\":" + __kidsJson() + "," + __commonJson() + "}";
  }
}

class JLabel extends Component {
  // Alignment constants (JLabel implements SwingConstants in real Swing).
  public static final int LEFT = 2;
  public static final int CENTER = 0;
  public static final int RIGHT = 4;
  public static final int LEADING = 10;
  public static final int TRAILING = 11;
  public static final int TOP = 1;
  public static final int BOTTOM = 3;

  String __text;
  Component __labelFor = null;
  char __mnemonic = 0; // displayed mnemonic (Alt-shortcut → focus the labelFor)
  public JLabel() { __text = ""; }
  public JLabel(String text) { __text = text; }
  public JLabel(String text, int horizontalAlignment) { __text = text; __halign = horizontalAlignment; }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public void setLabelFor(Component c) { __labelFor = c; }
  // A mnemonic underlines its letter; with setLabelFor, Alt+letter focuses the
  // associated component (int form takes a KeyEvent.VK_* code == the char).
  public void setDisplayedMnemonic(int key) { __mnemonic = (char) key; }
  public void setDisplayedMnemonic(char key) { __mnemonic = key; }
  public int getDisplayedMnemonic() { return __mnemonic; }
  public void setDisplayedMnemonicIndex(int index) {}
  public void setHorizontalAlignment(int alignment) { __halign = alignment; }
  public int getHorizontalAlignment() { return __halign < 0 ? LEADING : __halign; }
  public void setVerticalAlignment(int alignment) {}
  String __json() {
    String f = __labelFor == null ? "" : ",\"for\":\"" + __labelFor.__cid + "\"";
    String m = __mnemonic == 0 ? "" : ",\"mnemonic\":\"" + __mnemonic + "\"";
    return "{\"type\":\"label\",\"text\":\"" + Component.__esc(__text) + "\"" + f + m + "," + __commonJson() + "}";
  }
}

class JButton extends Component {
  String __text;
  String __command = null;   // action command (defaults to the button text)
  char __mnemonic = 0;       // Alt-shortcut letter (0 = none)
  ActionListener __listener = null;
  public JButton() { __text = ""; }
  public JButton(String text) { __text = text; }
  // Build from an Action: pull the name/tooltip/mnemonic/enabled and fire the
  // action on click. The action is shared, so a menu item built from the same
  // Action stays in sync.
  public JButton(Action a) {
    Object n = a.getValue(Action.NAME);
    __text = n == null ? "" : "" + n;
    Object sd = a.getValue(Action.SHORT_DESCRIPTION);
    if (sd != null) __ttip = "" + sd;
    Object mn = a.getValue(Action.MNEMONIC_KEY);
    if (mn != null) { int code = (Integer) mn; __mnemonic = (char) code; }
    __action = a; // enabled tracks the action live
    __listener = a;
    __SwingRuntime.__interactive = true;
  }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public void setActionCommand(String command) { __command = command; }
  public String getActionCommand() { return __command == null ? __text : __command; }
  public void setMnemonic(int key) { __mnemonic = (char) key; }
  public void setMnemonic(char key) { __mnemonic = key; }
  public int getMnemonic() { return __mnemonic; }
  public void addActionListener(ActionListener l) {
    __listener = l;
    __SwingRuntime.__interactive = true;
  }
  // Programmatically activate the button (fires its ActionListener).
  public void doClick() { __onEvent(); }
  void __onEvent() {
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this, getActionCommand()));
  }
  boolean __listens() { return __listener != null; }
  String __json() {
    String m = __mnemonic == 0 ? "" : ",\"mnemonic\":\"" + __mnemonic + "\"";
    return "{\"type\":\"button\",\"text\":\"" + Component.__esc(__text) + "\"" + m + "," + __commonJson() + "}";
  }
}

// A two-state button: pressed or not. Like a checkbox but styled as a button
// (aria-pressed). Fires its ActionListener and/or ItemListener on toggle.
class JToggleButton extends Component {
  String __text;
  boolean __sel;
  String __command = null;
  ActionListener __actionListener = null;
  ItemListener __itemListener = null;
  public JToggleButton() { __text = ""; __sel = false; }
  public JToggleButton(String text) { __text = text; __sel = false; }
  public JToggleButton(String text, boolean selected) { __text = text; __sel = selected; }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public boolean isSelected() { return __sel; }
  public void setSelected(boolean selected) { __sel = selected; }
  public void setActionCommand(String command) { __command = command; }
  public String getActionCommand() { return __command == null ? __text : __command; }
  public void addActionListener(ActionListener l) { __actionListener = l; __SwingRuntime.__interactive = true; }
  public void addItemListener(ItemListener l) { __itemListener = l; __SwingRuntime.__interactive = true; }
  public void doClick() { __sel = !__sel; __onEvent(); }
  void __setFromHost(String value) { __sel = value.equals("true"); }
  void __onEvent() {
    if (__itemListener != null) {
      int state = __sel ? ItemEvent.SELECTED : ItemEvent.DESELECTED;
      __itemListener.itemStateChanged(new ItemEvent(this, state, __text));
    }
    if (__actionListener != null) __actionListener.actionPerformed(new ActionEvent(this, getActionCommand()));
  }
  boolean __listens() { return __actionListener != null || __itemListener != null; }
  String __json() {
    return "{\"type\":\"toggle\",\"text\":\"" + Component.__esc(__text) + "\",\"selected\":" + __sel
        + "," + __commonJson() + "}";
  }
}

class JTextField extends Component {
  public static final int LEFT = 2;
  public static final int CENTER = 0;
  public static final int RIGHT = 4;
  public static final int LEADING = 10;
  public static final int TRAILING = 11;

  String __text;
  int __cols;
  boolean __editable = true;
  boolean __password = false;
  String __command = null;
  ActionListener __actionListener = null;
  public JTextField() { __text = ""; __cols = 0; }
  public JTextField(int cols) { __text = ""; __cols = cols; }
  public JTextField(String text) { __text = text; __cols = 0; }
  public JTextField(String text, int cols) { __text = text; __cols = cols; }
  public String getText() { return __text; }
  public void setText(String text) { __text = text; }
  public int getColumns() { return __cols; }
  public void setColumns(int cols) { __cols = cols; }
  public void setEditable(boolean editable) { __editable = editable; }
  public boolean isEditable() { return __editable; }
  public void setHorizontalAlignment(int alignment) { __halign = alignment; }
  public int getHorizontalAlignment() { return __halign < 0 ? LEADING : __halign; }
  // The ActionListener fires when the user presses Enter in the field.
  public void addActionListener(ActionListener l) { __actionListener = l; __SwingRuntime.__interactive = true; }
  public void setActionCommand(String command) { __command = command; }
  public String getActionCommand() { return __command == null ? __text : __command; }
  boolean __listens() { return __actionListener != null; }
  void __onEvent() {
    if (__actionListener != null) __actionListener.actionPerformed(new ActionEvent(this, getActionCommand()));
  }
  // The text synced into __text before this; fire insert/remove by length delta.
  void __onDoc() {
    if (__document == null || __document.__listener == null) return;
    DocumentEvent e = new DocumentEvent(__document);
    if (__text.length() > __lastText.length()) __document.__listener.insertUpdate(e);
    else if (__text.length() < __lastText.length()) __document.__listener.removeUpdate(e);
    else __document.__listener.changedUpdate(e);
    __lastText = __text;
  }
  void __setFromHost(String value) { __text = value; }
  String __json() {
    String pw = __password ? ",\"password\":true" : "";
    return "{\"type\":\"textfield\",\"text\":\"" + Component.__esc(__text) + "\",\"columns\":" + __cols
        + ",\"editable\":" + __editable + pw + "," + __commonJson() + "}";
  }
}

// A masked text field. getPassword() returns the text as a char[] (real Swing
// avoids String for passwords); the renderer shows an <input type=password>.
class JPasswordField extends JTextField {
  public JPasswordField() { __password = true; }
  public JPasswordField(int cols) { __cols = cols; __password = true; }
  public JPasswordField(String text) { __text = text; __password = true; }
  public JPasswordField(String text, int cols) { __text = text; __cols = cols; __password = true; }
  public char[] getPassword() { return __text.toCharArray(); }
  public void setEchoChar(char c) {}
  public char getEchoChar() { return '*'; }
}

class JTextArea extends Component {
  String __text;
  int __rows, __cols;
  boolean __editable = true;
  boolean __wrap = false;
  public JTextArea() { __text = ""; __rows = 0; __cols = 0; }
  public JTextArea(int rows, int cols) { __text = ""; __rows = rows; __cols = cols; }
  public JTextArea(String text) { __text = text; __rows = 0; __cols = 0; }
  public JTextArea(String text, int rows, int cols) { __text = text; __rows = rows; __cols = cols; }
  public String getText() { return __text; }
  public void setText(String text) { __text = text; }
  public void append(String text) { __text = __text + text; }
  // Insert str at position pos (0..length); shifts the rest right.
  public void insert(String str, int pos) {
    if (pos < 0 || pos > __text.length()) {
      throw new IllegalArgumentException("Invalid insert position: " + pos);
    }
    __text = __text.substring(0, pos) + str + __text.substring(pos);
  }
  // Replace the text in [start, end) with str.
  public void replaceRange(String str, int start, int end) {
    if (start > end) throw new IllegalArgumentException("end before start");
    if (start < 0 || end > __text.length()) {
      throw new IllegalArgumentException("Invalid range: " + start + ".." + end);
    }
    __text = __text.substring(0, start) + str + __text.substring(end);
  }
  // Number of lines: an empty area has 1 line; each '\n' starts another.
  public int getLineCount() {
    int lines = 1;
    for (int i = 0; i < __text.length(); i++) {
      if (__text.charAt(i) == '\n') lines++;
    }
    return lines;
  }
  // Offset where a line begins (0-indexed line).
  public int getLineStartOffset(int line) throws BadLocationException {
    if (line < 0) throw new BadLocationException("Negative line", -1);
    if (line >= getLineCount()) throw new BadLocationException("No such line", __text.length() + 1);
    int seen = 0;
    if (line == 0) return 0;
    for (int i = 0; i < __text.length(); i++) {
      if (__text.charAt(i) == '\n') {
        seen++;
        if (seen == line) return i + 1;
      }
    }
    return __text.length();
  }
  // Offset just past a line's end (its terminator, or the document end).
  public int getLineEndOffset(int line) throws BadLocationException {
    if (line < 0) throw new BadLocationException("Negative line", -1);
    if (line >= getLineCount()) throw new BadLocationException("No such line", __text.length() + 1);
    if (line == getLineCount() - 1) return __text.length();
    return getLineStartOffset(line + 1);
  }
  // The line that contains the given offset (0..length).
  public int getLineOfOffset(int offset) throws BadLocationException {
    if (offset < 0 || offset > __text.length()) {
      throw new BadLocationException("Can't translate offset to line", offset);
    }
    int line = 0;
    for (int i = 0; i < offset; i++) {
      if (__text.charAt(i) == '\n') line++;
    }
    return line;
  }
  // The substring of `length` chars starting at `offset`.
  public String getText(int offset, int length) throws BadLocationException {
    if (offset < 0 || length < 0 || offset + length > __text.length()) {
      throw new BadLocationException("Invalid range", offset);
    }
    return __text.substring(offset, offset + length);
  }
  public void setEditable(boolean editable) { __editable = editable; }
  public boolean isEditable() { return __editable; }
  public void setLineWrap(boolean wrap) { __wrap = wrap; }
  public void setWrapStyleWord(boolean word) {}
  public int getRows() { return __rows; }
  public int getColumns() { return __cols; }
  void __onDoc() {
    if (__document == null || __document.__listener == null) return;
    DocumentEvent e = new DocumentEvent(__document);
    if (__text.length() > __lastText.length()) __document.__listener.insertUpdate(e);
    else if (__text.length() < __lastText.length()) __document.__listener.removeUpdate(e);
    else __document.__listener.changedUpdate(e);
    __lastText = __text;
  }
  void __setFromHost(String value) { __text = value; }
  String __json() {
    return "{\"type\":\"textarea\",\"text\":\"" + Component.__esc(__text) + "\",\"rows\":" + __rows
        + ",\"columns\":" + __cols + ",\"editable\":" + __editable + ",\"wrap\":" + __wrap
        + "," + __commonJson() + "}";
  }
}

class JCheckBox extends Component {
  String __text;
  boolean __sel;
  ItemListener __itemListener = null;
  ActionListener __actionListener = null;
  public JCheckBox() { __text = ""; __sel = false; }
  public JCheckBox(String text) { __text = text; __sel = false; }
  public JCheckBox(String text, boolean selected) { __text = text; __sel = selected; }
  public boolean isSelected() { return __sel; }
  public void setSelected(boolean selected) { __sel = selected; }
  public String getText() { return __text; }
  public void setText(String text) { __text = text; }
  public void addItemListener(ItemListener l) { __itemListener = l; __SwingRuntime.__interactive = true; }
  public void addActionListener(ActionListener l) { __actionListener = l; __SwingRuntime.__interactive = true; }
  void __setFromHost(String value) { __sel = value.equals("true"); }
  void __onEvent() {
    if (__itemListener != null) {
      int state = __sel ? ItemEvent.SELECTED : ItemEvent.DESELECTED;
      __itemListener.itemStateChanged(new ItemEvent(this, state, __text));
    }
    if (__actionListener != null) __actionListener.actionPerformed(new ActionEvent(this));
  }
  boolean __listens() { return __itemListener != null || __actionListener != null; }
  String __json() {
    return "{\"type\":\"checkbox\",\"text\":\"" + Component.__esc(__text) + "\",\"selected\":" + __sel
        + "," + __commonJson() + "}";
  }
}

class JRadioButton extends Component {
  String __text;
  boolean __sel;
  String __group = null; // ButtonGroup id, for mutual exclusivity
  ItemListener __itemListener = null;
  ActionListener __actionListener = null;
  public JRadioButton() { __text = ""; __sel = false; }
  public JRadioButton(String text) { __text = text; __sel = false; }
  public JRadioButton(String text, boolean selected) { __text = text; __sel = selected; }
  public boolean isSelected() { return __sel; }
  public void setSelected(boolean selected) { __sel = selected; }
  public String getText() { return __text; }
  public void setText(String text) { __text = text; }
  public void addItemListener(ItemListener l) { __itemListener = l; __SwingRuntime.__interactive = true; }
  public void addActionListener(ActionListener l) { __actionListener = l; __SwingRuntime.__interactive = true; }
  void __setFromHost(String value) { __sel = value.equals("true"); }
  void __onEvent() {
    if (__itemListener != null) {
      int state = __sel ? ItemEvent.SELECTED : ItemEvent.DESELECTED;
      __itemListener.itemStateChanged(new ItemEvent(this, state, __text));
    }
    if (__actionListener != null) __actionListener.actionPerformed(new ActionEvent(this));
  }
  boolean __listens() { return __itemListener != null || __actionListener != null; }
  String __json() {
    String g = __group == null ? "" : ",\"group\":\"" + __group + "\"";
    return "{\"type\":\"radio\",\"text\":\"" + Component.__esc(__text) + "\",\"selected\":" + __sel
        + g + "," + __commonJson() + "}";
  }
}

// Groups radio buttons for mutual exclusivity. The shared id becomes the DOM
// radios' `name`, so the browser enforces "only one selected"; the VM syncs
// every group member's state from the host on each event.
class ButtonGroup {
  String __gid;
  java.util.ArrayList<JRadioButton> __buttons = new java.util.ArrayList<JRadioButton>();
  java.util.ArrayList<JRadioButtonMenuItem> __menuItems = new java.util.ArrayList<JRadioButtonMenuItem>();
  public ButtonGroup() { __gid = "g" + Component.__nextId(); }
  public void add(JRadioButton b) { b.__group = __gid; __buttons.add(b); }
  public void add(JRadioButtonMenuItem m) { m.__bgroup = this; __menuItems.add(m); }
  public int getButtonCount() { return __buttons.size() + __menuItems.size(); }
  // Select `chosen` and deselect its group siblings (menu radios have no shared
  // DOM name, so the VM enforces exclusivity).
  void __selectRadioMenu(JRadioButtonMenuItem chosen) {
    for (int i = 0; i < __menuItems.size(); i++) {
      __menuItems.get(i).__sel = (__menuItems.get(i) == chosen);
    }
  }
}

class JComboBox extends Component {
  java.util.ArrayList<String> __items = new java.util.ArrayList<String>();
  int __selectedIndex = -1;
  boolean __editable = false;
  String __editText = ""; // the current text in editable mode (may be custom)
  ActionListener __actionListener = null;
  public JComboBox() {}
  public JComboBox(String[] items) {
    for (int i = 0; i < items.length; i++) __items.add(items[i]);
    if (__items.size() > 0) { __selectedIndex = 0; __editText = __items.get(0); }
  }
  public void addItem(String item) {
    __items.add(item);
    if (__selectedIndex < 0) { __selectedIndex = 0; __editText = item; }
  }
  public void insertItemAt(String item, int index) { __items.add(index, item); }
  public void removeItem(String item) {
    int i = __items.indexOf(item);
    if (i >= 0) removeItemAt(i);
  }
  public void removeItemAt(int index) {
    __items.remove(index);
    if (__selectedIndex >= __items.size()) __selectedIndex = __items.size() - 1;
  }
  public void removeAllItems() { __items.clear(); __selectedIndex = -1; __editText = ""; }
  public int getItemCount() { return __items.size(); }
  public String getItemAt(int index) { return __items.get(index); }
  public int getSelectedIndex() { return __editable ? __items.indexOf(__editText) : __selectedIndex; }
  public void setSelectedIndex(int index) {
    __selectedIndex = index;
    if (index >= 0 && index < __items.size()) __editText = __items.get(index);
  }
  public Object getSelectedItem() {
    if (__editable) return __editText.equals("") ? null : __editText;
    if (__selectedIndex < 0 || __selectedIndex >= __items.size()) return null;
    return __items.get(__selectedIndex);
  }
  // For an editable combo, accepts a value not in the list (a custom entry).
  public void setSelectedItem(Object item) {
    String s = item == null ? "" : "" + item;
    __editText = s;
    __selectedIndex = __items.indexOf(s);
  }
  public void setEditable(boolean editable) { __editable = editable; }
  public boolean isEditable() { return __editable; }
  public void addActionListener(ActionListener l) { __actionListener = l; __SwingRuntime.__interactive = true; }
  void __setFromHost(String value) {
    if (__editable) {
      __editText = value;
      __selectedIndex = __items.indexOf(value);
    } else {
      __selectedIndex = Integer.parseInt(value);
      if (__selectedIndex >= 0 && __selectedIndex < __items.size()) __editText = __items.get(__selectedIndex);
    }
  }
  void __onEvent() {
    if (__actionListener != null) __actionListener.actionPerformed(new ActionEvent(this));
  }
  boolean __listens() { return __actionListener != null; }
  String __json() {
    StringBuilder opts = new StringBuilder("[");
    for (int i = 0; i < __items.size(); i++) {
      if (i > 0) opts.append(",");
      opts.append("\"").append(Component.__esc(__items.get(i))).append("\"");
    }
    opts.append("]");
    // Editable: an <input list=…> with a datalist; the value is the text (which
    // may be a custom entry). Non-editable: a <select> keyed by index.
    String ed = __editable
        ? ",\"editable\":true,\"text\":\"" + Component.__esc(__editText) + "\""
        : "";
    return "{\"type\":\"combobox\",\"items\":" + opts.toString() + ",\"selectedIndex\":" + __selectedIndex
        + ed + "," + __commonJson() + "}";
  }
}

// The selection modes as constants (real Swing puts these on
// ListSelectionModel). Also a small selection model that carries the
// ListSelectionListener for a JTable's getSelectionModel().addListSelectionListener.
class ListSelectionModel {
  public static final int SINGLE_SELECTION = 0;
  public static final int SINGLE_INTERVAL_SELECTION = 1;
  public static final int MULTIPLE_INTERVAL_SELECTION = 2;
  ListSelectionListener __listener = null;
  public void addListSelectionListener(ListSelectionListener l) {
    __listener = l;
    __SwingRuntime.__interactive = true;
  }
  public void setSelectionMode(int mode) {} // single-row selection in this model
}

// A mutable list model. A JList built on one re-reads it on every render, so
// addElement/remove/clear appear as soon as the event loop repaints.
class DefaultListModel {
  java.util.ArrayList<String> __elements = new java.util.ArrayList<String>();
  java.util.ArrayList<ListDataListener> __dataListeners = new java.util.ArrayList<ListDataListener>();
  public DefaultListModel() {}
  public void addListDataListener(ListDataListener l) { __dataListeners.add(l); }
  public void removeListDataListener(ListDataListener l) { __dataListeners.remove(l); }
  // Notify listeners of an add / remove / change over [i0, i1], as real Swing's
  // AbstractListModel does when the model is mutated.
  void __fireAdded(int i0, int i1) {
    ListDataEvent e = new ListDataEvent(this, ListDataEvent.INTERVAL_ADDED, i0, i1);
    for (ListDataListener l : __dataListeners) l.intervalAdded(e);
  }
  void __fireRemoved(int i0, int i1) {
    ListDataEvent e = new ListDataEvent(this, ListDataEvent.INTERVAL_REMOVED, i0, i1);
    for (ListDataListener l : __dataListeners) l.intervalRemoved(e);
  }
  void __fireChanged(int i0, int i1) {
    ListDataEvent e = new ListDataEvent(this, ListDataEvent.CONTENTS_CHANGED, i0, i1);
    for (ListDataListener l : __dataListeners) l.contentsChanged(e);
  }
  public void addElement(String element) {
    __elements.add(element);
    __fireAdded(__elements.size() - 1, __elements.size() - 1);
  }
  public void add(int index, String element) {
    __elements.add(index, element);
    __fireAdded(index, index);
  }
  public String get(int index) { return __elements.get(index); }
  public String getElementAt(int index) { return __elements.get(index); }
  public String elementAt(int index) { return __elements.get(index); }
  public String set(int index, String element) {
    String previous = __elements.set(index, element);
    __fireChanged(index, index);
    return previous;
  }
  public String remove(int index) {
    String removed = __elements.remove(index);
    __fireRemoved(index, index);
    return removed;
  }
  // Element params are String (the model holds Strings), where real Swing uses
  // Object — the bundled ArrayList types these overloads by element type.
  public boolean removeElement(String element) {
    int index = __elements.indexOf(element);
    if (index < 0) return false;
    __elements.remove(index);
    __fireRemoved(index, index);
    return true;
  }
  public void removeAllElements() { __clearAndFire(); }
  public void clear() { __clearAndFire(); }
  void __clearAndFire() {
    int last = __elements.size() - 1;
    __elements.clear();
    if (last >= 0) __fireRemoved(0, last);
  }
  public int getSize() { return __elements.size(); }
  public int size() { return __elements.size(); }
  public boolean isEmpty() { return __elements.isEmpty(); }
  public boolean contains(String element) { return __elements.contains(element); }
  public int indexOf(String element) { return __elements.indexOf(element); }
  public String firstElement() { return __elements.get(0); }
  public String lastElement() { return __elements.get(__elements.size() - 1); }
}

class JList extends Component {
  java.util.ArrayList<String> __items = new java.util.ArrayList<String>();
  DefaultListModel __model = null;
  // Selected indices, in the order the host reports them.
  java.util.ArrayList<Integer> __selected = new java.util.ArrayList<Integer>();
  int __visibleRows = 8;
  int __mode = 2; // MULTIPLE_INTERVAL_SELECTION, matching real JList
  ListSelectionListener __listener = null;
  public JList() {}
  public JList(String[] items) { setListData(items); }
  public JList(DefaultListModel model) { __model = model; }
  public void setModel(DefaultListModel model) { __model = model; }
  public DefaultListModel getModel() { return __model; }
  public void setListData(String[] items) {
    __model = null;
    __items = new java.util.ArrayList<String>();
    for (int i = 0; i < items.length; i++) __items.add(items[i]);
  }
  // The live element list: the model's when model-backed, else the static data.
  java.util.ArrayList<String> __data() { return __model != null ? __model.__elements : __items; }
  public int getSelectedIndex() { return __selected.isEmpty() ? -1 : __selected.get(0); }
  public void setSelectedIndex(int index) {
    __selected = new java.util.ArrayList<Integer>();
    if (index >= 0) __selected.add(index);
  }
  public int[] getSelectedIndices() {
    int[] out = new int[__selected.size()];
    for (int i = 0; i < __selected.size(); i++) out[i] = __selected.get(i);
    return out;
  }
  public Object getSelectedValue() {
    if (__selected.isEmpty()) return null;
    int index = __selected.get(0);
    if (index < 0 || index >= __data().size()) return null;
    return __data().get(index);
  }
  public Object[] getSelectedValues() {
    Object[] out = new Object[__selected.size()];
    for (int i = 0; i < __selected.size(); i++) out[i] = __data().get(__selected.get(i));
    return out;
  }
  public boolean isSelectedIndex(int index) { return __selected.contains(index); }
  public boolean isSelectionEmpty() { return __selected.isEmpty(); }
  public void clearSelection() { __selected = new java.util.ArrayList<Integer>(); }
  public void setVisibleRowCount(int rows) { __visibleRows = rows; }
  public int getVisibleRowCount() { return __visibleRows; }
  public void setSelectionMode(int mode) { __mode = mode; }
  public int getSelectionMode() { return __mode; }
  public void addListSelectionListener(ListSelectionListener l) { __listener = l; __SwingRuntime.__interactive = true; }
  boolean __multiple() { return __mode != 0; } // 0 == SINGLE_SELECTION
  // The host sends the selected indices comma-separated ("" when none, or
  // "-1" from a single <select> with nothing chosen); ignore invalid ones.
  void __setFromHost(String value) {
    __selected = new java.util.ArrayList<Integer>();
    int start = 0;
    while (start < value.length()) {
      int comma = value.indexOf(",", start);
      String part = comma < 0 ? value.substring(start) : value.substring(start, comma);
      if (part.length() > 0) {
        int index = Integer.parseInt(part);
        if (index >= 0) __selected.add(index);
      }
      if (comma < 0) break;
      start = comma + 1;
    }
  }
  void __onEvent() {
    if (__listener != null) __listener.valueChanged(new ListSelectionEvent(this));
  }
  boolean __listens() { return __listener != null; }
  String __json() {
    java.util.ArrayList<String> data = __data();
    StringBuilder opts = new StringBuilder("[");
    for (int i = 0; i < data.size(); i++) {
      if (i > 0) opts.append(",");
      opts.append("\"").append(Component.__esc(data.get(i))).append("\"");
    }
    opts.append("]");
    StringBuilder sel = new StringBuilder("[");
    for (int i = 0; i < __selected.size(); i++) {
      if (i > 0) sel.append(",");
      sel.append(__selected.get(i));
    }
    sel.append("]");
    return "{\"type\":\"list\",\"items\":" + opts.toString() + ",\"selectedIndices\":" + sel.toString()
        + ",\"multiple\":" + __multiple() + ",\"rows\":" + __visibleRows + "," + __commonJson() + "}";
  }
}

class JSlider extends Component {
  int __min, __max, __value;
  ChangeListener __changeListener = null;
  public JSlider() { __min = 0; __max = 100; __value = 50; }
  public JSlider(int min, int max) { __min = min; __max = max; __value = (min + max) / 2; }
  public JSlider(int min, int max, int value) { __min = min; __max = max; __value = value; }
  public int getValue() { return __value; }
  public void setValue(int value) { __value = value; }
  public int getMinimum() { return __min; }
  public int getMaximum() { return __max; }
  public void addChangeListener(ChangeListener l) { __changeListener = l; __SwingRuntime.__interactive = true; }
  void __setFromHost(String value) { __value = Integer.parseInt(value); }
  void __onEvent() {
    if (__changeListener != null) __changeListener.stateChanged(new ChangeEvent(this));
  }
  boolean __listens() { return __changeListener != null; }
  String __json() {
    return "{\"type\":\"slider\",\"min\":" + __min + ",\"max\":" + __max + ",\"value\":" + __value
        + "," + __commonJson() + "}";
  }
}

// A progress bar (display only): determinate with a value, or indeterminate.
class JProgressBar extends Component {
  int __min = 0;
  int __max = 100;
  int __value = 0;
  boolean __indeterminate = false;
  boolean __stringPainted = false;
  String __string = null;
  public JProgressBar() {}
  public JProgressBar(int min, int max) { __min = min; __max = max; }
  public void setValue(int value) { __value = value; }
  public int getValue() { return __value; }
  public void setMinimum(int min) { __min = min; }
  public int getMinimum() { return __min; }
  public void setMaximum(int max) { __max = max; }
  public int getMaximum() { return __max; }
  public void setIndeterminate(boolean b) { __indeterminate = b; }
  public boolean isIndeterminate() { return __indeterminate; }
  public void setStringPainted(boolean b) { __stringPainted = b; }
  public boolean isStringPainted() { return __stringPainted; }
  public void setString(String s) { __string = s; }
  public String getString() {
    if (__string != null) return __string;
    int span = __max - __min;
    int pct = span <= 0 ? 0 : (__value - __min) * 100 / span;
    return pct + "%";
  }
  public int getPercentComplete() {
    int span = __max - __min;
    return span <= 0 ? 0 : (__value - __min) * 100 / span;
  }
  String __json() {
    String extra = ",\"stringPainted\":" + __stringPainted;
    if (__stringPainted) extra += ",\"string\":\"" + Component.__esc(getString()) + "\"";
    return "{\"type\":\"progressbar\",\"min\":" + __min + ",\"max\":" + __max + ",\"value\":" + __value
        + ",\"indeterminate\":" + __indeterminate + extra + "," + __commonJson() + "}";
  }
}

// SpinnerNumberModel + JSpinner: an integer spinner. getValue() returns an
// Object (a boxed Integer), as in real Swing.
class SpinnerNumberModel {
  int __value;
  int __min;
  int __max;
  int __step;
  public SpinnerNumberModel() { __value = 0; __min = 0; __max = 100; __step = 1; }
  public SpinnerNumberModel(int value, int min, int max, int step) {
    __value = value; __min = min; __max = max; __step = step;
  }
  public Object getValue() { return __value; }
  public Object getNumber() { return __value; }
  public void setValue(int value) { __value = value; }
  public int getMinimum() { return __min; }
  public int getMaximum() { return __max; }
  public int getStepSize() { return __step; }
}

class JSpinner extends Component {
  SpinnerNumberModel __model;
  ChangeListener __changeListener = null;
  public JSpinner() { __model = new SpinnerNumberModel(); }
  public JSpinner(SpinnerNumberModel model) { __model = model; }
  public Object getValue() { return __model.__value; }
  public void setValue(int value) { __model.__value = value; }
  public SpinnerNumberModel getModel() { return __model; }
  public void addChangeListener(ChangeListener l) { __changeListener = l; __SwingRuntime.__interactive = true; }
  void __setFromHost(String value) { __model.__value = Integer.parseInt(value); }
  void __onEvent() {
    if (__changeListener != null) __changeListener.stateChanged(new ChangeEvent(this));
  }
  boolean __listens() { return __changeListener != null; }
  String __json() {
    return "{\"type\":\"spinner\",\"value\":" + __model.__value + ",\"min\":" + __model.__min
        + ",\"max\":" + __model.__max + ",\"step\":" + __model.__step + "," + __commonJson() + "}";
  }
}

// A mutable table model. A JTable built on one re-reads it every render, so
// addRow/removeRow/setValueAt appear as soon as the event loop repaints. Cells
// are stored in row-major order as their string form (this subset shows cells
// as text) — real Swing keeps Objects, and its element params are Object.
class DefaultTableModel {
  java.util.ArrayList<String> __columns = new java.util.ArrayList<String>();
  java.util.ArrayList<String> __cells = new java.util.ArrayList<String>();
  java.util.ArrayList<TableModelListener> __tableListeners = new java.util.ArrayList<TableModelListener>();
  public DefaultTableModel() {}
  public DefaultTableModel(Object[][] data, Object[] columnNames) {
    for (int c = 0; c < columnNames.length; c++) __columns.add(__str(columnNames[c]));
    for (int r = 0; r < data.length; r++) addRow(data[r]);
  }
  static String __str(Object v) { return v == null ? "" : "" + v; }
  public void addTableModelListener(TableModelListener l) { __tableListeners.add(l); }
  public void removeTableModelListener(TableModelListener l) { __tableListeners.remove(l); }
  // Notify listeners of a change over rows [first, last] (of the given type),
  // as real Swing's AbstractTableModel does when the model is mutated.
  void __fireTable(int firstRow, int lastRow, int column, int type) {
    TableModelEvent e = new TableModelEvent(this, firstRow, lastRow, column, type);
    for (TableModelListener l : __tableListeners) l.tableChanged(e);
  }
  public int getColumnCount() { return __columns.size(); }
  public int getRowCount() { return __columns.isEmpty() ? 0 : __cells.size() / __columns.size(); }
  public String getColumnName(int col) { return __columns.get(col); }
  public Object getValueAt(int row, int col) { return __cells.get(row * __columns.size() + col); }
  public void setValueAt(Object value, int row, int col) {
    __cells.set(row * __columns.size() + col, __str(value));
    __fireTable(row, row, col, TableModelEvent.UPDATE);
  }
  // Cells are editable by default; override this to make (some) read-only.
  public boolean isCellEditable(int row, int col) { return true; }
  public void addRow(Object[] rowData) {
    int row = getRowCount();
    for (int c = 0; c < __columns.size(); c++) {
      __cells.add(__str(c < rowData.length ? rowData[c] : null));
    }
    __fireTable(row, row, TableModelEvent.ALL_COLUMNS, TableModelEvent.INSERT);
  }
  public void insertRow(int row, Object[] rowData) {
    int base = row * __columns.size();
    for (int c = 0; c < __columns.size(); c++) {
      __cells.add(base + c, __str(c < rowData.length ? rowData[c] : null));
    }
    __fireTable(row, row, TableModelEvent.ALL_COLUMNS, TableModelEvent.INSERT);
  }
  public void removeRow(int row) {
    int cols = __columns.size();
    for (int c = 0; c < cols; c++) __cells.remove(row * cols);
    __fireTable(row, row, TableModelEvent.ALL_COLUMNS, TableModelEvent.DELETE);
  }
  public void addColumn(Object columnName) {
    int oldCols = __columns.size();
    int rows = oldCols == 0 ? 0 : __cells.size() / oldCols;
    __columns.add(__str(columnName));
    for (int r = rows - 1; r >= 0; r--) __cells.add((r + 1) * oldCols, "");
    // A new column is a structure change (HEADER_ROW), like fireTableStructureChanged.
    __fireTable(TableModelEvent.HEADER_ROW, TableModelEvent.HEADER_ROW,
        TableModelEvent.ALL_COLUMNS, TableModelEvent.UPDATE);
  }
  public void setRowCount(int rowCount) {
    int cols = __columns.size();
    int oldRows = getRowCount();
    int target = rowCount * cols;
    while (__cells.size() > target) __cells.remove(__cells.size() - 1);
    while (__cells.size() < target) __cells.add("");
    int newRows = getRowCount();
    if (newRows > oldRows) {
      __fireTable(oldRows, newRows - 1, TableModelEvent.ALL_COLUMNS, TableModelEvent.INSERT);
    } else if (newRows < oldRows) {
      __fireTable(newRows, oldRows - 1, TableModelEvent.ALL_COLUMNS, TableModelEvent.DELETE);
    }
  }
}

// A grid of rows and columns. Backed either by fixed Object[][] data or a
// mutable DefaultTableModel; row selection (single row) reports through
// getSelectionModel()'s listener.
class JTable extends Component {
  Object[][] __data;
  Object[] __columns;
  DefaultTableModel __model = null;
  int __selectedRow = -1;
  ListSelectionModel __selectionModel = new ListSelectionModel();
  public JTable() { __data = new Object[0][0]; __columns = new Object[0]; }
  public JTable(Object[][] data, Object[] columns) { __data = data; __columns = columns; }
  public JTable(DefaultTableModel model) { __data = new Object[0][0]; __columns = new Object[0]; __model = model; }
  public void setModel(DefaultTableModel model) { __model = model; }
  public DefaultTableModel getModel() { return __model; }
  public int getRowCount() { return __model != null ? __model.getRowCount() : __data.length; }
  public int getColumnCount() { return __model != null ? __model.getColumnCount() : __columns.length; }
  public String getColumnName(int col) {
    return __model != null ? __model.getColumnName(col) : "" + __columns[col];
  }
  public Object getValueAt(int row, int col) {
    return __model != null ? __model.getValueAt(row, col) : __data[row][col];
  }
  public void setValueAt(Object value, int row, int col) {
    if (__model != null) __model.setValueAt(value, row, col);
    else __data[row][col] = value;
  }
  public int getSelectedRow() { return __selectedRow; }
  public void setRowSelectionInterval(int index0, int index1) { __selectedRow = index0; }
  public void clearSelection() { __selectedRow = -1; }
  public ListSelectionModel getSelectionModel() { return __selectionModel; }
  public boolean isCellEditable(int row, int col) {
    return __model != null ? __model.isCellEditable(row, col) : true;
  }
  // The host sends either a row index (selection) or "edit:<row>,<col>,<value>"
  // (a committed cell edit, which updates the value model without firing the
  // selection listener).
  void __setFromHost(String value) {
    if (value.startsWith("edit:")) {
      String rest = value.substring(5);
      int c1 = rest.indexOf(",");
      int c2 = rest.indexOf(",", c1 + 1);
      int row = Integer.parseInt(rest.substring(0, c1));
      int col = Integer.parseInt(rest.substring(c1 + 1, c2));
      setValueAt(rest.substring(c2 + 1), row, col);
    } else {
      __selectedRow = Integer.parseInt(value);
    }
  }
  void __onEvent() {
    if (__selectionModel.__listener != null) {
      __selectionModel.__listener.valueChanged(new ListSelectionEvent(this));
    }
  }
  boolean __listens() { return __selectionModel.__listener != null; }
  boolean __editable() { return isCellEditable(0, 0); }
  String __colsJson() {
    StringBuilder s = new StringBuilder("[");
    for (int c = 0; c < getColumnCount(); c++) {
      if (c > 0) s.append(",");
      s.append("\"").append(Component.__esc(getColumnName(c))).append("\"");
    }
    return s.append("]").toString();
  }
  String __rowsJson() {
    StringBuilder s = new StringBuilder("[");
    for (int r = 0; r < getRowCount(); r++) {
      if (r > 0) s.append(",");
      s.append("[");
      for (int c = 0; c < getColumnCount(); c++) {
        if (c > 0) s.append(",");
        Object cell = getValueAt(r, c);
        s.append("\"").append(Component.__esc(cell == null ? "" : "" + cell)).append("\"");
      }
      s.append("]");
    }
    return s.append("]").toString();
  }
  String __json() {
    return "{\"type\":\"table\",\"headers\":" + __colsJson() + ",\"cells\":" + __rowsJson()
        + ",\"selectedRow\":" + __selectedRow + ",\"editable\":" + __editable()
        + "," + __commonJson() + "}";
  }
}

// ----- Menus -----

class JMenuItem extends Component {
  String __text;
  boolean __sep = false;
  ActionListener __listener = null;
  KeyStroke __accel = null;
  public JMenuItem() { __text = ""; }
  public JMenuItem(String text) { __text = text; }
  // Build from an Action: name/tooltip/accelerator/enabled + fire the action.
  public JMenuItem(Action a) {
    Object n = a.getValue(Action.NAME);
    __text = n == null ? "" : "" + n;
    Object sd = a.getValue(Action.SHORT_DESCRIPTION);
    if (sd != null) __ttip = "" + sd;
    Object ac = a.getValue(Action.ACCELERATOR_KEY);
    if (ac != null) __accel = (KeyStroke) ac;
    __action = a; // enabled tracks the action live
    __listener = a;
    __SwingRuntime.__interactive = true;
  }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public void setAccelerator(KeyStroke k) { __accel = k; __SwingRuntime.__interactive = true; }
  public KeyStroke getAccelerator() { return __accel; }
  // The ",\"accel\":\"Ctrl+S\"" fragment (both the display hint and the key the
  // renderer matches a keydown against), or "" when there's no accelerator.
  String __accelJson() {
    if (__accel == null) return "";
    return ",\"accel\":\"" + Component.__esc(__accel.__text()) + "\"";
  }
  public void addActionListener(ActionListener l) { __listener = l; __SwingRuntime.__interactive = true; }
  void __onEvent() {
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this, __text));
  }
  boolean __listens() { return __listener != null; }
  String __json() {
    if (__sep) return "{\"type\":\"separator\"}";
    return "{\"type\":\"menuitem\",\"text\":\"" + Component.__esc(__text) + "\""
        + __accelJson() + "," + __commonJson() + "}";
  }
}

// A menu item with a checkbox. Clicking toggles it and fires the listeners.
class JCheckBoxMenuItem extends JMenuItem {
  boolean __sel = false;
  ItemListener __itemListener = null;
  public JCheckBoxMenuItem() {}
  public JCheckBoxMenuItem(String text) { __text = text; }
  public JCheckBoxMenuItem(String text, boolean selected) { __text = text; __sel = selected; }
  public boolean isSelected() { return __sel; }
  public void setSelected(boolean selected) { __sel = selected; }
  public boolean getState() { return __sel; }
  public void setState(boolean state) { __sel = state; }
  public void addItemListener(ItemListener l) { __itemListener = l; __SwingRuntime.__interactive = true; }
  boolean __listens() { return __listener != null || __itemListener != null; }
  void __onEvent() {
    __sel = !__sel; // activating a check item toggles it
    if (__itemListener != null) {
      int state = __sel ? ItemEvent.SELECTED : ItemEvent.DESELECTED;
      __itemListener.itemStateChanged(new ItemEvent(this, state, __text));
    }
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this, __text));
  }
  String __json() {
    return "{\"type\":\"checkmenuitem\",\"text\":\"" + Component.__esc(__text) + "\",\"selected\":" + __sel
        + __accelJson() + "," + __commonJson() + "}";
  }
}

// A menu item radio button; a ButtonGroup makes a set of them mutually
// exclusive. Selecting one deselects its group siblings.
class JRadioButtonMenuItem extends JMenuItem {
  boolean __sel = false;
  ButtonGroup __bgroup = null;
  public JRadioButtonMenuItem() {}
  public JRadioButtonMenuItem(String text) { __text = text; }
  public JRadioButtonMenuItem(String text, boolean selected) { __text = text; __sel = selected; }
  public boolean isSelected() { return __sel; }
  public void setSelected(boolean selected) { __sel = selected; }
  boolean __listens() { return true; } // always dispatches (to update the selection)
  void __onEvent() {
    if (__bgroup != null) __bgroup.__selectRadioMenu(this);
    else __sel = true;
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this, __text));
  }
  String __json() {
    return "{\"type\":\"radiomenuitem\",\"text\":\"" + Component.__esc(__text) + "\",\"selected\":" + __sel
        + __accelJson() + "," + __commonJson() + "}";
  }
}

// A JMenu is itself a JMenuItem (as in real Swing), so it can be added to
// another JMenu as a submenu. It holds its own items.
class JMenu extends JMenuItem {
  java.util.ArrayList<JMenuItem> __items = new java.util.ArrayList<JMenuItem>();
  public JMenu() {}
  public JMenu(String text) { __text = text; }
  // Adding an item (or a submenu, which is also a JMenuItem) registers it so a
  // click can find and fire it.
  public void add(JMenuItem item) { __items.add(item); __SwingRuntime.__register(item); }
  public void addSeparator() {
    JMenuItem s = new JMenuItem();
    s.__sep = true;
    __items.add(s);
  }
  public int getItemCount() { return __items.size(); }
  String __json() {
    StringBuilder s = new StringBuilder("{\"type\":\"menu\",\"text\":\"");
    s.append(Component.__esc(__text)).append("\",\"items\":[");
    for (int i = 0; i < __items.size(); i++) {
      if (i > 0) s.append(",");
      s.append(__items.get(i).__json());
    }
    return s.append("]}").toString();
  }
}

class JMenuBar {
  java.util.ArrayList<JMenu> __menus = new java.util.ArrayList<JMenu>();
  public JMenuBar() {}
  public void add(JMenu menu) { __menus.add(menu); }
  public int getMenuCount() { return __menus.size(); }
  String __json() {
    StringBuilder s = new StringBuilder("{\"menus\":[");
    for (int i = 0; i < __menus.size(); i++) {
      if (i > 0) s.append(",");
      s.append(__menus.get(i).__json());
    }
    return s.append("]}").toString();
  }
}

// javax.swing.JOptionPane: standard modal dialogs. Each show* blocks (via the
// native System.__uiDialog) until the user answers, then returns the result —
// a message dialog returns nothing, a confirm returns an option code, an input
// returns the typed text (or null when dismissed).
class JOptionPane {
  public static final int YES_NO_OPTION = 0;
  public static final int YES_NO_CANCEL_OPTION = 1;
  public static final int OK_CANCEL_OPTION = 2;
  public static final int DEFAULT_OPTION = -1;
  public static final int YES_OPTION = 0;
  public static final int OK_OPTION = 0;
  public static final int NO_OPTION = 1;
  public static final int CANCEL_OPTION = 2;
  public static final int CLOSED_OPTION = -1;
  public static final int PLAIN_MESSAGE = -1;
  public static final int ERROR_MESSAGE = 0;
  public static final int INFORMATION_MESSAGE = 1;
  public static final int WARNING_MESSAGE = 2;
  public static final int QUESTION_MESSAGE = 3;

  public static void showMessageDialog(Object parent, Object message) {
    System.__uiDialog("message", "" + message);
  }
  public static void showMessageDialog(Object parent, Object message, String title, int messageType) {
    System.__uiDialog("message", "" + message);
  }
  public static int showConfirmDialog(Object parent, Object message) {
    return __parse(System.__uiDialog("confirm:" + YES_NO_CANCEL_OPTION, "" + message));
  }
  public static int showConfirmDialog(Object parent, Object message, String title, int optionType) {
    return __parse(System.__uiDialog("confirm:" + optionType, "" + message));
  }
  public static String showInputDialog(Object parent, Object message) {
    return System.__uiDialog("input", "" + message);
  }
  public static String showInputDialog(Object message) {
    return System.__uiDialog("input", "" + message);
  }
  static int __parse(String s) {
    if (s == null) return CLOSED_OPTION;
    try { return Integer.parseInt(s); } catch (Exception e) { return CLOSED_OPTION; }
  }
}

// javax.swing.Timer: fires its ActionListener every `delay` ms. The host
// schedules the wakeup (see App.awaitSwingEvent); the loop dispatches the
// tick. A running timer keeps the app alive (like any animation).
class Timer {
  String __tid;
  int __delay;
  boolean __running = false;
  boolean __repeats = true;
  ActionListener __listener;
  public Timer(int delay, ActionListener listener) {
    __tid = "t" + Component.__nextId();
    __delay = delay;
    __listener = listener;
  }
  public void start() {
    __running = true;
    __SwingRuntime.__interactive = true;
    __SwingRuntime.__addTimer(this);
  }
  public void stop() { __running = false; }
  public void restart() { start(); }
  public boolean isRunning() { return __running; }
  public void setDelay(int delay) { __delay = delay; }
  public int getDelay() { return __delay; }
  public void setRepeats(boolean repeats) { __repeats = repeats; }
  public void addActionListener(ActionListener l) { __listener = l; }
  void __fire() {
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this));
    if (!__repeats) stop();
  }
}

// java.lang.Runnable — the target of SwingUtilities.invokeLater and friends.
interface Runnable {
  void run();
}

// javax.swing.SwingUtilities. caturra runs single-threaded and synchronously,
// so there is no separate event-dispatch thread: invokeLater / invokeAndWait
// simply run the task now (the common pattern is to build the UI and call
// setVisible inside it, which then enters the event loop as usual).
class SwingUtilities {
  public static void invokeLater(Runnable r) { if (r != null) r.run(); }
  public static void invokeAndWait(Runnable r) { if (r != null) r.run(); }
  public static boolean isEventDispatchThread() { return true; }
}

class JFrame extends Container {
  String __title;
  int __w = 0;
  int __h = 0;

  public JFrame() { __title = ""; }
  public JFrame(String title) { __title = title; }

  // Real JFrame default: closing hides the window (the program keeps
  // running). Students set EXIT_ON_CLOSE to end it — see __SwingRuntime.
  int __closeOp = 1; // HIDE_ON_CLOSE
  JMenuBar __menuBar = null;

  public void setTitle(String title) { __title = title; }
  public String getTitle() { return __title; }
  public void setSize(int width, int height) { __w = width; __h = height; }
  public void setSize(Dimension d) { __w = d.width; __h = d.height; }
  public void setPreferredSize(Dimension d) { __w = d.width; __h = d.height; }
  public void setJMenuBar(JMenuBar bar) { __menuBar = bar; }
  public JMenuBar getJMenuBar() { return __menuBar; }
  public void setDefaultCloseOperation(int op) { __closeOp = op; }
  public int getDefaultCloseOperation() { return __closeOp; }
  public void setResizable(boolean resizable) {}
  public void setLocationRelativeTo(Object o) {}
  public void pack() {}
  // A JFrame IS its content pane here, so getContentPane().add(...) / setLayout
  // work as students expect.
  public Container getContentPane() { return this; }
  public void setContentPane(Container c) {}

  public static final int EXIT_ON_CLOSE = 3;
  public static final int DISPOSE_ON_CLOSE = 2;
  public static final int HIDE_ON_CLOSE = 1;
  public static final int DO_NOTHING_ON_CLOSE = 0;

  public void setVisible(boolean visible) {
    if (!visible) return;
    __render();
    // If any listener was registered, hand control to the event loop; it
    // blocks on the host, dispatching events until the window closes. With
    // no listeners (or no interactive host) this returns and the static
    // swing.json is the whole story (Phase 1 batch render).
    if (__SwingRuntime.__interactive) __SwingRuntime.__loop(this);
  }

  String __jsonTree() {
    String menubar = __menuBar == null ? "" : ",\"menubar\":" + __menuBar.__json();
    String focus = __SwingRuntime.__focusRequest == null ? ""
        : ",\"focus\":\"" + __SwingRuntime.__focusRequest + "\"";
    return "{\"type\":\"frame\",\"title\":\"" + Component.__esc(__title) + "\",\"width\":" + __w
        + ",\"height\":" + __h + ",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson() + menubar + focus
        + ",\"timers\":" + __SwingRuntime.__timersJson() + "," + __commonJson() + "}";
  }

  void __render() {
    try {
      java.io.PrintWriter w = new java.io.PrintWriter(new java.io.File("swing.json"));
      w.print(__jsonTree());
      w.close();
    } catch (Exception e) {}
  }
}

// ----- java.awt.event + the Phase 2 event loop -----

class ActionEvent {
  // Modifier masks (java.awt.event.ActionEvent), used with KeyStroke too.
  public static final int SHIFT_MASK = 1;
  public static final int CTRL_MASK = 2;
  public static final int META_MASK = 4;
  public static final int ALT_MASK = 8;
  Object __src;
  String __command = null;
  public ActionEvent(Object source) { __src = source; }
  public ActionEvent(Object source, String command) { __src = source; __command = command; }
  public Object getSource() { return __src; }
  public String getActionCommand() { return __command; }
}

// java.awt.event.InputEvent modifier masks (the newer *_DOWN_MASK forms and the
// legacy ones); both are accepted by KeyStroke.
class InputEvent {
  public static final int SHIFT_DOWN_MASK = 64;
  public static final int CTRL_DOWN_MASK = 128;
  public static final int META_DOWN_MASK = 256;
  public static final int ALT_DOWN_MASK = 512;
  public static final int SHIFT_MASK = 1;
  public static final int CTRL_MASK = 2;
  public static final int META_MASK = 4;
  public static final int ALT_MASK = 8;
}

// javax.swing.KeyStroke: a key + modifiers, used as a menu accelerator. Its
// display text (e.g. "Ctrl+S") is also the match string the renderer compares a
// keydown against.
class KeyStroke {
  int __code;
  int __mod;
  KeyStroke(int code, int mod) { __code = code; __mod = mod; }
  public static KeyStroke getKeyStroke(int keyCode, int modifiers) { return new KeyStroke(keyCode, modifiers); }
  public static KeyStroke getKeyStroke(char keyChar) {
    int c = keyChar;
    if (c >= 97 && c <= 122) c -= 32; // upper-case a letter
    return new KeyStroke(c, 0);
  }
  public int getKeyCode() { return __code; }
  public int getModifiers() { return __mod; }
  String __text() {
    boolean ctrl = (__mod & 128) != 0 || (__mod & 2) != 0;
    boolean shift = (__mod & 64) != 0 || (__mod & 1) != 0;
    boolean alt = (__mod & 512) != 0 || (__mod & 8) != 0;
    String s = "";
    if (ctrl) s += "Ctrl+";
    if (shift) s += "Shift+";
    if (alt) s += "Alt+";
    return s + __keyText();
  }
  String __keyText() {
    if (__code >= 65 && __code <= 90) return "" + (char) __code;  // A-Z
    if (__code >= 48 && __code <= 57) return "" + (char) __code;  // 0-9
    if (__code == 10) return "Enter";
    if (__code == 32) return "Space";
    if (__code == 27) return "Esc";
    if (__code == 37) return "Left";
    if (__code == 38) return "Up";
    if (__code == 39) return "Right";
    if (__code == 40) return "Down";
    if (__code == 8) return "Backspace";
    if (__code == 9) return "Tab";
    if (__code == 127) return "Delete";
    return "" + (char) __code;
  }
}

interface ActionListener {
  void actionPerformed(ActionEvent e);
}

// javax.swing.Action + AbstractAction: a reusable command that can drive a
// JButton, JMenuItem, or JToolBar at once — sharing its name, tooltip
// (SHORT_DESCRIPTION), mnemonic, accelerator, enabled state, and its
// actionPerformed. Values are indexed by the string keys below (putValue /
// getValue), matching real Swing.
interface Action extends ActionListener {
  public static final String NAME = "Name";
  public static final String SHORT_DESCRIPTION = "ShortDescription";
  public static final String LONG_DESCRIPTION = "LongDescription";
  public static final String MNEMONIC_KEY = "MnemonicKey";
  public static final String ACCELERATOR_KEY = "AcceleratorKey";
  public static final String ACTION_COMMAND_KEY = "ActionCommandKey";
  public static final String SELECTED_KEY = "SwingSelectedKey";
  Object getValue(String key);
  void putValue(String key, Object value);
  boolean isEnabled();
  void setEnabled(boolean b);
}

abstract class AbstractAction implements Action {
  java.util.ArrayList<String> __keys = new java.util.ArrayList<String>();
  java.util.ArrayList<Object> __values = new java.util.ArrayList<Object>();
  boolean __enabled = true;
  public AbstractAction() {}
  public AbstractAction(String name) { putValue(Action.NAME, name); }
  public Object getValue(String key) {
    int i = __keys.indexOf(key);
    return i >= 0 ? __values.get(i) : null;
  }
  public void putValue(String key, Object value) {
    int i = __keys.indexOf(key);
    if (i >= 0) __values.set(i, value);
    else { __keys.add(key); __values.add(value); }
  }
  public boolean isEnabled() { return __enabled; }
  public void setEnabled(boolean b) { __enabled = b; }
}

class ItemEvent {
  public static final int SELECTED = 1;
  public static final int DESELECTED = 2;
  Object __src;
  int __state;
  Object __item;
  public ItemEvent(Object source, int state, Object item) {
    __src = source; __state = state; __item = item;
  }
  public Object getSource() { return __src; }
  public int getStateChange() { return __state; }
  public Object getItem() { return __item; }
}

interface ItemListener {
  void itemStateChanged(ItemEvent e);
}

class ChangeEvent {
  Object __src;
  public ChangeEvent(Object source) { __src = source; }
  public Object getSource() { return __src; }
}

interface ChangeListener {
  void stateChanged(ChangeEvent e);
}

// javax.swing.event.DocumentEvent / DocumentListener + javax.swing.text.Document:
// the text-change model. addDocumentListener fires insertUpdate / removeUpdate
// on every edit (changedUpdate is for attribute changes — not used for plain
// text, but included for source compatibility).
class DocumentEvent {
  Object __src;
  public DocumentEvent(Object source) { __src = source; }
  public Object getSource() { return __src; }
}

interface DocumentListener {
  void insertUpdate(DocumentEvent e);
  void removeUpdate(DocumentEvent e);
  void changedUpdate(DocumentEvent e);
}

class Document {
  Component __owner;
  DocumentListener __listener = null;
  Document(Component owner) { __owner = owner; }
  public void addDocumentListener(DocumentListener l) {
    __listener = l;
    __SwingRuntime.__interactive = true;
  }
  public int getLength() { return __owner.__lastText.length(); }
}

// A checked exception thrown by the text-offset queries (getLineStartOffset,
// getText, ...) when an offset or line is out of range — like the real JDK.
class BadLocationException extends Exception {
  int __offset;
  public BadLocationException(String message, int offset) { super(message); __offset = offset; }
  public int offsetRequested() { return __offset; }
}

class ListSelectionEvent {
  Object __src;
  public ListSelectionEvent(Object source) { __src = source; }
  public Object getSource() { return __src; }
  public boolean getValueIsAdjusting() { return false; }
}

// Fired by a ListModel (DefaultListModel) when its contents change.
class ListDataEvent {
  public static final int CONTENTS_CHANGED = 0;
  public static final int INTERVAL_ADDED = 1;
  public static final int INTERVAL_REMOVED = 2;
  Object __src;
  int __type, __i0, __i1;
  public ListDataEvent(Object source, int type, int index0, int index1) {
    __src = source;
    __type = type;
    __i0 = index0;
    __i1 = index1;
  }
  public Object getSource() { return __src; }
  public int getType() { return __type; }
  public int getIndex0() { return __i0; }
  public int getIndex1() { return __i1; }
}

interface ListDataListener {
  void intervalAdded(ListDataEvent e);
  void intervalRemoved(ListDataEvent e);
  void contentsChanged(ListDataEvent e);
}

// Fired by a TableModel (DefaultTableModel) when its data or structure changes.
class TableModelEvent {
  public static final int INSERT = 1;
  public static final int UPDATE = 0;
  public static final int DELETE = -1;
  public static final int HEADER_ROW = -1;
  public static final int ALL_COLUMNS = -1;
  Object __src;
  int __firstRow, __lastRow, __column, __type;
  public TableModelEvent(Object source, int firstRow, int lastRow, int column, int type) {
    __src = source;
    __firstRow = firstRow;
    __lastRow = lastRow;
    __column = column;
    __type = type;
  }
  public Object getSource() { return __src; }
  public int getFirstRow() { return __firstRow; }
  public int getLastRow() { return __lastRow; }
  public int getColumn() { return __column; }
  public int getType() { return __type; }
}

interface TableModelListener {
  void tableChanged(TableModelEvent e);
}

interface ListSelectionListener {
  void valueChanged(ListSelectionEvent e);
}

class MouseEvent {
  Object __src;
  int __x, __y;
  public MouseEvent(Object source, int x, int y) { __src = source; __x = x; __y = y; }
  public Object getSource() { return __src; }
  public int getX() { return __x; }
  public int getY() { return __y; }
  public int getButton() { return 1; } // BUTTON1
}

interface MouseListener {
  void mouseClicked(MouseEvent e);
  void mousePressed(MouseEvent e);
  void mouseReleased(MouseEvent e);
  void mouseEntered(MouseEvent e);
  void mouseExited(MouseEvent e);
}

interface MouseMotionListener {
  void mouseDragged(MouseEvent e);
  void mouseMoved(MouseEvent e);
}

// The mouse listener interfaces aren't functional (no lambdas), so students
// subclass this adapter and override only what they need — usually via
// `new MouseAdapter() { ... }`. Like java.awt.event.MouseAdapter it covers
// both MouseListener and MouseMotionListener.
class MouseAdapter implements MouseListener, MouseMotionListener {
  public void mouseClicked(MouseEvent e) {}
  public void mousePressed(MouseEvent e) {}
  public void mouseReleased(MouseEvent e) {}
  public void mouseEntered(MouseEvent e) {}
  public void mouseExited(MouseEvent e) {}
  public void mouseDragged(MouseEvent e) {}
  public void mouseMoved(MouseEvent e) {}
}

// A key press on the focused surface. getKeyCode() returns the AWT virtual-key
// code (VK_LEFT, VK_A, …, matching real java.awt.event.KeyEvent constants) and
// getKeyChar() the character typed (CHAR_UNDEFINED for action keys like arrows).
class KeyEvent {
  Object __src;
  int __code;
  char __ch;

  public static final char CHAR_UNDEFINED = (char) 65535;

  // Arrow / action keys (VK codes match java.awt.event.KeyEvent).
  public static final int VK_LEFT = 37;
  public static final int VK_UP = 38;
  public static final int VK_RIGHT = 39;
  public static final int VK_DOWN = 40;
  public static final int VK_ENTER = 10;
  public static final int VK_SPACE = 32;
  public static final int VK_ESCAPE = 27;
  public static final int VK_TAB = 9;
  public static final int VK_BACK_SPACE = 8;
  public static final int VK_SHIFT = 16;
  public static final int VK_CONTROL = 17;
  public static final int VK_ALT = 18;
  // Letters and digits share their uppercase ASCII value with the VK code
  // (VK_A == 'A' == 65, VK_0 == '0' == 48), so common WASD/arrow games work.
  public static final int VK_A = 65;
  public static final int VK_B = 66;
  public static final int VK_C = 67;
  public static final int VK_D = 68;
  public static final int VK_E = 69;
  public static final int VK_F = 70;
  public static final int VK_G = 71;
  public static final int VK_H = 72;
  public static final int VK_I = 73;
  public static final int VK_J = 74;
  public static final int VK_K = 75;
  public static final int VK_L = 76;
  public static final int VK_M = 77;
  public static final int VK_N = 78;
  public static final int VK_O = 79;
  public static final int VK_P = 80;
  public static final int VK_Q = 81;
  public static final int VK_R = 82;
  public static final int VK_S = 83;
  public static final int VK_T = 84;
  public static final int VK_U = 85;
  public static final int VK_V = 86;
  public static final int VK_W = 87;
  public static final int VK_X = 88;
  public static final int VK_Y = 89;
  public static final int VK_Z = 90;
  public static final int VK_0 = 48;
  public static final int VK_1 = 49;
  public static final int VK_2 = 50;
  public static final int VK_3 = 51;
  public static final int VK_4 = 52;
  public static final int VK_5 = 53;
  public static final int VK_6 = 54;
  public static final int VK_7 = 55;
  public static final int VK_8 = 56;
  public static final int VK_9 = 57;

  public KeyEvent(Object source, int code, char ch) { __src = source; __code = code; __ch = ch; }
  public Object getSource() { return __src; }
  public int getKeyCode() { return __code; }
  public char getKeyChar() { return __ch; }
}

interface KeyListener {
  void keyPressed(KeyEvent e);
  void keyReleased(KeyEvent e);
  void keyTyped(KeyEvent e);
}

// KeyListener isn't functional (three methods), so students subclass this
// adapter — usually `new KeyAdapter() { public void keyPressed(...) {...} }`.
class KeyAdapter implements KeyListener {
  public void keyPressed(KeyEvent e) {}
  public void keyReleased(KeyEvent e) {}
  public void keyTyped(KeyEvent e) {}
}

// The event pump. setVisible enters __loop, which renders the current tree
// and blocks on System.__uiAwait for the next event. The host returns the
// clicked component's id plus newline-separated "id=value" field states;
// we sync those into the widgets, then fire the clicked component's
// listener. All Swing dispatch stays here in Java — the VM only provides
// the one blocking native hook.
class __SwingRuntime {
  static boolean __interactive = false;
  static java.util.ArrayList<Component> __live = new java.util.ArrayList<Component>();
  static java.util.ArrayList<Timer> __timers = new java.util.ArrayList<Timer>();
  // A pending requestFocus target ("<cid>:<seq>"); the seq lets the same
  // component be focused again after focus moved away.
  static String __focusRequest = null;
  static int __focusSeq = 0;

  static void __requestFocus(String cid) {
    // Not marked interactive: even a static form's single (batch) render honors
    // the request, and an interactive app's live render does too.
    __focusSeq = __focusSeq + 1;
    __focusRequest = cid + ":" + __focusSeq;
  }

  static void __register(Component c) {
    for (int i = 0; i < __live.size(); i++) {
      if (__live.get(i).__cid.equals(c.__cid)) return;
    }
    __live.add(c);
  }

  static void __addTimer(Timer t) {
    for (int i = 0; i < __timers.size(); i++) {
      if (__timers.get(i).__tid.equals(t.__tid)) return;
    }
    __timers.add(t);
  }

  static Timer __findTimer(String tid) {
    for (int i = 0; i < __timers.size(); i++) {
      if (__timers.get(i).__tid.equals(tid)) return __timers.get(i);
    }
    return null;
  }

  // Running timers only (a stopped timer stays in the list but drops out
  // of the JSON so the host stops scheduling it).
  static String __timersJson() {
    StringBuilder s = new StringBuilder("[");
    boolean first = true;
    for (int i = 0; i < __timers.size(); i++) {
      Timer t = __timers.get(i);
      if (!t.__running) continue;
      if (!first) s.append(",");
      first = false;
      s.append("{\"id\":\"").append(t.__tid).append("\",\"delay\":").append(t.__delay).append("}");
    }
    return s.append("]").toString();
  }

  static Component __find(String cid) {
    for (int i = 0; i < __live.size(); i++) {
      Component c = __live.get(i);
      if (c.__cid.equals(cid)) return c;
    }
    return null;
  }

  static void __loop(JFrame frame) {
    while (true) {
      String payload = System.__uiAwait(frame.__jsonTree());
      if (payload == null) return; // host ended the session (Stop / no host)
      int nl = payload.indexOf("\n");
      String cid = nl < 0 ? payload : payload.substring(0, nl);
      // The window's close button: end the program for EXIT/DISPOSE (the
      // usual EXIT_ON_CLOSE), otherwise keep running (HIDE/DO_NOTHING).
      if (cid.equals("__close")) {
        if (frame.__closeOp == JFrame.EXIT_ON_CLOSE
            || frame.__closeOp == JFrame.DISPOSE_ON_CLOSE) {
          return;
        }
        continue;
      }
      String body = nl < 0 ? "" : payload.substring(nl + 1);
      __applyFields(body);
      // A timer tick: "__timer:<id>" — fire that timer's ActionListener.
      if (cid.startsWith("__timer:")) {
        Timer t = __findTimer(cid.substring(8));
        if (t != null && t.__running) t.__fire();
        continue;
      }
      Component c = __find(cid);
      if (c != null) {
        // A "__key=type,code,char" line is a keyboard event; "__drag=x,y" a
        // mouse drag; "__mouse=x,y" a click; a component with none of these is
        // a control activation (button/checkbox/…).
        int[] key = __keyOf(body);
        int[] drag = __coordOf(body, "__drag=");
        int[] click = __coordOf(body, "__mouse=");
        // A "__doc=" line marks a per-keystroke DocumentListener edit (fires
        // insert/remove) vs a normal control activation (button/Enter/...).
        if (__hasLine(body, "__doc=")) c.__onDoc();
        else if (key != null) c.__onKey(key[0], key[1], (char) key[2]);
        else if (drag != null) c.__onDrag(drag[0], drag[1]);
        else if (click != null) c.__onMouse(click[0], click[1]);
        else c.__onEvent();
      }
    }
  }

  // Whether the event body contains a line with the given prefix.
  static boolean __hasLine(String body, String prefix) {
    String rest = body;
    while (rest.length() > 0) {
      int nl = rest.indexOf("\n");
      String line = nl < 0 ? rest : rest.substring(0, nl);
      rest = nl < 0 ? "" : rest.substring(nl + 1);
      if (line.startsWith(prefix)) return true;
    }
    return false;
  }

  // Parse a "__key=type,code,char" line into {type, code, char}, or null.
  static int[] __keyOf(String body) {
    String rest = body;
    while (rest.length() > 0) {
      int nl = rest.indexOf("\n");
      String line = nl < 0 ? rest : rest.substring(0, nl);
      rest = nl < 0 ? "" : rest.substring(nl + 1);
      if (line.startsWith("__key=")) {
        String v = line.substring(6);
        int c1 = v.indexOf(",");
        if (c1 < 0) return null;
        int c2 = v.indexOf(",", c1 + 1);
        if (c2 < 0) return null;
        int type = Integer.parseInt(v.substring(0, c1));
        int code = Integer.parseInt(v.substring(c1 + 1, c2));
        int ch = Integer.parseInt(v.substring(c2 + 1));
        return new int[]{type, code, ch};
      }
    }
    return null;
  }

  // Parse an "<prefix>x,y" line out of the event body, or null if absent.
  static int[] __coordOf(String body, String prefix) {
    String rest = body;
    while (rest.length() > 0) {
      int nl = rest.indexOf("\n");
      String line = nl < 0 ? rest : rest.substring(0, nl);
      rest = nl < 0 ? "" : rest.substring(nl + 1);
      if (line.startsWith(prefix)) {
        String coords = line.substring(prefix.length());
        int comma = coords.indexOf(",");
        if (comma < 0) return null;
        int x = Integer.parseInt(coords.substring(0, comma));
        int y = Integer.parseInt(coords.substring(comma + 1));
        return new int[]{x, y};
      }
    }
    return null;
  }

  // Sync the DOM's live field values (sent with every event) back into the
  // widgets, so a listener reading getText()/isSelected() sees what the
  // user actually typed or toggled.
  static void __applyFields(String body) {
    String rest = body;
    while (rest.length() > 0) {
      int nl = rest.indexOf("\n");
      String line = nl < 0 ? rest : rest.substring(0, nl);
      rest = nl < 0 ? "" : rest.substring(nl + 1);
      int eq = line.indexOf("=");
      if (eq < 0) continue;
      String id = line.substring(0, eq);
      // Values are percent-escaped (%25 -> %, %0A -> newline) so a multi-line
      // JTextArea survives the newline-delimited payload. Decode %0A first.
      String value = line.substring(eq + 1).replace("%0A", "\n").replace("%25", "%");
      Component c = __find(id);
      if (c != null) c.__setFromHost(value);
    }
  }
}
