// AUTO-GENERATED from Code.org CSA Unit 1 (csa1-2025). Do not edit by hand.
// Each level's grid comes from its serialized_maze; the runner class is
// renamed to Main so the playground's entry point resolves.

export interface NeighborhoodLevelFile {
  path: string;
  text: string;
}

export interface NeighborhoodLevel {
  name: string;
  lesson: string;
  grid: string;
  files: NeighborhoodLevelFile[];
}

export const NEIGHBORHOOD_LEVELS: NeighborhoodLevel[] = [
  {
    name: 'Investigate and Modify: Working with Java Files',
    lesson: 'Lesson 2: Java Lab',
    grid: '1,0 1,1 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter\n    Painter sofia = new Painter();\n\n    sofia.move();\n\n    sofia.takePaint();\n\n    sofia.turnLeft();\n    sofia.turnLeft();\n    sofia.turnLeft();\n\n    sofia.move();\n    sofia.paint("orange");\n\n    // ---------- ADD YOUR CODE BELOW THIS LINE ----------\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Skill Building: Painter Objects',
    lesson: 'Lesson 4: Instantiating Objects',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate the missing Painter object.\n     * -----------------------------------------------------------------------------\n     */\n\n    Painter ezra = new Painter();\n\n    ezra.turnLeft();\n    ezra.turnLeft();\n    ezra.turnLeft();\n\n    ezra.move();\n    ezra.move();\n    ezra.move();\n    ezra.move();\n    ezra.move();\n\n    ezra.turnLeft();\n\n    ezra.move();\n    ezra.move();\n    ezra.move();\n    ezra.move();\n    ezra.move();\n\n    ezra.turnLeft();\n\n    ezra.move();\n    ezra.move();\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Painter Objects',
    lesson: 'Lesson 4: Instantiating Objects',
    grid: '1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate the missing Painter object.\n     * -----------------------------------------------------------------------------\n     */\n\n    Painter blake = new Painter();\n\n    blake.move();\n    blake.move();\n    blake.move();\n\n    blake.turnLeft();\n    blake.turnLeft();\n    blake.turnLeft();\n\n    blake.move();\n    blake.move();\n    blake.move();\n    blake.move();\n    blake.move();\n    blake.move();\n    blake.move();\n\n    blake.turnLeft();\n    blake.turnLeft();\n    blake.turnLeft();\n\n    blake.move();\n    blake.move();\n    blake.move();\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painter Class Methods',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object\n    Painter silas = new Painter();\n\n    // Moves forward three spaces\n    silas.move();\n    silas.move();\n    silas.move();\n\n    // Turns right by turning left three times\n    silas.turnLeft();\n    silas.turnLeft();\n    silas.turnLeft();\n\n    // Moves forward two spaces\n    silas.move();\n    silas.move();\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n    silas.turnLeft();\n    silas.turnLeft();\n\n    silas.move();\n    silas.move();\n\n    silas.turnLeft();\n\n    silas.move();\n    silas.move();\n    silas.move();\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painting (1)',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'public class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 instructions for the student to complete\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 instructions for the student to complete\n     * -----------------------------------------------------------------------------\n     *\n     * description of method to write\n     */\n\n    \n\n    \n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n    \n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painting (2)',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'public class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 instructions for the student to complete\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 instructions for the student to complete\n     * -----------------------------------------------------------------------------\n     *\n     * description of method to write\n     */\n\n    \n\n    \n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n    \n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painting (3)',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'public class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 instructions for the student to complete\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 instructions for the student to complete\n     * -----------------------------------------------------------------------------\n     *\n     * description of method to write\n     */\n\n    \n\n    \n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n    \n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigating AI-Generated Code',
    lesson: 'Lesson 8: AI-Generated Code',
    grid: '1,100 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    FlagPainter flag = new FlagPainter();\n    flag.takeAllPaint();\n    flag.paintFlagOfBelgium();\n  }\n}',
      },
      {
        path: 'FlagPainter.java',
        text: 'import org.code.neighborhood.*;\n\npublic class FlagPainter extends PainterPlus {\n  \n  /*\n   * Paints the entire flag of Belgium\n   */\n  public void paintFlagOfBelgium() {\n    paintStripe("black");\n    moveToNextStripe();\n    paintStripe("yellow");\n    moveToNextStripe();\n    paintStripe("red");\n    resetPosition();\n  }\n\n  /*\n   * Paints a vertical stripe of the flag\n   */\n  public void paintStripe(String color) {\n    for (int i = 0; i < 8; i++) {\n      paint(color);\n      if (canMove()) {\n        move();\n      }\n    }\n    turnAround();\n    for (int i = 0; i < 7; i++) {\n      move();\n    }\n    turnLeft();\n  }\n\n  /*\n   * Moves to the starting position for the next stripe\n   */\n  public void moveToNextStripe() {\n    move();\n    turnRight();\n  }\n\n  /*\n   * Resets the Painter to the starting position\n   */\n  public void resetPosition() {\n    turnLeft();\n    while (canMove()) {\n      move();\n    }\n    turnLeft();\n    while (canMove()) {\n      move();\n    }\n    turnAround();\n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all of the paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n\n  // Turns around\n  public void turnAround() {\n    turnLeft();\n    turnLeft();\n  }\n\n  // Paints a line based on how much paint the Painter has\n  public void paintLine(int paintAmount, String color) {\n    setPaint(paintAmount);\n\n    while (hasPaint()) {\n      paint(color);\n      \n      if (canMove()) {\n        move();\n      }\n    }\n  }\n\n  // Moves to the next row\n  public void moveToNextRow() {\n    if (canMove("south")) {\n      if (isFacingEast()) {\n        turnRight();\n        move();\n        turnRight();\n      }\n      else {\n        turnLeft();\n        move();\n        turnLeft();\n      }\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Creating PainterPlus (1)',
    lesson: 'Lesson 9: Inheritance',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n    // Creates a PainterPlus object\n    PainterPlus amelia = new PainterPlus();\n\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Creating PainterPlus (2)',
    lesson: 'Lesson 9: Inheritance',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n    // Creates a PainterPlus object\n    PainterPlus amelia = new PainterPlus();\n\n    // Moves forward until unable to move\n    while (amelia.canMove()) {\n      amelia.move();\n    }\n\n    // Turns right by turning left three times\n    amelia.turnLeft();\n    amelia.turnLeft();\n    amelia.turnLeft();\n\n    // Moves forward until unable to move\n    while (amelia.canMove()) {\n      amelia.move();\n    }\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Writing Methods',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n    // Creates a PainterPlus object\n    PainterPlus amelia = new PainterPlus();\n\n    // Moves forward until unable to move\n    while (amelia.canMove()) {\n      amelia.move();\n    }\n\n    // Turns right by turning left three times\n    amelia.turnRight();\n\n    // Moves forward until unable to move\n    while (amelia.canMove()) {\n      amelia.move();\n    }\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n\n  // Turns the Painter object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate: Programming Style and Tools (1)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.Painter;\n\npublic class Main {\npublic static void main(String[] args) {\nPainterPlus p = new PainterPlus();\np.turnRight();\np.move();\np.move();\np.turnLeft();\np.move();\np.move();\np.turnRight();\np.move();\np.turnLeft();\np.move();\np.move();\np.move();\n}\n}\n',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.Painter;\n\npublic class PainterPlus extends Painter {\npublic void turnRight() {\nturnLeft();\nturnLeft();\nturnLeft();\n} \n}',
      },
    ],
  },
  {
    name: 'Investigate: Programming Style and Tools (2)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.Painter;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus myPainter = new PainterPlus();\n\n    /*\n     * Moves south to the sidewalk\n     */\n    myPainter.turnRight(); // Turns right to face south\n    myPainter.move();\n    myPainter.move();\n\n    /*\n     * Moves east to the sidewalk\n     */\n    myPainter.turnLeft(); // Turns left to face east\n    myPainter.move();\n    myPainter.move();\n\n    /*\n     * Moves south to the sidewalk\n     */\n    myPainter.turnRight(); // Turns right to face south\n    myPainter.move();\n\n    /*\n     * Moves east to the food truck\n     */\n    myPainter.turnLeft(); // Turns left to face east\n    myPainter.move();\n    myPainter.move();\n    myPainter.move();\n    \n  }\n}\n',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.Painter;\n\n/*\n * Creates a PainterPlus\n * PainterPlus is a subclass of Painter.\n */\npublic class PainterPlus extends Painter {\n\n  /*\n   * Turns a PainterPlus object to the right\n   * by turning left three times\n   */\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate: Programming Style and Tools (3)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Debugging',
    lesson: 'Lesson 13: Debugging Strategies',
    grid: '1,0 1,20 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus marie = new PainterPlus();\n\n    String currentDirection;\n    int currentPaint;\n\n    marie.move();\n    marie.takeAllPaint();\n\n    while (marie.canMove()) {\n      marie.paintThenMove("coral");\n      currentPaint = marie.getMyPaint();\n      System.out.println("Paint remaining after moving east: " + currentPaint);\n      \n      marie.turnRight();\n      currentDirection = marie.getDirection();\n      System.out.println("Currently facing: " + currentDirection);\n      \n      marie.paintThenMove("coral");\n      marie.turnLeft();\n    }\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n\n  // Paints then moves forward while the Painter can move\n  public void paintThenMove(String color) {\n    while (canMove()) {\n      paint(color);\n      move();\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'With Paint Bucket',
    lesson: 'Sandbox: The Neighborhood',
    grid: '1,0 1,10 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.Painter;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n\n  }\n}',
      },
    ],
  },
  {
    name: 'No Paint Bucket',
    lesson: 'Sandbox: The Neighborhood',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.Painter;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n\n  }\n}',
      },
    ],
  },
];
