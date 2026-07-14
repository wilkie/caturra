var e=[{name:`Predict and Run: 2D Arrays`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[][] augustGrades = { {85, 90, 78},
                             {92, 87, 80},
                             {76, 89, 97} };

    Teacher msHamilton = new Teacher(augustGrades);

    System.out.println("\\nPrint First Row:");
    System.out.println(msHamilton.getStudentGrades(0));

    System.out.println("\\nPrint Second Row:");
    System.out.println(msHamilton.getStudentGrades(1));

    System.out.println("\\nPrint Third Row:");
    System.out.println(msHamilton.getStudentGrades(2));
  }
}`},{path:`Teacher.java`,text:`public class Teacher {

  private int[][] weeklyGrades;

  public Teacher(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int[][] getWeeklyGrades() {
    return weeklyGrades;
  }

  public void setWeeklyGrades(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int getGrade(int row, int col) {
    return weeklyGrades[row][col];
  }

  public void setGrade(int row, int col, int newValue) {
    weeklyGrades[row][col] = newValue;
  }

  public String getStudentGrades(int week) {
    String result = "";
    
    for (int index = 0; index < weeklyGrades[week].length; index++) {
      result += weeklyGrades[week][index] + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: Creating 2D Arrays (a)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a SocialMedia object
    SocialMedia analyst = new SocialMedia();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 2D array to store the array returned from calling the
     * setupData() method, then print the contents of the 2D array using the
     * dataToString() method in the SocialMedia class.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`SocialMedia.java`,text:`/*
 * Manages data about users on social media services
 */
public class SocialMedia {

  private int[][] numUsers;    // The 2D array containing the number of users for each service

  /*
   * Uses the setupData() method to initialize the 2D array numUsers
   */
  public SocialMedia() {
    this.numUsers = setupData();
  }

  /*
   * Returns the 2D array containing the number
   * of users for each social media service
   */
  public int[][] getNumUsers() {
    return numUsers;
  }

  /*
   * Declares, initializes, and returns a 2D int array
   */
  public int[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array that has 3 columns and 4 rows.
     * -----------------------------------------------------------------------------
     */

        
    return null;
  }

  /*
   * Returns a String containing the contents of the 2D array data
   */
  public String dataToString(int[][] data) {
    String result = "";

    for (int row = 0; row < data.length; row++) {
      for (int col = 0; col < data[0].length; col++) {
        result += data[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }

}`}],validationFiles:[{path:`SocialMediaTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SocialMedia.java Test")
public class SocialMediaTest {

  String messageGap = "\\n       ";
  String message;
  SocialMedia testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array is dataType[][] nameOfArray = new dataType[rows][cols]." + messageGap;
    testObject = new SocialMedia();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array with 3 columns => ")
  public void testSetupDataReturnsArrayWithThreeColumns() {
    int[][] expected = testObject.setupData();
    assertEquals(3, expected[0].length, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("setupData() returns a 2D array with 4 rows => ")
  public void testSetupDataReturnsArrayWithFourRows() {
    int[][] expected = testObject.setupData();
    assertEquals(4, expected.length, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Creating 2D Arrays (b)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a CarAccidents object
    CarAccidents safety = new CarAccidents();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 2D array to store the array returned from calling the
     * setupData() method, then print the contents of the 2D array using the
     * dataToString() method in the CarAccidents class.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`CarAccidents.java`,text:`/*
 * Manages data about car accidents
 */
public class CarAccidents {

  private int[][] accidents;    // The 2D array containing the number of major car accidents

  /*
   * Uses the setupData() method to initialize the 2D array accidents
   */
  public CarAccidents() {
    this.accidents = setupData();
  }

  /*
   * Returns the 2D array containing the number of major car accidents
   */
  public int[][] getAccidents() {
    return accidents;
  }

  /*
   * Declares, initializes, and returns a 2D int array
   */
  public int[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array that has 5 columns and 3 rows.
     * -----------------------------------------------------------------------------
     */

    
    return null;
  }

  /*
   * Returns a String containing the contents of the 2D array data
   */
  public String dataToString(int[][] data) {
    String result = "";

    for (int row = 0; row < data.length; row++) {
      for (int col = 0; col < data[0].length; col++) {
        result += data[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`CarAccidentsTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CarAccidents.java Test")
public class CarAccidentsTest {

  String messageGap = "\\n       ";
  String message;
  CarAccidents testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array is dataType[][] nameOfArray = new dataType[rows][cols]." + messageGap;
    testObject = new CarAccidents();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array with 5 columns => ")
  public void testSetupDataReturnsArrayWithFiveColumns() {
    int[][] expected = testObject.setupData();
    assertEquals(5, expected[0].length, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("setupData() returns a 2D array with 3 rows => ")
  public void testSetupDataReturnsArrayWithThreeRows() {
    int[][] expected = testObject.setupData();
    assertEquals(3, expected.length, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Creating 2D Arrays (c)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Teacher object
    Teacher msCook = new Teacher();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 2D array to store the array returned from calling the
     * setupData() method, then print the contents of the 2D array using the
     * dataToString() method in the Teacher class.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Teacher.java`,text:`/*
 * Manages data about student test scores
 */
public class Teacher {

  private int[][] scores;    // The 2D array containing test scores

  /*
   * Uses the setupData() method to initialize the 2D array scores
   */
  public Teacher() {
    this.scores = setupData();
  }

  /*
   * Returns the 2D array containing the test scores
   */
  public int[][] getScores() {
    return scores;
  }

  /*
   * Declares, initializes, and returns a 2D int array
   */
  public int[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array that has 3 columns and 5 rows.
     * -----------------------------------------------------------------------------
     */

    
    return null;
  }

  /*
   * Returns a String containing the contents of the 2D array data
   */
  public String dataToString(int[][] data) {
    String result = "";

    for (int row = 0; row < data.length; row++) {
      for (int col = 0; col < data[0].length; col++) {
        result += data[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`TeacherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Teacher.java Test")
public class TeacherTest {

  String messageGap = "\\n       ";
  String message;
  Teacher testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array is dataType[][] nameOfArray = new dataType[rows][cols]." + messageGap;
    testObject = new Teacher();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array with 3 columns => ")
  public void testSetupDataReturnsArrayWithThreeColumns() {
    int[][] expected = testObject.setupData();
    assertEquals(3, expected[0].length, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("setupData() returns a 2D array with 5 rows => ")
  public void testSetupDataReturnsArrayWithFiveRows() {
    int[][] expected = testObject.setupData();
    assertEquals(5, expected.length, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Creating 2D Arrays (d)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Researcher object
    Researcher data = new Researcher();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 2D array to store the array returned from calling the
     * setupData() method, then print the contents of the 2D array using the
     * dataToString() method in the Researcher class.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Researcher.java`,text:`/*
 * Manages data from the World Happiness Report
 */
public class Researcher {

  private double[][] happiness;    // The 2D array containing the data from the World Happiness Report

  /*
   * Uses the setupData() method to initialize the 2D array happiness
   */
  public Researcher() {
    this.happiness = setupData();
  }

  /*
   * Returns the 2D array containing World Happiness Report data
   */
  public double[][] getHappiness() {
    return happiness;
  }

  /*
   * Declares, initializes, and returns a 2D double array
   */
  public double[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array that has 4 columns and 6 rows.
     * -----------------------------------------------------------------------------
     */

    
    return null;
  }

  /*
   * Returns a String containing the contents of the 2D array data
   */
  public String dataToString(double[][] data) {
    String result = "";

    for (int row = 0; row < data.length; row++) {
      for (int col = 0; col < data[0].length; col++) {
        result += data[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`ResearcherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Researcher.java Test")
public class ResearcherTest {

  String messageGap = "\\n       ";
  String message;
  Researcher testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array is dataType[][] nameOfArray = new dataType[rows][cols]." + messageGap;
    testObject = new Researcher();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array with 4 columns => ")
  public void testSetupDataReturnsArrayWithFourColumns() {
    double[][] expected = testObject.setupData();
    assertEquals(4, expected[0].length, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("setupData() returns a 2D array with 6 rows => ")
  public void testSetupDataReturnsArrayWithSixRows() {
    double[][] expected = testObject.setupData();
    assertEquals(6, expected.length, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Using Initializer Lists (a)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Researcher object
    Researcher analyst = new Researcher();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 2D array to store the array returned from calling the
     * setupData() method, then print the contents of the 2D array using the
     * dataToString() method in the Researcher class.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Researcher.java`,text:`/*
 * Manages data about the cost of living in countries
 */
public class Researcher {

  private double[][] livingCost;    // The 2D array containing the cost of living index for each country

  /*
   * Uses the setupData() method to initialize the 2D array happiness
   */
  public Researcher() {
    this.livingCost = setupData();
  }

  /*
   * Returns the 2D array containing the cost
   * of living index for each country
   */
  public double[][] getLivingCost() {
    return livingCost;
  }

  /*
   * Declares, initializes, and returns a 2D double array
   */
  public double[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array containing the values for each country.
     * -----------------------------------------------------------------------------
     */

    
    return null;
  }

  /*
   * Returns a String containing the contents of the 2D array data
   */
  public String dataToString(double[][] data) {
    String result = "";

    for (int row = 0; row < data.length; row++) {
      for (int col = 0; col < data[0].length; col++) {
        result += String.format("%-8.2f", data[row][col]);
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`ResearcherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Researcher.java Test")
public class ResearcherTest {

  String messageGap = "\\n       ";
  String message;
  Researcher testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array with initializer lists is";
    message += "\\n        dataType[][] nameOfArray = { {row1Value1, row1Value2}, {row2Value2, row2Value2} };" + messageGap;
    testObject = new Researcher();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array containing the specified values => ")
  public void testSetupDataReturns2DArray() {
    double[][] expected = { {77.75, 36.84, 77.44, 72.95}, {33.24, 8.27, 28.16, 25.24}, {70.22, 34.33, 70.01, 67.86},
                            {74.13, 25.33, 73.64, 71.84}, {70.13, 42.07, 70.37, 70.07} };
    double[][] actual = testObject.setupData();
    assertArrayEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Using Initializer Lists (b)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates an Owner object
    Owner foodTruck = new Owner();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 2D array to store the array returned from calling the
     * setupData() method, then print the contents of the 2D array using the
     * dataToString() method in the Owner class.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Owner.java`,text:`/*
 * Manages data about the inventory for each food truck
 */
public class Owner {

  private int[][] inventories;    // The 2D array containing the inventory for each food truck

  /*
   * Uses the setupData() method to initialize the 2D array happiness
   */
  public Owner() {
    this.inventories = setupData();
  }

  /*
   * Returns the 2D array containing
   * inventories for each food truck
   */
  public int[][] getInventories() {
    return inventories;
  }

  /*
   * Declares, initializes, and returns a 2D int array
   */
  public int[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array containing the values for each food truck.
     * -----------------------------------------------------------------------------
     */

    
    return null;
  }

  /*
   * Returns a String containing the contents of the 2D array data
   */
  public String dataToString(int[][] data) {
    String result = "";

    for (int row = 0; row < data.length; row++) {
      for (int col = 0; col < data[0].length; col++) {
        result += String.format("%-4d", data[row][col]);
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`OwnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Owner.java Test")
public class OwnerTest {

  String messageGap = "\\n       ";
  String message;
  Owner testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array with initializer lists is";
    message += "\\n        dataType[][] nameOfArray = { {row1Value1, row1Value2}, {row2Value2, row2Value2} };" + messageGap;
    testObject = new Owner();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array containing the specified values => ")
  public void testSetupDataReturns2DArray() {
    int[][] expected = { {25, 17, 22}, {18, 12, 15}, {21, 29, 27}, {30, 10, 23} };
    int[][] actual = testObject.setupData();
    assertArrayEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Using Initializer Lists (c)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates an InstagramScene object
    InstagramScene scene = new InstagramScene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the setupData() method and pass the 2D array that is returned to a
     * call to the drawResults() method.
     * -----------------------------------------------------------------------------
     */


    
    
    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`InstagramScene.java`,text:`import org.code.theater.*;

/*
 * Creates a scene to show the an Instagram user
 * and their information
 */
public class InstagramScene extends Scene {

  private String[][] users;   // The 2D array of Instagram users

  /*
   * Uses the setupData() method to initialize the 2D array happiness
   */
  public InstagramScene() {
    this.users = setupData();
  }

  /*
   * Returns the 2D array of Instagram users
   */
  public String[][] getUsers() {
    return users;
  }

  /*
   * Declares, initializes, and returns a 2D array
   */
  public String[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array containing the values for each user.
     * -----------------------------------------------------------------------------
     */


    
    return null;
  }

  /*
   * Returns the image file for the specified user
   */
  public String getUserImage(String user) {
    String userImage = "";

    if (user.equals("Instagram")) {
      userImage = "instagram.png";
    }
    else if (user.equals("Cristiano Ronaldo")) {
      userImage = "cristianoronaldo.jpg";
    }
    else if (user.equals("Ariana Grande")) {
      userImage = "arianagrande.png";
    }
    else if (user.equals("Dwayne Johnson")) {
      userImage = "dwaynejohnson.jpg";
    }
    else if (user.equals("Kylie Jenner")) {
      userImage = "kyliejenner.jpg";
    }

    return userImage;
  }

  /*
   * Draws the specified text at the given x and y
   * position, then pauses for 0.3 seconds
   */
  public void drawTextAndPause(String text, int xPos, int yPos) {
    drawText(text, xPos, yPos);
    pause(0.3);
  }

  /*
   * Creates a scene by drawing the image of the user
   * and text containing the information about the user
   */
  public void drawResults(String[][] users) {
    for (int row = 0; row < users.length; row++) {
      for (int col = 0; col < users[0].length; col++) {
        drawTextAndPause(users[row][0], 50, 50);
        drawTextAndPause("Followers: " + users[row][2], 200, 200);
        drawTextAndPause(users[row][3], 50, 350);
        
        String userImage = getUserImage(users[row][1]);
        drawImage(userImage, 20, 75, 125);
      }

      pause(1);
      clear("white");
    }
  }
  
}`}],validationFiles:[{path:`InstagramSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("InstagramScene.java Test")
public class InstagramSceneTest {

  String messageGap = "\\n       ";
  String message;
  InstagramScene testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array with initializer lists is";
    message += "\\n        dataType[][] nameOfArray = { {row1Value1, row1Value2}, {row2Value2, row2Value2} };" + messageGap;
    testObject = new InstagramScene();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array containing the specified values => ")
  public void testSetupDataReturns2DArray() {
    String[][] expected = { {"@instagram", "Instagram", "353", "Social media platform"},
                        {"@cristiano", "Cristiano Ronaldo", "224", "Footballer"},
                        {"@arianagrande", "Ariana Grande", "190", "Musician and actress"},
                        {"@therock", "Dwayne Johnson", "187", "Actor and professional wrestler"},
                        {"@kyliejenner", "Kylie Jenner", "181", "TV personality and businesswoman"} }; 
    String[][] actual = testObject.setupData();
    assertArrayEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: Using Initializer Lists (d)`,lesson:`Lesson 1: Two-Dimensional (2D) Arrays`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a CerealScene object
    CerealScene scene = new CerealScene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the setupData() method and pass the 2D array that is returned to a
     * call to the drawResults() method.
     * -----------------------------------------------------------------------------
     */



    
    
    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`CerealScene.java`,text:`import org.code.theater.*;

/*
 * Creates a scene to show the nutritional
 * information about cereals
 */
public class CerealScene extends Scene {

  private int[][] nutrition;   // The 2D array of nutritional values

  /*
   * Uses the setupData() method to initialize the 2D array happiness
   */
  public CerealScene() {
    this.nutrition = setupData();
  }

  /*
   * Returns the 2D array containing the nutritional values
   */
  public int[][] getNutrition() {
    return nutrition;
  }

  /*
   * Declares, initializes, and returns a 2D array
   */
  public int[][] setupData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a 2D array containing the nutritional information
     * for each cereal.
     * -----------------------------------------------------------------------------
     */

    
    return null;
  }

  /*
   * Returns the name of the cereal for the specified row
   */
  public String getCerealName(int row) {
    String name = "";

    if (row == 0) {
      name = "Apple Jacks";
    }
    else if (row == 1) {
      name = "Cheerios";
    }
    else if (row == 2) {
      name = "Lucky Charms";
    }
    else if (row == 3) {
      name = "Raisin Bran";
    }

    return name;
  }

  /*
   * Sets the color of the text based on the nutritional value
   */
  public void changeTextColor(int nutritionalValue) {
    if (nutritionalValue > 12) {
      setTextColor("red");
    }
    else if (nutritionalValue > 5) {
      setTextColor("orange");
    }
    else {
      setTextColor("green");
    }
  }

  /*
   * Returns a String containing the nutritional label and value
   */
  public String getNutritionalText(int col, int nutritionalValue) {
    String textToDraw = "";
    
    if (col == 0) {
      textToDraw = "Calories: ";
    }
    else if (col == 1) {
      textToDraw = "Protein: ";
    }
    else if (col == 2) {
      textToDraw = "Fat: ";
    }
    else if (col == 3) {
      textToDraw = "Sodium: ";
    }
    else if (col == 4) {
      textToDraw = "Fiber: ";
    }
    else if (col == 5) {
      textToDraw = "Carbs: ";
    }
    else {
      textToDraw = "Sugar: ";
    }

    textToDraw += nutritionalValue;
    return textToDraw;
  }

  /*
   * Draws the nutritional values for each cereal in the scene
   */
  public void drawResults(int[][] values) {
    for (int row = 0; row < values.length; row++) {
      setTextColor("black");
      
      drawText(getCerealName(row), 100, 50);
      int yPosition = 75;
      
      for (int col = 0; col < values[0].length; col++) {
        changeTextColor(values[row][col]);
        drawText(getNutritionalText(col, values[row][col]), 100, yPosition);
        yPosition += 25;
      }

      pause(1);
      clear("white");
    }
  }
  
}`}],validationFiles:[{path:`CerealSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CerealScene.java Test")
public class CerealSceneTest {

  String messageGap = "\\n       ";
  String message;
  CerealScene testObject;
   
  @BeforeEach
  public void setup() {
    message = "The syntax for creating a 2D array with initializer lists is";
    message += "\\n        dataType[][] nameOfArray = { {row1Value1, row1Value2}, {row2Value2, row2Value2} };" + messageGap;
    testObject = new CerealScene();
  }
   
  @Test
  @Order(1)
  @DisplayName("setupData() returns a 2D array containing the specified values => ")
  public void testSetupDataReturns2DArray() {
    int[][] expected = { {110, 2, 0, 125, 1, 11, 14}, {110, 6, 2, 290, 2, 17, 1},
                       {110, 2, 1, 180, 0, 12, 12}, {120, 3, 1, 210, 5, 14, 12} }; 
    int[][] actual = testObject.setupData();
    assertArrayEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: 2D Array Elements`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher
 */
public class Teacher {

  private int[][] weeklyGrades;

  public Teacher(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int[][] getWeeklyGrades() {
    return weeklyGrades;
  }

  public void setWeeklyGrades(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int getGrade(int row, int col) {
    return weeklyGrades[row][col];
  }

  public void setGrade(int row, int col, int newValue) {
    weeklyGrades[row][col] = newValue;
  }

  public String getStudentGrades(int week) {
    String result = "";
    
    for (int index = 0; index < weeklyGrades[week].length; index++) {
      result += weeklyGrades[week][index] + "\\n";
    }

    return result;
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: 2D Array Elements #1`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[][] augustGrades = { {85, 90, 78}, 
                             {92, 87, 80}, 
                             {76, 89, 97} };

    Teacher msHamilton = new Teacher(augustGrades);

    int currentStudent = msHamilton.getGrade(1, 1);
    System.out.println(currentStudent);

    msHamilton.setGrade(1, 1, 100);

    currentStudent = msHamilton.getGrade(1, 1);
    System.out.println(currentStudent);
    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */

  }
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher
 */
public class Teacher {

  private int[][] weeklyGrades;

  public Teacher(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int[][] getWeeklyGrades() {
    return weeklyGrades;
  }

  public void setWeeklyGrades(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int getGrade(int row, int col) {
    return weeklyGrades[row][col];
  }

  public void setGrade(int row, int col, int newValue) {
    weeklyGrades[row][col] = newValue;
  }

  public String getStudentGrades(int week) {
    String result = "";
    
    for (int index = 0; index < weeklyGrades[week].length; index++) {
      result += weeklyGrades[week][index] + "\\n";
    }

    return result;
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: 2D Array Elements #2`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher
 */
public class Teacher {

  private int[][] weeklyGrades;

  public Teacher(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int[][] getWeeklyGrades() {
    return weeklyGrades;
  }

  public void setWeeklyGrades(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int getGrade(int row, int col) {
    return weeklyGrades[row][col];
  }

  public void setGrade(int row, int col, int newValue) {
    weeklyGrades[row][col] = newValue;
  }

  public String getStudentGrades(int week) {
    String result = "";
    
    for (int index = 0; index < weeklyGrades[week].length; index++) {
      result += weeklyGrades[week][index] + "\\n";
    }

    return result;
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: 2D Array Elements #3`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[][] augustGrades = { {85, 90, 78}, {92, 87, 80}, {76, 89, 97} };

    Teacher msHamilton = new Teacher(augustGrades);

    int currentStudent = msHamilton.getGrade(1, 1);
    System.out.println(currentStudent);

    msHamilton.setGrade(1, 1, 100);

    currentStudent = msHamilton.getGrade(1, 1);
    System.out.println(currentStudent);

    System.out.println("\\nPrint First Row:");
    System.out.println(msHamilton.getStudentGrades(0));
    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    


    
    
  }
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher
 */
public class Teacher {

  private int[][] weeklyGrades;

  public Teacher(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int[][] getWeeklyGrades() {
    return weeklyGrades;
  }

  public void setWeeklyGrades(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  public int getGrade(int row, int col) {
    return weeklyGrades[row][col];
  }

  public void setGrade(int row, int col, int newValue) {
    weeklyGrades[row][col] = newValue;
  }

  public String getStudentGrades(int week) {
    String result = "";
    
    for (int index = 0; index < weeklyGrades[week].length; index++) {
      result += weeklyGrades[week][index] + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: 2D Array Elements (a)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of letters that make up a password
    String[][] password = { {"a", "l", "g"},
                            {"o", "r", "i"},
                            {"t", "h", "m"} };

    // Calls the Encryption object
    Encryption encrypt = new Encryption(password);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the swapLetters() method, then print the updated 2D array using
     * the passwordToString() method in the Encryption class.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Encryption.java`,text:`/*
 * Encrypts a password that is organized in a 2D array
 */
public class Encryption {

  private String[][] letters;     // The 2D array of letters that make up the password

  /*
   * Initializes letters to the 2D array of letters
   */
  public Encryption(String[][] letters) {
    this.letters = letters;
  }

  /*
   * Returns the 2D array of letters
   */
  public String[][] getLetters() {
    return letters;
  }

  /*
   * Swaps the letter at row 0 column 0 with the letter at
   * row 2 column 2 and swaps the letter at row 0 column 2
   * with the letter at row 2 column 0
   */
  public void swapLetters() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Modify the 2D array letters by swapping the letter at [0, 0] with the
     * letter at [2, 2], and swapping the letter at [0, 2] with the letter at [2, 0].
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns a String containing each letter in letters
   */
  public String passwordToString() {
    String result = "";

    for (int row = 0; row < letters.length; row++) {
      for (int col = 0; col < letters[0].length; col++) {
        result += letters[row][col];
      }
    }

    return result;
  }
  
}`}],validationFiles:[{path:`EncryptionTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Encryption.java Test")
public class EncryptionTest {

  String messageGap = "\\n       ";
  String[][] testPassword;
  String[][] expectedPassword;
  Encryption testObject;
   
  @BeforeEach
  public void setup() {   
    getRandomPassword();
    testObject = new Encryption(testPassword);

    expectedPassword = new String[testPassword.length][testPassword[0].length];
    getExpectedResult();
  }
   
  @Test
  @Order(1)
  @DisplayName("swapLetters() swaps the letter at  row 0 column 0 with the letter at row 2 column 2 => ")
  public void testSwapLettersSwapsFirstLetterSet() {
    String message = "The letter at row 0 column 0 should be moved to row 2 column 2, and the letter at";
    message += "\\n        row 2 column 2 should be moved to row 0 column 0. Use a temp variable to help";
    message += "\\n        swap the letters at these positions in the 2D array." + messageGap;

    testObject.swapLetters();
    
    assertEquals(expectedPassword[0][0], testPassword[0][0], message);
    assertEquals(expectedPassword[2][2], testPassword[2][2], message);
  }
   
  @Test
  @Order(2)
  @DisplayName("swapLetters() swaps the letter at row 0 column 2 with the letter at row 2 column 0 => ")
  public void testSwapLettersSwapsSecondLetterSet() {
    String message = "The letter at row 0 column 2 should be moved to row 2 column 0, and the letter at";
    message += "\\n        row 2 column 0 should be moved to row 0 column 2. Use a temp variable to help";
    message += "\\n        swap the letters at these positions in the 2D array." + messageGap;

    testObject.swapLetters();
    
    assertEquals(expectedPassword[0][2], testPassword[0][2], message);
    assertEquals(expectedPassword[2][0], testPassword[2][0], message);
  }

  private void getRandomPassword() {
    int rand = (int)(Math.random() * 4) + 1;

    if (rand == 1) {
      testPassword = new String[][]{{"b", "y", "t"}, {"e", "c", "o"}, {"d", "e", "s"}};
    }
    else if (rand == 2) {
      testPassword = new String[][]{{"c", "a", "c"}, {"h", "e", "a"}, {"b", "l", "e"}};
    }
    else if (rand == 3) {
      testPassword = new String[][]{{"d", "e", "b"}, {"u", "g", "g"}, {"i", "n", "g"}};
    }
    else {
      testPassword = new String[][]{{"e", "x", "e"}, {"c", "u", "t"}, {"i", "o", "n"}};
    }
  }

  private void getExpectedResult() {
    for (int row = 0; row < testPassword.length; row++) {
      for (int col = 0; col < testPassword[0].length; col++) {
        expectedPassword[row][col] = testPassword[row][col];
      }
    }
    
    String temp = expectedPassword[0][0];
    expectedPassword[0][0] = expectedPassword[2][2];
    expectedPassword[2][2] = temp;
    
    temp = expectedPassword[0][2];
    expectedPassword[0][2] = expectedPassword[2][0];
    expectedPassword[2][0] = temp;
  }

}`}],dataFiles:[]},{name:`Practice: 2D Array Elements (b)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of integers to represent a board
    int[][] numbers = { {5, 9, 3, 1},
                        {7, 2, 8, 6},
                        {4, 0, 1, 9},
                        {3, 5, 2, 8} };

    // Creates a Game object
    Game boardGame = new Game(numbers);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcScore() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Game.java`,text:`/*
 * Represents a board game
 */
public class Game {

  private int[][] board;    // The 2D array of integers that represent the board

  /*
   * Initializes board to the specified 2D array of integers
   */
  public Game(int[][] board) {
    this.board = board;
  }

  /*
   * Returns the 2D array board
   */
  public int[][] getBoard() {
    return board;
  }

  /*
   * Calculates and returns the sum of all four corners of the board
   */
  public int calcScore() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the sum of the four corners of the board.
     * -----------------------------------------------------------------------------
     */
    
    return -1;
  }
  
}`}],validationFiles:[{path:`GameTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Game.java Test")
public class GameTest {

  String messageGap = "\\n       ";
  int[][] testNumbers;
  Game testObject;
   
  @BeforeEach
  public void setup() {
    testNumbers = getRandomNumbers();
    testObject = new Game(testNumbers);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcScore() returns the sum of all four corners of the 2D array board => ")
  public void testCalcScoreReturnsSum() {
    String message = "The four corners of the board are at [0, 0], [0, 3], [3, 0], and [3][3]." + messageGap;
      
    int expected = testNumbers[0][0] + testNumbers[0][3] + testNumbers[3][0] + testNumbers[3][3];
    int actual = testObject.calcScore();

    assertEquals(expected, actual, message);
  }
   
  private int[][] getRandomNumbers() {
    int[][] temp = new int[4][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 10);
      }
    }

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: 2D Array Elements (c)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the number of views each day of the week
    int[][] views = { {235, 189, 75, 212, 325},
                      {417, 352, 287, 457, 316},
                      {197, 428, 0, 208, 201},
                      {270, 297, 174, 199, 167},
                      {117, 241, 308, 155, 278} };

    // Creates a Video object
    Video content = new Video(views);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the fillMissingData() method and print the updated 2D array using
     * the viewsToString() method in the Video class.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Video.java`,text:`/*
 * Represents a video on a streaming video service
 */
public class Video {

  private int[][] dailyViews;       // The 2D array containing the number of views each day of the week

  /*
   * Initializes dailyViews to the specified 2D array of views
   */
  public Video(int[][] dailyViews) {
    this.dailyViews = dailyViews;
  }

  /*
   * Returns the 2D array dailyViews
   */
  public int[][] getDailyViews() {
    return dailyViews;
  }

  /*
   * Replaces the center element in dailyViews with the
   * average of the element above and below the center
   * element and to the left and right of the center
   */
  public void fillMissingData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Set the middle element to the average of the elements above and below the
     * middle element and the elements to the left and right of the middle element.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns a String containing the values in dailyViews
   */
  public String viewsToString() {
    String result = "";

    for (int row = 0; row < dailyViews.length; row++) {
      result += "Week " + (row + 1) + ": ";
      
      for (int col = 0; col < dailyViews[0].length; col++) {
        result += String.format("%-4d", dailyViews[row][col]);
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`VideoTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Video.java Test")
public class VideoTest {

  String messageGap = "\\n       ";
  int[][] testViews;
  Video testObject;
   
  @BeforeEach
  public void setup() {
    testViews = getRandomViews();
    testObject = new Video(testViews);
  }
   
  @Test
  @Order(1)
  @DisplayName("fillMissingData() replaces the center element => ")
  public void testFillMissingDataReplacesCenterElement() {
    String message = "The center element of the 2D array can be found at row 2 column 2." + messageGap;
    testObject.fillMissingData();
    assertTrue(testViews[2][2] != 0, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("fillMissingData() sets the center element to the average of the elements above, below, left, and right => ")
  public void testCenterElementIsAverageOfSurroundElements() {
    String message = "The element above the center element can be found at row 1 column 2, and the element below is at";
    message += "\\n        row 2 column 1. The element to the left is at row 2 column 3, and the element to the right is";
    message += "\\n        row 3 column 2. Find the average of these elements and set the center to the result." + messageGap;

    int expected = (testViews[1][2] + testViews[2][1] + testViews[2][3] + testViews[3][2]) / 4;
    testObject.fillMissingData();

    assertEquals(expected, testViews[2][2], message);
  }

  private int[][] getRandomViews() {
    int[][] temp = new int[5][5];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 500) + 100;
      }
    }

    temp[2][2] = 0;
    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: 2D Array Elements (d)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the number of floors for each land plot
    int[][] floors = {{1, 2, 2},
                      {2, 5, 1},
                      {3, 1, 2}};

    // Creates a Community object
    Community downtown = new Community(floors);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the swapFloors() method, and print the updated 2D array using the
     * landToString() method in the Community class. Then call the setCommunityCenter()
     * method and print the updated 2D array using the landToString() method.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Community.java`,text:`/*
 * Represents a community
 */
public class Community {

  private int[][] landPlots;     // The 2D array containing the number of floors at each land plot

  /*
   * Initializes landPlots to the specified 2D array of land plots
   */
  public Community(int[][] landPlots) {
    this.landPlots = landPlots;
  }

  /*
   * Returns the 2D array of land plots
   */
  public int[][] getLandPlots() {
    return landPlots;
  }

  /*
   * Swaps the values in the first and last positions of
   * the first row in the 2D array landPlots
   */
  public void swapFloors() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Swap the first and last elements of the first row in landPlots.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Gets the sum of the first building in the first row and
   * the last building in the last row and assigns the result
   * to the center position of the 2D array landPlots
   */
  public void setCommunityCenter() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Set the center element in landPlots to the sum of the first element in
     * the first row and the last element in the last row.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns a String containing the number of floors for each
   * land plot in the 2D array landPlots
   */
  public String landToString() {
    String result = "";

    for (int row = 0; row < landPlots.length; row++) {
      for (int col = 0; col < landPlots[0].length; col++) {
        result += String.format("%-4d", landPlots[row][col]);
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`CommunityTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Community.java Test")
public class CommunityTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Community testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Community(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("swapFloors() swaps the first and last positions of the first row in landPlots => ")
  public void testSwapFloorsSwapsFirstAndLast() {
    String message = "The value at row 0 column 0 should be moved to row 0 column 2, and the value at";
    message += "\\n        row 0 column 2 should be moved to row 0 column 0. Use a temp variable to help";
    message += "\\n        swap the values at these positions in the 2D array." + messageGap;

    int[][] expected = getExpectedResult();
    testObject.swapFloors();

    assertArrayEquals(expected, testValues, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("setCommunityCenter() sets the center element to the sum of the first element in the first row and the last element in the last row => ")
  public void testSetCommunityCenterSetsCenterElement() {
    String message = "The center element can be found at [1, 1], the first element in the first row at [0, 0],";
    message+= "\\n        and the last element in the last row at [2, 2]. Set the center element to the sum of the other two elements." + messageGap;
      
    int expected = testValues[0][0] + testValues[2][2];
    testObject.setCommunityCenter();
    
    assertEquals(expected, testValues[1][1], message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[3][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 10) + 1;
      }
    }

    return temp;
  }

  private int[][] getExpectedResult() {
    int[][] temp = new int[3][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = testValues[row][col];
      }
    }

    int swap = temp[0][0];
    temp[0][0] = temp[0][2];
    temp[0][2] = swap;

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating and Using 2D Arrays (a)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of Pet objects
    Pet[][] pets = { {new Pet("Dog", 2), new Pet("Dog", 5), new Pet("Dog", 8)},
                     {new Pet("Hamster", 1), new Pet("Hamster", 2), new Pet("Hamster", 3)},
                     {new Pet("Bird", 5), new Pet("Bird", 7), new Pet("Bird", 12)} };

    // Creates an Adoption object
    Adoption petAdoptions = new Adoption(pets);

    // Prints the Adoption object
    System.out.println(petAdoptions);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the reversePets() method, then print the Adoption object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Adoption.java`,text:`/*
 * Represents a pet adoption center
 */
public class Adoption {

  private Pet[][] pets;      // The 2D array of Pet objects

  /*
   * Initializes pets to the specified 2D array of Pet objects
   */
  public Adoption(Pet[][] pets) {
    this.pets = pets;
  }

  /*
   * Returns the 2D array of Pet objects
   */
  public Pet[][] getPets() {
    return pets;
  }

  /*
   * Reverses the row located at index by swapping the first
   * and last elements in the row
   */
  public void reversePets(int index) {
    /* ----------------------------------------- TO DO -----------------------------------------
     * ✅ Reverse the row located at the parameter index by swapping the first and last elements.
     * -----------------------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns a String containing the information for
   * each Pet object in the 2D array pets
   */
  public String toString() {
    String result = "";

    for (int row = 0; row < pets.length; row++) {
      for (int col = 0; col < pets[0].length; col++) {
        result += pets[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }
  
}`},{path:`Pet.java`,text:`/*
 * Represents a pet
 */
public class Pet {

  private String type;    // The type of a pet
  private int age;       // The age of a pet

  /*
   * Sets type and age to the specified values
   */
  public Pet(String type, int age) {
    this.type = type;
    this.age = age;
  }

  /*
   * Returns the type of the pet
   */
  public String getType() {
    return type;
  }

  /*
   * Returns the age of the pet
   */
  public int getAge() {
    return age;
  }

  /*
   * Returns a String containing the type
   * and age of the pet
   */
  public String toString() {
    return type + ": " + age;
  }
  
}`}],validationFiles:[{path:`AdoptionTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Adoption.java Test")
public class AdoptionTest {

  String messageGap = "\\n       ";
  Pet[][] testPets;
  Adoption testObject;
   
  @BeforeEach
  public void setup() {
    testPets = new Pet[][]{ {new Pet("Dog", 2), new Pet("Dog", 5), new Pet("Dog", 8)},
                            {new Pet("Hamster", 1), new Pet("Hamster", 2), new Pet("Hamster", 3)},
                            {new Pet("Bird", 5), new Pet("Bird", 7), new Pet("Bird", 12)} }; 
    testObject = new Adoption(testPets);
  }
   
  @Test
  @Order(1)
  @DisplayName("reversePets() reverses the elements in the row at the parameter index in pets => ")
  public void testReversePetsSwapsFirstAndLast() {
    String message = "Reverse the row in pets at the parameter index by swapping the first and last elements in the row" + messageGap;

    int row = (int)(Math.random() * testPets.length);
    Pet[] expected = getExpectedResult(row);
    testObject.reversePets(row);

    assertArrayEquals(expected, testPets[row], message);
  }

  private Pet[] getExpectedResult(int row) {
    Pet[] temp = testPets[row];

    Pet swap = temp[0];
    temp[0] = temp[2];
    temp[2] = swap;
    
    swap = temp[1];
    temp[1] = temp[1];
    temp[1] = swap;

    return temp;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating and Using 2D Arrays (b)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of daily profits
    double[][] profits = { {234.50, 1567.80, -350.25, 890.10},
                           {2345.67, -1200.00, 450.50, 1234.56} };

    // Creates an Owner object
    Owner foodTruck = new Owner(profits);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcDifference() method and print the result.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Owner.java`,text:`/*
 * Represents an owner of a food truck
 */
public class Owner {

  private double[][] profits;     // The 2D array containing the daily profits each week

  /*
   * Initializes profits to the specified 2D array of daily profits
   */
  public Owner(double[][] profits) {
    this.profits = profits;
  }

  /*
   * Returns the 2D array of daily profits
   */
  public double[][] getProfits() {
    return profits;
  }

  /*
   * Returns the difference between the total profits the
   * first week and the total profits the second week
   */
  public double calcDifference() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate the sum of the elements in the first row and the sum of the
     * elements in the second row, and return the difference between the two.
     * -----------------------------------------------------------------------------
     */

    
    return -1;
  }
  
}`}],validationFiles:[{path:`OwnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Owner.java Test")
public class OwnerTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  Owner testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Owner(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcDifference() returns the difference between the first and second week's profits => ")
  public void testCalcDifferenceReturnsDifference() {
    String message = "Find the total of each element in the first row and the total of each element in";
    message += "\\n        the second row. Return the result of the first total minus the second total." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcDifference();

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[2][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 300 + 100;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double[] firstWeek = testValues[0];
    double[] secondWeek = testValues[1];
    
    double firstTotal = 0;
    double secondTotal = 0;

    for (int index = 0; index < firstWeek.length; index++) {
      firstTotal += firstWeek[index];
    }

    for (int index = 0; index < secondWeek.length; index++) {
      secondTotal += secondWeek[index];
    }

    return firstTotal - secondTotal;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating and Using 2D Arrays (c)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the costt of living index for multiple countries
    double[][] values = { {70.22, 84.12, 74.13},
                          {24.43, 66.47, 77.03},
                          {70.13, 30.71, 69.65} };

    // Creates a Research object
    Research analyst = new Research(values);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverage() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Research.java`,text:`/*
 * Analyzes data about the cost of living in different countries
 */
public class Research {

  private double[][] costOfLiving;     // The 2D array containing the cost of living index for multiple countries

  /*
   * Initializes costOfLiving to the specified 2D array containing
   * the cost of living index for multiple countries
   */
  public Research(double[][] costOfLiving) {
    this.costOfLiving = costOfLiving;
  }

  /*
   * Returns the costOfLiving 2D array
   */
  public double[][] getCostOfLiving() {
    return costOfLiving;
  }

  /*
   * Calculates and return the average of the column
   * at the parameter columnIndex
   */
  public double calcAverage(int columnIndex) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Add each element in the column columnIndex, divide by the length 
     *    of the column, and return the result.
     * -----------------------------------------------------------------------------
     */

    return -1;
  }

}`}],validationFiles:[{path:`ResearchTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Research.java Test")
public class ResearchTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  Research testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Research(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcAverage() returns the average of the values in columnIndex => ")
  public void testCalcAverageReturnsAverageOfColumn() {
    String message = "Add up all the elements in costOfLiving in the column at the parameter columnIndex." + messageGap;

    int col = (int)(Math.random() * 3);
    double expected = (testValues[0][col] + testValues[1][col] + testValues[2][col]) / testValues[0].length;
    double actual = testObject.calcAverage(col);

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[3][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 100;
      }
    }

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating and Using 2D Arrays (d)`,lesson:`Lesson 2: 2D Array Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of weekly tasks
    String[][] weeklyTasks = { {"Read chapter in a textbook and take notes", "Complete a math worksheet"},
                               {"Write research paper", "Participate in a group presentation"},
                               {"Create flashcards to study for test", "Work on a coding project"} };

    // Creates a Student object
    Student maria = new Student(weeklyTasks);

    // Prints the Student object
    System.out.println(maria);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the swapWeeks() method, then print the updated Student object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Student.java`,text:`/*
 * Represents a student
 */
public class Student {

  private String[][] tasks;   // The 2D array of weekly tasks

  /*
   * Initializes tasks to the specified 2D array of weekly tasks
   */
  public Student(String[][] tasks) {
    this.tasks = tasks;
  }

  /*
   * Returns the 2D array of weekly tasks
   */
  public String[][] getTasks() {
    return tasks;
  }

  /*
   * Swaps the first and last rows of tasks
   */
  public void swapWeeks() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Swap the first and last rows of the 2D array tasks.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns a String containing each task in tasks
   */
  public String toString() {
    String result = "";

    for (int row = 0; row < tasks.length; row++) {
      result += "Week " + (row + 1) + ": ";
      
      for (int col = 0; col < tasks[0].length; col++) {
        result += tasks[row][col] + ", ";
      }

      result += "\\n";
    }

    return result;
  }
  
  
}`}],validationFiles:[{path:`StudentTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Student.java Test")
public class StudentTest {

  String messageGap = "\\n       ";
  String[][] testTasks;
  Student testObject;
   
  @BeforeEach
  public void setup() {
    testTasks = new String[][]{ {"Read chapter in a textbook and take notes", "Complete a math worksheet"},
                                {"Write research paper", "Participate in a group presentation"},
                                {"Create flashcards to study for test", "Work on a coding project"} };
    testObject = new Student(testTasks);
  }
   
  @Test
  @Order(1)
  @DisplayName("swapWeeks() moves the elements in the first row to the last row => ")
  public void testSwapWeeksFirstRow() {
    String message = "Remember that each row in a 2D array is a 1D array. Swap the 1D array in the";
    message += "\\n        first row to the last row." + messageGap;

    String[] expected = {"Read chapter in a textbook and take notes", "Complete a math worksheet"};
    testObject.swapWeeks();
    
    assertArrayEquals(expected, testTasks[2], message);
  }
   
  @Test
  @Order(2)
  @DisplayName("swapWeeks() moves the elements in the last row to the first row => ")
  public void testSwapWeeksLastRow() {
    String message = "Remember that each row in a 2D array is a 1D array. Swap the 1D array in the";
    message += "\\n        last row to the first row." + messageGap;
      
    String[] expected = {"Create flashcards to study for test", "Work on a coding project"};
    testObject.swapWeeks();

    assertArrayEquals(expected, testTasks[0], message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Nested Loops and 2D Arrays`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[][] weeklyGrades = { {85, 90, 78, 91}, {92, 87, 80, 95}, {76, 89, 97, 93} };

    Teacher msMitchell = new Teacher(weeklyGrades);

    System.out.println(msMitchell.getGrades());
    
  }
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher that manages weekly grades for students
 */
public class Teacher {

  private int[][] weeklyGrades;    // Stores the weekly grades for the teacher's students

  /*
   * Constructor to create a Teacher object with
   * the specified 2D array of weekly student grades
   */
  public Teacher(int[][] weeklyGrades) {
    this.weeklyGrades = weeklyGrades;
  }

  /*
   * Returns the 2D array of weekly student grades
   */
  public int[][] getWeeklyGrades() {
    return weeklyGrades;
  }

  /*
   * Returns a String containing the values in the weeklyGrades array
   */
  public String getGrades() {
    String result = "";
    
    for (int row = 0; row < weeklyGrades.length; row++) {
      for (int col = 0; col < weeklyGrades[0].length; col++) {
        result += weeklyGrades[row][col] + " ";
      }
      result += "\\n";
  }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (a) #1`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of number of ratings received for several movies
    int[][] dailyRatings = { {43, 68, 112, 29},
                             {85, 76, 91, 54},
                             {39, 121, 72, 93},
                             {56, 81, 98, 65} };

    // Creates a Streaming object
    Streaming movies = new Streaming(dailyRatings);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcTotalRatings() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Streaming.java`,text:`/*
 * Represents a movie streaming platform
 */
public class Streaming {

  private int[][] ratings;      // The 2D array of the number of ratings received for several movies

  /*
   * Initializes ratings to the specified 2D array of the number of ratings received
   */
  public Streaming(int[][] ratings) {
    this.ratings = ratings;
  }

  /*
   * Returns the 2D array ratings
   */
  public int[][] getRatings() {
    return ratings;
  }

  /*
   * Returns the total ratings received
   */
  public int calcTotalRatings() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array ratings in row-major order. Calculate and return
     * the sum of all the values in ratings.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`StreamingTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Streaming.java Test")
public class StreamingTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Streaming testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Streaming(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcTotalRatings() returns the sum of all values in the 2D array ratings => ")
  public void testCalcTotalRatings() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Then return the total." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.calcTotalRatings();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 200) + 50;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int total = 0;

    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }

    return total;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (b) #1`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of the sales for multiple departments each day
    double[][] salesData = { {1500.50, 1600.75, 1200.25, 1400.90, 1700.25, 1800.50, 1900.75},
                             {1000.25, 900.50, 1100.75, 1200.90, 1000.25, 800.75, 900.50},
                             {800.90, 700.25, 600.50, 500.75, 400.25, 300.90, 200.50},
                             {2000.75, 1900.50, 1800.25, 1700.90, 2000.50, 2100.75, 2200.25},
                             {3500.25, 3200.90, 2800.50, 3100.75, 4000.25, 4200.50, 4500.75} };

    // Creates a Store object
    Store deptStore = new Store(salesData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcTotalSales() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Store.java`,text:`/*
 * Represents a department store
 */
public class Store {

  private double[][] sales;       // The 2D array containing the sales for each department in a week

  /*
   * Initializes sales to the specified 2D array of sales
   */
  public Store(double[][] sales) {
    this.sales = sales;
  }

  /*
   * Returns the 2D array sales
   */
  public double[][] getSales() {
    return sales;
  }

  /*
   * Returns the total sales
   */
  public double calcTotalSales() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array sales in row-major order. Calculate and return
     * the sum of all the values in sales.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`StoreTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Store.java Test")
public class StoreTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  Store testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Store(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcTotalSales() returns the sum of all values in the 2D array sales => ")
  public void testCalcTotalSales() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Then return the total." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcTotalSales();

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 200 + 50;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double total = 0;

    for (double[] row : testValues) {
      for (double value : row) {
        total += value;
      }
    }

    return total;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (c) #1`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of points scored by several teams
    int[][] pointsData = { {85, 90, 75, 80, 95, 88, 92},
                           {70, 75, 80, 85, 90, 95, 100},
                           {95, 85, 90, 87, 92, 80, 85},
                           {80, 85, 75, 78, 87, 90, 92},
                           {90, 95, 85, 88, 93, 80, 85} };

    // Creates a League object
    League sports = new League(pointsData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcTotalScores() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
 
  }
}`},{path:`League.java`,text:`/*
 * Represents a sports league
 */
public class League {

  private int[][] points;     // The total points scored by all teams

  /*
   * Initializes points to the specified 2D array of points
   */
  public League(int[][] points) {
    this.points = points;
  }

  /*
   * Returns the 2D array points
   */
  public int[][] getPoints() {
    return points;
  }

  /*
   * Returns the total points scored by all teams
   */
  public int calcTotalScores() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array points in row-major order. Calculate and return
     * the sum of all the values in points.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`LeagueTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("League.java Test")
public class LeagueTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  League testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new League(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcTotalScores() returns the sum of all values in the 2D array points => ")
  public void testCalcTotalScores() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Then return the total." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.calcTotalScores();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 200) + 50;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int total = 0;

    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }

    return total;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (d) #1`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the number of vehicles
    int[][] trafficData = { {500, 700, 650, 750, 800, 850, 900},
                            {350, 450, 550, 650, 750, 850, 950},
                            {800, 750, 700, 650, 600, 550, 500},
                            {600, 550, 700, 750, 800, 650, 600},
                            {900, 850, 800, 750, 700, 650, 600},
                            {450, 550, 650, 750, 850, 950, 1050} };

    // Creates a Traffic object
    Traffic city = new Traffic(trafficData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcTotalVehicles() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Traffic.java`,text:`/*
 * Analyzes traffic data
 */
public class Traffic {

  private int[][] vehicles;    // The 2D array containing the number of vehicles

  /*
   * Initializes vehicles to the specified 2D array of vehicles
   */
  public Traffic(int[][] vehicles) {
    this.vehicles = vehicles;
  }

  /*
   * Returns the 2D array vehicles
   */
  public int[][] getVehicles() {
    return vehicles;
  }

  /*
   * Returns the total number of vehicles that passed
   * through all intersections
   */
  public int calcTotalVehicles() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array vehicles in row-major order. Calculate and return
     * the sum of all the values in vehicles.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`TrafficTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Traffic.java Test")
public class TrafficTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Traffic testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Traffic(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcTotalVehicles() returns the sum of all values in the 2D array vehicles => ")
  public void testCalcTotalVehicles() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Then return the total." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.calcTotalVehicles();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 200) + 50;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int total = 0;

    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }

    return total;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (a) #2`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of student test scores
    int[][] testScores = { {85, 90, 80, 70},
                           {90, 80, 75, 85},
                           {70, 75, 80, 90},
                           {80, 90, 85, 75},
                           {75, 70, 80, 85} };

    // Creates a Teacher object
    Teacher msSanchez = new Teacher(testScores);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageScores() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

  }
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher
 */
public class Teacher {

  private int[][] testScores;    // The 2D array of test scores for several students

  /*
   * Initializes testScores to the specified 2D array of test scores
   */
  public Teacher(int[][] testScores) {
    this.testScores = testScores;
  }

  /*
   * Returns the 2D array testScores
   */
  public int[][] getTestScores() {
    return testScores;
  }

  /*
   * Returns the average of all test scores
   */
  public double calcAverageScores() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array testScores in row-major order. Calculate and return
     * the average of all the values in testScores by dividing the total by the
     * number of rows times the number of columns.
     * -----------------------------------------------------------------------------
     */
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`TeacherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Teacher.java Test")
public class TeacherTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Teacher testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Teacher(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcAverageScores() returns the average of all values in the 2D array testScores => ")
  public void testCalcAverageScores() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Return the result of the total divided by the number of rows times the number";
    message += "\\n        of columns (nameOfArray.length * nameOfArray[0].length)." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcAverageScores();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 100) + 50;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double total = 0;

    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }

    return total / (testValues.length * testValues[0].length);
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (b) #2`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of customer satisfaction ratings for several products
    int[][] ratingsData = { {4, 5, 3, 4, 5, 4, 5, 3, 5, 4, 5, 3, 5, 4, 5, 3, 4, 5, 4, 5},
                            {3, 4, 5, 4, 3, 4, 5, 4, 3, 4, 5, 4, 3, 4, 5, 4, 3, 4, 5, 4},
                            {5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4},
                            {4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5, 4, 5},
                            {3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4} };

    // Creates a Product object
    Product electronics = new Product(ratingsData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageRating() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Product.java`,text:`/*
 * Represents a product
 */
public class Product {

  private int[][] ratings;    // The 2D array of customer satisfaction ratings

  /*
   * Initializes ratings to the specified 2D array of customer satisfaction ratings
   */
  public Product(int[][] ratings) {
    this.ratings = ratings;
  }

  /*
   * Returns the 2D array ratings
   */
  public int[][] getRatings() {
    return ratings;
  }

  /*
   * Returns the average of all customer satisfaction ratings
   */
  public double calcAverageRating() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array ratings in row-major order. Calculate and return
     * the average of all the values in ratings by dividing the total by the
     * number of rows times the number of columns.
     * -----------------------------------------------------------------------------
     */
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`ProductTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Product.java Test")
public class ProductTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Product testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Product(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcAverageRating() returns the average of all values in the 2D array ratings => ")
  public void testCalcAverageRating() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Return the result of the total divided by the number of rows times the number";
    message += "\\n        of columns (nameOfArray.length * nameOfArray[0].length)." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcAverageRating();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 5) + 1;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double total = 0;

    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }

    return total / (testValues.length * testValues[0].length);
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (c) #2`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the performance metrics for several employees
    int[][] performanceData = { {8, 7, 9, 6, 8, 7, 9},
                                {9, 9, 8, 8, 7, 7, 9},
                                {7, 8, 6, 7, 9, 8, 7},
                                {6, 7, 8, 8, 7, 6, 9},
                                {9, 8, 7, 6, 9, 8, 7},
                                {7, 6, 9, 8, 7, 8, 9} };

    // Creates a Manager object
    Manager julie = new Manager(performanceData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageMetric() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Manager.java`,text:`/*
 * Represents a manager
 */
public class Manager {

  private int[][] performance;    // The 2D array of performance metrics for several employees

  /*
   * Initializes performance to the specified 2D array performance metrics
   */
  public Manager(int[][] performance) {
    this.performance = performance;
  }

  /*
   * Returns the 2D array performance
   */
  public int[][] getPerformance() {
    return performance;
  }

  /*
   * Returns the average of all performance metrics
   */
  public double calcAverageMetric() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array performance in row-major order. Calculate and return
     * the average of all the values in performance by dividing the total by the
     * number of rows times the number of columns.
     * -----------------------------------------------------------------------------
     */
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`ManagerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Manager.java Test")
public class ManagerTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Manager testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Manager(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcAverageMetric() returns the average of all values in the 2D array performance => ")
  public void testCalcAverageMetric() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Return the result of the total divided by the number of rows times the number";
    message += "\\n        of columns (nameOfArray.length * nameOfArray[0].length)." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcAverageMetric();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 10) + 1;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double total = 0;

    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }

    return total / (testValues.length * testValues[0].length);
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing 2D Arrays (d) #2`,lesson:`Lesson 3: Row-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the AQI values for a city
    double[][] aqiValues = { {49.1, 52.3, 45.6, 60.2, 63.4, 56.8, 54.7},
                             {58.2, 62.4, 70.1, 66.5, 58.7, 51.9, 59.0},
                             {63.7, 70.3, 69.8, 67.6, 64.9, 63.2, 65.1},
                             {60.0, 59.2, 61.3, 62.8, 59.3, 60.5, 57.8},
                             {56.1, 63.5, 59.9, 53.2, 50.6, 46.8, 55.3} };

    // Creates a City object
    City home = new City(aqiValues);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageAQI() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`City.java`,text:`/*
 * Analyzes data about air quality in a city
 */
public class City {

  private double[][] airQuality;    // The 2D array of AQI values for each day

  /*
   * Initializes airQuality to the specified 2D array of AQI values
   */
  public City(double[][] airQuality) {
    this.airQuality = airQuality;
  }

  /*
   * Returns the 2D array airQuality
   */
  public double[][] getAirQuality() {
    return airQuality;
  }

  /*
   * Returns the average of all AQI values
   */
  public double calcAverageAQI() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array airQuality in row-major order. Calculate and return
     * the average of all the values in airQuality by dividing the total by the
     * number of rows times the number of columns.
     * -----------------------------------------------------------------------------
     */
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`CityTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("City.java Test")
public class CityTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  City testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new City(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcAverageAQI() returns the average of all values in the 2D array airQuality => ")
  public void testCalcAverageAQI() {
    String message = "Traverse the 2D array in row-major order, and add each value to a total variable.";
    message += "\\n        Return the result of the total divided by the number of rows times the number";
    message += "\\n        of columns (nameOfArray.length * nameOfArray[0].length)." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcAverageAQI();

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[5][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 100 + 40;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double total = 0;

    for (double[] row : testValues) {
      for (double value : row) {
        total += value;
      }
    }

    return total / (testValues.length * testValues[0].length);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Column-Major Traversal`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[][] alphabet = {{"A", "B", "C"},
                           {"D", "E", "F"},
                           {"G", "H", "I"}};

    for (int col = 0; col < alphabet[0].length; col++) {
      for (int row = 0; row < alphabet.length; row++) {
        System.out.print(alphabet[row][col] + " ");
      }

      System.out.println();
    }
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Column-Major Traversal (a) #1`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of dates
    String[] dates = {"March 29th", "March 30th", "March 31st", "April 1st", "April 2nd"};

    // Creates a 2D array containing the daily temperatures for several cities
    double[][] cityTemps = {{37.27, 40.66, 53.53, 52.36, 48.87},
                            {62.2, 61.47, 60.31, 62.42, 62.38},
                            {81.5, 80.04, 81.61, 85.91, 85.69},
                            {56.41, 48.6, 51.67, 48.72, 52.65}};

    // Creates a Temperatures object
    Temperatures cities = new Temperatures(dates, cityTemps);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the compareConsecutiveCities() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Temperatures.java`,text:`/*
 * Analyzes temperature data for several cities and dates
 */
public class Temperatures {

  private String[] dates;         // The 1D array of dates
  private double[][] cityTemps;   // The 2D array of temperatures for several cities

  /*
   * Initializes dates to the specified 1D array containing the dates
   * for each column of cityTemps and initializes cityTemps to the
   * specified 2D array of temperatures for several cities
   */
  public Temperatures(String[] dates, double[][] cityTemps) {
    this.dates = dates;
    this.cityTemps = cityTemps;
  }

  /*
   * Returns the 1D array of dates
   */
  public String[] getDates() {
    return dates;
  }

  /*
   * Returns the 2D array of temperatures
   */
  public double[][] getCityTemps() {
    return cityTemps;
  }

  /*
   * Compares consecutive city temperatures for each day in cityTemps
   */
  public String compareConsecutiveCities() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the cityTemps array in column-major order.
     *  Compare the temperature of each city with the next city in the same column.
     *  Return a String containing comparisons for each day.
     * -----------------------------------------------------------------------------
     */

    

    return "";
  }

  /*
   * Returns the date for a given column
   */
  public String getDate(int column) {
    if (column < dates.length) {
      return dates[column];
    }
    else {
      return "Unknown";
    }
  }
}
`}],validationFiles:[{path:`TemperaturesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Temperatures.java Test")
public class TemperaturesTest {

  String messageGap = "\\n       ";
  String[] testDates;
  double[][] testTemps;
  Temperatures testObject;
   
  @BeforeEach
  public void setup() {
    testDates = getRandomDates();
    testTemps = getRandomTemps();
    testObject = new Temperatures(testDates, testTemps);
  }
   
  @Test
  @Order(1)
  @DisplayName("compareConsecutiveCities() returns temperature differences between consecutive cities for each day => ")
  public void testCompareConsecutiveCities() {
    String message = "For each column (day), iterate through rows in column-major order.";
    message += "\\n        Compare the temperature of each city with the next city in the column.";
    message += "\\n        Return a String containing these comparisons for each day, formatted properly." + messageGap;

    String expected = getExpectedResult();
    String actual = testObject.compareConsecutiveCities();

    assertEquals(expected, actual, message);
  }

  private String[] getRandomDates() {
    String[] months = {"January", "February", "March", "April", "May", "June",
                       "July", "August", "September", "October", "November", "December"};
    
    String[] temp = new String[5];

    for (int index = 0; index < temp.length; index++) {
      int randomIndex = (int)(Math.random() * months.length);
      temp[index] = months[randomIndex] + " " + (int)(Math.random() * 20 + 1);
    }

    return temp;
  }

  private double[][] getRandomTemps() {
    double[][] temp = new double[4][5];  // 4 cities, 5 days

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 90 + 30;
      }
    }

    return temp;
  }

  private String getExpectedResult() {
    StringBuilder result = new StringBuilder();

    for (int col = 0; col < testTemps[0].length; col++) { // Iterate through days
      result.append(testDates[col]).append(":\\n");

      for (int row = 0; row < testTemps.length - 1; row++) { // Compare consecutive cities
        double first = testTemps[row][col];
        double second = testTemps[row + 1][col];
        result.append(String.format("City %d vs City %d: %.2f°F difference\\n", row + 1, row + 2, Math.abs(first - second)));
      }

      result.append("\\n");
    }

    return result.toString();
  }
  
}
`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (b) #1`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of dog breeds
    String[] breeds = {"Beagle", "Dalmation", "Golden Retriever", "Pug"};

    // Creates a 2D array with the minimum and maximum heights of each breed
    int[][] heights = {{13, 15}, {19, 23}, {22, 24}, {10, 12}};

    // Creates a DogData object
    DogData familyDog = new DogData(breeds, heights);

    // Calls the findHeight() method and prints the result
    System.out.println(familyDog.findHeight(20));
    
  }
}`},{path:`DogData.java`,text:`/*
 * Analyzes data about dog breeds
 */
public class DogData {

  private String[] breeds;  // The 1D array of dog breeds
  private int[][] data;     // The 2D array of data to analyze

  /*
   * Initializes breeds to the specified 1D array of dog breeds,
   * and initializes data to the specified 2D array of dog data
   */
  public DogData(String[] breeds, int[][] data) {
    this.breeds = breeds;
    this.data = data;
  }

  /*
   * Returns a String containing the dog breed that has a minimum height and
   * the dog breed that has a maximum height less than or equal to maxHeight
   */
  public String findHeight(int maxHeight) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a String containing the dog breed that has a minimum height and
     * the dog breed that has a maximum height less than or equal to maxHeight.
     * -----------------------------------------------------------------------------
     */

    

    return "";
  }
  
}`}],validationFiles:[{path:`DogDataTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("DogData.java Test")
public class DogDataTest {

  String messageGap = "\\n       ";
  String[] testBreeds;
  int[][] testHeights;
  DogData testObject;
  String[] expectedBreeds;
  int[] expectedHeights;
   
  @BeforeEach
  public void setup() {
    testBreeds = getRandomBreeds();
    testHeights = getRandomHeights();
    testObject = new DogData(testBreeds, testHeights);
  }
   
  @Test
  @Order(1)
  @DisplayName("findHeight() returns a String containing the dog breeds with min or max heights less than 20 => ")
  public void testFindHeight() {
    String message = "Before the loop, create a new empty String. As you traverse the 2D array, check if the value";
    message += "\\n        is less than or equal to maxHeight and the column is 0. If so, concatenate the expected information";
    message += "\\n        to the String to return. If the value is less than or equal to maxHeight and the column is 1,";
    message += "\\n        concatenate the expected information to the String to return." + messageGap;

    getExpectedResults(20);
    String actual = testObject.findHeight(20);

    for (int index = 0; index < expectedBreeds.length; index++) {
      assertTrue(actual.contains(expectedBreeds[index]) && actual.contains(expectedHeights[index] + ""), message);
    }
  }

  private String[] getRandomBreeds() {
    String[] breeds = {"Australian Cattle Dog", "Bichon Frise", "Chow Chow", "Dalmation", "English Cocker Spaniel",
                       "French Bulldog", "Golden Retriever", "Irish Setter", "Keeshond", "Lhasa Apso"};
    
    String[] temp = new String[5];

    for (int index = 0; index < temp.length; index++) {
      int randomIndex = (int)(Math.random() * breeds.length);
      temp[index] = breeds[randomIndex];
    }

    return temp;
  }

  private int[][] getRandomHeights() {
    int[][] temp = new int[4][2];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 25 + 10);
      }
    }

    return temp;
  }

  private void getExpectedResults(int maxHeight) {
    int numResults = getNumResults(maxHeight);
    expectedBreeds = new String[numResults];
    expectedHeights = new int[numResults];
    int index = 0;
    
    for (int col = 0; col < testHeights[0].length; col++) {
      for (int row = 0; row < testHeights.length; row++) {
        if (testHeights[row][col] <= maxHeight && col == 0) {
          expectedBreeds[index] = testBreeds[col];
          expectedHeights[index] = testHeights[row][col];
          index++;
        }

        if (testHeights[row][col] <= maxHeight && col == 1) {
          expectedBreeds[index] = testBreeds[col];
          expectedHeights[index] = testHeights[row][col];
          index++;
        }
      }
    }
  }

  private int getNumResults(int maxHeight) {
    int count = 0;
    
    for (int col = 0; col < testHeights[0].length; col++) {
      for (int row = 0; row < testHeights.length; row++) {
        if (testHeights[row][col] <= maxHeight && col == 0) {
          count++;
        }

        if (testHeights[row][col] <= maxHeight && col == 1) {
          count++;
        }
      }
    }

    return count;
  }
  
}`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (c) — Theater`,lesson:`Lesson 4: Column-Major Traversal`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of social media icons
    String[] icons = {"facebook.png", "youtube.png", "twitter.png", "instagram.png"};

    // Creates a 1D array of age groups
    String[] ageGroups = {"18-29", "30-49", "50-64", "65 and up"};

    // Creates a 2D array containing the percentage of users in each age group for each social media app
    int[][] percentages = {{22, 29, 39, 47}, 
                           {41, 38, 18, 37},
                           {23, 21, 18, 12}, 
                           {15, 12, 6, 4}};

    // Creates a SocialMediaScene object
    SocialMediaScene scene = new SocialMediaScene(icons, ageGroups, percentages);

    // Draws the results in the scene
    scene.drawResults();

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`SocialMediaScene.java`,text:`import org.code.theater.*;

/*
 * Creates a scene based on which social media platform
 * has the largest percentage of users in an age group
 */
public class SocialMediaScene extends Scene {

  private String[] icons;        // The 1D array of social media icons
  private String[] ageGroups;    // The 1D array of age groups
  private int[][] percentages;   // The 2D array of age group percentages

  /*
   * Initializes icons and ageGroups to the specified 1D arrays of social media
   * icons and age groups, and initializes percentages to the specified 2D array
   * of age group percentages
   */
  public SocialMediaScene(String[] icons, String[] ageGroups, int[][] percentages) {
    this.icons = icons;
    this.ageGroups = ageGroups;
    this.percentages = percentages;
  }

  /*
   * Checks for duplicate percentages within a given social media platform (column)
   */
  public boolean checkForDuplicates(int platformIndex) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the percentages array in column-major order and compare each 
     * value to every other value in the same column. If any two values are equal, 
     * return true to indicate duplicates were found. Otherwise, return false.
     * -----------------------------------------------------------------------------
     */

    
    
    return false;
  }

/*
 * Creates a scene to show which social media platforms have duplicate percentages.
 */
public void drawResults() {   
    for (int platformIndex = 0; platformIndex < icons.length; platformIndex++) {
        boolean hasDuplicates = checkForDuplicates(platformIndex);

        drawImage(icons[platformIndex], 50, 50, 150);

        if (hasDuplicates) {
            setTextColor("red");  // Highlight text if duplicates exist
            drawText("Duplicate Found!", 250, 80);
        } else {
            setTextColor("black");  // Default text color
            drawText("No Duplicates", 250, 80);
        }

        pause(1);
        clear("white");
    }
}
  
}`}],validationFiles:[{path:`SocialMediaSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SocialMediaScene.java Test")
public class SocialMediaSceneTest {

  String messageGap = "\\n       ";
  String[] testAgeGroups;
  int[][] testPercentages;
  SocialMediaScene testObject;
   
  @BeforeEach
  public void setup() {
    String[] testIcons = new String[4];
    testAgeGroups = new String[]{"18-29", "30-49", "50-64", "65 and up"};
    testPercentages = new int[][]{{100,55,66,77},{91,24,78,44},{40,30,39,85},{40,54,32,13}};
    testObject = new SocialMediaScene(testIcons, testAgeGroups, testPercentages);
  }

  @Test
  @Order(1)
  @DisplayName("checkForDuplicates() returns true if there are duplicate percentages in a column => ")
  public void testCheckForDuplicates() {
      String message = "Traverse the percentages array in column-major order, comparing each value";
      message += "\\n        to every other value in the same column. If any two values are equal, return true.";
      message += "\\n        Otherwise, return false." + messageGap;
      
      // Check each column separately
      for (int col = 0; col < testPercentages[0].length; col++) {
          boolean expected = getExpectedDuplicateResult(col);
          boolean actual = testObject.checkForDuplicates(col);
          assertEquals(expected, actual, "Column " + col + " - " + message);
      }
  }
  
  // Adjust getExpectedDuplicateResult() to check for duplicates in a specific column
  private boolean getExpectedDuplicateResult(int column) {
      for (int row1 = 0; row1 < testPercentages.length; row1++) {
          for (int row2 = row1 + 1; row2 < testPercentages.length; row2++) {
              if (testPercentages[row1][column] == testPercentages[row2][column]) {
                  return true;
              }
          }
      }
      return false;
  }
}
`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (d) — Theater`,lesson:`Lesson 4: Column-Major Traversal`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates the Book objects
    Book fahrenheit = new Book("Fahrenheit 451", "fahrenheit.jpg", 10721);
    Book gobletoffire = new Book("Harry Potter and the Goblet of Fire", "gobletoffire.jpg", 7758);
    Book orwell = new Book("1984", "1984.jpg", 21424);
    Book agreements = new Book("The Four Agreements", "agreements.jpg", 23308);
    Book gratitude = new Book("The Book of Gratitude", "gratitude.jpg", 10141);
    Book winFriends = new Book("How to Win Friends and Influence People", "winfriends.jpg", 25001);


    // Stores the Book objects in a 2D array
    Book[][] bestsellers = {{fahrenheit, gobletoffire, orwell},
                            {agreements, gratitude, winFriends}};

    // Creates a BookScene object
    BookScene scene = new BookScene(bestsellers);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the shiftBooks() method to shift the books into proper alphabetical 
     * order. Then, call the createScene() method with the updated 2D array of books.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the BookScene
    Theater.playScenes(scene);

  }
}`},{path:`Book.java`,text:`/*
 * Represents a book
 */
public class Book {

  private String title;         // The title of the book
  private String bookCover;     // The image file of the book cover
  private int numReviews;       // The number of reviews

  /*
   * Sets title, bookCover, and numReviews to the specified values
   */
  public Book(String title, String bookCover, int numReviews) {
    this.title = title;
    this.bookCover = bookCover;
    this.numReviews = numReviews;
  }

  /*
   * Returns the title of the book
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the image file of the book cover
   */
  public String getBookCover() {
    return bookCover;
  }

  /*
   * Returns the number of reviews
   */
  public int getNumReviews() {
    return numReviews;
  }
  
}
`},{path:`BookScene.java`,text:`import org.code.theater.*;

/*
 * Creates a scene based on which books have at least 10,000 reviews
 */
public class BookScene extends Scene {

  private Book[][] bestsellers;      // The 2D array of Book objects

  /*
   * Initializes bestsellers to the specified 2D array of Book objects
   */
  public BookScene(Book[][] bestsellers) {
    this.bestsellers = bestsellers;
  }
  
  /*
   * Returns the 2D array of Book objects
   */
  public Book[][] getBestsellers() {
      return bestsellers;
  }

  /*
   * Shifts the books in the bestsellers array by a given shift amount 
   * to bring them into proper alphabetical order.
   */
  public void shiftBooks(int shiftAmount) {
      /* ----------------------------------- TO DO -----------------------------------
       * ✅ Shift the books in the bestsellers array to the left by shiftAmount positions.
       * Use a loop to move the first book to the end of the list repeatedly for the 
       * given shift amount. Maintain the 2D structure while shifting the elements.
       * -----------------------------------------------------------------------------
       */
  
      
  
  }

  /*
   * Creates a scene that draws the book cover for each book in the given array
   */
  public void createScene(Book[] books) {
      for (int index = 0; index < books.length; index++) {
          drawImage(books[index].getBookCover(), 50, 50, 250);
          pause(0.5);
          clear("white");
      }
  }

  
}`}],validationFiles:[{path:`BookSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BookScene.java Test")
public class BookSceneTest {

  String messageGap = "\\n       ";
  Book[][] testBooks;
  BookScene testObject;
   
  @BeforeEach
  public void setup() {
    testBooks = getInitialBooks();
    testObject = new BookScene(copyBooks(testBooks)); // Ensure original is preserved
  }
   
  @Test
  @Order(1)
  @DisplayName("shiftBooks() shifts books to the left by a specified amount in each row => ")
  public void testShiftBooks() {
    String message = "Traverse each row in the bestsellers array and shift books to the left by shiftAmount places.";
    message += "\\n        Use a loop to move the first book in each row to the end, ensuring a cyclic shift.";
    message += "\\n        Ensure that books wrap around correctly and that the order is preserved after shifting." + messageGap;
      
    int shiftAmount = 1;  // Example shift amount
    Book[][] expected = getExpectedShiftedBooks(shiftAmount);
    testObject.shiftBooks(shiftAmount);
    Book[][] actual = testObject.getBestsellers();

    assertTrue(arraysAreEqual(expected, actual), message);
  }

  private Book[][] getInitialBooks() {
    return new Book[][] {
      {new Book("1984", "1984.jpg", 21424), 
       new Book("Fahrenheit 451", "fahrenheit.jpg", 10721), 
       new Book("Goblet of Fire", "gobletoffire.jpg", 7758)},
       
      {new Book("The 4 Agreements", "agreements.jpg", 23308), 
       new Book("Gratitude", "gratitude.jpg", 10141), 
       new Book("How to Win Friends", "winfriends.jpg", 25001)}
    };
  }

  private Book[][] getExpectedShiftedBooks(int shiftAmount) {
      Book[][] shiftedBooks = copyBooks(getInitialBooks()); // Properly clone the original books
  
      for (int row = 0; row < shiftedBooks.length; row++) {
          for (int count = 0; count < shiftAmount; count++) {
              Book temp = shiftedBooks[row][0];
  
              // Shift all elements to the left within the row
              for (int col = 0; col < shiftedBooks[row].length - 1; col++) {
                  shiftedBooks[row][col] = shiftedBooks[row][col + 1];
              }
  
              // Move the first book to the last position in that row
              shiftedBooks[row][shiftedBooks[row].length - 1] = temp;
          }
      }
  
      return shiftedBooks;
  }

  private Book[][] copyBooks(Book[][] original) {
      Book[][] copy = new Book[original.length][original[0].length];
      for (int row = 0; row < original.length; row++) {
          for (int col = 0; col < original[row].length; col++) {
              // Create a new Book instance instead of copying references
              Book originalBook = original[row][col];
              copy[row][col] = new Book(originalBook.getTitle(), originalBook.getBookCover(), originalBook.getNumReviews());
          }
      }
      return copy;
  }

  private boolean arraysAreEqual(Book[][] expected, Book[][] actual) {
      for (int row = 0; row < expected.length; row++) {
          for (int col = 0; col < expected[row].length; col++) {
              if (!expected[row][col].getTitle().equals(actual[row][col].getTitle())) {
                  return false;
              }
          }
      }
      return true;
  }

}
`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (a) #2`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of visitors to several exhibits each day
    int[][] visitorData = { {120, 125, 130, 135, 140, 150, 155},
                            {100, 95, 105, 110, 115, 120, 130},
                            {70, 80, 85, 90, 95, 100, 110},
                            {50, 60, 70, 80, 90, 100, 110},
                            {200, 220, 240, 260, 280, 300, 320}};

    // Creates a Museum object
    Museum history = new Museum(visitorData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the allExhibitsAbove(int dayIndex, int target) 
     * method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Museum.java`,text:`/*
 * Represents a museum
 */
public class Museum {

  private int[][] visitors;    // The 2D array of visitors to a museum

  /*
   * Initializes visitors to the specified 2D
   * array of visitors to the museum
   */
  public Museum(int[][] visitors) {
    this.visitors = visitors;
  }

  /*
   * Returns the 2D array visitors
   */
  public int[][] getVisitors() {
    return visitors;
  }

  /*
   * Determines if all exhibits on a given day had more than the target number of visitors
   */
  public boolean allExhibitsAbove(int dayIndex, int target) {
      /* ----------------------------------- TO DO -----------------------------------
       * ✅ Loop through the column at dayIndex and check if all exhibits had more 
       * than target visitors. If any exhibit had fewer visitors, return false. Otherwise,
       * return true.
       * -----------------------------------------------------------------------------
       */


    
      return false;
  }

  
}`}],validationFiles:[{path:`MuseumTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Museum.java Test")
public class MuseumTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Museum testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getTestValues();
    testObject = new Museum(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("allExhibitsAbove() returns true if all exhibits on a given day had more than the target visitors => ")
  public void testAllExhibitsAbove() {
    String message = "Loop through the column at dayIndex and check if all exhibits had more";
    message += "\\n        than target visitors. If any exhibit had fewer visitors, return false." + messageGap;

    int testDay = 2;  // Arbitrary day index
    int target = 75;  // Arbitrary visitor threshold

    boolean expected = getExpectedResult(testDay, target);
    boolean actual = testObject.allExhibitsAbove(testDay, target);

    assertEquals(expected, actual, message);
  }

  private int[][] getTestValues() {
    return new int[][]{
      {100, 80, 90, 60, 85, 110, 75},
      {120, 95, 88, 65, 78, 140, 80},
      {130, 85, 95, 70, 90, 150, 82},
      {110, 78, 89, 55, 86, 125, 79},
      {105, 92, 97, 68, 91, 130, 81}
    };
  }

  private boolean getExpectedResult(int dayIndex, int target) {
    for (int row = 0; row < testValues.length; row++) {
      if (testValues[row][dayIndex] <= target) {
        return false;
      }
    }
    return true;
  }
}
`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (b) #2`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of seats on a plane
    boolean[][] seats = { {false, false, false},
                          {false, true, true},
                          {true, false, false},
                          {true, true, true},
                          {true, false, true},
                          {false, true, true} };

    // Creates a Plane object
    Plane toSeattle = new Plane(seats);

    // Prints the Plane object
    System.out.println(toSeattle);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the upgradeMiddleSeat() method and print the updated Plane object.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Plane.java`,text:`/*
 * Represents a plane
 */
public class Plane {

  private boolean[][] seats;    // The 2D array of seats on a plane

  /*
   * Initializes seats to the specified 2D array of seats
   */
  public Plane(boolean[][] seats) {
    this.seats = seats;
  }

  /*
   * Returns the 2D array of seats
   */
  public boolean[][] getSeats() {
    return seats;
  }

  /*
   * Upgrades the first occupied middle seat to a
   * window seat if one is available
   */
  public void upgradeMiddleSeat() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find the first occupied middle seat (column 1) and upgrade it to the first
     * available window seat.
     * -----------------------------------------------------------------------------
     */
    
    
    
  }

  /*
   * Returns the row with the first available window seat
   */
  public int getAvailableWindowSeat() {
    int availableSeat = -1;
    
    for (int row = 0; row < seats.length; row++) {
      if (!seats[row][0]) {
        return row;
      }
    }

    return availableSeat;
  }

  /*
   * Returns a String containing the status of each seat on the plane
   */
  public String toString() {
    String result = "";

    for (int row = 0; row < seats.length; row++) {
      result += "Row #" + (row + 1) + ": ";
      
      for (int col = 0; col < seats[0].length; col++) {
        if (col == 0) {
          result += "[W] ";
        }
        else if (col == 1) {
          result += "[M] ";
        }
        else {
          result += "[A] ";
        }

        result += String.format("%-6s", seats[row][col]);
      }

      result += "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`PlaneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Plane.java Test")
public class PlaneTest {

  String messageGap = "\\n       ";
  boolean[][] testValues;
  Plane testObject;
  boolean[][] expectedValues;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Plane(testValues);
    expectedValues = getCopy();
  }
   
  @Test
  @Order(1)
  @DisplayName("upgradeMiddleSeat() sets the first occupied middle seat to a window seat if one is available => ")
  public void testUpgradeMiddleSeat() {
    String message = "Traverse the 2D array seats in column-major order. Check if a seat is in the middle (column 1) and is";
    message += "\\n        occupied (true). If so, check if the window seat (column 0) is available (false). If this is also";
    message += "\\n        true, set the middle seat to false and the first available window seat to true." + messageGap;
      
    getExpectedResult();
    testObject.upgradeMiddleSeat();

    assertArrayEquals(expectedValues, testValues, message);
  }

  private boolean[][] getRandomValues() {
    boolean[][] temp = new boolean[6][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        int result = (int)(Math.random() * 2) + 1;

        if (result == 1) {
          temp[row][col] = true;
        }
        else {
          temp[row][col] = false;
        }
      }
    }

    return temp;
  }

  private boolean[][] getCopy() {
    boolean[][] temp = new boolean[6][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = testValues[row][col];
      }
    }

    return temp;
  }

  private void getExpectedResult() {
    boolean windowSeatFound = false;
    int availableSeat = testObject.getAvailableWindowSeat();

    if (availableSeat == -1) {
      windowSeatFound = true;
    }

    for (int col = 0; col < expectedValues[0].length; col++) {
      for (int row = 0; row < expectedValues.length; row++) {
        if (!windowSeatFound && expectedValues[row][col] && col == 1) {
          expectedValues[row][col] = false;
          expectedValues[availableSeat][0] = true;
          windowSeatFound = true;
        }
      }
    }
  }
   
}`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (c) — Console`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of reactions for several posts
    int[][] reactions = { {10, 5, 8, 12},
                          {3, 6, 9, 7},
                          {2, 1, 4, 5} };

    // Creates a SocialMedia object
    SocialMedia app = new SocialMedia(reactions);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findHighest() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`SocialMedia.java`,text:`/*
 * Represents a social media app
 */
public class SocialMedia {

  private int[][] reactions;      // The 2D array containing the number of reactions

  /*
   * Initializes reactions to the 2D array containing the number of reactions
   */
  public SocialMedia(int[][] reactions) {
    this.reactions = reactions;
  }

  /*
   * Returns the 2D array reactions
   */
  public int[][] getReactions() {
    return reactions;
  }

  /*
   * Returns the location of the post with the highest total engagement
   */
  public int findHighest() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array reactions in column-major order and find the sum
     * of each column. Return the column index of the column that has the largest
     * number of reactions.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`SocialMediaTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SocialMedia.java Test")
public class SocialMediaTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  SocialMedia testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new SocialMedia(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("findHighest() returns the column index that has the most number of reactions => ")
  public void testFindHighest() {
    String message = "Find the sum of each column and check if the sum is greater than a max sum. Return";
    message += "\\n        the column index of the column that has the largest total number of reactions." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.findHighest();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[3][4];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 20) + 5;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int maxReactions = 0;
    int postFound = -1;

    for (int col = 0; col < testValues[0].length; col++) {
      int numReactions = 0;
      
      for (int row = 0; row < testValues.length; row++) {
        numReactions += testValues[row][col];
      }

      if (numReactions > maxReactions) {
        maxReactions = numReactions;
        postFound = col;
      }
    }

    return postFound;
  }
  
}`}],dataFiles:[]},{name:`Practice: Column-Major Traversal (d) — Console`,lesson:`Lesson 4: Column-Major Traversal`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the daily energy consumption for several apartments
    double[][] energyConsumption = { {10.5, 8.2, 7.1, 12.0, 9.3, 11.1, 13.2},
                                     {5.3, 6.0, 7.9, 8.1, 9.2, 10.0, 11.6},
                                     {9.1, 11.0, 8.4, 7.3, 10.2, 12.1, 6.5},
                                     {8.9, 7.5, 6.6, 5.8, 9.1, 10.5, 11.9} };

    // Creates an Energy object
    Energy apts = new Energy(energyConsumption);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findMinEnergy() and findMaxEnergy() methods and print the results.
     * -----------------------------------------------------------------------------
     */


    

  }
}`},{path:`Energy.java`,text:`/*
 * Analyzes data about energy consumption
 */
public class Energy {

  private double[][] dailyEnergy;     // The 2D array of energy consumption for several apartments

  /*
   * Initializes dailyEnergy to the specified 2D array
   * of energy consumption for several apartments
   */
  public Energy(double[][] dailyEnergy) {
    this.dailyEnergy = dailyEnergy;
  }

  /*
   * Returns the 2D array dailyEnergy
   */
  public double[][] getDailyEnergy() {
    return dailyEnergy;
  }

  /*
   * Returns the day with the lowest total energy consumption
   */
  public int findMinEnergy() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array dailyEnergy and find the sum of each column. Return
     * the column index that has the lowest total energy consumption.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Returns the day with the highest total energy consumption
   */
  public int findMaxEnergy() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array dailyEnergy and find the sum of each column. Return
     * the column index that has the highest total energy consumption.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`EnergyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Energy.java Test")
public class EnergyTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  Energy testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Energy(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("findMinEnergy() returns the column index with the minimum total energy consumption => ")
  public void testFindMinEnergy() {
    String message = "Traverse the 2D array dailyEnergy in column-major order to find the sum of each column.";
    message += "\\n        Return the column index that has the lowest total energy consumption" + messageGap;
      
    int expected = getExpectedMin();
    int actual = testObject.findMinEnergy();

    assertEquals(expected, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("findMaxEnergy() returns the column index with the maximum total energy consumption => ")
  public void testFindMaxEnergy() {
    String message = "Traverse the 2D array dailyEnergy in column-major order to find the sum of each column.";
    message += "\\n        Return the column index that has the highest total energy consumption" + messageGap;
      
    int expected = getExpectedMax();
    int actual = testObject.findMaxEnergy();

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[4][7];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 30 + 10;
      }
    }

    return temp;
  }

  private int getExpectedMin() {
    int minDay = -1;
    double minEnergy = 1000;

    for (int col = 0; col < testValues[0].length; col++) {
      double totalEnergy = 0;
      
      for (int row = 0; row < testValues.length; row++) {
        totalEnergy += testValues[row][col];
      }

      if (totalEnergy < minEnergy) {
        minEnergy = totalEnergy;
        minDay = col;
      }
    }

    return minDay;
  }

  private int getExpectedMax() {
    int maxDay = -1;
    double maxEnergy = 0;

    for (int col = 0; col < testValues[0].length; col++) {
      double totalEnergy = 0;
      
      for (int row = 0; row < testValues.length; row++) {
        totalEnergy += testValues[row][col];
      }

      if (totalEnergy > maxEnergy) {
        maxEnergy = totalEnergy;
        maxDay = col;
      }
    }

    return maxDay;
  }

}`}],dataFiles:[]},{name:`Predict and Run: Enhanced For Loops and Objects`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Numbers.java`,text:`public class Numbers {

  public static String getValuesEnhanced(int[][] numbers) {
    String result = "";
    
    for (int[] row : numbers) {
      for (int value : row) {
        result += value + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static String getValuesRegular(int[][] numbers) {
    String result = "";
    
    for (int row = 0; row < numbers.length; row++) {
      for (int col = 0; col < numbers[0].length; col++) {
        result += numbers[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static void changeValues(int[][] numbers) {
    for (int[] row : numbers) {
      for (int value : row) {
        value *= 2;
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Enhanced For Loops #1`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Numbers.java`,text:`public class Numbers {

  public static String getValuesEnhanced(int[][] numbers) {
    String result = "";
    
    for (int[] row : numbers) {
      for (int value : row) {
        result += value + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static String getValuesRegular(int[][] numbers) {
    String result = "";
    
    for (int row = 0; row < numbers.length; row++) {
      for (int col = 0; col < numbers[0].length; col++) {
        result += numbers[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static void changeValues(int[][] numbers) {
    for (int[] row : numbers) {
      for (int value : row) {
        value *= 2;
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Enhanced For Loops #2`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Numbers.java`,text:`public class Numbers {

  public static String getValuesEnhanced(int[][] numbers) {
    String result = "";
    
    for (int[] row : numbers) {
      for (int value : row) {
        result += value + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static String getValuesRegular(int[][] numbers) {
    String result = "";
    
    for (int row = 0; row < numbers.length; row++) {
      for (int col = 0; col < numbers[0].length; col++) {
        result += numbers[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static void changeValues(int[][] numbers) {
    for (int[] row : numbers) {
      for (int value : row) {
        value *= 2;
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Enhanced For Loops #3`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Numbers.java`,text:`public class Numbers {

  public static String getValuesEnhanced(int[][] numbers) {
    String result = "";
    
    for (int[] row : numbers) {
      for (int value : row) {
        result += value + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static String getValuesRegular(int[][] numbers) {
    String result = "";
    
    for (int row = 0; row < numbers.length; row++) {
      for (int col = 0; col < numbers[0].length; col++) {
        result += numbers[row][col] + " ";
      }

      result += "\\n";
    }

    return result;
  }

  public static void changeValues(int[][] numbers) {
    for (int[] row : numbers) {
      for (int value : row) {
        value *= 2;
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (a) #1`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the number of times books have been borred
    int[][] bookBorrowCount = { {3, 5, 2}, {1, 4, 6}, {2, 5, 3} };

    // Creates a Library object
    Library library = new Library(bookBorrowCount);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getMostBorrowed() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Library.java`,text:`/*
 * Represents a library
 */
public class Library {

  private int[][] bookCount;      // The 2D array containing the number of times books have been borrowed

  /*
   * Initializes bookCount to the specified 2D array containing
   * the number of times books have been borrowed
   */
  public Library(int[][] bookCount) {
    this.bookCount = bookCount;
  }

  /*
   * Returns the 2D array bookCount
   */
  public int[][] getBookCount() {
    return bookCount;
  }

  /*
   * Returns the most number of times a book has been borrowed
   */
  public int getMostBorrowed() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find and return the value in the 2D array bookCount that is the highest.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`LibraryTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Library.java Test")
public class LibraryTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Library testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Library(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("getMostBorrowed() returns the most number of times a book has been borrowed => ")
  public void testGetMostBorrowed() {
    String message = "Use a variable to keep track of the highest number found. Traverse the";
    message += "\\n        2D array bookCount to check if each value is greater than the max. If so,";
    message += "\\n        update the max. Then return the highest value found." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.getMostBorrowed();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[3][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 20) + 5;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int max = 0;
    
    for (int[] row : testValues) {
      for (int value : row) {
        if (value > max) {
          max = value;
        }
      }
    }
    
    return max;
  }
  
}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (b) #1`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the number of animals spotted on each tour
    int[][] animalCount = {{3, 5, 2}, {1, 4, 6}, {2, 5, 3}, {6, 2, 1}};

    // Creates a WildlifeTour object
    WildlifeTour wildlifeTour = new WildlifeTour(animalCount);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getTotalAnimals() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`WildlifeTour.java`,text:`/*
 * Represents a wildlife tour at a park
 */
public class WildlifeTour {
  
  private int[][] animalCount;     // The 2D array containing the number of animals spotted during several tours

  /*
   * Initializes animalCount to the specified 2D array containing
   * the number of animals spotted during several tours
   */
  public WildlifeTour(int[][] animalCount) {
    this.animalCount = animalCount;
  }

  /*
   * Returns the 2D array animalCount
   */
  public int[][] getAnimalCount() {
    return animalCount;
  }

  /*
   * Returns the total number of animals spotted during all tours
   */
  public int getTotalAnimals() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the total of all values in the 2D array animalCount.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`WildlifeTourTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("WildlifeTour.java Test")
public class WildlifeTourTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  WildlifeTour testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new WildlifeTour(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("getTotalAnimals() returns the total number of animals spotted during all tours => ")
  public void testGetTotalAnimals() {
    String message = "Traverse the 2D array animalCount to find and return the total." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.getTotalAnimals();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[4][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 10) + 2;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int total = 0;
    
    for (int[] row : testValues) {
      for (int value : row) {
        total += value;
      }
    }
    
    return total;
  }

}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (c) — Console`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the costs of customer orders
    double[][] orderCost = {{25.5, 15.25, 12.75}, {18.5, 20.0, 16.75}, {14.25, 21.5, 19.75}};

    // Creates a Restaurant object
    Restaurant restaurant = new Restaurant(orderCost);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getHighestCost() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Restaurant.java`,text:`/*
 * Represents a restaurant
 */
public class Restaurant {
  
  private double[][] orders;     // The 2D array containing the cost of several customer orders

  /*
   * Initializes orders to the specified 2D array containing
   * the cost of several customer orders
   */
  public Restaurant(double[][] orders) {
    this.orders = orders;
  }

  /*
   * Returns the 2D array orders
   */
  public double[][] getOrders() {
    return orders;
  }

  /*
   * Returns the largest customer order
   */
  public double getHighestCost() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array orders to find and return the largest customer order.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`RestaurantTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Restaurant.java Test")
public class RestaurantTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  Restaurant testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Restaurant(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("getHighestCost() finds and returns the largest customer order => ")
  public void testGetHighestCost() {
    String message = "Traverse the 2D array orders and check if each value is larger than the current max";
    message += "\\n        value found. Return the largest value in the 2D array." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.getHighestCost();

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[3][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 30 + 10;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double max = 0;
    
    for (double[] row : testValues) {
      for (double value : row) {
        if (value > max) {
          max = value;
        }
      }
    }
    
    return max;
  }
   
}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (d) — Console`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the activity points for several students
    int[][] activityPoints = {{3, 5, 2}, {1, 4, 6}, {2, 5, 3}, {6, 2, 1}};

    // Creates a School object
    School school = new School(activityPoints);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getHighestActivity() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`School.java`,text:`/*
 * Represents a school
 */
public class School {
  
  private int[][] activityPoints;   // The 2D array containing the activity points for several students

  /*
   * Initializes activityPoints to the specified 2D array
   * containing the activity points for several students
   */
  public School(int[][] activityPoints) {
      this.activityPoints = activityPoints;
  }

  /*
   * Returns the 2D array activityPoints
   */
  public int[][] getActivityPoints() {
    return activityPoints;
  }

  /*
   * Returns the row index for the student that has the
   * highest total activity points
   */
  public int getHighestActivity() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array activityPoints to calculate the total activity
     * points for each student (row). Return the row index that has the
     * largest total activity points.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`SchoolTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("School.java Test")
public class SchoolTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  School testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new School(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("getHighestActivity() returns the column index with the highest total activity points => ")
  public void testGetHighestActivity() {
    String message = "Use an enhanced for loop for the outer loop, and a regular for loop for the inner loop. Find";
    message += "\\n        the total for each column, and return the column index with the highest total." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testObject.getHighestActivity();

    assertEquals(expected, actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[4][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 30) + 5;
      }
    }

    return temp;
  }

  private int getExpectedResult() {
    int highestPoints = 0;
    int highestStudent = 0;
    int currentStudent = 0;
    
    for (int[] student : testValues) {
      int studentTotal = 0;
      
      for (int activityScore : student) {
        studentTotal += activityScore;
      }
      
      if (studentTotal > highestPoints) {
        highestPoints = studentTotal;
        highestStudent = currentStudent;
      }

      currentStudent++;
    }
    
    return highestStudent;
  }
   
}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (a) #2`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing each food truck inventory
    int[][] inventories = {{25, 17, 22}, {18, 12, 15}, {21, 19, 27}, {30, 10, 23}};

    // Creates an Owner object
    Owner jessie = new Owner(inventories);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the hasMinimum() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Owner.java`,text:`/*
 * Represents an owner of a food truck business
 */
public class Owner {

  private int[][] inventories;   // The 2D array of dessert inventories

  /*
   * Initializes inventories to the specified 2D
   * array of dessert inventories
   */
  public Owner(int[][] inventories) {
    this.inventories = inventories;
  }

  /*
   * Returns the 2D array inventories
   */
  public int[][] getInventories() {
    return inventories;
  }

  /*
   * Returns true if any food truck has a quantity of
   * desserts that are more than minValue
   */
  public boolean hasMinimum(int minValue) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the inventories array and return true if any food truck has a
     * quantity of desserts more than minValue. Otherwise, return false.
     * -----------------------------------------------------------------------------
     */

    

    return false;
  }
  
}`}],validationFiles:[{path:`OwnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Owner.java Test")
public class OwnerTest {

  String messageGap = "\\n       ";
  int[][] testValues;
  Owner testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Owner(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("hasMinimum() returns true if any food truck has a quantity greater than minValue => ")
  public void testHasMinimumReturnsTrue() {
    String message = "Return true if any value is greater than minValue." + messageGap;
    boolean actual = testObject.hasMinimum(1);
    assertTrue(actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("hasMinimum() returns false if no food truck has a quantity greater than minValue => ")
  public void testHasMinimumReturnsFalse() {
    String message = "Return false if none of the values are greater than minValue." + messageGap;
    boolean actual = testObject.hasMinimum(100);
    assertFalse(actual, message);
  }

  private int[][] getRandomValues() {
    int[][] temp = new int[4][3];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = (int)(Math.random() * 20) + 10;
      }
    }

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (b) #2`,lesson:`Lesson 5: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing the daily temperatures for several cities
    double[][] cityTemps = {{37.27, 40.66, 53.53, 52.36, 48.87},
                            {62.2, 61.47, 60.31, 62.42, 62.38},
                            {81.5, 80.04, 81.61, 85.91, 85.69},
                            {56.41, 48.6, 51.67, 48.72, 52.65}};

    // Creates a Temperatures object
    Temperatures cities = new Temperatures(cityTemps);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverage() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Temperatures.java`,text:`/*
 * Analyzes data about daily temperatures in cities
 */
public class Temperatures {

  private double[][] dailyTemps;  // The 2D array of daily temperatures

  /*
   * Initializes dailyTemps to the specified
   * 2D array of daily temperatures
   */
  public Temperatures(double[][] dailyTemps) {
    this.dailyTemps = dailyTemps;
  }

  /*
   * Returns the 2D array dailyTemps
   */
  public double[][] getDailyTemps() {
    return dailyTemps;
  }

  /*
   * Returns the overall average of all temperatures
   */
  public double calcAverage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the overall average temperature.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`TemperaturesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Temperatures.java Test")
public class TemperaturesTest {

  String messageGap = "\\n       ";
  double[][] testValues;
  Temperatures testObject;
   
  @BeforeEach
  public void setup() {
    testValues = getRandomValues();
    testObject = new Temperatures(testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcAverage() returns the overall average of all temperatures => ")
  public void testCalcAverage() {
    String message = "Find the total of all values in the 2D array. Divide the total by the";
    message += "\\n        result of the number of rows times the number of columns to get the average." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testObject.calcAverage();

    assertEquals(expected, actual, message);
  }

  private double[][] getRandomValues() {
    double[][] temp = new double[4][5];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = Math.random() * 90 + 30;
      }
    }

    return temp;
  }

  private double getExpectedResult() {
    double total = 0;

    for (double[] row : testValues) {
      for (double value : row) {
        total += value;
      }
    }

    return total / (testValues.length * testValues[0].length);
  }
   
}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (c) — Theater`,lesson:`Lesson 5: Enhanced For Loops`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of artwork by artist
    Art[][] artwork = {{new Art("weepingwoman.jpg", "Pablo Picasso"), new Art("oldguitarist.jpeg", "Pablo Picasso")},
                       {new Art("monalisa.jpg", "Leonardo da Vinci"), new Art("ginevrabenci.jpg", "Leonardo da Vinci")},
                       {new Art("sunrise.jpeg", "Claude Monet"), new Art("artistsgarden.jpg", "Claude Monet")}};

    // Creates an ArtScene object
    ArtScene scene = new ArtScene(artwork);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawScene() method with the 1D array returned from calling the
     * findArtByArtist() method.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`Art.java`,text:`/*
 * Represents a piece of art
 */
public class Art {

  private String image;    // The name of the image file
  private String artist;   // The name of the artist

  /*
   * Sets image and artist to the specified values
   */
  public Art(String image, String artist) {
    this.image = image;
    this.artist = artist;
  }

  /*
   * Returns the name of the image file
   */
  public String getImage() {
    return image;
  }

  /*
   * Returns the name of the artist
   */
  public String getArtist() {
    return artist;
  }
  
}`},{path:`ArtScene.java`,text:`import org.code.theater.*;

/*
 * Creates an animation displaying artwork
 */
public class ArtScene extends Scene {

  private Art[][] artwork;   // The 2D array of artwork

  /*
   * Initializes artwork to the specified 2D array of artwork
   */
  public ArtScene(Art[][] artwork) {
    this.artwork = artwork;
  }

  /*
   * Returns the 2D array of artwork
   */
  public Art[][] getArtwork() {
    return artwork;
  }

  /*
   * Returns the row of artwork created by artistToFind
   */
  public Art[] findArtByArtist(String artistToFind) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find and return the row of artwork created by artistToFind.
     * -----------------------------------------------------------------------------
     */


    

    return null;
  }

  /*
   * Draws an artist's artwork in the scene
   */
  public void drawScene(Art[] artistWork) {
    drawText("Artwork by " + artistWork[0].getArtist(), 100, 50);
    pause(1);
    clear("white");
    
    for (Art image : artistWork) {
      drawImage(image.getImage(), 0, 0, getWidth());
      pause(2);
      clear("white");
    }
  }
  
}`}],validationFiles:[{path:`ArtSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ArtScene.java Test")
public class ArtSceneTest {

  String messageGap = "\\n       ";
  Art[][] testArtwork;
  ArtScene testObject;
   
  @BeforeEach
  public void setup() {
    testArtwork = new Art[][]{{new Art("weepingwoman.jpg", "Pablo Picasso"), new Art("oldguitarist.jpeg", "Pablo Picasso")},
                              {new Art("monalisa.jpg", "Leonardo da Vinci"), new Art("ginevrabenci.jpg", "Leonardo da Vinci")},
                              {new Art("sunrise.jpeg", "Claude Monet"), new Art("artistsgarden.jpg", "Claude Monet")}};
    testObject = new ArtScene(testArtwork);
  }
   
  @Test
  @Order(1)
  @DisplayName("findArtByArtist() returns the row of artwork created by the artist to find => ")
  public void testFindArtByArtistReturnsArtwork() {
    String message = "If the artist's name matches the artist to find, return that row (the 1D array)." + messageGap;
      
    String testArtist = getRandomArtist();
    Art[] expected = getExpectedResult(testArtist);
    Art[] actual = testObject.findArtByArtist(testArtist);

    assertArrayEquals(expected, actual, message);
  }
  
  @Test
  @Order(2)
  @DisplayName("findArtByArtist() returns null if no artwork found by the artist to find => ")
  public void testFindArtByArtistReturnsNull() {
    String message = "If no matching artist is found, return null." + messageGap;
    Art[] actual = testObject.findArtByArtist("some artist");
    assertNull(actual, message);
  }
   
  private String getRandomArtist() {
    int choice = (int)(Math.random() * testArtwork.length);
    return testArtwork[choice][0].getArtist();
  }

  public Art[] getExpectedResult(String artistToFind) {
    for (Art[] artist : testArtwork) {
      for (Art work : artist) {
        if (work.getArtist().equals(artistToFind)) {
          return artist;
        }
      }
    }

    return null;
  }
  
}`}],dataFiles:[]},{name:`Practice: Enhanced For Loops and 2D Arrays (d) — Theater`,lesson:`Lesson 5: Enhanced For Loops`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of SocialMedia objects
    SocialMedia[][] socialMediaData = 
        {{new SocialMedia("Twitter", 2018, 1318), new SocialMedia("Facebook", 2018, 9021), new SocialMedia("Instagram", 2018, 2755)},
         {new SocialMedia("Twitter", 2019, 1330), new SocialMedia("Facebook", 2019, 9736), new SocialMedia("Instagram", 2019, 3586)},
         {new SocialMedia("Twitter", 2020, 1358), new SocialMedia("Facebook", 2020, 10841), new SocialMedia("Instagram", 2020, 4031)}};

    // Creates a SocialMediaScene object
    SocialMediaScene scene = new SocialMediaScene(socialMediaData);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawScene() method with the 1D array returned from calling the
     * getValuesByYear() method.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(scene);

  }
}`},{path:`SocialMedia.java`,text:`/*
 * Represents a social media app
 */
public class SocialMedia {

  private String name;     // The name of a social media app
  private int year;        // The year for the number of users on a social media app
  private int numUsers;    // The number of users a social media app had in a year

  /*
   * Sets name, year, and numUsers to the specified values
   */
  public SocialMedia(String name, int year, int numUsers) {
    this.name = name;
    this.year = year;
    this.numUsers = numUsers;
  }

  /*
   * Returns the name of the social media app
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the year
   */
  public int getYear() {
    return year;
  }

  /*
   * Returns the number of users the social media app had in a year
   */
  public int getNumUsers() {
    return numUsers;
  }

  /*
   * Returns a String containing the name of the social media
   * app and number of users for the year
   */
  public String toString() {
    return name + ": " + numUsers + " users in " + year;
  }
  
}`},{path:`SocialMediaScene.java`,text:`import org.code.theater.*;

/*
 * Creates an animation to visualize the number
 * users social media platforms have each year
 */
public class SocialMediaScene extends Scene {

  private SocialMedia[][] userData;     // The 2D array containing the number of users for several years

  /*
   * Initializes userData to the specified 2D array containing the 
   * number of users on each app for several years
   */
  public SocialMediaScene(SocialMedia[][] userData) {
    this.userData = userData;
  }

  /*
   * Returns the 2D array userData
   */
  public SocialMedia[][] getUserData() {
    return userData;
  }

  /*
   * Returns a 1D array containing the SocialMedia objects
   * that match the parameter yearToFind
   */
  public SocialMedia[] getValuesByYear(int yearToFind) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array userData. If the SocialMedia object's year matches
     * the yearToFind, add the SocialMedia object to a 1D array, and return the
     * 1D array containing each SocialMedia object for that year.
     * -----------------------------------------------------------------------------
     */


    

    return null;
  }

  /*
   * Returns the image file for a social media app
   */
  public String getImage(String appName) {
    if (appName.equals("Twitter")) {
      return "twitter.png";
    }
    else if (appName.equals("Facebook")) {
      return "facebook.png";
    }
    else {
      return "instagram.png";
    }
  }

  /*
   * Creates the scene by drawing the icons for each social media
   * platform and number of users that year
   */
  public void drawScene(SocialMedia[] results) {
    for (SocialMedia app : results) {
      String icon = getImage(app.getName());
      drawImage(icon, 100, 50, 200);
      drawText(app + "", 75, 300);
      pause(1);
      clear("white");
    }
  }
  
}`}],validationFiles:[{path:`SocialMediaSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SocialMediaScene.java Test")
public class SocialMediaSceneTest {

  String messageGap = "\\n       ";
  SocialMedia[][] testData;
  SocialMediaScene testObject;
   
  @BeforeEach
  public void setup() {
    testData = new SocialMedia[][]{{new SocialMedia("Twitter", 2018, 1318), new SocialMedia("Facebook", 2018, 9021), new SocialMedia("Instagram", 2018, 2755)},
         {new SocialMedia("Twitter", 2019, 1330), new SocialMedia("Facebook", 2019, 9736), new SocialMedia("Instagram", 2019, 3586)},
         {new SocialMedia("Twitter", 2020, 1358), new SocialMedia("Facebook", 2020, 10841), new SocialMedia("Instagram", 2020, 4031)}};

    testObject = new SocialMediaScene(testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("getValuesByYear() returns a 1D array containig the SocialMedia objects that match yearToFind => ")
  public void testGetValuesByYear() {
    String message = "If the SocialMedia object's year equals yearToFind, add the object to a 1D array of SocialMedia";
    message += "\\n        objects to return. The 1D array should be of length 3." + messageGap;

    int randomYear = getRandomYear();
    SocialMedia[] expected = getExpectedResult(randomYear);
    SocialMedia[] actual = testObject.getValuesByYear(randomYear);

    assertArrayEquals(expected, actual, message);
  }

  private int getRandomYear() {
    int[] years = {2018, 2019, 2020};
    return years[(int)(Math.random() * years.length)];
  }

  private SocialMedia[] getExpectedResult(int yearToFind) {
    SocialMedia[] results = new SocialMedia[3];
    int index = 0;

    for (SocialMedia[] year : testData) {
      for (SocialMedia app : year) {
        if (app.getYear() == yearToFind) {
          results[index] = app;
          index++;
        }
      }
    }

    return results;
  }

}`}],dataFiles:[]},{name:`Predict and Run: Image Filters`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`BrightenedImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class BrightenedImage extends Image {

  private Pixel[][] imagePixels;

  public BrightenedImage(String imageFile) {
    super(imageFile);
    
    imagePixels = getPixelsFromImage();
  }

  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

  public void brightenColors(int amount) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        Pixel currentPixel = imagePixels[row][col];

        currentPixel.setRed(currentPixel.getRed() + amount);
        currentPixel.setGreen(currentPixel.getGreen() + amount);
        currentPixel.setBlue(currentPixel.getBlue() + amount);
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Image Filters #1`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`BrightenedImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class BrightenedImage extends Image {

  private Pixel[][] imagePixels;

  public BrightenedImage(String imageFile) {
    super(imageFile);
    
    imagePixels = getPixelsFromImage();
  }

  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

  public void brightenColors(int amount) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        Pixel currentPixel = imagePixels[row][col];

        currentPixel.setRed(currentPixel.getRed() + amount);
        currentPixel.setGreen(currentPixel.getGreen() + amount);
        currentPixel.setBlue(currentPixel.getBlue() + amount);
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Image Filters #2`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`BrightenedImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class BrightenedImage extends Image {

  private Pixel[][] imagePixels;

  public BrightenedImage(String imageFile) {
    super(imageFile);
    
    imagePixels = getPixelsFromImage();
  }

  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

  public void brightenColors(int amount) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        Pixel currentPixel = imagePixels[row][col];

        currentPixel.setRed(currentPixel.getRed() + amount);
        currentPixel.setGreen(currentPixel.getGreen() + amount);
        currentPixel.setBlue(currentPixel.getBlue() + amount);
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Image Filters #3`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`BrightenedImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class BrightenedImage extends Image {

  private Pixel[][] imagePixels;

  public BrightenedImage(String imageFile) {
    super(imageFile);
    
    imagePixels = getPixelsFromImage();
  }

  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

  public void brightenColors(int amount) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        Pixel currentPixel = imagePixels[row][col];

        currentPixel.setRed(currentPixel.getRed() + amount);
        currentPixel.setGreen(currentPixel.getGreen() + amount);
        currentPixel.setBlue(currentPixel.getBlue() + amount);
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Creating Image Filters`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Instantiate an ImagePlus object and draw the image in the scene.
     * -----------------------------------------------------------------------------
     */


    
    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7: Instantiate a CustomImage and draw your image in the scene without
     * any filters. Then call your filter method and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    
    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`CustomImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be customized with image filters
 */
public class CustomImage extends ImagePlus {

  /*
   * Sets the superclass filename to the specified filename
   */
  public CustomImage(String filename) {
    super(filename);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Level 7: Write your image filter method.
   * -----------------------------------------------------------------------------
   */


  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Create and return a 2D array of Pixel objects to contain the
     * pixels that make up the image.
     * -----------------------------------------------------------------------------
     */
    

    return null;
  }

}`}],validationFiles:[{path:`ImagePlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ImagePlus.java Test")
public class ImagePlusTest {

  String messageGap = "\\n       ";
  ImagePlus testObject;
   
  @BeforeEach
  public void setup() {
    String testImage = getRandomImage();
    testObject = new ImagePlus(testImage);
  }
   
  @Test
  @Order(1)
  @DisplayName("getPixelsFromImage() returns a 2D array of Pixel objects that make up the image => ")
  public void testGetPixelsFromImage() {
    String message = "Create a 2D array of Pixel objects where the number of rows is the height of the image and";
    message += "\\n        the number of columns is the width of the image. Set each Pixel object to the pixel";
    message += "\\n        located at the correct x (col) and y (row) position." + messageGap;
      
    Pixel[][] expected = getExpectedResult();
    Pixel[][] actual = testObject.getPixelsFromImage();

    assertArrayEquals(expected, actual, message);
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private Pixel[][] getExpectedResult() {
    Pixel[][] tempPixels = new Pixel[testObject.getHeight()][testObject.getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = testObject.getPixel(col, row);
      }
    }

    return tempPixels;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating Image Filters (a)`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Instantiate an ImagePlus object and draw the image in the scene.
     * -----------------------------------------------------------------------------
     */


    
    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7: Instantiate a CustomImage and draw your image in the scene without
     * any filters. Then call your filter method and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    
    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`CustomImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be customized with image filters
 */
public class CustomImage extends ImagePlus {

  /*
   * Sets the superclass filename to the specified filename
   */
  public CustomImage(String filename) {
    super(filename);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Level 7: Write your image filter method.
   * -----------------------------------------------------------------------------
   */


  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Create and return a 2D array of Pixel objects to contain the
     * pixels that make up the image.
     * -----------------------------------------------------------------------------
     */
    

    return null;
  }

}`}],validationFiles:[{path:`CustomImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CustomImage.java Test")
public class CustomImageTest {

  String messageGap = "\\n       ";
  CustomImage testObject;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new CustomImage(randomImage);
  }
   
  @Test
  @Order(1)
  @DisplayName("zeroRed() sets the red value of each Pixel to 0 => ")
  public void testZeroRed() {
    String message = "Use the getPixelsFromImage() method in the ImagePlus class to get the 2D array";
    message += "\\n        of Pixel objects that make up the CustomImage. Traverse the 2D array and set";
    message += "\\n        the red value of each Pixel object to 0 using the setRed() method." + messageGap;
      
    testObject.zeroRed();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        assertEquals(0, actual[row][col].getRed(), message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating Image Filters (b)`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Instantiate an ImagePlus object and draw the image in the scene.
     * -----------------------------------------------------------------------------
     */


    
    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7: Instantiate a CustomImage and draw your image in the scene without
     * any filters. Then call your filter method and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    
    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`CustomImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be customized with image filters
 */
public class CustomImage extends ImagePlus {

  /*
   * Sets the superclass filename to the specified filename
   */
  public CustomImage(String filename) {
    super(filename);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Level 7: Write your image filter method.
   * -----------------------------------------------------------------------------
   */


  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Create and return a 2D array of Pixel objects to contain the
     * pixels that make up the image.
     * -----------------------------------------------------------------------------
     */
    

    return null;
  }

}`}],validationFiles:[{path:`CustomImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CustomImage.java Test")
public class CustomImageTest {

  String messageGap = "\\n       ";
  CustomImage testObject;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new CustomImage(randomImage);
  }
   
  @Test
  @Order(1)
  @DisplayName("zerolue() sets the blue value of each Pixel to 0 => ")
  public void testZeroBlue() {
    String message = "Use the getPixelsFromImage() method in the ImagePlus class to get the 2D array";
    message += "\\n        of Pixel objects that make up the CustomImage. Traverse the 2D array and set";
    message += "\\n        the blue value of each Pixel object to 0 using the setBlue() method." + messageGap;
      
    testObject.zeroBlue();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        assertEquals(0, actual[row][col].getBlue(), message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating Image Filters (c)`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Instantiate an ImagePlus object and draw the image in the scene.
     * -----------------------------------------------------------------------------
     */


    
    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7: Instantiate a CustomImage and draw your image in the scene without
     * any filters. Then call your filter method and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    
    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`CustomImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be customized with image filters
 */
public class CustomImage extends ImagePlus {

  /*
   * Sets the superclass filename to the specified filename
   */
  public CustomImage(String filename) {
    super(filename);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Level 7: Write your image filter method.
   * -----------------------------------------------------------------------------
   */


  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Create and return a 2D array of Pixel objects to contain the
     * pixels that make up the image.
     * -----------------------------------------------------------------------------
     */
    

    return null;
  }

}`}],validationFiles:[{path:`CustomImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CustomImage.java Test")
public class CustomImageTest {

  String messageGap = "\\n       ";
  CustomImage testObject;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new CustomImage(randomImage);
  }
   
  @Test
  @Order(1)
  @DisplayName("zeroGreen() sets the green value of each Pixel to 0 => ")
  public void testZeroGreen() {
    String message = "Use the getPixelsFromImage() method in the ImagePlus class to get the 2D array";
    message += "\\n        of Pixel objects that make up the CustomImage. Traverse the 2D array and set";
    message += "\\n        the green value of each Pixel object to 0 using the setGreen() method." + messageGap;
      
    testObject.zeroGreen();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        assertEquals(0, actual[row][col].getGreen(), message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating Image Filters (d)`,lesson:`Lesson 6: Images in The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Instantiate an ImagePlus object and draw the image in the scene.
     * -----------------------------------------------------------------------------
     */


    
    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7: Instantiate a CustomImage and draw your image in the scene without
     * any filters. Then call your filter method and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    
    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`CustomImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be customized with image filters
 */
public class CustomImage extends ImagePlus {

  /*
   * Sets the superclass filename to the specified filename
   */
  public CustomImage(String filename) {
    super(filename);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Level 7: Write your image filter method.
   * -----------------------------------------------------------------------------
   */


  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6: Create and return a 2D array of Pixel objects to contain the
     * pixels that make up the image.
     * -----------------------------------------------------------------------------
     */
    

    return null;
  }

}`}],validationFiles:[{path:`CustomImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CustomImage.java Test")
public class CustomImageTest {

  String messageGap = "\\n       ";
  CustomImage testObject;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new CustomImage(randomImage);
  }
   
  @Test
  @Order(1)
  @DisplayName("keepColor() sets the green and blue values of each Pixel to 0 with the argument \\"red\\" => ")
  public void testKeepColorKeepsRed() {
    String message = "Use the getPixelsFromImage() method in the ImagePlus class to get the 2D array";
    message += "\\n        of Pixel objects that make up the CustomImage. Traverse the 2D array and set";
    message += "\\n        the green and blue values of each Pixel object to 0." + messageGap;
      
    testObject.keepColor("red");
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        assertEquals(0, actual[row][col].getGreen(), message);
        assertEquals(0, actual[row][col].getBlue(), message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("keepColor() sets the red and blue values of each Pixel to 0 with the argument \\"green\\" => ")
  public void testKeepColorKeepsGreen() {
    String message = "Use the getPixelsFromImage() method in the ImagePlus class to get the 2D array";
    message += "\\n        of Pixel objects that make up the CustomImage. Traverse the 2D array and set";
    message += "\\n        the red and blue values of each Pixel object to 0." + messageGap;
      
    testObject.keepColor("green");
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        assertEquals(0, actual[row][col].getRed(), message);
        assertEquals(0, actual[row][col].getBlue(), message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("keepColor() sets the red and green values of each Pixel to 0 with the argument \\"blue\\" => ")
  public void testKeepColorKeepsBlue() {
    String message = "Use the getPixelsFromImage() method in the ImagePlus class to get the 2D array";
    message += "\\n        of Pixel objects that make up the CustomImage. Traverse the 2D array and set";
    message += "\\n        the red and green values of each Pixel object to 0." + messageGap;
      
    testObject.keepColor("blue");
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        assertEquals(0, actual[row][col].getRed(), message);
        assertEquals(0, actual[row][col].getGreen(), message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
   
}`}],dataFiles:[]},{name:`Predict and Run: The indexOf() Method`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`PhraseFinder.java`,text:`/*
 * Analyzes phrases and sentences
 */
public class PhraseFinder {

  public static String countPhrases(String[][] sentences, String targetPhrase) {
    String result = "";
    
    for (int row = 0; row < sentences.length; row++) {
      int phraseCount = 0;
      
      for (int col = 0; col < sentences[0].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1) {
          phraseCount++;
        }
      }

      result += "Row #" + row + ": " + phraseCount + " times\\n";
    }

    return result;
  }

  public static String findLongestString(String[][] sentences, String targetPhrase) {
    int longestLength = 0;
    int longestRow = -1;
    int longestCol = -1;
    
    for (int row = 0; row < sentences.length; row++) {
      for (int col = 0; col < sentences[row].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1 && currentPhrase.length() > longestLength) {
          longestLength = currentPhrase.length();
          longestRow = row;
          longestCol = col;
        }
      }
    }

    if (longestRow == -1 && longestCol == -1) {
      return "No string containing the target phrase was found.";
    } else {
      return "The longest string containing the target phrase is \\"" + sentences[longestRow][longestCol] +
             "\\" at row #" + longestRow + " and col #" + longestCol + ".";
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The indexOf() Method #1`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`PhraseFinder.java`,text:`/*
 * Analyzes phrases and sentences
 */
public class PhraseFinder {

  public static String countPhrases(String[][] sentences, String targetPhrase) {
    String result = "";
    
    for (int row = 0; row < sentences.length; row++) {
      int phraseCount = 0;
      
      for (int col = 0; col < sentences[0].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1) {
          phraseCount++;
        }
      }

      result += "Row #" + row + ": " + phraseCount + " times\\n";
    }

    return result;
  }

  public static String findLongestString(String[][] sentences, String targetPhrase) {
    int longestLength = 0;
    int longestRow = -1;
    int longestCol = -1;
    
    for (int row = 0; row < sentences.length; row++) {
      for (int col = 0; col < sentences[row].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1 && currentPhrase.length() > longestLength) {
          longestLength = currentPhrase.length();
          longestRow = row;
          longestCol = col;
        }
      }
    }

    if (longestRow == -1 && longestCol == -1) {
      return "No string containing the target phrase was found.";
    } else {
      return "The longest string containing the target phrase is \\"" + sentences[longestRow][longestCol] +
             "\\" at row #" + longestRow + " and col #" + longestCol + ".";
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The indexOf() Method #2`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`PhraseFinder.java`,text:`/*
 * Analyzes phrases and sentences
 */
public class PhraseFinder {

  public static String countPhrases(String[][] sentences, String targetPhrase) {
    String result = "";
    
    for (int row = 0; row < sentences.length; row++) {
      int phraseCount = 0;
      
      for (int col = 0; col < sentences[0].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1) {
          phraseCount++;
        }
      }

      result += "Row #" + row + ": " + phraseCount + " times\\n";
    }

    return result;
  }

  public static String findLongestString(String[][] sentences, String targetPhrase) {
    int longestLength = 0;
    int longestRow = -1;
    int longestCol = -1;
    
    for (int row = 0; row < sentences.length; row++) {
      for (int col = 0; col < sentences[row].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1 && currentPhrase.length() > longestLength) {
          longestLength = currentPhrase.length();
          longestRow = row;
          longestCol = col;
        }
      }
    }

    if (longestRow == -1 && longestCol == -1) {
      return "No string containing the target phrase was found.";
    } else {
      return "The longest string containing the target phrase is \\"" + sentences[longestRow][longestCol] +
             "\\" at row #" + longestRow + " and col #" + longestCol + ".";
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The indexOf() Method #3`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`PhraseFinder.java`,text:`/*
 * Analyzes phrases and sentences
 */
public class PhraseFinder {

  public static String countPhrases(String[][] sentences, String targetPhrase) {
    String result = "";
    
    for (int row = 0; row < sentences.length; row++) {
      int phraseCount = 0;
      
      for (int col = 0; col < sentences[0].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1) {
          phraseCount++;
        }
      }

      result += "Row #" + row + ": " + phraseCount + " times\\n";
    }

    return result;
  }

  public static String findLongestString(String[][] sentences, String targetPhrase) {
    int longestLength = 0;
    int longestRow = -1;
    int longestCol = -1;
    
    for (int row = 0; row < sentences.length; row++) {
      for (int col = 0; col < sentences[row].length; col++) {
        String currentPhrase = sentences[row][col];
        int location = currentPhrase.indexOf(targetPhrase);

        if (location != -1 && currentPhrase.length() > longestLength) {
          longestLength = currentPhrase.length();
          longestRow = row;
          longestCol = col;
        }
      }
    }

    if (longestRow == -1 && longestCol == -1) {
      return "No string containing the target phrase was found.";
    } else {
      return "The longest string containing the target phrase is \\"" + sentences[longestRow][longestCol] +
             "\\" at row #" + longestRow + " and col #" + longestCol + ".";
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: The indexOf() Method (a)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Reads the data from the text files into 1D arrays for each week
    String[] weekOneReviews = FileReader.toStringArray("weekOne.txt");
    String[] weekTwoReviews = FileReader.toStringArray("weekTwo.txt");
    String[] weekThreeReviews = FileReader.toStringArray("weekThree.txt");

    // Creates a 2D array of reviews submitted each week
    String[][] productReviews = {weekOneReviews, weekTwoReviews, weekThreeReviews};

    // Creates a Review object
    Review review = new Review(productReviews);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countTimes() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`FileReader.java`,text:`import java.util.Scanner;
import java.util.ArrayList;
import java.io.File;
import java.io.FileNotFoundException;

public class FileReader {

  public static String[] toStringArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    String[] data = new String[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = currentValue;
    }

    return data;
  }

  public static int[] toIntArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    int[] data = new int[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Integer.parseInt(currentValue);
    }

    return data;
  }

  public static double[] toDoubleArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    double[] data = new double[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Double.parseDouble(currentValue);
    }

    return data;
  }

  private static Scanner createScanner(File theFile) {
    Scanner tempScanner = null;

    try {
      tempScanner = new Scanner(theFile);
    } catch(FileNotFoundException error) {
      System.out.println("File not found.");
    }

    return tempScanner;
  }

  private static ArrayList<String> readDataFromFile(String filename) {
    File dataFile = new File(filename);
    Scanner reader = createScanner(dataFile);
    ArrayList<String> temp = new ArrayList<String>();

    while (reader.hasNextLine()) {
      String currentLine = reader.nextLine().trim();
      temp.add(currentLine);
    }

    reader.close();
    return temp;
  }
  
}`},{path:`Review.java`,text:`/*
 * Analyzes customer reviews
 */
public class Review {
  
  private String[][] productReviews;     // The 2D array containing the reviews submitted each week

  /*
   * Initializes productReviews to the specified 2D array
   * containing the reviews submitted each week
   */
  public Review(String[][] productReviews) {
    this.productReviews = productReviews;
  }

  /*
   * Returns the 2D array of product reviews
   */
  public String[][] getProductReviews() {
    return productReviews;
  }

  /*
   * Counts and returns the number of times the
   * targetWord appears in all product reviews
   */
  public int countTimes(String targetWord) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array productReviews and count the number of times
     * targetWord appears in each review.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`ReviewTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Review.java Test")
public class ReviewTest {

  String messageGap = "\\n       ";
  String[][] testReviews;
  Review testObject;
   
  @BeforeEach
  public void setup() {
    testReviews = getTestReviews();
    testObject = new Review(testReviews);
  }
   
  @Test
  @Order(1)
  @DisplayName("countTimes() returns the number of times targetWord appears in each review => ")
  public void testCountTimes() {
    String message = "Use the indexOf() to determine if the targetWord appears in the review. If so,";
    message += "\\n        add it to a count and return the total number of times this is true." + messageGap;
      
    String testWord = getRandomWord();
    int expected = getExpectedResult(testWord);
    int actual = testObject.countTimes(testWord);

    assertEquals(expected, actual, message);
  }

  private String[][] getTestReviews() {
    String[] weekOneReviews = FileReader.toStringArray("weekOne.txt");
    String[] weekTwoReviews = FileReader.toStringArray("weekTwo.txt");
    String[] weekThreeReviews = FileReader.toStringArray("weekThree.txt");
    String[][] productReviews = {weekOneReviews, weekTwoReviews, weekThreeReviews};
    return productReviews;
  }

  private String getRandomWord() {
    String[] testWords = {"product", "price", "time", "quality", "easy"};
    return testWords[(int)(Math.random() * testWords.length)];
  }

  private int getExpectedResult(String targetWord) {
    int count = 0;
    
    for (String[] review : testReviews) {
      for (String word : review) {
        if (word.indexOf(targetWord) != -1) {
          count++;
        }
      }
    }
    
    return count;
  }
   
}`}],dataFiles:[{path:`weekOne.txt`,text:`Love it! So easy to use.
Meh, it's okay. Could be better.
Works great! Highly recommend.
Disappointed. Doesn't work as advertised.
Exactly what I needed. Thanks!`},{path:`weekThree.txt`,text:`Perfect! Exactly what I was looking for.
Meh. Average quality, average price.
Amazing product! So happy with my purchase.
Doesn't work at all. Don't waste your time.
Not bad, but could be better.`},{path:`weekTwo.txt`,text:`Terrible quality. Broke after a few uses.
So glad I bought this. Game changer!
Not impressed. Wouldn't buy again.
Great product for the price.
Avoid! Waste of money.`}]},{name:`Skill Building: The indexOf() Method (b)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates 1D arrays from the data in the text files
    String[] firstEssay = FileReader.toStringArray("firstEssay.txt");
    String[] secondEssay = FileReader.toStringArray("secondEssay.txt");
    String[] thirdEssay = FileReader.toStringArray("thirdEssay.txt");

    // Creates a 2D array containing the student essays
    String[][] essays = {firstEssay, secondEssay, thirdEssay};

    // Creates a Teacher object
    Teacher msSmith = new Teacher(essays);

    // Creates a 1D array of phrases
    String[] phrasesToFind = {"popular", "learn new things", "benefits"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findPhrases() method and print the result.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`FileReader.java`,text:`import java.util.Scanner;
import java.util.ArrayList;
import java.io.File;
import java.io.FileNotFoundException;

public class FileReader {

  public static String[] toStringArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    String[] data = new String[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = currentValue;
    }

    return data;
  }

  public static int[] toIntArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    int[] data = new int[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Integer.parseInt(currentValue);
    }

    return data;
  }

  public static double[] toDoubleArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    double[] data = new double[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Double.parseDouble(currentValue);
    }

    return data;
  }

  private static Scanner createScanner(File theFile) {
    Scanner tempScanner = null;

    try {
      tempScanner = new Scanner(theFile);
    } catch(FileNotFoundException error) {
      System.out.println("File not found.");
    }

    return tempScanner;
  }

  private static ArrayList<String> readDataFromFile(String filename) {
    File dataFile = new File(filename);
    Scanner reader = createScanner(dataFile);
    ArrayList<String> temp = new ArrayList<String>();

    while (reader.hasNextLine()) {
      String currentLine = reader.nextLine().trim();
      temp.add(currentLine);
    }

    reader.close();
    return temp;
  }
  
}`},{path:`Teacher.java`,text:`/*
 * Represents a teacher
 */
public class Teacher {
  
  private String[][] studentEssays;    // The 2D array of student essays submitted each week

  /*
   * Initializes studentEssays to the 2D array of
   * student essays submitted each week
   */
  public Teacher(String[][] studentEssays) {
    this.studentEssays = studentEssays;
  }

  /*
   * Returns the 2D array studentEssays
   */
  public String[][] getStudentEssays() {
    return studentEssays;
  }

  /*
   * Returns the student essay that contains any of
   * the phrases in the parameter targetPhrases
   */
  public String findPhrases(String[] targetPhrases) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array studentEssays. For each phrase in studentEssays,
     * check if any of the target phrases can be found. If so, use the essayToString()
     * method to return the 1D array containing the student essay as a String.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return "";
  }

  /*
   * Returns a String containing the phrases that
   * make up a student essay
   */
  public String essayToString(String[] essay) {
    String result = "";

    for (int index = 0; index < essay.length; index++) {
      result += essay[index] + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`TeacherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Teacher.java Test")
public class TeacherTest {

  String messageGap = "\\n       ";
  String[][] testEssays;
  Teacher testObject;
   
  @BeforeEach
  public void setup() {
    testEssays = getTestEssays();
    testObject = new Teacher(testEssays);
  }
   
  @Test
  @Order(1)
  @DisplayName("findPhrases() returns the student essay that contains one of the target phrases => ")
  public void testFindPhrases() {
    String message = "Traverse the 2D array of student essays, then use another nested loop to go through";
    message += "\\n        each target phrase and check if any target phrase can be found in a phrase in a";
    message += "\\n        student essay. If so, return the student essay (the 1D array) as a String." + messageGap;

    String[] phrasesToFind = {"popular", "learn new things", "benefits"};
    String expected = getExpectedResult(phrasesToFind);
    String actual = testObject.findPhrases(phrasesToFind);

    assertEquals(expected, actual, message);
  }

  private String[][] getTestEssays() {
    String[] firstEssay = FileReader.toStringArray("firstEssay.txt");
    String[] secondEssay = FileReader.toStringArray("secondEssay.txt");
    String[] thirdEssay = FileReader.toStringArray("thirdEssay.txt");
    String[][] essays = {firstEssay, secondEssay, thirdEssay};
    return essays;
  }

  private String getExpectedResult(String[] targetPhrases) {
    for (String[] essay : testEssays) {
      for (String phrase : targetPhrases) {
        for (int index = 0; index < targetPhrases.length; index++) {
          if (phrase.indexOf(targetPhrases[index]) != -1) {
            return testObject.essayToString(essay);
          }
        }
      }
    }
    
    return "No essay contains any of the target phrases.";
  }
  
}`}],dataFiles:[{path:`firstEssay.txt`,text:`The benefits of reading are numerous and varied.
Reading can improve our cognitive abilities, including critical thinking and problem-solving skills.
It can also increase our vocabulary, enhance our creativity, and reduce stress.
By making reading a regular part of our lives, we can enjoy these benefits and more.`},{path:`secondEssay.txt`,text:`Basketball was invented in 1891 by James Naismith, a Canadian physical education instructor.
It was initially played with a soccer ball and two peach baskets for hoops.
The game quickly gained popularity and was introduced to the Olympics in 1936.
Today, basketball is one of the most popular sports in the world, with millions of fans and players worldwide.`},{path:`thirdEssay.txt`,text:`Reading is a great way to learn new things and broaden our horizons.
It can improve our cognitive abilities, including critical thinking and problem-solving skills.
It can also increase our vocabulary, enhance our creativity, and reduce stress.
By making reading a regular part of our lives, we can enjoy these benefits and more.`}]},{name:`Skill Building: The indexOf() Method (c)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates several 1D arrays containing the emails from the text files
    String[] firstWeek = FileReader.toStringArray("weekOne.txt");
    String[] secondWeek = FileReader.toStringArray("weekTwo.txt");
    String[] thirdWeek = FileReader.toStringArray("weekThree.txt");

    // Creates a 2D array of emails sent for several weeks
    String[][] customerEmails = {firstWeek, secondWeek, thirdWeek};

    // Creates an Email object
    Email email = new Email(customerEmails);

    // Creates a 1D array of keywords to search for
    String[] keywords = {"order", "confirmation"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findEmail() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

  }
}`},{path:`Email.java`,text:`/*
 * Represents an email provider
 */
public class Email {
  
  private String[][] customerEmails;   // The 2D array containing emails sent over several weeks

  /*
   * Initializes customerEmails to the 2D array containing
   * emails sent over several weeks
   */
  public Email(String[][] customerEmails) {
    this.customerEmails = customerEmails;
  }

  /*
   * Returns the 2D array customerEmails
   */
  public String[][] getCustomerEmails() {
    return customerEmails;
  }

  /*
   * Returns the email that contains a keyword in the 1D array keywords
   */
  public String findEmail(String[] keywords) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array of customer emails. Then go through each keyword
     * in the 1D array keywords and check if any email contains any of the keywords.
     * Return the email that contains any of the keywords.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return "";
  }
  
}`},{path:`FileReader.java`,text:`import java.util.Scanner;
import java.util.ArrayList;
import java.io.File;
import java.io.FileNotFoundException;

public class FileReader {

  public static String[] toStringArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    String[] data = new String[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = currentValue;
    }

    return data;
  }

  public static int[] toIntArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    int[] data = new int[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Integer.parseInt(currentValue);
    }

    return data;
  }

  public static double[] toDoubleArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    double[] data = new double[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Double.parseDouble(currentValue);
    }

    return data;
  }

  private static Scanner createScanner(File theFile) {
    Scanner tempScanner = null;

    try {
      tempScanner = new Scanner(theFile);
    } catch(FileNotFoundException error) {
      System.out.println("File not found.");
    }

    return tempScanner;
  }

  private static ArrayList<String> readDataFromFile(String filename) {
    File dataFile = new File(filename);
    Scanner reader = createScanner(dataFile);
    ArrayList<String> temp = new ArrayList<String>();

    while (reader.hasNextLine()) {
      String currentLine = reader.nextLine().trim();
      temp.add(currentLine);
    }

    reader.close();
    return temp;
  }
  
}`}],validationFiles:[{path:`EmailTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Email.java Test")
public class EmailTest {

  String messageGap = "\\n       ";
  String[][] testEmails;
  Email testObject;
   
  @BeforeEach
  public void setup() {
    testEmails = getTestEmails();
    testObject = new Email(testEmails);
  }
   
  @Test
  @Order(1)
  @DisplayName("findEmail() returns the email that contains keywords in the parameter keywords => ")
  public void testFindEmail() {
    String message = "Traverse the 2D array customerEmails, then go through the keywords array. If the";
    message += "\\n        email contains one of the keywords, return the email." + messageGap;
      
    String[] keywords = {"order", "confirmation"};
    String expected = getExpectedResult(keywords);
    String actual = testObject.findEmail(keywords);

    assertEquals(expected, actual, message);
  }

  private String[][] getTestEmails() {
    String[] firstWeek = FileReader.toStringArray("weekOne.txt");
    String[] secondWeek = FileReader.toStringArray("weekTwo.txt");
    String[] thirdWeek = FileReader.toStringArray("weekThree.txt");
    String[][] customerEmails = {firstWeek, secondWeek, thirdWeek};
    return customerEmails;
  }

  private String getExpectedResult(String[] keywords) {
    for (String[] week : testEmails) {
      for (String email : week) {
        for (int index = 0; index < keywords.length; index++) {
          if (email.indexOf(keywords[index]) != -1) {
            return email;
          }
        }
      }
    }
    
    return "No email contains the specified keywords.";
  }
  
}`}],dataFiles:[{path:`weekOne.txt`,text:`Important: Your order confirmation
Thank you for your purchase
Order #1234
Shipping information
Delivery estimate`},{path:`weekThree.txt`,text:`New feature: Product recommendations
Check out our latest products
Limited time offer
Save on your next purchase
Exclusive discount code`},{path:`weekTwo.txt`,text:`Reminder: Upcoming appointment
Don't forget to bring your ID
Location and time
What to expect
Cancel or reschedule`}]},{name:`Skill Building: The indexOf() Method (d)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates 1D arrays for the search results
    String[] pancakesResults = FileReader.toStringArray("pancakes.txt");
    String[] coffeeResults = FileReader.toStringArray("coffee.txt");
    String[] yogaResults = FileReader.toStringArray("yoga.txt");

    // Creates a 2D array of all the search results
    String[][] searchResults = {pancakesResults, coffeeResults, yogaResults};

    // Creates a Results object
    Results searches = new Results(searchResults);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findSearchResult() method and print the results.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`FileReader.java`,text:`import java.util.Scanner;
import java.util.ArrayList;
import java.io.File;
import java.io.FileNotFoundException;

public class FileReader {

  public static String[] toStringArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    String[] data = new String[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = currentValue;
    }

    return data;
  }

  public static int[] toIntArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    int[] data = new int[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Integer.parseInt(currentValue);
    }

    return data;
  }

  public static double[] toDoubleArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    double[] data = new double[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Double.parseDouble(currentValue);
    }

    return data;
  }

  private static Scanner createScanner(File theFile) {
    Scanner tempScanner = null;

    try {
      tempScanner = new Scanner(theFile);
    } catch(FileNotFoundException error) {
      System.out.println("File not found.");
    }

    return tempScanner;
  }

  private static ArrayList<String> readDataFromFile(String filename) {
    File dataFile = new File(filename);
    Scanner reader = createScanner(dataFile);
    ArrayList<String> temp = new ArrayList<String>();

    while (reader.hasNextLine()) {
      String currentLine = reader.nextLine().trim();
      temp.add(currentLine);
    }

    reader.close();
    return temp;
  }
  
}`},{path:`Results.java`,text:`/*
 * Represents search results
 */
public class Results {
  
  private String[][] searchResults;     // The 2D array of search results on several pages

  /*
   * Initializes searchResults to the 2D array of search results
   */
  public Results(String[][] searchResults) {
    this.searchResults = searchResults;
  }

  /*
   * Returns the 2D array of search results
   */
  public String[][] getSearchResults() {
    return searchResults;
  }

  /*
   * Returns the search results that contains the query
   */
  public String findSearchResult(String query) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array searchResults. If the result contains the query,
     * return the result. If no results are found, return a String letting the
     * user know that no results were found.
     * -----------------------------------------------------------------------------
     */
    
    

    return "";
  }
  
}`}],validationFiles:[{path:`ResultsTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Results.java Test")
public class ResultsTest {

  String messageGap = "\\n       ";
  String[][] testResults;
  Results testObject;
   
  @BeforeEach
  public void setup() {
    testResults = getTestResults();
    testObject = new Results(testResults);
  }
   
  @Test
  @Order(1)
  @DisplayName("findSearchResult() returns the first result that contains the query => ")
  public void testFindSearchResult() {
    String message = "Traverse the 2D array of search results. Return the first search result that contains the query." + messageGap;
      
    String expected = getExpectedResult("benefits of yoga");
    String actual = testObject.findSearchResult("benefits of yoga");

    assertEquals(expected, actual, message);
  }

  private String[][] getTestResults() {
    String[] pancakesResults = FileReader.toStringArray("pancakes.txt");
    String[] coffeeResults = FileReader.toStringArray("coffee.txt");
    String[] yogaResults = FileReader.toStringArray("yoga.txt");
    String[][] searchResults = {pancakesResults, coffeeResults, yogaResults};
    return searchResults;
  }

  private String getExpectedResult(String query) {
    for (int i = 0; i < testResults.length; i++) {
      for (int j = 0; j < testResults[0].length; j++) {
        String snippet = testResults[i][j];
        
        if (snippet.indexOf(query) != -1) {
          return snippet;
        }
      }
    }

    return "No search results containing " + query;
  }
  
}`}],dataFiles:[{path:`coffee.txt`,text:`Best coffee shops in town
Find the perfect coffee shop for studying, working, or just hanging out
How to brew the perfect cup of coffee at home
Coffee vs. tea: which is better for your health?
10 creative coffee drink recipes to try at home`},{path:`pancakes.txt`,text:`How to make pancakes
Learn the secret to fluffy pancakes with this easy recipe
Pancake mix vs. homemade pancakes: which is better?
Gluten-free pancake recipes
Top 10 pancake toppings to try`},{path:`yoga.txt`,text:`Yoga for beginners
Learn the basics of yoga with these easy poses
The benefits of yoga for mental health
How to find the right yoga class for you
The best yoga mats for home practice`}]},{name:`Practice: 2D Array Algorithms (a)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of sentences
    String[][] sentences = { {"The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"},
                             {"She", "sells", "seashells", "by", "the", "seashore"},
                             {"The", "cat", "in", "the", "hat", "went", "to", "the", "mat"},
                             {"To", "be", "or", "not", "to", "be", "that", "is", "the", "question"} };

    // Creates a TextProcessor object
    TextProcessor document = new TextProcessor(sentences);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countNumWords() method and print the result.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`TextProcessor.java`,text:`/*
 * Analyzes document text
 */
public class TextProcessor {
  
  private String[][] documentText;     // The 2D array containing the sentences of a document

  /*
   * Initializes documentText to the specified 2D array
   * containing the sentences of a document
   */
  public TextProcessor(String[][] documentText) {
    this.documentText = documentText;
  }

  /*
   * Returns the 2D array documentText
   */
  public String[][] getDocumentText() {
    return documentText;
  }

  /*
   * Returns the number of words in the document text that have the
   * same length as targetLength and start with the letter startsWith
   */
  public int countNumWords(int targetLength, String startsWith) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 2D array documentText. Count and return the number of
     * words in documentText that have the same length as targetLength and where
     * the location of startsWith is at index 0 in the String.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }
  
}`}],validationFiles:[{path:`TextProcessorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TextProcessor.java Test")
public class TextProcessorTest {

  String messageGap = "\\n       ";
  String[][] testSentences;
  TextProcessor testObject;
   
  @BeforeEach
  public void setup() {
    testSentences = getTestSentences();
    testObject = new TextProcessor(testSentences);
  }
   
  @Test
  @Order(1)
  @DisplayName("countNumWords() returns the number of words that are targetLength and first letter is startsWith => ")
  public void testCountNumWords() {
    String message = "Check if each word in the sentences have the same length as targetLength and where";
    message += "\\n        the first letter is startsWith (letter is at index 0)." + messageGap;
      
    int expected = getExpectedResult(3, "t");
    int actual = testObject.countNumWords(3, "t");

    assertEquals(expected, actual, message);
  }

  private String[][] getTestSentences() {
    String[][] sentences = { {"The", "quick", "brown", "fox", "jumps", "over", "the", "lazy", "dog"},
                             {"She", "sells", "seashells", "by", "the", "seashore"},
                             {"The", "cat", "in", "the", "hat", "went", "to", "the", "mat"},
                             {"To", "be", "or", "not", "to", "be", "that", "is", "the", "question"} };
    return sentences;
  }

  private int getExpectedResult(int targetLength, String startsWith) {
    int numWords = 0;
    
    for (String[] sentence : testSentences) {
      for (String word : sentence) {        
        if (word.length() == targetLength && word.indexOf(startsWith) == 0) {
          numWords++;
        }
      }
    }
    
    return numWords;
  }
   
}`}],dataFiles:[]},{name:`Practice: 2D Array Algorithms (b)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array containing sentences by an author
    String[][] writings = {{"I", "know", "why", "the", "caged", "bird", "sings"},
                           {"You", "may", "write", "me", "down", "in", "history"},
                           {"I", "mean", "precisely", "the", "opposite", "of", "that"}};

    // Creates a Literature object
    Literature author = new Literature(writings);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countNumWords() method and prints the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Literature.java`,text:`/*
 * Represents a literary work
 */
public class Literature {

  private String[][] authorWritings;    // The 2D array containing sentences from multiple works

  /*
   * Initializes writings to the specified 2D array
   * containing sentences from multiple works
   */
  public Literature(String[][] authorWritings) {
    this.authorWritings = authorWritings;
  }

  /*
   * Returns the 2D array authorWritings
   */
  public String[][] getAuthorWritings() {
    return authorWritings;
  }

  /*
   * Returns the number of words that start with targetLetter
   */
  public int countNumWords(String targetLetter) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Counts and returns the number of words in authorWritings that start
     * with the parameter targetLetter.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`LiteratureTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Literature.java Test")
public class LiteratureTest {

  String messageGap = "\\n       ";
  String[][] testSentences;
  Literature testObject;
   
  @BeforeEach
  public void setup() {
    testSentences = new String[][]{{"I", "know", "why", "the", "caged", "bird", "sings"},
                                   {"You", "may", "write", "me", "down", "in", "history"},
                                   {"I", "mean", "precisely", "the", "opposite", "of", "that"}};
    testObject = new Literature(testSentences);
  }
   
  @Test
  @Order(1)
  @DisplayName("countNumWords() returns the number of words that starts with targetLetter => ")
  public void testCountNumWords() {
    String message = "Count and return the number of words that start with targetLetter (index = 0)." + messageGap;
    int actual = testObject.countNumWords("m");
    assertEquals(3, actual, message);
  }
   
}`}],dataFiles:[]},{name:`Practice: 2D Array Algorithms (c)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of paragraphs
    String[][] phrases = { {"Bonjour!", "Comment allez-vous?", "Je m'appelle Jacques."},
                           {"Hola!", "Cómo estás?", "Mi nombre es Maria."},
                           {"Hello!", "How are you?", "My name is John."} };

    // Creates a Translator object
    Translator text = new Translator(phrases);

    // Creates a 1D array of target words
    String[] targetWords = {"Comment", "John"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findSentences() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Translator.java`,text:`/*
 * Handles translating phrases
 */
public class Translator {

  private String[][] phrases;   // The 2D array of phrases to translate

  /*
   * Initializes phrases to the specified2D array of phrases
   */
  public Translator(String[][] phrases) {
    this.phrases = phrases;
  }

  /*
   * Returns the 2D array phrases
   */
  public String[][] getPhrases() {
    return phrases;
  }

  /*
   * Returns a String containing the sentences that
   * contain at least one of the words in targetWords
   */
  public String findSentences(String[] targetWords) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Write the findSentences() method to find and return the sentences that 
     *    contain at least one of the words in the parameter targetWords.
     * -----------------------------------------------------------------------------
     */

    

    return "";
  }

}`}],validationFiles:[{path:`TranslatorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Translator.java Test")
public class TranslatorTest {

  String messageGap = "\\n       ";
  String[][] testPhrases;
  Translator testObject;
   
  @BeforeEach
  public void setup() {
    testPhrases = new String[][]{ {"Bonjour!", "Comment allez-vous?", "Je m'appelle Jacques."},
                                  {"Hola!", "Cómo estás?", "Mi nombre es Maria."},
                                  {"Hello!", "How are you?", "My name is John."} };
    testObject = new Translator(testPhrases);
  }
   
  @Test
  @Order(1)
  @DisplayName("findSentences() returns the sentences that contain at least one target word => ")
  public void testFindSentences() {
    String message = "Traverse the 2D array phrases, then go through targetWords and check if";
    message += "\\n        each word can be found in a sentence. Return the number of sentences";
    message += "\\n        that contain at least one target word." + messageGap;
      
    String[] targetWords = {"Comment", "John"};
    String expected = getExpectedResult(targetWords);
    String actual = testObject.findSentences(targetWords);

    assertEquals(expected, actual, message);
  }

  private String getExpectedResult(String[] targetWords) {
    String result = "";

    for (String[] sentences : testPhrases) {
      for (String sentence : sentences) {
        for (int index = 0; index < targetWords.length; index++) {
          if (sentence.indexOf(targetWords[index]) != -1) {
            result += sentence + "\\n";
          }
        }
      }
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: 2D Array Algorithms (d)`,lesson:`Lesson 7: 2D Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of social media posts
    String[][] socialMediaPosts = {{"Check out our new #product!", "Get it now and receive a 10% #discount!"},
                                   {"Just posted a new #blog article!", "Learn how to save #money with our tips and tricks."},
                                   {"#HappyFriday everyone!", "What are your plans for the #weekend?"}};

    // Creates a Marketing object
    Marketing campaign = new Marketing(socialMediaPosts);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countNumTimes() and calcPercentage() methods and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Marketing.java`,text:`/*
 * Analyzes social media posts
 */
public class Marketing {

  private String[][] socialMediaPosts;    // The 2D array of posts made for several weeks

  /*
   * Initializes socialMediaPosts to the specified 2D array
   * containing posts made for several weeks
   */
  public Marketing(String[][] socialMediaPosts) {
    this.socialMediaPosts = socialMediaPosts;
  }

  /*
   * Returns the 2D array socialMediaPosts
   */
  public String[][] getSocialMediaPosts() {
    return socialMediaPosts;
  }

  /*
   * Returns the number of occurrences of targetLetter in socialMediaPosts
   */
  public int countNumTimes(String targetLetter) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Count the number of times targetLetter can be found in all posts.
     * -----------------------------------------------------------------------------
     */
    
    

    return -1;
  }

  /*
   * Calculates and returns the percentage of targetLetter
   * compared to the total length of all posts
   */
  public double calcPercentage(int numTimes) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find the total length of all posts, then divide numTimes by the total
     * length. Return the result of this times 100 to get the percentage.
     * -----------------------------------------------------------------------------
     */
    
    

    return -1;
  }
  
}`}],validationFiles:[{path:`MarketingTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Marketing.java Test")
public class MarketingTest {

  String messageGap = "\\n       ";
  String[][] testPosts;
  Marketing testObject;
   
  @BeforeEach
  public void setup() {
    testPosts = new String[][]{{"Check out our new #product!", "Get it now and receive a 10% #discount!"},
                               {"Just posted a new #blog article!", "Learn how to save #money with our tips and tricks."},
                               {"#HappyFriday everyone!", "What are your plans for the #weekend?"}};
    testObject = new Marketing(testPosts);
  }
   
  @Test
  @Order(1)
  @DisplayName("countNumTimes() returns the number of occurrences of targetLetter in all posts => ")
  public void testCountNumTimes() {
    String message = "Count the number of times targetLetter can be found in all posts." + messageGap;
      
    int expected = getExpectedCount("#");
    int actual = testObject.countNumTimes("#");

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("calcPercentage() returns the percentage that targetLetter appears compared to the total length of all posts => ")
  public void testCalcPercentage() {
    String message = "Find the total length of all posts. Divide the parameter numTimes by the total length, then";
    message += "\\n        multiply this result by 100 to get the percentage." + messageGap;

    int expectedCount = getExpectedCount("#");
    double expected = getExpectedPercentage(expectedCount);
    double actual = testObject.calcPercentage(expectedCount);

    assertEquals(expected, actual, message);
  }

  private int getExpectedCount(String targetLetter) {
    int numLetter = 0;
    
    for (String[] week : testPosts) {
      for (String post : week) {
        if (post.indexOf(targetLetter) != -1) {
          numLetter++;
        }
      }
    }

    return numLetter;
  }

  public double getExpectedPercentage(int numTimes) {
    double totalLength = 0;

    for (String[] week : testPosts) {
      for (String post : week) {
        totalLength += post.length();
      }
    }

    return numTimes / totalLength * 100;
  }
   
}`}],dataFiles:[]},{name:`Predict and Run: Image Filters`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    Scene myScene = new Scene();
    SepiaImage myImage = new SepiaImage("paintbrushes.jpg");

    myScene.drawImage(myImage, 0, 0, myScene.getWidth());
    myScene.pause(0.5);

    myImage.makeSepia();

    myScene.drawImage(myImage, 0, 0, myScene.getWidth());

    Theater.playScenes(myScene);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`SepiaImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a sepia image
 */
public class SepiaImage extends ImagePlus {

  /*
   * Constructor to create a SepiaImage
   * with the specified file name
   */
  public SepiaImage(String filename) {
    super(filename);
  }

  /*
   * Applies a sepia filter to the image
   */
  public void makeSepia() {
    Pixel[][] pixels = getPixelsFromImage();

    for (int row = 0; row < pixels.length; row++) {
      for (int col = 0; col < pixels[0].length; col++) {
        Pixel currentPixel = pixels[row][col];

        int currentRed = currentPixel.getRed();
        int currentGreen = currentPixel.getGreen();
        int currentBlue = currentPixel.getBlue();

        int newRed = (int)((0.393 * currentRed) + (0.769 * currentGreen) + (0.189 * currentBlue));
        int newGreen = (int)((0.349 * currentRed) + (0.686 * currentGreen) + (0.168 * currentBlue));
        int newBlue = (int)((0.272 * currentRed) + (0.534 * currentGreen) + (0.131 * currentBlue));

        currentPixel.setRed(newRed);
        currentPixel.setGreen(newGreen);
        currentPixel.setBlue(newBlue);
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Modifying Colors (a)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a NegativeImage object using an image from the Assets Manager
     * or your own image. Draw the image in the scene without a filter, then call
     * the makeNegative() method and draw the image in the scene with a filter.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`NegativeImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a negative image
 */
public class NegativeImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public NegativeImage(String filename) {
    super(filename);
  }

  /*
   * Inverts the colors in the image by setting the red,
   * green, and blue color values of each Pixel object to
   * the result of 255 minus their current values
   */
  public void makeNegative() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, then traverse the 2D array of pixels. Set
     * the red, green, and blue color values of each Pixel object to the result of
     * 255 minus the current values.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`}],validationFiles:[{path:`NegativeImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NegativeImage.java Test")
public class NegativeImageTest {

  String messageGap = "\\n       ";
  NegativeImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new NegativeImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("makeNegative() sets the red value of each Pixel object to the result of 255 minus current red => ")
  public void testMakeNegativeModifiesRed() {
    String message = "Set the red value of each Pixel object to the result of 255 minus the current red value." + messageGap;

    testObject.makeNegative();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = 255 - originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("makeNegative() sets the green value of each Pixel object to the result of 255 minus current green => ")
  public void testMakeNegativeModifiesGreen() {
    String message = "Set the green value of each Pixel object to the result of 255 minus the current green value." + messageGap;

    testObject.makeNegative();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = 255 - originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("makeNegative() sets the blue value of each Pixel object to the result of 255 minus current blue => ")
  public void testMakeNegativeModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the result of 255 minus the current blue value." + messageGap;

    testObject.makeNegative();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = 255 - originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
  
}`}],dataFiles:[]},{name:`Practice: Modifying Colors (b)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a PixelatedImage object, and draw the image in the scene
     * without any filters. Call the pixelate() method, then draw the image again
     * in the scene.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`PixelatedImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a pixelated image
 */
public class PixelatedImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public PixelatedImage(String filename) {
    super(filename);
  }

  /*
   * Applies a pixelate filter to each Pixel object by dividing the image into a grid
   * of equal-sized rectangular regions and setting the color of each Pixel object in
   * a region to the color of the first Pixel object in the region
   */
  public void pixelate(int gridSize) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse each row of pixels by the gridSize amount (instead of by 1), then
     * traverse each column of pixels by the gridSize amount (instead of by 1). Get
     * the average red, green, and blue values of the pixels in that region, and
     * set each red, green, and blue values in that region to the averages.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[{path:`PixelatedImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PixelatedImage.java Test")
public class PixelatedImageTest {

  String messageGap = "\\n       ";
  PixelatedImage testObject;
  PixelatedImage originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new PixelatedImage(randomImage);
    originalObject = new PixelatedImage(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("pixelate() sets the red value of Pixel objects in a region to the average red value in the region => ")
  public void testPixelateModifiesRed() {
    String message = "Set the red value of Pixel objects in a region to the average red value in the region." + messageGap;

    getExpectedResult(20);
    testObject.pixelate(20);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("pixelate() sets the green value of Pixel objects in a region to the average green value in the region => ")
  public void testPixelateModifiesGreen() {
    String message = "Set the green value of Pixel objects in a region to the average green value in the region." + messageGap;

    getExpectedResult(20);
    testObject.pixelate(20);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("pixelate() sets the blue value of Pixel objects in a region to the average blue value in the region => ")
  public void testPixelateModifiesBlue() {
    String message = "Set the blue value of Pixel objects in a region to the average blue value in the region." + messageGap;

    getExpectedResult(20);
    testObject.pixelate(20);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult(int gridSize) {
    Pixel[][] pixels = originalObject.getImagePixels();

    for (int row = 0; row < pixels.length; row += gridSize) {
      for (int col = 0; col < pixels[0].length; col += gridSize) {
        int endRow = Math.min(row + gridSize, pixels.length);
        int endCol = Math.min(col + gridSize, pixels[0].length);
        int avgRed = 0;
        int avgGreen = 0;
        int avgBlue = 0;

        for (int regionRow = row; regionRow < endRow; regionRow++) {
          for (int regionCol = col; regionCol < endCol; regionCol++) {
            avgRed += pixels[regionRow][regionCol].getRed();
            avgGreen += pixels[regionRow][regionCol].getGreen();
            avgBlue += pixels[regionRow][regionCol].getBlue();
          }
        }

        int totalPixelsInRegion = (endRow - row) * (endCol - col);
        avgRed /= totalPixelsInRegion;
        avgGreen /= totalPixelsInRegion;
        avgBlue /= totalPixelsInRegion;

        for (int regionRow = row; regionRow < endRow; regionRow++) {
          for (int regionCol = col; regionCol < endCol; regionCol++) {
            pixels[regionRow][regionCol].setRed(avgRed);
            pixels[regionRow][regionCol].setGreen(avgGreen);
            pixels[regionRow][regionCol].setBlue(avgBlue);
          }
        }
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Modifying Colors (c)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a ShiftedImage object, and draw the image in the scene with
     * no filters. Then call the colorShift() method, and draw the image in the
     * scene again with the filter.
     * -----------------------------------------------------------------------------
     */

    

    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`ShiftedImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a color shifted image
 */
public class ShiftedImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public ShiftedImage(String filename) {
    super(filename);
  }

  /*
   * Shifts the color of each Pixel object by a fixed amount
   */
  public void colorShift(int value) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Add the parameter value to each of the red, green, and blue color values
     * of each Pixel object.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`}],validationFiles:[{path:`ShiftedImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ShiftedImage.java Test")
public class ShiftedImageTest {

  String messageGap = "\\n       ";
  ShiftedImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new ShiftedImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("colorShift() sets the red value of each Pixel object to the result of current red plus value => ")
  public void testColorShiftModifiesRed() {
    String message = "Set the red value of each Pixel object to the result of the current red plus value." + messageGap;

    int randomValue = (int)(Math.random() * 50) + 10;
    testObject.colorShift(randomValue);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed() + randomValue;
        expectedRed = Math.min(255, expectedRed);
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("colorShift() sets the green value of each Pixel object to the result of current green plus value => ")
  public void testColorShiftModifiesGreen() {
    String message = "Set the green value of each Pixel object to the result of the current green plus value." + messageGap;

    int randomValue = (int)(Math.random() * 50) + 10;
    testObject.colorShift(randomValue);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen() + randomValue;
        expectedGreen = Math.min(255, expectedGreen);
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("colorShift() sets the blue value of each Pixel object to the result of current blue plus value => ")
  public void testColorShiftModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the result of the current blue plus value." + messageGap;

    int randomValue = (int)(Math.random() * 50) + 10;
    testObject.colorShift(randomValue);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue() + randomValue;
        expectedBlue = Math.min(255, expectedBlue);
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
  
}`}],dataFiles:[]},{name:`Practice: Modifying Colors (d)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a ContrastImage object, and draw the image in the scene with
     * no filters. Then call the adjustContrast() method, and draw the image in the
     * scene again with the filter.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ContrastImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an high or low contrast image
 */
public class ContrastImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public ContrastImage(String filename) {
    super(filename);
  }

  /*
   * Adjusts the contrast of the image by multiplying the
   * red, green, and blue values by the multiplier
   */
  public void adjustContrast(int multiplier) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Multiply the red, green, and blue values of each Pixel by the multiplier,
     * and set the color values to the result.
     * -----------------------------------------------------------------------------
     */

    

    
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`}],validationFiles:[{path:`ContrastImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ContrastImage.java Test")
public class ContrastImageTest {

  String messageGap = "\\n       ";
  ContrastImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new ContrastImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("adjustContrast() sets the red value of each Pixel to the result of current red times the multiplier => ")
  public void testAdjustContrastModifiesRed() {
    String message = "Set the red value of each Pixel object to the result of current red times the multiplier." + messageGap;

    testObject.adjustContrast(10);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed() * 10;
        expectedRed = Math.min(255, expectedRed);
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("adjustContrast() sets the green value of each Pixel to the result of current green times the multiplier => ")
  public void testAdjustContrastModifiesGreen() {
    String message = "Set the green value of each Pixel object to the result of current green times the multiplier." + messageGap;

    testObject.adjustContrast(10);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen() * 10;
        expectedGreen = Math.min(255, expectedGreen);
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("adjustContrast() sets the blue value of each Pixel to the result of current blue times the multiplier => ")
  public void testAdjustContrastModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the result of current blue times the multiplier." + messageGap;

    testObject.adjustContrast(10);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue() * 10;
        expectedBlue = Math.min(255, expectedBlue);
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Images (a)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a BlurImage object, then draw the image in the scene with no
     * filter. Then call the applyBlur() method, and draw the image in the scene
     * again with the filter.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`BlurImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a blurred image
 */
public class BlurImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public BlurImage(String filename) {
    super(filename);
  }

  /*
   * Applies a Gaussian blur by finding the average of the red,
   * green, and blue color values of the current Pixel object and
   * its top-left neighboring Pixel object using a weighted average
   */
  public void applyBlur() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, then traverse the 2D array of pixels. Use
     * the provided calcWeightedRed(), calcWeightedGreen(), and calcWeightedBlue()
     * methods to get the weighted averages and set each Pixel color to the result.
     * -----------------------------------------------------------------------------
     */



    
  }

  /*
   * Returns a weighted red average of the pixels around the specified row and col
   */
  public int calcWeightedRed(Pixel[][] pixels, int row, int col) {
    int avgRed = (pixels[row-1][col-1].getRed() + pixels[row-1][col].getRed() + pixels[row-1][col+1].getRed() +
                  pixels[row][col-1].getRed() + pixels[row][col].getRed() + pixels[row][col+1].getRed() +
                  pixels[row+1][col-1].getRed() + pixels[row+1][col].getRed() + pixels[row+1][col+1].getRed()) / 9;
    return avgRed;
  }

  /*
   * Returns a weighted green average of the pixels around the specified row and col 
   */
  public int calcWeightedGreen(Pixel[][] pixels, int row, int col) {
    int avgGreen = (pixels[row-1][col-1].getGreen() + pixels[row-1][col].getGreen() + pixels[row-1][col+1].getGreen() +
                    pixels[row][col-1].getGreen() + pixels[row][col].getGreen() + pixels[row][col+1].getGreen() +
                    pixels[row+1][col-1].getGreen() + pixels[row+1][col].getGreen() + pixels[row+1][col+1].getGreen()) / 9;
    return avgGreen;
  }

  /*
   * Returns a weighted blue average of the pixels around the specified row and col
   */
  public int calcWeightedBlue(Pixel[][] pixels, int row, int col) {
    int avgBlue = (pixels[row-1][col-1].getBlue() + pixels[row-1][col].getBlue() + pixels[row-1][col+1].getBlue() +
                   pixels[row][col-1].getBlue() + pixels[row][col].getBlue() + pixels[row][col+1].getBlue() +
                   pixels[row+1][col-1].getBlue() + pixels[row+1][col].getBlue() + pixels[row+1][col+1].getBlue()) / 9;
    return avgBlue;
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`}],validationFiles:[{path:`BlurImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BlurImage.java Test")
public class BlurImageTest {

  String messageGap = "\\n       ";
  BlurImage testObject;
  BlurImage originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new BlurImage(randomImage);
    originalObject = new BlurImage(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("applyBlur() sets the red value of each Pixel object to the weighted average of surrounding red values => ")
  public void testApplyBlurModifiesRed() {
    String message = "Set the red value of each Pixel object to the weighted red value returned from calling calcWeightedRed()." + messageGap;

    getExpectedResult();
    testObject.applyBlur();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("applyBlur() sets the green value of each Pixel object to the weighted average of surrounding green values => ")
  public void testApplyBlurModifiesGreen() {
    String message = "Set the green value of each Pixel object to the weighted green value returned from calling calcWeightedGreen()." + messageGap;

    getExpectedResult();
    testObject.applyBlur();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("applyBlur() sets the blue value of each Pixel object to the weighted average of surrounding blue values => ")
  public void testApplyBlurModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the weighted blue value returned from calling calcWeightedBlue()." + messageGap;

    getExpectedResult();
    testObject.applyBlur();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult() {
    for (int row = 1; row < originalPixels.length - 1; row++) {
      for (int col = 1; col < originalPixels[0].length - 1; col++) {
        int weightedRed = originalObject.calcWeightedRed(originalPixels, row, col);
        int weightedGreen = originalObject.calcWeightedGreen(originalPixels, row, col);
        int weightedBlue = originalObject.calcWeightedBlue(originalPixels, row, col);

        Pixel currentPixel = originalPixels[row][col];
        currentPixel.setRed(weightedRed);
        currentPixel.setGreen(weightedGreen);
        currentPixel.setBlue(weightedBlue);
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Images (b)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a SepiaImage object, then draw the image in the scene with no
     * filter. Add a pause(), call the applySepia() method, and draw the image again
     * with the filter.
     * -----------------------------------------------------------------------------
     */



    
    // Plays the scene
    Theater.playScenes(images);
    
  }
}
`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`SepiaImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a sepia-toned image
 */
public class SepiaImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public SepiaImage(String filename) {
    super(filename);
  }

  /*
   * Applies a sepia filter by calculating new red, green, and blue color
   * values based on sepia tone formulas and setting the Pixel object to the result
   */
  public void applySepia() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * For each Pixel object, get the red, green, and blue values and calculate
     * the new color values using the sepia formulas. Clamp the values to be between
     * 0 and 255 using if statements. Set the color values to the sepia result.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}
`}],validationFiles:[{path:`SepiaImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SepiaImage.java Test")
public class SepiaImageTest {

  String messageGap = "\\n       ";
  SepiaImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;

  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new SepiaImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }

  @Test
  @Order(1)
  @DisplayName("applySepia() modifies red color values using sepia formula => ")
  public void testRedSepiaValues() {
    String message = "Use the sepia formula to calculate and update the red value." + messageGap;
    getExpectedResult();
    testObject.applySepia();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("applySepia() modifies green color values using sepia formula => ")
  public void testGreenSepiaValues() {
    String message = "Use the sepia formula to calculate and update the green value." + messageGap;
    getExpectedResult();
    testObject.applySepia();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("applySepia() modifies blue color values using sepia formula => ")
  public void testBlueSepiaValues() {
    String message = "Use the sepia formula to calculate and update the blue value." + messageGap;
    getExpectedResult();
    testObject.applySepia();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {
      "astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg",
      "dog.jpg", "donut.jpg", "electricguitars.jpg", "fashion.jpg",
      "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg",
      "motivation.jpg", "nature.jpg", "paintbrushes.jpg", "soccer.jpg",
      "speakers.jpg", "starwars.jpg", "track.jpg"
    };
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult() {
    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        Pixel pixel = originalPixels[row][col];
        int red = pixel.getRed();
        int green = pixel.getGreen();
        int blue = pixel.getBlue();

        int newRed = (int)(0.393 * red + 0.769 * green + 0.189 * blue);
        int newGreen = (int)(0.349 * red + 0.686 * green + 0.168 * blue);
        int newBlue = (int)(0.272 * red + 0.534 * green + 0.131 * blue);

        if (newRed > 255) newRed = 255;
        if (newRed < 0) newRed = 0;
        if (newGreen > 255) newGreen = 255;
        if (newGreen < 0) newGreen = 0;
        if (newBlue > 255) newBlue = 255;
        if (newBlue < 0) newBlue = 0;

        pixel.setRed(newRed);
        pixel.setGreen(newGreen);
        pixel.setBlue(newBlue);
      }
    }
  }
}
`}],dataFiles:[]},{name:`Practice: Manipulating Images (c)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a MirrorImage object, and draw the image in the scene with
     * no filters. Then call the mirrorVertical() method and draw the image in the
     * scene again with the filter.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`MirrorImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a mirrored image
 */
public class MirrorImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public MirrorImage(String filename) {
    super(filename);
  }

  /*
   * Mirrors the image vertically
   */
  public void mirrorVertical() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, then traverse each row of Pixel objects
     * and the left half of the columns. Set the color of the Pixel object on the
     * left side of the image to the color of the Pixel object on the right side.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`}],validationFiles:[{path:`MirrorImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MirrorImage.java Test")
public class MirrorImageTest {

  String messageGap = "\\n       ";
  MirrorImage testObject;
  MirrorImage originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new MirrorImage(randomImage);
    originalObject = new MirrorImage(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("mirrorVertical() sets the color of the left Pixel object to the color of the Pixel object on the right => ")
  public void testMirrorVertical() {
    String message = "Set the color each left Pixel object to the color of the Pixel object on the right side of the image." + messageGap;

    getExpectedResult();
    testObject.mirrorVertical();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int expectedGreen = originalPixels[row][col].getGreen();
        int expectedBlue = originalPixels[row][col].getBlue();
        
        int actualRed = actual[row][col].getRed();
        int actualGreen = actual[row][col].getGreen();
        int actualBlue = actual[row][col].getBlue();
        
        assertEquals(expectedRed, actualRed, message);
        assertEquals(expectedGreen, actualGreen, message);
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult() {
    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length / 2; col++) {
        Pixel leftPixel = originalPixels[row][col];
        Pixel rightPixel = originalPixels[row][originalPixels[0].length - col - 1];
        leftPixel.setColor(rightPixel.getColor());
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Images (d)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a BlurImage object, and draw the image in the scene with no
     * filter. Then call the motionBlur() method, and draw the image again with
     * the filter.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`BlurImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a blurred image
 */
public class BlurImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public BlurImage(String filename) {
    super(filename);
  }

  /*
   * Applies a motion blur to the image
   */
  public void motionBlur(int length, String direction) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Find the average of the red, green, and blue color values of the current
     * Pixel object and a set of neighboring Pixel objects in a line segment.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`}],validationFiles:[{path:`BlurImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BlurImage.java Test")
public class BlurImageTest {

  String messageGap = "\\n       ";
  BlurImage testObject;
  BlurImage originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new BlurImage(randomImage);
    originalObject = new BlurImage(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("motionBlur() sets the red value of each Pixel object to the average in a line segment => ")
  public void testMotionBlurModifiesRed() {
    String message = "Set the red value of each Pixel object to the average in a line segment." + messageGap;

    int randomLength = (int)(Math.random() * 10) + 1;
    getExpectedResult(randomLength, "horizontal");
    testObject.motionBlur(randomLength, "horizontal");
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("motionBlur() sets the green value of each Pixel object to the average in a line segment => ")
  public void testMotionBlurModifiesGreen() {
    String message = "Set the green value of each Pixel object to the average in a line segment." + messageGap;

    int randomLength = (int)(Math.random() * 10) + 1;
    getExpectedResult(randomLength, "horizontal");
    testObject.motionBlur(randomLength, "horizontal");
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("motionBlur() sets the blue value of each Pixel object to the average in a line segment => ")
  public void testMotionBlurModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the average in a line segment." + messageGap;

    int randomLength = (int)(Math.random() * 10) + 1;
    getExpectedResult(randomLength, "horizontal");
    testObject.motionBlur(randomLength, "horizontal");
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult(int length, String direction) {
    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        int totalRed = 0;
        int totalGreen = 0;
        int totalBlue = 0;

        int x = col;
        int y = row;
        int count = 0;

        while (count < length && x < originalPixels[0].length && y < originalPixels.length) {
          Pixel currentPixel = originalPixels[y][x];
          totalRed += currentPixel.getRed();
          totalGreen += currentPixel.getGreen();
          totalBlue += currentPixel.getBlue();
          count++;

          if (direction.equals("horizontal")) {
            x++;
          }
          else if (direction.equals("vertical")) {
            y++;
          }
          else if (direction.equals("diagonal")) {
            x++;
            y++;
          }
        }

        int avgRed = totalRed / count;
        int avgGreen = totalGreen / count;
        int avgBlue = totalBlue / count;

        Pixel currentPixel = originalPixels[row][col];
        currentPixel.setRed(avgRed);
        currentPixel.setGreen(avgGreen);
        currentPixel.setBlue(avgBlue);
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Pixels and Images (a)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a SaturateImage object, and draw the image in the scene with
     * no filter. Call the saturate() method, then draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`SaturateImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a saturated image
 */
public class SaturateImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public SaturateImage(String filename) {
    super(filename);
  }

  /*
   * Applies a saturation filter to the image
   */
  public void saturate(int factor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Calculate the average of the red, green, and blue values of each Pixel object,
     * then calculate the adjusted grayscale value. Use this result to determine the
     * value to set the red, green, and blue values. If any of these are over 255, set
     * them to 255. Set the red, green, and blue color values of each Pixel to the result.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`}],validationFiles:[{path:`SaturateImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SaturateImage.java Test")
public class SaturateImageTest {

  String messageGap = "\\n       ";
  SaturateImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new SaturateImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("saturate() sets the red value of each Pixel object to the expected result => ")
  public void testSaturateModifiesRed() {
    String message = "Set the red value of each Pixel object to the expected result." + messageGap;

    getExpectedResult(10);
    testObject.saturate(10);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("saturate() sets the green value of each Pixel object to the expected result => ")
  public void testSaturateModifiesGreen() {
    String message = "Set the green value of each Pixel object to the expected result." + messageGap;

    getExpectedResult(10);
    testObject.saturate(10);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("saturate() sets the blue value of each Pixel object to the expected result => ")
  public void testSaturateModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the expected result." + messageGap;

    getExpectedResult(10);
    testObject.saturate(10);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult(int factor) {
    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        Pixel currentPixel = originalPixels[row][col];
        int grayValue = currentPixel.getRed() + currentPixel.getGreen() + currentPixel.getBlue();
        grayValue /= 3;

        int newGrayValue = (grayValue + (grayValue - 255) * factor);
        
        int redValue = 2 * newGrayValue - currentPixel.getRed();
        int greenValue = 2 * newGrayValue - currentPixel.getGreen();
        int blueValue = 2 * newGrayValue - currentPixel.getBlue();

        if (redValue > 255) {
          redValue = 255;
        }

        if (greenValue > 255) {
          greenValue = 255;
        }

        if (blueValue > 255) {
          blueValue = 255;
        }

        currentPixel.setRed(redValue);
        currentPixel.setGreen(greenValue);
        currentPixel.setBlue(blueValue);
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Pixels and Images (b)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a ThresholdImage object, and draw the image in the scene
     * with no filter. Then call the threshold() method, and draw the image in the
     * scene with the filter.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`ThresholdImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image with a threshold filter
 */
public class ThresholdImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public ThresholdImage(String filename) {
    super(filename);
  }

  /*
   * Applies a threshold filter to an image
   */
  public void threshold(int value) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Calculate the average of the red, green, and blue value of each Pixel. If the
     * average is less than the parameter value, set the color of the Pixel to BLACK.
     * Otherwise, set the color of the Pixel to WHITE.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`}],validationFiles:[{path:`ThresholdImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ThresholdImage.java Test")
public class ThresholdImageTest {

  String messageGap = "\\n       ";
  ThresholdImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new ThresholdImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("threshold() sets the color of each Pixel to BLACK if the grayscale value is less than value, otherwise WHITE => ")
  public void testThresholdSetsColor() {
    String message = "Find the average of the red, green, and blue values. If the average is less than value,";
    message += "\\n        use setColor() to set the color of the Pixel to BLACK. Otherwise, set it to WHITE." + messageGap;

    getExpectedResult(20);
    testObject.threshold(20);
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int expectedGreen = originalPixels[row][col].getGreen();
        int expectedBlue = originalPixels[row][col].getBlue();
        
        int actualRed = actual[row][col].getRed();
        int actualGreen = actual[row][col].getGreen();
        int actualBlue = actual[row][col].getBlue();
        
        assertEquals(expectedRed, actualRed, message);
        assertEquals(expectedGreen, actualGreen, message);
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult(int value) {
    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        Pixel currentPixel = originalPixels[row][col];
        int grayValue = currentPixel.getRed() + currentPixel.getGreen() + currentPixel.getBlue();
        grayValue /= 3;

        if (grayValue < value) {
          currentPixel.setColor(Color.BLACK);
        }
        else {
          currentPixel.setColor(Color.WHITE);
        }
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Pixels and Images (c)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a ColorImage object, and draw the image in the scene with no
     * filter. Then call the colorize() method, and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ColorImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a colorized image
 */
public class ColorImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public ColorImage(String filename) {
    super(filename);
  }

  /*
   * Applies a colorize filter by converting each Pixel to grayscale and applying
   * a color to it based on its grayscale value
   */
  public void colorize() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Calculate the average of the red, green, and blue color values of each Pixel,
     * and set the red, green, and blue values of the Pixel based on the average.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`}],validationFiles:[{path:`ColorImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ColorImage.java Test")
public class ColorImageTest {

  String messageGap = "\\n       ";
  ColorImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new ColorImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("colorize() sets the red value of each Pixel object to the expected value => ")
  public void testColorizeModifiesRed() {
    String message = "Set the red value of each Pixel object to the expected value." + messageGap;

    getExpectedResult();
    testObject.colorize();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("colorize() sets the green value of each Pixel object to the expected value => ")
  public void testColorizeModifiesGreen() {
    String message = "Set the green value of each Pixel object to the expected value." + messageGap;

    getExpectedResult();
    testObject.colorize();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("colorize() sets the blue value of each Pixel object to the expected value => ")
  public void testColorizeModifiesBlue() {
    String message = "Set the blue value of each Pixel object to the expected value." + messageGap;

    getExpectedResult();
    testObject.colorize();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult() {
    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        Pixel currentPixel = originalPixels[row][col];
        int grayValue = currentPixel.getRed() + currentPixel.getGreen() + currentPixel.getBlue();
        grayValue /= 3;

        if (grayValue < 85) {
          currentPixel.setRed(255);
          currentPixel.setGreen(0);
          currentPixel.setBlue(0);
        }
        else if (grayValue < 170) {
          currentPixel.setRed(0);
          currentPixel.setGreen(255);
          currentPixel.setBlue(0);
        }
        else {
          currentPixel.setRed(0);
          currentPixel.setGreen(0);
          currentPixel.setBlue(255);
        }
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Manipulating Pixels and Images (d)`,lesson:`Lesson 8: Modifying Images`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Scene object
    Scene images = new Scene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a SharpImage object, and draw the image in the scene with no
     * filter. Then call the sharpen() method and draw the image in the scene again.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(images);
    
  }
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] pixels;    // The 2D array of pixels

  /*
   * Sets the superclass filename to the specified filename
   * and calls the getPixelsFromImage() method to initialize
   * the 2D array of Pixel objects that make up the image
   */
  public ImagePlus(String filename) {
    super(filename);   // Calls the Image class constructor

    // Initialize the pixels array by getting the pixels from the image
    pixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of pixels
   */
  public Pixel[][] getImagePixels() {
    return pixels;
  }

  /*
   * Returns the pixels in the image as a 2D array of Pixel objects
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] tempPixels = new Pixel[getHeight()][getWidth()];
    
    for (int row = 0; row < tempPixels.length; row++) {
      for (int col = 0; col < tempPixels[0].length; col++) {
        tempPixels[row][col] = getPixel(col, row);
      }
    }

    return tempPixels;
  }

}`},{path:`SharpImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a sharpened image
 */
public class SharpImage extends ImagePlus {

  /*
   * Calls the superclass constructor to initialize pixels
   */
  public SharpImage(String filename) {
    super(filename);
  }

  /*
   * Sharpens the image by calculating the difference between the color values of the current
   * and neighboring Pixel objects and adjust the color values to emphasize the edges
   */
  public void sharpen() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the pixels from the image, and traverse the 2D array of Pixel objects.
     * Calculate the difference between the current pixel and its top-left
     * neighbor in red, green, and blue values. Then find the average difference by
     * dividing these differences by 3. Adjust the color values of the current pixel
     * by adding the average difference to the current color values. Make sure the
     * new colors are less than or equal to 255! Then set the color values for the
     * current pixel to the new color values.
     * -----------------------------------------------------------------------------
     */



    
  }
  
}`}],validationFiles:[{path:`SharpImageTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SharpImage.java Test")
public class SharpImageTest {

  String messageGap = "\\n       ";
  SharpImage testObject;
  ImagePlus originalObject;
  Pixel[][] originalPixels;
   
  @BeforeEach
  public void setup() {
    String randomImage = getRandomImage();
    testObject = new SharpImage(randomImage);
    originalObject = new ImagePlus(randomImage);
    originalPixels = originalObject.getImagePixels();
  }
   
  @Test
  @Order(1)
  @DisplayName("sharpen() sets the red value of each Pixel object to the expected result => ")
  public void testSharpenModifiesRed() {
    String message = "Calculate the difference between the current Pixel and its top-left neighbor in red, green,";
    message += "\\n        and blue color values. Then calculate the average difference by adding the three color";
    message += "\\n        differences and dividing by 3. Adjust the color values of the current Pixel by adding the";
    message += "\\n        average difference to the current color values and updating the Pixel object." + messageGap;

    getExpectedResult();
    testObject.sharpen();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedRed = originalPixels[row][col].getRed();
        int actualRed = actual[row][col].getRed();
        assertEquals(expectedRed, actualRed, message);
      }
    }
  }

  @Test
  @Order(2)
  @DisplayName("sharpen() sets the green value of each Pixel object to the expected result => ")
  public void testSharpenModifiesGreen() {
    String message = "Calculate the difference between the current Pixel and its top-left neighbor in red, green,";
    message += "\\n        and blue color values. Then calculate the average difference by adding the three color";
    message += "\\n        differences and dividing by 3. Adjust the color values of the current Pixel by adding the";
    message += "\\n        average difference to the current color values and updating the Pixel object." + messageGap;

    getExpectedResult();
    testObject.sharpen();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedGreen = originalPixels[row][col].getGreen();
        int actualGreen = actual[row][col].getGreen();
        assertEquals(expectedGreen, actualGreen, message);
      }
    }
  }

  @Test
  @Order(3)
  @DisplayName("sharpen() sets the blue value of each Pixel object to the expected result => ")
  public void testSharpenModifiesBlue() {
    String message = "Calculate the difference between the current Pixel and its top-left neighbor in red, green,";
    message += "\\n        and blue color values. Then calculate the average difference by adding the three color";
    message += "\\n        differences and dividing by 3. Adjust the color values of the current Pixel by adding the";
    message += "\\n        average difference to the current color values and updating the Pixel object." + messageGap;

    getExpectedResult();
    testObject.sharpen();
    Pixel[][] actual = testObject.getImagePixels();

    for (int row = 0; row < actual.length; row++) {
      for (int col = 0; col < actual[0].length; col++) {
        int expectedBlue = originalPixels[row][col].getBlue();
        int actualBlue = actual[row][col].getBlue();
        assertEquals(expectedBlue, actualBlue, message);
      }
    }
  }

  private String getRandomImage() {
    String[] images = {"astronaut.jpg", "baseball.jpg", "basketball.jpg", "cellphone.jpg", "dog.jpg", "donut.jpg", "electricguitars.jpg",
                     "fashion.jpg", "football.jpg", "games.jpg", "guitar.jpg", "laptop.jpg", "motivation.jpg", "nature.jpg",
                     "paintbrushes.jpg", "soccer.jpg", "speakers.jpg", "starwars.jpg", "track.jpg"};
    return images[(int)(Math.random() * images.length)];
  }

  private void getExpectedResult() {
    for (int row = 1; row < originalPixels.length - 1; row++) {
      for (int col = 1; col < originalPixels[0].length - 1; col++) {
        Pixel currentPixel = originalPixels[row][col];
        
        int redDiff = currentPixel.getRed() - originalPixels[row - 1][col - 1].getRed();
        int greenDiff = currentPixel.getGreen() - originalPixels[row - 1][col - 1].getGreen();
        int blueDiff = currentPixel.getBlue() - originalPixels[row - 1][col - 1].getBlue();
        int averageDiff = (redDiff + greenDiff + blueDiff) / 3;

        int newRed = currentPixel.getRed() + averageDiff;
        int newGreen = currentPixel.getGreen() + averageDiff;
        int newBlue = currentPixel.getBlue() + averageDiff;

        if (newRed > 255) {
          newRed = 255;
        }

        if (newGreen > 255) {
          newGreen = 255;
        }

        if (newBlue > 255) {
          newBlue = 255;
        }

        currentPixel.setRed(newRed);
        currentPixel.setGreen(newGreen);
        currentPixel.setBlue(newBlue);
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Investigate and Modify: Image Manipulations #1`,lesson:`Lesson 9: Impacts of Programs`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    // Creates an ImageScene object
    ImageScene scene = new ImageScene();

    // Creates ChromaImage objects
    ChromaImage camera = new ChromaImage("camera.jpg");
    ChromaImage donuts = new ChromaImage("donuts.jpg");

    // Draws the ChromaImage object in the scene
    scene.drawChromaImage(camera, "donuts.jpg");

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





    
    
    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`ChromaImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image with a solid color background
 */
public class ChromaImage extends ImagePlus {

  /*
   * Calls the ImagePlus constructor to create the Image
   * object and initialize the 2D imagePixels array
   */
  public ChromaImage(String imageFile) {
    super(imageFile);
  }

  /*
   * Replaces the green pixels in photo
   */
  public void replaceGreenBackground(String newBackgroundFile) {
    ImagePlus newBackground = new ImagePlus(newBackgroundFile);
    Pixel[][] newPixels = newBackground.getImagePixels();
    Pixel[][] originalPixels = getImagePixels();

    Pixel fromPixel = null;
    Pixel toPixel = null;

    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        toPixel = originalPixels[row][col];

        if (toPixel.getGreen() > toPixel.getRed()) {
          fromPixel = newPixels[row][col];
          toPixel.setColor(fromPixel.getColor());
        }
      }
    }
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] imagePixels;     // The 2D array of Pixel objects that make up the image

  /*
   * Initializes photo from a specified image file and calls
   * the getImagePixels() method to initialize imagePixels
   */
  public ImagePlus(String imageFile) {
    super(imageFile);
    imagePixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of Pixel objects that make up the image
   */
  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  /*
   * Returns a 2D array of Pixel objects that make up the Image object
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] imagePixels = new Pixel[getHeight()][getWidth()];

    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        imagePixels[row][col] = getPixel(col, row);
      }
    }

    return imagePixels;
  }

  /*
   * Detects the edges of objects in an image
   */
  public void detectEdges(int edgeDistance) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length - 1; col++) {
        Pixel leftPixel = imagePixels[row][col];
        Pixel rightPixel = imagePixels[row][col + 1];

        if (getDistance(leftPixel.getColor(), rightPixel.getColor()) > edgeDistance) {
          leftPixel.setColor(Color.BLACK);
        }
        else {
          leftPixel.setColor(Color.WHITE);
        }
      }
    }
  }

  /*
   * Different approach to detecting edges in an image
   */
  public void anotherEdgeDetection(int edgeDistance) {
    for (int row = 0; row < imagePixels.length - 1; row++) {
      for (int col = 0; col < imagePixels[0].length - 1; col++) {
        Pixel currentPixel = imagePixels[row][col];
        Pixel bottomPixel = imagePixels[row + 1][col];
        Pixel rightPixel = imagePixels[row][col + 1];

        int currentIntensity = getAverage(currentPixel);
        int bottomIntensity = getAverage(bottomPixel);
        int rightIntensity = getAverage(rightPixel);

        int bottomDiff = Math.abs(currentIntensity - bottomIntensity);
        int rightDiff = Math.abs(currentIntensity - rightIntensity);

        if (bottomDiff > edgeDistance || rightDiff > edgeDistance) {
          currentPixel.setColor(Color.BLACK);
        }
        else {
          currentPixel.setColor(Color.WHITE);
        }
      }
    }
  }

  /*
   * Returns the distance between firstColor and secondColor
   */
  public double getDistance(Color firstColor, Color secondColor) {
    double redDistance = firstColor.getRed() - secondColor.getRed();
    double greenDistance = firstColor.getGreen() - secondColor.getGreen();
    double blueDistance = firstColor.getBlue() - secondColor.getBlue();
    double distance = Math.sqrt(redDistance * redDistance + greenDistance * greenDistance + blueDistance * blueDistance);
    return distance;
  }

  /*
   * Returns the average of the red, green, and blue color values
   * of the specified Pixel object
   */
  public int getAverage(Pixel thePixel) {
    return (thePixel.getRed() + thePixel.getGreen() + thePixel.getBlue()) / 3;
  }
  
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a scene with images
 */
public class ImageScene extends Scene {

  /*
   * Draws the specified ChromaImage object in the scene
   */
  public void drawChromaImage(ChromaImage photo, String newBackgroundImage) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.replaceGreenBackground(newBackgroundImage);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }

  /*
   * Draws the specified ImagePlus object in the scene
   */
  public void drawImageWithEdges(ImagePlus photo) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.detectEdges(2);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }

  /*
   * Draws the specified ImagePlus object in the scene
   */
  public void drawImageWithAltEdges(ImagePlus photo) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.anotherEdgeDetection(2);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Image Manipulations #2`,lesson:`Lesson 9: Impacts of Programs`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ChromaImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image with a solid color background
 */
public class ChromaImage extends ImagePlus {

  /*
   * Calls the ImagePlus constructor to create the Image
   * object and initialize the 2D imagePixels array
   */
  public ChromaImage(String imageFile) {
    super(imageFile);
  }

  /*
   * Replaces the green pixels in photo
   */
  public void replaceGreenBackground(String newBackgroundFile) {
    ImagePlus newBackground = new ImagePlus(newBackgroundFile);
    Pixel[][] newPixels = newBackground.getImagePixels();
    Pixel[][] originalPixels = getImagePixels();

    Pixel fromPixel = null;
    Pixel toPixel = null;

    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        toPixel = originalPixels[row][col];

        if (toPixel.getGreen() > toPixel.getRed()) {
          fromPixel = newPixels[row][col];
          toPixel.setColor(fromPixel.getColor());
        }
      }
    }
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] imagePixels;     // The 2D array of Pixel objects that make up the image

  /*
   * Initializes photo from a specified image file and calls
   * the getImagePixels() method to initialize imagePixels
   */
  public ImagePlus(String imageFile) {
    super(imageFile);
    imagePixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of Pixel objects that make up the image
   */
  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  /*
   * Returns a 2D array of Pixel objects that make up the Image object
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] imagePixels = new Pixel[getHeight()][getWidth()];

    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        imagePixels[row][col] = getPixel(col, row);
      }
    }

    return imagePixels;
  }

  /*
   * Detects the edges of objects in an image
   */
  public void detectEdges(int edgeDistance) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length - 1; col++) {
        Pixel leftPixel = imagePixels[row][col];
        Pixel rightPixel = imagePixels[row][col + 1];

        if (getDistance(leftPixel.getColor(), rightPixel.getColor()) > edgeDistance) {
          leftPixel.setColor(Color.BLACK);
        }
        else {
          leftPixel.setColor(Color.WHITE);
        }
      }
    }
  }

  /*
   * Different approach to detecting edges in an image
   */
  public void anotherEdgeDetection(int edgeDistance) {
    for (int row = 0; row < imagePixels.length - 1; row++) {
      for (int col = 0; col < imagePixels[0].length - 1; col++) {
        Pixel currentPixel = imagePixels[row][col];
        Pixel bottomPixel = imagePixels[row + 1][col];
        Pixel rightPixel = imagePixels[row][col + 1];

        int currentIntensity = getAverage(currentPixel);
        int bottomIntensity = getAverage(bottomPixel);
        int rightIntensity = getAverage(rightPixel);

        int bottomDiff = Math.abs(currentIntensity - bottomIntensity);
        int rightDiff = Math.abs(currentIntensity - rightIntensity);

        if (bottomDiff > edgeDistance || rightDiff > edgeDistance) {
          currentPixel.setColor(Color.BLACK);
        }
        else {
          currentPixel.setColor(Color.WHITE);
        }
      }
    }
  }

  /*
   * Returns the distance between firstColor and secondColor
   */
  public double getDistance(Color firstColor, Color secondColor) {
    double redDistance = firstColor.getRed() - secondColor.getRed();
    double greenDistance = firstColor.getGreen() - secondColor.getGreen();
    double blueDistance = firstColor.getBlue() - secondColor.getBlue();
    double distance = Math.sqrt(redDistance * redDistance + greenDistance * greenDistance + blueDistance * blueDistance);
    return distance;
  }

  /*
   * Returns the average of the red, green, and blue color values
   * of the specified Pixel object
   */
  public int getAverage(Pixel thePixel) {
    return (thePixel.getRed() + thePixel.getGreen() + thePixel.getBlue()) / 3;
  }
  
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a scene with images
 */
public class ImageScene extends Scene {

  /*
   * Draws the specified ChromaImage object in the scene
   */
  public void drawChromaImage(ChromaImage photo, String newBackgroundImage) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.replaceGreenBackground(newBackgroundImage);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }

  /*
   * Draws the specified ImagePlus object in the scene
   */
  public void drawImageWithEdges(ImagePlus photo) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.detectEdges(2);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }

  /*
   * Draws the specified ImagePlus object in the scene
   */
  public void drawImageWithAltEdges(ImagePlus photo) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.anotherEdgeDetection(2);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Image Manipulations #3`,lesson:`Lesson 9: Impacts of Programs`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ChromaImage.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image with a solid color background
 */
public class ChromaImage extends ImagePlus {

  /*
   * Calls the ImagePlus constructor to create the Image
   * object and initialize the 2D imagePixels array
   */
  public ChromaImage(String imageFile) {
    super(imageFile);
  }

  /*
   * Replaces the green pixels in photo
   */
  public void replaceGreenBackground(String newBackgroundFile) {
    ImagePlus newBackground = new ImagePlus(newBackgroundFile);
    Pixel[][] newPixels = newBackground.getImagePixels();
    Pixel[][] originalPixels = getImagePixels();

    Pixel fromPixel = null;
    Pixel toPixel = null;

    for (int row = 0; row < originalPixels.length; row++) {
      for (int col = 0; col < originalPixels[0].length; col++) {
        toPixel = originalPixels[row][col];

        if (toPixel.getGreen() > toPixel.getRed()) {
          fromPixel = newPixels[row][col];
          toPixel.setColor(fromPixel.getColor());
        }
      }
    }
  }
  
}`},{path:`ImagePlus.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an image that can be modified with filters and effects
 */
public class ImagePlus extends Image {

  private Pixel[][] imagePixels;     // The 2D array of Pixel objects that make up the image

  /*
   * Initializes photo from a specified image file and calls
   * the getImagePixels() method to initialize imagePixels
   */
  public ImagePlus(String imageFile) {
    super(imageFile);
    imagePixels = getPixelsFromImage();
  }

  /*
   * Returns the 2D array of Pixel objects that make up the image
   */
  public Pixel[][] getImagePixels() {
    return imagePixels;
  }

  /*
   * Returns a 2D array of Pixel objects that make up the Image object
   */
  public Pixel[][] getPixelsFromImage() {
    Pixel[][] imagePixels = new Pixel[getHeight()][getWidth()];

    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length; col++) {
        imagePixels[row][col] = getPixel(col, row);
      }
    }

    return imagePixels;
  }

  /*
   * Detects the edges of objects in an image
   */
  public void detectEdges(int edgeDistance) {
    for (int row = 0; row < imagePixels.length; row++) {
      for (int col = 0; col < imagePixels[0].length - 1; col++) {
        Pixel leftPixel = imagePixels[row][col];
        Pixel rightPixel = imagePixels[row][col + 1];

        if (getDistance(leftPixel.getColor(), rightPixel.getColor()) > edgeDistance) {
          leftPixel.setColor(Color.BLACK);
        }
        else {
          leftPixel.setColor(Color.WHITE);
        }
      }
    }
  }

  /*
   * Different approach to detecting edges in an image
   */
  public void anotherEdgeDetection(int edgeDistance) {
    for (int row = 0; row < imagePixels.length - 1; row++) {
      for (int col = 0; col < imagePixels[0].length - 1; col++) {
        Pixel currentPixel = imagePixels[row][col];
        Pixel bottomPixel = imagePixels[row + 1][col];
        Pixel rightPixel = imagePixels[row][col + 1];

        int currentIntensity = getAverage(currentPixel);
        int bottomIntensity = getAverage(bottomPixel);
        int rightIntensity = getAverage(rightPixel);

        int bottomDiff = Math.abs(currentIntensity - bottomIntensity);
        int rightDiff = Math.abs(currentIntensity - rightIntensity);

        if (bottomDiff > edgeDistance || rightDiff > edgeDistance) {
          currentPixel.setColor(Color.BLACK);
        }
        else {
          currentPixel.setColor(Color.WHITE);
        }
      }
    }
  }

  /*
   * Returns the distance between firstColor and secondColor
   */
  public double getDistance(Color firstColor, Color secondColor) {
    double redDistance = firstColor.getRed() - secondColor.getRed();
    double greenDistance = firstColor.getGreen() - secondColor.getGreen();
    double blueDistance = firstColor.getBlue() - secondColor.getBlue();
    double distance = Math.sqrt(redDistance * redDistance + greenDistance * greenDistance + blueDistance * blueDistance);
    return distance;
  }

  /*
   * Returns the average of the red, green, and blue color values
   * of the specified Pixel object
   */
  public int getAverage(Pixel thePixel) {
    return (thePixel.getRed() + thePixel.getGreen() + thePixel.getBlue()) / 3;
  }
  
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a scene with images
 */
public class ImageScene extends Scene {

  /*
   * Draws the specified ChromaImage object in the scene
   */
  public void drawChromaImage(ChromaImage photo, String newBackgroundImage) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.replaceGreenBackground(newBackgroundImage);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }

  /*
   * Draws the specified ImagePlus object in the scene
   */
  public void drawImageWithEdges(ImagePlus photo) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.detectEdges(2);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }

  /*
   * Draws the specified ImagePlus object in the scene
   */
  public void drawImageWithAltEdges(ImagePlus photo) {
    drawImage(photo, 0, 0, getWidth());
    pause(2);

    photo.anotherEdgeDetection(2);
    drawImage(photo, 0, 0, getWidth());
    pause(2);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Personal Narrative Project`,lesson:`Lesson 10a: Personal Narrative Project`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {




    

    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Personal Narrative Project`,lesson:`Lesson 10b: Personal Narrative Project [1-Day Version]`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {




    

    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`CSA 2023 Console Sandbox_2025`,lesson:`Sandbox: Console`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`CSA 2023 Theater Sandbox_2025`,lesson:`Sandbox: The Theater`,view:`theater`,grid:`1,0 1,10 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {




  }
}`}],validationFiles:[],dataFiles:[]}];export{e as LEVELS};