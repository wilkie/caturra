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
  openJvmSession,
  type DebugCommandName,
  type DebugControlResponse,
  type DebugPauseSnapshot,
  type Diagnostic,
  type JvmSessionApi,
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
import {
  assetManifest,
  assetUrl,
  decodePixels,
  decodeSamples,
  encodePixels,
  encodeSamples,
  isSound,
  referencedAssets,
} from './theater-assets.js';
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

/** The picker's optgroups, in the order they appear. Widening `group` to this
 * union means a typo in a demo's group fails the build rather than silently
 * dropping it from the list. */
const SWING_GROUPS = [
  'Getting started',
  'Layout',
  'Text',
  'Choosers & indicators',
  'Lists',
  'Tables',
  'Trees',
  'Menus, actions & dialogs',
  'Graphics & animation',
  'Styling & accessibility',
] as const;

type SwingGroup = (typeof SWING_GROUPS)[number];

interface SwingLevel {
  name: string;
  group: SwingGroup;
  starter: string;
}

// Swing examples: each starter builds a javax.swing component tree that
// renders as accessible DOM (real <button>/<input>/<label> elements with
// keyboard navigation and screen-reader names). Interactivity is Phase 2.
const SWING_LEVELS: SwingLevel[] = [
  {
    name: 'Click counter',
    group: 'Getting started',
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
    name: 'Quit button',
    group: 'Getting started',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static int clicks = 0;
  static JLabel label;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Quit demo");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new FlowLayout());

    label = new JLabel("Clicks: 0");
    JButton bump = new JButton("Bump");
    bump.addActionListener(e -> {
      Main.clicks = Main.clicks + 1;
      Main.label.setText("Clicks: " + Main.clicks);
    });

    // System.exit(0) ends the program right away — the window stops
    // responding and the frame dims to show it is no longer running.
    JButton quit = new JButton("Quit");
    quit.addActionListener(e -> System.exit(0));

    frame.add(label);
    frame.add(bump);
    frame.add(quit);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Greeter form',
    group: 'Getting started',
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
    group: 'Getting started',
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
    group: 'Graphics & animation',
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
    group: 'Graphics & animation',
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
    group: 'Graphics & animation',
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
    name: 'Keyboard mover',
    group: 'Graphics & animation',
    starter: `import javax.swing.*;
import java.awt.*;
import java.awt.event.*;

public class Main {
  static int x = 140;
  static int y = 90;
  static Board board;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Keyboard mover");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    board = new Board();
    board.setPreferredSize(new Dimension(300, 180));
    board.setToolTipText("Use the arrow keys or WASD to move the square");
    status = new JLabel("at (140, 90)");

    // A KeyListener on the focusable board: each arrow (or WASD) key press
    // nudges the square, clamps it to the board, and repaints. The board is
    // focused automatically, so the keys work as soon as the program runs.
    board.addKeyListener(new KeyAdapter() {
      public void keyPressed(KeyEvent e) {
        int code = e.getKeyCode();
        if (code == KeyEvent.VK_LEFT || code == KeyEvent.VK_A) Main.x = Main.x - 10;
        if (code == KeyEvent.VK_RIGHT || code == KeyEvent.VK_D) Main.x = Main.x + 10;
        if (code == KeyEvent.VK_UP || code == KeyEvent.VK_W) Main.y = Main.y - 10;
        if (code == KeyEvent.VK_DOWN || code == KeyEvent.VK_S) Main.y = Main.y + 10;
        if (Main.x < 15) Main.x = 15;
        if (Main.x > 285) Main.x = 285;
        if (Main.y < 15) Main.y = 15;
        if (Main.y > 165) Main.y = 165;
        Main.status.setText("at (" + Main.x + ", " + Main.y + ")");
        Main.board.repaint();
      }
    });

    frame.add(board, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}

class Board extends JPanel {
  public void paintComponent(Graphics g) {
    super.paintComponent(g);
    g.setColor(new Color(24, 28, 44));
    g.fillRect(0, 0, getWidth(), getHeight());
    g.setColor(new Color(80, 200, 255));
    g.fillRect(Main.x - 12, Main.y - 12, 24, 24);
  }
}
`,
  },
  {
    name: 'Shapes and fonts',
    group: 'Graphics & animation',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Shapes and fonts");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // setFont styles a widget's text, too — not just custom drawing.
    JLabel title = new JLabel("Shapes and Fonts");
    title.setFont(new Font("SansSerif", Font.BOLD, 20));

    ShapePanel panel = new ShapePanel();
    panel.setPreferredSize(new Dimension(320, 220));
    panel.setToolTipText("A drawing with a polygon, an arc, and sized text");

    frame.add(title, BorderLayout.NORTH);
    frame.add(panel, BorderLayout.CENTER);
    frame.setVisible(true);
  }
}

class ShapePanel extends JPanel {
  public void paintComponent(Graphics g) {
    super.paintComponent(g);
    g.setColor(new Color(18, 22, 34));
    g.fillRect(0, 0, getWidth(), getHeight());

    // A filled triangle (drawPolygon/fillPolygon take parallel x/y arrays).
    int[] xs = {60, 20, 100};
    int[] ys = {40, 110, 110};
    g.setColor(new Color(80, 200, 255));
    g.fillPolygon(xs, ys, 3);

    // A pie-slice arc: bounds (x, y, w, h) + start and sweep angles (degrees,
    // counterclockwise from 3 o'clock).
    g.setColor(new Color(255, 180, 60));
    g.fillArc(160, 30, 90, 90, 40, 260);

    // A thick pen via Graphics2D.setStroke (a 6px line, not the default 1px).
    Graphics2D g2 = (Graphics2D) g;
    g2.setStroke(new BasicStroke(6));
    g2.setColor(new Color(255, 90, 90));
    g2.drawLine(30, 205, 290, 205);

    // Sized, bold text via setFont — no longer a fixed 14px.
    g.setColor(new Color(240, 240, 240));
    g.setFont(new Font("SansSerif", Font.BOLD, 26));
    g.drawString("Score: 42", 30, 185);
  }
}
`,
  },
  {
    name: 'Bordered form',
    group: 'Styling & accessibility',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.border.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Bordered form");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // A matte border draws per-side accents (here a top and bottom rule).
    JLabel banner = new JLabel("Settings");
    banner.setBorder(BorderFactory.createMatteBorder(4, 0, 4, 0, new Color(220, 140, 40)));

    // A titled border groups related controls into a labelled group box
    // (rendered as an accessible role=group named by its title).
    JPanel group = new JPanel();
    group.setLayout(new BoxLayout(group, BoxLayout.Y_AXIS));
    group.setBorder(BorderFactory.createTitledBorder("Preferences"));
    group.add(new JCheckBox("Enable sound"));
    group.add(new JCheckBox("Dark mode"));
    group.add(new JCheckBox("Autosave"));

    // A compound border: a coloured line outside, padding inside.
    JLabel note = new JLabel("Pick your settings above.");
    note.setBorder(BorderFactory.createCompoundBorder(
        BorderFactory.createLineBorder(new Color(60, 120, 220), 2),
        BorderFactory.createEmptyBorder(6, 10, 6, 10)));

    frame.add(banner, BorderLayout.NORTH);
    frame.add(group, BorderLayout.CENTER);
    frame.add(note, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Tabbed pane',
    group: 'Layout',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JTabbedPane tabs;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Tabbed pane");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    tabs = new JTabbedPane();

    // Each tab holds its own panel of controls; only the selected one shows.
    JPanel home = new JPanel();
    home.add(new JLabel("Welcome home!"));
    tabs.addTab("Home", home);

    JPanel profile = new JPanel();
    profile.setLayout(new BoxLayout(profile, BoxLayout.Y_AXIS));
    profile.add(new JLabel("Name:"));
    profile.add(new JTextField("Ada", 12));
    tabs.addTab("Profile", profile);

    JPanel about = new JPanel();
    about.add(new JLabel("Version 1.0"));
    tabs.addTab("About", about);

    // A ChangeListener fires when the selected tab changes.
    status = new JLabel("On tab: Home");
    tabs.addChangeListener(e -> {
      int i = Main.tabs.getSelectedIndex();
      Main.status.setText("On tab: " + Main.tabs.getTitleAt(i));
    });

    frame.add(tabs, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Show / hide',
    group: 'Getting started',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel secret;
  static boolean shown = true;

  public static void main(String[] args) {
    // The idiomatic bootstrap: build the UI on the event thread. caturra runs
    // it right away (there is no separate thread).
    SwingUtilities.invokeLater(() -> { Main.buildUi(); });
  }

  static void buildUi() {
    JFrame frame = new JFrame("Show / hide");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    secret = new JLabel("Now you see me!");

    JButton toggle = new JButton("Toggle");
    toggle.addActionListener(e -> {
      Main.shown = !Main.shown;
      Main.secret.setVisible(Main.shown); // setVisible(false) hides it
    });

    // getContentPane() returns the frame's content area (the frame itself here).
    frame.getContentPane().add(toggle, BorderLayout.NORTH);
    frame.getContentPane().add(secret, BorderLayout.CENTER);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Menu options',
    group: 'Menus, actions & dialogs',
    starter: `import javax.swing.*;
import java.awt.*;
import java.awt.event.*;

public class Main {
  static JCheckBoxMenuItem wrap;
  static JRadioButtonMenuItem light;
  static JRadioButtonMenuItem dark;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Menu options");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JMenuBar bar = new JMenuBar();
    JMenu view = new JMenu("View");

    // A checkbox menu item toggles on each click.
    wrap = new JCheckBoxMenuItem("Word Wrap");
    wrap.addActionListener(e -> Main.update());
    view.add(wrap);
    view.addSeparator();

    // Radio menu items in a ButtonGroup are mutually exclusive.
    ButtonGroup themes = new ButtonGroup();
    light = new JRadioButtonMenuItem("Light", true);
    dark = new JRadioButtonMenuItem("Dark");
    themes.add(light);
    themes.add(dark);
    light.addActionListener(e -> Main.update());
    dark.addActionListener(e -> Main.update());
    view.add(light);
    view.add(dark);

    bar.add(view);
    frame.setJMenuBar(bar);

    status = new JLabel("Wrap: off, Theme: Light");
    frame.add(status, BorderLayout.CENTER);
    frame.setVisible(true);
  }

  static void update() {
    String theme = Main.light.isSelected() ? "Light" : "Dark";
    String w = Main.wrap.isSelected() ? "on" : "off";
    Main.status.setText("Wrap: " + w + ", Theme: " + theme);
  }
}
`,
  },
  {
    name: 'GridBag form',
    group: 'Layout',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("GridBag form");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new GridBagLayout());

    // One reusable GridBagConstraints, mutated between adds (each add copies
    // it, so every component keeps its own cell).
    GridBagConstraints gbc = new GridBagConstraints();
    gbc.insets = new Insets(4, 4, 4, 4);

    gbc.gridx = 0; gbc.gridy = 0; gbc.anchor = GridBagConstraints.EAST;
    JTextField name = new JTextField(12);
    // A label mnemonic: Alt+N underlines the N and focuses the name field.
    JLabel nameLabel = new JLabel("Name:");
    nameLabel.setLabelFor(name);
    nameLabel.setDisplayedMnemonic('N');
    frame.add(nameLabel, gbc);
    gbc.gridx = 1; gbc.weightx = 1.0; gbc.fill = GridBagConstraints.HORIZONTAL;
    frame.add(name, gbc);

    gbc.gridx = 0; gbc.gridy = 1; gbc.weightx = 0;
    gbc.fill = GridBagConstraints.NONE; gbc.anchor = GridBagConstraints.EAST;
    frame.add(new JLabel("Email:"), gbc);
    gbc.gridx = 1; gbc.weightx = 1.0; gbc.fill = GridBagConstraints.HORIZONTAL;
    frame.add(new JTextField(12), gbc);

    // A submit button spanning both columns.
    gbc.gridx = 0; gbc.gridy = 2; gbc.gridwidth = 2; gbc.weightx = 0;
    gbc.fill = GridBagConstraints.NONE; gbc.anchor = GridBagConstraints.CENTER;
    JButton submit = new JButton("Submit");
    submit.addActionListener(e -> Main.status.setText("Submitted: " + name.getText()));
    frame.add(submit, gbc);

    gbc.gridy = 3;
    status = new JLabel("Fill in the form");
    frame.add(status, gbc);

    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Toggle buttons',
    group: 'Choosers & indicators',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JToggleButton bold;
  static JToggleButton italic;
  static JLabel preview;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Toggle buttons");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // A JToggleButton stays pressed until clicked again (aria-pressed).
    JPanel bar = new JPanel();
    bold = new JToggleButton("Bold");
    italic = new JToggleButton("Italic");
    bold.addActionListener(e -> Main.update());
    italic.addActionListener(e -> Main.update());
    bar.add(bold);
    bar.add(italic);

    preview = new JLabel("Style: plain");

    frame.add(bar, BorderLayout.NORTH);
    frame.add(preview, BorderLayout.CENTER);
    frame.setVisible(true);
  }

  static void update() {
    String s = "";
    if (Main.bold.isSelected()) s = s + "bold ";
    if (Main.italic.isSelected()) s = s + "italic ";
    if (s.equals("")) s = "plain";
    Main.preview.setText("Style: " + s.trim());
  }
}
`,
  },
  {
    name: 'Accessible controls',
    group: 'Styling & accessibility',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Accessible controls");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new FlowLayout());

    // A slider with no visible JLabel — named for assistive tech the Swing way,
    // via getAccessibleContext() (maps to aria-label / aria-description).
    JSlider volume = new JSlider(0, 11, 5);
    volume.getAccessibleContext().setAccessibleName("Volume");
    volume.getAccessibleContext().setAccessibleDescription("Playback level from 0 to 11");
    volume.addChangeListener(e -> Main.status.setText("Volume: " + volume.getValue()));

    // A search field, likewise named without a separate label.
    JTextField search = new JTextField(12);
    search.getAccessibleContext().setAccessibleName("Search");

    // A named panel groups the controls — screen readers announce the group.
    JPanel controls = new JPanel();
    controls.getAccessibleContext().setAccessibleName("Audio settings");
    controls.add(volume);
    controls.add(search);

    status = new JLabel("Volume: 5");

    frame.add(controls);
    frame.add(status);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Login form',
    group: 'Text',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JTextField user;
  static JPasswordField pass;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Login");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new GridLayout(0, 2));

    frame.add(new JLabel("User:"));
    user = new JTextField(12);
    frame.add(user);

    frame.add(new JLabel("Password:"));
    pass = new JPasswordField(12); // masked input; getPassword() -> char[]
    frame.add(pass);

    JButton login = new JButton("Log in");
    login.setMnemonic('L'); // underlines the L; Alt+L activates
    login.addActionListener(e -> {
      String p = new String(Main.pass.getPassword());
      Main.status.setText("Welcome, " + Main.user.getText() + " (" + p.length() + " chars)");
    });
    frame.add(login);

    status = new JLabel("Enter credentials");
    frame.add(status);

    user.requestFocus(); // focus the first field, ready to type
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Combo box model',
    group: 'Choosers & indicators',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static DefaultComboBoxModel model;
  static JComboBox combo;
  static JLabel status;
  static int next = 1;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Toppings");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // The model holds the combo's items and its selection. Mutating it updates
    // the JComboBox and fires ListDataEvents.
    String[] toppings = {"Cheese", "Basil"};
    model = new DefaultComboBoxModel(toppings);

    status = new JLabel("2 toppings");
    model.addListDataListener(new ListDataListener() {
      public void intervalAdded(ListDataEvent e) {
        Main.status.setText(Main.model.getSize() + " toppings");
      }
      public void intervalRemoved(ListDataEvent e) {
        Main.status.setText(Main.model.getSize() + " toppings");
      }
      public void contentsChanged(ListDataEvent e) {}
    });

    combo = new JComboBox(model);
    combo.addActionListener(e -> Main.status.setText("Chose " + Main.combo.getSelectedItem()));

    JButton add = new JButton("Add topping");
    add.addActionListener(e -> {
      Main.model.addElement("Topping " + Main.next);
      Main.next = Main.next + 1;
    });

    JLabel prompt = new JLabel("Topping:");
    prompt.setLabelFor(combo);
    JPanel top = new JPanel();
    top.add(prompt);
    top.add(combo);
    top.add(add);

    frame.add(top, BorderLayout.NORTH);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Editable combo',
    group: 'Choosers & indicators',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JComboBox size;
  static JLabel preview;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Editable combo");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // An editable combo: pick a preset from the list OR type a custom value.
    String[] presets = {"Small", "Medium", "Large"};
    size = new JComboBox(presets);
    size.setEditable(true);
    size.addActionListener(e ->
        Main.preview.setText("Size: " + Main.size.getSelectedItem()));

    preview = new JLabel("Size: Small");

    frame.add(new JLabel("Choose or type a size:"), BorderLayout.NORTH);
    frame.add(size, BorderLayout.CENTER);
    frame.add(preview, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Search box',
    group: 'Text',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JTextField query;
  static JLabel result;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Search");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JLabel prompt = new JLabel("Type a query and press Enter:");

    // A JTextField's ActionListener fires when the user presses Enter.
    query = new JTextField(20);
    query.addActionListener(e ->
        Main.result.setText("You searched for: " + Main.query.getText()));

    result = new JLabel("No search yet");

    frame.add(prompt, BorderLayout.NORTH);
    frame.add(query, BorderLayout.CENTER);
    frame.add(result, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Live counter',
    group: 'Getting started',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JTextField field;
  static JLabel count;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Live counter");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JLabel prompt = new JLabel("Type — the count updates as you go:");

    // A DocumentListener fires on every edit (not just on Enter/blur).
    field = new JTextField(20);
    field.getDocument().addDocumentListener(new DocumentListener() {
      public void insertUpdate(DocumentEvent e) { Main.update(); }
      public void removeUpdate(DocumentEvent e) { Main.update(); }
      public void changedUpdate(DocumentEvent e) { Main.update(); }
    });

    count = new JLabel("0 characters");

    frame.add(prompt, BorderLayout.NORTH);
    frame.add(field, BorderLayout.CENTER);
    frame.add(count, BorderLayout.SOUTH);
    frame.setVisible(true);
  }

  static void update() {
    Main.count.setText(Main.field.getText().length() + " characters");
  }
}
`,
  },
  {
    name: 'Styled widgets',
    group: 'Styling & accessibility',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Styled widgets");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // setHorizontalAlignment + a preferred size give the label a fixed box
    // with right-aligned text.
    JLabel title = new JLabel("Right aligned", SwingConstants.RIGHT);
    title.setPreferredSize(new Dimension(240, 30));

    // setPreferredSize sizes the button; Color.darker() shades a base colour.
    JButton big = new JButton("Big button");
    big.setPreferredSize(new Dimension(200, 60));
    Color base = new Color(60, 120, 220);
    big.setBackground(base.darker());
    big.setForeground(Color.WHITE);
    big.addActionListener(e -> title.setText("Clicked!"));

    // A right-aligned numeric field.
    JTextField amount = new JTextField("42", 10);
    amount.setHorizontalAlignment(JTextField.RIGHT);

    frame.add(title, BorderLayout.NORTH);
    frame.add(big, BorderLayout.CENTER);
    frame.add(amount, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Absolute layout',
    group: 'Layout',
    starter: `import javax.swing.*;

public class Main {
  public static void main(String[] args) {
    JFrame f = new JFrame("Absolute layout");
    f.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    // setLayout(null) turns off the layout manager: every child is placed by
    // its own setBounds(x, y, width, height).
    f.setLayout(null);

    JLabel title = new JLabel("Placed with setBounds");
    title.setBounds(20, 20, 260, 24);
    f.add(title);

    JButton b1 = new JButton("Hello, World!");
    b1.setBounds(90, 100, 180, 40);
    b1.addActionListener(e -> title.setText("Clicked the button!"));
    f.add(b1);

    JButton b2 = new JButton("Down here");
    b2.setBounds(40, 200, 140, 30);
    f.add(b2);

    f.setSize(360, 300);
    f.setVisible(true);
  }
}
`,
  },
  {
    name: 'Shared action',
    group: 'Menus, actions & dialogs',
    starter: `import javax.swing.*;
import java.awt.*;
import java.awt.event.*;

public class Main {
  static Action save;
  static JLabel status;
  static int saves = 0;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Shared action");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // One Action drives BOTH the toolbar button and the menu item: they share
    // its name, tooltip, actionPerformed — and its enabled state.
    save = new AbstractAction("Save") {
      public void actionPerformed(ActionEvent e) {
        Main.saves = Main.saves + 1;
        Main.status.setText("Saved " + Main.saves + " time(s)");
        Main.save.setEnabled(false); // nothing more to save — disables BOTH
      }
    };
    save.putValue(Action.SHORT_DESCRIPTION, "Save the document");

    // Editing re-enables the shared Save (and both its components).
    JButton edit = new JButton("Edit");
    edit.addActionListener(e -> {
      Main.save.setEnabled(true);
      Main.status.setText("Edited — Save re-enabled");
    });

    JToolBar bar = new JToolBar("Actions");
    bar.add(save);
    bar.add(edit);

    JMenuBar menuBar = new JMenuBar();
    JMenu file = new JMenu("File");
    file.add(new JMenuItem(save));
    menuBar.add(file);

    status = new JLabel("Not saved yet");

    frame.setJMenuBar(menuBar);
    frame.add(bar, BorderLayout.NORTH);
    frame.add(status, BorderLayout.CENTER);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Toolbar',
    group: 'Menus, actions & dialogs',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Toolbar");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // A toolbar strip: arrow keys move between buttons (one Tab stop).
    JToolBar bar = new JToolBar("Formatting");
    addButton(bar, "Bold");
    addButton(bar, "Italic");
    bar.addSeparator();
    addButton(bar, "Clear");

    status = new JLabel("Ready");

    frame.add(bar, BorderLayout.NORTH);
    frame.add(status, BorderLayout.CENTER);
    frame.setVisible(true);
  }

  static void addButton(JToolBar bar, String label) {
    JButton b = new JButton(label);
    b.addActionListener(e -> Main.status.setText(label + " clicked"));
    bar.add(b);
  }
}
`,
  },
  {
    name: 'Split pane',
    group: 'Layout',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel content;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Split pane");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // Left: a folder list. Each button updates the content on the right.
    JPanel nav = new JPanel();
    nav.setLayout(new BoxLayout(nav, BoxLayout.Y_AXIS));
    String[] folders = {"Inbox", "Sent", "Drafts"};
    for (String folder : folders) {
      JButton b = new JButton(folder);
      b.addActionListener(e -> {
        Main.content.setText("Showing: " + ((JButton) e.getSource()).getText());
      });
      nav.add(b);
    }

    content = new JLabel("Showing: Inbox");

    // A draggable divider separates the two — drag it or use the arrow keys.
    JSplitPane split = new JSplitPane(JSplitPane.HORIZONTAL_SPLIT, nav, content);
    split.setDividerLocation(120);

    frame.add(split, BorderLayout.CENTER);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Sketch pad',
    group: 'Graphics & animation',
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
    group: 'Menus, actions & dialogs',
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
    group: 'Layout',
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
    group: 'Menus, actions & dialogs',
    starter: `import javax.swing.*;
import java.awt.*;
import java.awt.event.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Notes");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);

    JMenuBar bar = new JMenuBar();

    JMenu file = new JMenu("File");
    JMenuItem newItem = new JMenuItem("New");
    // A keyboard accelerator: Ctrl+N works even without opening the menu.
    newItem.setAccelerator(KeyStroke.getKeyStroke(KeyEvent.VK_N, InputEvent.CTRL_DOWN_MASK));
    newItem.addActionListener(e -> Main.status.setText("New file"));
    JMenuItem openItem = new JMenuItem("Open");
    openItem.setAccelerator(KeyStroke.getKeyStroke(KeyEvent.VK_O, InputEvent.CTRL_DOWN_MASK));
    openItem.addActionListener(e -> Main.status.setText("Opened a file"));

    // A nested submenu: File > Export > (PNG | JPEG).
    JMenu export = new JMenu("Export");
    JMenuItem png = new JMenuItem("PNG");
    png.addActionListener(e -> Main.status.setText("Exported PNG"));
    JMenuItem jpeg = new JMenuItem("JPEG");
    jpeg.addActionListener(e -> Main.status.setText("Exported JPEG"));
    export.add(png);
    export.add(jpeg);

    // File > Quit ends the program with System.exit(0); the frame then dims.
    JMenuItem quitItem = new JMenuItem("Quit");
    quitItem.addActionListener(e -> System.exit(0));
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
    group: 'Text',
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
    name: 'Caret and selection',
    group: 'Text',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JTextField field;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Caret");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new FlowLayout());

    field = new JTextField("hello world", 18);
    JLabel prompt = new JLabel("Text:");
    prompt.setLabelFor(field);

    status = new JLabel("Select some text, then Inspect.");

    // The host keeps the caret in sync with the real cursor, so these read
    // whatever the user has actually selected.
    JButton inspect = new JButton("Inspect");
    inspect.addActionListener(e -> {
      String selected = Main.field.getSelectedText();
      if (selected == null) {
        Main.status.setText("Caret at " + Main.field.getCaretPosition() + ", nothing selected");
      } else {
        Main.status.setText("Selected \\"" + selected + "\\" at ["
            + Main.field.getSelectionStart() + "," + Main.field.getSelectionEnd() + "]");
      }
    });

    // selectAll() moves the real cursor in the browser.
    JButton all = new JButton("Select all");
    all.addActionListener(e -> Main.field.selectAll());

    frame.add(prompt);
    frame.add(field);
    frame.add(inspect);
    frame.add(all);
    frame.add(status);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Text area ops',
    group: 'Text',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JTextArea area;
  static JLabel info;
  static int added = 0;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Text ops");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    area = new JTextArea("Line 1", 6, 24);

    // append and insert edit the text; getLineCount() reports how many lines.
    JButton addLine = new JButton("Add line");
    addLine.addActionListener(e -> {
      Main.added = Main.added + 1;
      Main.area.append("\\nLine " + (Main.added + 1));
      Main.updateInfo();
    });

    JButton insertTop = new JButton("Insert at top");
    insertTop.addActionListener(e -> {
      Main.area.insert("== TOP ==\\n", 0);
      Main.updateInfo();
    });

    info = new JLabel("Lines: 1");

    JPanel buttons = new JPanel();
    buttons.add(addLine);
    buttons.add(insertTop);

    frame.add(buttons, BorderLayout.NORTH);
    frame.add(new JScrollPane(area), BorderLayout.CENTER);
    frame.add(info, BorderLayout.SOUTH);
    frame.setVisible(true);
  }

  static void updateInfo() {
    info.setText("Lines: " + area.getLineCount());
  }
}
`,
  },
  {
    name: 'Scroll pane (JScrollPane)',
    group: 'Layout',
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
    group: 'Lists',
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
    group: 'Lists',
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
    group: 'Lists',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static DefaultListModel model;
  static JList list;
  static JTextField field;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("To-do");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // The model holds the list's contents; mutating it updates the JList.
    model = new DefaultListModel();
    model.addElement("Buy milk");
    model.addElement("Walk the dog");

    // A ListDataListener fires whenever the model changes, so a summary label
    // stays in sync without the buttons updating it directly.
    model.addListDataListener(new ListDataListener() {
      public void intervalAdded(ListDataEvent e) { Main.summarize("added"); }
      public void intervalRemoved(ListDataEvent e) { Main.summarize("removed"); }
      public void contentsChanged(ListDataEvent e) { Main.summarize("changed"); }
    });

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

    // SOUTH: remove the selected item, plus the listener-driven summary.
    JButton remove = new JButton("Remove selected");
    remove.addActionListener(e -> {
      int i = Main.list.getSelectedIndex();
      if (i >= 0) {
        Main.model.remove(i);
      }
    });
    status = new JLabel("2 items");
    JPanel bottom = new JPanel();
    bottom.add(remove);
    bottom.add(status);

    frame.add(top, BorderLayout.NORTH);
    frame.add(list, BorderLayout.CENTER);
    frame.add(bottom, BorderLayout.SOUTH);
    frame.setVisible(true);
  }

  static void summarize(String what) {
    status.setText(model.getSize() + " items (last: " + what + ")");
  }
}
`,
  },
  {
    name: 'List cell renderer',
    group: 'Lists',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JList list;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Tasks");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    String[] tasks = {"buy milk", "!call the bank", "walk the dog", "!pay rent"};
    list = new JList(tasks);
    list.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
    list.setVisibleRowCount(6);

    // A cell renderer decides how each row is drawn. It is asked once per row,
    // with the row's value and index. Rows starting with "!" are urgent: they
    // get shouted and painted red.
    list.setCellRenderer(new DefaultListCellRenderer() {
      public Component getListCellRendererComponent(JList list, Object value,
          int index, boolean isSelected, boolean cellHasFocus) {
        Component base = super.getListCellRendererComponent(
            list, value, index, isSelected, cellHasFocus);
        JLabel label = (JLabel) base;
        String text = "" + value;
        if (text.startsWith("!")) {
          label.setText((index + 1) + ". " + text.substring(1).toUpperCase());
          label.setForeground(new Color(190, 30, 30));
        } else {
          label.setText((index + 1) + ". " + text);
        }
        return label;
      }
    });

    status = new JLabel("Pick a task.");
    list.addListSelectionListener(e ->
        Main.status.setText("Picked " + Main.list.getSelectedValue()));

    frame.add(new JScrollPane(list), BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Custom list model',
    group: 'Lists',
    starter: `import javax.swing.*;
import java.awt.*;
import java.util.ArrayList;
import javax.swing.event.*;

// A custom AbstractListModel: implement getSize/getElementAt over your own
// data, and call fireIntervalAdded to notify listeners of a new element.
class SquaresModel extends AbstractListModel {
  ArrayList<Integer> values = new ArrayList<Integer>();
  public int getSize() { return values.size(); }
  public Object getElementAt(int index) { return values.get(index); }
  public void addSquare(int n) {
    values.add(n * n);
    fireIntervalAdded(this, values.size() - 1, values.size() - 1);
  }
}

public class Main {
  static SquaresModel model;
  static JList list;
  static JLabel status;
  static int next = 1;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Squares");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    model = new SquaresModel();
    model.addSquare(1);
    model.addSquare(2);
    next = 3;

    // The JList accepts any ListModel. A listener tracks the size.
    status = new JLabel("2 squares");
    model.addListDataListener(new ListDataListener() {
      public void intervalAdded(ListDataEvent e) {
        Main.status.setText(Main.model.getSize() + " squares");
      }
      public void intervalRemoved(ListDataEvent e) {}
      public void contentsChanged(ListDataEvent e) {}
    });

    list = new JList(model);
    list.setSelectionMode(ListSelectionModel.SINGLE_SELECTION);
    list.setVisibleRowCount(6);

    JButton add = new JButton("Add square");
    add.addActionListener(e -> {
      Main.model.addSquare(Main.next);
      Main.next = Main.next + 1;
    });

    frame.add(add, BorderLayout.NORTH);
    frame.add(new JScrollPane(list), BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Table (JTable)',
    group: 'Tables',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JTable table;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Scoreboard");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    String[] columns = {"Player", "Score"};
    Object[][] data = {
        {"Ada", "10"},
        {"Bo", "7"},
        {"Cy", "13"}
    };
    // JTable cells are editable by default; this is a display table, so make it
    // read-only by overriding isCellEditable.
    table = new JTable(data, columns) {
      public boolean isCellEditable(int row, int col) { return false; }
    };

    // Row selection reports through the selection model's listener.
    table.getSelectionModel().addListSelectionListener(e -> {
      int row = Main.table.getSelectedRow();
      if (row >= 0) {
        Main.status.setText(
            Main.table.getValueAt(row, 0) + " has " + Main.table.getValueAt(row, 1) + " points");
      }
    });

    status = new JLabel("Select a player.");

    // Award a point to the selected row (getValueAt / setValueAt).
    JButton award = new JButton("Award point");
    award.addActionListener(e -> {
      int row = Main.table.getSelectedRow();
      if (row >= 0) {
        int score = Integer.parseInt("" + Main.table.getValueAt(row, 1));
        Main.table.setValueAt("" + (score + 1), row, 1);
        Main.status.setText(Main.table.getValueAt(row, 0) + " now has " + (score + 1) + " points");
      }
    });

    frame.add(status, BorderLayout.NORTH);
    frame.add(table, BorderLayout.CENTER);
    frame.add(award, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Editable table (DefaultTableModel)',
    group: 'Tables',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static DefaultTableModel model;
  static JTable table;
  static JTextField nameField;
  static JTextField qtyField;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Inventory");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // The model holds the rows; addRow/removeRow update the table live.
    String[] cols = {"Item", "Qty"};
    Object[][] data = {{"Apples", "3"}, {"Bread", "1"}};
    model = new DefaultTableModel(data, cols);
    table = new JTable(model);

    status = new JLabel("Add or select an item.");
    table.getSelectionModel().addListSelectionListener(e -> {
      int row = Main.table.getSelectedRow();
      if (row >= 0 && row < Main.model.getRowCount()) {
        Main.status.setText("Selected: " + Main.model.getValueAt(row, 0));
      }
    });

    // A TableModelListener fires on every model change — adding or removing a
    // row, or editing a cell inline — reporting what happened.
    model.addTableModelListener(new TableModelListener() {
      public void tableChanged(TableModelEvent e) {
        String kind = "changed";
        if (e.getType() == TableModelEvent.INSERT) kind = "row added";
        else if (e.getType() == TableModelEvent.DELETE) kind = "row removed";
        else if (e.getType() == TableModelEvent.UPDATE) kind = "cell edited";
        Main.status.setText(kind + " (" + Main.model.getRowCount() + " rows)");
      }
    });

    // NORTH: a small form to append a row.
    nameField = new JTextField(8);
    qtyField = new JTextField(3);
    JLabel itemLabel = new JLabel("Item:");
    itemLabel.setLabelFor(nameField);
    JLabel qtyLabel = new JLabel("Qty:");
    qtyLabel.setLabelFor(qtyField);
    JButton add = new JButton("Add");
    add.addActionListener(e -> {
      String name = Main.nameField.getText();
      if (!name.equals("")) {
        Object[] row = {name, Main.qtyField.getText()};
        Main.model.addRow(row);
        Main.nameField.setText("");
        Main.qtyField.setText("");
      }
    });
    JPanel top = new JPanel();
    top.add(itemLabel);
    top.add(nameField);
    top.add(qtyLabel);
    top.add(qtyField);
    top.add(add);

    // SOUTH: remove the selected row, plus a status label.
    JButton remove = new JButton("Remove selected");
    remove.addActionListener(e -> {
      int row = Main.table.getSelectedRow();
      if (row >= 0 && row < Main.model.getRowCount()) {
        Main.model.removeRow(row);
      }
    });
    JPanel bottom = new JPanel();
    bottom.add(remove);
    bottom.add(status);

    frame.add(top, BorderLayout.NORTH);
    frame.add(table, BorderLayout.CENTER);
    frame.add(bottom, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Tree (JTree)',
    group: 'Trees',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JTree tree;
  static DefaultTreeModel model;
  static JLabel status;
  static int added = 0;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Colors");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // Build the node tree; the root starts expanded, deeper nodes collapsed.
    DefaultMutableTreeNode root = new DefaultMutableTreeNode("Colors");
    DefaultMutableTreeNode warm = new DefaultMutableTreeNode("Warm");
    warm.add(new DefaultMutableTreeNode("Red"));
    warm.add(new DefaultMutableTreeNode("Orange"));
    DefaultMutableTreeNode cool = new DefaultMutableTreeNode("Cool");
    cool.add(new DefaultMutableTreeNode("Blue"));
    cool.add(new DefaultMutableTreeNode("Green"));
    root.add(warm);
    root.add(cool);

    // The model owns the nodes; mutate it so listeners hear about the change.
    model = new DefaultTreeModel(root);
    tree = new JTree(model);

    model.addTreeModelListener(new TreeModelListener() {
      public void treeNodesInserted(TreeModelEvent e) {
        Main.status.setText("Added " + e.getChildren()[0] + " under " + e.getTreePath());
      }
      public void treeNodesRemoved(TreeModelEvent e) {}
      public void treeNodesChanged(TreeModelEvent e) {}
      public void treeStructureChanged(TreeModelEvent e) {}
    });

    // A cell renderer decides how each node is drawn. It is asked once per
    // visible node, with the node, whether it is a leaf, and its row.
    tree.setCellRenderer(new DefaultTreeCellRenderer() {
      public Component getTreeCellRendererComponent(JTree tree, Object value,
          boolean selected, boolean expanded, boolean leaf, int row, boolean hasFocus) {
        Component base = super.getTreeCellRendererComponent(
            tree, value, selected, expanded, leaf, row, hasFocus);
        JLabel label = (JLabel) base;
        // Leaves are green; the colour resets between nodes, so it does not
        // bleed onto the folders drawn after them.
        if (leaf) {
          label.setForeground(new Color(20, 120, 40));
        } else {
          label.setText(value + " (" + ((DefaultMutableTreeNode) value).getChildCount() + ")");
        }
        return label;
      }
    });

    status = new JLabel("Click a node. Arrow keys navigate and expand.");
    tree.addTreeSelectionListener(e -> {
      DefaultMutableTreeNode node =
          (DefaultMutableTreeNode) Main.tree.getLastSelectedPathComponent();
      if (node != null) {
        Main.status.setText(node.getUserObject()
            + (node.isLeaf() ? " (leaf)" : " (" + node.getChildCount() + " children)")
            + " at " + Main.tree.getSelectionPath());
      }
    });

    // Insert into the model: the tree redraws and the listener fires.
    JButton add = new JButton("Add shade");
    add.addActionListener(e -> {
      DefaultMutableTreeNode node =
          (DefaultMutableTreeNode) Main.tree.getLastSelectedPathComponent();
      if (node != null && !node.isLeaf()) {
        Main.added = Main.added + 1;
        DefaultMutableTreeNode shade = new DefaultMutableTreeNode("Shade " + Main.added);
        Main.model.insertNodeInto(shade, node, node.getChildCount());
        Main.tree.expandPath(new TreePath(node.getPath()));
      }
    });

    frame.add(add, BorderLayout.NORTH);
    frame.add(new JScrollPane(tree), BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Sortable table',
    group: 'Tables',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JTable table;
  static DefaultTableModel model;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Scores");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    String[] cols = {"Player", "Score"};
    Object[][] data = {{"Ada", "9"}, {"Bo", "10"}, {"Cy", "2"}};
    model = new DefaultTableModel(data, cols);
    table = new JTable(model);

    // Click a header to sort: ascending, then descending, then unsorted.
    // Numbers sort numerically, so 10 comes after 9.
    table.setAutoCreateRowSorter(true);

    // Preferred widths come from the column model.
    table.getColumnModel().getColumn(0).setPreferredWidth(160);
    table.getColumnModel().getColumn(1).setPreferredWidth(60);

    status = new JLabel("Click a header to sort; click a row to select.");
    table.getSelectionModel().addListSelectionListener(e -> {
      int viewRow = Main.table.getSelectedRow();
      if (viewRow >= 0) {
        // Row indices are VIEW indices — convert before asking the model.
        int modelRow = Main.table.convertRowIndexToModel(viewRow);
        Main.status.setText("Row " + viewRow + " is model row " + modelRow
            + ": " + Main.model.getValueAt(modelRow, 0));
      }
    });

    frame.add(new JScrollPane(table), BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Table cell renderer',
    group: 'Tables',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  public static void main(String[] args) {
    JFrame frame = new JFrame("Scores");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    String[] cols = {"Player", "Score"};
    Object[][] data = {{"Ada", "10"}, {"Bo", "-3"}, {"Cy", "7"}};
    DefaultTableModel model = new DefaultTableModel(data, cols);
    JTable table = new JTable(model);

    // A cell renderer decides how each cell in its column is drawn. It is asked
    // once per cell, with the value, the row, and the column.
    DefaultTableCellRenderer scores = new DefaultTableCellRenderer() {
      public Component getTableCellRendererComponent(JTable table, Object value,
          boolean isSelected, boolean hasFocus, int row, int column) {
        Component base = super.getTableCellRendererComponent(
            table, value, isSelected, hasFocus, row, column);
        JLabel label = (JLabel) base;
        // Negative scores go red. The colour resets between cells, so this
        // does not bleed onto the next row.
        if (("" + value).startsWith("-")) {
          label.setForeground(new Color(190, 30, 30));
        }
        return label;
      }
    };
    // Alignment is set once and sticks for every cell in the column.
    scores.setHorizontalAlignment(SwingConstants.RIGHT);

    table.getColumnModel().getColumn(1).setCellRenderer(scores);
    table.getColumnModel().getColumn(1).setHeaderValue("Points");

    frame.add(new JScrollPane(table), BorderLayout.CENTER);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Custom table model',
    group: 'Tables',
    starter: `import javax.swing.*;
import java.awt.*;
import java.util.ArrayList;
import javax.swing.event.*;

// A custom AbstractTableModel: implement getRowCount/getColumnCount/getValueAt
// over your own data, and call fireTableRowsInserted to refresh listeners.
class PeopleModel extends AbstractTableModel {
  ArrayList<String> names = new ArrayList<String>();
  ArrayList<Integer> ages = new ArrayList<Integer>();
  String[] cols = {"Name", "Age"};
  public int getRowCount() { return names.size(); }
  public int getColumnCount() { return 2; }
  public String getColumnName(int c) { return cols[c]; }
  public Object getValueAt(int r, int c) {
    if (c == 0) return names.get(r);
    return ages.get(r);
  }
  public void addPerson(String name, int age) {
    names.add(name);
    ages.add(age);
    fireTableRowsInserted(names.size() - 1, names.size() - 1);
  }
}

public class Main {
  static PeopleModel model;
  static JTextField nameField;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("People");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    model = new PeopleModel();
    model.addPerson("Ada", 36);
    model.addPerson("Bo", 29);

    // The JTable accepts any TableModel. A listener tracks the row count.
    status = new JLabel("2 people");
    model.addTableModelListener(new TableModelListener() {
      public void tableChanged(TableModelEvent e) {
        Main.status.setText(Main.model.getRowCount() + " people");
      }
    });

    JTable table = new JTable(model);

    nameField = new JTextField(10);
    JLabel nameLabel = new JLabel("Name:");
    nameLabel.setLabelFor(nameField);
    JButton add = new JButton("Add person");
    add.addActionListener(e -> {
      String name = Main.nameField.getText();
      if (!name.equals("")) {
        Main.model.addPerson(name, name.length());
        Main.nameField.setText("");
      }
    });

    JPanel top = new JPanel();
    top.add(nameLabel);
    top.add(nameField);
    top.add(add);

    frame.add(top, BorderLayout.NORTH);
    frame.add(new JScrollPane(table), BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Progress + spinner',
    group: 'Choosers & indicators',
    starter: `import javax.swing.*;
import java.awt.*;
import javax.swing.event.*;

public class Main {
  static JSpinner spinner;
  static JProgressBar bar;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Level");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    JLabel prompt = new JLabel("Level (0-100):");
    spinner = new JSpinner(new SpinnerNumberModel(40, 0, 100, 5));
    prompt.setLabelFor(spinner);

    bar = new JProgressBar(0, 100);
    bar.setValue(40);
    bar.setStringPainted(true);

    status = new JLabel("Level is 40.");

    // Changing the spinner drives the progress bar.
    spinner.addChangeListener(e -> {
      int value = (Integer) Main.spinner.getValue();
      Main.bar.setValue(value);
      Main.status.setText("Level is " + value + ".");
    });

    JPanel top = new JPanel();
    top.add(prompt);
    top.add(spinner);

    frame.add(top, BorderLayout.NORTH);
    frame.add(bar, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Edit cells (JTable)',
    group: 'Tables',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JTable table;
  static DefaultTableModel model;
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Editable");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
    frame.setLayout(new BorderLayout());

    // JTable cells are editable by default — double-click to edit, Enter to
    // commit. The edit updates the model, so getValueAt sees the new value.
    String[] cols = {"Item", "Qty"};
    Object[][] data = {{"Apples", "3"}, {"Bread", "1"}, {"Milk", "2"}};
    model = new DefaultTableModel(data, cols);
    table = new JTable(model);

    status = new JLabel("Double-click a Qty cell to edit it.");

    JButton total = new JButton("Total qty");
    total.addActionListener(e -> {
      int sum = 0;
      for (int r = 0; r < Main.model.getRowCount(); r++) {
        sum += Integer.parseInt("" + Main.model.getValueAt(r, 1));
      }
      Main.status.setText("Total quantity: " + sum);
    });

    frame.add(total, BorderLayout.NORTH);
    frame.add(table, BorderLayout.CENTER);
    frame.add(status, BorderLayout.SOUTH);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Box layout',
    group: 'Layout',
    starter: `import javax.swing.*;
import java.awt.*;

public class Main {
  static JLabel status;

  public static void main(String[] args) {
    JFrame frame = new JFrame("Menu");
    frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);

    // A vertical stack: buttons separated by struts, glue pushes the status
    // label to the bottom.
    JPanel panel = new JPanel();
    panel.setLayout(new BoxLayout(panel, BoxLayout.Y_AXIS));

    JButton one = new JButton("One");
    one.addActionListener(e -> Main.status.setText("Picked One"));
    JButton two = new JButton("Two");
    two.addActionListener(e -> Main.status.setText("Picked Two"));
    JButton three = new JButton("Three");
    three.addActionListener(e -> Main.status.setText("Picked Three"));

    status = new JLabel("Pick a button.");

    panel.add(one);
    panel.add(Box.createVerticalStrut(8));
    panel.add(two);
    panel.add(Box.createVerticalStrut(8));
    panel.add(three);
    panel.add(Box.createVerticalGlue());
    panel.add(status);

    frame.add(panel);
    frame.setVisible(true);
  }
}
`,
  },
  {
    name: 'Sign-up form',
    group: 'Getting started',
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
    group: 'Getting started',
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

/**
 * Where to run the engine. A configured `VITE_SANDBOX_ORIGIN` wins (the
 * deployed choice); otherwise a `?sandbox=<origin>` query parameter is
 * honored only for loopback origins, so local dev and e2e can drive the
 * cross-origin iframe path without a dedicated build while a deployed
 * bundle can't be pointed at an arbitrary host. Unset means a
 * same-origin worker.
 */
function resolveSandboxOrigin(): string | undefined {
  const configured = import.meta.env.VITE_SANDBOX_ORIGIN;
  if (configured) {
    return configured;
  }
  const param = new URLSearchParams(window.location.search).get('sandbox');
  if (param && /^https?:\/\/(localhost|127\.0\.0\.1)(:\d+)?$/.test(param)) {
    return param;
  }
  return undefined;
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
  // The Swing look-and-feel (skins every widget via the --swing-* token layer).
  const [laf, setLaf] = useState('system');
  const [levels, setLevels] = useState<CsaLevelMeta[]>([]);
  const [view, setView] = useState<'none' | 'neighborhood' | 'theater' | 'swing'>('none');
  const [swingDialog, setSwingDialog] = useState<{ kind: string; message: string } | null>(null);
  const [swingDialogInput, setSwingDialogInput] = useState('');
  // The last dialog shown, kept during MUI's fade-out so the closing dialog
  // doesn't flash its default text as swingDialog goes null.
  const lastDialogRef = useRef<{ kind: string; message: string } | null>(null);
  // True once an interactive Swing app has finished — the shown frame is inert.
  const [swingStopped, setSwingStopped] = useState(false);
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
  // Whether the selected level/example is a theater one, so its stage should
  // stay up across a run even when the program draws nothing (a starter whose
  // main() is still a TO DO body writes no theater.log).
  const theaterStageRef = useRef(false);
  const swingVizRef = useRef<SwingViz | null>(null);
  // Lazily opened once, then reused. Do NOT create the session in the
  // useRef initializer: that expression runs on every render, and in
  // sandbox mode each call appends an iframe and loads the engine — so
  // re-renders would pile up dozens of orphaned engines.
  const sessionRef = useRef<Promise<JvmSessionApi> | null>(null);
  const getSession = (): Promise<JvmSessionApi> => {
    const existing = sessionRef.current;
    if (existing) {
      return existing;
    }
    const created = openJvmSession({ sandboxOrigin: resolveSandboxOrigin() });
    sessionRef.current = created;
    return created;
  };
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
  const writeDataFiles = async (session: JvmSessionApi): Promise<void> => {
    for (const file of allOpenFiles()) {
      if (!file.path.endsWith('.java')) {
        await session.writeFile(file.path, file.text);
      }
    }
  };

  /**
   * Load the starter assets this program names — only those, since there are
   * hundreds — so `playSound(name)` and `drawImage(name, …)` use the real thing.
   * When the program also reads samples, preload them into the VFS so
   * `SoundLoader.read` returns real audio instead of silence. Names with no
   * asset (nothing bundled, nothing fetched) are left alone: they stay silent
   * or draw the placeholder box.
   */
  const preloadAssets = async (session: JvmSessionApi): Promise<void> => {
    const sources = collectSources();
    const named = referencedAssets(sources.map((file) => file.text).join('\n'));
    const viz = theaterVizRef.current;
    if (named.length === 0 || !viz) {
      return;
    }
    const available = await assetManifest();
    const sounds = new Map<string, string>();
    const images = new Map<string, string>();
    for (const name of named) {
      const path = available[name];
      if (path !== undefined) {
        (isSound(name) ? sounds : images).set(name, assetUrl(path));
      }
    }
    await viz.loadImages(images);
    // Ship the pixels into the VFS so `new Image(name)` sees the real thing.
    if (sources.some((file) => file.text.includes('Image'))) {
      for (const name of images.keys()) {
        const pixels = viz.pixelsOf(name);
        if (pixels) {
          await session.writeFile(`__caturra_image_${name}`, encodePixels(pixels));
        }
      }
    }
    if (sounds.size === 0) {
      return;
    }
    const samples = await viz.loadAssets(sounds);
    if (!sources.some((file) => file.text.includes('SoundLoader'))) {
      return; // decoded for playback; no need to ship samples into the VFS
    }
    for (const [name, floats] of samples) {
      await session.writeFile(`__caturra_sound_${name}`, encodeSamples(floats));
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
      theaterStageRef.current = level.view === 'theater';
      if (level.view === 'neighborhood') {
        currentGridRef.current = level.grid;
        setView('neighborhood');
        neighborhoodVizRef.current?.load(level.grid, '');
      } else if (level.view === 'theater') {
        // Show the stage straight away; Run fills it (dispatch is by imports).
        setView('theater');
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
    theaterStageRef.current = true;
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
    const session = await getSession();
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
    const session = await getSession();
    let log = '';
    try {
      log = await session.readTextFile('theater.log');
    } catch {
      // No scene was played — nothing to draw.
    }
    if (log.trim() === '') {
      // The program played no scenes — the usual case for a level whose starter
      // main() is still a TO DO body. Leave a theater level's (cleared) stage up
      // instead of collapsing the view out from under the student.
      setView(theaterStageRef.current ? 'theater' : 'none');
      return;
    }
    // Read the buffers behind each playSound(double[]) and drawImage(Image, …)
    // so the viz can play / draw what the program actually produced.
    const pcm = new Map<number, Float32Array>();
    const engineImages = new Map<number, ImageData>();
    for (const line of log.split('\n')) {
      const sound = /^sound pcm (\d+) /.exec(line);
      if (sound) {
        const id = Number(sound[1]);
        try {
          pcm.set(id, decodeSamples(await session.readTextFile(`__caturra_pcm_${String(id)}`)));
        } catch {
          // Missing sample file — skip this clip.
        }
        continue;
      }
      const image = /^image obj (\d+) /.exec(line);
      if (image) {
        const id = Number(image[1]);
        try {
          const pixels = decodePixels(await session.readFile(`__caturra_img_${String(id)}`));
          if (pixels) {
            engineImages.set(id, pixels);
          }
        } catch {
          // Missing pixel file — falls back to the placeholder box.
        }
      }
    }
    await theaterVizRef.current?.setEngineImages(engineImages);
    setView('theater');
    void theaterVizRef.current?.play(log, pcm);
  };

  const renderSwing = async (): Promise<void> => {
    const session = await getSession();
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
    setSwingStopped(false);
    setPhase('running');
    clearConsole();
    neighborhoodVizRef.current?.stop();
    try {
      const session = await getSession();
      await writeDataFiles(session);
      const neighborhood = isNeighborhoodProgram();
      const theater = !neighborhood && isTheaterProgram();
      const swing = !neighborhood && !theater && isSwingProgram();
      if (neighborhood) {
        await session.writeFile('grid.txt', currentGridRef.current);
        await session.remove('neighborhood.jsonl').catch(() => undefined);
      } else if (theater) {
        await session.remove('theater.log').catch(() => undefined);
        await preloadAssets(session);
        // Clear the stage for the new run, but keep a theater level's stage on
        // screen rather than making it vanish the moment Run is pressed.
        if (theaterStageRef.current) {
          theaterVizRef.current?.reset();
          setView('theater');
        } else {
          setView('none');
        }
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
      // An interactive Swing app whose loop returned (window closed) has exited;
      // dim the last frame so it's clear nothing is listening anymore. (Read via
      // the function so flow analysis doesn't assume the ref's value.)
      if (swingRenderedLive()) {
        setSwingStopped(true);
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
    const session = await getSession();
    session.terminate();
    sessionRef.current = null;
    await getSession();
    append('\n^C program stopped\n');
    setPhase('idle');
  };

  const testProgram = async (): Promise<void> => {
    setPhase('testing');
    clearConsole();
    setTestResults([]);
    try {
      const session = await getSession();
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
    neighborhoodVizRef.current?.stop();
    try {
      const session = await getSession();
      await writeDataFiles(session);
      // The same per-view setup and post-run render as runProgram: a debugged
      // theater or neighborhood program should end on its drawing, not on a
      // blank stage.
      const neighborhood = isNeighborhoodProgram();
      const theater = !neighborhood && isTheaterProgram();
      const swing = !neighborhood && !theater && isSwingProgram();
      if (neighborhood) {
        await session.writeFile('grid.txt', currentGridRef.current);
        await session.remove('neighborhood.jsonl').catch(() => undefined);
      } else if (theater) {
        await session.remove('theater.log').catch(() => undefined);
        await preloadAssets(session);
        // Clear the stage for the new run, but keep a theater level's stage on
        // screen rather than making it vanish the moment Debug is pressed.
        if (theaterStageRef.current) {
          theaterVizRef.current?.reset();
          setView('theater');
        } else {
          setView('none');
        }
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
        // Render on exit like Run does — unless the debugger terminated the
        // program mid-way, in which case there is no finished drawing to show.
        if (result.status !== 'error' && result.status !== 'stopped') {
          if (neighborhood) {
            await renderNeighborhood();
          } else if (theater) {
            await renderTheater();
          } else if (swing && !swingRenderedLive()) {
            // A non-interactive Swing UI renders once on exit (batch), like Run.
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
      const session = await getSession();
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

  // The dialog's `open` follows swingDialog, but its CONTENT reads the last
  // shown dialog so it doesn't flash default text during MUI's fade-out (when
  // swingDialog has already gone null).
  if (swingDialog !== null) {
    lastDialogRef.current = swingDialog;
  }
  const shownDialog = swingDialog ?? lastDialogRef.current;

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
              {groupSwingLevels(SWING_LEVELS)}
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
            <Box
              sx={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                gap: 1,
              }}
            >
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Typography variant="subtitle2" id="swing-heading">
                  Swing UI
                </Typography>
                {swingStopped && (
                  <Typography
                    data-testid="swing-stopped-badge"
                    variant="caption"
                    sx={{
                      px: 0.8,
                      py: 0.2,
                      borderRadius: 1,
                      bgcolor: 'action.disabledBackground',
                      color: 'text.secondary',
                    }}
                  >
                    Exited — not running
                  </Typography>
                )}
              </Box>
              <TextField
                select
                size="small"
                label="Look and Feel"
                value={laf}
                onChange={(event) => {
                  setLaf(event.target.value);
                }}
                slotProps={{
                  select: {
                    native: true,
                    inputProps: {
                      id: 'swing-laf',
                      'data-testid': 'swing-laf',
                      'aria-label': 'Look and Feel',
                    },
                  },
                  inputLabel: { shrink: true },
                }}
                sx={{ minWidth: 150 }}
              >
                <option value="system">System (theme)</option>
                <option value="metal">Metal</option>
                <option value="nimbus">Nimbus</option>
                <option value="contrast">High Contrast</option>
              </TextField>
            </Box>
            <div
              id="swing-root"
              data-testid="swing-root"
              data-laf={laf}
              className={swingStopped ? 'swing-stopped' : undefined}
              ref={swingMountRef}
            />
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
          {shownDialog?.kind === 'input'
            ? 'Input'
            : shownDialog?.kind === 'message'
              ? 'Message'
              : 'Select an Option'}
        </DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ whiteSpace: 'pre-wrap' }}>
            {shownDialog?.message}
          </DialogContentText>
          {shownDialog?.kind === 'input' && (
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
        <DialogActions>{shownDialog ? dialogActions(shownDialog.kind) : null}</DialogActions>
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

/** Group the Swing demos into optgroups. The option's value stays its index in
 * SWING_LEVELS, so grouping is purely presentational and the array need not be
 * ordered by group. */
function groupSwingLevels(levels: SwingLevel[]): React.JSX.Element[] {
  const indexed = levels.map((level, index) => ({ level, index }));
  return SWING_GROUPS.flatMap((group) => {
    const items = indexed.filter((entry) => entry.level.group === group);
    if (items.length === 0) {
      return [];
    }
    return [
      <optgroup key={group} label={group}>
        {items.map(({ level, index }) => (
          <option key={level.name} value={index}>
            {level.name}
          </option>
        ))}
      </optgroup>,
    ];
  });
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
