import javax.swing.*;
import javax.swing.event.*;
import javax.swing.tree.*;
import java.awt.*;
import java.awt.event.*;
public class Rig2 {
    static JComponent[] c = new JComponent[20];
    static JMenu menu; static JMenuItem mi; static JCheckBoxMenuItem cmi; static JRadioButtonMenuItem rmi;
    static void fail(String what) { throw new IllegalStateException(what); }
    public static void main(String[] args) throws Exception {
        SwingUtilities.invokeAndWait(() -> {
            JFrame f = new JFrame("R");
            JMenuBar bar = new JMenuBar();
            menu = new JMenu("M");
            mi = new JMenuItem("I"); mi.addActionListener(e -> fail("menu item action"));
            cmi = new JCheckBoxMenuItem("C"); cmi.addActionListener(e -> fail("check menu action"));
            rmi = new JRadioButtonMenuItem("R"); rmi.addActionListener(e -> fail("radio menu action"));
            menu.add(mi); menu.add(cmi); menu.add(rmi); bar.add(menu); f.setJMenuBar(bar);
            f.setLayout(new GridLayout(5, 2));
            JToggleButton t = new JToggleButton("T"); t.addActionListener(e -> fail("toggle action")); c[0] = t;
            JCheckBox x = new JCheckBox("X"); x.addActionListener(e -> fail("check action")); c[1] = x;
            JRadioButton r = new JRadioButton("R"); r.addActionListener(e -> fail("radio action")); c[2] = r;
            JTextField tf = new JTextField("abc", 10);
            tf.getDocument().addDocumentListener(new DocumentListener() {
                public void insertUpdate(DocumentEvent e) {}
                public void removeUpdate(DocumentEvent e) { fail("doc remove"); }
                public void changedUpdate(DocumentEvent e) {}
            });
            c[3] = tf;
            JComboBox<String> combo = new JComboBox<>(new String[] {"one", "two", "three"});
            combo.addActionListener(e -> fail("combo action")); c[4] = combo;
            JList<String> list = new JList<>(new String[] {"a", "b", "c"});
            list.addListSelectionListener(e -> fail("list selection")); c[5] = list;
            JSlider slider = new JSlider(0, 100, 50); slider.addChangeListener(e -> fail("slider change")); c[6] = slider;
            JSpinner spinner = new JSpinner(new SpinnerNumberModel(5, 0, 10, 1));
            spinner.addChangeListener(e -> fail("spinner change")); c[7] = spinner;
            JTabbedPane tabs = new JTabbedPane();
            tabs.addTab("one", new JLabel("1")); tabs.addTab("two", new JLabel("2"));
            tabs.addChangeListener(e -> fail("tabs change")); c[8] = tabs;
            DefaultMutableTreeNode root = new DefaultMutableTreeNode("root");
            root.add(new DefaultMutableTreeNode("leaf"));
            JTree tree = new JTree(root);
            tree.addTreeSelectionListener(e -> fail("tree selection")); c[9] = tree;
            f.add(t); f.add(x); f.add(r); f.add(tf); f.add(combo); f.add(list); f.add(slider); f.add(spinner); f.add(tabs); f.add(tree);
            f.setSize(600, 600); f.setLocation(20, 20); f.setVisible(true);
        });
        Robot robot = new Robot();
        robot.setAutoDelay(40);
        robot.waitForIdle(); Thread.sleep(400);
        for (int i = 0; i < 3; i++) { mark("click " + i); click(robot, c[i], 8, 8); }
        mark("doc remove"); click(robot, c[3], 60, 8); key(robot, KeyEvent.VK_END); key(robot, KeyEvent.VK_BACK_SPACE);
        mark("combo"); click(robot, c[4], 10, 8);
        JComboBox<?> combo = (JComboBox<?>) c[4];
        Thread.sleep(300);
        Point cp = c[4].getLocationOnScreen();
        clickAt(robot, cp.x + 20, cp.y + c[4].getHeight() + 30);
        mark("list"); JList<?> list = (JList<?>) c[5]; Rectangle cell = list.getCellBounds(1, 1);
        click(robot, c[5], cell.x + 5, cell.y + cell.height / 2);
        mark("slider"); Point sp = c[6].getLocationOnScreen(); int w = c[6].getWidth(), h = c[6].getHeight();
        robot.mouseMove(sp.x + w / 2, sp.y + h / 2); robot.mousePress(InputEvent.BUTTON1_DOWN_MASK);
        robot.mouseMove(sp.x + w / 2 + 30, sp.y + h / 2); robot.mouseRelease(InputEvent.BUTTON1_DOWN_MASK);
        robot.waitForIdle(); Thread.sleep(200);
        mark("spinner"); JButton up = null;
        for (Component k : ((JSpinner) c[7]).getComponents()) if (k instanceof JButton && up == null) up = (JButton) k;
        click(robot, up, 3, 3);
        mark("tabs"); Rectangle tab = ((JTabbedPane) c[8]).getBoundsAt(1);
        click(robot, c[8], tab.x + 5, tab.y + 5);
        mark("tree"); Rectangle row = ((JTree) c[9]).getRowBounds(0);
        click(robot, c[9], row.x + 5, row.y + row.height / 2);
        mark("menu item"); click(robot, menu, 5, 5); Thread.sleep(300); click(robot, mi, 5, 5);
        mark("check menu"); click(robot, menu, 5, 5); Thread.sleep(300); click(robot, cmi, 5, 5);
        mark("radio menu"); click(robot, menu, 5, 5); Thread.sleep(300); click(robot, rmi, 5, 5);
        System.exit(0);
    }
    static void mark(String s) throws Exception { Thread.sleep(200); System.err.println("=== " + s); }
    static void click(Robot robot, Component comp, int dx, int dy) throws Exception {
        Point p = comp.getLocationOnScreen(); clickAt(robot, p.x + dx, p.y + dy);
    }
    static void clickAt(Robot robot, int x, int y) throws Exception {
        robot.mouseMove(x, y);
        robot.mousePress(InputEvent.BUTTON1_DOWN_MASK);
        robot.mouseRelease(InputEvent.BUTTON1_DOWN_MASK);
        robot.waitForIdle(); Thread.sleep(200);
    }
    static void key(Robot robot, int code) throws Exception {
        robot.keyPress(code); robot.keyRelease(code); robot.waitForIdle(); Thread.sleep(200);
    }
}
