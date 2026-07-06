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

// ----- Component model -----

class Component {
  static int __counter = 0;
  static int __nextId() { int n = __counter; __counter = __counter + 1; return n; }

  String __cid;
  boolean __enabled = true;
  String __ttip = null;
  Color __bg = null;
  Color __fg = null;

  Component() { __cid = "c" + __nextId(); }

  public void setEnabled(boolean enabled) { __enabled = enabled; }
  public boolean isEnabled() { return __enabled; }
  public void setToolTipText(String text) { __ttip = text; }
  public void setBackground(Color c) { __bg = c; }
  public void setForeground(Color c) { __fg = c; }

  // JSON escaping without char/StringBuilder ops the subset may not model.
  static String __esc(String s) {
    String out = "";
    for (int i = 0; i < s.length(); i++) {
      String ch = s.substring(i, i + 1);
      if (ch.equals("\"")) out += "\\\"";
      else if (ch.equals("\\")) out += "\\\\";
      else if (ch.equals("\n")) out += "\\n";
      else if (ch.equals("\t")) out += "\\t";
      else out += ch;
    }
    return out;
  }

  // The shared trailing fields every node carries (no braces, no leading comma).
  String __commonJson() {
    String s = "\"id\":\"" + __cid + "\",\"enabled\":" + __enabled;
    if (__ttip != null) s += ",\"tooltip\":\"" + Component.__esc(__ttip) + "\"";
    if (__bg != null) s += ",\"bg\":\"" + __bg.__r + "," + __bg.__g + "," + __bg.__b + "\"";
    if (__fg != null) s += ",\"fg\":\"" + __fg.__r + "," + __fg.__g + "," + __fg.__b + "\"";
    return s;
  }

  // Overridden by every concrete component; returns one JSON object.
  String __json() { return "{\"type\":\"component\"," + __commonJson() + "}"; }
}

class Container extends Component {
  java.util.ArrayList<Component> __kids = new java.util.ArrayList<Component>();
  LayoutManager __layout = null;

  public void add(Component c) { __kids.add(c); }
  // BorderLayout-style constraints are accepted but Phase 1 lays out in flow.
  public void add(Component c, Object constraints) { __kids.add(c); }
  public void setLayout(LayoutManager m) { __layout = m; }

  String __layoutJson() {
    if (__layout == null) return "\"flow\"";
    return "\"" + __layout.__desc() + "\"";
  }

  String __kidsJson() {
    String s = "[";
    for (int i = 0; i < __kids.size(); i++) {
      if (i > 0) s += ",";
      s += __kids.get(i).__json();
    }
    return s + "]";
  }
}

// ----- javax.swing widgets -----

class JPanel extends Container {
  public JPanel() {}
  public JPanel(LayoutManager m) { __layout = m; }
  String __json() {
    return "{\"type\":\"panel\",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson() + "," + __commonJson() + "}";
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
  public JButton() { __text = ""; }
  public JButton(String text) { __text = text; }
  public void setText(String text) { __text = text; }
  public String getText() { return __text; }
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
  String __json() {
    return "{\"type\":\"textfield\",\"text\":\"" + Component.__esc(__text) + "\",\"columns\":" + __cols
        + "," + __commonJson() + "}";
  }
}

class JCheckBox extends Component {
  String __text;
  boolean __sel;
  public JCheckBox() { __text = ""; __sel = false; }
  public JCheckBox(String text) { __text = text; __sel = false; }
  public JCheckBox(String text, boolean selected) { __text = text; __sel = selected; }
  public boolean isSelected() { return __sel; }
  public void setSelected(boolean selected) { __sel = selected; }
  public String getText() { return __text; }
  public void setText(String text) { __text = text; }
  String __json() {
    return "{\"type\":\"checkbox\",\"text\":\"" + Component.__esc(__text) + "\",\"selected\":" + __sel
        + "," + __commonJson() + "}";
  }
}

class JFrame extends Container {
  String __title;
  int __w = 0;
  int __h = 0;

  public JFrame() { __title = ""; }
  public JFrame(String title) { __title = title; }

  public void setTitle(String title) { __title = title; }
  public String getTitle() { return __title; }
  public void setSize(int width, int height) { __w = width; __h = height; }
  public void setSize(Dimension d) { __w = d.width; __h = d.height; }
  public void setPreferredSize(Dimension d) { __w = d.width; __h = d.height; }
  public void setDefaultCloseOperation(int op) {}
  public void setResizable(boolean resizable) {}
  public void setLocationRelativeTo(Object o) {}
  public void pack() {}

  public static final int EXIT_ON_CLOSE = 3;
  public static final int DISPOSE_ON_CLOSE = 2;
  public static final int HIDE_ON_CLOSE = 1;
  public static final int DO_NOTHING_ON_CLOSE = 0;

  public void setVisible(boolean visible) {
    if (visible) __render();
  }

  void __render() {
    String json = "{\"type\":\"frame\",\"title\":\"" + Component.__esc(__title) + "\",\"width\":" + __w
        + ",\"height\":" + __h + ",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson() + "," + __commonJson() + "}";
    try {
      java.io.PrintWriter w = new java.io.PrintWriter(new java.io.File("swing.json"));
      w.print(json);
      w.close();
    } catch (Exception e) {}
  }
}
