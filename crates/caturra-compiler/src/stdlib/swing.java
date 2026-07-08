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
  public int getRed() { return __r; }
  public int getGreen() { return __g; }
  public int getBlue() { return __b; }
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

interface LayoutManager {
  String __desc();
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
  String __ttip = null;
  Color __bg = null;
  Color __fg = null;
  // The BorderLayout region ("North".."Center") this component was added with,
  // or null when it carries no layout constraint.
  String __region = null;

  Component() { __cid = "c" + __nextId(); }

  public void setEnabled(boolean enabled) { __enabled = enabled; }
  public boolean isEnabled() { return __enabled; }
  public void setToolTipText(String text) { __ttip = text; }
  public void setBackground(Color c) { __bg = c; }
  public void setForeground(Color c) { __fg = c; }

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
    String s = "\"id\":\"" + __cid + "\",\"enabled\":" + __enabled;
    // Only widgets with a listener dispatch on interaction; the renderer
    // wires a control's native event to the VM only when this is set, so an
    // input read at submit-time (no listener) doesn't round-trip on every key.
    if (__listens()) s += ",\"listens\":true";
    if (__ttip != null) s += ",\"tooltip\":\"" + Component.__esc(__ttip) + "\"";
    if (__bg != null) s += ",\"bg\":\"" + __bg.__r + "," + __bg.__g + "," + __bg.__b + "\"";
    if (__fg != null) s += ",\"fg\":\"" + __fg.__r + "," + __fg.__g + "," + __fg.__b + "\"";
    if (__region != null) s += ",\"region\":\"" + Component.__esc(__region) + "\"";
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

class Container extends Component {
  java.util.ArrayList<Component> __kids = new java.util.ArrayList<Component>();
  LayoutManager __layout = null;

  public void add(Component c) { __kids.add(c); __SwingRuntime.__register(c); }
  // A BorderLayout constraint (e.g. BorderLayout.NORTH) records the region the
  // renderer places the child in; other layouts ignore it.
  public void add(Component c, Object constraints) {
    if (constraints != null) c.__region = "" + constraints;
    add(c);
  }
  public void setLayout(LayoutManager m) { __layout = m; }

  String __layoutJson() {
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

class JPanel extends Container {
  int __pw = 200;
  int __ph = 150;
  MouseListener __mouseListener = null;
  MouseMotionListener __motionListener = null;
  KeyListener __keyListener = null;
  public JPanel() {}
  public JPanel(LayoutManager m) { __layout = m; }
  public void setPreferredSize(Dimension d) { __pw = d.width; __ph = d.height; }
  public int getWidth() { return __pw; }
  public int getHeight() { return __ph; }
  public void addMouseListener(MouseListener l) { __mouseListener = l; __SwingRuntime.__interactive = true; }
  public void addMouseMotionListener(MouseMotionListener l) { __motionListener = l; __SwingRuntime.__interactive = true; }
  public void addKeyListener(KeyListener l) { __keyListener = l; __SwingRuntime.__interactive = true; }
  // A panel receiving key events must be focusable and hold focus. The renderer
  // handles both when a key listener is attached, so these are accepted no-ops
  // for source compatibility with real Swing.
  public void setFocusable(boolean b) {}
  public void requestFocus() {}
  public void requestFocusInWindow() {}
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
    Graphics g = new Graphics();
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

class JLabel extends Component {
  String __text;
  Component __labelFor = null;
  public JLabel() { __text = ""; }
  public JLabel(String text) { __text = text; }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public void setLabelFor(Component c) { __labelFor = c; }
  String __json() {
    String f = __labelFor == null ? "" : ",\"for\":\"" + __labelFor.__cid + "\"";
    return "{\"type\":\"label\",\"text\":\"" + Component.__esc(__text) + "\"" + f + "," + __commonJson() + "}";
  }
}

class JButton extends Component {
  String __text;
  ActionListener __listener = null;
  public JButton() { __text = ""; }
  public JButton(String text) { __text = text; }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public void addActionListener(ActionListener l) {
    __listener = l;
    __SwingRuntime.__interactive = true;
  }
  void __onEvent() {
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this));
  }
  boolean __listens() { return __listener != null; }
  String __json() {
    return "{\"type\":\"button\",\"text\":\"" + Component.__esc(__text) + "\"," + __commonJson() + "}";
  }
}

class JTextField extends Component {
  String __text;
  int __cols;
  public JTextField() { __text = ""; __cols = 0; }
  public JTextField(int cols) { __text = ""; __cols = cols; }
  public JTextField(String text) { __text = text; __cols = 0; }
  public JTextField(String text, int cols) { __text = text; __cols = cols; }
  public String getText() { return __text; }
  public void setText(String text) { __text = text; }
  public int getColumns() { return __cols; }
  void __setFromHost(String value) { __text = value; }
  String __json() {
    return "{\"type\":\"textfield\",\"text\":\"" + Component.__esc(__text) + "\",\"columns\":" + __cols
        + "," + __commonJson() + "}";
  }
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
  public void setEditable(boolean editable) { __editable = editable; }
  public boolean isEditable() { return __editable; }
  public void setLineWrap(boolean wrap) { __wrap = wrap; }
  public void setWrapStyleWord(boolean word) {}
  public int getRows() { return __rows; }
  public int getColumns() { return __cols; }
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
  public ButtonGroup() { __gid = "g" + Component.__nextId(); }
  public void add(JRadioButton b) { b.__group = __gid; __buttons.add(b); }
  public int getButtonCount() { return __buttons.size(); }
}

class JComboBox extends Component {
  java.util.ArrayList<String> __items = new java.util.ArrayList<String>();
  int __selectedIndex = -1;
  ActionListener __actionListener = null;
  public JComboBox() {}
  public JComboBox(String[] items) {
    for (int i = 0; i < items.length; i++) __items.add(items[i]);
    if (__items.size() > 0) __selectedIndex = 0;
  }
  public void addItem(String item) {
    __items.add(item);
    if (__selectedIndex < 0) __selectedIndex = 0;
  }
  public int getItemCount() { return __items.size(); }
  public String getItemAt(int index) { return __items.get(index); }
  public int getSelectedIndex() { return __selectedIndex; }
  public void setSelectedIndex(int index) { __selectedIndex = index; }
  public Object getSelectedItem() {
    if (__selectedIndex < 0 || __selectedIndex >= __items.size()) return null;
    return __items.get(__selectedIndex);
  }
  public void addActionListener(ActionListener l) { __actionListener = l; __SwingRuntime.__interactive = true; }
  void __setFromHost(String value) { __selectedIndex = Integer.parseInt(value); }
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
    return "{\"type\":\"combobox\",\"items\":" + opts.toString() + ",\"selectedIndex\":" + __selectedIndex
        + "," + __commonJson() + "}";
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
  public DefaultListModel() {}
  public void addElement(String element) { __elements.add(element); }
  public void add(int index, String element) { __elements.add(index, element); }
  public String get(int index) { return __elements.get(index); }
  public String getElementAt(int index) { return __elements.get(index); }
  public String elementAt(int index) { return __elements.get(index); }
  public String set(int index, String element) { return __elements.set(index, element); }
  public String remove(int index) { return __elements.remove(index); }
  // Element params are String (the model holds Strings), where real Swing uses
  // Object — the bundled ArrayList types these overloads by element type.
  public boolean removeElement(String element) { return __elements.remove(element); }
  public void removeAllElements() { __elements.clear(); }
  public void clear() { __elements.clear(); }
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
  public DefaultTableModel() {}
  public DefaultTableModel(Object[][] data, Object[] columnNames) {
    for (int c = 0; c < columnNames.length; c++) __columns.add(__str(columnNames[c]));
    for (int r = 0; r < data.length; r++) addRow(data[r]);
  }
  static String __str(Object v) { return v == null ? "" : "" + v; }
  public int getColumnCount() { return __columns.size(); }
  public int getRowCount() { return __columns.isEmpty() ? 0 : __cells.size() / __columns.size(); }
  public String getColumnName(int col) { return __columns.get(col); }
  public Object getValueAt(int row, int col) { return __cells.get(row * __columns.size() + col); }
  public void setValueAt(Object value, int row, int col) {
    __cells.set(row * __columns.size() + col, __str(value));
  }
  // Cells are editable by default; override this to make (some) read-only.
  public boolean isCellEditable(int row, int col) { return true; }
  public void addRow(Object[] rowData) {
    for (int c = 0; c < __columns.size(); c++) {
      __cells.add(__str(c < rowData.length ? rowData[c] : null));
    }
  }
  public void insertRow(int row, Object[] rowData) {
    int base = row * __columns.size();
    for (int c = 0; c < __columns.size(); c++) {
      __cells.add(base + c, __str(c < rowData.length ? rowData[c] : null));
    }
  }
  public void removeRow(int row) {
    int cols = __columns.size();
    for (int c = 0; c < cols; c++) __cells.remove(row * cols);
  }
  public void addColumn(Object columnName) {
    int oldCols = __columns.size();
    int rows = oldCols == 0 ? 0 : __cells.size() / oldCols;
    __columns.add(__str(columnName));
    for (int r = rows - 1; r >= 0; r--) __cells.add((r + 1) * oldCols, "");
  }
  public void setRowCount(int rowCount) {
    int cols = __columns.size();
    int target = rowCount * cols;
    while (__cells.size() > target) __cells.remove(__cells.size() - 1);
    while (__cells.size() < target) __cells.add("");
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
  public JMenuItem() { __text = ""; }
  public JMenuItem(String text) { __text = text; }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
  public void addActionListener(ActionListener l) { __listener = l; __SwingRuntime.__interactive = true; }
  void __onEvent() {
    if (__listener != null) __listener.actionPerformed(new ActionEvent(this));
  }
  boolean __listens() { return __listener != null; }
  String __json() {
    if (__sep) return "{\"type\":\"separator\"}";
    return "{\"type\":\"menuitem\",\"text\":\"" + Component.__esc(__text) + "\"," + __commonJson() + "}";
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
    return "{\"type\":\"frame\",\"title\":\"" + Component.__esc(__title) + "\",\"width\":" + __w
        + ",\"height\":" + __h + ",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson() + menubar
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
  Object __src;
  public ActionEvent(Object source) { __src = source; }
  public Object getSource() { return __src; }
}

interface ActionListener {
  void actionPerformed(ActionEvent e);
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

class ListSelectionEvent {
  Object __src;
  public ListSelectionEvent(Object source) { __src = source; }
  public Object getSource() { return __src; }
  public boolean getValueIsAdjusting() { return false; }
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
        if (key != null) c.__onKey(key[0], key[1], (char) key[2]);
        else if (drag != null) c.__onDrag(drag[0], drag[1]);
        else if (click != null) c.__onMouse(click[0], click[1]);
        else c.__onEvent();
      }
    }
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
