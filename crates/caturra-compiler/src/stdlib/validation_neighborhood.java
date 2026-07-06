// Bundled org.code.validation NeighborhoodTestRunner — injected only when
// org.code.neighborhood is also imported (it reads __NbhdWorld's action log).

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
