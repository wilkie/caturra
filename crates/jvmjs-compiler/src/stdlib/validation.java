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

class NeighborhoodTestRunner {
  public static NeighborhoodLog run() {
    __NbhdWorld.resetLog();
    Main.main(new String[] {});
    int n = __NbhdWorld.logCount;
    // Distinct painter ids, in first-seen order.
    String[] ids = new String[n];
    int idn = 0;
    for (int i = 0; i < n; i++) {
      String id = __NbhdWorld.logId[i];
      boolean seen = false;
      for (int j = 0; j < idn; j++) if (ids[j].equals(id)) { seen = true; break; }
      if (!seen) { ids[idn] = id; idn++; }
    }
    PainterLog[] logs = new PainterLog[idn];
    for (int k = 0; k < idn; k++) logs[k] = buildLog(ids[k]);
    return new NeighborhoodLog(logs, buildFinalOutput());
  }

  private static PainterLog buildLog(String id) {
    int n = __NbhdWorld.logCount;
    int c = 0;
    for (int i = 0; i < n; i++) if (__NbhdWorld.logId[i].equals(id)) c++;
    PainterEvent[] events = new PainterEvent[c];
    int sx = 0, sy = 0, ex = 0, ey = 0, sp = 0, ep = 0, k = 0;
    String sd = "north", ed = "north";
    boolean first = true;
    for (int i = 0; i < n; i++) {
      if (!__NbhdWorld.logId[i].equals(id)) continue;
      NeighborhoodActionType t = fromName(__NbhdWorld.logType[i]);
      events[k++] = new PainterEvent(t, __NbhdWorld.logColor[i], __NbhdWorld.logDir[i]);
      int x = __NbhdWorld.logX[i], y = __NbhdWorld.logY[i], p = __NbhdWorld.logPaint[i];
      String d = __NbhdWorld.logDir[i];
      if (first) { sx = x; sy = y; sd = d; sp = p; first = false; }
      ex = x; ey = y; ed = d; ep = p;
    }
    return new PainterLog(id, new Position(sx, sy, sd), new Position(ex, ey, ed), sp, ep, events);
  }

  private static String[][] buildFinalOutput() {
    int size = __NbhdWorld.getGrid().getSize();
    String[][] g = new String[size][size];
    for (int i = 0; i < __NbhdWorld.logCount; i++) {
      String t = __NbhdWorld.logType[i];
      if (t.equals("PAINT")) g[__NbhdWorld.logX[i]][__NbhdWorld.logY[i]] = __NbhdWorld.logColor[i];
      else if (t.equals("REMOVE_PAINT")) g[__NbhdWorld.logX[i]][__NbhdWorld.logY[i]] = null;
    }
    return g;
  }

  private static NeighborhoodActionType fromName(String s) {
    if (s.equals("INITIALIZE_PAINTER")) return NeighborhoodActionType.INITIALIZE_PAINTER;
    if (s.equals("MOVE")) return NeighborhoodActionType.MOVE;
    if (s.equals("PAINT")) return NeighborhoodActionType.PAINT;
    if (s.equals("REMOVE_PAINT")) return NeighborhoodActionType.REMOVE_PAINT;
    if (s.equals("TAKE_PAINT")) return NeighborhoodActionType.TAKE_PAINT;
    if (s.equals("HIDE_PAINTER")) return NeighborhoodActionType.HIDE_PAINTER;
    if (s.equals("SHOW_PAINTER")) return NeighborhoodActionType.SHOW_PAINTER;
    if (s.equals("TURN_LEFT")) return NeighborhoodActionType.TURN_LEFT;
    if (s.equals("HIDE_BUCKETS")) return NeighborhoodActionType.HIDE_BUCKETS;
    if (s.equals("SHOW_BUCKETS")) return NeighborhoodActionType.SHOW_BUCKETS;
    if (s.equals("CAN_MOVE")) return NeighborhoodActionType.CAN_MOVE;
    if (s.equals("IS_ON_BUCKET")) return NeighborhoodActionType.IS_ON_BUCKET;
    if (s.equals("IS_ON_PAINT")) return NeighborhoodActionType.IS_ON_PAINT;
    return null;
  }
}
