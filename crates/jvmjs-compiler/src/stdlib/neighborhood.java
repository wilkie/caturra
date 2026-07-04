// Bundled clean-room implementation of org.code.neighborhood (Code.org CSA).
// Auto-injected by the compiler when a source imports org.code.neighborhood.
// Support types are prefixed `__Nbhd` to avoid colliding with student classes;
// only `Painter` is public API. Phase 1 is a headless simulation: state and
// console output are faithful; the visual signal-message stream is Phase 2.

enum __NbhdDir {
  NORTH("north"), EAST("east"), SOUTH("south"), WEST("west");
  private String s;
  __NbhdDir(String s) { this.s = s; }
  __NbhdDir turnLeft() { if (this == NORTH) return WEST; return values()[ordinal() - 1]; }
  String getDirectionString() { return s; }
  boolean isNorth() { return this == NORTH; }
  boolean isSouth() { return this == SOUTH; }
  boolean isEast() { return this == EAST; }
  boolean isWest() { return this == WEST; }
  static __NbhdDir fromString(String t) {
    if (t.equalsIgnoreCase("north")) return NORTH;
    if (t.equalsIgnoreCase("south")) return SOUTH;
    if (t.equalsIgnoreCase("east")) return EAST;
    if (t.equalsIgnoreCase("west")) return WEST;
    throw new RuntimeException("Invalid direction: " + t);
  }
}

class __NbhdSquare {
  String color;
  boolean passable;
  int paintCount;
  __NbhdSquare(int tileType, int value) {
    this.passable = (tileType == 1 || tileType == 2 || tileType == 3 || tileType == 5);
    this.paintCount = value;
  }
  void setColor(String c) {
    if (containsPaint()) throw new RuntimeException("Invalid paint location");
    if (passable && paintCount == 0) color = c;
  }
  boolean isPassable() { return passable; }
  void collectPaint() {
    if (containsPaint()) paintCount--;
    else System.out.println("There's no paint to collect here");
  }
  void removePaint() {
    if (color != null) color = null;
    else System.out.println("There's no paint to remove here");
  }
  boolean containsPaint() { return paintCount > 0; }
  String getPrintableDescription() {
    if (!passable) return "x";
    if (color != null) return color;
    return String.valueOf(paintCount);
  }
  boolean hasColor() { return color != null; }
  String getColor() { return color; }
}

class __NbhdGrid {
  __NbhdSquare[][] g;
  int width;
  int height;
  __NbhdGrid(__NbhdSquare[][] squares) { g = squares; height = squares.length; width = squares[0].length; }
  void printGrid() {
    for (int y = 0; y < height; y++) {
      String row = "";
      for (int x = 0; x < width; x++) { if (x > 0) row += ","; row += g[x][y].getPrintableDescription(); }
      System.out.println(row);
    }
  }
  boolean validLocation(int x, int y) {
    return x >= 0 && y >= 0 && x < width && y < height && g[x][y].isPassable();
  }
  __NbhdSquare getSquare(int x, int y) {
    if (validLocation(x, y)) return g[x][y];
    throw new RuntimeException("Could not get square");
  }
  int getSize() { return g.length; }
}

class __NbhdWorld {
  static __NbhdGrid grid;
  // The animation command stream the frontend replays: one JSON
  // ClientMessage per line, matching javabuilder's wire format
  // ({"type":"NEIGHBORHOOD","value":...,"detail":...}). Rewritten to
  // the VFS on each action so it is complete when main returns.
  // A single writer kept open for the whole run: PrintWriter.println
  // appends write-through to the VFS, so each action is O(1) and the
  // file is always complete (no end-of-program hook needed).
  static java.io.PrintWriter writer;
  static void emit(String value, String detail) {
    try {
      if (writer == null) writer = new java.io.PrintWriter(new java.io.File("neighborhood.jsonl"));
      writer.println("{\"type\":\"NEIGHBORHOOD\",\"value\":\"" + value + "\",\"detail\":" + detail + "}");
    } catch (Exception e) {}
  }
  static __NbhdGrid getGrid() {
    if (grid == null) grid = load();
    return grid;
  }
  static __NbhdGrid load() {
    try {
      java.util.Scanner sc = new java.util.Scanner(new java.io.File("grid.txt"));
      java.util.ArrayList<String> lines = new java.util.ArrayList<String>();
      while (sc.hasNextLine()) {
        String ln = sc.nextLine().trim();
        if (ln.length() > 0) lines.add(ln);
      }
      sc.close();
      int height = lines.size();
      int width = lines.get(0).split(" ").length;
      __NbhdSquare[][] sq = new __NbhdSquare[width][height];
      for (int y = 0; y < height; y++) {
        String[] cells = lines.get(y).split(" ");
        for (int x = 0; x < width; x++) {
          String[] parts = cells[x].split(",");
          int tileType = Integer.parseInt(parts[0]);
          int value = parts.length > 1 ? Integer.parseInt(parts[1]) : 0;
          sq[x][y] = new __NbhdSquare(tileType, value);
        }
      }
      return new __NbhdGrid(sq);
    } catch (Exception e) {
      __NbhdSquare[][] sq = new __NbhdSquare[10][10];
      for (int i = 0; i < 10; i++) for (int j = 0; j < 10; j++) sq[i][j] = new __NbhdSquare(1, 0);
      return new __NbhdGrid(sq);
    }
  }
}

class Painter {
  private static int lastId = 0;
  int x, y, remainingPaint;
  __NbhdDir direction;
  boolean hasInfinitePaint;
  __NbhdGrid grid;
  String id;
  public Painter() { this(0, 0, "East", 0, true); }
  public Painter(int x, int y, String direction, int paint) { this(x, y, direction, paint, false); }
  private Painter(int x, int y, String direction, int paint, boolean couldBeInfinite) {
    this.x = x; this.y = y;
    this.direction = __NbhdDir.fromString(direction);
    this.remainingPaint = paint;
    this.grid = __NbhdWorld.getGrid();
    int gridSize = grid.getSize();
    this.hasInfinitePaint = couldBeInfinite ? gridSize >= 20 : false;
    if (x < 0 || y < 0 || x >= gridSize || y >= gridSize) throw new RuntimeException("Invalid location");
    this.id = "painter-" + lastId; lastId++;
    __NbhdWorld.emit("INITIALIZE_PAINTER", "{" + __idf() + ",\"direction\":\"" + this.direction.getDirectionString()
        + "\",\"x\":\"" + x + "\",\"y\":\"" + y + "\",\"paint\":\"" + remainingPaint + "\"}");
  }
  private String __idf() { return "\"id\":\"" + id + "\""; }
  public void turnLeft() {
    direction = direction.turnLeft();
    __NbhdWorld.emit("TURN_LEFT", "{" + __idf() + ",\"direction\":\"" + direction.getDirectionString() + "\"}");
  }
  public void move() {
    if (isValidMovement(direction)) {
      if (direction.isNorth()) y--;
      else if (direction.isSouth()) y++;
      else if (direction.isEast()) x++;
      else x--;
    } else {
      throw new RuntimeException("Invalid move");
    }
    __NbhdWorld.emit("MOVE", "{" + __idf() + ",\"direction\":\"" + direction.getDirectionString() + "\"}");
  }
  public void paint(String color) {
    if (hasPaint()) {
      grid.getSquare(x, y).setColor(color);
      remainingPaint--;
      __NbhdWorld.emit("PAINT", "{" + __idf() + ",\"color\":\"" + color + "\"}");
    } else {
      System.out.println("There is no more paint in the painter's bucket");
    }
  }
  public void scrapePaint() {
    grid.getSquare(x, y).removePaint();
    __NbhdWorld.emit("REMOVE_PAINT", "{" + __idf() + "}");
  }
  public int getMyPaint() { return remainingPaint; }
  public void hidePainter() { __NbhdWorld.emit("HIDE_PAINTER", "{" + __idf() + "}"); }
  public void showPainter() { __NbhdWorld.emit("SHOW_PAINTER", "{" + __idf() + "}"); }
  public void takePaint() {
    if (grid.getSquare(x, y).containsPaint()) {
      grid.getSquare(x, y).collectPaint();
      remainingPaint++;
      __NbhdWorld.emit("TAKE_PAINT", "{" + __idf() + "}");
    } else {
      System.out.println("There is no paint to collect here");
    }
  }
  public boolean isOnPaint() { return grid.getSquare(x, y).hasColor(); }
  public boolean isOnBucket() { return grid.getSquare(x, y).containsPaint(); }
  public boolean hasPaint() { if (hasInfinitePaint) return true; return remainingPaint > 0; }
  public boolean canMove(String d) { return isValidMovement(__NbhdDir.fromString(d)); }
  public boolean canMove() { return isValidMovement(direction); }
  public String getColor() { return grid.getSquare(x, y).getColor(); }
  public boolean isFacingNorth() { return direction.isNorth(); }
  public boolean isFacingEast() { return direction.isEast(); }
  public boolean isFacingSouth() { return direction.isSouth(); }
  public boolean isFacingWest() { return direction.isWest(); }
  public boolean facingNorth() { return isFacingNorth(); }
  public boolean facingEast() { return isFacingEast(); }
  public boolean facingSouth() { return isFacingSouth(); }
  public boolean facingWest() { return isFacingWest(); }
  public int getX() { return x; }
  public int getY() { return y; }
  public String getDirection() { return direction.getDirectionString(); }
  public void showBuckets() { __NbhdWorld.emit("SHOW_BUCKETS", "{}"); }
  public void hideBuckets() { __NbhdWorld.emit("HIDE_BUCKETS", "{}"); }
  public void setPaint(int paint) {
    if (paint < 0) { System.out.println("Paint amount must not be a negative number."); return; }
    if (hasInfinitePaint) return;
    remainingPaint = paint;
  }
  private boolean isValidMovement(__NbhdDir d) {
    if (d.isNorth()) return grid.validLocation(x, y - 1);
    if (d.isSouth()) return grid.validLocation(x, y + 1);
    if (d.isEast()) return grid.validLocation(x + 1, y);
    return grid.validLocation(x - 1, y);
  }
}
