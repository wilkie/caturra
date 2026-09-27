import javax.swing.*;
import javax.swing.event.*;
import java.awt.*;
import java.awt.event.*;
public class Rig1 {
    static JComponent[] c = new JComponent[16];
    static void fail(String what) { throw new IllegalStateException(what); }
    public static void main(String[] args) throws Exception {
        SwingUtilities.invokeAndWait(() -> {
            JFrame f = new JFrame("R");
            f.setLayout(new GridLayout(4, 2));
            JButton b = new JButton("B"); b.addActionListener(e -> fail("button action")); c[0] = b;
            JToggleButton t = new JToggleButton("T");
            t.addItemListener(e -> fail("toggle item")); t.addActionListener(e -> fail("toggle action")); c[1] = t;
            JCheckBox x = new JCheckBox("X");
            x.addItemListener(e -> fail("check item")); x.addActionListener(e -> fail("check action")); c[2] = x;
            JRadioButton r = new JRadioButton("R");
            r.addItemListener(e -> fail("radio item")); r.addActionListener(e -> fail("radio action")); c[3] = r;
            JTextField tf = new JTextField(10);
            tf.addActionListener(e -> fail("field action"));
            tf.getDocument().addDocumentListener(new DocumentListener() {
                public void insertUpdate(DocumentEvent e) { fail("doc insert"); }
                public void removeUpdate(DocumentEvent e) { fail("doc remove"); }
                public void changedUpdate(DocumentEvent e) {}
            });
            c[4] = tf;
            JPanel p = new JPanel();
            p.setFocusable(true);
            p.addMouseListener(new MouseAdapter() {
                public void mousePressed(MouseEvent e) { fail("mouse pressed"); }
                public void mouseReleased(MouseEvent e) { fail("mouse released"); }
                public void mouseClicked(MouseEvent e) { fail("mouse clicked"); }
            });
            p.addMouseMotionListener(new MouseMotionAdapter() {
                public void mouseDragged(MouseEvent e) { fail("mouse dragged"); }
            });
            p.addKeyListener(new KeyAdapter() {
                public void keyPressed(KeyEvent e) { fail("key pressed"); }
                public void keyTyped(KeyEvent e) { fail("key typed"); }
                public void keyReleased(KeyEvent e) { fail("key released"); }
            });
            c[5] = p;
            f.add(b); f.add(t); f.add(x); f.add(r); f.add(tf); f.add(p);
            f.setSize(400, 400); f.setLocation(20, 20); f.setVisible(true);
        });
        Robot robot = new Robot();
        robot.setAutoDelay(40);
        robot.waitForIdle(); Thread.sleep(400);
        for (int i = 0; i < 4; i++) { mark("click " + i); click(robot, c[i]); }
        mark("field"); click(robot, c[4]);
        mark("field typed a"); key(robot, KeyEvent.VK_A);
        mark("field backspace"); key(robot, KeyEvent.VK_BACK_SPACE);
        mark("field enter"); key(robot, KeyEvent.VK_ENTER);
        mark("panel click"); click(robot, c[5]);
        mark("panel drag");
        Point q = c[5].getLocationOnScreen();
        robot.mouseMove(q.x + 10, q.y + 10);
        robot.mousePress(InputEvent.BUTTON1_DOWN_MASK);
        robot.mouseMove(q.x + 30, q.y + 30);
        robot.mouseRelease(InputEvent.BUTTON1_DOWN_MASK);
        robot.waitForIdle(); Thread.sleep(200);
        SwingUtilities.invokeAndWait(() -> c[5].requestFocusInWindow());
        robot.waitForIdle(); Thread.sleep(200);
        mark("panel key b"); key(robot, KeyEvent.VK_B);
        System.exit(0);
    }
    static void mark(String s) throws Exception {
        Thread.sleep(200);
        System.err.println("=== " + s);
    }
    static void click(Robot robot, JComponent comp) throws Exception {
        Point p = comp.getLocationOnScreen();
        robot.mouseMove(p.x + 8, p.y + 8);
        robot.mousePress(InputEvent.BUTTON1_DOWN_MASK);
        robot.mouseRelease(InputEvent.BUTTON1_DOWN_MASK);
        robot.waitForIdle(); Thread.sleep(200);
    }
    static void key(Robot robot, int code) throws Exception {
        robot.keyPress(code); robot.keyRelease(code);
        robot.waitForIdle(); Thread.sleep(200);
    }
}
