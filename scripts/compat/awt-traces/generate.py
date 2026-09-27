#!/usr/bin/env python3
"""Turn the measured JDK 11 traces in this directory into the interpreter's
table of Swing event-dispatch frames (crates/caturra-vm/src/interpreter/awt_frames.rs).

A listener that a window event runs is, on a JDK, many frames deep: the
component's own firing chain (`AbstractButton.fireActionPerformed` and the
model beneath it), the UI delegate that turned the mouse or key into that
(`BasicButtonListener.mouseReleased`), AWT's routing of the event to the
component (`LightweightDispatcher`, the keyboard focus manager), and the event
queue. None of it can be measured without a display, so the traces here were
captured by `capture.sh`: a GUI build of the same JDK (Ubuntu's
openjdk-11-jre, 11.0.32, beside the installed headless one) under Xvfb, with
`java.awt.Robot` making the clicks and keystrokes a user would.

Each scenario below names the message its listener threw, and where the chain
SPLITS: the HEAD (from the listener's caller down to just above the split) is
what the bundled component's `__fire…` method stands for, and the TAIL (the
split frame down to the queue's `dispatchEvent`) what its event entry point
(`__onEvent`, or `doClick`) stands for. That way a listener fired by a click
and the same listener fired by `doClick()` each show a JDK's frames. A
scenario with no split maps the whole chain to one method. The pump beneath
(`EventDispatchThread.pumpEvents…run`) is the dispatch thread's own mapping.

Run it after re-capturing: `python3 scripts/compat/awt-traces/generate.py`.
"""

import pathlib
import re

HERE = pathlib.Path(__file__).parent
OUT = HERE.parents[2] / "crates/caturra-vm/src/interpreter/awt_frames.rs"

# (capture, message, split frame or None, head key, tail key or None)
SCENARIOS = [
    ("rig1", "button action", "BasicButtonListener.mouseReleased", ("JButton", "__fireAction"), ("JButton", "__onEvent")),
    ("rig3", "button doClick", "AbstractButton.doClick(", ("JButton", "__fireAction"), ("JButton", "doClick")),
    ("rig1", "toggle item", "BasicButtonListener.mouseReleased", ("JToggleButton", "__fireItem"), ("JToggleButton", "__onEvent")),
    ("rig2", "toggle action", "BasicButtonListener.mouseReleased", ("JToggleButton", "__fireAction"), ("JToggleButton", "__onEvent")),
    ("rig3", "toggle doClick", "AbstractButton.doClick(", ("JToggleButton", "__fireAction"), ("JToggleButton", "doClick")),
    ("rig3", "toggle item doClick", "AbstractButton.doClick(", ("JToggleButton", "__fireItem"), ("JToggleButton", "doClick")),
    ("rig1", "check item", "BasicButtonListener.mouseReleased", ("JCheckBox", "__fireItem"), ("JCheckBox", "__onEvent")),
    ("rig2", "check action", "BasicButtonListener.mouseReleased", ("JCheckBox", "__fireAction"), ("JCheckBox", "__onEvent")),
    ("rig1", "radio item", "BasicButtonListener.mouseReleased", ("JRadioButton", "__fireItem"), ("JRadioButton", "__onEvent")),
    ("rig2", "radio action", "BasicButtonListener.mouseReleased", ("JRadioButton", "__fireAction"), ("JRadioButton", "__onEvent")),
    ("rig1", "field action", "SwingUtilities.notifyAction", ("JTextField", "__fireAction"), ("JTextField", "__onEvent")),
    ("rig1", "doc insert", None, ("JTextComponent", "__fireInsert"), None),
    ("rig2", "doc remove", None, ("JTextComponent", "__fireRemove"), None),
    ("rig1", "mouse pressed", None, ("JPanel", "__firePressed"), None),
    ("rig1", "mouse released", None, ("JPanel", "__fireReleased"), None),
    ("rig1", "mouse clicked", None, ("JPanel", "__fireClicked"), None),
    ("rig1", "mouse dragged", None, ("JPanel", "__fireDragged"), None),
    ("rig1", "key pressed", None, ("JPanel", "__fireKeyPressed"), None),
    ("rig1", "key typed", None, ("JPanel", "__fireKeyTyped"), None),
    ("rig1", "key released", None, ("JPanel", "__fireKeyReleased"), None),
    ("rig2", "combo action", "BasicComboPopup$Handler.mouseReleased", ("JComboBox", "__fireAction"), ("JComboBox", "__onEvent")),
    ("rig2", "list selection", "BasicListUI$Handler.adjustSelection", ("JList", "__fireSelection"), ("JList", "__onEvent")),
    ("rig2", "slider change", "BasicSliderUI$TrackListener", ("JSlider", "__fireChange"), ("JSlider", "__onEvent")),
    ("rig2", "spinner change", "BasicSpinnerUI$ArrowButtonHandler", ("JSpinner", "__fireChange"), ("JSpinner", "__onEvent")),
    ("rig2", "tabs change", "BasicTabbedPaneUI$Handler", ("JTabbedPane", "__fireChange"), ("JTabbedPane", "__onEvent")),
    ("rig2", "tree selection", "BasicTreeUI.selectPathForEvent", ("JTree", "__fireSelection"), ("JTree", "__onEvent")),
    ("rig3", "table selection", "BasicTableUI$Handler", ("JTable", "__fireSelection"), ("JTable", "__onEvent")),
    ("rig2", "menu item action", "BasicMenuItemUI.doClick", ("JMenuItem", "__fireAction"), ("JMenuItem", "__onEvent")),
    ("rig2", "check menu action", "BasicMenuItemUI.doClick", ("JCheckBoxMenuItem", "__fireAction"), ("JCheckBoxMenuItem", "__onEvent")),
    ("rig3", "check menu item", "BasicMenuItemUI.doClick", ("JCheckBoxMenuItem", "__fireItem"), ("JCheckBoxMenuItem", "__onEvent")),
    ("rig2", "radio menu action", "BasicMenuItemUI.doClick", ("JRadioButtonMenuItem", "__fireAction"), ("JRadioButtonMenuItem", "__onEvent")),
]

LIBRARY = re.compile(r"^\tat (java\.desktop|java\.base)/")
QUEUE_END = "java.awt.EventQueue.dispatchEvent(EventQueue.java:742)"


def traces(capture):
    """Every trace in a capture, keyed by the message its exception carried
    (the FIRST trace for a message: a JList click reports twice, press first)."""
    found = {}
    current = None
    for line in (HERE / f"{capture}.txt").read_text().splitlines():
        head = re.match(r"^(?:Exception in thread \"[^\"]+\" )?java\.lang\.IllegalStateException: (.*)$", line)
        if head:
            current = head.group(1)
            if current in found:
                current = None
            else:
                found[current] = []
            continue
        if current is not None and line.startswith("\tat "):
            found[current].append(line)
    return found


def main():
    captures = {}
    table = {}
    for capture, message, split, head_key, tail_key in SCENARIOS:
        if capture not in captures:
            captures[capture] = traces(capture)
        lines = captures[capture][message]
        start = next(i for i, line in enumerate(lines) if LIBRARY.match(line))
        library = [line[len("\tat "):] for line in lines[start:]]
        # A window event's chain ends at the queue's dispatch; a `doClick`
        # made by the program ends where the program's own frames resume.
        end = next((i for i, line in enumerate(library) if line.endswith(QUEUE_END)), None)
        if end is None:
            end = next(i for i, line in enumerate(library) if not LIBRARY.match("\tat " + line)) - 1
        library = library[: end + 1]
        if split is None:
            parts = [(head_key, library)]
        else:
            at = next(i for i, line in enumerate(library) if split in line)
            tail = library[at:]
            parts = [(head_key, library[:at]), (tail_key, tail)]
        for key, frames in parts:
            if key in table and table[key] != frames:
                raise SystemExit(f"{key} measured two ways:\n{table[key]}\n{frames}")
            table[key] = frames
    out = [
        "//! GENERATED by scripts/compat/awt-traces/generate.py from the JDK 11 traces",
        "//! beside it — do not edit by hand. What a JDK's event-dispatch thread shows",
        "//! beneath a listener that a window event (or `doClick`) ran, keyed by the",
        "//! bundled Swing method that stands for those frames.",
        "",
        "pub(super) fn swing_frame_lines(class: &str, method: &str) -> Option<&'static [&'static str]> {",
        "    Some(match (class, method) {",
    ]
    for (owner, method), frames in sorted(table.items()):
        out.append(f'        ("{owner}", "{method}") => &[')
        for frame in frames:
            out.append(f'            "{frame}",')
        out.append("        ],")
    out += ["        _ => return None,", "    })", "}", ""]
    OUT.write_text("\n".join(out))
    print(f"{len(table)} mappings from {len(SCENARIOS)} scenarios -> {OUT.relative_to(HERE.parents[2])}")


if __name__ == "__main__":
    main()
