// AUTO-GENERATED from Code.org CSA Unit 1 (csa1-2025). Do not edit by hand.
// STARTING code only (no solutions). BubbleChoice bubbles expanded via their
// scripts/*.bubble_choice files; levels that inherit code via
// project_template_level_name pull it from the template. Runner class -> Main.

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
    name: 'Predict and Run: A Java Program',
    lesson: 'Lesson 2: Java Lab',
    grid: '1,0 1,1 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter\n    Painter sofia = new Painter();\n\n    sofia.move();\n\n    sofia.takePaint();\n\n    sofia.turnLeft();\n    sofia.turnLeft();\n    sofia.turnLeft();\n\n    sofia.move();\n    sofia.paint("orange");\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n    \n    \n  }\n}',
      },
    ],
  },
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
    name: 'Predict and Run: Painter Class Methods',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object\n    Painter silas = new Painter();\n\n    // Moves forward three spaces\n    silas.move();\n    silas.move();\n    silas.move();\n\n    // Turns right by turning left three times\n    silas.turnLeft();\n    silas.turnLeft();\n    silas.turnLeft();\n\n    // Moves forward two spaces\n    silas.move();\n    silas.move();\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painter Class Methods #1',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object\n    Painter silas = new Painter();\n\n    // Moves forward three spaces\n    silas.move();\n    silas.move();\n    silas.move();\n\n    // Turns right by turning left three times\n    silas.turnLeft();\n    silas.turnLeft();\n    silas.turnLeft();\n\n    // Moves forward two spaces\n    silas.move();\n    silas.move();\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painter Class Methods #2',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object\n    Painter silas = new Painter();\n\n    // Moves forward three spaces\n    silas.move();\n    silas.move();\n    silas.move();\n\n    // Print direction before turning\n    System.out.println("Before turning: " + silas.getDirection());\n\n    // Turns right by turning left three times\n    silas.turnLeft();\n    silas.turnLeft();\n    silas.turnLeft();\n\n    // Print direction after turning\n    System.out.println("After turning: " + silas.getDirection());\n\n    // Moves forward two spaces\n    silas.move();\n    silas.move();\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n    // Use print statements to show direction before and after each turn you write\n\n\n    \n    \n  }\n}\n',
      },
    ],
  },
  {
    name: 'Practice: Painter Class Methods (a)',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate through The Neighborhood to reach the food truck.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Painter Class Methods (b)',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate through The Neighborhood to reach the traffic cone.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Painter Class Methods (c)',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate through The Neighborhood to reach the house.\n     * -----------------------------------------------------------------------------\n     */\n    \n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Painter Class Methods (d)',
    lesson: 'Lesson 5: Methods',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0\n1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate through The Neighborhood to reach the traffic cone.\n     * -----------------------------------------------------------------------------\n     */\n    \n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Predict and Run: Painting',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0\n0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object\n    Painter ella = new Painter();\n\n    // Moves forward one space\n    ella.move();\n\n    // Takes paint three times to get all the paint from the bucket\n    ella.takePaint();\n    ella.takePaint();\n    ella.takePaint();\n\n    // Moves and paints three times\n    ella.paint("white");\n    ella.move();\n    ella.paint("white");\n    ella.move();\n    ella.paint("white");\n\n    // Turns right by turning left three times\n    ella.turnLeft();\n    ella.turnLeft();\n    ella.turnLeft();\n\n    // Moves forward one space\n    ella.move();\n\n    // Takes paint three times to get all the paint from the bucket\n    ella.takePaint();\n    ella.takePaint();\n    ella.takePaint();\n\n    // Moves and paints three times\n    ella.paint("white");\n    ella.move();\n    ella.paint("white");\n    ella.move();\n    ella.paint("white");\n\n    // Moves forward one space then takes paint three times to get all the paint from the bucket\n    ella.move();\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE HERE TO TAKE PAINT FROM THE PAINT BUCKET ---- */\n\n    \n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Painting #1',
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
    name: 'Investigate and Modify: Painting #2',
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
    name: 'Investigate and Modify: Painting #3',
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
    name: 'Practice: Moving and Painting #1',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,5 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate and paint the path to the traffic cone.\n     * -----------------------------------------------------------------------------\n     */\n    \n\n\n\n    \n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Moving and Painting #2',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate and paint the path to the traffic cone.\n     * -----------------------------------------------------------------------------\n     */\n    \n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Moving and Painting #3',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,3 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,4 1,0 1,0 1,0 1,3 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate and paint the path to the traffic cone.\n     * -----------------------------------------------------------------------------\n     */\n    \n\n\n\n    \n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Moving and Painting #4',
    lesson: 'Lesson 6: Methods with Parameters',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,4 0,0 0,0 0,0 1,0 1,0 0,0 0,0\n1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 1,4 1,0 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,4\n1,4 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n1,0 1,0 1,4 1,0 1,0 1,0 1,4 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Instantiate a Painter object, then use the methods in the Painter class\n     * to navigate and paint the path to the traffic cone.\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n    \n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Predict and Run: Loops and Boolean Methods',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object called jesse\n    Painter jesse = new Painter();\n\n    // Moves forward one space\n    jesse.move();\n\n    while (jesse.isOnBucket()) {\n      jesse.takePaint();\n    }\n\n    while (jesse.hasPaint()) {\n      jesse.move();\n      jesse.paint("pink");\n    }\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    // Moves forward one space\n    jesse.move();\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    while (jesse.canMove()) {\n      jesse.move();\n    }\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Loops and Boolean Methods #1',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object called jesse\n    Painter jesse = new Painter();\n\n    // Moves forward one space\n    jesse.move();\n\n    while (jesse.isOnBucket()) {\n      jesse.takePaint();\n    }\n\n    while (jesse.hasPaint()) {\n      jesse.move();\n      jesse.paint("pink");\n    }\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    // Moves forward one space\n    jesse.move();\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    while (jesse.canMove()) {\n      jesse.move();\n    }\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Loops and Boolean Methods #2',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object called jesse\n    Painter jesse = new Painter();\n\n    // Moves forward one space\n    jesse.move();\n\n    while (jesse.isOnBucket()) {\n      jesse.takePaint();\n    }\n\n    while (jesse.hasPaint()) {\n      jesse.move();\n      jesse.paint("pink");\n    }\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    // Moves forward one space\n    jesse.move();\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    while (jesse.canMove()) {\n      jesse.move();\n    }\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Loops and Boolean Methods #3',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object called jesse\n    Painter jesse = new Painter();\n\n    // Moves forward one space\n    jesse.move();\n\n    while (jesse.isOnBucket()) {\n      jesse.takePaint();\n    }\n\n    while (jesse.hasPaint()) {\n      jesse.move();\n      jesse.paint("pink");\n    }\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    // Moves forward one space\n    jesse.move();\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    while (jesse.canMove()) {\n      jesse.move();\n    }\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Loops and Boolean Methods #4',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a Painter object called jesse\n    Painter jesse = new Painter();\n\n    // Moves forward one space\n    jesse.move();\n\n    while (jesse.isOnBucket()) {\n      jesse.takePaint();\n    }\n\n    while (jesse.hasPaint()) {\n      jesse.move();\n      jesse.paint("pink");\n    }\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    // Moves forward one space\n    jesse.move();\n\n    // Turns right by turning left three times\n    jesse.turnLeft();\n    jesse.turnLeft();\n    jesse.turnLeft();\n\n    // Print whether jesse has paint before entering the loop\n    System.out.println("Has paint? " + jesse.hasPaint());\n\n    while (jesse.canMove()) {\n      System.out.println("Still in loop, current direction: " + jesse.getDirection());\n      jesse.move();\n    }\n    System.out.println("Exited loop.");\n  }\n}\n',
      },
    ],
  },
  {
    name: 'Practice: Using Loops and Boolean Methods (a)',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Implement your algorithm to have a Painter object take all of the paint\n     * from a paint bucket.\n     * -----------------------------------------------------------------------------\n     */\n    \n\n\n\n    \n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Using Loops and Boolean Methods (b)',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Implement your algorithm to have a Painter object move forward one\n     * space as long as it can move in the direction it is facing.\n     * -----------------------------------------------------------------------------\n     */\n    \n\n\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Using Loops and Boolean Methods (c)',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,3 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Implement your algorithm to have a Painter object paint the current\n     * space and move forward one space until it runs out of paint.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Using Loops and Boolean Methods (d)',
    lesson: 'Lesson 7: Loops',
    grid: '1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Implement your algorithm to have a Painter object move forward, turn,\n     * and paint in a square until it runs out of paint.\n     * -----------------------------------------------------------------------------\n     */\n    \n\n\n\n\n\n    \n    \n  }\n}',
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
    name: 'Practice: Creating PainterPlus #1',
    lesson: 'Lesson 9: Inheritance',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /*\n     * ------------------------------ TO DO ------------------------------\n     * Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -------------------------------------------------------------------\n     */\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Creating PainterPlus #2',
    lesson: 'Lesson 9: Inheritance',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Predict and Run: Writing Methods',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus olivia = new PainterPlus();\n\n    olivia.move();\n\n    while (olivia.isOnBucket()) {\n      olivia.takePaint();\n    }\n\n    olivia.paintSquare();\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Paints a square\n  public void paintSquare() {\n    while (hasPaint()) {\n      move();\n      paint("blue");\n      turnLeft();\n      turnLeft();\n      turnLeft();\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Void Methods #1',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus olivia = new PainterPlus();\n\n    olivia.move();\n\n    while (olivia.isOnBucket()) {\n      olivia.takePaint();\n    }\n\n    olivia.paintSquare();\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Paints a square\n  public void paintSquare() {\n    while (hasPaint()) {\n      move();\n      paint("blue");\n      turnLeft();\n      turnLeft();\n      turnLeft();\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Void Methods #2',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus olivia = new PainterPlus();\n\n    olivia.move();\n\n    while (olivia.isOnBucket()) {\n      olivia.takePaint();\n    }\n\n    olivia.paintSquare();\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Paints a square\n  public void paintSquare() {\n    while (hasPaint()) {\n      move();\n      paint("blue");\n      turnLeft();\n      turnLeft();\n      turnLeft();\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Void Methods #3',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus olivia = new PainterPlus();\n\n    olivia.move();\n\n    while (olivia.isOnBucket()) {\n      olivia.takePaint();\n    }\n\n    olivia.paintSquare();\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Paints a square\n  public void paintSquare() {\n    while (hasPaint()) {\n      move();\n      paint("blue");\n      turnLeft();\n      turnLeft();\n      turnLeft();\n    }\n  }\n  \n}',
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
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n\n\n    \n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: PainterPlus Methods (a)',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,0 1,6 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n\n\n    \n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: PainterPlus Methods (b)',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n\n\n    \n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: PainterPlus Methods (c)',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,4 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: PainterPlus Methods (d)',
    lesson: 'Lesson 10: Writing Methods',
    grid: '1,0 1,0 1,0 1,0 1,8 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n\n\n    \n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate: Programming Style and Tools #1',
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
    name: 'Investigate: Programming Style and Tools #2',
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
    name: 'Investigate: Programming Style and Tools #3',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the PainterPlus class, then instantiate a PainterPlus object.\n     * -----------------------------------------------------------------------------\n     */\n\n\n\n\n\n    \n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Investigate: Programming Style and Tools #4',
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
    name: 'Practice: Conducting a Code Review (a)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 1,0 1,0 1,0 1,0 1,6 0,0 1,0 1,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus colin = new PainterPlus();\n\n    // Moves forward until PainterPlus reaches obstacle\n    while (colin.canMove()) {\n      colin.move();\n    }\n\n    // Takes all the paint from the paint bucket\n    while (colin.isOnBucket()) {\n      colin.takePaint();\n    }\n\n    // Turns right\n    colin.turnRight();\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Call the move() and paintLongDashes() methods to move forward and paint\n     * dashed lines behind the taxes while the PainterPlus object can move forward.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n    \n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Paints and moves two spaces\n  public void paintLongDashes(String color) {\n    paint(color);\n    move();\n    paint(color);\n    move();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Conducting a Code Review (b)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus callie = new PainterPlus();\n\n    // Turns right\n    callie.turnRight();\n\n    // Moves forward while PainterPlus can move then turns left\n    callie.moveTurnLeft();\n\n    // Moves forward while PainterPlus can move then turns right\n    callie.moveTurnRight();\n\n    // Moves forward while PainterPlus can move\n    while (callie.canMove()) {\n      callie.move();\n    }\n\n    // Moves forward while PainterPlus can move then turns right\n    callie.moveTurnRight();\n\n    // Moves forward while PainterPlus can move then turns left\n    callie.moveTurnLeft();\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Call the moveTurnLeft() and move() methods to move forward while\n     * PainterPlus can move then turn left then continue moving to reach the end.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n    \n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Moves forward while the Painter object can move then turns left\n  public void moveTurnLeft() {\n    while (canMove()) {\n      move();\n    }\n\n    turnLeft();\n  }\n\n  // Moves forward while the Painter object can move then turns right\n  public void moveTurnRight() {\n    while (canMove()) {\n      move();\n    }\n\n    turnRight();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Conducting a Code Review (c)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,12 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0\n0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0\n0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0\n0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,12\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus melissa = new PainterPlus();\n\n    // Turns right\n    melissa.turnRight();\n\n    // Moves forward two spaces\n    melissa.move();\n    melissa.move();\n    melissa.move();\n\n    // Takes all the paint from the paint bucket\n    melissa.takeAllPaint();\n\n    // Turns left\n    melissa.turnLeft();\n\n    // Paints a line while PainterPlus can move\n    while (melissa.canMove()) {\n      melissa.paintThenMove("white");\n    }\n\n    // Paints the last space\n    melissa.paint("white");\n\n    // Turns right\n    melissa.turnRight();\n\n    // Moves forward while PainterPlus can move\n    while (melissa.canMove()) {\n      melissa.move();\n    }\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Call the takeAllPaint(), turnRight(), and paintThenMove() methods to take all\n     * the paint from the bucket, turn right, then paint a line behind the food trucks.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n\n    \n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all the paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n\n  // Paints then moves forward one space\n  public void paintThenMove(String color) {\n    paint(color);\n    move();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Conducting a Code Review (d)',
    lesson: 'Lesson 11: Programming Style and Feedback',
    grid: '1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,5 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n1,5 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0\n1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 1,5 1,0 1,0\n1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,5 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 1,0 1,0 1,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus brooke = new PainterPlus();\n\n    // Moves forward one space\n    brooke.move();\n\n    // Takes all paint from the paint bucket\n    brooke.takeAllPaint();\n\n    // Moves while PainterPlus can move then turns right\n    brooke.moveTurnRight();\n\n    // Moves while PainterPlus can move\n    while (brooke.canMove()) {\n      brooke.move();\n    }\n\n    // Takes all paint from the paint bucket\n    brooke.takeAllPaint();\n\n    // Turns right\n    brooke.turnRight();\n\n    // Moves while PainterPlus can move then turns left\n    brooke.moveTurnLeft();\n\n    // Moves while PainterPlus can move then turns right\n    brooke.moveTurnRight();\n\n    // Moves while PainterPlus can move\n    while (brooke.canMove()) {\n      brooke.move();\n    }\n\n    // Takes all paint from the paint bucket\n    brooke.takeAllPaint();\n\n    // Turns left\n    brooke.turnLeft();\n\n    // Moves while PainterPlus can move then turns left\n    brooke.moveTurnLeft();\n\n    // Moves while PainterPlus can move\n    while (brooke.canMove()) {\n      brooke.move();\n    }\n\n    // Takes all paint from the paint bucket\n    brooke.takeAllPaint();\n\n    // Turns around\n    brooke.turnLeft();\n    brooke.turnLeft();\n\n    // Moves while PainterPlus can move then turns right\n    brooke.moveTurnRight();\n\n    // Moves while PainterPlus can move then turns right\n    brooke.moveTurnRight();\n\n    // Moves while PainterPlus can move then turns right\n    brooke.moveTurnRight();\n\n    // Moves while PainterPlus can move then turns left\n    brooke.moveTurnLeft();\n\n    // Moves while PainterPlus can move then turns left\n    brooke.moveTurnLeft();\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Call the moveTurnLeft(), move(), and takeAllPaint() methods to navigate to\n     * the last paint bucket and take all of the paint from the paint bucket.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n\n    \n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n\n  // Moves while the Painter object can move then turns left\n  public void moveTurnLeft() {\n    while (canMove()) {\n      move();\n    }\n\n    turnLeft();\n  }\n\n  // Moves while the Painter object can move then turns right\n  public void moveTurnRight() {\n    while (canMove()) {\n      move();\n    }\n\n    turnRight();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Predict and Run: Selection Statements',
    lesson: 'Lesson 12: Selection Statements',
    grid: '1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0\n1,1 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus akira = new PainterPlus();\n\n    if (akira.canMove("south")) {\n      akira.turnRight();\n      akira.move();\n    }\n\n    if (akira.isOnBucket()) {\n      akira.takePaint();\n      akira.turnLeft();\n      akira.move();\n      akira.paint("orange");\n    }\n\n    if (akira.isOnPaint()) {\n      akira.move();\n    }\n\n    if (akira.isFacingEast()) {\n      akira.move();\n    }\n\n    if (akira.canMove("north")) {\n      akira.turnLeft();\n      akira.move();\n    }\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n\n\n    \n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the PainterPlus object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Selection Statements',
    lesson: 'Lesson 12: Selection Statements',
    grid: '1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0\n1,1 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus akira = new PainterPlus();\n\n    if (akira.canMove("south")) {\n      akira.turnRight();\n      akira.move();\n    }\n\n    if (akira.isOnBucket()) {\n      akira.takePaint();\n      akira.turnLeft();\n      akira.move();\n      akira.paint("orange");\n    }\n\n    if (akira.isOnPaint()) {\n      akira.move();\n    }\n\n    if (akira.isFacingEast()) {\n      akira.move();\n    }\n\n    if (akira.canMove("north")) {\n      akira.turnLeft();\n      akira.move();\n    }\n\n    /* ---- \ud83d\udd0e ADD YOUR CODE BELOW THIS LINE ---- */\n\n\n\n\n    \n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the PainterPlus object to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Practice: Writing Algorithms (a)',
    lesson: 'Lesson 12: Selection Statements',
    grid: '1,0 1,0 1,0 1,6 0,0 0,0 1,0 1,0 1,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,6 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the TrafficPainter class and the method paintLines().\n     * Then, instantiate a TrafficPainter object and use the paintLines() method.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Writing Algorithms (b)',
    lesson: 'Lesson 12: Selection Statements',
    grid: '1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the MarathonPainter class and the methods moveToSouth() and moveToEast().\n     * Then, instantiate a MarathonPainter object and use the methods you wrote.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n\n\n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Writing Algorithms (c)',
    lesson: 'Lesson 12: Selection Statements',
    grid: '1,0 1,12 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0\n1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the MuralPainter class and the method paintDiagonal().\n     * Then, instantiate a MuralPainter object and use the paintDiagonal() method.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n    \n\n\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Practice: Writing Algorithms (d)',
    lesson: 'Lesson 12: Selection Statements',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0 0,0\n1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,6 1,0 1,0 0,0\n1,0 1,0 0,0 0,0 1,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0\n1,0 1,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 0,0 1,0 1,6 0,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 0,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 0,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    /* ----------------------------------- TO DO -----------------------------------\n     * \u2705 Write the EventPainter class and the methods takePaintThenPaint(), moveToNextRow(),\n     * and paintRow(). Then, instantiate a EventPainter object and use the methods.\n     * -----------------------------------------------------------------------------\n     */\n\n    \n\n\n\n\n    \n    \n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Predict and Run: Variables and Printing',
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
    name: 'Practice: Debugging in The Neighborhood (a)',
    lesson: 'Lesson 13: Debugging Strategies',
    grid: '1,0 1,13 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0\n0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    // Creates a PainterPlus object\n    PainterPlus molly = new PainterPlus();\n\n    // Move forward while PainterPlus can move\n    while (molly.canMove()) {\n      // Take all of the paint if PainterPlus is on a paint bucket\n      if (molly.isOnBucket()) {\n        molly.takeAllPaint();\n      }\n\n      // Paint the current space if PainterPlus has paint\n      if (molly.hasPaint()) {\n        molly.paint("red");\n      }\n\n      // Move forward one space\n      molly.move();\n    }\n\n    // Paint while PainterPlus has paint\n    while (molly.hasPaint()) {\n      molly.paint("red");\n\n      // Move forward one space if PainterPlus can move\n      if (molly.canMove()) {\n        molly.move();\n      }\n    }\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  // Turns the Painter to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all of the paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Predict and Run: Selection and Logic',
    lesson: 'Lesson 14: Two-Way Selection Statements',
    grid: '1,0 1,7 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 0,0 1,6 1,0 1,2 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,5 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,3 1,0 1,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus sierra = new PainterPlus();\n    sierra.paintToAndAroundTruck("white");\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  public void paintToAndAroundTruck(String color) {\n    while (canMove()) {\n      paintPath(color);\n    }\n    \n    makeTurn();\n\n    while (canMove()) {\n      paintTruckBorder(color);\n      returnToStart();\n    }\n  }\n\n  public void paintPath(String color) {\n    moveOrTakePaint();\n    paintIfHasPaint(color);\n    turnSouth();\n  }\n\n  public void paintTruckBorder(String color) {\n    moveOrTakePaint();\n    paintIfHasPaint(color);\n    makeTurn();\n  }\n\n  // Turns the Painter to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all of the paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n\n  public void moveOrTakePaint() {\n    if (!isOnBucket()) {\n      move();\n    }\n\n    takeAllPaint();\n  }\n\n  public void paintIfHasPaint(String color) {\n    if (hasPaint()) {\n      paint(color);\n    }\n  }\n\n  public void turnSouth() {\n    if (canMove("south")) {\n      turnRight();\n    }\n  }\n\n  public void makeTurn() {\n    if (!canMove()) {\n      turnLeft();\n    }\n  }\n\n  public void returnToStart() {\n    if (isFacingWest()) {\n      if (canMove("north")) {\n        turnRight();\n      }\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Investigate and Modify: Selection and Logic',
    lesson: 'Lesson 14: Two-Way Selection Statements',
    grid: '1,0 1,7 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 0,0 1,6 1,0 1,2 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,5 0,0 0,0\n0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,3 1,0 1,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n    PainterPlus sierra = new PainterPlus();\n    sierra.paintToAndAroundTruck("white");\n    \n  }\n}',
      },
      {
        path: 'PainterPlus.java',
        text: 'import org.code.neighborhood.*;\n\npublic class PainterPlus extends Painter {\n\n  public void paintToAndAroundTruck(String color) {\n    while (canMove()) {\n      paintPath(color);\n    }\n    \n    makeTurn();\n\n    while (canMove()) {\n      paintTruckBorder(color);\n      returnToStart();\n    }\n  }\n\n  public void paintPath(String color) {\n    moveOrTakePaint();\n    paintIfHasPaint(color);\n    turnSouth();\n  }\n\n  public void paintTruckBorder(String color) {\n    moveOrTakePaint();\n    paintIfHasPaint(color);\n    makeTurn();\n  }\n\n  // Turns the Painter to the right\n  public void turnRight() {\n    turnLeft();\n    turnLeft();\n    turnLeft();\n  }\n\n  // Takes all of the paint from a paint bucket\n  public void takeAllPaint() {\n    while (isOnBucket()) {\n      takePaint();\n    }\n  }\n\n  public void moveOrTakePaint() {\n    if (!isOnBucket()) {\n      move();\n    }\n\n    takeAllPaint();\n  }\n\n  public void paintIfHasPaint(String color) {\n    if (hasPaint()) {\n      paint(color);\n    }\n  }\n\n  public void turnSouth() {\n    if (canMove("south")) {\n      turnRight();\n    }\n  }\n\n  public void makeTurn() {\n    if (!canMove()) {\n      turnLeft();\n    }\n  }\n\n  public void returnToStart() {\n    if (isFacingWest()) {\n      if (canMove("north")) {\n        turnRight();\n      }\n    }\n  }\n  \n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #1',
    lesson: 'Lesson 15a: Asphalt Art Project',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #2',
    lesson: 'Lesson 15a: Asphalt Art Project',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #3',
    lesson: 'Lesson 15a: Asphalt Art Project',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #4',
    lesson: 'Lesson 15a: Asphalt Art Project',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #1',
    lesson: 'Lesson 15b: Asphalt Art Project [1-Day Version]',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #2',
    lesson: 'Lesson 15b: Asphalt Art Project [1-Day Version]',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #3',
    lesson: 'Lesson 15b: Asphalt Art Project [1-Day Version]',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
      },
    ],
  },
  {
    name: 'Asphalt Art Project #4',
    lesson: 'Lesson 15b: Asphalt Art Project [1-Day Version]',
    grid: '1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0\n',
    files: [
      {
        path: 'Main.java',
        text: 'import org.code.neighborhood.*;\n\npublic class Main {\n  public static void main(String[] args) {\n\n\n\n    \n  }\n}',
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
