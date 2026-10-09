var e=[{name:`Predict and Run: A Java Program`,lesson:`Lesson 2: Java Lab`,view:`neighborhood`,grid:`1,0 1,1 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter
    Painter sofia = new Painter();

    sofia.move();

    sofia.takePaint();

    sofia.turnLeft();
    sofia.turnLeft();
    sofia.turnLeft();

    sofia.move();
    sofia.paint("orange");

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Working with Java Files`,lesson:`Lesson 2: Java Lab`,view:`neighborhood`,grid:`1,0 1,1 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter
    Painter sofia = new Painter();

    sofia.move();

    sofia.takePaint();

    sofia.turnLeft();
    sofia.turnLeft();
    sofia.turnLeft();

    sofia.move();
    sofia.paint("orange");

    // ---------- ADD YOUR CODE BELOW THIS LINE ----------


    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Painter moves at least three times => ")
  public void testPainterMovesThreeTimes() {
    int numMoves = primaryPainterLog.actionCount(NeighborhoodActionType.MOVE);

    if (numMoves == 0) {
      message = "The Painter does not move forward. Try giving the move() instruction to the Painter.";
    }
    else if (numMoves == 1) {
      message = "The Painter only moves forward once. Try giving the move() instruction to the Painter to move two more times.";
    }
    else if (numMoves == 2) {
      message = "The Painter only moves forward twice. Try giving the move() instruction to the Painter to move one more time.";
    }

    message += messageGap;
    
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.MOVE, 3), message);
  }

}`}],dataFiles:[]},{name:`Practice: Syntax Errors (a)`,lesson:`Lesson 2: Java Lab`,view:`neighborhood`,grid:`1,0 1,3 1,0 1,0 1,0 1,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter called amelia
    Painter amelia = new Painter();

    // Moves forward one space
    amelia.move();

    // Takes paint three times to get all the paint from the bucket
    amelia.takePaint();
    amelia.takepaint();
    amelia.takePaint();

    // Turns right by turning left three times
    amelia.turnLeft();
    amelia.turnLeft();
    amelia.turnLeft();

    // Moves and paints three times
    amelia.move();
    amelia.paint("LawnGreen");
    amelia.move();
    amelia.paint("LawnGreen");
    amelia.move();
    amelia.paint("LawnGreen");

    // Moves forward one space to get off the paint
    amelia.move();
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Painter paints three times => ")
  public void testPainterPaintsThreeTimes() {
    int numPaint = primaryPainterLog.actionCount(NeighborhoodActionType.PAINT);

    if (numPaint == 0) {
      message = "The Painter does not paint any spaces. Try giving the paint() instruction to the Painter.";
    }
    else if (numPaint == 1) {
      message = "The Painter only paints one space. Try giving the paint() instruction to the Painter to paint two more times.";
    }
    else if (numPaint == 2) {
      message = "The Painter only paints two spaces. Try giving the paint() instruction to the Painter to paint one more time.";
    }

    message += messageGap;
    
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.PAINT, 3), message);
  }

}`}],dataFiles:[]},{name:`Practice: Syntax Errors (b)`,lesson:`Lesson 2: Java Lab`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,3 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter called mason
    Painter mason = new Painter();

    // Moves forward one space
    mason.move();

    // Turns right by turning left three times
    mason.turnLeft();
    mason.turnLeft();
    mason.turnLeft();

    // Moves forward one space
    mason.move();

    // Takes paint three times to get all the paint from the bucket
    mason.takePaint();
    mason.takePaint();
    mason.takepaint();

    // Moves and paints three times
    mason.move();
    mason.paint("SkyBlue");
    mason.move();
    mason.paint("SkyBlue");
    mason.move()
    mason.paint("SkyBlue");

    // Moves forward one space to get off the paint
    mason.move();
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Painter paints three times => ")
  public void testPainterPaintsThreeTimes() {
    int numPaint = primaryPainterLog.actionCount(NeighborhoodActionType.PAINT);

    if (numPaint == 0) {
      message = "The Painter does not paint any spaces. Try giving the paint() instruction to the Painter.";
    }
    else if (numPaint == 1) {
      message = "The Painter only paints one space. Try giving the paint() instruction to the Painter to paint two more times.";
    }
    else if (numPaint == 2) {
      message = "The Painter only paints two spaces. Try giving the paint() instruction to the Painter to paint one more time.";
    }

    message += messageGap;
    
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.PAINT, 3), message);
  }

}`}],dataFiles:[]},{name:`Practice: Syntax Errors (c)`,lesson:`Lesson 2: Java Lab`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,4 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter called aria
    Painter aria = new Painter();

    // Moves forward four spaces
    aria.move();
    aria.move();
    aria.move();
    aria.move();

    // Turns right by turning left three times
    aria.turnLeft();
    aria.turnLeft()
    aria.turnLeft();

    // Moves forward one space
    aria.move();

    // Takes paint four times to get all the paint from the bucket
    aria.takePaint();
    aRia.takePaint();
    aria.takePaint();
    aria.takePaint();

    // Moves and paints four times
    aria.move();
    aria.paint("plum");
    aria.move();
    aria.paint("plum");
    aria.move;
    aria.paint("plum");
    aria.move();
    aria.paint("plum");

    // Moves forward one space to get off the paint
    aria.move();
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Painter paints three times => ")
  public void testPainterPaintsThreeTimes() {
    int numPaint = primaryPainterLog.actionCount(NeighborhoodActionType.PAINT);

    if (numPaint == 0) {
      message = "The Painter does not paint any spaces. Try giving the paint() instruction to the Painter.";
    }
    else if (numPaint == 1) {
      message = "The Painter only paints one space. Try giving the paint() instruction to the Painter to paint three more times.";
    }
    else if (numPaint == 2) {
      message = "The Painter only paints two spaces. Try giving the paint() instruction to the Painter to paint two more times.";
    }
    else if (numPaint == 3) {
      message = "The Painter only paints three spaces. Try giving the paint() instruction to the Painter to paint one more time.";
    }

    message += messageGap;
    
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.PAINT, 4), message);
  }

}`}],dataFiles:[]},{name:`Practice: Syntax Errors (d)`,lesson:`Lesson 2: Java Lab`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,4 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,4 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter called madison
    Painter madison = new Painter();

    // Moves forward one space
    madison.move();

    // Turns right by turning left three times
    madison.turnLeft;
    madison.turnLeft();
    madison.turnLeft();

    // Moves forward four spaces
    madison.move();
    madison.move();
    madison.move();
    madison.move();

    // Takes paint four times to get all the paint from the bucket
    madison.takePaint();
    madison.takepaint();
    madison.takePaint();
    madison.takePaint();

    // Paints the first half of the smile
    madison.paint("gold");
    madison.move();
    madison.turnLeft();
    madison.move();
    madison.paint("gold");
    madison.move();
    madison.turnLeft();
    madison.turnLeft()
    madison.turnleft();
    madison.move();
    madison.paint("gold");
    madison.turnLeft();
    madison.move();
    madison.paint("gold");
    madison.move();

    // Takes paint four times to get all the paint from the bucket
    madison.takePaint();
    madison.takePaint();
    madison.takePaint();
    madison.takePaint();

    // Paints the second half of the smile
    madison.paint("gold");
    madison.move();
    madison.paint("gold");
    madison.move();
    madison.turnLeft();
    madison.move();
    madison.paint("gold");
    madison.turnLeft();
    madison.turnLeft();
    madison.turnLeft();
    madison.move();
    madison.turnLeft();
    madison.move();
    Madison.paint("gold");

    // Moves forward two spaces
    madison.move();
    madison.move();

    // Takes paint four times to get all the paint from the bucket
    madison.takePaint();
    madison.takePaint();
    madison.takePaint();
    madison.takePaint();

    // Turns left then moves forward two spaces
    madison.turnLeft();
    madison.move();
    madison.move();

    // Paints the right eye
    madison.paint("gold");

    // Moves forward three spaces
    madison.move();
    madison.move();
    madison.move();

    // Paints the left eye
    madison.paint("gold");

    // Moves forward one space to get off the paint
    madison.move();
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Painter paints 10 times => ")
  public void testPainterPaintsTwelveTimes() {
    int numPaint = primaryPainterLog.actionCount(NeighborhoodActionType.PAINT);
    message = getErrorMessage(numPaint);
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.PAINT, 10), message);
  }


  private String getErrorMessage(int numPaint) {
    if (numPaint == 0) {
      message = "The Painter does not paint any spaces. Try giving the paint() instruction to the Painter.";
    }
    else if (numPaint == 1) {
      message = "The Painter only paints one space. Try giving the paint() instruction to the Painter to paint nine more times.";
    }
    else if (numPaint == 2) {
      message = "The Painter only paints two spaces. Try giving the paint() instruction to the Painter to paint eight more times.";
    }
    else if (numPaint == 3) {
      message = "The Painter only paints three spaces. Try giving the paint() instruction to the Painter to paint seven more times.";
    }
    else if (numPaint == 4) {
      message = "The Painter only paints four spaces. Try giving the paint() instruction to the Painter to paint six more times.";
    }
    else if (numPaint == 5) {
      message = "The Painter only paints five spaces. Try giving the paint() instruction to the Painter to paint five more times.";
    }
    else if (numPaint == 6) {
      message = "The Painter only paints six spaces. Try giving the paint() instruction to the Painter to paint four more times.";
    }
    else if (numPaint == 7) {
      message = "The Painter only paints seven spaces. Try giving the paint() instruction to the Painter to paint three more times.";
    }
    else if (numPaint == 8) {
      message = "The Painter only paints eight spaces. Try giving the paint() instruction to the Painter to paint two more times.";
    }
    else if (numPaint == 9) {
      message = "The Painter only paints nine spaces. Try giving the paint() instruction to the Painter to paint one more time.";
    }

    message += messageGap;
    return message;
  }
}`}],dataFiles:[]},{name:`Predict and Run: Classes and Objects`,lesson:`Lesson 3: Classes and Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- NOTE -----------------------------------
     * This program will run, but you won't see anything happen. The Painter class
     * in this program is a simplified version of the real one you will be using.
     * ----------------------------------------------------------------------------
     */
    
    Painter lisa = new Painter();
    lisa.move();
    
  }
}`},{path:`Painter.java`,text:`/*
 * Represents a painter in The Neighborhood
 */
public class Painter {

  private int xLocation;
  private int yLocation;
  private String direction;
  private int remainingPaint;

  public Painter() {
    /* code not shown */
    System.out.println("> Created Painter object");
  }

  public void move() {
     /* code not shown */
    System.out.println("> Moved Painter one square forward");
  }

  public void turnLeft() {
    /* code not shown */
  }

  public void paint(String color) {
    /* code not shown */
  }

  public void takePaint() {
    /* code not shown */
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The Painter Class`,lesson:`Lesson 3: Classes and Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     *
     * description of method to write
     */

    

    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
    
  }
}`},{path:`Painter.java`,text:`/*
 * Represents a painter in The Neighborhood
 */
public class Painter {

  private int xLocation;
  private int yLocation;
  private String direction;
  private int remainingPaint;

  public Painter() {
    /* code not shown */
    System.out.println("> Created Painter object");
  }

  public void move() {
     /* code not shown */
    System.out.println("> Moved Painter one square forward");
  }

  public void turnLeft() {
    /* code not shown */
  }

  public void paint(String color) {
    /* code not shown */
  }

  public void takePaint() {
    /* code not shown */
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: Painter Objects`,lesson:`Lesson 4: Instantiating Objects`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate the missing Painter object.
     * -----------------------------------------------------------------------------
     */

    

    ezra.turnLeft();
    ezra.turnLeft();
    ezra.turnLeft();

    ezra.move();
    ezra.move();
    ezra.move();
    ezra.move();
    ezra.move();

    ezra.turnLeft();

    ezra.move();
    ezra.move();
    ezra.move();
    ezra.move();
    ezra.move();

    ezra.turnLeft();

    ezra.move();
    ezra.move();
    
  }
}`}],validationFiles:[{path:`NeighborhoodTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class NeighborhoodTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate a Painter object => ")
  public void testInstantiatePainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(primaryPainterLog, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Objects`,lesson:`Lesson 4: Instantiating Objects`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate the missing Painter object.
     * -----------------------------------------------------------------------------
     */

    

    blake.move();
    blake.move();
    blake.move();

    blake.turnLeft();
    blake.turnLeft();
    blake.turnLeft();

    blake.move();
    blake.move();
    blake.move();
    blake.move();
    blake.move();
    blake.move();
    blake.move();

    blake.turnLeft();
    blake.turnLeft();
    blake.turnLeft();

    blake.move();
    blake.move();
    blake.move();
    
  }
}`}],validationFiles:[{path:`NeighborhoodTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class NeighborhoodTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate a Painter object => ")
  public void testInstantiatePainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(primaryPainterLog, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Objects (a)`,lesson:`Lesson 4: Instantiating Objects`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate the missing Painter objects.
     * -----------------------------------------------------------------------------
     */


    
    
    emma.move();
    emma.move();

    emma.turnLeft();
    emma.turnLeft();
    emma.turnLeft();

    emma.move();
    emma.move();
    emma.move();
    emma.move();

    ashley.move();
    ashley.move();
    ashley.move();
    ashley.move();
    ashley.move();

    ashley.turnLeft();
    ashley.turnLeft();
    ashley.turnLeft();

    ashley.move();
    ashley.move();
    ashley.move();
    ashley.move();
    
  }
}`}],validationFiles:[{path:`NeighborhoodTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class NeighborhoodTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate the first missing Painter object => ")
  public void testInstantiateFirstPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("Instantiate the second missing Painter object => ")
  public void testInstantiateSecondPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[1], message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Objects (b)`,lesson:`Lesson 4: Instantiating Objects`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate the missing Painter objects.
     * -----------------------------------------------------------------------------
     */


    

    alex.turnLeft();
    alex.turnLeft();
    alex.turnLeft();

    alex.move();
    alex.move();
    alex.move();
    alex.move();
    alex.move();
    alex.move();
    alex.move();

    alex.turnLeft();

    alex.move();
    alex.move();
    alex.move();
    alex.move();
    alex.move();

    elias.move();
    elias.move();
    elias.move();
    elias.move();
    elias.move();
    elias.move();
    elias.move();

    elias.turnLeft();
    elias.turnLeft();
    elias.turnLeft();

    elias.move();
    elias.move();
    elias.move();
    elias.move();
    elias.move();
    elias.move();
    
  }
}`}],validationFiles:[{path:`NeighborhoodTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class NeighborhoodTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate the first missing Painter object => ")
  public void testInstantiateFirstPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("Instantiate the second missing Painter object => ")
  public void testInstantiateSecondPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[1], message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Objects (c)`,lesson:`Lesson 4: Instantiating Objects`,view:`neighborhood`,grid:`1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0
0,0 1,0 0,0 1,0 0,0 0,0 0,0 1,0
0,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0
0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate the missing Painter objects.
     * -----------------------------------------------------------------------------
     */


    

    jude.move();

    jude.turnLeft();
    jude.turnLeft();
    jude.turnLeft();

    jude.move();
    jude.move();

    jude.turnLeft();

    jude.move();
    jude.move();

    jude.turnLeft();

    jude.move();
    jude.move();

    jude.turnLeft();
    jude.turnLeft();
    jude.turnLeft();

    jude.move();
    jude.move();
    jude.move();
    jude.move();

    jude.turnLeft();
    jude.turnLeft();
    jude.turnLeft();

    jude.move();
    jude.move();
    jude.move();
    jude.move();
    jude.move();
    jude.move();

    jude.turnLeft();
    jude.turnLeft();
    jude.turnLeft();

    jude.move();
    jude.move();

    jude.turnLeft();
    jude.turnLeft();
    jude.turnLeft();

    jude.move();
    jude.move();
    jude.move();

    evan.move();

    evan.turnLeft();
    evan.turnLeft();
    evan.turnLeft();

    evan.move();
    evan.move();

    evan.turnLeft();

    evan.move();
    evan.move();

    evan.turnLeft();

    evan.move();
    evan.move();

    evan.turnLeft();
    evan.turnLeft();
    evan.turnLeft();

    evan.move();
    evan.move();
    evan.move();
    evan.move();

    evan.turnLeft();
    evan.turnLeft();
    evan.turnLeft();

    evan.move();
    evan.move();
    evan.move();
    evan.move();
    evan.move();
    evan.move();

    evan.turnLeft();
    evan.turnLeft();
    evan.turnLeft();

    evan.move();
    evan.move();

    evan.turnLeft();

    evan.move();

    evan.turnLeft();
    evan.turnLeft();
    evan.turnLeft();

    evan.move();
    evan.move();
    evan.move();
    evan.move();
    
  }
}`}],validationFiles:[{path:`NeighborhoodTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class NeighborhoodTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate the first missing Painter object => ")
  public void testInstantiateFirstPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("Instantiate the second missing Painter object => ")
  public void testInstantiateSecondPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[1], message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Objects (d)`,lesson:`Lesson 4: Instantiating Objects`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate the missing Painter objects.
     * -----------------------------------------------------------------------------
     */


    

    leo.move();
    leo.move();
    leo.move();
    leo.move();

    leo.turnLeft();
    leo.turnLeft();
    leo.turnLeft();

    leo.move();
    leo.move();
    leo.move();

    leo.turnLeft();

    leo.move();
    leo.move();

    akira.turnLeft();
    akira.turnLeft();
    akira.turnLeft();

    akira.move();
    akira.move();
    akira.move();
    akira.move();

    akira.turnLeft();

    akira.move();
    akira.move();
    akira.move();

    akira.turnLeft();
    akira.turnLeft();
    akira.turnLeft();

    akira.move();
    akira.move();
    
  }
}`}],validationFiles:[{path:`NeighborhoodTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class NeighborhoodTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate the first missing Painter object => ")
  public void testInstantiateFirstPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("Instantiate the second missing Painter object => ")
  public void testInstantiateSecondPainterObject() {
    message = "Instantiate a Painter object by using the new keyword and calling the constructor.";
    message += "\\n        Be sure to name it the same as the name of the Painter object in the program.";
    message += messageGap;
      
    assertNotNull(painterLogs[1], message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Painter Class Methods`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object
    Painter silas = new Painter();

    // Moves forward three spaces
    silas.move();
    silas.move();
    silas.move();

    // Turns right by turning left three times
    silas.turnLeft();
    silas.turnLeft();
    silas.turnLeft();

    // Moves forward two spaces
    silas.move();
    silas.move();

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Painter Class Methods #1`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object
    Painter silas = new Painter();

    // Moves forward three spaces
    silas.move();
    silas.move();
    silas.move();

    // Turns right by turning left three times
    silas.turnLeft();
    silas.turnLeft();
    silas.turnLeft();

    // Moves forward two spaces
    silas.move();
    silas.move();

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Painter Class Methods #2`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object
    Painter silas = new Painter();

    // Moves forward three spaces
    silas.move();
    silas.move();
    silas.move();

    // Print direction before turning
    System.out.println("Before turning: " + silas.getDirection());

    // Turns right by turning left three times
    silas.turnLeft();
    silas.turnLeft();
    silas.turnLeft();

    // Print direction after turning
    System.out.println("After turning: " + silas.getDirection());

    // Moves forward two spaces
    silas.move();
    silas.move();

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */
    // Use print statements to show direction before and after each turn you write


    
    
  }
}
`}],validationFiles:[],dataFiles:[]},{name:`Practice: Painter Class Methods (a)`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate through The Neighborhood to reach the food truck.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigates to x location 4 to reach the food truck => ")
  public void testNavigateToXLocation() {
    message = "Use the move() method to move the Painter object forward one space.";
    message += messageGap;
      
    int expected = 4;
    int actual = primaryPainterLog.getEndingPosition().getX();

    assertEquals(expected, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to y location 2 to reach the food truck => ")
  public void testNavigateToYLocation() {
    message = "Use the turnLeft() method to turn the Painter object, then use ";
    message += "\\n        the move() method to move the Painter object forward.";
    message += messageGap;
      
    int expected = 2;
    int actual = primaryPainterLog.getEndingPosition().getY();

    assertEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Class Methods (b)`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate through The Neighborhood to reach the traffic cone.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigates to x location 5 to reach the traffic cone => ")
  public void testNavigateToXLocation() {
    message = "Use the move() method to move the Painter object forward one space.";
    message += messageGap;
      
    int expected = 5;
    int actual = primaryPainterLog.getEndingPosition().getX();

    assertEquals(expected, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to y location 5 to reach the traffic cone => ")
  public void testNavigateToYLocation() {
    message = "Use the turnLeft() method to turn the Painter object, then use ";
    message += "\\n        the move() method to move the Painter object forward.";
    message += messageGap;
      
    int expected = 5;
    int actual = primaryPainterLog.getEndingPosition().getY();

    assertEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Class Methods (c)`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate through The Neighborhood to reach the house.
     * -----------------------------------------------------------------------------
     */
    
    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigates to x location 5 to reach the traffic cone => ")
  public void testNavigateToXLocation() {
    message = "Use the move() method to move the Painter object forward one space.";
    message += messageGap;
      
    int expected = 5;
    int actual = primaryPainterLog.getEndingPosition().getX();

    assertEquals(expected, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to y location 5 to reach the traffic cone => ")
  public void testNavigateToYLocation() {
    message = "Use the turnLeft() method to turn the Painter object, then use ";
    message += "\\n        the move() method to move the Painter object forward.";
    message += messageGap;
      
    int expected = 5;
    int actual = primaryPainterLog.getEndingPosition().getY();

    assertEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Painter Class Methods (d)`,lesson:`Lesson 5: Methods`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate through The Neighborhood to reach the traffic cone.
     * -----------------------------------------------------------------------------
     */
    
    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigates to x location 3 to reach the traffic cone => ")
  public void testNavigateToXLocation() {
    message = "Use the move() method to move the Painter object forward one space.";
    message += messageGap;
      
    int expected = 3;
    int actual = primaryPainterLog.getEndingPosition().getX();

    assertEquals(expected, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to y location 9 to reach the traffic cone => ")
  public void testNavigateToYLocation() {
    message = "Use the turnLeft() method to turn the Painter object, then use ";
    message += "\\n        the move() method to move the Painter object forward.";
    message += messageGap;
      
    int expected = 9;
    int actual = primaryPainterLog.getEndingPosition().getY();

    assertEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Painting`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object
    Painter ella = new Painter();

    // Moves forward one space
    ella.move();

    // Takes paint three times to get all the paint from the bucket
    ella.takePaint();
    ella.takePaint();
    ella.takePaint();

    // Moves and paints three times
    ella.paint("white");
    ella.move();
    ella.paint("white");
    ella.move();
    ella.paint("white");

    // Turns right by turning left three times
    ella.turnLeft();
    ella.turnLeft();
    ella.turnLeft();

    // Moves forward one space
    ella.move();

    // Takes paint three times to get all the paint from the bucket
    ella.takePaint();
    ella.takePaint();
    ella.takePaint();

    // Moves and paints three times
    ella.paint("white");
    ella.move();
    ella.paint("white");
    ella.move();
    ella.paint("white");

    // Moves forward one space then takes paint three times to get all the paint from the bucket
    ella.move();

    /* ---- 🔎 ADD YOUR CODE HERE TO TAKE PAINT FROM THE PAINT BUCKET ---- */

    

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Painting #1`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     *
     * description of method to write
     */

    

    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Painting #2`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     *
     * description of method to write
     */

    

    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Painting #3`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 1,3 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,3 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,3 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0
0,0 0,0 0,0 1,3 1,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ instructions for the student to complete
     * -----------------------------------------------------------------------------
     *
     * description of method to write
     */

    

    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Moving and Painting #1`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,5 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate and paint the path to the traffic cone.
     * -----------------------------------------------------------------------------
     */
    



    

    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Take paint five times to get all the paint from the paint bucket => ")
  public void testTakePaintFiveTimes() {
    int takePaintCount = primaryPainterLog.actionCount(NeighborhoodActionType.TAKE_PAINT);
    
    message = getTakePaintErrorMessage(takePaintCount, 5);
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 5), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigate to x location 3 to reach the traffic cone => ")
  public void testNavigateToTrafficConeXLocation() {
    int xPosition = primaryPainterLog.getEndingPosition().getX();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected x location (3).";    
    message += messageGap;
      
    assertEquals(3, xPosition, message);
  }

  @Test
  @Order(3)
  @DisplayName("Navigate to y location 2 to reach the traffic cone => ")
  public void testNavigateToTrafficConeYLocation() {
    int yPosition = primaryPainterLog.getEndingPosition().getY();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected y location (2).";    
    message += messageGap;
    
    assertEquals(2, yPosition, message);
  }

  @Test
  @Order(4)
  @DisplayName("Paint the path to the traffic cone => ")
  public void testPaintPathToTrafficCone() {
    boolean[][] expected = new boolean[8][8];
    expected[0][1] = true;
    expected[0][2] = true;
    expected[1][2] = true;
    expected[2][2] = true;
    expected[3][2] = true;

    if (primaryPainterLog.actionCount(NeighborhoodActionType.PAINT) == 0) {
      message = "Use the paint() method to paint each space as the Painter moves.";
    }
    else {
      message = "One or more spaces were not painted. Use the paint() method to paint each space as the Painter moves.";
    }

    message += messageGap;
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }

  private String getTakePaintErrorMessage(int takePaintCount, int expectedTakePaintCount) {
    String result = "The Painter only took ";

    if (takePaintCount == 1) {
      result += takePaintCount + " unit of paint.";
    }
    else {
      result += takePaintCount + " units of paint.";
    }

    int remainingTakePaintCount = expectedTakePaintCount - takePaintCount;
    result += messageGap + " Call the takePaint() method " + remainingTakePaintCount + " more times while the Painter object is on a paint bucket to get all the paint.";

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Moving and Painting #2`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate and paint the path to the traffic cone.
     * -----------------------------------------------------------------------------
     */
    

    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Painter paints exactly 7 spaces => ")
  public void testPaintedExactlySixTiles() {
    int paintActions = primaryPainterLog.actionCount(NeighborhoodActionType.PAINT);
  
    message = "Use the paint() method to paint exactly 7 spaces along the path.";
    message += messageGap;
  
    assertEquals(7, paintActions, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigate to x location 4 => ")
  public void testNavigateToTrafficConeXLocation() {
    int xPosition = primaryPainterLog.getEndingPosition().getX();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected x location (4).";    
    message += messageGap;
      
    assertEquals(4, xPosition, message);
  }

  @Test
  @Order(3)
  @DisplayName("Navigate to y location 2 => ")
  public void testNavigateToTrafficConeYLocation() {
    int yPosition = primaryPainterLog.getEndingPosition().getY();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected y location (2).";    
    message += messageGap;
    
    assertEquals(2, yPosition, message);
  }

  @Test
  @Order(4)
  @DisplayName("Paint the path to the traffic cone => ")
  public void testPaintPathToTrafficCone() {
    boolean[][] expected = new boolean[8][8];
    expected[0][0] = true;
    expected[1][0] = true;
    expected[2][0] = true;
    expected[3][0] = true;
    expected[3][1] = true;
    expected[3][2] = true;
    expected[4][2] = true;

    if (primaryPainterLog.actionCount(NeighborhoodActionType.PAINT) == 0) {
      message = "Use the paint() method to paint each space as the Painter moves.";
    }
    else {
      message = "One or more spaces were not painted. Use the paint() method to paint each space as the Painter moves.";
    }

    message += messageGap;
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
}
`}],dataFiles:[]},{name:`Practice: Moving and Painting #3`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,3 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,4 1,0 1,0 1,0 1,3 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate and paint the path to the traffic cone.
     * -----------------------------------------------------------------------------
     */
    



    


    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Take paint ten times to get all the paint from the paint bucket => ")
  public void testTakePaintTenTimes() {
    int takePaintCount = primaryPainterLog.actionCount(NeighborhoodActionType.TAKE_PAINT);
    
    message = getTakePaintErrorMessage(takePaintCount, 10);
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 10), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigate to x location 4 to reach the traffic cone => ")
  public void testNavigateToTrafficConeXLocation() {
    int xPosition = primaryPainterLog.getEndingPosition().getX();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected x location (4).";    
    message += messageGap;
      
    assertEquals(4, xPosition, message);
  }

  @Test
  @Order(3)
  @DisplayName("Navigate to y location 3 to reach the traffic cone => ")
  public void testNavigateToTrafficConeYLocation() {
    int yPosition = primaryPainterLog.getEndingPosition().getY();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected y location (3).";    
    message += messageGap;
    
    assertEquals(3, yPosition, message);
  }

  @Test
  @Order(4)
  @DisplayName("Paint the path to the traffic cone => ")
  public void testPaintPathToTrafficCone() {
    boolean[][] expected = new boolean[8][8];
    expected[0][2] = true;
    expected[0][3] = true;
    expected[0][4] = true;
    expected[0][5] = true;
    expected[1][5] = true;
    expected[2][5] = true;
    expected[3][5] = true;
    expected[4][5] = true;
    expected[4][4] = true;
    expected[4][3] = true;

    if (primaryPainterLog.actionCount(NeighborhoodActionType.PAINT) == 0) {
      message = "Use the paint() method to paint each space as the Painter moves.";
    }
    else {
      message = "One or more spaces were not painted. Use the paint() method to paint each space as the Painter moves.";
    }

    message += messageGap;
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }

  private String getTakePaintErrorMessage(int takePaintCount, int expectedTakePaintCount) {
    String result = "The Painter only took ";

    if (takePaintCount == 1) {
      result += takePaintCount + " unit of paint.";
    }
    else {
      result += takePaintCount + " units of paint.";
    }

    int remainingTakePaintCount = expectedTakePaintCount - takePaintCount;
    result += messageGap + " Call the takePaint() method " + remainingTakePaintCount + " more times while the Painter object is on a paint bucket to get all the paint.";

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Moving and Painting #4`,lesson:`Lesson 6: Methods with Parameters`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,4 0,0 0,0 0,0 1,0 1,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,4 1,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,4
1,4 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 1,4 1,0 1,0 1,0 1,4 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painter object, then use the methods in the Painter class
     * to navigate and paint the path to the traffic cone.
     * -----------------------------------------------------------------------------
     */



    


    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Take paint 24 times to get all the paint from the paint bucket => ")
  public void testTakePaintTwentyFourTimes() {
    int takePaintCount = primaryPainterLog.actionCount(NeighborhoodActionType.TAKE_PAINT);
    
    message = getTakePaintErrorMessage(takePaintCount, 24);
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 24), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigate to x location 5 to reach the traffic cone => ")
  public void testNavigateToTrafficConeXLocation() {
    int xPosition = primaryPainterLog.getEndingPosition().getX();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected x location (5).";    
    message += messageGap;
      
    assertEquals(5, xPosition, message);
  }

  @Test
  @Order(3)
  @DisplayName("Navigate to y location 1 to reach the traffic cone => ")
  public void testNavigateToTrafficConeYLocation() {
    int yPosition = primaryPainterLog.getEndingPosition().getY();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected y location (1).";    
    message += messageGap;
    
    assertEquals(1, yPosition, message);
  }

  @Test
  @Order(4)
  @DisplayName("Paint the path to the traffic cone => ")
  public void testPaintPathToTrafficCone() {
    boolean[][] expected = new boolean[8][8];
    expected[0][1] = true;
    expected[0][2] = true;
    expected[0][3] = true;
    expected[0][4] = true;
    expected[0][5] = true;
    expected[0][6] = true;
    expected[0][7] = true;
    expected[1][7] = true;
    expected[2][7] = true;
    expected[3][7] = true;
    expected[4][7] = true;
    expected[5][7] = true;
    expected[6][7] = true;
    expected[7][7] = true;
    expected[7][6] = true;
    expected[7][5] = true;
    expected[7][4] = true;
    expected[7][3] = true;
    expected[6][3] = true;
    expected[5][3] = true;
    expected[4][3] = true;
    expected[4][2] = true;
    expected[4][1] = true;
    expected[5][1] = true;

    if (primaryPainterLog.actionCount(NeighborhoodActionType.PAINT) == 0) {
      message = "Use the paint() method to paint each space as the Painter moves.";
    }
    else {
      message = "One or more spaces were not painted. Use the paint() method to paint each space as the Painter moves.";
    }

    message += messageGap;
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }

  private String getTakePaintErrorMessage(int takePaintCount, int expectedTakePaintCount) {
    String result = "The Painter only took ";

    if (takePaintCount == 1) {
      result += takePaintCount + " unit of paint.";
    }
    else {
      result += takePaintCount + " units of paint.";
    }

    int remainingTakePaintCount = expectedTakePaintCount - takePaintCount;
    result += messageGap + " Call the takePaint() method " + remainingTakePaintCount + " more times while the Painter object is on a paint bucket to get all the paint.";

    return result;
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Loops and Boolean Methods`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object called jesse
    Painter jesse = new Painter();

    // Moves forward one space
    jesse.move();

    while (jesse.isOnBucket()) {
      jesse.takePaint();
    }

    while (jesse.hasPaint()) {
      jesse.move();
      jesse.paint("pink");
    }

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    // Moves forward one space
    jesse.move();

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    while (jesse.canMove()) {
      jesse.move();
    }
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Loops and Boolean Methods #1`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object called jesse
    Painter jesse = new Painter();

    // Moves forward one space
    jesse.move();

    while (jesse.isOnBucket()) {
      jesse.takePaint();
    }

    while (jesse.hasPaint()) {
      jesse.move();
      jesse.paint("pink");
    }

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    // Moves forward one space
    jesse.move();

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    while (jesse.canMove()) {
      jesse.move();
    }
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Loops and Boolean Methods #2`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object called jesse
    Painter jesse = new Painter();

    // Moves forward one space
    jesse.move();

    while (jesse.isOnBucket()) {
      jesse.takePaint();
    }

    while (jesse.hasPaint()) {
      jesse.move();
      jesse.paint("pink");
    }

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    // Moves forward one space
    jesse.move();

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    while (jesse.canMove()) {
      jesse.move();
    }
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Loops and Boolean Methods #3`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object called jesse
    Painter jesse = new Painter();

    // Moves forward one space
    jesse.move();

    while (jesse.isOnBucket()) {
      jesse.takePaint();
    }

    while (jesse.hasPaint()) {
      jesse.move();
      jesse.paint("pink");
    }

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    // Moves forward one space
    jesse.move();

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    while (jesse.canMove()) {
      jesse.move();
    }
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Loops and Boolean Methods #4`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,6 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Painter object called jesse
    Painter jesse = new Painter();

    // Moves forward one space
    jesse.move();

    while (jesse.isOnBucket()) {
      jesse.takePaint();
    }

    while (jesse.hasPaint()) {
      jesse.move();
      jesse.paint("pink");
    }

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    // Moves forward one space
    jesse.move();

    // Turns right by turning left three times
    jesse.turnLeft();
    jesse.turnLeft();
    jesse.turnLeft();

    // Print whether jesse has paint before entering the loop
    System.out.println("Has paint? " + jesse.hasPaint());

    while (jesse.canMove()) {
      System.out.println("Still in loop, current direction: " + jesse.getDirection());
      jesse.move();
    }
    System.out.println("Exited loop.");
  }
}
`}],validationFiles:[],dataFiles:[]},{name:`Practice: Using Loops and Boolean Methods (a)`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement your algorithm to have a Painter object take all of the paint
     * from a paint bucket.
     * -----------------------------------------------------------------------------
     */
    



    

    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Check if the Painter object is on a paint bucket => ")
  public void testPainterIsOnBucket() {
    message = "Call the isOnBucket() method as the while loop condition to check if the Painter object is on a paint bucket.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.IS_ON_BUCKET, 6), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Take all of the paint from the paint bucket => ")
  public void testPainterTakeAllPaint() {
    int takePaintCount = primaryPainterLog.actionCount(NeighborhoodActionType.TAKE_PAINT);

    if (takePaintCount < 5) {
      message = "Call the takePaint() method in the body of the while loop to take the paint from the paint bucket.";
    }
    else {
      message = "The Painter may have tried to take more paint than what is available in the paint bucket.";
    }
    
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 5), message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Using Loops and Boolean Methods (b)`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement your algorithm to have a Painter object move forward one
     * space as long as it can move in the direction it is facing.
     * -----------------------------------------------------------------------------
     */
    





    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Check if the Painter object can move forward in the direction it is facing => ")
  public void testPainterCanMove() {
    message = "Call the canMove() method as the while loop condition to check if the Painter object can move forward.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.CAN_MOVE, 3), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Move forward as long as there are no obstacles => ")
  public void testPainterMovesForward() {
    int moveCount = primaryPainterLog.actionCount(NeighborhoodActionType.MOVE);

    if (moveCount < 2) {
      message = "Call the move() method in the body of the while loop to move forward.";
    }
    else {
      message = "The Painter may have tried to move when it could not move forward.";
    }
    
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.MOVE, 2), message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Using Loops and Boolean Methods (c)`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,3 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement your algorithm to have a Painter object paint the current
     * space and move forward one space until it runs out of paint.
     * -----------------------------------------------------------------------------
     */

    




    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Paints while the Painter object has paint => ")
  public void testPainterPaintWhileHasPaint() {
    message = "Call the hasPaint() method as the while loop condition to check if the Painter object has paint.";
    message += "\\n        Call the paint() method in the body of the while loop to paint while hasPaint() is true.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.PAINT, 3), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Move forward while the Painter object has paint => ")
  public void testPainterMovesForward() {
    message = "Call the hasPaint() method as the while loop condition to check if the Painter object has paint.";
    message += "\\n        Call the move() method in the body of the while loop to move forward while hasPaint() is true.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.MOVE, 3), message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Using Loops and Boolean Methods (d)`,lesson:`Lesson 7: Loops`,view:`neighborhood`,grid:`1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement your algorithm to have a Painter object move forward, turn,
     * and paint in a square until it runs out of paint.
     * -----------------------------------------------------------------------------
     */
    





    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Move forward while the Painter object has paint => ")
  public void testPainterMovesForward() {
    message = "Call the hasPaint() method as the while loop condition to check if the Painter object has paint.";
    message += "\\n        Call the move() method in the body of the while loop to move forward while hasPaint() is true.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.MOVE, 4), message);
  }

  @Test
  @Order(2)
  @DisplayName("Turn left three times to turn right => ")
  public void testPainterTurnsRight() {
    message = "Call the hasPaint() method as the while loop condition to check if the Painter object has paint.";
    message += "\\n        Call the turnLeft() method in the body of the while loop to turn left 3x while hasPaint() is true.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionAtLeast(NeighborhoodActionType.TURN_LEFT, 12), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Paint while the Painter object has paint => ")
  public void testPainterPaintWhileHasPaint() {
    message = "Call the hasPaint() method as the while loop condition to check if the Painter object has paint.";
    message += "\\n        Call the paint() method in the body of the while loop to paint while hasPaint() is true.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.PAINT, 4), message);
  }
   
}`}],dataFiles:[]},{name:`Investigating AI-Generated Code #1`,lesson:`Lesson 8: AI-Generated Code`,view:`neighborhood`,grid:`1,100 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    FlagPainter flag = new FlagPainter();
    flag.takeAllPaint();
    flag.paintFlagOfBelgium();
  }
}`},{path:`FlagPainter.java`,text:`import org.code.neighborhood.*;

public class FlagPainter extends PainterPlus {
  
  /*
   * Paints the entire flag of Belgium
   */
  public void paintFlagOfBelgium() {
    paintStripe("black");
    moveToNextStripe();
    paintStripe("yellow");
    moveToNextStripe();
    paintStripe("red");
    resetPosition();
  }

  /*
   * Paints a vertical stripe of the flag
   */
  public void paintStripe(String color) {
    for (int i = 0; i < 8; i++) {
      paint(color);
      if (canMove()) {
        move();
      }
    }
    turnAround();
    for (int i = 0; i < 7; i++) {
      move();
    }
    turnLeft();
  }

  /*
   * Moves to the starting position for the next stripe
   */
  public void moveToNextStripe() {
    move();
    turnRight();
  }

  /*
   * Resets the Painter to the starting position
   */
  public void resetPosition() {
    turnLeft();
    while (canMove()) {
      move();
    }
    turnLeft();
    while (canMove()) {
      move();
    }
    turnAround();
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Turns around
  public void turnAround() {
    turnLeft();
    turnLeft();
  }

  // Paints a line based on how much paint the Painter has
  public void paintLine(int paintAmount, String color) {
    setPaint(paintAmount);

    while (hasPaint()) {
      paint(color);
      
      if (canMove()) {
        move();
      }
    }
  }

  // Moves to the next row
  public void moveToNextRow() {
    if (canMove("south")) {
      if (isFacingEast()) {
        turnRight();
        move();
        turnRight();
      }
      else {
        turnLeft();
        move();
        turnLeft();
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigating AI-Generated Code #2`,lesson:`Lesson 8: AI-Generated Code`,view:`neighborhood`,grid:`1,100 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    FlagPainter flag = new FlagPainter();
    flag.takeAllPaint();
    flag.paintFlagOfBelgium();
  }
}`},{path:`FlagPainter.java`,text:`import org.code.neighborhood.*;

public class FlagPainter extends PainterPlus {

  /*
   * Paints the entire flag of Belgium
   */
  public void paintFlagOfBelgium() {
    /* ---- 🔎 This is where the student implemented AI Tutor's suggestion ---- */
    turnRight();
    
    paintStripe("black");
    moveToNextStripe();
    paintStripe("yellow");
    moveToNextStripe();
    paintStripe("red");
    resetPosition();
  }

  /*
   * Paints a vertical stripe of the flag
   */
  public void paintStripe(String color) {
    for (int i = 0; i < 8; i++) {
      paint(color);
      if (canMove()) {
        move();
      }
    }
    turnAround();
    for (int i = 0; i < 7; i++) {
      move();
    }
    turnLeft();
  }

  /*
   * Moves to the starting position for the next stripe
   */
  public void moveToNextStripe() {
    move();
    turnRight();
  }

  /*
   * Resets the Painter to the starting position
   */
  public void resetPosition() {
    turnLeft();
    while (canMove()) {
      move();
    }
    turnLeft();
    while (canMove()) {
      move();
    }
    turnAround();
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Turns around
  public void turnAround() {
    turnLeft();
    turnLeft();
  }

  // Paints a line based on how much paint the Painter has
  public void paintLine(int paintAmount, String color) {
    setPaint(paintAmount);

    while (hasPaint()) {
      paint(color);
      
      if (canMove()) {
        move();
      }
    }
  }

  // Moves to the next row
  public void moveToNextRow() {
    if (canMove("south")) {
      if (isFacingEast()) {
        turnRight();
        move();
        turnRight();
      }
      else {
        turnLeft();
        move();
        turnLeft();
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Creating PainterPlus #1`,lesson:`Lesson 9: Inheritance`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /*
     * ------------------------------ TO DO ------------------------------
     * Write the PainterPlus class, then instantiate a PainterPlus object.
     * -------------------------------------------------------------------
     */

    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PainterPlus.java Test")
public class PainterPlusTest {

  String message;
  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("Create a PainterPlus class => ")
  public void testCreatePainterPlusClass() {
    message = "Create a new file named PainterPlus.java and write the class header. Remember that the ";
    message += "\\n        class header contains the keyword \\"class\\" and the name of the class.";
    message += messageGap;

    List<String> classNames = ValidationHelper.getClassNames();
      
    assertTrue(classNames.contains("PainterPlus"), message);
  }
   
  @Test
  @Order(2)
  @SuppressWarnings("unchecked")
  @DisplayName("PainterPlus is a subclass of the Painter class => ")
  public void testPainterPlusIsSubclassOfPainter() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the PainterPlus class is a subclass of the Painter class.";
    message += messageGap;
      
    Painter testPainter = new Painter();
    Class painterPlusClass = null;

    try {
      painterPlusClass = Class.forName("PainterPlus");
    } catch (Exception e) {
      message = "Your class header for PainterPlus is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterClass = testPainter.getClass();
    
    assertTrue(painterClass.isAssignableFrom(painterPlusClass), message);
  }

  @Test
  @Order(3)
  @DisplayName("Instantiate a PainterPlus object => ")
  public void testInstantiatePainterPlusObject() {
    message = "Instantiate a PainterPlus object in NeighborhoodRunner.java.";
    message += messageGap;
      
    PainterLog primaryPainterLog = getPrimaryPainterLog();
    assertNotNull(primaryPainterLog, message);
  }
   
  private PainterLog getPrimaryPainterLog() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;

    PainterLog temp = null;
    
    try {
      NeighborhoodLog neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      temp = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating PainterPlus #2`,lesson:`Lesson 9: Inheritance`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigate to x location 3 to reach the traffic cone => ")
  public void testNavigateToTrafficConeXLocation() {
    int xPosition = primaryPainterLog.getEndingPosition().getX();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected x location (3).";    
    message += messageGap;
      
    assertEquals(3, xPosition, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigate to y location 2 to reach the traffic cone => ")
  public void testNavigateToTrafficConeYLocation() {
    int yPosition = primaryPainterLog.getEndingPosition().getY();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected y location (2).";    
    message += messageGap;
    
    assertEquals(2, yPosition, message);
  }
   
}`}],dataFiles:[]},{name:`Predict and Run: Writing Methods`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus olivia = new PainterPlus();

    olivia.move();

    while (olivia.isOnBucket()) {
      olivia.takePaint();
    }

    olivia.paintSquare();
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Paints a square
  public void paintSquare() {
    while (hasPaint()) {
      move();
      paint("blue");
      turnLeft();
      turnLeft();
      turnLeft();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Void Methods #1`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus olivia = new PainterPlus();

    olivia.move();

    while (olivia.isOnBucket()) {
      olivia.takePaint();
    }

    olivia.paintSquare();
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Paints a square
  public void paintSquare() {
    while (hasPaint()) {
      move();
      paint("blue");
      turnLeft();
      turnLeft();
      turnLeft();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Void Methods #2`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus olivia = new PainterPlus();

    olivia.move();

    while (olivia.isOnBucket()) {
      olivia.takePaint();
    }

    olivia.paintSquare();
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Paints a square
  public void paintSquare() {
    while (hasPaint()) {
      move();
      paint("blue");
      turnLeft();
      turnLeft();
      turnLeft();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Void Methods #3`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,4 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus olivia = new PainterPlus();

    olivia.move();

    while (olivia.isOnBucket()) {
      olivia.takePaint();
    }

    olivia.paintSquare();
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Paints a square
  public void paintSquare() {
    while (hasPaint()) {
      move();
      paint("blue");
      turnLeft();
      turnLeft();
      turnLeft();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Writing Methods`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the PainterPlus class, then instantiate a PainterPlus object.
     * -----------------------------------------------------------------------------
     */





    

    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PainterPlus.java Test")
public class PainterPlusTest {

  NeighborhoodLog neighborhood;
  PainterLog primaryPainterLog;
  PainterPlus testPainterPlus;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPainterPlus = partialMockBuilder(PainterPlus.class)
      .addMockedMethod("turnLeft")
      .createMock();
  }

  @Test
  @Order(1)
  @DisplayName("turnRight() turns a PainterPlus object to the right => ")
  public void testTurnRight() {
    testPainterPlus.turnLeft();
    testPainterPlus.turnLeft();
    testPainterPlus.turnLeft();
    
    replay(testPainterPlus);

    testPainterPlus.turnRight();

    verify(testPainterPlus);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigate to x location 3 to reach the traffic cone => ")
  public void testNavigateToTrafficConeXLocation() {
    getNeighborhoodResults();
    int xPosition = primaryPainterLog.getEndingPosition().getX();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected x location (3).";    
    message += messageGap;
      
    assertEquals(3, xPosition, message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Navigate to y location 2 to reach the traffic cone => ")
  public void testNavigateToTrafficConeYLocation() {
    getNeighborhoodResults();
    int yPosition = primaryPainterLog.getEndingPosition().getY();

    message = "Call the turnLeft() and move() methods to navigate the Painter object to the expected y location (2).";    
    message += messageGap;
    
    assertEquals(2, yPosition, message);
  }
   
  private void getNeighborhoodResults() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Methods (a)`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,0 1,6 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the PainterPlus class, then instantiate a PainterPlus object.
     * -----------------------------------------------------------------------------
     */





    

    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PainterPlus.java Test")
public class PainterPlusTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  PainterPlus testPainterPlus;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @BeforeEach
  public void setupPainterPlus() {
    testPainterPlus = partialMockBuilder(PainterPlus.class)
      .addMockedMethod("isOnBucket")
      .addMockedMethod("takePaint")
      .createMock();
  }
   
  @Test
  @Order(1)
  @DisplayName("takeAllPaint() takes all paint from a paint bucket => ")
  public void testTakeAllPaint() {
    expect(testPainterPlus.isOnBucket()).andReturn(true).times(3);
    testPainterPlus.takePaint();
    testPainterPlus.takePaint();
    testPainterPlus.takePaint();
    expect(testPainterPlus.isOnBucket()).andReturn(false).times(1);
    
    replay(testPainterPlus);
    
    testPainterPlus.takeAllPaint();
    
    verify(testPainterPlus);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paints a path to the house => ")
  public void testPaintPathToHouse() {
    message = "Call the takeAllPaint() method to take all of the paint from the paint bucket.";
    message += "\\n        Use the move() and paint() methods to paint the path to the house.";
    message += messageGap;
      
    boolean[][] expected = new boolean[8][8];
    
    expected[2][0] = true;
    expected[2][1] = true;
    expected[2][2] = true;
    expected[2][3] = true;
    expected[3][3] = true;
    expected[4][3] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Methods (b)`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the PainterPlus class, then instantiate a PainterPlus object.
     * -----------------------------------------------------------------------------
     */





    

    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PainterPlus.java Test")
public class PainterPlusTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  PainterPlus testPainterPlus;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @BeforeEach
  public void setupPainterPlus() {
    testPainterPlus = partialMockBuilder(PainterPlus.class)
      .addMockedMethod("canMove", new Class[]{})
      .addMockedMethod("move")
      .createMock();
  }
   
  @Test
  @Order(1)
  @DisplayName("moveFast() moves forward as long as there are no obstacles => ")
  public void testMoveFast() {
    expect(testPainterPlus.canMove()).andReturn(true).times(3);
    testPainterPlus.move();
    testPainterPlus.move();
    testPainterPlus.move();
    expect(testPainterPlus.canMove()).andReturn(false).times(1);
    
    replay(testPainterPlus);
    
    testPainterPlus.moveFast();
    
    verify(testPainterPlus);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to x location 7 to reach the traffic cone => ")
  public void testNavigateToXLocation() {
    int xLocation = primaryPainterLog.getEndingPosition().getX();
    
    message = "Call the moveFast() method to navigate through The Neighborhood.";
    message += "\\n        Use the turnLeft() and turnRight() methods to turn where needed.";
    message += messageGap;
      
    assertEquals(7, xLocation, message);
  }

  @Test
  @Order(3)
  @DisplayName("Navigates to y location 6 to reach the traffic cone => ")
  public void testNavigateToYLocation() {
    int yLocation = primaryPainterLog.getEndingPosition().getY();
    
    message = "Call the moveFast() method to navigate through The Neighborhood.";
    message += "\\n        Use the turnLeft() and turnRight() methods to turn where needed.";
    message += messageGap;
      
    assertEquals(6, yLocation, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Methods (c)`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,4 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PainterPlus.java Test")
public class PainterPlusTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  PainterPlus testPainterPlus;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @BeforeEach
  public void setupPainterPlus() {
    testPainterPlus = partialMockBuilder(PainterPlus.class)
      .addMockedMethod("hasPaint")
      .addMockedMethod("paint")
      .addMockedMethod("move")
      .createMock();
  }
   
  @Test
  @Order(1)
  @DisplayName("paintToEmpty() paints then moves forward while a PainterPlus object has paint => ")
  public void testPaintToEmpty() {
    expect(testPainterPlus.hasPaint()).andReturn(true).times(3);
    testPainterPlus.paint("white");
    testPainterPlus.move();
    testPainterPlus.paint("white");
    testPainterPlus.move();
    testPainterPlus.paint("white");
    testPainterPlus.move();
    expect(testPainterPlus.hasPaint()).andReturn(false).times(1);
    
    replay(testPainterPlus);
    
    testPainterPlus.paintToEmpty("white");
    
    verify(testPainterPlus);
  }
   
  @Test
  @Order(2)
  @DisplayName("Takes all paint from the paint bucket => ")
  public void testTakePaintFromBucket() {   
    message = "Use the takePaint() method to take paint from the paint bucket.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 4), message);
  }

  @Test
  @Order(3)
  @DisplayName("Paints a line next to the food truck => ")
  public void testPaintNextToFoodTruck() {   
    message = "Call the paintToEmpty() method to paint and move while the PainterPlus object has paint.";
    message += messageGap;

    boolean[][] expected = new boolean[8][8];

    expected[1][0] = true;
    expected[1][1] = true;
    expected[1][2] = true;
    expected[1][3] = true;
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Methods (d)`,lesson:`Lesson 10: Writing Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,8 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the PainterPlus class, then instantiate a PainterPlus object.
     * -----------------------------------------------------------------------------
     */





    

    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PainterPlus.java Test")
public class PainterPlusTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  PainterPlus testPainterPlus;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @BeforeEach
  public void setupPainter() {
    testPainterPlus = partialMockBuilder(PainterPlus.class)
      .addMockedMethod("hasPaint")
      .addMockedMethod("paint")
      .addMockedMethod("move")
      .addMockedMethod("turnRight")
      .createMock();
  }
   
  @Test
  @Order(1)
  @DisplayName("paintDonut() moves, turns, and paints in a donut shape until PainterPlus object runs out of paint => ")
  public void testPaintDonut() {
    expect(testPainterPlus.hasPaint()).andReturn(true).times(1);
    testPainterPlus.move();
    testPainterPlus.turnRight();
    testPainterPlus.paint("white");
    testPainterPlus.move();
    testPainterPlus.paint("white");
    expect(testPainterPlus.hasPaint()).andReturn(false).times(1);
    
    replay(testPainterPlus);
    
    testPainterPlus.paintDonut("white");
    
    verify(testPainterPlus);
  }
   
  @Test
  @Order(2)
  @DisplayName("Takes all paint from the paint bucket => ")
  public void testTakePaintFromBucket() {
    message = "Use the takePaint() method to take paint from the paint bucket.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 8), message);
  }

  @Test
  @Order(3)
  @DisplayName("Paints a donut shape in The Neighborhood => ")
  public void testPaintDonutShape() {
    message = "Call the paintDonut() method to paint and move while the PainterPlus object has paint.";
    message += messageGap;

    boolean[][] expected = new boolean[8][8];

    expected[3][0] = true;
    expected[4][0] = true;
    expected[5][0] = true;
    expected[3][1] = true;
    expected[5][1] = true;
    expected[3][2] = true;
    expected[4][2] = true;
    expected[5][2] = true;
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Investigate: Programming Style and Tools #1`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.Painter;

public class Main {
public static void main(String[] args) {
PainterPlus p = new PainterPlus();
p.turnRight();
p.move();
p.move();
p.turnLeft();
p.move();
p.move();
p.turnRight();
p.move();
p.turnLeft();
p.move();
p.move();
p.move();
}
}
`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.Painter;

public class PainterPlus extends Painter {
public void turnRight() {
turnLeft();
turnLeft();
turnLeft();
} 
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate: Programming Style and Tools #2`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.Painter;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus myPainter = new PainterPlus();

    /*
     * Moves south to the sidewalk
     */
    myPainter.turnRight(); // Turns right to face south
    myPainter.move();
    myPainter.move();

    /*
     * Moves east to the sidewalk
     */
    myPainter.turnLeft(); // Turns left to face east
    myPainter.move();
    myPainter.move();

    /*
     * Moves south to the sidewalk
     */
    myPainter.turnRight(); // Turns right to face south
    myPainter.move();

    /*
     * Moves east to the food truck
     */
    myPainter.turnLeft(); // Turns left to face east
    myPainter.move();
    myPainter.move();
    myPainter.move();
    
  }
}
`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.Painter;

/*
 * Creates a PainterPlus
 * PainterPlus is a subclass of Painter.
 */
public class PainterPlus extends Painter {

  /*
   * Turns a PainterPlus object to the right
   * by turning left three times
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate: Programming Style and Tools #3`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the PainterPlus class, then instantiate a PainterPlus object.
     * -----------------------------------------------------------------------------
     */





    

    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate: Programming Style and Tools #4`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {


    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Conducting a Code Review (a)`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,6 0,0 1,0 1,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus colin = new PainterPlus();

    // Moves forward until PainterPlus reaches obstacle
    while (colin.canMove()) {
      colin.move();
    }

    // Takes all the paint from the paint bucket
    while (colin.isOnBucket()) {
      colin.takePaint();
    }

    // Turns right
    colin.turnRight();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the move() and paintLongDashes() methods to move forward and paint
     * dashed lines behind the taxes while the PainterPlus object can move forward.
     * -----------------------------------------------------------------------------
     */

    



    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter object to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Paints and moves two spaces
  public void paintLongDashes(String color) {
    paint(color);
    move();
    paint(color);
    move();
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from the paint bucket => ")
  public void testTakePaintFromBuckets() {
    message = "Use the takePaint() to take paint from the bucket while the Painter object is on the paint bucket.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 6), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paints lines behind each taxi => ")
  public void testPaintLinesBehindTaxis() {
    message = "Use the paintLongDashes() method to paint lines behind each taxi while the Painter object can move forward.";
    message += messageGap;
      
    boolean[][] expected = new boolean[10][10];

    expected[5][1] = true;
    expected[5][2] = true;
    expected[5][4] = true;
    expected[5][5] = true;
    expected[5][7] = true;
    expected[5][8] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Conducting a Code Review (b)`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus callie = new PainterPlus();

    // Turns right
    callie.turnRight();

    // Moves forward while PainterPlus can move then turns left
    callie.moveTurnLeft();

    // Moves forward while PainterPlus can move then turns right
    callie.moveTurnRight();

    // Moves forward while PainterPlus can move
    while (callie.canMove()) {
      callie.move();
    }

    // Moves forward while PainterPlus can move then turns right
    callie.moveTurnRight();

    // Moves forward while PainterPlus can move then turns left
    callie.moveTurnLeft();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the moveTurnLeft() and move() methods to move forward while
     * PainterPlus can move then turn left then continue moving to reach the end.
     * -----------------------------------------------------------------------------
     */

    



    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter object to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Moves forward while the Painter object can move then turns left
  public void moveTurnLeft() {
    while (canMove()) {
      move();
    }

    turnLeft();
  }

  // Moves forward while the Painter object can move then turns right
  public void moveTurnRight() {
    while (canMove()) {
      move();
    }

    turnRight();
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigates to x location 10 to reach the end of the path => ")
  public void testNavigateToXLocation() {
    int xLocation = primaryPainterLog.getEndingPosition().getX();
    
    message = "Use the moveTurnLeft() and moveTurnRight() methods to navigate to the end of the path.";
    message += messageGap;
      
    assertEquals(10, xLocation, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to y location 13 to reach the end of the path => ")
  public void testNavigateToYLocation() {
    int yLocation = primaryPainterLog.getEndingPosition().getY();
    
    message = "Use the moveTurnLeft() and moveTurnRight() methods to navigate to the end of the path.";
    message += messageGap;
      
    assertEquals(13, yLocation, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Conducting a Code Review (c)`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,12 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0
0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,12
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus melissa = new PainterPlus();

    // Turns right
    melissa.turnRight();

    // Moves forward two spaces
    melissa.move();
    melissa.move();
    melissa.move();

    // Takes all the paint from the paint bucket
    melissa.takeAllPaint();

    // Turns left
    melissa.turnLeft();

    // Paints a line while PainterPlus can move
    while (melissa.canMove()) {
      melissa.paintThenMove("white");
    }

    // Paints the last space
    melissa.paint("white");

    // Turns right
    melissa.turnRight();

    // Moves forward while PainterPlus can move
    while (melissa.canMove()) {
      melissa.move();
    }

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the takeAllPaint(), turnRight(), and paintThenMove() methods to take all
     * the paint from the bucket, turn right, then paint a line behind the food trucks.
     * -----------------------------------------------------------------------------
     */

    




    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter object to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Paints then moves forward one space
  public void paintThenMove(String color) {
    paint(color);
    move();
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from both paint buckets => ")
  public void testTakePaintFromBuckets() {
    message = "Use the takePaint() to take paint from the bucket while the Painter object is on the paint bucket.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 24), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paints lines in front of and behind the food trucks => ")
  public void testPaintLinesAroundFoodTrucks() {
    message = "Use the paintThenMove() method to paint a line while the Painter object can move.";
    message += messageGap;
      
    boolean[][] expected = new boolean[12][12];

    expected[0][3] = true;
    expected[1][3] = true;
    expected[2][3] = true;
    expected[3][3] = true;
    expected[4][3] = true;
    expected[5][3] = true;
    expected[6][3] = true;
    expected[7][3] = true;
    expected[8][3] = true;
    expected[9][3] = true;
    expected[10][3] = true;
    expected[11][3] = true;

    expected[0][8] = true;
    expected[1][8] = true;
    expected[2][8] = true;
    expected[3][8] = true;
    expected[4][8] = true;
    expected[5][8] = true;
    expected[6][8] = true;
    expected[7][8] = true;
    expected[8][8] = true;
    expected[9][8] = true;
    expected[10][8] = true;
    expected[11][8] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Conducting a Code Review (d)`,lesson:`Lesson 11: Programming Style and Feedback`,view:`neighborhood`,grid:`1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,5 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,5 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 1,5 1,0 1,0
1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0
1,0 1,0 1,0 1,5 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 1,0 1,0 1,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus brooke = new PainterPlus();

    // Moves forward one space
    brooke.move();

    // Takes all paint from the paint bucket
    brooke.takeAllPaint();

    // Moves while PainterPlus can move then turns right
    brooke.moveTurnRight();

    // Moves while PainterPlus can move
    while (brooke.canMove()) {
      brooke.move();
    }

    // Takes all paint from the paint bucket
    brooke.takeAllPaint();

    // Turns right
    brooke.turnRight();

    // Moves while PainterPlus can move then turns left
    brooke.moveTurnLeft();

    // Moves while PainterPlus can move then turns right
    brooke.moveTurnRight();

    // Moves while PainterPlus can move
    while (brooke.canMove()) {
      brooke.move();
    }

    // Takes all paint from the paint bucket
    brooke.takeAllPaint();

    // Turns left
    brooke.turnLeft();

    // Moves while PainterPlus can move then turns left
    brooke.moveTurnLeft();

    // Moves while PainterPlus can move
    while (brooke.canMove()) {
      brooke.move();
    }

    // Takes all paint from the paint bucket
    brooke.takeAllPaint();

    // Turns around
    brooke.turnLeft();
    brooke.turnLeft();

    // Moves while PainterPlus can move then turns right
    brooke.moveTurnRight();

    // Moves while PainterPlus can move then turns right
    brooke.moveTurnRight();

    // Moves while PainterPlus can move then turns right
    brooke.moveTurnRight();

    // Moves while PainterPlus can move then turns left
    brooke.moveTurnLeft();

    // Moves while PainterPlus can move then turns left
    brooke.moveTurnLeft();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the moveTurnLeft(), move(), and takeAllPaint() methods to navigate to
     * the last paint bucket and take all of the paint from the paint bucket.
     * -----------------------------------------------------------------------------
     */

    




    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter object to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Moves while the Painter object can move then turns left
  public void moveTurnLeft() {
    while (canMove()) {
      move();
    }

    turnLeft();
  }

  // Moves while the Painter object can move then turns right
  public void moveTurnRight() {
    while (canMove()) {
      move();
    }

    turnRight();
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from both paint buckets => ")
  public void testTakePaintFromBuckets() {
    message = "Use the takePaint() to take paint from the bucket while the Painter object is on the paint bucket.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 25), message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Selection Statements`,lesson:`Lesson 12: Selection Statements`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0
1,1 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus akira = new PainterPlus();

    if (akira.canMove("south")) {
      akira.turnRight();
      akira.move();
    }

    if (akira.isOnBucket()) {
      akira.takePaint();
      akira.turnLeft();
      akira.move();
      akira.paint("orange");
    }

    if (akira.isOnPaint()) {
      akira.move();
    }

    if (akira.isFacingEast()) {
      akira.move();
    }

    if (akira.canMove("north")) {
      akira.turnLeft();
      akira.move();
    }

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the PainterPlus object to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Selection Statements`,lesson:`Lesson 12: Selection Statements`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0
1,1 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus akira = new PainterPlus();

    if (akira.canMove("south")) {
      akira.turnRight();
      akira.move();
    }

    if (akira.isOnBucket()) {
      akira.takePaint();
      akira.turnLeft();
      akira.move();
      akira.paint("orange");
    }

    if (akira.isOnPaint()) {
      akira.move();
    }

    if (akira.isFacingEast()) {
      akira.move();
    }

    if (akira.canMove("north")) {
      akira.turnLeft();
      akira.move();
    }

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the PainterPlus object to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Writing Algorithms (a)`,lesson:`Lesson 12: Selection Statements`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,6 0,0 0,0 1,0 1,0 1,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,6 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the TrafficPainter class and the method paintLines().
     * Then, instantiate a TrafficPainter object and use the paintLines() method.
     * -----------------------------------------------------------------------------
     */

    





    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TrafficPainter.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Create a TrafficPainter class => ")
  public void testCreateTrafficPainterClass() {
    message = "Create a new file named TrafficPainter.java and write the class header. Remember that the ";
    message += "\\n        class header contains the keyword \\"class\\" and the name of the class.";
    message += messageGap;

    List<String> classNames = ValidationHelper.getClassNames();
      
    assertTrue(classNames.contains("TrafficPainter"), message);
  }

  @Test
  @Order(2)
  @SuppressWarnings("unchecked")
  @DisplayName("TrafficPainter is a subclass of the PainterPlus class => ")
  public void testTrafficPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the TrafficPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class trafficPainterClass = null;

    try {
      trafficPainterClass = Class.forName("TrafficPainter");
    } catch (Exception e) {
      message = "Your class header for TrafficPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(trafficPainterClass), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Takes all of the paint from the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Use the takePaint() method inside a while loop to take all of the paint from a paint bucket. You could also";
    message += "\\n        add a takeAllPaint() method to your PainterPlus class that you can use in TrafficPainter or NeighborhoodRunner.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 12), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("Paints lines next to the taxis => ")
  public void testPaintLinesNextToTaxis() {
    message = "Paint lines next to the taxis, but be sure not to paint the space between the taxis.";
    message += "\\n        Try checking if the Painter can move east and keep moving forward if this is true.";
    message += messageGap;
      
    boolean[][] expected = new boolean[10][10];

    expected[3][0] = true;
    expected[3][1] = true;
    expected[3][2] = true;
    expected[3][4] = true;
    expected[3][5] = true;
    expected[3][6] = true;
    expected[6][1] = true;
    expected[6][2] = true;
    expected[6][3] = true;
    expected[6][5] = true;
    expected[6][6] = true;
    expected[6][7] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (b)`,lesson:`Lesson 12: Selection Statements`,view:`neighborhood`,grid:`1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the MarathonPainter class and the methods moveToSouth() and moveToEast().
     * Then, instantiate a MarathonPainter object and use the methods you wrote.
     * -----------------------------------------------------------------------------
     */

    





    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MarathonPainter.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Create a MarathonPainter class => ")
  public void testCreateMarathonPainterClass() {
    message = "Create a new file named MarathonPainter.java and write the class header. Remember that the ";
    message += "\\n        class header contains the keyword \\"class\\" and the name of the class.";
    message += messageGap;

    List<String> classNames = ValidationHelper.getClassNames();
      
    assertTrue(classNames.contains("MarathonPainter"), message);
  }

  @Test
  @Order(2)
  @SuppressWarnings("unchecked")
  @DisplayName("MarathonPainter is a subclass of the PainterPlus class => ")
  public void testMarathonPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the MarathonPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class marathonPainterClass = null;

    try {
      marathonPainterClass = Class.forName("MarathonPainter");
    } catch (Exception e) {
      message = "Your class header for MarathonPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(marathonPainterClass), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Navigates to x location 11 to reach the traffic cone => ")
  public void testNavigateToXLocation() {
    int xLocation = primaryPainterLog.getEndingPosition().getX();
    
    message = "Use the methods in the MarathonPainter class to move east until the MarathonPainter can move south.";
    message += messageGap;
      
    assertEquals(11, xLocation, message);
  }
   
  @Test
  @Order(4)
  @DisplayName("Navigates to y location 0 to reach the traffic cone => ")
  public void testNavigateToYLocation() {
    int yLocation = primaryPainterLog.getEndingPosition().getY();
    
    message = "Use the methods in the MarathonPainter class to move east until the MarathonPainter can move south.";
    message += messageGap;
      
    assertEquals(0, yLocation, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (c)`,lesson:`Lesson 12: Selection Statements`,view:`neighborhood`,grid:`1,0 1,12 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 1,0
1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the MuralPainter class and the method paintDiagonal().
     * Then, instantiate a MuralPainter object and use the paintDiagonal() method.
     * -----------------------------------------------------------------------------
     */

    
    





    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MuralPainter.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Create a MuralPainter class => ")
  public void testCreateMuralPainterClass() {
    message = "Create a new file named MuralPainter.java and write the class header. Remember that the ";
    message += "\\n        class header contains the keyword \\"class\\" and the name of the class.";
    message += messageGap;

    List<String> classNames = ValidationHelper.getClassNames();
      
    assertTrue(classNames.contains("MuralPainter"), message);
  }

  @Test
  @Order(2)
  @SuppressWarnings("unchecked")
  @DisplayName("MuralPainter is a subclass of the PainterPlus class => ")
  public void testMuralPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the MuralPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class muralPainterClass = null;

    try {
      muralPainterClass = Class.forName("MuralPainter");
    } catch (Exception e) {
      message = "Your class header for MuralPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(muralPainterClass), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Takes all of the paint from the paint bucket => ")
  public void testTakeAllPaintFromBucket() {
    message = "Use the takePaint() method inside a while loop to take all of the paint from a paint bucket. You could also";
    message += "\\n        add a takeAllPaint() method to your PainterPlus class that you can use in MuralPainter or NeighborhoodRunner.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 12), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("Paints a diagonal line across The Neighborhood => ")
  public void testPaintDiagonalLine() {
    message = "Check which direction the Painter is facing to determine which direction to turn and move before painting the next space.";
    message += messageGap;
      
    boolean[][] expected = new boolean[12][12];

    expected[0][0] = true;
    expected[1][1] = true;
    expected[2][2] = true;
    expected[3][3] = true;
    expected[4][4] = true;
    expected[5][5] = true;
    expected[6][6] = true;
    expected[7][7] = true;
    expected[8][8] = true;
    expected[9][9] = true;
    expected[10][10] = true;
    expected[11][11] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (d)`,lesson:`Lesson 12: Selection Statements`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0 0,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,6 1,0 1,0 0,0
1,0 1,0 0,0 0,0 1,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 1,0 1,6 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the EventPainter class and the methods takePaintThenPaint(), moveToNextRow(),
     * and paintRow(). Then, instantiate a EventPainter object and use the methods.
     * -----------------------------------------------------------------------------
     */

    




    
    
    
  }
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("EventPainter.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Create a EventPainter class => ")
  public void testCreateEventPainterClass() {
    message = "Create a new file named EventPainter.java and write the class header. Remember that the ";
    message += "\\n        class header contains the keyword \\"class\\" and the name of the class.";
    message += messageGap;

    List<String> classNames = ValidationHelper.getClassNames();
      
    assertTrue(classNames.contains("EventPainter"), message);
  }

  @Test
  @Order(2)
  @SuppressWarnings("unchecked")
  @DisplayName("EventPainter is a subclass of the PainterPlus class => ")
  public void testEventPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the EventPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class eventPainterClass = null;

    try {
      eventPainterClass = Class.forName("EventPainter");
    } catch (Exception e) {
      message = "Your class header for EventPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(eventPainterClass), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Takes all of the paint from the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Use the takePaint() method inside a while loop to take all of the paint from a paint bucket. You could also";
    message += "\\n        add a takeAllPaint() method to your PainterPlus class that you can use in EventPainter or NeighborhoodRunner.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 12), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("Paints the areas inside the walls => ")
  public void testPaintAreas() {
    message = "Try breaking up the problem into two parts. The first row of each area requires the EventPainter to first collect paint,";
    message += "\\n        then move and paint. The EventPainter then has to move to the next row to paint the second row in the area.";
    message += messageGap;
      
    boolean[][] expected = new boolean[12][12];

    expected[3][7] = true;
    expected[3][8] = true;
    expected[3][9] = true;
    expected[4][7] = true;
    expected[4][8] = true;
    expected[4][9] = true;
    expected[8][3] = true;
    expected[8][4] = true;
    expected[9][3] = true;
    expected[9][4] = true;
    expected[10][3] = true;
    expected[10][4] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Variables and Printing`,lesson:`Lesson 13: Debugging Strategies`,view:`neighborhood`,grid:`1,0 1,20 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus marie = new PainterPlus();

    String currentDirection;
    int currentPaint;

    marie.move();
    marie.takeAllPaint();

    while (marie.canMove()) {
      marie.paintThenMove("coral");
      currentPaint = marie.getMyPaint();
      System.out.println("Paint remaining after moving east: " + currentPaint);
      
      marie.turnRight();
      currentDirection = marie.getDirection();
      System.out.println("Currently facing: " + currentDirection);
      
      marie.paintThenMove("coral");
      marie.turnLeft();
    }
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Paints then moves forward while the Painter can move
  public void paintThenMove(String color) {
    while (canMove()) {
      paint(color);
      move();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Debugging`,lesson:`Lesson 13: Debugging Strategies`,view:`neighborhood`,grid:`1,0 1,20 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus marie = new PainterPlus();

    String currentDirection;
    int currentPaint;

    marie.move();
    marie.takeAllPaint();

    while (marie.canMove()) {
      marie.paintThenMove("coral");
      currentPaint = marie.getMyPaint();
      System.out.println("Paint remaining after moving east: " + currentPaint);
      
      marie.turnRight();
      currentDirection = marie.getDirection();
      System.out.println("Currently facing: " + currentDirection);
      
      marie.paintThenMove("coral");
      marie.turnLeft();
    }
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Paints then moves forward while the Painter can move
  public void paintThenMove(String color) {
    while (canMove()) {
      paint(color);
      move();
    }
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from the paint bucket => ")
  public void testTakesAllPaintFromBucket() {
    message = "Use the takeAllPaint() method to take all of the paint from a paint bucket while the Painter object is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 20), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paints the path to the traffic cone => ")
  public void testPaintPathToTrafficCone() {
    message = "Use the paintThenMove() method to paint and move while the Painter object can move forward.";
    message += "\\n        Be sure to turn to navigate through The Neighborhood to reach the traffic cone.";
    message += messageGap;
      
    // act and assert code
  }
   
  private static void getNeighborhoodResults() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Debugging in The Neighborhood (a)`,lesson:`Lesson 13: Debugging Strategies`,view:`neighborhood`,grid:`1,0 1,13 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus molly = new PainterPlus();

    // Move forward while PainterPlus can move
    while (molly.canMove()) {
      // Take all of the paint if PainterPlus is on a paint bucket
      if (molly.isOnBucket()) {
        molly.takeAllPaint();
      }

      // Paint the current space if PainterPlus has paint
      if (molly.hasPaint()) {
        molly.paint("red");
      }

      // Move forward one space
      molly.move();
    }

    // Paint while PainterPlus has paint
    while (molly.hasPaint()) {
      molly.paint("red");

      // Move forward one space if PainterPlus can move
      if (molly.canMove()) {
        molly.move();
      }
    }
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from the paint bucket => ")
  public void testTakeAllPaintFromBucket() {
    message = "Use the takeAllPaint() method in the PainterPlus class to take all of the paint from ";
    message += "\\n        a paint bucket while the PainterPlus object is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 13), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paints the path to the house => ")
  public void testPaintPathToHouse() {
    message = "Paint the first row, then turn towards the house to move and paint the rest of the path to the house.";
    message += messageGap;
      
    boolean[][] expected = new boolean[12][12];

    expected[1][0] = true;
    expected[2][0] = true;
    expected[3][0] = true;
    expected[4][0] = true;
    expected[5][0] = true;
    expected[5][1] = true;
    expected[5][2] = true;
    expected[5][3] = true;
    expected[5][4] = true;
    expected[5][5] = true;
    expected[5][6] = true;
    expected[5][7] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
   
  private static void getNeighborhoodResults() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Debugging in The Neighborhood (b)`,lesson:`Lesson 13: Debugging Strategies`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 1,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 0,0 0,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0 1,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0 1,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus evan = new PainterPlus();

    // Turns right
    evan.turnRight();

    // Moves south then turns east
    evan.moveSouthTurnEast();

    // Moves east then turns north
    evan.moveEastTurnNorth();

    // Moves north then turns east
    evan.moveNorthTurnEast();

    // Moves east then turns south
    evan.moveEastTurnSouth();

    // Moves south then turns east
    evan.moveSouthTurnEast();

    // Moves east then turns north
    evan.moveEastTurnNorth();

    // Moves north then turns east
    evan.moveNorthTurnEast();

    // Moves east then turns south
    evan.moveEastTurnSouth();

    // Moves forward while PainterPlus can move
    while (evan.canMove()) {
      evan.move();
    }
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves north while facing north and can move
   * forward and turns right if can move east
   */
  public void moveNorthTurnEast() {
    while (isFacingNorth()) {
      if (canMove()) {
        move();
      }

      if (canMove("east")) {
        turnRight();
      }
    }
  }

  /*
   * Moves south while facing south and can move
   * forward and turns left if can move east
   */
  public void moveSouthTurnEast() {
    while (isFacingSouth()) {
      if (canMove()) {
        move();
      }

      if (canMove("east")) {
        turnLeft();
      }
    }
  }

  /*
   * Moves east while facing east and can move
   * forward and turns left if can move north
   */
  public void moveEastTurnNorth() {
    while (isFacingEast()) {
      if (canMove("north")) {
        turnLeft();
      }
    }
  }

  /*
   * Moves east while facing east and can move
   * forward and turns right if can move south
   */
  public void moveEastTurnSouth() {
    while (isFacingEast()) {
      if (canMove()) {
        move();
      }

      if (canMove("south")) {
        turnRight();
      }
    }
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Navigates to x location 10 to reach the traffic cone => ")
  public void testNavigateToXLocation() {
    int currentX = primaryPainterLog.getEndingPosition().getX();
    
    message = "Check that the Painter object moves while it is able to move";
    message += "\\n        and turns to the correct direction when it can no longer move.";
    message += messageGap;
      
    assertEquals(10, currentX, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Navigates to y location 10 to reach the traffic cone => ")
  public void testNavigateToYLocation() {
    int currentY = primaryPainterLog.getEndingPosition().getY();
    
    message = "Check that the Painter object moves while it is able to move";
    message += "\\n        and turns to the correct direction when it can no longer move.";
    message += messageGap;
      
    assertEquals(10, currentY, message);
  }
   
  private static void getNeighborhoodResults() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Debugging in The Neighborhood (c)`,lesson:`Lesson 13: Debugging Strategies`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,2 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
1,2 1,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,2 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,6 0,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus brooke = new PainterPlus();

    // Moves east while facing east
    while (brooke.isFacingEast()) {
      // Takes all of the paint from the paint bucket if on one
      brooke.takePaintIfOnBucket();

      // Turns right if can move south
      if (brooke.canMove("south")) {
        brooke.turnRight();
      }

      // Moves forward one space
      brooke.move();
    }

    // Paints the first area
    brooke.paintTwice("coral");
    brooke.turnAround();

    // Moves north while can move north
    while (brooke.canMove("north")) {
      brooke.move();

      // Turns right if can move east
      if (brooke.canMove("east")) {
        brooke.turnRight();
      }
    }

    // Moves east while facing east
    while (brooke.isFacingEast()) {
      brooke.move();

      // Turns right if can move south
      if (brooke.canMove("south")) {
        brooke.turnRight();
      }
    }

    // Moves south while facing south
    while (brooke.isFacingSouth()) {
      brooke.move();

      // Turns right and moves one space if can move west
      if (brooke.canMove("west")) {
        brooke.turnRight();
        brooke.move();
      }
    }

    // Gets the paint and paints the second area
    brooke.takePaintIfOnBucket();
    brooke.move();
    brooke.paintTwice("coral");
    brooke.turnAround();

    // Moves east while facing east
    while (brooke.isFacingEast()) {
      // Turns right if can move south
      if (brooke.canMove("south")) {
        brooke.turnRight();
      }

      brooke.move();
    }

    // Moves while can move then turns right
    brooke.moveTurnRight();

    // Moves while can move then turns right
    brooke.moveTurnRight();

    // Gets the paint and paints one side of the third area
    while (brooke.canMove()) {
      brooke.takePaintIfOnBucket();
      brooke.move();
      brooke.paintThree("coral"); 
    }

    // Moves to the other side of the third area
    brooke.turnLeft();
    brooke.move();
    brooke.turnLeft();

    // // Paints the other side of the third area
    brooke.paintTwice("coral");
    brooke.move();

    // Turns right
    brooke.turnRight();

    // Moves while can move then turns right
    brooke.moveTurnRight();

    // Moves forward while can move
    while (brooke.canMove()) {
      brooke.move();
    }

    // Gets the paint and paints the last area
    brooke.takePaintIfOnBucket();
    brooke.turnRight();
    brooke.move();
    brooke.paintTwice("coral");
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Turns around to face the opposite direction
  public void turnAround() {
    turnLeft();
    turnLeft();
  }

  // Moves forward while can move then turns right
  public void moveTurnRight() {
    while (canMove()) {
      move();
    }

    turnRight();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Takes all of the paint from a paint bucket if on a paint bucket
  public void takePaintIfOnBucket() {
    if (isOnBucket()) {
      takeAllPaint();
    }
  }

  // Paints two spaces
  public void paintTwice(String color) {
    if (hasPaint()) {
      paint(color);
      move();
      paint(color);
    }
  }

  // Paints three spaces
  public void paintThree(String color) {
    if (hasPaint()) {
      paint(color);
      move();
      paint(color);
      paint(color);
    }
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Check that the Painter takes all of the paint from the paint bucket when it is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 12), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paint the paths to the house => ")
  public void testPaintEachPaintToHouse() {
    message = "Check that the Painter paints two spaces in the shorter paths and all six ";
    message += "\\n        spaces in the larger path then navigates to the next path.";
    message += messageGap;
      
    boolean[][] expected = new boolean[12][12];

    expected[5][1] = true;
    expected[5][2] = true;
    expected[8][4] = true;
    expected[9][4] = true;
    expected[5][6] = true;
    expected[6][6] = true;
    expected[5][7] = true;
    expected[6][7] = true;
    expected[5][8] = true;
    expected[6][8] = true;
    expected[1][4] = true;
    expected[2][4] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
   
  private static void getNeighborhoodResults() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Debugging in The Neighborhood (d)`,lesson:`Lesson 13: Debugging Strategies`,view:`neighborhood`,grid:`1,0 1,15 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,9 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,15 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,7
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a PainterPlus object
    PainterPlus amos = new PainterPlus();

    // Moves, takes paint from the paint bucket, and paints the first row
    amos.moveAndPaint("purple");

    // Turns around
    amos.turnAround();

    // Moves back to the starting point then turns south
    while (amos.isFacingWest()) {
      amos.move();

      if (amos.canMove("south")) {
        amos.turnLeft();
      }
    }

    // Moves forward and paints second row while PainterPlus can move
    while (amos.canMove()) {
      amos.move();

      // Takes all of the paint from the paint bucket then paints previous space
      if (amos.isOnBucket()) {
        amos.takeAllPaint();
        amos.turnAround();
        amos.move();
        amos.turnAround();
      }
    }

    // Turns left
    amos.turnLeft();

    // Moves, takes paint from the paint bucket, and paints the third row
    amos.moveAndPaint("purple");

    // Turns right
    amos.turnRight();

    // Moves, takes paint from the paint bucket, and paints the last row
    amos.moveAndPaint("purple");
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Turns around to face the opposite direction
  public void turnAround() {
    turnLeft();
    turnLeft();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  // Moves and paints while the Painter can move forward
  public void moveAndPaint(String color) {
    while (canMove()) {
      move();

      if (hasPaint()) {
        paint("purple");
      }

      if (isOnBucket()) {
        takeAllPaint();
      }
    }
  }
  
}`}],validationFiles:[{path:`RunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NeighborhoodRunner.java Test")
public class RunnerTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Takes all of the paint from the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Check that the Painter object takes all of the paint from the paint bucket when it is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 46), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Paints the streets in The Neighborhood => ")
  public void testPaintStreets() {
    message = "Check that the Painter object moves and turns as needed to navigate The Neighborhood. The Painter";
    message += "\\n        object should be painting each space as it moves if it has paint.";
    message += messageGap;
      
    boolean[][] expected = new boolean[16][16];

    expected[0] = new boolean[]{true, true, true, true, true, true, true, true, true, false, false, false, false, false, false, false};
    
    expected[1][0] = true;
    expected[1][8] = true;
    
    expected[2][0] = true;
    expected[2][8] = true;
    
    expected[3][0] = true;
    expected[3][8] = true;
    
    expected[4][0] = true;
    expected[4][8] = true;
    
    expected[5][0] = true;
    expected[5][8] = true;
    
    expected[6][0] = true;
    expected[6][8] = true;
    
    expected[7][0] = true;
    expected[7][8] = true;
    
    expected[8][0] = true;
    expected[8][8] = true;
    
    expected[9][0] = true;
    expected[9][8] = true;
    
    expected[10][0] = true;
    expected[10][8] = true;
    
    expected[11][0] = true;
    expected[11][8] = true;
    
    expected[12][0] = true;
    expected[12][8] = true;
    
    expected[13][0] = true;
    expected[13][8] = true;
    
    expected[14][0] = true;
    expected[14][8] = true;
    
    expected[15] = new boolean[]{true, false, false, false, false, false, false, false, true, true, true, true, true, true, true, true};

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
   
  private static void getNeighborhoodResults() {
    message = "There is an error in your NeighborhoodRunner.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      PainterLog[] painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Selection and Logic`,lesson:`Lesson 14: Two-Way Selection Statements`,view:`neighborhood`,grid:`1,0 1,7 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 0,0 1,6 1,0 1,2 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,5 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,3 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus sierra = new PainterPlus();
    sierra.paintToAndAroundTruck("white");
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  public void paintToAndAroundTruck(String color) {
    while (canMove()) {
      paintPath(color);
    }
    
    makeTurn();

    while (canMove()) {
      paintTruckBorder(color);
      returnToStart();
    }
  }

  public void paintPath(String color) {
    moveOrTakePaint();
    paintIfHasPaint(color);
    turnSouth();
  }

  public void paintTruckBorder(String color) {
    moveOrTakePaint();
    paintIfHasPaint(color);
    makeTurn();
  }

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  public void moveOrTakePaint() {
    if (!isOnBucket()) {
      move();
    }

    takeAllPaint();
  }

  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  public void turnSouth() {
    if (canMove("south")) {
      turnRight();
    }
  }

  public void makeTurn() {
    if (!canMove()) {
      turnLeft();
    }
  }

  public void returnToStart() {
    if (isFacingWest()) {
      if (canMove("north")) {
        turnRight();
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Selection and Logic`,lesson:`Lesson 14: Two-Way Selection Statements`,view:`neighborhood`,grid:`1,0 1,7 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 0,0 1,6 1,0 1,2 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,5 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,3 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    PainterPlus sierra = new PainterPlus();
    sierra.paintToAndAroundTruck("white");
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

public class PainterPlus extends Painter {

  public void paintToAndAroundTruck(String color) {
    while (canMove()) {
      paintPath(color);
    }
    
    makeTurn();

    while (canMove()) {
      paintTruckBorder(color);
      returnToStart();
    }
  }

  public void paintPath(String color) {
    moveOrTakePaint();
    paintIfHasPaint(color);
    turnSouth();
  }

  public void paintTruckBorder(String color) {
    moveOrTakePaint();
    paintIfHasPaint(color);
    makeTurn();
  }

  // Turns the Painter to the right
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  // Takes all of the paint from a paint bucket
  public void takeAllPaint() {
    while (isOnBucket()) {
      takePaint();
    }
  }

  public void moveOrTakePaint() {
    if (!isOnBucket()) {
      move();
    }

    takeAllPaint();
  }

  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  public void turnSouth() {
    if (canMove("south")) {
      turnRight();
    }
  }

  public void makeTurn() {
    if (!canMove()) {
      turnLeft();
    }
  }

  public void returnToStart() {
    if (isFacingWest()) {
      if (canMove("north")) {
        turnRight();
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Writing Algorithms (a)`,lesson:`Lesson 14: Two-Way Selection Statements`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,6 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,6 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,6 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a TrafficPainter object and call the navigateTraffic() method.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`TrafficPainter.java`,text:`import org.code.neighborhood.*;

public class TrafficPainter {

  /*
   * Navigates and paints the path by moving forward if TrafficPainter can
   * move, taking all of the paint from a paint bucket if the TrafficPainter
   * is standing on one, and turning in the appropriate direction
   */
  public void navigateTraffic(String color) {
    turnRight();

    while (canMove()) {
      paintIfHasPaint(color);
      move();
      checkForBucket();
      makeTurn();
    }
  }

  /*
   * Checks if TrafficPainter is on a paint bucket and takes
   * all of the paint from the bucket if so
   */
  public void checkForBucket() {
    if (isOnBucket()) {
      takeAllPaint();
    }
  }

  /*
   * Paints the current space if TrafficPainter has paint
   */
  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the makeTurn() method to choose the correct direction to turn.
   * -----------------------------------------------------------------------------
   *
   * Chooses the direction to turn based on whether TrafficPainter can move south
   */
  public void makeTurn() {
    
    


    
  }

}`}],validationFiles:[{path:`TrafficPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TrafficPainter.java Test")
public class TrafficPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @SuppressWarnings("unchecked")
  @DisplayName("TrafficPainter is a subclass of the PainterPlus class => ")
  public void testSTrafficPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the TrafficPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class trafficPainterClass = null;

    try {
      trafficPainterClass = Class.forName("TrafficPainter");
    } catch (Exception e) {
      message = "Your class header for TrafficPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(trafficPainterClass), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Takes all of the paint from all of the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Make sure to take all of the paint from the paint buckets when the TrafficPainter object is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 18), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Paints the path through The Neighborhood => ")
  public void testPaintsPath() {
    message = "Paint the current space, then move forward if TrafficPainter can move. If the TrafficPainter still has paint, paint the next space.";
    message += messageGap;
      
    boolean[][] expected = new boolean[12][12];

    expected[0][1] = true;
    expected[0][2] = true;
    expected[0][3] = true;
    expected[1][3] = true;
    expected[2][3] = true;
    expected[3][3] = true;
    expected[4][3] = true;
    expected[4][4] = true;
    expected[4][5] = true;
    expected[5][5] = true;
    expected[6][5] = true;
    expected[7][5] = true;
    expected[8][5] = true;
    expected[8][4] = true;
    expected[9][4] = true;
    expected[10][4] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (b)`,lesson:`Lesson 14: Two-Way Selection Statements`,view:`neighborhood`,grid:`1,0 1,5 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,5 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 1,0 1,0 1,5 1,0 1,0 1,0 1,0 1,5 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,5 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0
0,0 1,0 1,0 1,0 1,0 1,5 1,0 1,0 1,0 1,0 1,5 1,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a DecorPainter object and call the paintStreets() method.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`DecorPainter.java`,text:`import org.code.neighborhood.*;

public class DecorPainter {

  /*
   * Navigates and paints the path by moving forward if DecorPainter can
   * move, taking all of the paint from a paint bucket if the DecorPainter
   * is standing on one, and turning in the appropriate direction
   */
  public void paintStreets(String color) {
    while (canMove()) {
      paintIfHasPaint(color);
      move();
      checkForBucket();
      makeTurn();
    }

    finishStreet(color);
  }

  /*
   * Checks if DecorPainter is on a paint bucket and takes
   * all of the paint from the bucket if so
   */
  public void checkForBucket() {
    if (isOnBucket()) {
      takeAllPaint();
    }
  }

  /*
   * Paints the current space if DecorPainter has paint
   */
  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the makeTurn() method to choose the correct direction to turn.
   * -----------------------------------------------------------------------------
   *
   * Chooses the direction to turn based on whether DecorPainter can move south
   */
  public void makeTurn() {
    
    


    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the makeTurn() method to choose the correct direction to turn.
   * -----------------------------------------------------------------------------
   *
   * Turns DecorPainter to the west to finish the last street
   */
  public void finishStreet(String color) {
    
    
    

    
  }
  
}`}],validationFiles:[{path:`DecorPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("DecorPainter.java Test")
public class DecorPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @SuppressWarnings("unchecked")
  @DisplayName("DecorPainter is a subclass of the PainterPlus class => ")
  public void testDecorPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the DecorPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class decorPainterClass = null;

    try {
      decorPainterClass = Class.forName("DecorPainter");
    } catch (Exception e) {
      message = "Your class header for DecorPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(decorPainterClass), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Takes all of the paint from all of the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Make sure to take all of the paint from the paint buckets when the TrafficPainter object is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 35), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Paints the streets in The Neighborhood => ")
  public void testPaintStreets() {
    message = "Paint the current space, then move forward if DecorPainter can move. If the TrafficPainter still has paint, paint the next space.";
    message += messageGap;
      
    boolean[][] expected = new boolean[14][14];

    expected[1][0] = true;
    expected[2][0] = true;
    expected[3][0] = true;
    expected[4][0] = true;
    expected[4][1] = true;
    expected[4][2] = true;
    expected[4][3] = true;
    expected[4][4] = true;
    expected[5][4] = true;
    expected[6][4] = true;
    expected[7][4] = true;
    expected[8][4] = true;
    expected[9][4] = true;
    expected[10][4] = true;
    expected[11] = new boolean[]{false, false, false, false, true, true, true, true, true, true, true, true, true, true};
    expected[2][13] = true;
    expected[3][13] = true;
    expected[4][13] = true;
    expected[5][13] = true;
    expected[6][13] = true;
    expected[7][13] = true;
    expected[8][13] = true;
    expected[9][13] = true;
    expected[10][13] = true;
    expected[11][13] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (c)`,lesson:`Lesson 14: Two-Way Selection Statements`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,28 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a FishPainter object and call the paintMural() method.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`FishPainter.java`,text:`import org.code.neighborhood.*;

public class FishPainter {

  /*
   * Navigates to the position and gets the paint from the paint bucket
   * then paints a mural of a fish in The Neighborhood
   */
  public void paintMural(String firstColor, String secondColor) {
    getToPosition();
    paintFish(firstColor, secondColor);
  }

  /*
   * Paints a fish
   */
  public void paintFish(String firstColor, String secondColor) {
    paintTop(firstColor, secondColor);
    turnRight();
    paintMiddle(firstColor, secondColor);
    turnLeft();
    paintMiddle(firstColor, secondColor);
    turnRight();
    paintBottom(firstColor, secondColor);
  }

  /*
   * Navigates to the starting position and retrieves paint
   */
  public void getToPosition() {
    turnRight();

    while (isFacingSouth()) {
      move();
      checkForBucket();

      if (canMove("east")) {
        move();
        turnLeft();
      }
    }
  }

  /*
   * Checks if FishPainter is on a paint bucket and takes
   * all of the paint from the bucket if so
   */
  public void checkForBucket() {
    if (isOnBucket()) {
      takeAllPaint();
    }
  }

  /*
   * Moves and paints once with the color
   */
  public void paintOnce(String color) {
    move();
    paint(color);
  }

  /*
   * Moves and paints twice with the color
   */
  public void paintTwice(String color) {
    move();
    paint(color);
    move();
    paint(color);
  }

  /*
   * Paints the top of the fish
   */
  public void paintTop(String firstColor, String secondColor) {
    paintOnce(firstColor);
    paintTwice(secondColor);
    paintOnce(firstColor);
    paintOnce(secondColor);
    move();
    paintOnce(secondColor);
  }

  /*
   * Paints the bottom of the fish
   */
  public void paintBottom(String firstColor, String secondColor) {
    paintOnce(secondColor);
    turnRight();
    move();
    paintOnce(secondColor);
    paintOnce(firstColor);
    paintTwice(secondColor);
    paintOnce(firstColor);
  }

  /*
   * Paints the middle portion of the fish. If the FishPainter is
   * facing west, it paints "black" for the fish's eye.
   */
  public void paintMiddle(String firstColor, String secondColor) {
    // paint two spaces with secondColor
    move();
    paint(secondColor);

    makeTurn();
    
    move();
    paint(secondColor);
    paintOnce(firstColor);
    paintTwice(secondColor);
    paintOnce(firstColor);

    // paint two spaces with secondColor
    move();

    paintEyeOrBody(secondColor);
    
    move();
    paint(secondColor);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the makeTurn() method to paint the middle section of the fish.
   * -----------------------------------------------------------------------------
   *
   * Turns the FishPainter based on the direction it is currently facing
   */
  public void makeTurn() {
    
    
    

    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the paintEyeOrBody() method to paint the middle section of the fish.
   * -----------------------------------------------------------------------------
   *
   * Paints either the eye or part of the body
   */
  public void paintEyeOrBody(String color) {
    
    


    
  }
  
}`}],validationFiles:[{path:`FishPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("FishPainter.java Test")
public class FishPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @SuppressWarnings("unchecked")
  @DisplayName("FishPainter is a subclass of the PainterPlus class => ")
  public void testFishPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the FishPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class fishPainterClass = null;

    try {
      fishPainterClass = Class.forName("FishPainter");
    } catch (Exception e) {
      message = "Your class header for FishPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(fishPainterClass), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Takes all of the paint from all of the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Make sure to take all of the paint from the paint buckets when the FishPainter object is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 28), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Paints a mural of a fish in The Neighborhood => ")
  public void testPaintFishMural() {
    message = "Use if/else statements to determine which direction to turn or when to paint the eye while painting the middle.";
    message += messageGap;
      
    boolean[][] expected = new boolean[18][18];

    expected[0][8] = true;
    expected[0][9] = true;
    expected[1][7] = true;
    expected[1][8] = true;
    expected[1][9] = true;
    expected[1][10] = true;

    expected[2][7] = true;
    expected[2][8] = true;
    expected[2][9] = true;
    expected[2][10] = true;

    expected[3][7] = true;
    expected[3][8] = true;
    expected[3][9] = true;
    expected[3][10] = true;

    expected[4][7] = true;
    expected[4][8] = true;
    expected[4][9] = true;
    expected[4][10] = true;

    expected[5][7] = true;
    expected[5][8] = true;
    expected[5][9] = true;
    expected[5][10] = true;

    expected[6][8] = true;
    expected[6][9] = true;

    expected[7][7] = true;
    expected[7][8] = true;
    expected[7][9] = true;
    expected[7][10] = true;

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (d)`,lesson:`Lesson 14: Two-Way Selection Statements`,view:`neighborhood`,grid:`1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 1,4 1,0 1,0 1,0 1,0 1,5 0,0 0,0 0,0 0,0 0,0
1,5 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,5 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,5 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,5 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
1,5 1,0 1,0 1,0 1,0 1,4 1,0 1,0 1,0 1,0 1,5 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a SunPainter object and call the paintSunMural() method.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`SunPainter.java`,text:`import org.code.neighborhood.*;

public class SunPainter {

  /*
   * Navigates to the position and gets the paint from the paint buckets
   * then paints a mural of a sun in The Neighborhood
   */
  public void paintSunMural(String color) {
    getToPosition();
    paintSun(color);
  }

  /*
   * Navigates to the starting position
   */
  public void getToPosition() {
    turnRight();
    
    while (isFacingSouth()) {
      move();

      if (canMove("east")) {
        turnLeft();
      }
    }

    moveFast();
    turnRight();
  }

  /*
   * Paints a sun
   */
  public void paintSun(String color) {
    paintSunCenter(color);
    paintSoutheast(color);
    paintBottomMiddle(color);
    paintSouthwest(color);
    paintNorthwest(color);
    paintTopMiddle(color);
    paintNortheast(color);
  }

  /*
   * Paints the center part of the sun
   */
  public void paintSunCenter(String color) {
    moveNoBucket();
    turnLeft();
    paintLineAndTurn(color);
    moveNoBucket();
    paintLineAndTurn(color);
    moveNoBucket();
    paintLineAndTurn(color);
    moveNoBucket();
    paintLineAndTurn(color);
    paintLine(color);
    moveNoBucket();
    paintLine(color);
  }

  /*
   * Paints the southeast diagonal line
   */
  public void paintSoutheast(String color) {
    // Moves to bottom right corner
    moveFast();
    turnRight();
    moveFast();

    // Gets all of the paint from the paint bucket
    takeAllPaint();

    // Turns around to face north
    turnAround();

    while (!isOnPaint()) {
      paintDiagonalNW(color);
    }
  }

  /*
   * Paints the bottom middle line
   */
  public void paintBottomMiddle(String color) {
    // Turns to face south
    turnLeft();
    move();
    turnLeft();

    // Moves to bottom middle
    moveFast();

    // Gets all of the paint from the paint bucket
    takeAllPaint();

    // Turns around to face north
    turnAround();

    // Paints bottom center line
    paintLine(color);
  }

  /*
   * Paints the southwest diagonal line
   */
  public void paintSouthwest(String color) {
    // Moves to bottom left corner
    turnLeft();
    moveFast();
    turnLeft();
    moveFast();

    // Gets all of the paint from the paint bucket
    takeAllPaint();

    // Turns around
    turnAround();

    // Paints a diagonal line while not on paint
    while (!isOnPaint()) {
      paintDiagonalNE(color);
    }
  }

  /*
   * Paints the northwest diagonal line
   */
  public void paintNorthwest(String color) {
    // Moves to top left corner
    moveFast();
    turnLeft();
    moveFast();

    // Gets all of the paint from the paint bucket
    takeAllPaint();

    // Moves up a space
    turnRight();
    move();

    // Turns around to face south
    turnAround();

    // Paints a diagonal line while not on paint
    while (!isOnPaint()) {
      paintDiagonalSE(color);
    }
  }

  /*
   * Paints the top middle line
   */
  public void paintTopMiddle(String color) {
    // Moves to top middle
    turnLeft();
    move();
    turnLeft();
    moveFast();

    // Gets all of the paint from the paint bucket
    takeAllPaint();

    // Turns around
    turnAround();

    // Paints top center line
    paintLine(color);
  }

  /*
   * Paints the northeast diagonal line
   */
  public void paintNortheast(String color) {
    // Moves to top right corner
    turnLeft();
    moveFast();
    turnLeft();
    moveFast();

    // Gets all of the paint from the paint bucket
    takeAllPaint();

    // Turns around
    turnAround();

    // Paints a diagonal line while not on paint
    while (!isOnPaint()) {
      paintDiagonalSW(color);
    }
  }

  /*
   * Moves forward while SunPainter can move
   */
  public void moveFast() {
    while (canMove()) {
      move();
    }
  }

  /*
   * Turns the SunPainter around to face the opposite direction
   */
  public void turnAround() {
    turnLeft();
    turnLeft();
  }

  /*
   * Paints a straight line while SunPainter has paint
   */
  public void paintLine(String color) {
    while (hasPaint()) {
      paint(color);
      move();
    }
  }

  /*
   * Paints a diagonal line in the northwest direction
   */
  public void paintDiagonalNW(String color) {
    if (isFacingNorth()) {
      paint(color);
      move();
      turnLeft();
      move();
    }

    if (isFacingWest()) {
      paint(color);
      move();
      turnRight();
      move();
    }
  }

  /*
   * Paints a diagonal line in the northeast direction
   */
  public void paintDiagonalNE(String color) {
    if (isFacingNorth()) {
      paint(color);
      move();
      turnRight();
      move();
    }

    if (isFacingEast()) {
      paint(color);
      move();
      turnLeft();
      move();
    }
  }

  /*
   * Paints a diagonal line in the southeast direction
   */
  public void paintDiagonalSE(String color) {
    if (isFacingSouth()) {
      paint(color);
      move();
      turnLeft();
      move();
    }

    if (isFacingEast()) {
      paint(color);
      move();
      turnRight();
      move();
    }
  }

  /*
   * Paints a diagonal line in the southwest direction
   */
  public void paintDiagonalSW(String color) {
    if (isFacingSouth()) {
      paint(color);
      move();
      turnRight();
      move();
    }

    if (isFacingWest()) {
      paint(color);
      move();
      turnLeft();
      move();
    }
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the moveNoBucket() method to move while the SunPainter is not on a paint
   * bucket. Once the SunPainter is on a paint bucket, it should take all paint.
   * -----------------------------------------------------------------------------
   *
   * Moves forward while the SunPainter is not on a paint bucket,
   * then takes all of the paint from the paint bucket.
   */
  public void moveNoBucket() {
    
    


    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the paintLineAndTurn() method to paint a line then turn and move
   * based on the direction the SunPainter is currently facing.
   * -----------------------------------------------------------------------------
   *
   * Paints a line then turns based on the direction the SunPainter is facing
   */
  public void paintLineAndTurn(String color) {
    
    


    
  }
  
}`}],validationFiles:[{path:`SunPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import java.util.List;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SunPainter.java Test")
public class SunPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid.";
    message += messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @Test
  @Order(1)
  @SuppressWarnings("unchecked")
  @DisplayName("SunPainter is a subclass of the PainterPlus class => ")
  public void testSunPainterIsSubclassOfPainterPlus() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the SunPainter class is a subclass of the PainterPlus class.";
    message += messageGap;
      
    PainterPlus testPainter = new PainterPlus();
    Class sunPainterClass = null;

    try {
      sunPainterClass = Class.forName("SunPainter");
    } catch (Exception e) {
      message = "Your class header for SunPainter is either missing or the syntax is incorrect." + messageGap;
      fail(message);
    }
    
    Class painterPlusClass = testPainter.getClass();
    
    assertTrue(painterPlusClass.isAssignableFrom(sunPainterClass), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("Takes all of the paint from all of the paint buckets => ")
  public void testTakeAllPaintFromBuckets() {
    message = "Make sure to take all of the paint from the paint buckets when the SunPainter object is standing on one.";
    message += messageGap;
      
    assertTrue(primaryPainterLog.didActionExactly(NeighborhoodActionType.TAKE_PAINT, 53), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("Paints a mural of a sun in The Neighborhood => ")
  public void testPaintSunMural() {
    message = "Use if/else statements to move while not on a paint bucket and to paint and turn based on the direction the SunPainter is facing.";
    message += messageGap;
      
    boolean[][] expected = new boolean[16][16];

    expected[0] = new boolean[]{false, false, true, false, false, false, false, false, false, false, false, false, false, true, false, false};
    expected[1] = new boolean[]{false, false, false, true, false, false, false, false, false, false, false, false, true, false, false, false};
    expected[2] = new boolean[]{false, false, false, false, true, false, false, false, false, false, false, true, false, false, false, false};
    expected[3] = new boolean[]{false, false, false, false, false, true, true, true, true, true, true, false, false, false, false, false};
    expected[4] = new boolean[]{false, false, false, false, false, true, true, true, true, true, false, false, false, false, false, false};
    expected[5] = new boolean[]{false, false, true, true, true, true, true, true, true, true, true, true, true, true, false, false};
    expected[6] = new boolean[]{false, false, false, false, false, true, true, true, true, true, false, false, false, false, false, false};
    expected[7] = new boolean[]{false, false, false, false, false, true, true, true, true, true, true, false, false, false, false, false};
    expected[8] = new boolean[]{false, false, false, false, true, false, false, false, false, false, false, true, false, false, false, false};
    expected[9] = new boolean[]{false, false, false, true, false, false, false, false, false, false, false, false, true, false, false, false};
    expected[10] = new boolean[]{false, false, true, false, false, false, false, false, false, false, false, false, false, true, false, false};

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Asphalt Art Project #1`,lesson:`Lesson 15a: Asphalt Art Project`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #2`,lesson:`Lesson 15a: Asphalt Art Project`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #3`,lesson:`Lesson 15a: Asphalt Art Project`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #4`,lesson:`Lesson 15a: Asphalt Art Project`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #1`,lesson:`Lesson 15b: Asphalt Art Project [1-Day Version]`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #2`,lesson:`Lesson 15b: Asphalt Art Project [1-Day Version]`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #3`,lesson:`Lesson 15b: Asphalt Art Project [1-Day Version]`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Asphalt Art Project #4`,lesson:`Lesson 15b: Asphalt Art Project [1-Day Version]`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {



    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`With Paint Bucket`,lesson:`Sandbox: The Neighborhood`,view:`neighborhood`,grid:`1,0 1,10 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.Painter;

public class Main {
  public static void main(String[] args) {




  }
}`}],validationFiles:[],dataFiles:[]},{name:`No Paint Bucket`,lesson:`Sandbox: The Neighborhood`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.Painter;

public class Main {
  public static void main(String[] args) {




  }
}`}],validationFiles:[],dataFiles:[]}];export{e as LEVELS};