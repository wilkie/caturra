import javax.swing.*;
import javax.swing.table.*;
import java.awt.*;
import java.awt.event.*;
public class Rig3 {
    static JTable table; static JMenu menu; static JCheckBoxMenuItem cmi;
    static void fail(String what) { throw new IllegalStateException(what); }
    public static void main(String[] args) throws Exception {
        SwingUtilities.invokeAndWait(() -> {
            JButton b = new JButton("B"); b.addActionListener(e -> fail("button doClick"));
            try { b.doClick(); } catch (IllegalStateException e) { e.printStackTrace(); }
            JToggleButton t = new JToggleButton("T"); t.addActionListener(e -> fail("toggle doClick"));
            try { t.doClick(); } catch (IllegalStateException e) { e.printStackTrace(); }
            JToggleButton ti = new JToggleButton("T"); ti.addItemListener(e -> fail("toggle item doClick"));
            try { ti.doClick(); } catch (IllegalStateException e) { e.printStackTrace(); }
            JFrame f = new JFrame("R");
            JMenuBar bar = new JMenuBar(); menu = new JMenu("M");
            cmi = new JCheckBoxMenuItem("C"); cmi.addItemListener(e -> fail("check menu item"));
            menu.add(cmi); bar.add(menu); f.setJMenuBar(bar);
            table = new JTable(new DefaultTableModel(new Object[][] {{"a", 1}, {"b", 2}}, new Object[] {"x", "y"}));
            table.getSelectionModel().addListSelectionListener(e -> fail("table selection"));
            f.add(new JScrollPane(table));
            f.setSize(400, 300); f.setLocation(20, 20); f.setVisible(true);
        });
        Robot robot = new Robot(); robot.setAutoDelay(40); robot.waitForIdle(); Thread.sleep(400);
        System.err.println("=== table");
        Rectangle cell = table.getCellRect(1, 0, true);
        Point p = table.getLocationOnScreen();
        clickAt(robot, p.x + cell.x + 5, p.y + cell.y + cell.height / 2);
        System.err.println("=== check menu item");
        Point m = menu.getLocationOnScreen(); clickAt(robot, m.x + 5, m.y + 5); Thread.sleep(300);
        Point c = cmi.getLocationOnScreen(); clickAt(robot, c.x + 5, c.y + 5);
        Thread.sleep(300);
        System.exit(0);
    }
    static void clickAt(Robot robot, int x, int y) throws Exception {
        robot.mouseMove(x, y); robot.mousePress(InputEvent.BUTTON1_DOWN_MASK);
        robot.mouseRelease(InputEvent.BUTTON1_DOWN_MASK); robot.waitForIdle(); Thread.sleep(200);
    }
}
