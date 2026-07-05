// Bundled implementation of org.code.validation (Code.org CSA), injected when
// a source imports org.code.validation. Ported from javabuilder's user-facing
// classes; the tracker infrastructure is adapted to jvmjs — instead of a
// listener over ClientMessages, NeighborhoodTestRunner.run() invokes the
// student's main and reads the in-memory action log recorded by __NbhdWorld
// (see stdlib/neighborhood.java). Parallel-array log avoids java.util.Map.

enum NeighborhoodActionType {
  INITIALIZE_PAINTER, MOVE, PAINT, REMOVE_PAINT, TAKE_PAINT, HIDE_PAINTER,
  SHOW_PAINTER, TURN_LEFT, HIDE_BUCKETS, SHOW_BUCKETS, CAN_MOVE, IS_ON_BUCKET,
  IS_ON_PAINT
}

class Position {
  private int x, y;
  private String direction;
  public Position(int x, int y, String direction) { this.x = x; this.y = y; this.direction = direction; }
  public int getX() { return x; }
  public int getY() { return y; }
  public String getDirection() { return direction; }
}

class PainterEvent {
  private NeighborhoodActionType type;
  private String color, direction;
  public PainterEvent(NeighborhoodActionType type, String color, String direction) {
    this.type = type; this.color = color; this.direction = direction;
  }
  public NeighborhoodActionType getEventType() { return type; }
  public String getColor() { return color; }
  public String getDirection() { return direction; }
}

class PainterLog {
  private String painterId;
  private PainterEvent[] events;
  private int[] eventCounts;
  private Position startingPosition, endingPosition;
  private int startingPaintCount, endingPaintCount;

  public PainterLog(String painterId, Position startingPosition, Position endingPosition,
      int startingPaintCount, int endingPaintCount, PainterEvent[] events) {
    this.painterId = painterId;
    this.events = events;
    this.startingPosition = startingPosition;
    this.endingPosition = endingPosition;
    this.startingPaintCount = startingPaintCount;
    this.endingPaintCount = endingPaintCount;
    this.eventCounts = new int[13];
    for (int i = 0; i < events.length; i++) {
      if (events[i].getEventType() != null) eventCounts[events[i].getEventType().ordinal()]++;
    }
  }

  public boolean didActionOnce(NeighborhoodActionType t) { return didActionExactly(t, 1); }
  public boolean didActionExactly(NeighborhoodActionType t, int times) { return eventCounts[t.ordinal()] == times; }
  public boolean didActionAtLeast(NeighborhoodActionType t, int times) { return eventCounts[t.ordinal()] >= times; }
  public int actionCount(NeighborhoodActionType t) { return eventCounts[t.ordinal()]; }
  public Position getStartingPosition() { return startingPosition; }
  public Position getEndingPosition() { return endingPosition; }
  public int getStartingPaintCount() { return startingPaintCount; }
  public int getEndingPaintCount() { return endingPaintCount; }
  public String getPainterId() { return painterId; }
  public PainterEvent[] getEvents() { return events; }
}

class NeighborhoodLog {
  private PainterLog[] painterLogs;
  private String[][] finalOutput;
  public NeighborhoodLog(PainterLog[] painterLogs, String[][] finalOutput) {
    this.painterLogs = painterLogs;
    this.finalOutput = finalOutput;
  }
  public PainterLog[] getPainterLogs() { return painterLogs; }
  public String[][] getFinalOutput() { return finalOutput; }

  public boolean onePainterDidAction(NeighborhoodActionType t, int times) {
    for (int i = 0; i < painterLogs.length; i++) if (painterLogs[i].didActionExactly(t, times)) return true;
    return false;
  }
  public boolean actionHappened(NeighborhoodActionType t, int times) {
    int c = 0;
    for (int i = 0; i < painterLogs.length; i++) c += painterLogs[i].actionCount(t);
    return c == times;
  }
  public boolean finalOutputMatches(String[][] expected) {
    if (expected.length != finalOutput.length) return false;
    for (int i = 0; i < finalOutput.length; i++) {
      if (finalOutput[i].length != expected[i].length) return false;
      for (int j = 0; j < finalOutput[i].length; j++) {
        String e = expected[i][j], a = finalOutput[i][j];
        if (e == a) continue;
        if ((a == null && e != null) || !a.equals(e)) return false;
      }
    }
    return true;
  }
  public boolean finalOutputContainsPaint(boolean[][] expected) {
    if (expected.length != finalOutput.length) return false;
    for (int i = 0; i < finalOutput.length; i++) {
      if (finalOutput[i].length != expected[i].length) return false;
      for (int j = 0; j < finalOutput[i].length; j++) {
        boolean hasPaint = finalOutput[i][j] != null;
        if (expected[i][j] != hasPaint) return false;
      }
    }
    return true;
  }
}

class ValidationHelper {
  // The student's class names, baked in at compile time (see the generated
  // __ValidationClasses companion). Validators use this to check that the
  // student defined a class of a given name.
  public static java.util.ArrayList<String> getClassNames() {
    return __ValidationClasses.names();
  }
}

class SystemOutTestRunner {
  // Run the student's main and return one entry per System.out print/println
  // call, in order (blank messages dropped, matching javabuilder's tracker).
  public static java.util.ArrayList<String> run() {
    System.__captureStart();
    Main.main(new String[] {});
    String[] messages = System.__captureEnd();
    java.util.ArrayList<String> result = new java.util.ArrayList<String>();
    for (String message : messages) {
      if (message.length() > 0) {
        result.add(message);
      }
    }
    return result;
  }
}

