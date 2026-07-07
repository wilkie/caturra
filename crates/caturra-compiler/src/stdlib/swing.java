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
    // Only widgets with a listener dispatch on interaction; the renderer
    // wires a control's native event to the VM only when this is set, so an
    // input read at submit-time (no listener) doesn't round-trip on every key.
    if (__listens()) s += ",\"listens\":true";
    if (__ttip != null) s += ",\"tooltip\":\"" + Component.__esc(__ttip) + "\"";
    if (__bg != null) s += ",\"bg\":\"" + __bg.__r + "," + __bg.__g + "," + __bg.__b + "\"";
    if (__fg != null) s += ",\"fg\":\"" + __fg.__r + "," + __fg.__g + "," + __fg.__b + "\"";
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
  boolean __listens() { return false; }
}

class Container extends Component {
  java.util.ArrayList<Component> __kids = new java.util.ArrayList<Component>();
  LayoutManager __layout = null;

  public void add(Component c) { __kids.add(c); __SwingRuntime.__register(c); }
  // BorderLayout-style constraints are accepted but Phase 1 lays out in flow.
  public void add(Component c, Object constraints) { add(c); }
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

// java.awt.Graphics: a recorder. A custom JPanel's paintComponent draws into
// it; the commands are replayed onto a <canvas> by the renderer. Coordinates
// and the current color follow java.awt.Graphics (draw* strokes, fill* fills).
class Graphics {
  java.util.ArrayList<String> __cmds = new java.util.ArrayList<String>();
  public void setColor(Color c) { __cmds.add("setColor " + c.__r + " " + c.__g + " " + c.__b); }
  public void fillRect(int x, int y, int w, int h) { __cmds.add("fillRect " + x + " " + y + " " + w + " " + h); }
  public void drawRect(int x, int y, int w, int h) { __cmds.add("drawRect " + x + " " + y + " " + w + " " + h); }
  public void fillOval(int x, int y, int w, int h) { __cmds.add("fillOval " + x + " " + y + " " + w + " " + h); }
  public void drawOval(int x, int y, int w, int h) { __cmds.add("drawOval " + x + " " + y + " " + w + " " + h); }
  public void drawLine(int x1, int y1, int x2, int y2) { __cmds.add("drawLine " + x1 + " " + y1 + " " + x2 + " " + y2); }
  public void drawString(String s, int x, int y) { __cmds.add("drawString \"" + s + "\" " + x + " " + y); }
  boolean __empty() { return __cmds.size() == 0; }
  String __joined() {
    String out = "";
    for (int i = 0; i < __cmds.size(); i++) {
      if (i > 0) out += "\n";
      out += __cmds.get(i);
    }
    return out;
  }
}

class JPanel extends Container {
  int __pw = 200;
  int __ph = 150;
  MouseListener __mouseListener = null;
  MouseMotionListener __motionListener = null;
  public JPanel() {}
  public JPanel(LayoutManager m) { __layout = m; }
  public void setPreferredSize(Dimension d) { __pw = d.width; __ph = d.height; }
  public int getWidth() { return __pw; }
  public int getHeight() { return __ph; }
  public void addMouseListener(MouseListener l) { __mouseListener = l; __SwingRuntime.__interactive = true; }
  public void addMouseMotionListener(MouseMotionListener l) { __motionListener = l; __SwingRuntime.__interactive = true; }
  boolean __listens() { return __mouseListener != null || __motionListener != null; }
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
    // Which pointer events to wire: mouse (click) and/or drag (motion).
    String pointer = "";
    if (__mouseListener != null) pointer += ",\"mouse\":true";
    if (__motionListener != null) pointer += ",\"drag\":true";
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
    String opts = "[";
    for (int i = 0; i < __items.size(); i++) {
      if (i > 0) opts += ",";
      opts += "\"" + Component.__esc(__items.get(i)) + "\"";
    }
    opts += "]";
    return "{\"type\":\"combobox\",\"items\":" + opts + ",\"selectedIndex\":" + __selectedIndex
        + "," + __commonJson() + "}";
  }
}

// The selection modes, as constants (real Swing puts these on
// ListSelectionModel). JList defaults to MULTIPLE_INTERVAL_SELECTION.
class ListSelectionModel {
  public static final int SINGLE_SELECTION = 0;
  public static final int SINGLE_INTERVAL_SELECTION = 1;
  public static final int MULTIPLE_INTERVAL_SELECTION = 2;
}

class JList extends Component {
  java.util.ArrayList<String> __items = new java.util.ArrayList<String>();
  // Selected indices, in the order the host reports them.
  java.util.ArrayList<Integer> __selected = new java.util.ArrayList<Integer>();
  int __visibleRows = 8;
  int __mode = 2; // MULTIPLE_INTERVAL_SELECTION, matching real JList
  ListSelectionListener __listener = null;
  public JList() {}
  public JList(String[] items) { setListData(items); }
  public void setListData(String[] items) {
    __items = new java.util.ArrayList<String>();
    for (int i = 0; i < items.length; i++) __items.add(items[i]);
  }
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
    if (index < 0 || index >= __items.size()) return null;
    return __items.get(index);
  }
  public Object[] getSelectedValues() {
    Object[] out = new Object[__selected.size()];
    for (int i = 0; i < __selected.size(); i++) out[i] = __items.get(__selected.get(i));
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
    String opts = "[";
    for (int i = 0; i < __items.size(); i++) {
      if (i > 0) opts += ",";
      opts += "\"" + Component.__esc(__items.get(i)) + "\"";
    }
    opts += "]";
    String sel = "[";
    for (int i = 0; i < __selected.size(); i++) {
      if (i > 0) sel += ",";
      sel += __selected.get(i);
    }
    sel += "]";
    return "{\"type\":\"list\",\"items\":" + opts + ",\"selectedIndices\":" + sel
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

class JMenu {
  String __text;
  java.util.ArrayList<JMenuItem> __items = new java.util.ArrayList<JMenuItem>();
  public JMenu() { __text = ""; }
  public JMenu(String text) { __text = text; }
  public String getText() { return __text; }
  // Adding an item registers it so a click can find and fire it.
  public void add(JMenuItem item) { __items.add(item); __SwingRuntime.__register(item); }
  public void addSeparator() {
    JMenuItem s = new JMenuItem();
    s.__sep = true;
    __items.add(s);
  }
  public int getItemCount() { return __items.size(); }
  String __json() {
    String s = "{\"text\":\"" + Component.__esc(__text) + "\",\"items\":[";
    for (int i = 0; i < __items.size(); i++) {
      if (i > 0) s += ",";
      s += __items.get(i).__json();
    }
    return s + "]}";
  }
}

class JMenuBar {
  java.util.ArrayList<JMenu> __menus = new java.util.ArrayList<JMenu>();
  public JMenuBar() {}
  public void add(JMenu menu) { __menus.add(menu); }
  public int getMenuCount() { return __menus.size(); }
  String __json() {
    String s = "{\"menus\":[";
    for (int i = 0; i < __menus.size(); i++) {
      if (i > 0) s += ",";
      s += __menus.get(i).__json();
    }
    return s + "]}";
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
    String s = "[";
    boolean first = true;
    for (int i = 0; i < __timers.size(); i++) {
      Timer t = __timers.get(i);
      if (!t.__running) continue;
      if (!first) s += ",";
      first = false;
      s += "{\"id\":\"" + t.__tid + "\",\"delay\":" + t.__delay + "}";
    }
    return s + "]";
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
        // A "__drag=x,y" line is a mouse drag; "__mouse=x,y" is a click; a
        // component with neither is a control activation (button/checkbox/…).
        int[] drag = __coordOf(body, "__drag=");
        int[] click = __coordOf(body, "__mouse=");
        if (drag != null) c.__onDrag(drag[0], drag[1]);
        else if (click != null) c.__onMouse(click[0], click[1]);
        else c.__onEvent();
      }
    }
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
