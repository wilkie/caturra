import { useEffect, useRef, useState } from 'react';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import TextField from '@mui/material/TextField';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogActions from '@mui/material/DialogActions';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import BugReportIcon from '@mui/icons-material/BugReport';
import ScienceIcon from '@mui/icons-material/Science';
import StopIcon from '@mui/icons-material/Stop';
import AddIcon from '@mui/icons-material/Add';
import CloseIcon from '@mui/icons-material/Close';
import LightbulbIcon from '@mui/icons-material/Lightbulb';
import {
  JvmWorkerSession,
  type DebugCommandName,
  type DebugControlResponse,
  type DebugPauseSnapshot,
  type Diagnostic,
} from '@caturra/core';
import type { EditorState } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
import {
  breakpointLines,
  breakpointLinesInState,
  createEditor,
  createFileState,
  getSource,
  setPausedLine,
  setSource,
  showDiagnostics,
  toggleBreakpointAtLine,
  type SourceSquiggle,
} from './editor.js';
import { NeighborhoodViz, type NeighborhoodState } from './neighborhood.js';
import { TheaterViz } from './theater.js';
import { SwingViz } from './swing.js';
import {
  CSA_UNITS,
  levelHasSolution,
  loadLevel,
  type CsaLevelFile,
  type CsaLevelMeta,
} from './levels.js';

// Stable automation hooks (Playwright drives the editor through these
// rather than through CodeMirror's contenteditable internals).
declare global {
  interface Window {
    playground: {
      setSource: (text: string) => void;
      getSource: () => string;
      toggleBreakpoint: (line: number) => void;
      breakpointLines: () => number[];
      setFile: (name: string, text: string) => void;
      selectFile: (name: string) => void;
      activeFile: () => string;
      levelReady: () => Promise<void>;
      neighborhoodState: () => NeighborhoodState;
    };
  }
}

interface TheaterLevel {
  name: string;
  starter: string;
}

// Theater examples: each starter draws with org.code.theater onto the
// 400x400 stage (shapes/text are fully rendered; images are placeholders).
const THEATER_LEVELS: TheaterLevel[] = [
  {
    name: 'Shapes',
    starter: `import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {
    Scene scene = new Scene();
    scene.clear("aqua");

    scene.setFillColor(new Color(120, 190, 255));
    scene.setStrokeColor(new Color(20, 40, 80));
    scene.setStrokeWidth(3.0);
    scene.drawRectangle(40, 60, 320, 180);

    scene.setFillColor(Color.YELLOW);
    scene.drawEllipse(150, 40, 100, 100);

    scene.setStrokeColor(Color.RED);
    scene.setStrokeWidth(4.0);
    scene.drawLine(0, 380, 400, 300);

    scene.setFillColor(Color.GREEN);
    scene.setStrokeColor(new Color(0, 60, 0));
    scene.drawRegularPolygon(300, 320, 6, 50);

    scene.setTextColor("black");
    scene.setTextHeight(32);
    scene.setTextStyle(Font.SANS, FontStyle.BOLD);
    scene.drawText("Hello Theater", 55, 300);

    Theater.playScenes(scene);
  }
}
`,
  },
  {
    name: 'Animation',
    starter: `import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {
    Scene scene = new Scene();
    String[] colors = {"red", "orange", "yellow", "green", "blue", "purple"};

    // Each pause() is one frame: an expanding ring of rainbow color.
    for (int i = 0; i < 18; i++) {
      scene.clear("white");
      scene.setFillColor(new Color(colors[i % colors.length]));
      int r = 20 + i * 10;
      scene.drawEllipse(200 - r, 200 - r, 2 * r, 2 * r);

      scene.setTextColor("black");
      scene.setTextHeight(22);
      scene.setTextStyle(Font.MONO, FontStyle.NORMAL);
      scene.drawText("frame " + i, 150, 30);
      scene.pause(0.15);
    }
    Theater.playScenes(scene);
  }
}
`,
  },
];

interface SwingLevel {
  name: string;
  starter: string;
}

// Swing examples: each starter builds a javax.swing component tree that
// renders as accessible DOM (real <button>/<input>/<label> elements with
// keyboard navigation and screen-reader names). Interactivity is Phase 2.
const SWING_LEVELS: SwingLevel[] = [
  {
    name: 'Click counter',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static int count = 0;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Counter");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new GridLayout(2, 1));
    JLabel label = new JLabel("Clicks: 0");
    JButton button = new JButton("Click me");

    // The listener runs in the VM on every click, updates the label,
    // and the window re-renders. Try it with the keyboard too: Tab to
    // the button and press Enter or Space.
    button.addActionListener(e -> {
      Main.count++;
      label.setText("Clicks: " + Main.count);
    });

    frame.add(label);
    frame.add(button);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Greeter form',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Greeter");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new GridLayout(3, 2));

    JLabel prompt = new JLabel("Name:");
    JTextField field = new JTextField(12);
    prompt.setLabelFor(field);

    JLabel greeting = new JLabel("Type a name, then Greet.");
    JButton greet = new JButton("Greet");

    // Clicking reads whatever the user typed into the field.
    greet.addActionListener(e -> {
      greeting.setText("Hello, " + field.getText() + "!");
    });

    frame.add(prompt);
    frame.add(field);
    frame.add(greet);
    frame.add(greeting);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Controls demo',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel summary;
  static JComboBox size;
  static JSlider quantity;
  static JCheckBox extraCheese;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Pizza order");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new GridLayout(4, 2));

    JLabel sizeLabel = new JLabel("Size:");
    size = new JComboBox(new String[]{"Small", "Medium", "Large"});
    sizeLabel.setLabelFor(size);

    JLabel qtyLabel = new JLabel("Quantity:");
    quantity = new JSlider(1, 6, 1);
    qtyLabel.setLabelFor(quantity);

    extraCheese = new JCheckBox("Extra cheese");
    summary = new JLabel("1 Small pizza");

    // Each control fires its own kind of listener; update() rebuilds the
    // order summary from the live widget state.
    size.addActionListener(e -> Main.update());
    quantity.addChangeListener(e -> Main.update());
    extraCheese.addItemListener(e -> Main.update());

    frame.add(sizeLabel);
    frame.add(size);
    frame.add(qtyLabel);
    frame.add(quantity);
    frame.add(new JLabel(""));
    frame.add(extraCheese);
    frame.add(new JLabel("Order:"));
    frame.add(summary);
    frame.setVisible(true);
  }

  static void update() {
    String order = quantity.getValue() + " " + size.getSelectedItem() + " pizza";
    if (extraCheese.isSelected()) order = order + " with extra cheese";
    summary.setText(order);
  }
}
`,
  },
  {
    name: 'Custom drawing',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static int radius = 20;
  static DrawPanel panel;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Grow");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    panel = new DrawPanel();
    panel.setPreferredSize(new Dimension(240, 200));
    panel.setToolTipText("An orange circle you can grow");

    JButton grow = new JButton("Grow");
    // Clicking changes state and requests a repaint; the next render
    // re-runs paintComponent, so the circle redraws at the new size.
    grow.addActionListener(e -> {
      Main.radius = Main.radius + 15;
      Main.panel.repaint();
    });

    frame.add(panel, BorderLayout.CENTER);
    frame.add(grow, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}

// A custom drawing surface: override paintComponent and draw with Graphics.
class DrawPanel extends JPanel {
  public void paintComponent(Graphics g) {
    super.paintComponent(g);
    g.setColor(new Color(230, 240, 255));
    g.fillRect(0, 0, getWidth(), getHeight());

    int r = Main.radius;
    g.setColor(new Color(255, 140, 0));
    g.fillOval(120 - r, 100 - r, 2 * r, 2 * r);

    g.setColor(Color.BLACK);
    g.drawString("r = " + Main.radius, 10, 20);
  }
}
`,
  },
  {
    name: 'Click to draw',
    starter: `import javax.swing.*;
import java.awt.*;
import java.awt.event.*;
import java.util.ArrayList;

public class Main {
  static ArrayList<Integer> xs = new ArrayList<Integer>();
  static ArrayList<Integer> ys = new ArrayList<Integer>();
  static DotPanel panel;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Dots");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);

    panel = new DotPanel();
    panel.setPreferredSize(new Dimension(300, 220));
    panel.setToolTipText("Click to add a dot");

    // MouseListener has five methods, so subclass MouseAdapter and override
    // just the one you need. getX()/getY() are canvas coordinates.
    panel.addMouseListener(new MouseAdapter() {
      public void mousePressed(MouseEvent e) {
        Main.xs.add(e.getX());
        Main.ys.add(e.getY());
        Main.panel.repaint();
      }
    });

    frame.add(panel);
    frame.setVisible(true);
  }
}

class DotPanel extends JPanel {
  public void paintComponent(Graphics g) {
    super.paintComponent(g);
    g.setColor(new Color(245, 245, 250));
    g.fillRect(0, 0, getWidth(), getHeight());
    g.setColor(new Color(30, 90, 200));
    for (int i = 0; i < Main.xs.size(); i++) {
      int x = Main.xs.get(i);
      int y = Main.ys.get(i);
      g.fillOval(x - 6, y - 6, 12, 12);
    }
  }
}
`,
  },
  {
    name: 'Bouncing ball',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static int x = 30;
  static int dx = 6;
  static int ticks = 0;
  static BallPanel panel;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Bounce");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    panel = new BallPanel();
    panel.setPreferredSize(new Dimension(300, 160));
    panel.setToolTipText("A bouncing ball animation");
    status = new JLabel("ticks: 0");

    // A Timer fires its ActionListener every 40ms: move the ball, bounce at
    // the edges, and repaint. The host schedules the ticks.
    Timer timer = new Timer(40, e -> {
      Main.x = Main.x + Main.dx;
      if (Main.x < 20 || Main.x > 280) Main.dx = -Main.dx;
      Main.ticks = Main.ticks + 1;
      Main.status.setText("ticks: " + Main.ticks);
      Main.panel.repaint();
    });
    timer.start();

    frame.add(panel, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}

class BallPanel extends JPanel {
  public void paintComponent(Graphics g) {
    super.paintComponent(g);
    g.setColor(new Color(20, 24, 40));
    g.fillRect(0, 0, getWidth(), getHeight());
    g.setColor(new Color(255, 200, 0));
    g.fillOval(Main.x - 15, 65, 30, 30);
  }
}
`,
  },
  {
    name: 'Sketch pad',
    starter: `import javax.swing.*;
import java.awt.*;
import java.awt.event.*;
import java.util.ArrayList;

public class Main {
  static ArrayList<Integer> xs = new ArrayList<Integer>();
  static ArrayList<Integer> ys = new ArrayList<Integer>();
  static Pad panel;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Sketch");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    panel = new Pad();
    panel.setPreferredSize(new Dimension(320, 220));
    panel.setToolTipText("Drag to draw");

    // Dragging fires many mouseDragged events; the playground coalesces them
    // so only the latest position is processed each frame. Connecting the
    // points with lines keeps the stroke continuous.
    panel.addMouseMotionListener(new MouseAdapter() {
      public void mouseDragged(MouseEvent e) {
        Main.xs.add(e.getX());
        Main.ys.add(e.getY());
        Main.panel.repaint();
      }
    });

    frame.add(panel);
    frame.setVisible(true);
  }
}

class Pad extends JPanel {
  public void paintComponent(Graphics g) {
    super.paintComponent(g);
    g.setColor(new Color(250, 250, 252));
    g.fillRect(0, 0, getWidth(), getHeight());
    g.setColor(new Color(180, 30, 90));
    for (int i = 1; i < Main.xs.size(); i++) {
      g.drawLine(Main.xs.get(i - 1), Main.ys.get(i - 1), Main.xs.get(i), Main.ys.get(i));
    }
  }
}
`,
  },
  {
    name: 'Dialogs (JOptionPane)',
    starter: `import javax.swing.*;

public class Main {
  public static void main(String[] args) {
    // Each JOptionPane call blocks until you answer it — no JFrame needed.
    String name = JOptionPane.showInputDialog("What's your name?");
    if (name == null || name.equals("")) {
      name = "stranger";
    }

    int choice = JOptionPane.showConfirmDialog(
        null, "Nice to meet you, " + name + ".\\nShow a greeting?");

    if (choice == JOptionPane.YES_OPTION) {
      JOptionPane.showMessageDialog(null, "Hello, " + name + "!");
    } else {
      JOptionPane.showMessageDialog(null, "No problem.");
    }
  }
}
`,
  },
  {
    name: 'BorderLayout regions',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Layout");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // NORTH: a toolbar button spanning the top.
    JButton refresh = new JButton("Refresh");
    refresh.addActionListener(e -> Main.status.setText("Refreshed the view"));
    frame.add(refresh, BorderLayout.NORTH);

    // WEST: a side action down the left edge.
    JButton west = new JButton("Sidebar");
    west.addActionListener(e -> Main.status.setText("Opened the sidebar"));
    frame.add(west, BorderLayout.WEST);

    // EAST: an info panel on the right.
    frame.add(new JLabel("Details"), BorderLayout.EAST);

    // CENTER: the main content, which soaks up the remaining space.
    JTextArea center = new JTextArea(6, 24);
    center.setText("Main content area.\\nBorderLayout gives this the center.");
    center.setLineWrap(true);
    frame.add(center, BorderLayout.CENTER);

    // SOUTH: a status bar spanning the bottom.
    status = new JLabel("Ready.");
    frame.add(status, BorderLayout.SOUTH);

    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Menus (JMenuBar)',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Notes");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);

    JMenuBar bar = new JMenuBar();

    JMenu file = new JMenu("File");
    JMenuItem newItem = new JMenuItem("New");
    newItem.addActionListener(e -> Main.status.setText("New file"));
    JMenuItem openItem = new JMenuItem("Open");
    openItem.addActionListener(e -> Main.status.setText("Opened a file"));

    // A nested submenu: File > Export > (PNG | JPEG).
    JMenu export = new JMenu("Export");
    JMenuItem png = new JMenuItem("PNG");
    png.addActionListener(e -> Main.status.setText("Exported PNG"));
    JMenuItem jpeg = new JMenuItem("JPEG");
    jpeg.addActionListener(e -> Main.status.setText("Exported JPEG"));
    export.add(png);
    export.add(jpeg);

    JMenuItem quitItem = new JMenuItem("Quit");
    quitItem.addActionListener(e -> Main.status.setText("Bye!"));
    file.add(newItem);
    file.add(openItem);
    file.add(export);
    file.addSeparator();
    file.add(quitItem);

    JMenu help = new JMenu("Help");
    JMenuItem about = new JMenuItem("About");
    about.addActionListener(e -> JOptionPane.showMessageDialog(null, "Notes 1.0"));
    help.add(about);

    bar.add(file);
    bar.add(help);
    frame.setJMenuBar(bar);

    status = new JLabel("Choose a menu item.");
    frame.add(status);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Notepad (JTextArea)',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JTextArea notes;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Notepad");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JLabel prompt = new JLabel("Notes:");
    notes = new JTextArea(6, 30);
    notes.setLineWrap(true);
    prompt.setLabelFor(notes);

    status = new JLabel("Type something, then Count.");
    JButton count = new JButton("Count");
    count.addActionListener(e -> {
      String text = Main.notes.getText();
      int lines = text.equals("") ? 0 : 1;
      for (int i = 0; i < text.length(); i++) {
        if (text.charAt(i) == '\\n') {
          lines++;
        }
      }
      Main.status.setText(lines + " line(s), " + text.length() + " chars");
    });

    // NORTH label, CENTER text area, and a SOUTH bar holding the button + status.
    JPanel bottom = new JPanel();
    bottom.add(count);
    bottom.add(status);
    frame.add(prompt, BorderLayout.NORTH);
    frame.add(notes, BorderLayout.CENTER);
    frame.add(bottom, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Scroll pane (JScrollPane)',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Scroll");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // A tall list of buttons wrapped in a small scroll pane.
    JPanel list = new JPanel(new GridLayout(20, 1));
    for (int i = 1; i <= 20; i++) {
      JButton item = new JButton("Item " + i);
      item.addActionListener(
          e -> Main.status.setText("Picked " + ((JButton) e.getSource()).getText()));
      list.add(item);
    }

    JScrollPane scroll = new JScrollPane(list);
    scroll.setPreferredSize(new Dimension(220, 160));

    status = new JLabel("Scroll the list and pick an item.");

    frame.add(scroll, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'List (JList)',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JLabel status;
  static JList fruits;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Fruits");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JLabel prompt = new JLabel("Pick a fruit:");
    fruits = new JList(new String[]{
        "Apple", "Banana", "Cherry", "Date", "Elderberry", "Fig", "Grape"});
    fruits.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
    fruits.setVisibleRowCount(5);
    prompt.setLabelFor(fruits);

    status = new JLabel("Nothing selected yet.");
    fruits.addListSelectionListener(e -> {
      Object picked = Main.fruits.getSelectedValue();
      Main.status.setText(picked == null ? "Nothing selected." : "You picked " + picked);
    });

    frame.add(prompt, BorderLayout.NORTH);
    frame.add(fruits, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Multi-select list (JList)',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JLabel status;
  static JList toppings;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Pizza");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JLabel prompt = new JLabel("Choose toppings (Ctrl/Shift-click for several):");
    toppings = new JList(new String[]{
        "Cheese", "Mushroom", "Pepperoni", "Onion", "Olive", "Pepper"});
    // MULTIPLE_INTERVAL_SELECTION is the JList default; shown here for clarity.
    toppings.setSelectionMode(ListSelectionModel.MULTIPLE_INTERVAL_SELECTION);
    prompt.setLabelFor(toppings);

    status = new JLabel("No toppings yet.");
    toppings.addListSelectionListener(e -> {
      Object[] picks = Main.toppings.getSelectedValues();
      if (picks.length == 0) {
        Main.status.setText("No toppings yet.");
        return;
      }
      String list = "";
      for (int i = 0; i < picks.length; i++) {
        if (i > 0) list += ", ";
        list += picks[i];
      }
      Main.status.setText(picks.length + " topping(s): " + list);
    });

    frame.add(prompt, BorderLayout.NORTH);
    frame.add(toppings, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'To-do list (DefaultListModel)',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static DefaultListModel model;
  static JList list;
  static JTextField field;

  public static void main(String[] args) {
    JFrame frame = new JFrame("To-do");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // The model holds the list's contents; mutating it updates the JList.
    model = new DefaultListModel();
    model.addElement("Buy milk");
    model.addElement("Walk the dog");

    list = new JList(model);
    list.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
    list.setVisibleRowCount(6);

    // NORTH: a field + Add button to append a new item.
    field = new JTextField(16);
    JButton add = new JButton("Add");
    add.addActionListener(e -> {
      String text = Main.field.getText();
      if (!text.equals("")) {
        Main.model.addElement(text);
        Main.field.setText("");
      }
    });
    JPanel top = new JPanel();
    top.add(field);
    top.add(add);

    // SOUTH: remove the selected item.
    JButton remove = new JButton("Remove selected");
    remove.addActionListener(e -> {
      int i = Main.list.getSelectedIndex();
      if (i >= 0) {
        Main.model.remove(i);
      }
    });

    frame.add(top, BorderLayout.NORTH);
    frame.add(list, BorderLayout.CENTER);
    frame.add(remove, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Sign-up form',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Sign Up");
    frame.setLayout(new GridLayout(3, 2));

    // setLabelFor ties each label to its field, so a screen reader
    // announces "Name, edit text" when the field is focused.
    JLabel nameLabel = new JLabel("Name:");
    JTextField nameField = new JTextField(12);
    nameLabel.setLabelFor(nameField);

    JLabel emailLabel = new JLabel("Email:");
    JTextField emailField = new JTextField(12);
    emailLabel.setLabelFor(emailField);

    JCheckBox subscribe = new JCheckBox("Email me updates", true);
    JButton submit = new JButton("Submit");

    frame.add(nameLabel);
    frame.add(nameField);
    frame.add(emailLabel);
    frame.add(emailField);
    frame.add(subscribe);
    frame.add(submit);

    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Button panel',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Calculator");
    JPanel keypad = new JPanel(new GridLayout(4, 3));
    String[] keys = {"7","8","9","4","5","6","1","2","3","0",".","="};
    for (String key : keys) {
      keypad.add(new JButton(key));
    }
    frame.add(keypad);
    frame.setVisible(true);
  }
}
`,
  },
];

const DEFAULT_PROGRAM = `public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
`;

interface TestResult {
  passed: boolean;
  name: string;
  message: string;
}

interface ConsoleLine {
  text: string;
  kind: 'normal' | 'error';
}

type Phase = 'idle' | 'running' | 'debugging' | 'testing';

/** Parse the runner's `__VTEST\t<PASS|FAIL>\t<name>\t<message>` lines. */
function parseTestResults(output: string): TestResult[] {
  const results: TestResult[] = [];
  for (const line of output.split('\n')) {
    if (!line.startsWith('__VTEST\t')) {
      continue;
    }
    const [, status, name, message] = line.split('\t');
    results.push({ passed: status === 'PASS', name: name ?? '', message: message ?? '' });
  }
  return results;
}

/** Running Swing timers from a serialized component tree (for wakeups). */
function parseSwingTimers(tree: string): { id: string; delay: number }[] {
  try {
    const root = JSON.parse(tree) as { timers?: { id: string; delay: number }[] };
    return root.timers ?? [];
  } catch {
    return [];
  }
}

function formatDiagnostic(diagnostic: Diagnostic): string {
  const location = diagnostic.start
    ? `${diagnostic.path}:${String(diagnostic.start.line)}:${String(diagnostic.start.column)}`
    : diagnostic.path;
  return `${location}: ${diagnostic.severity}: ${diagnostic.message}\n`;
}

export function App(): React.JSX.Element {
  // ----- Reactive UI state -----
  const [version, setVersion] = useState('');
  const [ready, setReady] = useState(false);
  const [phase, setPhase] = useState<Phase>('idle');
  const [consoleLines, setConsoleLines] = useState<ConsoleLine[]>([]);
  const [fileNames, setFileNames] = useState<string[]>(['Main.java']);
  const [activeFile, setActiveFile] = useState('Main.java');
  const [unitIndex, setUnitIndex] = useState('');
  const [levelValue, setLevelValue] = useState('');
  const [theaterValue, setTheaterValue] = useState('');
  const [swingValue, setSwingValue] = useState('');
  const [levels, setLevels] = useState<CsaLevelMeta[]>([]);
  const [view, setView] = useState<'none' | 'neighborhood' | 'theater' | 'swing'>('none');
  const [swingDialog, setSwingDialog] = useState<{ kind: string; message: string } | null>(null);
  const [swingDialogInput, setSwingDialogInput] = useState('');
  const [testResults, setTestResults] = useState<TestResult[]>([]);
  const [hasSolution, setHasSolution] = useState(false);
  const [debugBar, setDebugBar] = useState(false);
  const [paused, setPaused] = useState<DebugPauseSnapshot | null>(null);
  const [stdin, setStdin] = useState('');
  const [watchInput, setWatchInput] = useState('');

  // ----- Imperative state (read inside async handlers via refs) -----
  const sourceElRef = useRef<HTMLDivElement>(null);
  const neighborhoodCanvasRef = useRef<HTMLCanvasElement>(null);
  const theaterCanvasRef = useRef<HTMLCanvasElement>(null);
  const swingMountRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<EditorView | null>(null);
  const neighborhoodVizRef = useRef<NeighborhoodViz | null>(null);
  const theaterVizRef = useRef<TheaterViz | null>(null);
  const swingVizRef = useRef<SwingViz | null>(null);
  const sessionRef = useRef<Promise<JvmWorkerSession>>(JvmWorkerSession.create());
  const inactiveFilesRef = useRef(new Map<string, EditorState>());
  const fileSquigglesRef = useRef(new Map<string, SourceSquiggle[]>());
  const activeFileRef = useRef('Main.java');
  const currentGridRef = useRef('');
  const validationFilesRef = useRef<CsaLevelFile[]>([]);
  const solutionFilesRef = useRef<CsaLevelFile[]>([]);
  const dataFilesRef = useRef<CsaLevelFile[]>([]);
  // Latest requested level value, so an out-of-order chunk load is ignored.
  const levelValueRef = useRef('');
  // Settles when the current level's on-demand content chunk has loaded.
  const levelLoadRef = useRef<Promise<void>>(Promise.resolve());
  const watchesRef = useRef<string[]>([]);
  // Settles the pending Swing event promise when the user activates a
  // control; the engine's event loop is parked until it resolves.
  const swingEventResolverRef = useRef<((payload: string | null) => void) | null>(null);
  // The newest drag payload seen while the loop was busy — coalesced so the
  // flood of mousemove events becomes at most one dispatch per render.
  const swingPendingMotionRef = useRef<string | null>(null);
  // Settles the pending JOptionPane dialog when the user answers it.
  const swingDialogResolverRef = useRef<((response: string | null) => void) | null>(null);
  // Whether this run rendered a live (interactive) Swing UI, so we don't
  // clobber the final frame with a stale batch render after it completes.
  const swingRenderedLiveRef = useRef(false);
  const stopRequestedRef = useRef(false);
  const wasStopped = (): boolean => stopRequestedRef.current;
  const debugResolverRef = useRef<((command: DebugCommandName | 'refresh') => void) | null>(null);

  activeFileRef.current = activeFile;

  // ----- Console -----
  const append = (text: string, kind: 'normal' | 'error' = 'normal'): void => {
    setConsoleLines((prev) => [...prev, { text, kind }]);
  };
  const clearConsole = (): void => {
    setConsoleLines([]);
  };

  // ----- File tabs (Main.java fixed; one EditorState per inactive file) -----
  const editor = (): EditorView => {
    if (editorRef.current === null) {
      throw new Error('editor not mounted');
    }
    return editorRef.current;
  };

  const allFileNames = (): string[] => {
    const names = new Set<string>([activeFileRef.current, ...inactiveFilesRef.current.keys()]);
    return ['Main.java', ...[...names].filter((n) => n !== 'Main.java').sort()];
  };
  const refreshTabs = (): void => {
    setFileNames(allFileNames());
  };

  const switchToFile = (name: string): void => {
    if (name === activeFileRef.current || !inactiveFilesRef.current.has(name)) {
      return;
    }
    inactiveFilesRef.current.set(activeFileRef.current, editor().state);
    const next = inactiveFilesRef.current.get(name);
    if (!next) {
      return;
    }
    inactiveFilesRef.current.delete(name);
    editor().setState(next);
    activeFileRef.current = name;
    setActiveFile(name);
    showDiagnostics(editor(), fileSquigglesRef.current.get(name) ?? []);
    refreshTabs();
  };

  const addFile = (name: string, text = ''): void => {
    // Keep an explicit extension (e.g. a `.txt` data file); only bare
    // names from the "+" prompt get the `.java` default.
    const fileName = name.includes('.') ? name : `${name}.java`;
    if (fileName === activeFileRef.current || inactiveFilesRef.current.has(fileName)) {
      switchToFile(fileName);
      return;
    }
    inactiveFilesRef.current.set(fileName, createFileState(text));
    refreshTabs();
    switchToFile(fileName);
  };

  const removeFile = (name: string): void => {
    if (name === 'Main.java') {
      return;
    }
    if (name === activeFileRef.current) {
      switchToFile('Main.java');
    }
    inactiveFilesRef.current.delete(name);
    fileSquigglesRef.current.delete(name);
    refreshTabs();
  };

  const loadLevelFiles = (files: CsaLevelFile[], dataFiles: CsaLevelFile[] = []): void => {
    // Switch to Main.java first: otherwise a non-Main file left active
    // from the previous level is the live editor (not in the inactive
    // map), survives the clear below, and its stale content shadows the
    // new level's same-named file (which addFile then only switches to).
    switchToFile('Main.java');
    for (const name of [...inactiveFilesRef.current.keys()]) {
      removeFile(name);
    }
    const mainFile = files.find((file) => file.path === 'Main.java');
    setSource(editor(), mainFile?.text ?? '');
    for (const file of files) {
      if (file.path !== 'Main.java') {
        addFile(file.path, file.text);
      }
    }
    // Data files (.txt read by the program) are editable tabs too; they
    // are written to the VFS before each run and skipped by the compiler.
    for (const file of dataFiles) {
      addFile(file.path, file.text);
    }
    const firstFile = files[0]?.path;
    switchToFile(mainFile || firstFile === undefined ? 'Main.java' : firstFile);
  };

  /** Every open tab as `{ path, text }` (the active tab reads live). */
  const allOpenFiles = (): { path: string; text: string }[] => {
    const files = [{ path: activeFileRef.current, text: getSource(editor()) }];
    for (const [name, state] of inactiveFilesRef.current) {
      files.push({ path: name, text: state.doc.toString() });
    }
    return files;
  };

  /** Compiler inputs: only Java tabs (data files aren't Java source). */
  const collectSources = (): { path: string; text: string }[] => {
    return allOpenFiles().filter((file) => file.path.endsWith('.java'));
  };

  /** Write the non-Java tabs (data files) into the VFS so the program can
   * read them via File/Scanner, reflecting any edits the student made. */
  const writeDataFiles = async (session: JvmWorkerSession): Promise<void> => {
    for (const file of allOpenFiles()) {
      if (!file.path.endsWith('.java')) {
        await session.writeFile(file.path, file.text);
      }
    }
  };

  const reportDiagnostics = (diagnostics: Diagnostic[]): void => {
    for (const diagnostic of diagnostics) {
      append(formatDiagnostic(diagnostic), diagnostic.severity === 'error' ? 'error' : 'normal');
    }
    fileSquigglesRef.current.clear();
    for (const diagnostic of diagnostics) {
      if (!diagnostic.start) {
        continue;
      }
      const squiggle: SourceSquiggle = {
        severity: diagnostic.severity,
        message: diagnostic.message,
        startLine: diagnostic.start.line,
        startColumn: diagnostic.start.column,
        endLine: diagnostic.end?.line,
        endColumn: diagnostic.end?.column,
      };
      const existing = fileSquigglesRef.current.get(diagnostic.path) ?? [];
      existing.push(squiggle);
      fileSquigglesRef.current.set(diagnostic.path, existing);
    }
    showDiagnostics(editor(), fileSquigglesRef.current.get(activeFileRef.current) ?? []);
  };

  // ----- Pickers -----
  const onUnitChange = (value: string): void => {
    setUnitIndex(value);
    const unit = CSA_UNITS[Number(value)];
    setLevels(unit ? unit.levels : []);
    setLevelValue('');
  };

  const onLevelChange = (value: string): void => {
    setLevelValue(value);
    levelValueRef.current = value;
    if (value === '') {
      levelLoadRef.current = Promise.resolve();
      return;
    }
    // Whether a solution exists is known synchronously (eager map), so the
    // Solve button appears immediately; its content arrives with the chunk.
    setHasSolution(levelHasSolution(Number(unitIndex), Number(value)));
    setTestResults([]);
    // The unit's content chunk loads on demand (see levels.ts).
    levelLoadRef.current = (async () => {
      const level = await loadLevel(Number(unitIndex), Number(value));
      if (!level || levelValueRef.current !== value) {
        return;
      }
      loadLevelFiles(level.files, level.dataFiles);
      validationFilesRef.current = level.validationFiles;
      solutionFilesRef.current = level.solutionFiles;
      dataFilesRef.current = level.dataFiles;
      if (level.view === 'neighborhood') {
        currentGridRef.current = level.grid;
        setView('neighborhood');
        neighborhoodVizRef.current?.load(level.grid, '');
      } else {
        setView('none');
      }
    })();
  };

  const onTheaterChange = (value: string): void => {
    setTheaterValue(value);
    const level = THEATER_LEVELS[Number(value)];
    if (!level) {
      return;
    }
    setSource(editor(), level.starter);
    validationFilesRef.current = [];
    solutionFilesRef.current = [];
    setHasSolution(false);
    setTestResults([]);
    setView('theater');
    theaterVizRef.current?.reset();
  };

  const onSwingChange = (value: string): void => {
    setSwingValue(value);
    const level = SWING_LEVELS[Number(value)];
    if (!level) {
      return;
    }
    setSource(editor(), level.starter);
    validationFilesRef.current = [];
    solutionFilesRef.current = [];
    setHasSolution(false);
    setTestResults([]);
    setView('swing');
    swingVizRef.current?.clear();
  };

  // ----- Neighborhood / theater rendering -----
  const isNeighborhoodProgram = (): boolean =>
    collectSources().some((source) => source.text.includes('org.code.neighborhood'));
  const isTheaterProgram = (): boolean =>
    collectSources().some(
      (source) =>
        source.text.includes('org.code.theater') || source.text.includes('org.code.media'),
    );
  const isSwingProgram = (): boolean =>
    collectSources().some(
      (source) => source.text.includes('javax.swing') || source.text.includes('java.awt'),
    );

  const renderNeighborhood = async (): Promise<void> => {
    const session = await sessionRef.current;
    let messagesText = '';
    try {
      messagesText = await session.readTextFile('neighborhood.jsonl');
    } catch {
      // No painter was created — nothing to animate.
    }
    if (messagesText.trim() === '') {
      setView('none');
      return;
    }
    const gridText = await session.readTextFile('grid.txt');
    setView('neighborhood');
    const viz = neighborhoodVizRef.current;
    if (viz) {
      const messages = viz.load(gridText, messagesText);
      viz.play(messages);
    }
  };

  const renderTheater = async (): Promise<void> => {
    const session = await sessionRef.current;
    let log = '';
    try {
      log = await session.readTextFile('theater.log');
    } catch {
      // No scene was played — nothing to draw.
    }
    if (log.trim() === '') {
      setView('none');
      return;
    }
    setView('theater');
    void theaterVizRef.current?.play(log);
  };

  const renderSwing = async (): Promise<void> => {
    const session = await sessionRef.current;
    let json = '';
    try {
      json = await session.readTextFile('swing.json');
    } catch {
      // No frame was shown — nothing to render.
    }
    if (json.trim() === '') {
      setView('none');
      return;
    }
    setView('swing');
    swingVizRef.current?.render(json);
  };

  // Read through a function so flow analysis doesn't assume the ref's value:
  // it is flipped by the event-loop callback during the awaited run.
  const swingRenderedLive = (): boolean => swingRenderedLiveRef.current;

  // Resolve the parked event promise when a control is activated.
  const dispatchSwingEvent = (payload: string): void => {
    // Coalesce drags: they fire far faster than the loop can render. If the
    // loop is parked, dispatch the latest now; otherwise keep only the newest
    // position so the next park picks it up (dropping the ones in between).
    if (payload.includes('\n__drag=')) {
      if (swingEventResolverRef.current) {
        const resolve = swingEventResolverRef.current;
        swingEventResolverRef.current = null;
        swingPendingMotionRef.current = null;
        resolve(payload);
      } else {
        swingPendingMotionRef.current = payload;
      }
      return;
    }
    const resolve = swingEventResolverRef.current;
    swingEventResolverRef.current = null;
    resolve?.(payload);
  };

  // JOptionPane: show a modal and park the engine (blocked in the worker)
  // until the user answers it.
  const awaitSwingDialog = (kind: string, message: string): Promise<string | null> =>
    new Promise((resolve) => {
      swingDialogResolverRef.current = resolve;
      setSwingDialogInput('');
      setSwingDialog({ kind, message });
    });

  const resolveSwingDialog = (response: string | null): void => {
    const resolve = swingDialogResolverRef.current;
    swingDialogResolverRef.current = null;
    setSwingDialog(null);
    resolve?.(response);
  };

  // The dialog's buttons and their JOptionPane response codes, by kind.
  const dialogActions = (kind: string): React.JSX.Element[] => {
    if (kind === 'message') {
      return [
        <Button
          key="ok"
          onClick={() => {
            resolveSwingDialog('');
          }}
        >
          OK
        </Button>,
      ];
    }
    if (kind === 'input') {
      return [
        <Button
          key="cancel"
          onClick={() => {
            resolveSwingDialog(null);
          }}
        >
          Cancel
        </Button>,
        <Button
          key="ok"
          onClick={() => {
            resolveSwingDialog(swingDialogInput);
          }}
        >
          OK
        </Button>,
      ];
    }
    // confirm:<optionType> — YES=0, NO=1, CANCEL=2 (OK maps to YES=0).
    const option = Number(kind.slice('confirm:'.length));
    const buttons: React.JSX.Element[] = [];
    if (option === 1 || option === 2) {
      buttons.push(
        <Button
          key="cancel"
          onClick={() => {
            resolveSwingDialog('2');
          }}
        >
          Cancel
        </Button>,
      );
    }
    if (option === 0 || option === 1) {
      buttons.push(
        <Button
          key="no"
          onClick={() => {
            resolveSwingDialog('1');
          }}
        >
          No
        </Button>,
      );
    }
    buttons.push(
      <Button
        key="yes"
        onClick={() => {
          resolveSwingDialog('0');
        }}
      >
        {option === 2 ? 'OK' : 'Yes'}
      </Button>,
    );
    return buttons;
  };

  // Interactive Swing: render the live tree (wiring controls to dispatch),
  // then park until the user activates a control OR a Timer fires. The
  // engine's event loop stays blocked in the worker until this resolves.
  const awaitSwingEvent = (tree: string): Promise<string | null> =>
    new Promise((resolve) => {
      swingRenderedLiveRef.current = true;
      const timeouts: ReturnType<typeof setTimeout>[] = [];
      const settle = (payload: string | null): void => {
        for (const timeout of timeouts) {
          clearTimeout(timeout);
        }
        swingEventResolverRef.current = null;
        resolve(payload);
      };
      // A control activation resolves through this ref (via dispatchSwingEvent).
      swingEventResolverRef.current = settle;
      setView('swing');
      swingVizRef.current?.render(tree, dispatchSwingEvent);
      // Race the running timers: whichever fires first wakes the loop; the
      // next iteration re-reads the tree and reschedules them.
      for (const timer of parseSwingTimers(tree)) {
        timeouts.push(
          setTimeout(
            () => {
              settle(`__timer:${timer.id}`);
            },
            Math.max(timer.delay, 1),
          ),
        );
      }
      // If a drag arrived while the loop was busy, dispatch its latest
      // position now instead of waiting for the next event.
      if (swingPendingMotionRef.current !== null) {
        const motion = swingPendingMotionRef.current;
        swingPendingMotionRef.current = null;
        settle(motion);
      }
    });

  // ----- Run / Test / Stop -----
  const runProgram = async (): Promise<void> => {
    stopRequestedRef.current = false;
    swingRenderedLiveRef.current = false;
    swingPendingMotionRef.current = null;
    setSwingDialog(null);
    setPhase('running');
    clearConsole();
    neighborhoodVizRef.current?.stop();
    try {
      const session = await sessionRef.current;
      await writeDataFiles(session);
      const neighborhood = isNeighborhoodProgram();
      const theater = !neighborhood && isTheaterProgram();
      const swing = !neighborhood && !theater && isSwingProgram();
      if (neighborhood) {
        await session.writeFile('grid.txt', currentGridRef.current);
        await session.remove('neighborhood.jsonl').catch(() => undefined);
      } else if (theater) {
        await session.remove('theater.log').catch(() => undefined);
        setView('none');
      } else if (swing) {
        await session.remove('swing.json').catch(() => undefined);
        setView('none');
      } else {
        setView('none');
      }
      const sources = collectSources();
      append(`$ javac ${sources.map((file) => file.path).join(' ')}\n`);
      const compiled = await session.compile(sources);
      reportDiagnostics(compiled.diagnostics);
      if (compiled.success) {
        append('$ java Main\n');
        const stdinLines = stdin === '' ? [] : stdin.split('\n');
        const result = await session.run('Main', {
          stdin: stdinLines,
          onStdout: (text) => {
            append(text);
          },
          onStderr: (text) => {
            append(text, 'error');
          },
          ...(swing ? { onSwingEvent: awaitSwingEvent, onSwingDialog: awaitSwingDialog } : {}),
        });
        if (result.status === 'error') {
          append(`${result.error ?? 'unknown VM error'}\n`, 'error');
        } else if (result.status === 'exited') {
          append(`(exit code ${String(result.exitCode)})\n`);
        }
        if (result.status !== 'error') {
          if (neighborhood) {
            await renderNeighborhood();
          } else if (theater) {
            await renderTheater();
          } else if (swing && !swingRenderedLive()) {
            // Interactive UIs already rendered live; only a non-interactive
            // window needs the one-shot batch render from swing.json.
            await renderSwing();
          }
        }
      }
    } catch (error) {
      if (!wasStopped()) {
        append(`${String(error)}\n`, 'error');
      }
    } finally {
      if (!wasStopped()) {
        setPhase('idle');
      }
    }
  };

  const stopProgram = async (): Promise<void> => {
    stopRequestedRef.current = true;
    swingEventResolverRef.current = null;
    swingPendingMotionRef.current = null;
    swingDialogResolverRef.current = null;
    setSwingDialog(null);
    // Clear any paused-debugger UI: a hard stop can happen mid-debug (an
    // interactive Swing session idling for the next event, say).
    debugResolverRef.current = null;
    setDebugBar(false);
    setPaused(null);
    setPausedLine(editor(), null);
    const session = await sessionRef.current;
    session.terminate();
    sessionRef.current = JvmWorkerSession.create();
    await sessionRef.current;
    append('\n^C program stopped\n');
    setPhase('idle');
  };

  const testProgram = async (): Promise<void> => {
    setPhase('testing');
    clearConsole();
    setTestResults([]);
    try {
      const session = await sessionRef.current;
      await writeDataFiles(session);
      const sources = [...collectSources(), ...validationFilesRef.current];
      append(`$ javac ${sources.map((source) => source.path).join(' ')}\n`);
      const compiled = await session.compile(sources);
      reportDiagnostics(compiled.diagnostics);
      if (!compiled.success) {
        return;
      }
      if (compiled.validationEntry === undefined) {
        append('(this level has no tests)\n');
        return;
      }
      let output = '';
      const result = await session.run(compiled.validationEntry, {
        onStdout: (text) => {
          output += text;
        },
        onStderr: (text) => {
          append(text, 'error');
        },
      });
      if (result.status === 'error') {
        append(`${result.error ?? 'unknown VM error'}\n`, 'error');
        return;
      }
      const results = parseTestResults(output);
      setTestResults(results);
      const passed = results.filter((r) => r.passed).length;
      append(`${String(passed)} / ${String(results.length)} tests passed\n`);
    } catch (error) {
      append(`${String(error)}\n`, 'error');
    } finally {
      setPhase('idle');
    }
  };

  /** Load the level's complete solution into the editor (dev-only overlay). */
  const solveProgram = (): void => {
    clearConsole();
    setTestResults([]);
    loadLevelFiles(solutionFilesRef.current, dataFilesRef.current);
  };

  // ----- Debugger -----
  const currentBreakpoints = (): { file: string; line: number }[] => {
    const breakpoints = breakpointLines(editor()).map((line) => ({
      file: activeFileRef.current,
      line,
    }));
    for (const [name, state] of inactiveFilesRef.current) {
      for (const line of breakpointLinesInState(state)) {
        breakpoints.push({ file: name, line });
      }
    }
    return breakpoints;
  };

  const nextDebugCommand = (): Promise<DebugCommandName | 'refresh'> =>
    new Promise((resolve) => {
      debugResolverRef.current = (command) => {
        debugResolverRef.current = null;
        resolve(command);
      };
    });

  const sendDebugCommand = (command: DebugCommandName): void => {
    debugResolverRef.current?.(command);
  };

  const addWatch = (): void => {
    const expression = watchInput.trim();
    if (expression !== '') {
      watchesRef.current = [...watchesRef.current, expression];
      setWatchInput('');
      debugResolverRef.current?.('refresh');
    }
  };

  const removeWatch = (index: number): void => {
    watchesRef.current = watchesRef.current.filter((_, i) => i !== index);
    debugResolverRef.current?.('refresh');
  };

  const debugProgram = async (): Promise<void> => {
    stopRequestedRef.current = false;
    swingRenderedLiveRef.current = false;
    swingPendingMotionRef.current = null;
    setSwingDialog(null);
    setPhase('debugging');
    clearConsole();
    try {
      const session = await sessionRef.current;
      await writeDataFiles(session);
      const swing = isSwingProgram();
      if (swing) {
        await session.remove('swing.json').catch(() => undefined);
        setView('none');
      }
      const sources = collectSources();
      append(`$ javac ${sources.map((file) => file.path).join(' ')}\n`);
      const compiled = await session.compile(sources);
      reportDiagnostics(compiled.diagnostics);
      if (compiled.success) {
        append('$ java Main (debugger attached)\n');
        const stdinLines = stdin === '' ? [] : stdin.split('\n');
        const result = await session.runDebug('Main', {
          stdin: stdinLines,
          breakpoints: currentBreakpoints(),
          watches: [...watchesRef.current],
          // An interactive Swing UI runs its event loop under the debugger,
          // so breakpoints inside listeners pause like any other code.
          ...(swing ? { onSwingEvent: awaitSwingEvent, onSwingDialog: awaitSwingDialog } : {}),
          onStdout: (text) => {
            append(text);
          },
          onStderr: (text) => {
            append(text, 'error');
          },
          onPause: async (snapshot): Promise<DebugControlResponse> => {
            setPaused(snapshot);
            const pausedFile = snapshot.frames[0]?.sourceFile;
            if (pausedFile !== undefined && pausedFile !== activeFileRef.current) {
              switchToFile(pausedFile);
            }
            setPausedLine(editor(), snapshot.frames[0]?.line ?? null);
            setDebugBar(true);
            const command = await nextDebugCommand();
            if (command === 'refresh') {
              return { command, watches: [...watchesRef.current] };
            }
            setPaused(null);
            setPausedLine(editor(), null);
            return {
              command,
              breakpoints: currentBreakpoints(),
              watches: [...watchesRef.current],
            };
          },
        });
        setDebugBar(false);
        setPaused(null);
        setPausedLine(editor(), null);
        if (result.status === 'error') {
          append(`${result.error ?? 'unknown VM error'}\n`, 'error');
        } else if (result.status === 'stopped') {
          append('(stopped by the debugger)\n');
        } else if (result.status === 'exited') {
          append(`(exit code ${String(result.exitCode)})\n`);
        }
        // A non-interactive Swing UI renders once on exit (batch), like Run.
        if (
          swing &&
          !swingRenderedLive() &&
          result.status !== 'error' &&
          result.status !== 'stopped'
        ) {
          await renderSwing();
        }
      }
    } catch (error) {
      if (!wasStopped()) {
        append(`${String(error)}\n`, 'error');
      }
    } finally {
      if (!wasStopped()) {
        setPhase('idle');
      }
    }
  };

  // ----- One-time setup: mount editor, wire hooks, fetch version -----
  useEffect(() => {
    if (sourceElRef.current === null) {
      return;
    }
    const view = createEditor(sourceElRef.current, DEFAULT_PROGRAM);
    editorRef.current = view;
    const nbViz = neighborhoodCanvasRef.current
      ? new NeighborhoodViz(neighborhoodCanvasRef.current)
      : null;
    const thViz = theaterCanvasRef.current ? new TheaterViz(theaterCanvasRef.current) : null;
    const swViz = swingMountRef.current ? new SwingViz(swingMountRef.current) : null;
    neighborhoodVizRef.current = nbViz;
    theaterVizRef.current = thViz;
    swingVizRef.current = swViz;

    window.playground = {
      setSource: (text) => {
        setSource(view, text);
      },
      getSource: () => getSource(view),
      toggleBreakpoint: (line) => {
        toggleBreakpointAtLine(view, line);
      },
      breakpointLines: () => breakpointLines(view),
      setFile: (name, text) => {
        addFile(name, text);
        setSource(view, text);
      },
      selectFile: (name) => {
        switchToFile(name);
      },
      activeFile: () => activeFileRef.current,
      levelReady: () => levelLoadRef.current,
      neighborhoodState: (): NeighborhoodState => nbViz?.state() ?? { colors: [], painters: [] },
    };

    void (async () => {
      const session = await sessionRef.current;
      setVersion(`engine v${await session.version()}`);
      setReady(true);
    })();

    return () => {
      view.destroy();
    };
  }, []);

  const runDisabled = !ready || phase === 'running' || phase === 'debugging';
  const debugDisabled = !ready || phase === 'running' || phase === 'debugging';
  const testDisabled = !ready || phase === 'running' || phase === 'testing';

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <AppBar position="static" color="default" elevation={1}>
        <Toolbar variant="dense" sx={{ gap: 2 }}>
          <Typography variant="h6" component="h1" sx={{ fontWeight: 700 }}>
            caturra playground
          </Typography>
          <Typography
            variant="body2"
            color="text.secondary"
            data-testid="engine-version"
            id="engine-version"
          >
            {version}
          </Typography>
        </Toolbar>
      </AppBar>

      <Box
        component="main"
        sx={{ display: 'flex', flex: 1, minHeight: 0, gap: 1, p: 1, alignItems: 'stretch' }}
      >
        {/* Editor pane */}
        <Paper
          className="editor-pane"
          variant="outlined"
          sx={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, p: 1, gap: 1 }}
        >
          <Stack
            direction="row"
            spacing={0.5}
            data-testid="file-tabs"
            id="file-tabs"
            sx={{ flexWrap: 'wrap', alignItems: 'center' }}
          >
            {fileNames.map((name) => (
              <Button
                key={name}
                data-file={name}
                size="small"
                variant={name === activeFile ? 'contained' : 'text'}
                onClick={() => {
                  switchToFile(name);
                }}
                sx={{ textTransform: 'none', minWidth: 0, py: 0.25 }}
                endIcon={
                  name !== 'Main.java' ? (
                    <CloseIcon
                      fontSize="inherit"
                      onClick={(event) => {
                        event.stopPropagation();
                        removeFile(name);
                      }}
                    />
                  ) : undefined
                }
              >
                {name}
              </Button>
            ))}
            <IconButton
              id="add-file"
              data-testid="add-file"
              size="small"
              title="Add a file"
              onClick={() => {
                const name = prompt('New file name (e.g. Helper.java)');
                if (name !== null && name.trim() !== '') {
                  addFile(name.trim());
                }
              }}
            >
              <AddIcon fontSize="small" />
            </IconButton>
          </Stack>

          <Box id="source" data-testid="source" ref={sourceElRef} sx={{ flex: 1, minHeight: 0 }} />
          <Typography variant="caption" color="text.secondary" className="hint">
            Click a line number to set a breakpoint.
          </Typography>

          <TextField
            label="Program input (one line per Scanner read)"
            value={stdin}
            onChange={(event) => {
              setStdin(event.target.value);
            }}
            multiline
            minRows={3}
            maxRows={3}
            size="small"
            slotProps={{ htmlInput: { spellCheck: false, id: 'stdin', 'data-testid': 'stdin' } }}
          />

          <Box className="run-row" sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
            <Button
              id="run"
              data-testid="run"
              variant="contained"
              startIcon={<PlayArrowIcon />}
              disabled={runDisabled}
              onClick={() => void runProgram()}
            >
              Compile &amp; Run
            </Button>
            <Button
              id="debug"
              data-testid="debug"
              variant="outlined"
              startIcon={<BugReportIcon />}
              disabled={debugDisabled}
              onClick={() => void debugProgram()}
            >
              Debug
            </Button>
            <Button
              id="test"
              data-testid="test"
              variant="outlined"
              startIcon={<ScienceIcon />}
              disabled={testDisabled}
              title="Run the level's tests"
              onClick={() => void testProgram()}
            >
              Test
            </Button>
            {hasSolution && (
              <Button
                id="solve"
                data-testid="solve"
                variant="outlined"
                startIcon={<LightbulbIcon />}
                disabled={!ready || phase !== 'idle'}
                title="Load the complete solution"
                onClick={solveProgram}
              >
                Solve
              </Button>
            )}
            {(phase === 'running' || phase === 'debugging') && (
              <Button
                id="stop-run"
                data-testid="stop-run"
                variant="outlined"
                color="error"
                startIcon={<StopIcon />}
                title="Stop the running program"
                onClick={() => void stopProgram()}
              >
                Stop
              </Button>
            )}
            <PickerSelect
              id="unit-select"
              label="Choose a CSA unit"
              placeholder="Unit…"
              value={unitIndex}
              onChange={onUnitChange}
            >
              {CSA_UNITS.map((unit, index) => (
                <option key={unit.name} value={index}>
                  {unit.name}
                </option>
              ))}
            </PickerSelect>
            <PickerSelect
              id="level-select"
              label="Load a level"
              placeholder="Level…"
              value={levelValue}
              onChange={onLevelChange}
            >
              {groupLevels(levels)}
            </PickerSelect>
            <PickerSelect
              id="theater-level"
              label="Load a Theater example"
              placeholder="Theater…"
              value={theaterValue}
              onChange={onTheaterChange}
            >
              {THEATER_LEVELS.map((level, index) => (
                <option key={level.name} value={index}>
                  {level.name}
                </option>
              ))}
            </PickerSelect>
            <PickerSelect
              id="swing-level"
              label="Load a Swing example"
              placeholder="Swing…"
              value={swingValue}
              onChange={onSwingChange}
            >
              {SWING_LEVELS.map((level, index) => (
                <option key={level.name} value={index}>
                  {level.name}
                </option>
              ))}
            </PickerSelect>
          </Box>
        </Paper>

        {/* Console pane */}
        <Paper
          className="console-pane"
          variant="outlined"
          sx={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, p: 1, gap: 1 }}
        >
          <Box id="debug-bar" data-testid="debug-bar" hidden={!debugBar}>
            <Button
              id="resume"
              data-testid="resume"
              size="small"
              onClick={() => {
                sendDebugCommand('continue');
              }}
            >
              Continue
            </Button>
            <Button
              id="step-over"
              data-testid="step-over"
              size="small"
              onClick={() => {
                sendDebugCommand('stepOver');
              }}
            >
              Step Over
            </Button>
            <Button
              id="step-into"
              data-testid="step-into"
              size="small"
              onClick={() => {
                sendDebugCommand('stepInto');
              }}
            >
              Step Into
            </Button>
            <Button
              id="step-out"
              data-testid="step-out"
              size="small"
              onClick={() => {
                sendDebugCommand('stepOut');
              }}
            >
              Step Out
            </Button>
            <Button
              id="stop"
              data-testid="stop"
              size="small"
              color="error"
              onClick={() => {
                sendDebugCommand('terminate');
              }}
            >
              Stop
            </Button>
          </Box>

          <Box id="paused-view" data-testid="paused-view" hidden={paused === null}>
            <Typography variant="subtitle2">Paused</Typography>
            <Box component="pre" id="frames" data-testid="frames" sx={{ m: 0 }}>
              {paused ? renderFrames(paused) : ''}
            </Box>
            <Typography variant="subtitle2">Watches</Typography>
            <Box className="watch-row" sx={{ display: 'flex', gap: 1, my: 0.5 }}>
              <TextField
                size="small"
                placeholder="e.g. total * 2"
                value={watchInput}
                onChange={(event) => {
                  setWatchInput(event.target.value);
                }}
                slotProps={{ htmlInput: { id: 'watch-input', 'data-testid': 'watch-input' } }}
              />
              <Button id="watch-add" data-testid="watch-add" size="small" onClick={addWatch}>
                Add
              </Button>
            </Box>
            <Box
              component="ul"
              id="watches"
              data-testid="watches"
              sx={{ listStyle: 'none', pl: 0, m: 0 }}
            >
              {(paused?.watchResults ?? []).map((result, index) => (
                <li key={`${result.expression}-${String(index)}`}>
                  <span className={result.error != null ? 'watch-error' : undefined}>
                    {result.error != null
                      ? `${result.expression} — ${result.error}`
                      : `${result.expression} = ${result.value ?? ''}`}
                  </span>
                  <button
                    title="Remove watch"
                    onClick={() => {
                      removeWatch(index);
                    }}
                  >
                    ×
                  </button>
                </li>
              ))}
            </Box>
          </Box>

          <Box id="viz" data-testid="viz" hidden={view !== 'neighborhood'}>
            <Typography variant="subtitle2">Neighborhood</Typography>
            <canvas
              id="neighborhood-canvas"
              data-testid="neighborhood-canvas"
              ref={neighborhoodCanvasRef}
            />
          </Box>
          <Box id="theater-viz" data-testid="theater-viz" hidden={view !== 'theater'}>
            <Typography variant="subtitle2">Theater</Typography>
            <canvas id="theater-canvas" data-testid="theater-canvas" ref={theaterCanvasRef} />
          </Box>
          <Box id="swing-viz" data-testid="swing-viz" hidden={view !== 'swing'}>
            <Typography variant="subtitle2" id="swing-heading">
              Swing UI
            </Typography>
            <div id="swing-root" data-testid="swing-root" ref={swingMountRef} />
          </Box>

          <Box
            component="ul"
            id="test-results"
            data-testid="test-results"
            hidden={testResults.length === 0}
            sx={{ listStyle: 'none', pl: 0, m: 0, fontFamily: 'monospace' }}
          >
            {testResults.map((result, index) => (
              <li
                key={`${result.name}-${String(index)}`}
                className={result.passed ? 'test-pass' : 'test-fail'}
              >
                {`${result.passed ? '✓' : '✗'} ${result.name}${
                  !result.passed && result.message ? ` — ${result.message}` : ''
                }`}
              </li>
            ))}
          </Box>

          <Typography variant="subtitle2">Console</Typography>
          <Box
            component="pre"
            id="console"
            data-testid="console"
            sx={{ flex: 1, minHeight: 0, overflow: 'auto', m: 0 }}
          >
            {consoleLines.map((line, index) => (
              <span key={index} className={line.kind === 'error' ? 'console-error' : undefined}>
                {line.text}
              </span>
            ))}
          </Box>
        </Paper>
      </Box>

      {/* JOptionPane: an accessible modal (focus-trapped, Escape closes). */}
      <Dialog
        open={swingDialog !== null}
        onClose={() => {
          resolveSwingDialog(null);
        }}
        aria-labelledby="swing-dialog-title"
        data-testid="swing-dialog"
      >
        <DialogTitle id="swing-dialog-title">
          {swingDialog?.kind === 'input'
            ? 'Input'
            : swingDialog?.kind === 'message'
              ? 'Message'
              : 'Select an Option'}
        </DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ whiteSpace: 'pre-wrap' }}>
            {swingDialog?.message}
          </DialogContentText>
          {swingDialog?.kind === 'input' && (
            <TextField
              autoFocus
              fullWidth
              variant="standard"
              value={swingDialogInput}
              onChange={(event) => {
                setSwingDialogInput(event.target.value);
              }}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  resolveSwingDialog(swingDialogInput);
                }
              }}
              slotProps={{ htmlInput: { 'data-testid': 'dialog-input', 'aria-label': 'Input' } }}
            />
          )}
        </DialogContent>
        <DialogActions>{swingDialog ? dialogActions(swingDialog.kind) : null}</DialogActions>
      </Dialog>
    </Box>
  );
}

function renderFrames(snapshot: DebugPauseSnapshot): string {
  const lines: string[] = [];
  for (const [index, frame] of snapshot.frames.entries()) {
    const at = frame.line === null ? frame.sourceFile : `${frame.sourceFile}:${String(frame.line)}`;
    lines.push(`${index === 0 ? '→' : ' '} ${frame.className}.${frame.methodName} (${at})`);
    for (const local of frame.locals) {
      lines.push(`      ${local.name} = ${local.value}`);
    }
  }
  return lines.join('\n');
}

function groupLevels(levels: CsaLevelMeta[]): React.JSX.Element[] {
  const groups: { lesson: string; items: { index: number; name: string }[] }[] = [];
  for (const [index, level] of levels.entries()) {
    let group = groups.at(-1);
    if (group?.lesson !== level.lesson) {
      group = { lesson: level.lesson, items: [] };
      groups.push(group);
    }
    group.items.push({ index, name: level.name });
  }
  return groups.map((group) => (
    <optgroup key={group.lesson} label={group.lesson}>
      {group.items.map((item) => (
        <option key={item.index} value={item.index}>
          {item.name}
        </option>
      ))}
    </optgroup>
  ));
}

interface PickerSelectProps {
  id: string;
  label: string;
  placeholder: string;
  value: string;
  onChange: (value: string) => void;
  children: React.ReactNode;
}

/** A native `<select>` (so Playwright's selectOption works) with MUI styling. */
function PickerSelect({
  id,
  label,
  placeholder,
  value,
  onChange,
  children,
}: PickerSelectProps): React.JSX.Element {
  return (
    <TextField
      select
      size="small"
      label={placeholder}
      value={value}
      onChange={(event) => {
        onChange(event.target.value);
      }}
      slotProps={{
        select: { native: true, inputProps: { id, 'data-testid': id, 'aria-label': label } },
        inputLabel: { shrink: true },
      }}
      sx={{ minWidth: 130 }}
    >
      <option value="" disabled>
        {placeholder}
      </option>
      {children}
    </TextField>
  );
}
