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
  public JPanel() {}
  public JPanel(LayoutManager m) { __layout = m; }
  public void setPreferredSize(Dimension d) { __pw = d.width; __ph = d.height; }
  public int getWidth() { return __pw; }
  public int getHeight() { return __ph; }
  public void addMouseListener(MouseListener l) { __mouseListener = l; __SwingRuntime.__interactive = true; }
  boolean __listens() { return __mouseListener != null; }
  void __onMouse(int x, int y) {
    if (__mouseListener != null) {
      // A real click fires press, then release, then clicked.
      MouseEvent e = new MouseEvent(this, x, y);
      __mouseListener.mousePressed(e);
      __mouseListener.mouseReleased(e);
      __mouseListener.mouseClicked(e);
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
    return "{\"type\":\"panel\",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson() + paint + "," + __commonJson() + "}";
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

  public void setTitle(String title) { __title = title; }
  public String getTitle() { return __title; }
  public void setSize(int width, int height) { __w = width; __h = height; }
  public void setSize(Dimension d) { __w = d.width; __h = d.height; }
  public void setPreferredSize(Dimension d) { __w = d.width; __h = d.height; }
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
    return "{\"type\":\"frame\",\"title\":\"" + Component.__esc(__title) + "\",\"width\":" + __w
        + ",\"height\":" + __h + ",\"layout\":" + __layoutJson()
        + ",\"children\":" + __kidsJson()
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

// The five-method MouseListener isn't a functional interface (no lambdas), so
// students subclass this adapter and override only what they need — usually
// via `new MouseAdapter() { ... }`.
class MouseAdapter implements MouseListener {
  public void mouseClicked(MouseEvent e) {}
  public void mousePressed(MouseEvent e) {}
  public void mouseReleased(MouseEvent e) {}
  public void mouseEntered(MouseEvent e) {}
  public void mouseExited(MouseEvent e) {}
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
        // A "__mouse=x,y" line marks a pointer event on the component, which
        // dispatches to its MouseListener with coordinates; otherwise it's a
        // control activation (button/checkbox/…).
        int[] xy = __mouseOf(body);
        if (xy != null) c.__onMouse(xy[0], xy[1]);
        else c.__onEvent();
      }
    }
  }

  static int[] __mouseOf(String body) {
    String rest = body;
    while (rest.length() > 0) {
      int nl = rest.indexOf("\n");
      String line = nl < 0 ? rest : rest.substring(0, nl);
      rest = nl < 0 ? "" : rest.substring(nl + 1);
      if (line.startsWith("__mouse=")) {
        String coords = line.substring(8);
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
      String value = line.substring(eq + 1);
      Component c = __find(id);
      if (c != null) c.__setFromHost(value);
    }
  }
}
