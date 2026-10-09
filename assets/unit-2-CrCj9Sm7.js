var e=[{name:`Predict and Run: Storing Multiple Values`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArrayManager.java`,text:`/*
 * Manages a one-dimensional (1D) array of integers
 */
public class ArrayManager {

  private int[] arrayValues;   // The 1D array of integers

  /*
   * Initializes arrayValues to contain a specified number of integers
   */
  public ArrayManager(int numValues) {
    arrayValues = new int[numValues];
  }

  /*
   * Initializes arrayValues to reference an existing 1D array of integers
   */
  public ArrayManager(int[] arrayValues) {
    this.arrayValues = arrayValues;
  }

  /*
   * Returns the number of integers stored in the 1D array arrayValues
   */
  public int getNumValues() {
    return arrayValues.length;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: 1D Arrays #1`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArrayManager.java`,text:`/*
 * Manages a one-dimensional (1D) array of integers
 */
public class ArrayManager {

  private int[] arrayValues;   // The 1D array of integers

  /*
   * Initializes arrayValues to contain a specified number of integers
   */
  public ArrayManager(int numValues) {
    arrayValues = new int[numValues];
  }

  /*
   * Initializes arrayValues to reference an existing 1D array of integers
   */
  public ArrayManager(int[] arrayValues) {
    this.arrayValues = arrayValues;
  }

  /*
   * Returns the number of integers stored in the 1D array arrayValues
   */
  public int getNumValues() {
    return arrayValues.length;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: 1D Arrays #2`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArrayManager.java`,text:`/*
 * Manages a one-dimensional (1D) array of integers
 */
public class ArrayManager {

  private int[] arrayValues;   // The 1D array of integers

  /*
   * Initializes arrayValues to contain a specified number of integers
   */
  public ArrayManager(int numValues) {
    arrayValues = new int[numValues];
  }

  /*
   * Initializes arrayValues to reference an existing 1D array of integers
   */
  public ArrayManager(int[] arrayValues) {
    this.arrayValues = arrayValues;
  }

  /*
   * Returns the number of integers stored in the 1D array arrayValues
   */
  public int getNumValues() {
    return arrayValues.length;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: 1D Arrays #3`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArrayManager.java`,text:`/*
 * Manages a one-dimensional (1D) array of integers
 */
public class ArrayManager {

  private int[] arrayValues;   // The 1D array of integers

  /*
   * Initializes arrayValues to contain a specified number of integers
   */
  public ArrayManager(int numValues) {
    arrayValues = new int[numValues];
  }

  /*
   * Initializes arrayValues to reference an existing 1D array of integers
   */
  public ArrayManager(int[] arrayValues) {
    this.arrayValues = arrayValues;
  }

  /*
   * Returns the number of integers stored in the 1D array arrayValues
   */
  public int getNumValues() {
    return arrayValues.length;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Getting the Length of a 1D Array (a)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    SocialMedia twitter = new SocialMedia("Twitter", 23);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the SocialMedia object.
     * -----------------------------------------------------------------------------
     */
    
    
    
    
  }
}`},{path:`SocialMedia.java`,text:`/*
 * Manages data about followers on a social media platform
 */
public class SocialMedia {

  private String name;        // The name of the social media platform
  private int[] followers;    // The number of followers multiple users have

  /*
   * Sets name to the specified name and initializes
   * followers to store the specified number of integers
   */
  public SocialMedia(String name, int numUsers) {
    this.name = name;
    followers = new int[numUsers];
  }

  /*
   * Returns a String containing the number of users in the 1D array followers
   */
  public String toString() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a String containing the text "Number of Users: " followed
     * by the length of the 1D array followers.
     * -----------------------------------------------------------------------------
     */
    
    return "";
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

  SocialMedia testSocialMedia;
  int randomUsers;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    randomUsers = (int)((Math.random() * 49) + 1);
    testSocialMedia = new SocialMedia("Some User", randomUsers);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return the String \\"Number of Users: \\" followed by the length of the 1D array followers => ")
  public void testFollowersLength() {
    message = "Use nameOfArray.length to get the length of a 1D array." + messageGap;

    String expected = "Number of Users: " + randomUsers;
    String actual = testSocialMedia.toString();
    
    assertEquals(expected, actual, message);
  }
 
}`}],dataFiles:[]},{name:`Practice: Getting the Length of a 1D Array (b)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    PetStore neighborhoodStore = new PetStore("Neighborhood Pet Store", 16);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the PetStore object.
     * -----------------------------------------------------------------------------
     */

    
    
    
  }
}`},{path:`PetStore.java`,text:`/*
 * Manages data about pet breeds available for adoption
 */
public class PetStore {

  private String name;      // The name of a pet store
  private String[] breeds;  // The breeds of pets available

  /*
   * Sets name to the specified name and initializes
   * breeds to store the number of specified values
   */
  public PetStore(String name, int numBreeds) {
    this.name = name;
    breeds = new String[numBreeds];
  }

  /*
   * Returns a String containing the number of breeds in the 1D array breeds
   */
  public String toString() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a String containing the text "Number of Breeds: " followed
     * by the length of the 1D array breeds.
     * -----------------------------------------------------------------------------
     */
    
    return "";
  }
  
}`}],validationFiles:[{path:`PetStoreTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PetStore.java Test")
public class PetStoreTest {

  PetStore testPetStore;
  int randomBreeds;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    randomBreeds = (int)((Math.random() * 49) + 1);
    testPetStore = new PetStore("Some Pet Store", randomBreeds);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return the String \\"Number of Breeds: \\" followed by the length of the 1D array breeds => ")
  public void testBreedsLength() {
    message = "Use nameOfArray.length to get the length of a 1D array." + messageGap;

    String expected = "Number of Breeds: " + randomBreeds;
    String actual = testPetStore.toString();
    
    assertEquals(expected, actual, message);
  }
 
}`}],dataFiles:[]},{name:`Practice: Getting the Length of a 1D Array (c)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    SolarSystem theSolarSystem = new SolarSystem("Our Solar System", 9);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the SolarSystem object.
     * -----------------------------------------------------------------------------
     */

    
    
    
  }
}`},{path:`SolarSystem.java`,text:`/*
 * Manages data about planets in a solar system
 */
public class SolarSystem {

  private String name;        // The name of a solar system
  private String[] planets;   // The names of planets in a solar system

  /*
   * Sets name to the specified name and initializes
   * planets to store the specified number of values
   */
  public SolarSystem(String name, int numPlanets) {
    this.name = name;
    planets = new String[numPlanets];
  }

  /*
   * Returns a String containing the number of planets in the 1D array planets
   */
  public String toString() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a String containing the text "Number of Planets: " followed
     * by the length of the 1D array planets.
     * -----------------------------------------------------------------------------
     */
    
    return "";
  }
  
}`}],validationFiles:[{path:`SolarSystemTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SolarSystem.java Test")
public class SolarSystemTest {

  SolarSystem testSolarSystem;
  int randomPlanets;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    randomPlanets = (int)((Math.random() * 49) + 1);
    testSolarSystem = new SolarSystem("Some Solar System", randomPlanets);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return the String \\"Number of Planets: \\" followed by the length of the 1D array planets => ")
  public void testPlanetsLength() {
    message = "Use nameOfArray.length to get the length of a 1D array." + messageGap;

    String expected = "Number of Planets: " + randomPlanets;
    String actual = testSolarSystem.toString();
    
    assertEquals(expected, actual, message);
  }
 
}`}],dataFiles:[]},{name:`Practice: Getting the Length of a 1D Array (d)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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

    ArrayPainter bruce = new ArrayPainter(2, 4, "south", 20, 10);
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the ArrayPainter object.
     * -----------------------------------------------------------------------------
     */   

    
    
    
  }
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that has a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] paintColors;    // The 1D array of paint colors

  /*
   * Sets the x, y, direction, and paint to default values and
   * initializes paintColors to store two paint colors
   */
  public ArrayPainter() {
    paintColors = new String[2];
  }

  /*
   * Sets x, y, direction, and paint to the specified values and initializes
   * paintColors to store the specified number of paint colors
   */
  public ArrayPainter(int x, int y, String direction, int paint, int numPaintColors) {
    super(x, y, direction, paint);
    paintColors = new String[numPaintColors];
  }

  /*
   * Returns a String containing the number of paint colors in the 1D array paintColors
   */
  public String toString() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a String containing the text "Number of Paint Colors: " followed
     * by the length of the 1D array paintColors.
     * -----------------------------------------------------------------------------
     */
    
    return "";
  }
  
}`}],validationFiles:[{path:`ArrayPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ArrayPainter.java Test")
public class ArrayPainterTest {

  ArrayPainter testArrayPainter;
  int randomPaintColors;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    randomPaintColors = (int)((Math.random() * 49) + 1);
    testArrayPainter = new ArrayPainter(0, 0, "east", 0, randomPaintColors);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return the String \\"Number of Paint Colors: \\" followed by the length of the 1D array paintColors => ")
  public void testPaintColorsLength() {
    message = "Use nameOfArray.length to get the length of a 1D array." + messageGap;

    String expected = "Number of Paint Colors: " + randomPaintColors;
    String actual = testArrayPainter.toString();
    
    assertEquals(expected, actual, message);
  }
 
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (a)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store 5 double values and use it to
     * instantiate a Customer object. Then, print the Customer object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Customer.java`,text:`/*
 * Manages data about a customer's purchases
 */
public class Customer {

  private double[] purchases;   // The amount spent on purchases

  /*
   * Initializes purchases to the parameter purchases
   */
  public Customer(double[] purchases) {
    this.purchases = purchases;
  }

  /*
   * Returns a String containing the number of purchases in the 1D array purchases
   */
  public String toString() {
    return "Number of Purchases: " + purchases.length;
  }
  
}`}],validationFiles:[{path:`CustomerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Customer.java Test")
public class CustomerTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    output = SystemOutTestRunner.run();
    expectedOutput = "Number of Purchases: 5";

    int lineLocation = findOutputMatch(expectedOutput);

    if (lineLocation == -1) {
      fail("Missing or incorrect console output!" + messageGap);
    }
    else {
      actualOutput = output.get(lineLocation);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate a Customer object with a 1D array of 5 double values => ")
  public void testCustomerArrayLength() {
    message = "Declare and initialize a 1D array to store 5 double values." + messageGap;

    int colon = actualOutput.indexOf(":");
    int actualLength = Integer.parseInt(actualOutput.substring(colon + 2));

    assertEquals(5, actualLength, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the Customer object => ")
  public void testCustomerRunnerOutput() {     
    message = "Print the Customer object using System.out.print() or System.out.println()." + messageGap;
    
    assertEquals(expectedOutput, actualOutput, message);
  }

  /*
   * Helper to find matching line of output or first
   * line of output if no match found
   */
  private int findOutputMatch(String expected) {
    int result = -1;

    for (int index = 0; index < output.size(); index++) {
      String line = output.get(index);
      
      if (line.equals(expected)) {
        result = index;
      }
    }

    return result;
  }
  
}
`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (b)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store 10 int values and use it to
     * instantiate a MusicSurvey object. Then, print the MusicSurvey object.
     * -----------------------------------------------------------------------------
     */

    

    
    
  }
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about the ages of survey respondents
 */
public class MusicSurvey {

  private int[] ages;    // The ages of respondents to the survey

  /*
   * Initializes ages to the parameter ages
   */
  public MusicSurvey(int[] ages) {
    this.ages = ages;
  }

  /*
   * Returns a String containing the number of respondents in the 1D array ages
   */
  public String toString() {
    return "Number of Respondents: " + ages.length;
  }
  
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    output = SystemOutTestRunner.run();
    expectedOutput = "Number of Respondents: 10";

    int lineLocation = findOutputMatch(expectedOutput);

    if (lineLocation == -1) {
      fail("Missing or incorrect console output!" + messageGap);
    }
    else {
      actualOutput = output.get(lineLocation);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate a MusicSurvey object with a 1D array of 10 int values => ")
  public void testMusicSurveyArrayLength() {
    message = "Declare and initialize a 1D array to store 10 int values." + messageGap;

    int colon = actualOutput.indexOf(":");
    int actualLength = Integer.parseInt(actualOutput.substring(colon + 2));

    assertEquals(10, actualLength, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the MusicSurvey object => ")
  public void testSurveyRunnerOutput() {     
    message = "Print the MusicSurvey object using System.out.print() or System.out.println()." + messageGap;
    
    assertEquals(expectedOutput, actualOutput, message);
  }

  /*
   * Helper to find matching line of output or first
   * line of output if no match found
   */
  private int findOutputMatch(String expected) {
    int result = -1;

    for (int index = 0; index < output.size(); index++) {
      String line = output.get(index);
      
      if (line.equals(expected)) {
        result = index;
      }
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (c)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store 20 String values
     * and a 1D array to store 20 double values use them to instantiate a
     * Netflix object. Then, print the Netflix object.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Netflix.java`,text:`/*
 * Manages data about Netflix subscription fees
 * in different countries
 */
public class Netflix {

  private String[] countries;    // The names of countries
  private double[] fees;         // The subscription fee in each country

  /*
   * Initializes countries to the parameter countries
   * and fees to the parameter fees
   */
  public Netflix(String[] countries, double[] fees) {
    this.countries = countries;
    this.fees = fees;
  }

  /*
   * Returns a String containing the number of countries in the 1D array
   * countries and number of subscription fees in the 1D array fees
   */
  public String toString() {
    return "Number of Countries: " + countries.length + "\\nNumber of Fees: " + fees.length;
  }
  
}`}],validationFiles:[{path:`NetflixTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Netflix.java Test")
public class NetflixTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    expectedOutput = "Number of Countries: 20\\nNumber of Fees: 20";
    output = SystemOutTestRunner.run();
    actualOutput = getOutputLines(output);

    if (actualOutput.length() == 0) {
      fail("Missing output printed to the console!");
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate a Netflix object with a 1D array of 20 String values => ")
  public void testNetflixCountriesArrayLength() {
    message = "Declare and initialize a 1D array to store 20 String values." + messageGap;

    String firstLine = actualOutput.substring(0, actualOutput.indexOf("\\n"));
    int colon = firstLine.indexOf(":");
    int actualLength = Integer.parseInt(firstLine.substring(colon + 2));

    assertEquals(20, actualLength, message);
  }

  @Test
  @Order(2)
  @DisplayName("Instantiate a Netflix object with a 1D array of 20 double values => ")
  public void testNetflixFeesArrayLength() {
    message = "Declare and initialize a 1D array to store 20 double values." + messageGap;

    String secondLine = actualOutput.substring(actualOutput.indexOf("\\n") + 1);
    int colon = secondLine.indexOf(":");
    int actualLength = Integer.parseInt(secondLine.substring(colon + 2));

    assertEquals(20, actualLength, message);
  }

  @Test
  @Order(3)
  @DisplayName("Print the Netflix object => ")
  public void testNetflixRunnerOutput() {     
    message = "Print the Netflix object using System.out.print() or System.out.println()." + messageGap;
    
    assertEquals(expectedOutput, actualOutput, message);
  }

  /*
   * Helper to get first line of output if
   * outputLines is greater than or equal to 1
   */
  private String getOutputLines(List<String> outputLines) {
    String result = "";

    if (outputLines.size() >= 1) {
      result = outputLines.get(0);
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (d)`,lesson:`Lesson 2: One-Dimensional (1D) Arrays`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
     * ✅ Declare and initialize a 1D array to store 12 String values
     * and a 1D array to store 12 int values use them to instantiate an
     * ArrayPainter object. Then, print the ArrayPainter object.
     * -----------------------------------------------------------------------------
     */
    
    



    
    
  }
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that has a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] paintColors;    // The 1D array of paint colors
  private int[] paintUnits;        // The 1D array of paint units

  /*
   * Sets x, y, direction, and paint to the specified values 
   * and initializes paintColors to the parameter paintColors
   * and paintUnits to the parameter paintUnits
   */
  public ArrayPainter(int x, int y, String direction, int paint, String[] paintColors, int[] paintUnits) {
    super(x, y, direction, paint);
    this.paintColors = paintColors;
    this.paintUnits = paintUnits;
  }

  /*
   * Returns a String containing the number of paint colors in the 1D array 
   * paintColors and the number of paint units in the 1D array paintUnits
   */
  public String toString() {
    return "Number of Paint Colors: " + paintColors.length + "\\nNumber of Paint Units: " + paintUnits.length;
  }
  
}`}],validationFiles:[{path:`ArrayPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
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
public class ArrayPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail("The Painter might not be instantiated." + messageGap);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Instantiate an ArrayPainter object => ")
  public void testArrayPainterInstantiated() {
    String message = "Create an ArrayPainter object with any starting values for x, y, direction, and paint.\\n";
    message += "        Pass a 1D array of String values for paintColors and a 1D array of int values for paintUnits." + messageGap;
      
    assertTrue(primaryPainterLog != null, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the ArrayPainter object => ")
  public void testPrintArrayPainterObject() {
    String message = "The ArrayPainter class has a toString() method to return information about the object.\\n";
    message += "        Use a print statement to print the ArrayPainter object." + messageGap;

    List<String> output = SystemOutTestRunner.run();
    String expected = "Number of Paint Colors: 12\\nNumber of Paint Units: 12";

    assertTrue(output.contains(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Initializer Lists`,lesson:`Lesson 3: Modifying Elements`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
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
}`},{path:`DashPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that paints dashes
 */
public class DashPainter extends Painter {

  private String[] colors;    // The 1D array of paint colors a DashPainter can use to paint with

  /*
   * Sets the starting x, y location, direction, and amount of paint
   * and initializes colors to the 1D array of colors
   */
  public DashPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Returns the color at the specified index from the 1D array colors
   */
  public String getColor(int index) {
    return colors[index];
  }

  /*
   * Sets the 1D array colors to newColors
   */
  public void setColors(String[] newColors) {
    colors = newColors;
  }

  /*
   * Moves then paints using a color from colors, then moves
   * again if able to move in the direction it is facing
   */
  public void movePaintMove(int index) {
    move();
    paint(colors[index]);

    if (canMove()) {
      move();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Assigning and Accessing Elements #1`,lesson:`Lesson 3: Modifying Elements`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
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
}`},{path:`DashPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that paints dashes
 */
public class DashPainter extends Painter {

  private String[] colors;    // The 1D array of paint colors a DashPainter can use to paint with

  /*
   * Sets the starting x, y location, direction, and amount of paint
   * and initializes colors to the 1D array of colors
   */
  public DashPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Returns the color at the specified index from the 1D array colors
   */
  public String getColor(int index) {
    return colors[index];
  }

  /*
   * Sets the 1D array colors to newColors
   */
  public void setColors(String[] newColors) {
    colors = newColors;
  }

  /*
   * Moves then paints using a color from colors, then moves
   * again if able to move in the direction it is facing
   */
  public void movePaintMove(int index) {
    move();
    paint(colors[index]);

    if (canMove()) {
      move();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Assigning and Accessing Elements #2`,lesson:`Lesson 3: Modifying Elements`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
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
}`},{path:`DashPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that paints dashes
 */
public class DashPainter extends Painter {

  private String[] colors;    // The 1D array of paint colors a DashPainter can use to paint with

  /*
   * Sets the starting x, y location, direction, and amount of paint
   * and initializes colors to the 1D array of colors
   */
  public DashPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Returns the color at the specified index from the 1D array colors
   */
  public String getColor(int index) {
    return colors[index];
  }

  /*
   * Sets the 1D array colors to newColors
   */
  public void setColors(String[] newColors) {
    colors = newColors;
  }

  /*
   * Moves then paints using a color from colors, then moves
   * again if able to move in the direction it is facing
   */
  public void movePaintMove(int index) {
    move();
    paint(colors[index]);

    if (canMove()) {
      move();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Assigning and Accessing Elements #3`,lesson:`Lesson 3: Modifying Elements`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0
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
}`},{path:`DashPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that paints dashes
 */
public class DashPainter extends Painter {

  private String[] colors;    // The 1D array of paint colors a DashPainter can use to paint with

  /*
   * Sets the starting x, y location, direction, and amount of paint
   * and initializes colors to the 1D array of colors
   */
  public DashPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Returns the color at the specified index from the 1D array colors
   */
  public String getColor(int index) {
    return colors[index];
  }

  /*
   * Sets the 1D array colors to newColors
   */
  public void setColors(String[] newColors) {
    colors = newColors;
  }

  /*
   * Moves then paints using a color from colors, then moves
   * again if able to move in the direction it is facing
   */
  public void movePaintMove(int index) {
    move();
    paint(colors[index]);

    if (canMove()) {
      move();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Creating 1D Arrays (a) #1`,lesson:`Lesson 3: Modifying Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store the specified values and use
     * it to instantiate a Cake object. Then, print the Cake object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Cake.java`,text:`/*
 * Represents a cake that can be sold at a food truck
 */
public class Cake {

  private String[] flavors;    // The 1D array of available flavors for a cake

  /*
   * Initializes flavors to the specified 1D array of flavors
   */
  public Cake(String[] flavors) {
    this.flavors = flavors;
  }

  /*
   * Returns a String containing each available flavor
   */
  public String toString() {
    String result = "Number of Available Flavors: " + flavors.length + "\\n";

    int index = 0;

    while (index < flavors.length) {
      result += "\\nFlavor #" + (index + 1) + ": " + flavors[index];
      index++;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`CakeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cake.java Test")
public class CakeTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    output = SystemOutTestRunner.run();
    expectedOutput = getExpectedOutput();

    int lineLocation = findOutputMatch("Number of Available Flavors:");
    
    if (lineLocation == -1) {
      fail("Missing or incorrect console output!" + messageGap);
    }
    else {
      actualOutput = output.get(lineLocation);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Cake flavors contains the specified values => ")
  public void testCakeFlavorsArray() {
    message = "Use an initializer list to declare and initialize the 1D array with the specified values.";
    message += "\\n        Then pass the 1D array you create as the argument to the Cake constructor and print the Cake object.";
    message += messageGap;

    assertEquals(expectedOutput, actualOutput, message);
  }

  private String getExpectedOutput() {
    String[] testFlavors = {"chocolate", "red velvet", "pumpkin spice", "angel food", "carrot", "funfetti"};
    
    String result = "Number of Available Flavors: " + testFlavors.length + "\\n";

    int index = 0;

    while (index < testFlavors.length) {
      result += "\\nFlavor #" + (index + 1) + ": " + testFlavors[index];
      index++;
    }

    return result;
  }

  /*
   * Helper to find matching line of output or first
   * line of output if no match found
   */
  private int findOutputMatch(String expected) {
    int result = -1;

    for (int index = 0; index < output.size(); index++) {
      String line = output.get(index);
      
      if (line.indexOf(expected) >= 0) {
        result = index;
      }
    }

    return result;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (b) #1`,lesson:`Lesson 3: Modifying Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store the specified values and use
     * it to instantiate a Billboard object. Then, print the Billboard object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Billboard.java`,text:`/*
 * Manages the top songs of all time
 */
public class Billboard {

  private String[] songTitles;     // The 1D array of the titles of top songs

  /*
   * Initializes songTitles to the 1D array of song titles
   */
  public Billboard(String[] songTitles) {
    this.songTitles = songTitles;
  }

  /*
   * Returns a String containing each song title
   */
  public String toString() {
    String result = "Number of Songs: " + songTitles.length + "\\n";

    int index = 0;

    while (index < songTitles.length) {
      result += "\\nSong #" + (index + 1) + ": " + songTitles[index];
      index++;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`BillboardTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Billboard.java Test")
public class BillboardTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    output = SystemOutTestRunner.run();
    expectedOutput = getExpectedOutput();

    int lineLocation = findOutputMatch("Number of Songs:");
    
    if (lineLocation == -1) {
      fail("Missing or incorrect console output!" + messageGap);
    }
    else {
      actualOutput = output.get(lineLocation);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("Billboard songTitles contains the specified values => ")
  public void testBillboardSongTitlesArray() {
    message = "Use an initializer list to declare and initialize the 1D array with the specified values.";
    message += "\\n        Then pass the 1D array you create as the argument to the Billboard constructor and print the Billboard object.";
    message += messageGap;

    assertEquals(expectedOutput, actualOutput, message);
  }

  private String getExpectedOutput() {
    String[] testSongs = {"Blinding Lights", "The Twist", "Uptown Funk!", "I Gotta Feeling", "Eye of the Tiger", "Somebody That I Used to Know"};
    
    String result = "Number of Songs: " + testSongs.length + "\\n";

    int index = 0;

    while (index < testSongs.length) {
      result += "\\nSong #" + (index + 1) + ": " + testSongs[index];
      index++;
    }

    return result;
  }

  /*
   * Helper to find matching line of output or first
   * line of output if no match found
   */
  private int findOutputMatch(String expected) {
    int result = -1;

    for (int index = 0; index < output.size(); index++) {
      String line = output.get(index);
      
      if (line.indexOf(expected) >= 0) {
        result = index;
      }
    }

    return result;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (c) #1`,lesson:`Lesson 3: Modifying Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store the specified values and use
     * it to instantiate a PetStore object. Then, print the PetStore object.
     * -----------------------------------------------------------------------------
     */
    


    
        
  }
}`},{path:`PetStore.java`,text:`/*
 * Manages the types of pets availale at a pet store
 */
public class PetStore {

  private String[] petTypes;      // The 1D array with the types of pets

  /*
   * Initializes petTypes to the specified 1D array of pet types
   */
  public PetStore(String[] petTypes) {
    this.petTypes = petTypes;
  }

  /*
   * Returns a String containing each type of pet
   */
  public String toString() {
    String result = "Number of Types of Pets: " + petTypes.length + "\\n";

    int index = 0;

    while (index < petTypes.length) {
      result += "\\nPet Type #" + (index + 1) + ": " + petTypes[index];
      index++;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`PetStoreTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PetStore.java Test")
public class PetStoreTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    output = SystemOutTestRunner.run();
    expectedOutput = getExpectedOutput();

    int lineLocation = findOutputMatch("Number of Types of Pets:");
    
    if (lineLocation == -1) {
      fail("Missing or incorrect console output!" + messageGap);
    }
    else {
      actualOutput = output.get(lineLocation);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("PetStore petTypes contains the specified values => ")
  public void testPetStorePetTypesArray() {
    message = "Use an initializer list to declare and initialize the 1D array with the specified values.";
    message += "\\n        Then pass the 1D array you create as the argument to the PetStore constructor and print the PetStore object.";
    message += messageGap;

    assertEquals(expectedOutput, actualOutput, message);
  }

  private String getExpectedOutput() {
    String[] testPetTypes = {"cat", "dog", "bird", "rabbit", "fish", "guinea pig"};
    
    String result = "Number of Types of Pets: " + testPetTypes.length + "\\n";

    int index = 0;

    while (index < testPetTypes.length) {
      result += "\\nPet Type #" + (index + 1) + ": " + testPetTypes[index];
      index++;
    }

    return result;
  }

  /*
   * Helper to find matching line of output or first
   * line of output if no match found
   */
  private int findOutputMatch(String expected) {
    int result = -1;

    for (int index = 0; index < output.size(); index++) {
      String line = output.get(index);
      
      if (line.indexOf(expected) >= 0) {
        result = index;
      }
    }

    return result;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (d) #1`,lesson:`Lesson 3: Modifying Elements`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D array to store the specified values and use
     * it to instantiate an ArrayPainter object. Then, print the ArrayPainter object.
     * -----------------------------------------------------------------------------
     */






    
    
  }
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter with a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] colors;      // The 1D array of paint colors

  /*
   * Sets the starting x and y location, direction, and amount of paint
   * and initializes colors to the specified 1D array of paint colors
   */
  public ArrayPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Returns a String containing each paint color
   */
  public String toString() {
    String result = "Number of Paint Colors: " + colors.length + "\\n";

    int index = 0;

    while (index < colors.length) {
      result += "\\nPaint Color #" + (index + 1) + ": " + colors[index];
      index++;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`ArrayPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ArrayPainter.java Test")
public class ArrayPainterTest {

  List<String> output;
  String expectedOutput;
  String actualOutput;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    output = SystemOutTestRunner.run();
    expectedOutput = getExpectedOutput();

    int lineLocation = findOutputMatch("Number of Paint Colors:");
    
    if (lineLocation == -1) {
      fail("Missing or incorrect console output!" + messageGap);
    }
    else {
      actualOutput = output.get(lineLocation);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("ArrayPainter colors contains the specified values => ")
  public void testArrayPaintColorsArray() {
    message = "Use an initializer list to declare and initialize the 1D array with the specified values.";
    message += "\\n        Then pass the 1D array you create as the argument to the ArrayPainter constructor and print the ArrayPainter object.";
    message += messageGap;

    assertEquals(expectedOutput, actualOutput, message);
  }

  private String getExpectedOutput() {
    String[] testPaintColors = {"crimson", "khaki", "lavender", "forestgreen", "powderblue", "plum"};
    
    String result = "Number of Paint Colors: " + testPaintColors.length + "\\n";

    int index = 0;

    while (index < testPaintColors.length) {
      result += "\\nPaint Color #" + (index + 1) + ": " + testPaintColors[index];
      index++;
    }

    return result;
  }

  /*
   * Helper to find matching line of output or first
   * line of output if no match found
   */
  private int findOutputMatch(String expected) {
    int result = -1;

    for (int index = 0; index < output.size(); index++) {
      String line = output.get(index);
      
      if (line.indexOf(expected) >= 0) {
        result = index;
      }
    }

    return result;
  }
   
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (a) #2`,lesson:`Lesson 3: Modifying Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the updateGrade() method to change one or more values in the
     * 1D array weeklyScores, then print the updates made to the Student object.
     * -----------------------------------------------------------------------------
     */

    int[] weekScores = {86, 77, 98, 81, 97};
    Student tessa = new Student("Tessa", weekScores);
    
    System.out.println(tessa);



    
    
  }
}`},{path:`Student.java`,text:`/*
 * Represents a student in a teacher's class
 */
public class Student {

  private String name;          // The name of a student
  private int[] quizGrades;     // The 1D array of daily quiz scores a student received

  /*
   * Sets name to the specified name and initializes quizGrades
   * to the specified 1D array of daily quiz scores
   */
  public Student(String name, int[] quizGrades) {
    this.name = name;
    this.quizGrades = quizGrades;
  }

  /*
   * Returns the name of the student
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the 1D array of daily quiz scores
   */
  public int[] getQuizGrades() {
    return quizGrades;
  }

  /*
   * Adds 5 to the score at the specified position in quizGrades
   */
  public void updateGrade(int position) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Add 5 to the value in quizGrades located at the parameter position.
     * -----------------------------------------------------------------------------
     */
    
    
  }

  /*
   * Returns a String containing the student's name and scores
   */
  public String toString() {
    String result = "Name: " + name + "\\nScores: ";

    int index = 0;

    while (index < quizGrades.length) {
      result += quizGrades[index] + " ";
      index += 1;
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

  int[] testScores;
  Student testStudent;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testScores = getRandomTestValues(5, 75, 100);
    testStudent = new Student("Some Student", testScores);
  }
   
  @Test
  @Order(1)
  @DisplayName("Add 5 to the value located at the parameter position in the 1D array quizGrades => ")
  public void testUpdateGrades() {
    message = "Update the value at the parameter position to the result of the current value plus 5." + messageGap;

    int expectedValue = testScores[2] + 5;
    testStudent.updateGrade(2);
    int[] updatedGrades = testStudent.getQuizGrades();
    int actualValue = updatedGrades[2];

    assertEquals(expectedValue, actualValue, message);
  }

  /*
   * Helper to generate an 1D int array with random values within a range
   */
  private int[] getRandomTestValues(int numValues, int minValue, int maxValue) {
    int[] tempValues = new int[numValues];

    int index = 0;

    while (index < numValues) {
      tempValues[index] = (int)((Math.random() * (maxValue - minValue)) + minValue);
      index++;
    }

    return tempValues;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (b) #2`,lesson:`Lesson 3: Modifying Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the updateProfits() method to change one or more values in the 1D
     * array truckProfits, then print the updates made to the FoodTruck object.
     * -----------------------------------------------------------------------------
     */

    double[] truckProfits = {115.74, 197.37, 136.80, 174.45, 113.63};
    FoodTruck tacoTruck = new FoodTruck("Tycho's Tacos", truckProfits);
    
    System.out.println(tacoTruck);



    
    
  }
}`},{path:`FoodTruck.java`,text:`/*
 * Represents a food truck
 */
public class FoodTruck {

  private String name;              // The name of a food truck
  private double[] dailyProfits;    // The 1D array of profits made each day

  /*
   * Sets name to the specified name and initializes dailyProfits
   * to the specified 1D array of daily profits
   */
  public FoodTruck(String name, double[] dailyProfits) {
    this.name = name;
    this.dailyProfits = dailyProfits;
  }

  /*
   * Returns the name of the food truck
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the 1D array of daily profits
   */
  public double[] getDailyProfits() {
    return dailyProfits;
  }

  /*
   * Subtracts 100 from the value at the specified position in dailyProfits
   */
  public void updateProfits(int position) {
    /* --------------------------------------- TO DO ---------------------------------------
     * ✅ Subtracts 100 from the value in dailyProfits located at the parameter position.
     * -------------------------------------------------------------------------------------
     */

       
  }

  /*
   * Returns a String containing the food truck's name and daily profits
   */
  public String toString() {
    String result = "Name: " + name + "\\nDaily Profits: ";

    int index = 0;

    while (index < dailyProfits.length) {
      result += dailyProfits[index] + " ";
      index += 1;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`FoodTruckTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("FoodTruck.java Test")
public class FoodTruckTest {

  double[] testProfits;
  FoodTruck testFoodTruck;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testProfits = getRandomTestValues(5, 100, 200);
    testFoodTruck = new FoodTruck("Some Food Truck", testProfits);
  }
   
  @Test
  @Order(1)
  @DisplayName("Subtract 100 from the value located at the parameter position in the 1D array dailyProfits => ")
  public void testUpdateDailyProfits() {
    message = "Update the value at the parameter position to the result of the current value minus 100." + messageGap;

    double expectedValue = testProfits[2] - 100;
    testFoodTruck.updateProfits(2);
    double[] updatedProfits = testFoodTruck.getDailyProfits();
    double actualValue = updatedProfits[2];

    assertEquals(expectedValue, actualValue, message);
  }

  /*
   * Helper to generate an 1D double array with random values within a range
   */
  private double[] getRandomTestValues(int numValues, double minValue, double maxValue) {
    double[] tempValues = new double[numValues];

    int index = 0;

    while (index < numValues) {
      tempValues[index] = (Math.random() * (maxValue - minValue)) + minValue;
      index++;
    }

    return tempValues;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (c) #2`,lesson:`Lesson 3: Modifying Elements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getBeatsPerSecond() method and print the result.
     * -----------------------------------------------------------------------------
     */

    String[] genres = {"Latin", "Rock", "Video game music", "Jazz", "R&B"};
    int[] bpm = {156, 119, 132, 84, 107};
    MusicSurvey results = new MusicSurvey(genres, bpm);
    
    System.out.println(results);


    

    
  }
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages the data from a music survey
 */
public class MusicSurvey {

  private String[] favGenres;      // The 1D array of respondents' favorite genres
  private int[] beatsPerMinute;    // The 1D array of the BPM of each genre

  /*
   * Initializes favGenres to the specified 1D array of favorite genres and initializes
   * beatsPerMinute to the specified 1D array of the BPM of each genre
   */
  public MusicSurvey(String[] favGenres, int[] beatsPerMinute) {
    this.favGenres = favGenres;
    this.beatsPerMinute = beatsPerMinute;
  }

  /*
   * Returns the 1D array of favorite genres
   */
  public String[] getFavGenres() {
    return favGenres;
  }

  /*
   * Returns the 1D array of the BPM of each genre
   */
  public int[] getBeatsPerMinute() {
    return beatsPerMinute;
  }

  /*
   * Returns the beats per second for the value at the parameter position
   * from the 1D array beatsPerMinute
   */
  public int getBeatsPerSecond(int position) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Divide the value at parameter position in the 1D array beatsPerMinute by
     * 60 to get the beats per second and return the result.
     * -----------------------------------------------------------------------------
     */
    
    return -1;
  }

  /*
   * Returns a String containing each favorite genre and its beats per minute
   */
  public String toString() {
    String result = "";
    int index = 0;

    while (index < favGenres.length) {
      result += favGenres[index] + " BPM: " + beatsPerMinute[index] + "\\n";
      index += 1;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  String[] testGenres;
  int[] testValues;
  MusicSurvey testMusicSurvey;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testGenres = new String[]{"Latin", "Rock", "Video game music", "Jazz", "R&B"};
    testValues = getRandomTestValues(5, 100, 200);
    testMusicSurvey = new MusicSurvey(testGenres, testValues);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return the result of the value located at the parameter position in the 1D array beatsPerMinute divided by 60 => ")
  public void testGetBeatsPerSecond() {
    message = "Divide the value at the parameter position by 60 and return the result." + messageGap;

    int expectedValue = testValues[2] / 60;
    int actualValue = testMusicSurvey.getBeatsPerSecond(2);

    assertEquals(expectedValue, actualValue, message);
  }

  /*
   * Helper to generate an 1D int array with random values within a range
   */
  private int[] getRandomTestValues(int numValues, int minValue, int maxValue) {
    int[] tempValues = new int[numValues];

    int index = 0;

    while (index < numValues) {
      tempValues[index] = (int)((Math.random() * (maxValue - minValue)) + minValue);
      index++;
    }

    return tempValues;
  }
  
}`}],dataFiles:[]},{name:`Practice: Creating 1D Arrays (d) #2`,lesson:`Lesson 3: Modifying Elements`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the updatePaintAmount() method to change one or more values in the
     * 1D array paintUnits, then print the updates made to the ArrayPainter object.
     * -----------------------------------------------------------------------------
     */

    String[] paintColors = {"Khaki", "Magenta", "LightCoral", "Beige", "Silver"};
    int[] paintAmounts = {5, 10, 8, 7, 6};
    ArrayPainter chase = new ArrayPainter(4, 3, "east", 20, paintColors, paintAmounts);

    System.out.println(chase);





    
    
  }
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter with a 1D array of paint colors and units
 */
public class ArrayPainter extends Painter {

  private String[] colors;     // The 1D array of paint colors
  private int[] paintUnits;    // The 1D array of the amount of paint for each color

  /*
   * Sets the starting x and y location, direction, and amount of paint and initializes colors to
   * the specified 1D array of paint colors and paintUnits to the specified 1D array of paint units
   */
  public ArrayPainter(int x, int y, String direction, int numPaint, String[] colors, int[] paintUnits) {
    super(x, y, direction, numPaint);
    this.colors = colors;
    this.paintUnits = paintUnits;
  }

  /*
   * Returns the 1D array of paint colors
   */
  public String[] getColors() {
    return colors;
  }

  /*
   * Returns the 1D array of paint units
   */
  public int[] getPaintUnits() {
    return paintUnits;
  }

  /*
   * Multiplies the value at the specified position in paintUnits by 2
   */
  public void updatePaintAmount(int position) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Multiply the value in paintUnits located at the parameter position by 2.
     * -----------------------------------------------------------------------------
     */

    
  }

  /*
   * Returns a String containing each paint color
   */
  public String toString() {
    String result = "";
    int index = 0;

    while (index < colors.length) {
      result += colors[index] + ": " + paintUnits[index] + "\\n";
      index += 1;
    }

    return result;
  }
}`}],validationFiles:[{path:`ArrayPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ArrayPainter.java Test")
public class ArrayPainterTest {

  String[] testPaintColors;
  int[] testPaintUnits;
  ArrayPainter testArrayPainter;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPaintColors = new String[]{"Khaki", "Magenta", "LightCoral", "Beige", "Silver"};
    testPaintUnits = getRandomTestValues(5, 5, 10);
    testArrayPainter = new ArrayPainter(0, 0, "east", 10, testPaintColors, testPaintUnits);
  }
   
  @Test
  @Order(1)
  @DisplayName("Multiply the value located at the parameter position in the 1D array paintUnits by 2 => ")
  public void testUpdatePaintAmount() {
    message = "Update the value at the parameter position to the result of the current value times 2." + messageGap;

    int expectedValue = testPaintUnits[2] * 2;
    testArrayPainter.updatePaintAmount(2);
    int[] updatedPaintUnits = testArrayPainter.getPaintUnits();
    int actualValue = updatedPaintUnits[2];

    assertEquals(expectedValue, actualValue, message);
  }

  /*
   * Helper to generate an 1D int array with random values within a range
   */
  private int[] getRandomTestValues(int numValues, int minValue, int maxValue) {
    int[] tempValues = new int[numValues];

    int index = 0;

    while (index < numValues) {
      tempValues[index] = (int)((Math.random() * (maxValue - minValue)) + minValue);
      index++;
    }

    return tempValues;
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Traversing 1D Arrays`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that has a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] colors;      // The 1D array of paint colors

  /*
   * Sets the x and y location, direction, and amount of paint to the specified
   * values and initializes colors to the specified 1D array of paint colors
   */
  public ArrayPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Turns the ArrayPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the ArrayPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the ArrayPainter has paint
   */
  public void paintIfHasPaint(int index) {
    if (hasPaint()) {
      paint(colors[index]);
    }
  }

  /*
   * Moves and paints one space if the ArrayPainter can move and has paint
   */
  public void moveAndPaint(int index) {
    moveIfCanMove();
    paintIfHasPaint(index);
  }

  /*
   * Paints with each color in the 1D array colors
   */
  public void paintColors() {
    int index = 0;

    while (index < colors.length) {
      moveAndPaint(index);
      index += 2;
    }
  }

  /*
   * Returns a String containing the paint colors the ArrayPainter has
   */
  public String toString() {
    String result = "ArrayPainter Colors: ";
    int index = 0;

    while (index < colors.length) {
      result += colors[index] + " ";
      index += 1;
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Traversing 1D Arrays #1`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that has a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] colors;      // The 1D array of paint colors

  /*
   * Sets the x and y location, direction, and amount of paint to the specified
   * values and initializes colors to the specified 1D array of paint colors
   */
  public ArrayPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Turns the ArrayPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the ArrayPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the ArrayPainter has paint
   */
  public void paintIfHasPaint(int index) {
    if (hasPaint()) {
      paint(colors[index]);
    }
  }

  /*
   * Moves and paints one space if the ArrayPainter can move and has paint
   */
  public void moveAndPaint(int index) {
    moveIfCanMove();
    paintIfHasPaint(index);
  }

  /*
   * Paints with each color in the 1D array colors
   */
  public void paintColors() {
    int index = 0;

    while (index < colors.length) {
      moveAndPaint(index);
      index += 2;
    }
  }

  /*
   * Returns a String containing the paint colors the ArrayPainter has
   */
  public String toString() {
    String result = "ArrayPainter Colors: ";
    int index = 0;

    while (index < colors.length) {
      result += colors[index] + " ";
      index += 1;
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Traversing 1D Arrays #2`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that has a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] colors;      // The 1D array of paint colors

  /*
   * Sets the x and y location, direction, and amount of paint to the specified
   * values and initializes colors to the specified 1D array of paint colors
   */
  public ArrayPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Turns the ArrayPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the ArrayPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the ArrayPainter has paint
   */
  public void paintIfHasPaint(int index) {
    if (hasPaint()) {
      paint(colors[index]);
    }
  }

  /*
   * Moves and paints one space if the ArrayPainter can move and has paint
   */
  public void moveAndPaint(int index) {
    moveIfCanMove();
    paintIfHasPaint(index);
  }

  /*
   * Paints with each color in the 1D array colors
   */
  public void paintColors() {
    int index = 0;

    while (index < colors.length) {
      moveAndPaint(index);
      index += 2;
    }
  }

  /*
   * Returns a String containing the paint colors the ArrayPainter has
   */
  public String toString() {
    String result = "ArrayPainter Colors: ";
    int index = 0;

    while (index < colors.length) {
      result += colors[index] + " ";
      index += 1;
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Traversing 1D Arrays #3`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`ArrayPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that has a 1D array of paint colors
 */
public class ArrayPainter extends Painter {

  private String[] colors;      // The 1D array of paint colors

  /*
   * Sets the x and y location, direction, and amount of paint to the specified
   * values and initializes colors to the specified 1D array of paint colors
   */
  public ArrayPainter(int x, int y, String direction, int numPaint, String[] colors) {
    super(x, y, direction, numPaint);
    this.colors = colors;
  }

  /*
   * Turns the ArrayPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the ArrayPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the ArrayPainter has paint
   */
  public void paintIfHasPaint(int index) {
    if (hasPaint()) {
      paint(colors[index]);
    }
  }

  /*
   * Moves and paints one space if the ArrayPainter can move and has paint
   */
  public void moveAndPaint(int index) {
    moveIfCanMove();
    paintIfHasPaint(index);
  }

  /*
   * Paints with each color in the 1D array colors
   */
  public void paintColors() {
    int index = 0;

    while (index < colors.length) {
      moveAndPaint(index);
      index += 2;
    }
  }

  /*
   * Returns a String containing the paint colors the ArrayPainter has
   */
  public String toString() {
    String result = "ArrayPainter Colors: ";
    int index = 0;

    while (index < colors.length) {
      result += colors[index] + " ";
      index += 1;
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Traverse a 1D Array (a)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] usRetailerNames = {"Wal-Mart Stores, Inc", "Costco Wholesale Corporation", "The Kroger Co.", "Amazon.com, Inc",
                          "The Home Depot", "Walgreens Boots Alliance, Inc.", "CVS Health Corporation", "Target Corporation",
                          "Lowe's Companies, Inc.", "Albertsons Companies, Inc.", "Best Buy Co., Inc."};
    
    Retailers usRetailers = new Retailers(usRetailerNames);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getAllRetailers() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`Retailers.java`,text:`/*
 * Manages data about top US retailers
 */
public class Retailers {

  private String[] storeNames;     // The 1D array of the names of top US retailers

  /*
   * Initializes storeNames to the specified 1D array of top US retailers
   */
  public Retailers(String[] storeNames) {
    this.storeNames = storeNames;
  }

  /*
   * Returns a String containing the names in the 1D array storeNames
   */
  public String getAllRetailers() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array storeNames and concatenate each value to result.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`RetailersTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Retailers.java Test")
public class RetailersTest {

  String[] testStoreNames;
  Retailers testRetailers;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testStoreNames = new String[]{"Wal-Mart Stores, Inc", "Costco Wholesale Corporation", "The Kroger Co.", "Amazon.com, Inc",
                                  "The Home Depot", "Walgreens Boots Alliance, Inc.", "CVS Health Corporation", "Target Corporation",
                                  "Lowe's Companies, Inc.", "Albertsons Companies, Inc.", "Best Buy Co., Inc."};

    testRetailers = new Retailers(testStoreNames);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each retailer on separate lines => ")
  public void testGetAllRetailers() {
    message = "Traverse the 1D array storeNames using a while loop and concatenate each value to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testStoreNames);
    String actual = testRetailers.getAllRetailers();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] testValues) {
    String result = "";

    for (int index = 0; index < testValues.length; index++) {
      result += testValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse a 1D Array (b)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] sets = {"Arc de Triomphe", "Architecture Studio", "Berlin", "Big Ben", "Buckingham Palace",
                     "Burj Khalifa", "Chicago", "Dubai", "Empire State Building", "Fallingwater", "Farnsworth House",
                     "Flatiron Building", "Great Pyramid of Giza", "Imperial Hotel", "John Hancock Center"};

    Lego legoSets = new Lego(sets);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getAllSets() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`Lego.java`,text:`/*
 * Manages data about Lego sets
 */
public class Lego {

  private String[] setNames;      // The 1D array of Lego set names

  /*
   * Initializes setNames to the specified 1D array of Lego set names
   */
  public Lego(String[] setNames) {
    this.setNames = setNames;
  }

  /*
   * Returns a String containing the names in the 1D array setNames
   */
  public String getAllSets() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array setNames and concatenate each value to result.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`LegoTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Lego.java Test")
public class LegoTest {

  String[] testSetNames;
  Lego testLego;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testSetNames = new String[]{"Arc de Triomphe", "Architecture Studio", "Berlin", "Big Ben", "Buckingham Palace",
                                "Burj Khalifa", "Chicago", "Dubai", "Empire State Building", "Fallingwater", "Farnsworth House",
                                "Flatiron Building", "Great Pyramid of Giza", "Imperial Hotel", "John Hancock Center"};

    testLego = new Lego(testSetNames);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each Lego set on separate lines => ")
  public void testGetAllSets() {
    message = "Traverse the 1D array setNames using a while loop and concatenate each value to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testSetNames);
    String actual = testLego.getAllSets();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] testValues) {
    String result = "";

    for (int index = 0; index < testValues.length; index++) {
      result += testValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse a 1D Array (c)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] libraries = {"Walker", "Whitney M. Young, Jr.", "Scottsdale", "West Lawn", "Rogers Park",
                          "Popular Library at Water Works", "Albany Park", "Avalon", "Brainerd", "Oriole Park"};

    Library chicagoLibraries = new Library(libraries);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getAllLibraries() method and print the result.
     * -----------------------------------------------------------------------------
     */
  

    
    
    
    
  }
}`},{path:`Library.java`,text:`/*
 * Manages data about public libraries in a city
 */
public class Library {

  private String[] libraryNames;     // The 1D array of libraries in a city

  /*
   * Initializes libraryNames to the specified 1D array of library names
   */
  public Library(String[] libraryNames) {
    this.libraryNames = libraryNames;
  }

  /*
   * Returns a String containing the names in the 1D array libraryNames
   */
  public String getAllLibraries() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array libraryNames and concatenate each value to result.
     * -----------------------------------------------------------------------------
     */

    

    return result;
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

  String[] testLibraryNames;
  Library testLibrary;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testLibraryNames = new String[]{"Walker", "Whitney M. Young, Jr.", "Scottsdale", "West Lawn", "Rogers Park",
                                    "Popular Library at Water Works", "Albany Park", "Avalon", "Brainerd", "Oriole Park"};

    testLibrary = new Library(testLibraryNames);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each library on separate lines => ")
  public void testGetAllLibraries() {
    message = "Traverse the 1D array libraryNames using a while loop and concatenate each value to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testLibraryNames);
    String actual = testLibrary.getAllLibraries();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] testValues) {
    String result = "";

    for (int index = 0; index < testValues.length; index++) {
      result += testValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse a 1D Array (d)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] ships = {"Black Pearl", "Queen Anne's Revenge", "The Flying Dutchman", "Red Lady", "The Sea Devil",
                      "The Bloodhound", "The Vicious Viper", "The Raging Rhino", "The Savage Shark", "The Fearsome Falcon"};

    PirateShip pirates = new PirateShip(ships);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getAllShips() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
    
  }
}`},{path:`PirateShip.java`,text:`/*
 * Manages data about pirate ships
 */
public class PirateShip {

  private String[] shipNames;       // The 1D array of pirate ship names

  /*
   * Initializes shipNames to the specified 1D array of pirate ship names
   */
  public PirateShip(String[] shipNames) {
    this.shipNames = shipNames;
  }

  /*
   * Returns a String containing the names in the 1D array shipNames
   */
  public String getAllShips() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array shipNames and concatenate each value to result.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`PirateShipTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PirateShip.java Test")
public class PirateShipTest {

  String[] testPirateShipNames;
  PirateShip testPirateShip;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPirateShipNames = new String[]{"Black Pearl", "Queen Anne's Revenge", "The Flying Dutchman", "Red Lady", "The Sea Devil",
                                       "The Bloodhound", "The Vicious Viper", "The Raging Rhino", "The Savage Shark", "The Fearsome Falcon"};

    testPirateShip = new PirateShip(testPirateShipNames);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each pirate ship on separate lines => ")
  public void testGetAllShips() {
    message = "Traverse the 1D array shipNames using a while loop and concatenate each value to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testPirateShipNames);
    String actual = testPirateShip.getAllShips();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] testValues) {
    String result = "";

    for (int index = 0; index < testValues.length; index++) {
      result += testValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse Multiple 1D Arrays (a)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] gameConsoles = {"Playstation 2 (PS2)", "Nintendo DS (DS)", "Nintendo Switch (NS)", "Game Boy (GB)", "Playstation 4 (PS4)",
                             "Playstation (PS)", "Nintendo Wii (Wii)", "Playstation 3 (PS3)", "Xbox 360 (X360)", "Game Boy Advance (GBA)"};

    double[] sales = {53.65, 57.92, 42.95, 43.18, 38.20, 40.78, 45.51, 29.92, 47.09, 40.39};

    VideoGame naConsoles = new VideoGame(gameConsoles, sales);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getConsolesInfo() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`VideoGame.java`,text:`/*
 * Manages data about gaming consoles
 */
public class VideoGame {

  private String[] consoles;        // The 1D array of the names of gaming consoles
  private double[] americaSales;    // The 1D array of the total sales for each gaming console

  /*
   * Initializes consoles to the specified 1D array of gaming console names and
   * americaSales to the specified 1D array of total sales for each gaming console
   */
  public VideoGame(String[] consoles, double[] americaSales) {
    this.consoles = consoles;
    this.americaSales = americaSales;
  }

  /*
   * Returns a String containing each gaming console's name and its total sales
   */
  public String getConsolesInfo() {
    String result = "";

    /* ----------------------------------------- TO DO -----------------------------------------
     * ✅ Traverse the 1D arrays consoles and americaSales and concatenate each value to result.
     * -----------------------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`VideoGameTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("VideoGame.java Test")
public class VideoGameTest {

  String[] testConsoles;
  double[] testSales;
  VideoGame testVideoGame;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testConsoles = new String[]{"Playstation 2 (PS2)", "Nintendo DS (DS)", "Nintendo Switch (NS)", "Game Boy (GB)", "Playstation 4 (PS4)",
                    "Playstation (PS)", "Nintendo Wii (Wii)", "Playstation 3 (PS3)", "Xbox 360 (X360)", "Game Boy Advance (GBA)"};

    testSales = new double[]{53.65, 57.92, 42.95, 43.18, 38.20, 40.78, 45.51, 29.92, 47.09, 40.39};

    testVideoGame = new VideoGame(testConsoles, testSales);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each gaming console and its total sales on separate lines => ")
  public void testGetConsolesInfo() {
    message = "Traverse the 1D array countries using a while loop and concatenate each value from countries and americaSales to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testConsoles, testSales);
    String actual = testVideoGame.getConsolesInfo();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, double[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += firstTestValues[index] + ": " + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse Multiple 1D Arrays (b)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] names = {"Carl Larsen Park", "Junipero Serra Playground", "Rolph Nicol Playground", "Alamo Square",
                      "Jose Coronado Playground", "Little Hollywood Park", "Lake Merced Park", "Golden Gate Park"};

    double[] scores = {0.795, 0.957, 0.864, 0.857, 0.859, 0.846, 0.73, 0.588};

    Park sanFrancisco = new Park(names, scores);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getParksInfo() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Park.java`,text:`/*
 * Manages data about parks in a city
 */
public class Park {

  private String[] parkNames;      // The 1D array of park names
  private double[] conditions;     // The 1D array of the condition scores for each park

  /*
   * Initializes parkNames to the specified 1D array of park names and
   * conditions to the 1D array of condition scores for each park
   */
  public Park(String[] parkNames, double[] conditions) {
    this.parkNames = parkNames;
    this.conditions = conditions;
  }

  /*
   * Returns a String containing each park's name and its condition score
   */
  public String getParksInfo() {
    String result = "";

    /* ----------------------------------------- TO DO -----------------------------------------
     * ✅ Traverse the 1D arrays parkNames and conditions and concatenate each value to result.
     * -----------------------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`ParkTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Park.java Test")
public class ParkTest {

  String[] testParks;
  double[] testScores;
  Park testPark;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testParks = new String[]{"Carl Larsen Park", "Junipero Serra Playground", "Rolph Nicol Playground", "Alamo Square",
                             "Jose Coronado Playground", "Little Hollywood Park", "Lake Merced Park", "Golden Gate Park"};

    testScores = new double[]{0.795, 0.957, 0.864, 0.857, 0.859, 0.846, 0.73, 0.588};

    testPark = new Park(testParks, testScores);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing each park's name and its condition score on separate lines => ")
  public void testGetParksInfo() {
    message = "Traverse the 1D array parkNames using a while loop and concatenate each value from parkNames and conditions to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testParks, testScores);
    String actual = testPark.getParksInfo();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, double[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += firstTestValues[index] + ": " + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse Multiple 1D Arrays (c)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] countries = {"Finland", "Ireland", "Norway", "France", "Netherlands", "Japan", "Sweden",
                          "Canada", "United Kingdom", "Portugal", "Switzerland", "Austria", "United States"};
    
    double[] food = {83.7, 81.7, 80.5, 80.2, 80.1, 79.5, 79.1, 79.1, 78.8, 78.7, 78.2, 78.1, 78.0};

    FoodSecurity countryData = new FoodSecurity(countries, food);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getCountriesInfo() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
    
  }
}`},{path:`FoodSecurity.java`,text:`/*
 * Manages data about food security in different countries
 */
public class FoodSecurity {

  private String[] countries;        // The 1D array of country names
  private double[] overallScores;    // The 1D array of overall food security index scores for each country

  /*
   * Initializes countries to the specified 1D array of country names and overallScores
   * to the specified 1D array of overall food security index scores
   */
  public FoodSecurity(String[] countries, double[] overallScores) {
    this.countries = countries;
    this.overallScores = overallScores;
  }

  /*
   * Returns a String containing each country's name and its food security index
   */
  public String getCountriesInfo() {
    String result = "";

    /* ----------------------------------------- TO DO -----------------------------------------
     * ✅ Traverse the 1D arrays countries and overallScores and concatenate each value to result.
     * -----------------------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`FoodSecurityTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("FoodSecurity.java Test")
public class FoodSecurityTest {

  String[] testCountries;
  double[] testIndexes;
  FoodSecurity testFoodSecurity;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testCountries = new String[]{"Finland", "Ireland", "Norway", "France", "Netherlands", "Japan", "Sweden",
                                 "Canada", "United Kingdom", "Portugal", "Switzerland", "Austria", "United States"};

    testIndexes = new double[]{83.7, 81.7, 80.5, 80.2, 80.1, 79.5, 79.1, 79.1, 78.8, 78.7, 78.2, 78.1, 78.0};

    testFoodSecurity = new FoodSecurity(testCountries, testIndexes);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each country and its food security index on separate lines => ")
  public void testGetCountriesInfo() {
    message = "Traverse the 1D array countries using a while loop and concatenate each value from countries and overallScores to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testCountries, testIndexes);
    String actual = testFoodSecurity.getCountriesInfo();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, double[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += firstTestValues[index] + ": " + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traverse Multiple 1D Arrays (d)`,lesson:`Lesson 4: Traversing 1D Arrays`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] players = {"LeBron James", "Michael Jordan", "Kareem Abdul-Jabbar", "Kobe Bryant", "Shaquille O'Neal",
                        "Tim Duncan", "Karl Malone", "Kevin Durant", "Jerry West", "Tony Parker"};

    int[] points = {7631, 5987, 5762, 5640, 5250, 5172, 4761, 4559, 4457, 4045};

    Basketball topTen = new Basketball(players, points);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getPlayersInfo() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
    
  }
}`},{path:`Basketball.java`,text:`/*
 * Manages data about basketball players
 */
public class Basketball {

  private String[] playerNames;    // The 1D array of basketball player names
  private int[] playerPoints;      // The 1D array of the total points scored by each player

  /*
   * Initializes playerNames to the specified 1D array of player names and
   * playerPoints to the 1D array of total points scored by each player
   */
  public Basketball(String[] playerNames, int[] playerPoints) {
    this.playerNames = playerNames;
    this.playerPoints = playerPoints;
  }

  /*
   * Returns a String containing each player's name and their total points scored
   */
  public String getPlayersInfo() {
    String result = "";

    /* ----------------------------------------- TO DO -----------------------------------------
     * ✅ Traverse the 1D arrays playerNames and playerPoints and concatenate each value to result.
     * -----------------------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`BasketballTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Basketball.java Test")
public class BasketballTest {

  String[] testNames;
  int[] testPoints;
  Basketball testBasketball;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testNames = new String[]{"LeBron James", "Michael Jordan", "Kareem Abdul-Jabbar", "Kobe Bryant", "Shaquille O'Neal",
                             "Tim Duncan", "Karl Malone", "Kevin Durant", "Jerry West", "Tony Parker"};

    testPoints = new int[]{7631, 5987, 5762, 5640, 5250, 5172, 4761, 4559, 4457, 4045};

    testBasketball = new Basketball(testNames, testPoints);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each player and their total points scored on separate lines => ")
  public void testGetPlayersInfo() {
    message = "Traverse the 1D array countries using a while loop and concatenate each value from playerNames and playerPoints to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testNames, testPoints);
    String actual = testBasketball.getPlayersInfo();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, int[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += firstTestValues[index] + ": " + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: For Loops`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`LoopPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that can paint in The Neighborhood
 */
public class LoopPainter extends Painter {

  /*
   * Sets the x and y location, starting direction, and amount
   * of paint to the specified values
   */
  public LoopPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the LoopPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the LoopPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the LoopPainter has paint
   */
  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  /*
   * Moves the LoopPainter a specified number of steps
   */
  public void moveTimes(int numSteps) {
    for (int steps = 0; steps < numSteps; steps++) {
      if (canMove()) {
        move();
      }
    }
  }

  /*
   * Paints and moves using the colors in paintColors
   */
  public void paintAndMove(String[] paintColors) {
    for (int index = 0; index < paintColors.length; index++) {
      paintIfHasPaint(paintColors[index]);
      moveIfCanMove();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: For Loops #1`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`LoopPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that can paint in The Neighborhood
 */
public class LoopPainter extends Painter {

  /*
   * Sets the x and y location, starting direction, and amount
   * of paint to the specified values
   */
  public LoopPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the LoopPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the LoopPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the LoopPainter has paint
   */
  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  /*
   * Moves the LoopPainter a specified number of steps
   */
  public void moveTimes(int numSteps) {
    for (int steps = 0; steps < numSteps; steps++) {
      if (canMove()) {
        move();
      }
    }
  }

  /*
   * Paints and moves using the colors in paintColors
   */
  public void paintAndMove(String[] paintColors) {
    for (int index = 0; index < paintColors.length; index++) {
      paintIfHasPaint(paintColors[index]);
      moveIfCanMove();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: For Loops #2`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`LoopPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that can paint in The Neighborhood
 */
public class LoopPainter extends Painter {

  /*
   * Sets the x and y location, starting direction, and amount
   * of paint to the specified values
   */
  public LoopPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the LoopPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the LoopPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the LoopPainter has paint
   */
  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  /*
   * Moves the LoopPainter a specified number of steps
   */
  public void moveTimes(int numSteps) {
    for (int steps = 0; steps < numSteps; steps++) {
      if (canMove()) {
        move();
      }
    }
  }

  /*
   * Paints and moves using the colors in paintColors
   */
  public void paintAndMove(String[] paintColors) {
    for (int index = 0; index < paintColors.length; index++) {
      paintIfHasPaint(paintColors[index]);
      moveIfCanMove();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: For Loops #3`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
}`},{path:`LoopPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that can paint in The Neighborhood
 */
public class LoopPainter extends Painter {

  /*
   * Sets the x and y location, starting direction, and amount
   * of paint to the specified values
   */
  public LoopPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the LoopPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves forward one space if the LoopPainter can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints the current space if the LoopPainter has paint
   */
  public void paintIfHasPaint(String color) {
    if (hasPaint()) {
      paint(color);
    }
  }

  /*
   * Moves the LoopPainter a specified number of steps
   */
  public void moveTimes(int numSteps) {
    for (int steps = 0; steps < numSteps; steps++) {
      if (canMove()) {
        move();
      }
    }
  }

  /*
   * Paints and moves using the colors in paintColors
   */
  public void paintAndMove(String[] paintColors) {
    for (int index = 0; index < paintColors.length; index++) {
      paintIfHasPaint(paintColors[index]);
      moveIfCanMove();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Converting Loops (a)`,lesson:`Lesson 5: For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] jamaicaNames = {"Jayden", "Aiden", "Joshua", "Daniel", "Nathaniel", "Nathan", "Malachi",
                             "Amelia", "Arianna", "Ariana", "Kaira", "Gabrielle", "Gabriella", "Brianna"};

    Names jamaica = new Names("Jamaica", jamaicaNames);

    System.out.println("Popular Names in " + jamaica.getCountryName() + ":");
    System.out.println(jamaica);
    
  }
}`},{path:`Names.java`,text:`/*
 * Manages data about popular names
 */
public class Names {

  private String countryName;       // The name of a country
  private String[] popularNames;    // The 1D array of the most popular names in a country

  /*
   * Sets countryName to the specified country name and initializes popularNames
   * to the specified 1D array of the most popular names in a country
   */
  public Names(String countryName, String[] popularNames) {
    this.countryName = countryName;
    this.popularNames = popularNames;
  }

  /*
   * Returns the name of the country
   */
  public String getCountryName() {
    return countryName;
  }

  /*
   * Returns a String containing each name in popularNames
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this while loop to use a for loop.
     * -----------------------------------------------------------------------------
     */

    int index = 0;

    while (index < popularNames.length) {
      result += popularNames[index] + "\\n";
      index++;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`NamesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Names.java Test")
public class NamesTest {

  String[] testCountryNames;
  Names testNames;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testCountryNames = new String[]{"Jayden", "Aiden", "Joshua", "Daniel", "Nathaniel", "Nathan", "Malachi",
                                    "Amelia", "Arianna", "Ariana", "Kaira", "Gabrielle", "Gabriella", "Brianna"};

    testNames = new Names("Some Country Name", testCountryNames);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the most popular names in a country => ")
  public void testNamesToString() {
    message = "Traverse the 1D array popularNames using a for loop and concatenate each value to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testCountryNames);
    String actual = testNames.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] testValues) {
    String result = "";

    for (int index = 0; index < testValues.length; index++) {
      result += testValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Converting Loops (b)`,lesson:`Lesson 5: For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] states = {"California", "Florida", "Kentucky", "Maryland", "New York", "Oklahoma", "Texas", "Washington"};
    int[] rates = {4871, 4743, 4113, 4057, 3745, 4714, 4917, 3594};

    UVIndex stateRates = new UVIndex(states, rates);
    
    System.out.println(stateRates);
    
  }
}`},{path:`UVIndex.java`,text:`/*
 * Manages data about the UV rates for multiple states
 */
public class UVIndex {

  private String[] stateNames;    // The 1D array of state names
  private int[] rates;            // The 1D array of UV rates for each state

  /*
   * Initializes stateNames to the specified 1D array of state names
   * and rates to the specified 1D array of UV rates for each state
   */
  public UVIndex(String[] stateNames, int[] rates) {
    this.stateNames = stateNames;
    this.rates = rates;
  }

  /*
   * Returns a String containing the names of each state and their UV rates
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this while loop to use a for loop.
     * -----------------------------------------------------------------------------
     */

    int index = 0;

    while (index < stateNames.length) {
      result += stateNames[index] + ": " + rates[index] + "\\n";
      index++;
    }

    return result;
  }
  
}`}],validationFiles:[{path:`UVIndexTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("UVIndex.java Test")
public class UVIndexTest {

  String[] testStates;
  int[] testRates;
  UVIndex testUVIndex;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testStates = new String[]{"California", "Florida", "Kentucky", "Maryland", "New York", "Oklahoma", "Texas", "Washington"};
    testRates = new int[]{4871, 4743, 4113, 4057, 3745, 4714, 4917, 3594};
    testUVIndex = new UVIndex(testStates, testRates);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name of each state and its UV rate => ")
  public void testUVIndexToString() {
    message = "Traverse the 1D array stateNames using a for loop and concatenate each value from stateNames and rates to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testStates, testRates);
    String actual = testUVIndex.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, int[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += firstTestValues[index] + ": " + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Converting Loops (c)`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a StreetPainter object
    StreetPainter painter = new StreetPainter(8, 2, "south", 50);

    // Creates a 1D array of paint colors
    String[] colors = {"red", "blue", "orange", "purple", "green"};

    // Calls the paintLines method with the 1D array of colors
    painter.paintLines(colors, 4);
    
  }
}`},{path:`StreetPainter.java`,text:`import org.code.neighborhood.*;

/*
 * A Painter that paints the street in The Neighborhood
 */
public class StreetPainter extends Painter {
  /* 
   * Instance variable to store a 1D array of paint colors 
   */
  private String[] paintColors;

  /*
   * Sets the x and y location, direction, and amount
   * of paint to the specified values
   */
  public StreetPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Paints the specified number of lines using the colors in paintColors
   */
  public void paintLines(String[] paintColors, int numSides) {
    // Store colors in the instance variable
    this.paintColors = paintColors;

    for (int count = 0; count < numSides; count++) {
      paintMultiLine(paintColors);
      turnRight();
    }
  }

  /*
   * Turns the SquarePainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Paints a multicolored line using the colors in paintColors
   */
  public void paintMultiLine(String[] paintColors) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this while loop to use a for loop.
     * -----------------------------------------------------------------------------
     */
    int index = 0;
    while (index < paintColors.length) {
      paint(paintColors[index]);
      move();
      index++;
    }
  }
}
`}],validationFiles:[{path:`StreetPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
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
@DisplayName("StreetPainter.java Test")
public class StreetPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  StreetPainter testStreetPainter;
  String[] testPaintColors;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @BeforeEach
  public void setupStreetPainter() {
    testPaintColors = new String[]{"red", "orange", "yellow", "green"};
    
    testStreetPainter = partialMockBuilder(StreetPainter.class)
      .withConstructor(8, 2, "south", 50)
      .addMockedMethods("paint", "move")
      .createMock();
  }

  @Test
  @Order(1)
  @DisplayName("Paint a multicolored line with each value in paintColors then turn right => ")
  public void testPaintMultiLine() {
    testStreetPainter.paint(testPaintColors[0]);
    testStreetPainter.move();
    testStreetPainter.paint(testPaintColors[1]);
    testStreetPainter.move();
    testStreetPainter.paint(testPaintColors[2]);
    testStreetPainter.move();
    testStreetPainter.paint(testPaintColors[3]);
    testStreetPainter.move();
    
    replay(testStreetPainter);

    testStreetPainter.paintMultiLine(testPaintColors);
    verify(testStreetPainter);
  }

  @Test
  @Order(2)
  @DisplayName("The StreetPainter object paints a square in The Neighborhood => ")
  public void testStreetPainterPaintsSquare() {
    message = "The StreetPainter object does not paint a square in The Neighborhood." + messageGap;
  
    boolean[][] expected = new boolean[12][12];
  
    expected[3] = new boolean[]{false, false, true, true, true, true, true, true, false, false, false, false};
    expected[4] = new boolean[]{false, false, true, false, false, false, false, true, false, false, false, false};
    expected[5] = new boolean[]{false, false, true, false, false, false, false, true, false, false, false, false};
    expected[6] = new boolean[]{false, false, true, false, false, false, false, true, false, false, false, false};
    expected[7] = new boolean[]{false, false, true, false, false, false, false, true, false, false, false, false};
    expected[8] = new boolean[]{false, false, true, true, true, true, true, true, false, false, false, false};
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Converting Loops (d)`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    // Creates a BackgroundPainter object
    BackgroundPainter painter = new BackgroundPainter(0, 0, "east", 100);

    // Creates a 1D array of paint colors
    String[] colors = {"pink", "lightpink", "hotpink", "deeppink", "firebrick", "red",
                       "crimson", "salmon", "lightsalmon", "mediumvioletred"};

    // Calls the paintMultiBackground() method with the 1D array of colors
    painter.paintMultiBackground(colors);

  }
}`},{path:`BackgroundPainter.java`,text:`import org.code.neighborhood.*;

/*
 * A Painter that paints the street in The Neighborhood
 */
public class BackgroundPainter extends Painter {

  /*
   * Sets the x and y location, direction, and amount
   * of paint to the specified values
   */
  public BackgroundPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the BackgroundPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Paints a multicolored background using the colors in paintColors
   */
  public void paintMultiBackground(String[] paintColors) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this while loop to use a for loop.
     * -----------------------------------------------------------------------------
     */
    
    int index = 0;

    while (index < paintColors.length) {
      paintLine(paintColors[index]);
      paint(paintColors[index]);
      turnToNextRow();
      index++;
    }
  }

  /*
   * Paints a line while the BackgroundPainter can move
   */
  public void paintLine(String color) {
    while (canMove()) {
      paint(color);
      move();
    }
  }

  /*
   * Turns to the next row based on the direction the BackgroundPainter is facing
   */
  public void turnToNextRow() {
    if (facingEast()) {
      turnRight();

      if (canMove()) {
        move();
      }
      
      turnRight();
    }
    else {
      turnLeft();

      if (canMove()) {
        move();
      }
      
      turnLeft();
    }
  }
  
}`}],validationFiles:[{path:`BackgroundPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
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
@DisplayName("BackgroundPainter.java Test")
public class BackgroundPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  BackgroundPainter testBackgroundPainter;
  String[] testPaintColors;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @BeforeEach
  public void setupStreetPainter() {
    testPaintColors = new String[]{"red", "orange", "yellow"};
    
    testBackgroundPainter = partialMockBuilder(BackgroundPainter.class)
      .withConstructor(0, 0, "east", 100)
      .addMockedMethods("paintLine", "paint", "turnToNextRow")
      .createMock();
  }

  @Test
  @Order(1)
  @DisplayName("Paint a multicolored background with each value in paintColors => ")
  public void testPaintMultiBackground() {
    testBackgroundPainter.paintLine(testPaintColors[0]);
    testBackgroundPainter.paint(testPaintColors[0]);
    testBackgroundPainter.turnToNextRow();
    testBackgroundPainter.paintLine(testPaintColors[1]);
    testBackgroundPainter.paint(testPaintColors[1]);
    testBackgroundPainter.turnToNextRow();
    testBackgroundPainter.paintLine(testPaintColors[2]);
    testBackgroundPainter.paint(testPaintColors[2]);
    testBackgroundPainter.turnToNextRow();
    
    replay(testBackgroundPainter);

    testBackgroundPainter.paintMultiBackground(testPaintColors);
    verify(testBackgroundPainter);
  }

  @Test
  @Order(2)
  @DisplayName("The BackgroundPainter object paints a multicolored background in The Neighborhood => ")
  public void testBackgroundPainterPaintsBackground() {
    message = "The BackgroundPainter object does not paint the background in The Neighborhood." + messageGap;
  
    boolean[][] expected = setAllToTrue(10);
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }

  private boolean[][] setAllToTrue(int gridSize) {
    boolean[][] temp = new boolean[gridSize][gridSize];

    for (int row = 0; row < temp.length; row++) {
      for (int col = 0; col < temp[0].length; col++) {
        temp[row][col] = true;
      }
    }

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: Using for Loops (a)`,lesson:`Lesson 5: For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] names = {"rose", "tulip", "daisy", "sunflower", "orchid", "jasmine", "lavender", "marigold"};
    
    String[] colors = {"red, pink, white", "red, pink, white", "white, yellow", "yellow, orange", "purple, pink, white",
                       "white, yellow", "purple", "orange, yellow"};
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Flowers object with the 1D arrays names and colors
     * then print the Flowers object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Flowers.java`,text:`/*
 * Manages data about flowers
 */
public class Flowers {

  private String[] flowerNames;    // The 1D array of flower names
  private String[] flowerColors;   // The 1D array of typical colors for each flower

  /*
   * Initializes flowerNames to the specified 1D array of flower names and
   * flowerColors to the specified 1D array of typical colors for each flower
   */
  public Flowers(String[] flowerNames, String[] flowerColors) {
    this.flowerNames = flowerNames;
    this.flowerColors = flowerColors;
  }

  /*
   * Returns a String containing the name and typical colors for each flower
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Concatenate each flower name from flowerNames and its typical colors from
     * flowerColors to result
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`FlowersTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Flowers.java Test")
public class FlowersTest {

  String[] testFlowerNames;
  String[] testFlowerColors;
  Flowers testFlowers;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testFlowerNames = new String[]{"rose", "tulip", "daisy", "sunflower", "orchid", "jasmine", "lavender", "marigold"};

    testFlowerColors = new String[]{"red, pink, white", "red, pink, white", "white, yellow", "yellow, orange",
                                    "purple, pink, white", "white, yellow", "purple", "orange, yellow"};
    
    testFlowers = new Flowers(testFlowerNames, testFlowerColors);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name and typical colors of each flower => ")
  public void testFlowersToString() {
    message = "Traverse the 1D array flowerNames using a for loop and concatenate each value from flowerNames and flowerColors to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testFlowerNames, testFlowerColors);
    String actual = testFlowers.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, String[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += firstTestValues[index] + ": " + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Using for Loops (b)`,lesson:`Lesson 5: For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] buses = {"2621", "1260", "418", "4522", "3124", "HT1502", "142"};
    String[] routes = {"J711", "M351", "3", "M271", "M373", "W796", "W633"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a SchoolBus object with the 1D arrays buses and routes
     * then print the SchoolBus object.
     * -----------------------------------------------------------------------------
     */





    
    
  }
}`},{path:`SchoolBus.java`,text:`/*
 * Manages data about school buses and their routes
 */
public class SchoolBus {

  private String[] busNumbers;      // The 1D array of bus numbers
  private String[] routeNumbers;    // The 1D array of route numbers for each school bus

  /*
   * Initializes busNumbers to the specified 1D array of bus numbers and
   * routeNumbers to the 1D array of route numbers for each school bus
   */
  public SchoolBus(String[] busNumbers, String[] routeNumbers) {
    this.busNumbers = busNumbers;
    this.routeNumbers = routeNumbers;
  }

  /*
   * Returns a String containing each bus number and its route number
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Concatenate each bus number from busNumbers and its route number from
     * routeNumbers to result
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`SchoolBusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SchoolBus.java Test")
public class SchoolBusTest {

  String[] testBusNumbers;
  String[] testRouteNumbers;
  SchoolBus testSchoolBus;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testBusNumbers = new String[]{"2621", "1260", "418", "4522", "3124", "HT1502", "142"};
    testRouteNumbers = new String[]{"J711", "M351", "3", "M271", "M373", "W796", "W633"};
    testSchoolBus = new SchoolBus(testBusNumbers, testRouteNumbers);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the bus and route number of each school bus => ")
  public void testSchoolBusToString() {
    message = "Traverse the 1D array busNumbers using a for loop and concatenate each value from busNumbers and routeNumbers to result.";
    message += "\\n        Be sure to use the \\\\n escape sequence to add a new line after each value.";
    message += messageGap;
      
    String expected = getExpectedString(testBusNumbers, testRouteNumbers);
    String actual = testSchoolBus.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected output
   */
  private String getExpectedString(String[] firstTestValues, String[] secondTestValues) {
    String result = "";

    for (int index = 0; index < firstTestValues.length; index++) {
      result += "Bus #" + firstTestValues[index] + " - Route #" + secondTestValues[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Using for Loops (c)`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a ParkPainter object, and create a 1D array of paint colors.
     * Then call the paintPark() method with the 1D array of colors.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`ParkPainter.java`,text:`import org.code.neighborhood.*;
import java.util.Scanner;

/*
 * Represents a Painter that paints in a park
 */
public class ParkPainter extends Painter {

  /*
   * Sets the starting x and y location, direction
   * amount of paint to the specified values
   */
  public ParkPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Paints a line while ParkPainter can move forward
   */
  public void paintLine(String color) {
    while (canMove()) {
      paint(color);
      move();
    }
  }

  /*
   * Paints lines around the park using the colors in paintColors
   */
  public void paintPark(String[] paintColors) {
    /* --------------------------------------- TO DO ---------------------------------------
     * ✅ Traverse the 1D array paintColors to paint a line with each color then turn left.
     * -------------------------------------------------------------------------------------
     */
    
    
  }

}`}],validationFiles:[{path:`ParkPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
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
@DisplayName("ParkPainter.java Test")
public class ParkPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  ParkPainter testParkPainter;
  String[] testPaintColors;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @BeforeEach
  public void setupParkPainter() {
    testPaintColors = new String[]{"red", "orange", "yellow", "green"};
    
    testParkPainter = partialMockBuilder(ParkPainter.class)
      .withConstructor(1, 9, "east", 40)
      .addMockedMethods("paintLine", "turnLeft")
      .createMock();
  }

  @Test
  @Order(1)
  @DisplayName("A ParkPainter object is instantiated at (1, 9) facing east with 40 units of paint => ")
  public void testParkPainterObject() {
    message = "The ParkPainter object does not start at (1, 9) facing east with 40 units of paint.";
    message += "\\n        Make sure the correct values are passed when calling the constructor.";
    message += "\\n       ";

    assertEquals(1, primaryPainterLog.getStartingPosition().getX(), message);
    assertEquals(9, primaryPainterLog.getStartingPosition().getY(), message);
    assertEquals("east", primaryPainterLog.getStartingPosition().getDirection(), message);
    assertEquals(40, primaryPainterLog.getStartingPaintCount(), message);
  }

  @Test
  @Order(2)
  @DisplayName("Paint a line with each value in paintColors then turn left => ")
  public void testParkPainterMoveAndPaint() {    
    testParkPainter.paintLine(testPaintColors[0]);
    testParkPainter.turnLeft();
    testParkPainter.paintLine(testPaintColors[1]);
    testParkPainter.turnLeft();
    testParkPainter.paintLine(testPaintColors[2]);
    testParkPainter.turnLeft();
    testParkPainter.paintLine(testPaintColors[3]);
    testParkPainter.turnLeft();
    
    replay(testParkPainter);

    testParkPainter.paintPark(testPaintColors);
    verify(testParkPainter);
  }

  @Test
  @Order(3)
  @DisplayName("The ParkPainter object paints a rectangle around the park benches in The Neighborhood => ")
  public void testParkPainterPaintsAroundParkBenches() {
    message = "The ParkPainter object does not paint a rectangle around the park benches.";
    message += "\\n        Create a String array of paint colors and call the paintPark() method.";
    message += "\\n       ";
  
    boolean[][] expected = new boolean[12][12];
  
    expected[1] = new boolean[]{false, false, true, true, true, true, true, true, true, true, false, false};
    expected[2] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[3] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[4] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[5] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[6] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[7] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[8] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[9] = new boolean[]{false, false, true, false, false, false, false, false, false, true, false, false};
    expected[10] = new boolean[]{false, false, true, true, true, true, true, true, true, true, false, false};
    
    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Using for Loops (d)`,lesson:`Lesson 5: For Loops`,view:`neighborhood`,grid:`0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a TrafficPainter object and create a 1D array of paint colors.
     * ✅ Call the shiftColors() method with the paint colors and shift amount.
     * ✅ Call the navigateTraffic() method with the modified paint colors.
     * -----------------------------------------------------------------------------
     */

    
    
    
  }
}`},{path:`TrafficPainter.java`,text:`import org.code.neighborhood.*;
import java.util.Scanner;

/*
 * Represents a Painter that navigates and paints in The Neighborhood
 */
public class TrafficPainter extends Painter {

  /*
   * Sets the x and y location, direction, and amount
   * of paint to the specified values
   */
  public TrafficPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the TrafficPainter to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Shifts the colors in the 1D array paintColors to the left
   */
  public void shiftColors(String[] paintColors, int shiftAmount) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Shift the colors in paintColors to the left by shiftAmount positions.
     * ✅ Loop shiftAmount times, moving each element one position to the left.
     * ✅ Store the first color before shifting, then move each color forward.
     * ✅ Place the stored first color at the end of the array after shifting.
     * -----------------------------------------------------------------------------
     */
    
  
  
  }
  
  /*
   * Paints a line while the TrafficPainter can move forward
   */
  public void paintLine(String color) {
    while (canMove()) {
      paint(color);
      move();
    }
  }

  /*
   * Navigates and paints a path using the colors in paintColors
   */
  public void navigateTraffic(String[] paintColors) {
    for (int index = 0; index < paintColors.length; index++) {
      paintLine(paintColors[index]);
      turnRight();
      paintLine(paintColors[index]);
      turnLeft();
    }
  }
  
}`}],validationFiles:[{path:`TrafficPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
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
@DisplayName("TrafficPainter.java Test")
public class TrafficPainterTest {

  static NeighborhoodLog neighborhood;
  static PainterLog[] painterLogs;
  static PainterLog primaryPainterLog;
  TrafficPainter testTrafficPainter;
  String[] testPaintColors;
  static String message;
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }

  @BeforeEach
  public void setupTrafficPainter() {
    testPaintColors = new String[]{"red", "orange", "yellow"};
    
    testTrafficPainter = partialMockBuilder(TrafficPainter.class)
      .withConstructor(0, 13, "north", 50)
      .addMockedMethods("paintLine", "turnLeft", "turnRight")
      .createMock();
  }

  @Test
  @Order(1)
  @DisplayName("A TrafficPainter object is instantiated at (0, 13) facing north with 50 units of paint => ")
  public void testTrafficPainterObjectStart() {
    message = "The TrafficPainter object does not start at (0, 13) facing north with 50 units of paint.";
    message += "\\n        Make sure the correct values are passed when calling the constructor.";
    message += "\\n       ";

    assertEquals(0, primaryPainterLog.getStartingPosition().getX(), message);
    assertEquals(13, primaryPainterLog.getStartingPosition().getY(), message);
    assertEquals("north", primaryPainterLog.getStartingPosition().getDirection(), message);
    assertEquals(50, primaryPainterLog.getStartingPaintCount(), message);
  }

  @Test
  @Order(2)
  @DisplayName("Check if shiftColors() correctly shifts the paint colors => ")
  public void testShiftColors() {    
      String[] originalColors = {"red", "orange", "yellow"};
      String[] testColors = {"red", "orange", "yellow"};
  
      testTrafficPainter.shiftColors(testColors, 1);
  
      assertNotEquals(originalColors[0], testColors[0], "The first color should have shifted.");
      assertEquals(originalColors[0], testColors[testColors.length - 1], "The original first color should now be last.");
  }

  
}`}],dataFiles:[]},{name:`Predict and Run: Reading Text Files`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {
  public static void main(String[] args) throws IOException{
    
    Student myStudent = new Student();
    System.out.println(myStudent);

    myStudent = myStudent.createStudentFromFile("student_data.txt");
    System.out.println(myStudent);
    
  }
}`},{path:`Student.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class Student {
    private String name;

    public Student() {
        name = "unknown";
    }

    public Student(String name) {
        this.name = name;
    }

    public Student createStudentFromFile(String filename) throws IOException {
        File file = new File(filename);
        Scanner scanner = new Scanner(file);
        String name = scanner.nextLine();
        scanner.close();
        return new Student(name);
    }

    public String getName() {
        return name;
    }

    public String toString() {
        return name;
    }
}
`}],validationFiles:[],dataFiles:[{path:`student_data.txt`,text:`Aiden Chen
Fatima Ahmed
Liam Smith
Priya Patel
Santiago Garcia
Hiroshi Tanaka
Amara Johnson
Elena Rossi
Nguyen Tran
Olga Ivanova`}]},{name:`Investigate and Modify: Reading Text Files`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {
  public static void main(String[] args) throws IOException{
    
    Student myStudent = new Student();
    System.out.println(myStudent);

    myStudent = myStudent.createStudentFromFile("student_data.txt");
    System.out.println(myStudent);
    
  }
}
  `},{path:`Student.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class Student {
    private String name;

    public Student() {
        name = "unknown";
    }

    public Student(String name) {
        this.name = name;
    }

    public Student createStudentFromFile(String filename) throws IOException {
        File file = new File(filename);
        Scanner scanner = new Scanner(file);
        String name = scanner.nextLine();
        scanner.close();
        return new Student(name);
    }

    public String getName() {
        return name;
    }

    public String toString() {
        return name;
    }
}`}],validationFiles:[],dataFiles:[{path:`student_data.txt`,text:`Aiden Chen
Fatima Ahmed
Liam Smith
Priya Patel
Santiago Garcia
Hiroshi Tanaka
Amara Johnson
Elena Rossi
Nguyen Tran
Olga Ivanova`}]},{name:`Investigate and Modify: Processing File Data`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {
  public static void main(String[] args) throws IOException{
    
    Student myStudent = new Student();
    System.out.println(myStudent);

    myStudent = myStudent.createStudentFromFile("student_data.txt");
    System.out.println(myStudent);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */

    
    
  }
}
  `},{path:`Student.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class Student {
    private String name;
    private double gpa;

    public Student() {
      name = "unknown";
      gpa = 0.0;
    }

    public Student(String name) {
      this.name = name;
      gpa = 0.0;
    }

    public Student(String name, double gpa){
      this.name = name;
      this.gpa = gpa;
    }

    public Student createStudentFromFile(String filename) throws IOException {
      File file = new File(filename);
      Scanner scanner = new Scanner(file);
      String name = scanner.nextLine();
      scanner.close();
      return new Student(name);
    }

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  
  
    public String getName() {
        return name;
    }

    public String toString() {
        return name;
    }
}`}],validationFiles:[],dataFiles:[{path:`gpa_data.txt`,text:`3.85
2.67
3.14
3.76
2.94
3.45
3.98
2.75
3.33
3.89`},{path:`student_data.txt`,text:`Aiden Chen
Fatima Ahmed
Liam Smith
Priya Patel
Santiago Garcia
Hiroshi Tanaka
Amara Johnson
Elena Rossi
Nguyen Tran
Olga Ivanova`}]},{name:`Reading and Processing Files (a)`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {

   /* ----------------------------------- TO DO -----------------------------------
    * ✅ Complete the missing portion of the main method in order to read from
    * a file using the countTemperatureValues(String filename) method
    * -----------------------------------------------------------------------------
    */
  public static void main(String[] args) /* ---- 🔎 REPLACE THIS COMMENT WITH YOUR CODE ---- */ {

    Temperature myTemperature = new Temperature();
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countTemperatureValues(String filename) and print the results
     * -----------------------------------------------------------------------------
     */    
    


    
    
    
  }
}`},{path:`Temperature.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner; 

public class Temperature {
    private double value;

    public Temperature() {
        this.value = 0.0;
    }

    public Temperature(double value) {
        this.value = value;
    }
    
    /*
     * Counts the number of temperature values in the associated .txt file
     */
    public int countTemperatureValues(String filename) throws IOException {    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use the File and Scanner classes to read from "filename" return 
     * the total number of temperatures that appear in the file.
     * -----------------------------------------------------------------------------
     */




      
      return -1;
    }
  
    public double getValue() {
        return value;
    }

    public void setValue(double value) {
        this.value = value;
    }
}`}],validationFiles:[],dataFiles:[{path:`temperature_data.txt`,text:`32.0°F  
45.5°F  
60.0°F  
72.5°F  
85.0°F  
95.7°F  
100.0°F  
110.3°F  
55.4°F  
40.2°F  
66.8°F  
77.1°F  
82.6°F  
90.0°F  
104.9°F`}]},{name:`Reading and Processing Files (b)`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {

   /* ----------------------------------- TO DO -----------------------------------
    * ✅ Complete the missing portion of the main method in order to read from
    * a file using the findTotalInventoryValue(String filename) method
    * -----------------------------------------------------------------------------
    */
  public static void main(String[] args) throws IOException {

    Inventory myInventory = new Inventory();
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findTotalInventoryValue(String filename) and print the results
     * -----------------------------------------------------------------------------
     */    
    


    
    
    
  }
}`},{path:`Inventory.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner; 

public class Inventory {
    private String name;
    private double price;

    public Inventory() {
        this.name = "unknown";
        this.price = 0.0;
    }

    public Inventory(String name, double price) {
        this.name = name;
        this.price = price;
    }

    /*
     * Calculates the total inventory value from the associated .txt file
     */
    public double findTotalInventoryValue(String filename) throws IOException {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use the File and Scanner classes to read from "filename" and return 
     * the sum of all of the inventory prices that appear in the file.
     * -----------------------------------------------------------------------------
     */
      double totalValue = -1.0;
      
      
      
      return totalValue;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public double getPrice() {
        return price;
    }

    public void setPrice(double price) {
        this.price = price;
    }
}`}],validationFiles:[],dataFiles:[{path:`inventory_data.txt`,text:`34.99
23.50
45.00
12.75
78.20
56.10
19.99
5.49
89.95
43.80
60.25
10.00
25.99
67.40
29.95`}]},{name:`Reading and Processing Files (c)`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {

   /* ----------------------------------- TO DO -----------------------------------
    * ✅ Complete the missing portion of the main method in order to read from
    * a file using the countFalseResponses(String filename) method
    * -----------------------------------------------------------------------------
    */
  public static void main(String[] args) /* ---- 🔎 REPLACE THIS COMMENT WITH YOUR CODE ---- */ {

    Survey mySurvey = new Survey();
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countFalseResponses(String filename) and print the results
     * -----------------------------------------------------------------------------
     */    
    


    
    
    
  }
}`},{path:`Survey.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class Survey {
    private boolean response;

    public Survey() {
        this.response = false;
    }

    public Survey(boolean response) {
        this.response = response;
    }

    /*
     * Counts the number of false responses in the associated .txt file
     */
    public int countFalseResponses(String filename) throws IOException {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use the File and Scanner classes to read from "filename" and return 
     * the total number of false responses that appear in the file.
     * -----------------------------------------------------------------------------
     */
      int falseResponses = -1;



      
      return falseRespones;
    }

    public boolean getResponse() {
        return response;
    }

    public void setResponse(boolean response) {
        this.response = response;
    }
}`}],validationFiles:[],dataFiles:[{path:`survey_data.txt`,text:`true
false
true
true
false
true
false
false
true
false
true
true
false
false
true`}]},{name:`Reading and Processing Files (d)`,lesson:`Lesson 6: Text Files`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main {

   /* ----------------------------------- TO DO -----------------------------------
    * ✅ Complete the missing portion of the main method in order to read from
    * a file using the combinedInfo(String stateFilename, String populationFilename) method
    * -----------------------------------------------------------------------------
    */
  public static void main(String[] args) /* ---- 🔎 REPLACE THIS COMMENT WITH YOUR CODE ---- */ {

    State myState = new State();
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the combinedInfo(String stateFilename, String populationFilename)
     * -----------------------------------------------------------------------------
     */    
    


    
    
    
  }
}`},{path:`State.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class State {
    private String name;
    private int population;

    public State(){
      name = "unkown";
      population = 0;
    }
  
    public State(String name, int population) {
        this.name = name;
        this.population = population;
    }

  /*
   * Reads parallel text files and prints the combined information
   */
    public void combinedInfo(String stateFilename, String populationFilename) throws IOException {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use the File and Scanner classes to read from "stateFilename"
     * "populationFilename" and print the parallel values. Printed values should  
     * be in the format "State - Population: [value]" like the example below:
     *    California - Population: 39512223
     * -----------------------------------------------------------------------------
     */




      
      
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public int getPopulation() {
        return population;
    }

    public void setPopulation(int population) {
        this.population = population;
    }
}`}],validationFiles:[],dataFiles:[{path:`population_data.txt`,text:`39512223
28995881
21477737
19453561
12801989
12671821
11689100
10617423
10488084
9986857`},{path:`state_data.txt`,text:`California
Texas
Florida
New York
Pennsylvania
Illinois
Ohio
Georgia
North Carolina
Michigan`}]},{name:`Investigate and Modify: Reading a File`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Channel tedTalks = new Channel("TED", "titles.txt", "durations.txt", "views.txt");

    tedTalks.run();
    
    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
    
  }
}`},{path:`Channel.java`,text:`import java.util.Scanner;

/*
 * Represents a channel on YouTube
 */
public class Channel {

  private String name;             // The name of a channel
  private String[] videoTitles;    // The 1D array of video titles on a channel
  private int[] durations;         // The 1D array of the duration of each video
  private int[] numViews;          // The 1D array of the number of views for each video

  /*
   * Sets name to the specified channel name and reads the data from titlesFile, durationsFile,
   * and numViewsFile to initialize the 1D arrays videoTitles, durations, and numViews
   */
  public Channel(String name, String titlesFile, String durationsFile, String numViewsFile) {
    this.name = name;
    
    videoTitles = FileReader.toStringArray(titlesFile);
    durations = FileReader.toIntArray(durationsFile);
    numViews = FileReader.toIntArray(numViewsFile);
  }

  /*
   * Repeatedly prompts the user and shows the
   * results until they quit the program
   */
  public void run() {
    int selection = 0;
    
    while (selection != 3) {
      selection = promptUser();
      System.out.println(showData(selection) + "\\n");
    }

    System.out.println("Goodbye!");
  }

  /*
   * Prompts the user to choose from provided options
   */
  public int promptUser() {
    Scanner input = new Scanner(System.in);
    
    System.out.println("What would you like to know about the " + name + " channel? ");
    System.out.println("1: The shortest video");
    System.out.println("2: The video with the most views");
    System.out.println("3: Quit");
    System.out.print("Enter your choice (1, 2, or 3): ");
    
    int choice = input.nextInt();
    input.close();
    return choice;
  }

  /*
   * Returns a String containing the results based on the user's choice
   */
  public String showData(int choice) {
    int index = 0;
    String result = "";
    
    if (choice == 1) {
      index = getShortestVideo();
      result = "\\nResult Found:\\n----------\\n" + getVideoInfo(index);
    }
      
    if (choice == 2) {
      index = getMaxNumViews();
      result = "\\nResult Found:\\n----------\\n" + getVideoInfo(index);
    }

    return result;
  }

  /*
   * Returns the location of the shortest video in durations
   */
  public int getShortestVideo() {
    int minValue = durations[0];
    int minLocation = 0;

    for (int index = 0; index < durations.length; index++) {
      if (durations[index] < minValue) {
        minValue = durations[index];
        minLocation = index;
      }
    }

    return minLocation;
  }

  /*
   * Returns the location of the maximum number of views in numViews
   */
  public int getMaxNumViews() {
    int maxValue = numViews[0];
    int maxLocation = 0;
    
    for (int index = 0; index < numViews.length; index++) {
      if (numViews[index] > maxValue) {
        maxValue = numViews[index];
        maxLocation = index;
      }
    }

    return maxLocation;
  }

  /*
   * Returns the information for the video at the specified index
   */
  public String getVideoInfo(int index) {
    return videoTitles[index] + "\\nDuration: " + durations[index] + "\\nNumber of Views: " + numViews[index];
  }

  /*
   * Returns a String containing each video title and its duration and number of views
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < videoTitles.length; index++) {
      result += videoTitles[index] + " (" + durations[index] + " mins, " + numViews[index] + " views)\\n";
    }

    return result;
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
  
}`}],validationFiles:[],dataFiles:[{path:`durations.txt`,text:`10
9
7
66
55
7
8
15
13
6
9
10
9
16
13
10
15
17
21
6
15
12
18
9
9`},{path:`titles.txt`,text:`The Future Will Be Shaped by Optimists 
An Olympic Champion's Unwavering Advocacy for Mothers in Sports 
The African Swamp Protecting Earth's Environment 
Elon Musk: A future worth getting excited about 
Elon Musk talks Twitter, Tesla and how his brain works
Ukraine's Fight to Keep Educating Its Children 
Why Entrepreneurship Flourishes in The Countryside 
Stand with Ukraine in the Fight against Evil 
The Deep Sea's Medicinal Secrets 
Climate Change Isn't a Distant Threat -- It's Our Reality 
3 Things Men Can Do to Promote Gender Equity 
The Power of Purpose in Business 
Are Ad Agencies, PR Firms and Lobbyists Destroying the Climate? 
The Hidden Power of Sad Songs and Rainy Days 
What It's Like To Be a War Refugee 
Self-Assembling Robots and the Potential of Artificial Evolution 
How to Design Mosquitoes Out of Cities 
3 Ways to Find Hope in Hopeless Times 
The Critical Role of Black Mothers -- and How to Support All Moms 
What Does a Voice of the Future Sound Like? 
The Creativity and Community Behind Fanfiction 
How to Share Public Money Fairly 
Why US Laws Must Expand Beyond the Nuclear Family 
3 Ways Your Company's Data Can Jump-Start Climate Action 
What Seaweed and Cow Burps Have to Do with Climate Change `},{path:`views.txt`,text:`51273
25597
21430
3609893
4405481
38252
32369
137166
34071
39376
37296
51525
25438
82889
31855
63375
36946
45843
14674
273563
25954
39904
39526
25959
18783`}]},{name:`Practice: Writing Algorithms with Arrays (a) #1`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toIntArray() to read the exams.txt file and store
     * the result in a 1D array. Instantiate an APExams object with the 1D array,
     * then call the calcTotalExams() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageExams() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`APExams.java`,text:`/*
 * Manages data about AP CS exams taken in each state
 */
public class APExams {

  private String[] states;   // The 1D array of state names
  private int[] csExams;     // The 1D array of the number of AP CS exams taken in each state

  /*
   * Reads the data from states to initialize states and initializes csExams
   * to the specified 1D array of the number of AP CS exams taken in each state
   */
  public APExams(int[] csExams) {
    states = FileReader.toStringArray("states.txt");
    this.csExams = csExams;
  }

  /*
   * Returns the information for the state at the specified index
   */
  public String getStateInfo(int index) {
    return states[index] + "\\nNumber of Exams: " + csExams[index];
  }

  /*
   * Returns the total number of AP CS exams taken in all states
   */
  public int calcTotalExams() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse csExams to calculate and return the total number of AP CS exams
     * taken in all states.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Returns the average number of AP CS exams taken in each state
   */
  public int calcAverageExams() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average number of AP CS exams taken in each state.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Returns a String containing each state name and the number of AP CS exams taken
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < states.length; index++) {
      result += states[index] + ": " + csExams[index] + " AP CS exams taken\\n";
    }

    return result;
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
  
}`}],validationFiles:[{path:`APExamsTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("APExams.java Test")
public class APExamsTest {

  APExams testAPExams;
  int[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toIntArray("exams.txt");
    testAPExams = new APExams(testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the total number of AP CS exams taken in all states => ")
  public void testCalcTotalExams() {
    message = "Create a variable to keep track of the total number of AP CS exams. Then traverse";
    message += "\\n        the 1D array csExams and add each value to the total, then return the result.";
    message += messageGap;
      
    int expected = getExpectedResult(testData);
    int actual = testAPExams.calcTotalExams();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private int getExpectedResult(int[] values) {
    int total = 0;

    for (int num : values) {
      total += num;
    }

    return total;
  }
  
}`}],dataFiles:[{path:`exams.txt`,text:`2399
100
1587
1406
31189
2584
3251
513
14864
7221
782
429
8572
2883
521
236
1462
1191
242
7662
5451
4504
1432
400
1199
42
514
1701
403
9391
270
13304
6273
109
3754
500
714
6104
617
2159
26
2046
17307
612
150
6034
4034
352
2080
112`},{path:`states.txt`,text:`Alabama
Alaska
Arizona
Arkansas
California
Colorado
Connecticut
Delaware
Florida
Georgia
Hawaii
Idaho
Illinois
Indiana
Iowa
Kansas
Kentucky
Louisiana
Maine
Maryland
Massachusetts
Michigan
Minnesota
Mississippi
Missouri
Montana
Nebraska
Nevada
New Hampshire
New Jersey
New Mexico
New York
North Carolina
North Dakota
Ohio
Oklahoma
Oregon
Pennsylvania
Rhode Island
South Carolina
South Dakota
Tennessee
Texas
Utah
Vermont
Virginia
Washington
West Virginia
Wisconsin
Wyoming`}]},{name:`Practice: Writing Algorithms with Arrays (b) #1`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toIntArray() to read the dailyPlays.txt file and
     * store the result in a 1D array. Instantiate a Song object with the 1D
     * array, then call the calcTotalPlays() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAveragePlays() method and print the result.
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
  
}`},{path:`Song.java`,text:`/*
 * Manages data about a song
 */
public class Song {

  private String title;       // The title of the song
  private int[] songPlays;    // The 1D array of the number of times a song was played

  /*
   * Sets title to the specified title of the song and initializes songPlays
   * to the specified 1D array of the number of times a song was played
   */
  public Song(String title, int[] songPlays) {
    this.title = title;
    this.songPlays = songPlays;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the number of plays for the day at the specified index
   */
  public String getSongInfo(int index) {
    return songPlays[index] + " plays";
  }

  /*
   * Returns the total number of times the song was played
   */
  public int calcTotalPlays() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse songPlays to calculate and return the total number of times the
     * song was played on the streaming music service.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns the average number of times the song was played each day
   */
  public int calcAveragePlays() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average number of times the song was played.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns a String containing the number of plays the song had each day
   */
  public String toString() {
    String result = title + " Daily Plays\\n----------\\n";

    for (int index = 0; index < songPlays.length; index++) {
      result += "Day " + (index + 1) + ": " + songPlays[index] + "\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`SongTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Song.java Test")
public class SongTest {

  Song testSong;
  int[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toIntArray("dailyPlays.txt");
    testSong = new Song("Some Song", testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the total number of times the song was played => ")
  public void testCalcTotalPlays() {
    message = "Create a variable to keep track of the total number of times the song was played. Then";
    message += "\\n        traverse the 1D array songPlays and add each value to the total, then return the result.";
    message += messageGap;
      
    int expected = getExpectedResult(testData);
    int actual = testSong.calcTotalPlays();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private int getExpectedResult(int[] values) {
    int total = 0;

    for (int num : values) {
      total += num;
    }

    return total;
  }
  
}`}],dataFiles:[{path:`dailyPlays.txt`,text:`4750185
5102574
6149812
5811600
4506688
4873139
5009612
5562741
5799207
7068846
6882528
5372271
5364677
5532812
5901056
6708201
7723706
6543891
5942353
6314185
6903992
8020508
10905559
21273357
16216593
4510287
2037749
1483957
1224254
1091757
1500302`}]},{name:`Practice: Writing Algorithms with Arrays (c) #1`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toDoubleArray() to read the storeProfits.txt file and
     * store the result in a 1D array. Instantiate a Store object with the 1D array,
     * then call the calcTotalProfits() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageProfits() method and print the result.
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
  
}`},{path:`Store.java`,text:`/*
 * Manages data about store profits
 */
public class Store {

  private String name;              // The name of the store
  private double[] storeProfits;    // The 1D array of the profits a store made each day

  /*
   * Sets name to the specified name of the store and initializes storeProfits
   * to the specified 1D array of the profits a store made each day
   */
  public Store(String name, double[] storeProfits) {
    this.name = name;
    this.storeProfits = storeProfits;
  }

  /*
   * Returns the name of the store
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the profits for the day at the specified index
   */
  public String getProfitsInfo(int index) {
    return "Profits made: " + storeProfits[index];
  }

  /*
   * Returns the total profits made by the store
   */
  public double calcTotalProfits() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse storeProfits to calculate and return the total profits made.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns the average profits made by the store each day 
   */
  public double calcAverageProfits() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average profits made by the store each day.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns a String containing the profits the store made each day
   */
  public String toString() {
    String result = name + "'s Daily Profits\\n----------\\n";

    for (int index = 0; index < storeProfits.length; index++) {
      result += "Day " + (index + 1) + ": " + storeProfits[index] + "\\n";
    }

    return result;
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

  Store testStore;
  double[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toDoubleArray("storeProfits.txt");
    testStore = new Store("Some Store", testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the total profits made by the store => ")
  public void testCalcTotalProfits() {
    message = "Create a variable to keep track of the total profits made. Then traverse the 1D";
    message += "\\n        array storeProfits and add each value to the total, then return the result.";
    message += messageGap;
      
    double expected = getExpectedResult(testData);
    double actual = testStore.calcTotalProfits();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private double getExpectedResult(double[] values) {
    double total = 0;

    for (double num : values) {
      total += num;
    }

    return total;
  }
  
}`}],dataFiles:[{path:`storeProfits.txt`,text:`-16.28
2121.66
598.28
421.13
-256.02
1572.71
1373.42
1474.12
3427.62
772.90
6438.50
1082.35`}]},{name:`Practice: Writing Algorithms with Arrays (d) #1`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toIntArray() to read the episodes.txt file and
     * store the result in a 1D array. Instantiate a TVEpisodes object with the 1D
     * array, then call the calcTotalLength() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageLength() method and print the result.
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
  
}`},{path:`TVEpisodes.java`,text:`/*
 * Manages data about episodes in a TV series
 */
public class TVEpisodes {

  private String title;              // The title of a TV show
  private String[] episodeTitles;    // The 1D array of episode titles
  private int[] episodeLengths;      // The 1D array of the number of minutes in each episode

  /*
   * Sets title to the specified title of the TV show and initializes episodeTitles
   * to the specified 1D array of the episode titles and episodeLengths to the 1D
   * array of the number of minutes in each episode
   */
  public TVEpisodes(String title, int[] episodeLengths) {
    this.title = title;
    episodeTitles = FileReader.toStringArray("titles.txt");
    this.episodeLengths = episodeLengths;
  }

  /*
   * Returns the name of the TV show
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the episode title and its number of minutes at the specified index
   */
  public String getEpisodeInfo(int index) {
    return episodeTitles[index] + ": " + episodeLengths[index] + " minutes\\n";
  }

  /*
   * Returns the total number of minutes for all episodes
   */
  public int calcTotalLength() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse episodeLengths to calculate and return the total number of
     * minutes for all TV episodes.
     * -----------------------------------------------------------------------------
     */


    return -1;
  }

  /*
   * Returns the average number of minutes for each episode
   */
  public int calcAverageLength() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average number of minutes for each TV episode.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns a String containing the name and length of each episode
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < episodeTitles.length; index++) {
      result += episodeTitles[index] + ": " + episodeLengths[index] + " minutes\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`TVEpisodesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TVEpisodes.java Test")
public class TVEpisodesTest {

  TVEpisodes testTVEpisodes;
  int[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toIntArray("episodes.txt");
    testTVEpisodes = new TVEpisodes("Some TV Show", testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the total number of minutes for all TV episodes => ")
  public void testCalcTotalLength() {
    message = "Create a variable to keep track of the total number of minutes for all TV episodes. Then";
    message += "\\n        traverse the 1D array episodeLengths and add each value to the total, then return the result.";
    message += messageGap;
      
    int expected = getExpectedResult(testData);
    int actual = testTVEpisodes.calcTotalLength();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private int getExpectedResult(int[] values) {
    int total = 0;

    for (int num : values) {
      total += num;
    }

    return total;
  }
  
}`}],dataFiles:[{path:`episodes.txt`,text:`37
30
34
38
32
41
37
45
50
38
32
36
43
30
35
44`},{path:`titles.txt`,text:`Chapter 1: The Mandalorian
Chapter 2: The Child
Chapter 3: The Sin
Chapter 4: Sanctuary
Chapter 5: The Gunslinger
Chapter 6: The Prisoner
Chapter 7: The Reckoning
Chapter 8: Redemption
Chapter 9: The Marshal
Chapter 10: The Passenger
Chapter 11: The Heiress
Chapter 12: The Siege
Chapter 13: The Jedi
Chapter 14: The Tragedy
Chapter 15: The Believer
Chapter 16: The Rescue`}]},{name:`Practice: Writing Algorithms with Arrays (a) #2`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toIntArray() to read the exams.txt file and store
     * the result in a 1D array. Instantiate an APExams object with the 1D array,
     * then call the calcTotalExams() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageExams() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`APExams.java`,text:`/*
 * Manages data about AP CS exams taken in each state
 */
public class APExams {

  private String[] states;   // The 1D array of state names
  private int[] csExams;     // The 1D array of the number of AP CS exams taken in each state

  /*
   * Reads the data from states to initialize states and initializes csExams
   * to the specified 1D array of the number of AP CS exams taken in each state
   */
  public APExams(int[] csExams) {
    states = FileReader.toStringArray("states.txt");
    this.csExams = csExams;
  }

  /*
   * Returns the information for the state at the specified index
   */
  public String getStateInfo(int index) {
    return states[index] + "\\nNumber of Exams: " + csExams[index];
  }

  /*
   * Returns the total number of AP CS exams taken in all states
   */
  public int calcTotalExams() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse csExams to calculate and return the total number of AP CS exams
     * taken in all states.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Returns the average number of AP CS exams taken in each state
   */
  public int calcAverageExams() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average number of AP CS exams taken in each state.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Returns a String containing each state name and the number of AP CS exams taken
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < states.length; index++) {
      result += states[index] + ": " + csExams[index] + " AP CS exams taken\\n";
    }

    return result;
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
  
}`}],validationFiles:[{path:`APExamsTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("APExams.java Test")
public class APExamsTest {

  APExams testAPExams;
  int[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toIntArray("exams.txt");
    testAPExams = new APExams(testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average number of AP CS exams taken in each state => ")
  public void testCalcAverageExams() {
    message = "Use the calcTotalExams() method to get the total number of AP CS exams. Then divide";
    message += "\\n        this result by the length of the 1D array csExams and return the result.";
    message += messageGap;
      
    int expected = getExpectedResult(testData);
    int actual = testAPExams.calcAverageExams();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private int getExpectedResult(int[] values) {
    int total = 0;

    for (int num : values) {
      total += num;
    }

    return total / values.length;
  }
  
}`}],dataFiles:[{path:`exams.txt`,text:`2399
100
1587
1406
31189
2584
3251
513
14864
7221
782
429
8572
2883
521
236
1462
1191
242
7662
5451
4504
1432
400
1199
42
514
1701
403
9391
270
13304
6273
109
3754
500
714
6104
617
2159
26
2046
17307
612
150
6034
4034
352
2080
112`},{path:`states.txt`,text:`Alabama
Alaska
Arizona
Arkansas
California
Colorado
Connecticut
Delaware
Florida
Georgia
Hawaii
Idaho
Illinois
Indiana
Iowa
Kansas
Kentucky
Louisiana
Maine
Maryland
Massachusetts
Michigan
Minnesota
Mississippi
Missouri
Montana
Nebraska
Nevada
New Hampshire
New Jersey
New Mexico
New York
North Carolina
North Dakota
Ohio
Oklahoma
Oregon
Pennsylvania
Rhode Island
South Carolina
South Dakota
Tennessee
Texas
Utah
Vermont
Virginia
Washington
West Virginia
Wisconsin
Wyoming`}]},{name:`Practice: Writing Algorithms with Arrays (b) #2`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toIntArray() to read the dailyPlays.txt file and
     * store the result in a 1D array. Instantiate a Song object with the 1D
     * array, then call the calcTotalPlays() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAveragePlays() method and print the result.
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
  
}`},{path:`Song.java`,text:`/*
 * Manages data about a song
 */
public class Song {

  private String title;       // The title of the song
  private int[] songPlays;    // The 1D array of the number of times a song was played

  /*
   * Sets title to the specified title of the song and initializes songPlays
   * to the specified 1D array of the number of times a song was played
   */
  public Song(String title, int[] songPlays) {
    this.title = title;
    this.songPlays = songPlays;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the number of plays for the day at the specified index
   */
  public String getSongInfo(int index) {
    return songPlays[index] + " plays";
  }

  /*
   * Returns the total number of times the song was played
   */
  public int calcTotalPlays() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse songPlays to calculate and return the total number of times the
     * song was played on the streaming music service.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns the average number of times the song was played each day
   */
  public int calcAveragePlays() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average number of times the song was played.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns a String containing the number of plays the song had each day
   */
  public String toString() {
    String result = title + " Daily Plays\\n----------\\n";

    for (int index = 0; index < songPlays.length; index++) {
      result += "Day " + (index + 1) + ": " + songPlays[index] + "\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`SongTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Song.java Test")
public class SongTest {

  Song testSong;
  int[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toIntArray("dailyPlays.txt");
    testSong = new Song("Some Song", testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average number of times the song was played => ")
  public void testCalcAveragePlays() {
    message = "Use the calcTotalPlays() method to get the total number of times the song was played. Then divide";
    message += "\\n        this result by the length of the 1D array songPlays and return the result.";
    message += messageGap;
      
    int expected = getExpectedResult(testData);
    int actual = testSong.calcAveragePlays();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private int getExpectedResult(int[] values) {
    int total = 0;

    for (int num : values) {
      total += num;
    }

    return total / values.length;
  }
  
}`}],dataFiles:[{path:`dailyPlays.txt`,text:`4750185
5102574
6149812
5811600
4506688
4873139
5009612
5562741
5799207
7068846
6882528
5372271
5364677
5532812
5901056
6708201
7723706
6543891
5942353
6314185
6903992
8020508
10905559
21273357
16216593
4510287
2037749
1483957
1224254
1091757
1500302`}]},{name:`Practice: Writing Algorithms with Arrays (c) #2`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toDoubleArray() to read the storeProfits.txt file and
     * store the result in a 1D array. Instantiate a Store object with the 1D array,
     * then call the calcTotalProfits() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageProfits() method and print the result.
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
  
}`},{path:`Store.java`,text:`/*
 * Manages data about store profits
 */
public class Store {

  private String name;              // The name of the store
  private double[] storeProfits;    // The 1D array of the profits a store made each day

  /*
   * Sets name to the specified name of the store and initializes storeProfits
   * to the specified 1D array of the profits a store made each day
   */
  public Store(String name, double[] storeProfits) {
    this.name = name;
    this.storeProfits = storeProfits;
  }

  /*
   * Returns the name of the store
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the profits for the day at the specified index
   */
  public String getProfitsInfo(int index) {
    return "Profits made: " + storeProfits[index];
  }

  /*
   * Returns the total profits made by the store
   */
  public double calcTotalProfits() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse storeProfits to calculate and return the total profits made.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns the average profits made by the store each day 
   */
  public double calcAverageProfits() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average profits made by the store each day.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns a String containing the profits the store made each day
   */
  public String toString() {
    String result = name + "'s Daily Profits\\n----------\\n";

    for (int index = 0; index < storeProfits.length; index++) {
      result += "Day " + (index + 1) + ": " + storeProfits[index] + "\\n";
    }

    return result;
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

  Store testStore;
  double[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toDoubleArray("storeProfits.txt");
    testStore = new Store("Some Store", testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average profits made by the store each day => ")
  public void testCalcAverageProfits() {
    message = "Use the calcTotalProfits() method to get the total profits made by the store. Then divide";
    message += "\\n        this result by the length of the 1D array storeProfts and return the result.";
    message += messageGap;
      
    double expected = getExpectedResult(testData);
    double actual = testStore.calcAverageProfits();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private double getExpectedResult(double[] values) {
    double total = 0;

    for (double num : values) {
      total += num;
    }

    return total / values.length;
  }
  
}`}],dataFiles:[{path:`storeProfits.txt`,text:`-16.28
2121.66
598.28
421.13
-256.02
1572.71
1373.42
1474.12
3427.62
772.90
6438.50
1082.35`}]},{name:`Practice: Writing Algorithms with Arrays (d) #2`,lesson:`Lesson 7: Preconditions and Postconditions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the FileReader.toIntArray() to read the episodes.txt file and
     * store the result in a 1D array. Instantiate a TVEpisodes object with the 1D
     * array, then call the calcTotalLength() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageLength() method and print the result.
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
  
}`},{path:`TVEpisodes.java`,text:`/*
 * Manages data about episodes in a TV series
 */
public class TVEpisodes {

  private String title;              // The title of a TV show
  private String[] episodeTitles;    // The 1D array of episode titles
  private int[] episodeLengths;      // The 1D array of the number of minutes in each episode

  /*
   * Sets title to the specified title of the TV show and initializes episodeTitles
   * to the specified 1D array of the episode titles and episodeLengths to the 1D
   * array of the number of minutes in each episode
   */
  public TVEpisodes(String title, int[] episodeLengths) {
    this.title = title;
    episodeTitles = FileReader.toStringArray("titles.txt");
    this.episodeLengths = episodeLengths;
  }

  /*
   * Returns the name of the TV show
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the episode title and its number of minutes at the specified index
   */
  public String getEpisodeInfo(int index) {
    return episodeTitles[index] + ": " + episodeLengths[index] + " minutes\\n";
  }

  /*
   * Returns the total number of minutes for all episodes
   */
  public int calcTotalLength() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse episodeLengths to calculate and return the total number of
     * minutes for all TV episodes.
     * -----------------------------------------------------------------------------
     */


    return -1;
  }

  /*
   * Returns the average number of minutes for each episode
   */
  public int calcAverageLength() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the average number of minutes for each TV episode.
     * -----------------------------------------------------------------------------
     */



    return -1;
  }

  /*
   * Returns a String containing the name and length of each episode
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < episodeTitles.length; index++) {
      result += episodeTitles[index] + ": " + episodeLengths[index] + " minutes\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`TVEpisodesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TVEpisodes.java Test")
public class TVEpisodesTest {

  TVEpisodes testTVEpisodes;
  int[] testData;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testData = FileReader.toIntArray("episodes.txt");
    testTVEpisodes = new TVEpisodes("Some TV Show", testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average number of minutes of each TV episode => ")
  public void testCalcAverageLength() {
    message = "Use the calcTotalLength() method to get the total minutes for all episodes. Then divide";
    message += "\\n        this result by the length of the 1D array episodeLengths and return the result.";
    message += messageGap;
      
    int expected = getExpectedResult(testData);
    int actual = testTVEpisodes.calcAverageLength();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  private int getExpectedResult(int[] values) {
    int total = 0;

    for (int num : values) {
      total += num;
    }

    return total / values.length;
  }
  
}`}],dataFiles:[{path:`episodes.txt`,text:`37
30
34
38
32
41
37
45
50
38
32
36
43
30
35
44`},{path:`titles.txt`,text:`Chapter 1: The Mandalorian
Chapter 2: The Child
Chapter 3: The Sin
Chapter 4: Sanctuary
Chapter 5: The Gunslinger
Chapter 6: The Prisoner
Chapter 7: The Reckoning
Chapter 8: Redemption
Chapter 9: The Marshal
Chapter 10: The Passenger
Chapter 11: The Heiress
Chapter 12: The Siege
Chapter 13: The Jedi
Chapter 14: The Tragedy
Chapter 15: The Believer
Chapter 16: The Rescue`}]},{name:`Analyzing Data Sets #1`,lesson:`Lesson 8: Ethics in Data`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main{
  public static void main(String[] args) throws IOException{
    
    
    MusicData playsStudy = new MusicData();
    String mostPopularSong = 
            playsStudy.findMostPopularSong(
            "plays.txt","titles.txt");

    System.out.println(mostPopularSong);

    
  }
}`},{path:`MusicData.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class MusicData {
  
    /**
     * Calculates the most popular song in a data set based on total plays
     */
    public String findMostPopularSong(String playsData, String titlesData) throws IOException {
        File playsFile = new File(playsData);
        File titlesFile = new File(titlesData);
      
        Scanner playsScanner = new Scanner(playsFile);
        Scanner titlesScanner = new Scanner(titlesFile);

        int maxPlays = 0;
        String maxSong = "";
      
        while (playsScanner.hasNext()) {
            int currentPlays = playsScanner.nextInt();
            String currentTitle = titlesScanner.nextLine();
            
            if (currentPlays > maxPlays) {
                maxPlays = currentPlays;
                maxSong = currentTitle;       
            }
        }
        playsScanner.close();
        titlesScanner.close();

        return maxSong + " has the most plays with " + maxPlays + " total plays.";
    }

}`}],validationFiles:[],dataFiles:[{path:`artists.txt`,text:`Queen
Led Zeppelin
John Lennon
Nirvana
Eagles
AC/DC
Guns N' Roses
Queen
The Rolling Stones
Van Halen
Led Zeppelin
Pink Floyd
Led Zeppelin
Ozzy Osbourne
AC/DC
Metallica
Led Zeppelin
Guns N' Roses
Deep Purple
Derek and the Dominos
`},{path:`genre.txt`,text:`Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
`},{path:`plays.txt`,text:`245
300
450
200
220
150
170
190
210
180
275
340
415
195
205
160
185
215
225
195
`},{path:`titles.txt`,text:`Bohemian Rhapsody
Stairway to Heaven
Imagine
Smells Like Teen Spirit
Hotel California
Back in Black
Sweet Child o' Mine
Another One Bites the Dust
Paint It Black
Jump
Whole Lotta Love
Comfortably Numb
Black Dog
Crazy Train
Highway to Hell
Enter Sandman
Kashmir
Paradise City
Smoke on the Water
Layla
`}]},{name:`Analyzing Data Sets #2`,lesson:`Lesson 8: Ethics in Data`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main{
  public static void main(String[] args) throws IOException{
    
    
    HealthData cholesterolStudy = new HealthData();
    double averageCholesterol = 
            cholesterolStudy.calculateAverageCholesterolWithHeartDisease(
            "cholesterol.txt","heart_disease.txt");

    System.out.println("Average Cholesterol: " + averageCholesterol);

    
  }
}`},{path:`HealthData.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class HealthData {
  
    /**
     * Calculates the average cholesterol level for people with heart disease
     */
    public double calculateAverageCholesterolWithHeartDisease(String cholesterolData, String heartDiseaseData) throws IOException {
        File cholesterolFile = new File(cholesterolData);
        File heartDiseaseFile = new File(heartDiseaseData);
      
        Scanner cholesterolScanner = new Scanner(cholesterolFile);
        Scanner heartDiseaseScanner = new Scanner(heartDiseaseFile);

        int totalCholesterol = 0;
        int count = 0;

        while (cholesterolScanner.hasNext()) {
            int currentCholesterol = cholesterolScanner.nextInt();
            boolean heartDiseasePresent = heartDiseaseScanner.nextBoolean();
            
            if (heartDiseasePresent) {
                totalCholesterol += currentCholesterol;
                count++;
            }
        }
        cholesterolScanner.close();
        heartDiseaseScanner.close();

        if (count == 0) {
            return 0.0;
        }

        return (double) totalCholesterol / count;
    }

}`}],validationFiles:[],dataFiles:[{path:`age.txt`,text:`46
59
54
50
47
60
46
58
50
50
60
43
47
42
60
41
51
45
41
60`},{path:`cholesterol.txt`,text:`200
257
204
213
280
200
284
170
222
167
281
238
209
163
158
239
202
279
233
241`},{path:`gender.txt`,text:`Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male
Male`},{path:`heart_disease.txt`,text:`TRUE
FALSE
FALSE
FALSE
TRUE
FALSE
TRUE
TRUE
TRUE
TRUE
TRUE
FALSE
FALSE
FALSE
FALSE
FALSE
TRUE
FALSE
FALSE
TRUE`}]},{name:`Analyzing Data Sets #3`,lesson:`Lesson 8: Ethics in Data`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main{
  public static void main(String[] args) throws IOException{
    
    
    TestData scoresStudy = new TestData();
    double averageScore = scoresStudy.findAverageScore("scores.txt");

    System.out.println("Average score: " + averageScore);

    
  }
}`},{path:`TestData.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class TestData {
  
    /**
     * Calculates average test score
     */
    public double findAverageScore(String scoresData) throws IOException {
        File scoresFile = new File(scoresData);
      
        Scanner scoresScanner = new Scanner(scoresFile);

        int total = 0;
        int count = 0;
      
        while (scoresScanner.hasNext()) {
            total += scoresScanner.nextInt();
            count++;
        }
        scoresScanner.close();

        double average = 0.0;
        if(count > 0){
          average = (double) total / count;
        }
        return average;
    }

}`}],validationFiles:[],dataFiles:[{path:`gender.txt`,text:`Female
Male
Male
Female
Female
Male
Male
Female
Male
Female
Female
Male
Female
Female
Male
Male
Female
Male
Female
Male`},{path:`school_type.txt`,text:`Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private
Private`},{path:`scores.txt`,text:`85
92
88
91
87
90
93
89
86
94
88
91
87
90
92
89
86
94
85
93`},{path:`state.txt`,text:`California
New York
Texas
Florida
Nevada
Ohio
Georgia
New Jersey
Washington
Illinois
Colorado
Arizona
Michigan
Massachusetts
Virginia
Oregon
Pennsylvania
Maryland
Indiana
North Carolina`}]},{name:`Analyzing Data Sets #4`,lesson:`Lesson 8: Ethics in Data`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.io.IOException;

public class Main{
  public static void main(String[] args) throws IOException{
    
    
    SalaryData salaryStudy = new SalaryData();
    double averageSalary = salaryStudy.findAverageSalary("annual_salary.txt");

    System.out.println("The average salary is: $" + averageSalary);

    
  }
}`},{path:`SalaryData.java`,text:`import java.io.File;
import java.io.IOException;
import java.util.Scanner;

public class SalaryData {
  
    /**
     * Calculates average salary
     */
    public double findAverageSalary(String salaryData) throws IOException {
        File salaryFile = new File(salaryData);
      
        Scanner salaryScanner = new Scanner(salaryFile);

        int total = 0;
        int count = 0;
      
        while (salaryScanner.hasNext()) {
            total += salaryScanner.nextInt();
            count++;
        }
        salaryScanner.close();

        double average = 0.0;
        if(count > 0){
          average = (double) total / count;
        }
        return average;
    }

}`}],validationFiles:[],dataFiles:[{path:`annual_salary.txt`,text:`120000
95000
135000
110000
125000
98000
140000
102000
115000
108000
130000
99000
145000
112000
123000
107000
128000
97000
139000
104000`},{path:`city.txt`,text:`San Francisco
New York
Los Angeles
Seattle
Chicago
Boston
Washington, D.C.
Austin
San Diego
Dallas
San Francisco
New York
Los Angeles
Seattle
Chicago
Boston
Washington, D.C.
Austin
San Diego
Dallas`},{path:`employment_type.txt`,text:`Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time
Full-Time`},{path:`profession.txt`,text:`Software Engineer
Lawyer
Doctor
Financial Analyst
Architect
Engineer
Consultant
Data Scientist
Manager
Pharmacist
Dentist
Business Executive
Surgeon
Attorney
Marketing Director
Software Developer
Investment Banker
Accountant
Product Manager
Engineer`}]},{name:`Predict and Run: Polymorphism`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Circle.java`,text:`/*
 * Represents a circle that is a type of shape
 */
public class Circle extends Shape {

  /*
   * Sets type and width to the specified values
   */
  public Circle(String type, double width) {
    super(type, width);
  }

  /*
   * Calculates and returns the area of the circle
   */
  public double calculateArea() {
    double radius = getWidth() / 2;
    return 3.14 * (radius * radius);
  }
  
}`},{path:`Rectangle.java`,text:`/*
 * Represents a rectangle that is a type of shape
 */
public class Rectangle extends Shape {

  private double height;     // The height of a rectangle

  /*
   * Sets the type, width, and height to the specified values
   */
  public Rectangle(String type, double width, double height) {
    super(type, width);
    this.height = height;
  }

  /*
   * Returns the height of the rectangle
   */
  public double getHeight() {
    return height;
  }

  /*
   * Sets height to the new height
   */
  public void setHeight(double height) {
    this.height = height;
  }

  /*
   * Calculates and returns the area of the rectangle
   */
  public double calculateArea() {
    return getWidth() * height;
  }
  
}`},{path:`Shape.java`,text:`/*
 * Represents a shape
 */
public class Shape {

  private String type;      // The type of shape
  private double width;     // The width of a shape

  /*
   * Sets type to the specified type
   * and width to the specified width
   */
  public Shape(String type, double width) {
    this.type = type;
    this.width = width;
  }

  /*
   * Returns the type of the shape
   */
  public String getType() {
    return type;
  }

  /*
   * Sets type to the new type
   */
  public void setType(String type) {
    this.type = type;
  }

  /*
   * Returns the width of the shape
   */
  public double getWidth() {
    return width;
  }

  /*
   * Sets width to the new width
   */
  public void setWidth(double width) {
    this.width = width;
  }

  /*
   * Calculates and returns the area of the shape
   */
  public double calculateArea() {
    return width * width;
  }

  /*
   * Returns a String containing the type of shape and its width
   */
  public String toString() {
    return type + "\\nWidth: " + width;
  }
  
}`},{path:`ShapeManager.java`,text:`/*
 * Manages data about shapes
 */
public class ShapeManager {

  private Shape[] allShapes;     // The 1D array of Shape objects

  /*
   * Initializes allShapes to the specified 1D array of Shape objects
   */
  public ShapeManager(Shape[] allShapes) {
    this.allShapes = allShapes;
  }

  /*
   * Sets allShapes to the specified 1D array of new shapes
   */
  public void setAllShapes(Shape[] newShapes) {
    allShapes = newShapes;
  }

  /*
   * Returns a String containing the type of shape and its area for
   * all shapes in the 1D array allShapes
   */
  public String calculateAllAreas() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += "Area of " + theShape.getType() + ": " + theShape.calculateArea() + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing the information about each shape
   */
  public String toString() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += theShape + "\\n\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Polymorphism #1`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Circle.java`,text:`/*
 * Represents a circle that is a type of shape
 */
public class Circle extends Shape {

  /*
   * Sets type and width to the specified values
   */
  public Circle(String type, double width) {
    super(type, width);
  }

  /*
   * Calculates and returns the area of the circle
   */
  public double calculateArea() {
    double radius = getWidth() / 2;
    return 3.14 * (radius * radius);
  }
  
}`},{path:`Rectangle.java`,text:`/*
 * Represents a rectangle that is a type of shape
 */
public class Rectangle extends Shape {

  private double height;     // The height of a rectangle

  /*
   * Sets the type, width, and height to the specified values
   */
  public Rectangle(String type, double width, double height) {
    super(type, width);
    this.height = height;
  }

  /*
   * Returns the height of the rectangle
   */
  public double getHeight() {
    return height;
  }

  /*
   * Sets height to the new height
   */
  public void setHeight(double height) {
    this.height = height;
  }

  /*
   * Calculates and returns the area of the rectangle
   */
  public double calculateArea() {
    return getWidth() * height;
  }
  
}`},{path:`Shape.java`,text:`/*
 * Represents a shape
 */
public class Shape {

  private String type;      // The type of shape
  private double width;     // The width of a shape

  /*
   * Sets type to the specified type
   * and width to the specified width
   */
  public Shape(String type, double width) {
    this.type = type;
    this.width = width;
  }

  /*
   * Returns the type of the shape
   */
  public String getType() {
    return type;
  }

  /*
   * Sets type to the new type
   */
  public void setType(String type) {
    this.type = type;
  }

  /*
   * Returns the width of the shape
   */
  public double getWidth() {
    return width;
  }

  /*
   * Sets width to the new width
   */
  public void setWidth(double width) {
    this.width = width;
  }

  /*
   * Calculates and returns the area of the shape
   */
  public double calculateArea() {
    return width * width;
  }

  /*
   * Returns a String containing the type of shape and its width
   */
  public String toString() {
    return type + "\\nWidth: " + width;
  }
  
}`},{path:`ShapeManager.java`,text:`/*
 * Manages data about shapes
 */
public class ShapeManager {

  private Shape[] allShapes;     // The 1D array of Shape objects

  /*
   * Initializes allShapes to the specified 1D array of Shape objects
   */
  public ShapeManager(Shape[] allShapes) {
    this.allShapes = allShapes;
  }

  /*
   * Sets allShapes to the specified 1D array of new shapes
   */
  public void setAllShapes(Shape[] newShapes) {
    allShapes = newShapes;
  }

  /*
   * Returns a String containing the type of shape and its area for
   * all shapes in the 1D array allShapes
   */
  public String calculateAllAreas() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += "Area of " + theShape.getType() + ": " + theShape.calculateArea() + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing the information about each shape
   */
  public String toString() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += theShape + "\\n\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Polymorphism #2`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Circle.java`,text:`/*
 * Represents a circle that is a type of shape
 */
public class Circle extends Shape {

  /*
   * Sets type and width to the specified values
   */
  public Circle(String type, double width) {
    super(type, width);
  }

  /*
   * Calculates and returns the area of the circle
   */
  public double calculateArea() {
    double radius = getWidth() / 2;
    return 3.14 * (radius * radius);
  }
  
}`},{path:`Rectangle.java`,text:`/*
 * Represents a rectangle that is a type of shape
 */
public class Rectangle extends Shape {

  private double height;     // The height of a rectangle

  /*
   * Sets the type, width, and height to the specified values
   */
  public Rectangle(String type, double width, double height) {
    super(type, width);
    this.height = height;
  }

  /*
   * Returns the height of the rectangle
   */
  public double getHeight() {
    return height;
  }

  /*
   * Sets height to the new height
   */
  public void setHeight(double height) {
    this.height = height;
  }

  /*
   * Calculates and returns the area of the rectangle
   */
  public double calculateArea() {
    return getWidth() * height;
  }
  
}`},{path:`Shape.java`,text:`/*
 * Represents a shape
 */
public class Shape {

  private String type;      // The type of shape
  private double width;     // The width of a shape

  /*
   * Sets type to the specified type
   * and width to the specified width
   */
  public Shape(String type, double width) {
    this.type = type;
    this.width = width;
  }

  /*
   * Returns the type of the shape
   */
  public String getType() {
    return type;
  }

  /*
   * Sets type to the new type
   */
  public void setType(String type) {
    this.type = type;
  }

  /*
   * Returns the width of the shape
   */
  public double getWidth() {
    return width;
  }

  /*
   * Sets width to the new width
   */
  public void setWidth(double width) {
    this.width = width;
  }

  /*
   * Calculates and returns the area of the shape
   */
  public double calculateArea() {
    return width * width;
  }

  /*
   * Returns a String containing the type of shape and its width
   */
  public String toString() {
    return type + "\\nWidth: " + width;
  }
  
}`},{path:`ShapeManager.java`,text:`/*
 * Manages data about shapes
 */
public class ShapeManager {

  private Shape[] allShapes;     // The 1D array of Shape objects

  /*
   * Initializes allShapes to the specified 1D array of Shape objects
   */
  public ShapeManager(Shape[] allShapes) {
    this.allShapes = allShapes;
  }

  /*
   * Sets allShapes to the specified 1D array of new shapes
   */
  public void setAllShapes(Shape[] newShapes) {
    allShapes = newShapes;
  }

  /*
   * Returns a String containing the type of shape and its area for
   * all shapes in the 1D array allShapes
   */
  public String calculateAllAreas() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += "Area of " + theShape.getType() + ": " + theShape.calculateArea() + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing the information about each shape
   */
  public String toString() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += theShape + "\\n\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Polymorphism #3`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Circle.java`,text:`/*
 * Represents a circle that is a type of shape
 */
public class Circle extends Shape {

  /*
   * Sets type and width to the specified values
   */
  public Circle(String type, double width) {
    super(type, width);
  }

  /*
   * Calculates and returns the area of the circle
   */
  public double calculateArea() {
    double radius = getWidth() / 2;
    return 3.14 * (radius * radius);
  }
  
}`},{path:`Rectangle.java`,text:`/*
 * Represents a rectangle that is a type of shape
 */
public class Rectangle extends Shape {

  private double height;     // The height of a rectangle

  /*
   * Sets the type, width, and height to the specified values
   */
  public Rectangle(String type, double width, double height) {
    super(type, width);
    this.height = height;
  }

  /*
   * Returns the height of the rectangle
   */
  public double getHeight() {
    return height;
  }

  /*
   * Sets height to the new height
   */
  public void setHeight(double height) {
    this.height = height;
  }

  /*
   * Calculates and returns the area of the rectangle
   */
  public double calculateArea() {
    return getWidth() * height;
  }
  
}`},{path:`Shape.java`,text:`/*
 * Represents a shape
 */
public class Shape {

  private String type;      // The type of shape
  private double width;     // The width of a shape

  /*
   * Sets type to the specified type
   * and width to the specified width
   */
  public Shape(String type, double width) {
    this.type = type;
    this.width = width;
  }

  /*
   * Returns the type of the shape
   */
  public String getType() {
    return type;
  }

  /*
   * Sets type to the new type
   */
  public void setType(String type) {
    this.type = type;
  }

  /*
   * Returns the width of the shape
   */
  public double getWidth() {
    return width;
  }

  /*
   * Sets width to the new width
   */
  public void setWidth(double width) {
    this.width = width;
  }

  /*
   * Calculates and returns the area of the shape
   */
  public double calculateArea() {
    return width * width;
  }

  /*
   * Returns a String containing the type of shape and its width
   */
  public String toString() {
    return type + "\\nWidth: " + width;
  }
  
}`},{path:`ShapeManager.java`,text:`/*
 * Manages data about shapes
 */
public class ShapeManager {

  private Shape[] allShapes;     // The 1D array of Shape objects

  /*
   * Initializes allShapes to the specified 1D array of Shape objects
   */
  public ShapeManager(Shape[] allShapes) {
    this.allShapes = allShapes;
  }

  /*
   * Sets allShapes to the specified 1D array of new shapes
   */
  public void setAllShapes(Shape[] newShapes) {
    allShapes = newShapes;
  }

  /*
   * Returns a String containing the type of shape and its area for
   * all shapes in the 1D array allShapes
   */
  public String calculateAllAreas() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += "Area of " + theShape.getType() + ": " + theShape.calculateArea() + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing the information about each shape
   */
  public String toString() {
    String result = "";

    for (Shape theShape : allShapes) {
      result += theShape + "\\n\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Arrays of Objects (a)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 1D array of Pet objects. Instantiate a PetStore object with
     * the 1D array, then print the PetStore object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Cat.java`,text:`/*
 * Represents a cat that is a type of pet
 */
public class Cat extends Pet {

  /*
   * Sets type to "Cat"
   */
  public Cat() {
    super("Cat");
  }
  
}`},{path:`Dog.java`,text:`/*
 * Represents a dog that is a type of pet
 */
public class Dog extends Pet {

  /*
   * Sets type to "Dog"
   */
  public Dog() {
    super("Dog");
  }
  
}`},{path:`Pet.java`,text:`/*
 * Represents a pet at a pet store
 */
public class Pet {

  private String type;    // The type of pet

  /*
   * Sets type to "Pet"
   */
  public Pet() {
    this("Pet");
  }

  /*
   * Sets type to the specified type
   */
  public Pet(String type) {
    this.type = type;
  }

  /*
   * Returns the type of pet
   */
  public String getType() {
    return type;
  }

}`},{path:`PetStore.java`,text:`/*
 * Represents a pet store
 */
public class PetStore {

  private Pet[] availablePets;    // The 1D array of Pet objects

  /*
   * Initializes availablePets to the 1D array of Pet objects
   */
  public PetStore(Pet[] availablePets) {
    this.availablePets = availablePets;
  }

  /*
   * Returns a String containing the type of each pet in availablePets
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array availablePets and use the getType() method in the
     * Pet class to concatenate the type of each pet to result on separate lines.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`PetStoreTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PetStore.java Test")
public class PetStoreTest {

  Pet[] testPets;
  PetStore testPetStore;
  List<String> output;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPets = new Pet[]{ new Pet(), new Cat(), new Dog() };
    testPetStore = new PetStore(testPets);

    output = SystemOutTestRunner.run();
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the type of each Pet object in availablePets => ")
  public void testToString() {
    message = "Traverse the 1D array availablePets and concatenate each pet type to result." + messageGap;
      
    String expected = getExpectedString();
    String actual = testPetStore.toString();

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the PetStore object => ")
  public void testPrintPetStoreObject() {
    message = "Declare and initialize a 1D array to store a Pet, a Cat, and a Dog object.";
    message += "\\n        Instantiate a PetStore object with the 1D array, then print the PetStore object.";
    message += messageGap;

    String expected = getExpectedString();
    String actual = getActualOutput();
    
    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected string
   */
  private String getExpectedString() {
    String result = "";

    for (Pet current : testPets) {
      result += current.getType() + "\\n";
    }

    return result;
  }

  /*
   * Helper to get output
   */
  public String getActualOutput() {
    String actual = "";
    
    if (output.size() > 0) {
      actual = output.get(0);
    }

    return actual;
  }
  
}`}],dataFiles:[]},{name:`Practice: Arrays of Objects (b)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 1D array of Family objects. Instantiate a Household object with
     * the 1D array, then print the Household object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Brother.java`,text:`/*
 * Represents a brother that is a type of family member
 */
public class Brother extends Family {

  /*
   * Sets title to "Brother"
   */
  public Brother() {
    super("Brother");
  }
  
}`},{path:`Family.java`,text:`/*
 * Represents a family member
 */
public class Family {

  private String title;    // The title of a family member

  /*
   * Sets title to "Relative"
   */
  public Family() {
    this("Relative");
  }

  /*
   * Sets title to the specified title
   */
  public Family(String title) {
    this.title = title;
  }

  /*
   * Returns the title of the family member
   */
  public String getTitle() {
    return title;
  }
  
}`},{path:`Father.java`,text:`/*
 * Represents a father that is a type of family member
 */
public class Father extends Family {

  /*
   * Sets title to "Father"
   */
  public Father() {
    super("Father");
  }
  
}`},{path:`Household.java`,text:`/*
 * Represents a family household
 */
public class Household {

  private Family[] familyMembers;    // The 1D array of Family objects

  /*
   * Initializes familyMembers to the 1D array of Family objects
   */
  public Household(Family[] familyMembers) {
    this.familyMembers = familyMembers;
  }

  /*
   * Returns a String containing each type of family member
   */
  public String toString() {
    String result = "";

    /* -------------------------------------- TO DO -------------------------------------
     * ✅ Traverse the 1D array familyMembers and use the getTitle() method in the Family
     * class to concatenate the title of each family member to result on separate lines.
     * -----------------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`},{path:`Mother.java`,text:`/*
 * Represents a mother that is a type of family member
 */
public class Mother extends Family {

  /*
   * Sets title to "Mother"
   */
  public Mother() {
    super("Mother");
  }
  
}`},{path:`Sister.java`,text:`/*
 * Represents a sister that is a type of family member
 */
public class Sister extends Family {

  /*
   * Sets title to "Sister"
   */
  public Sister() {
    super("Sister");
  }
  
}`}],validationFiles:[{path:`HouseholdTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Household.java Test")
public class HouseholdTest {

  Family[] testFamily;
  Household testHousehold;
  List<String> output;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testFamily = new Family[]{ new Mother(), new Father(), new Brother(), new Sister() };
    testHousehold = new Household(testFamily);

    output = SystemOutTestRunner.run();
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the title of each Family object in familyMembers => ")
  public void testToString() {
    message = "Traverse the 1D array familyMembers and concatenate each family member title to result." + messageGap;
      
    String expected = getExpectedString();
    String actual = testHousehold.toString();

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the Household object => ")
  public void testPrintHouseholdObject() {
    message = "Declare and initialize a 1D array to store a Mother, a Father, a Brother, and a Sister object.";
    message += "\\n        Instantiate a Household object with the 1D array, then print the Household object.";
    message += messageGap;

    String expected = getExpectedString();
    String actual = getActualOutput();
    
    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected string
   */
  private String getExpectedString() {
    String result = "";

    for (Family current : testFamily) {
      result += current.getTitle() + "\\n";
    }

    return result;
  }

  /*
   * Helper to get output
   */
  public String getActualOutput() {
    String actual = "";
    
    if (output.size() > 0) {
      actual = output.get(0);
    }

    return actual;
  }
  
}`}],dataFiles:[]},{name:`Practice: Arrays of Objects (c)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 1D array of Planet objects. Instantiate a Universe object with
     * the 1D array, then print the Universe object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Neptunian.java`,text:`/*
 * Represents a Neptune-like planet
 */
public class Neptunian extends Planet {

  /*
   * Sets name to the specified name and type to "Neptunian"
   */
  public Neptunian(String name) {
    super(name, "Neptunian");
  }
  
}`},{path:`Planet.java`,text:`/*
 * Represents a planet
 */
public class Planet {

  private String name;    // The name of a planet
  private String type;    // The type of a planet

  /*
   * Sets name to "Unknown" and type to "Unknown"
   */
  public Planet() {
    this("Unknown", "Unknown");
  }

  /*
   * Sets name and type to the specified values
   */
  public Planet(String name, String type) {
    this.name = name;
    this.type = type;
  }

  /*
   * Returns the name of the planet
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the type of the planet
   */
  public String getType() {
    return type;
  }

  /*
   * Returns a String containing the name and type of the planet
   */
  public String toString() {
    return name + ": " + type;
  }
}`},{path:`Terrestrial.java`,text:`/*
 * Represents a terrestrial planet
 */
public class Terrestrial extends Planet {

  /*
   * Sets name to the specified name and type to "Terrestrial"
   */
  public Terrestrial(String name) {
    super(name, "Terrestrial");
  }
  
}`},{path:`Universe.java`,text:`/*
 * Represents the universe
 */
public class Universe {

  private Planet[] worlds;    // The 1D array of Planet objects

  /*
   * Initializes worlds to the 1D array of Planet objects
   */
  public Universe(Planet[] worlds) {
    this.worlds = worlds;
  }

  /*
   * Returns a String containing each planet's name and type
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array worlds and concatenate each Planet object to
     * result on separate lines.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`UniverseTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Universe.java Test")
public class UniverseTest {

  Planet[] testPlanets;
  Universe testUniverse;
  List<String> output;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPlanets = new Planet[]{ new Planet(), new Neptunian("HAT-P-26b"), new Terrestrial("TRAPPIST-1 e") };
    testUniverse = new Universe(testPlanets);

    output = SystemOutTestRunner.run();
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the name and type of each Planet object in worlds => ")
  public void testToString() {
    message = "Traverse the 1D array worlds and concatenate each planet name and type to result." + messageGap;
      
    String expected = getExpectedString();
    String actual = testUniverse.toString();

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the Universe object => ")
  public void testPrintUniverseObject() {
    message = "Declare and initialize a 1D array to store a Planet, a Neptunian, and a Terrestrial object.";
    message += "\\n        Instantiate a Universe object with the 1D array, then print the Universe object.";
    message += messageGap;

    String expected = getExpectedString();
    String actual = getActualOutput();
    
    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected string
   */
  private String getExpectedString() {
    String result = "";

    for (Planet current : testPlanets) {
      result += current + "\\n";
    }

    return result;
  }

  /*
   * Helper to get output
   */
  public String getActualOutput() {
    String actual = "";
    
    if (output.size() > 0) {
      actual = output.get(0);
    }

    return actual;
  }
  
}`}],dataFiles:[]},{name:`Practice: Arrays of Objects (d)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 1D array of MenuItem objects. Instantiate a Restaurant object
     * with the 1D array, then print the Restaurant object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Burger.java`,text:`/*
 * Represents a burger that is a type of menu item
 */
public class Burger extends MenuItem {

  /*
   * Sets type to "Burger" and price to the specified price
   */
  public Burger(double price) {
    super("Burger", price);
  }
  
}`},{path:`MenuItem.java`,text:`/*
 * Represents a menu item
 */
public class MenuItem {

  private String type;    // The type of a menu item
  private double price;   // The price of a menu item

  /*
   * Sets type to "Unknown" and price to 1.50
   */
  public MenuItem() {
    this("Unknown", 1.50);
  }
  
  /*
   * Sets type to the specified type and
   * price to the specified price
   */
  public MenuItem(String type, double price) {
    this.type = type;
    this.price = price;
  }

  /*
   * Returns the type of the menu item
   */
  public String getType() {
    return type;
  }

  /*
   * Returns the price of the menu item
   */
  public double getPrice() {
    return price;
  }

  /*
   * Returns a String containing the type and price of the menu item
   */
  public String toString() {
    return type + ": " + price;
  }
  
}`},{path:`Pasta.java`,text:`/*
 * Represents a pasta that is a type of menu item
 */
public class Pasta extends MenuItem {

  /*
   * Sets type to "Pasta" and price to the specified price
   */
  public Pasta(double price) {
    super("Pasta", price);
  }
  
}`},{path:`Pizza.java`,text:`/*
 * Represents a pizza that is a type of menu item
 */
public class Pizza extends MenuItem {

  /*
   * Sets type to "Pizza" and price to the specified price
   */
  public Pizza(double price) {
    super("Pizza", price);
  }
  
}`},{path:`Restaurant.java`,text:`/*
 * Represents a restaurant
 */
public class Restaurant {

  private MenuItem[] menu;    // The 1D array of MenuItem objects

  /*
   * Initializes menu to the 1D array of MenuItem objects
   */
  public Restaurant(MenuItem[] menu) {
    this.menu = menu;
  }

  /*
   * Returns a String containing the type and price of each menu item
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array menu and concatenate each MenuItem object to
     * result on separate lines.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`}],validationFiles:[{path:`RestaurantTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Restaurant.java Test")
public class RestaurantTest {

  MenuItem[] testMenuItems;
  Restaurant testRestaurant;
  List<String> output;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testMenuItems = new MenuItem[]{ new Burger(6.75), new Pizza(12.99), new Pasta(24.50) };
    testRestaurant = new Restaurant(testMenuItems);

    output = SystemOutTestRunner.run();
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the type and price of each MenuItem object in availableItems => ")
  public void testToString() {
    message = "Traverse the 1D array availableItems and concatenate each MenuItem type and price to result." + messageGap;
      
    String expected = getExpectedString();
    String actual = testRestaurant.toString();

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Print the Restaurant object => ")
  public void testPrintRestaurantObject() {
    message = "Declare and initialize a 1D array to store a Burger, a Pizza, and a Pasta object.";
    message += "\\n        Instantiate a Restaurant object with the 1D array, then print the Restaurant object.";
    message += messageGap;

    String expected = getExpectedString();
    String actual = getActualOutput();
    
    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected string
   */
  private String getExpectedString() {
    String result = "";

    for (MenuItem current : testMenuItems) {
      result += current + "\\n";
    }

    return result;
  }

  /*
   * Helper to get output
   */
  public String getActualOutput() {
    String actual = "";
    
    if (output.size() > 0) {
      actual = output.get(0);
    }

    return actual;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing Arrays of Objects (a)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Employee[] businessEmployees = { new Employee("Ellis", 40000),
                                     new Manager("Kristina", 45000, 2000),
                                     new Employee("Daphne", 50000),
                                     new Manager("David", 50000, 5000) };

    Business techCompany = new Business(businessEmployees);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the Business object.
     * -----------------------------------------------------------------------------
     */

    

    
  }
}`},{path:`Business.java`,text:`/*
 * Represents a business
 */
public class Business {

  private Employee[] employees;    // The 1D array of Employee objects

  /*
   * Initializes employees to the 1D array of Employee objects
   */
  public Business(Employee[] employees) {
    this.employees = employees;
  }

  /*
   * Returns a String containing the information about each employee
   */
  public String toString() {
    String result = "";
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array employees and concatenate each employee's name
     * and salary to result on separate lines.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`},{path:`Employee.java`,text:`/*
 * Represents an employee
 */
public class Employee {

  private String name;         // The name of a employee
  private double baseSalary;   // The base salary of an employee

  /*
   * Sets name to the specified name and baseSalary to the specified salary
   */
  public Employee(String name, double baseSalary) {
    this.name = name;
    this.baseSalary = baseSalary;
  }

  /*
   * Returns the name of the employee
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the employee's base salary
   */
  public double getSalary() {
    return baseSalary;
  }
  
}`},{path:`Manager.java`,text:`/*
 * Represents a manager that is a type of employee
 */
public class Manager extends Employee {

  private double bonus;  // The bonus a manager receives

  /*
   * Sets name, salary, and bonus to the specified values
   */
  public Manager(String name, double salary, double bonus) {
    super(name, salary);
    this.bonus = bonus;
  }

  /*
   * Returns the name of the manager
   */
  public String getName() {
    return "*" + super.getName();
  }

  /*
   * Returns the manager's salary with bonus
   */
  public double getSalary() {
    return super.getSalary() + bonus;
  }
  
}`}],validationFiles:[{path:`BusinessTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Business.java Test")
public class BusinessTest {

  Employee[] testEmployees;
  Business testBusiness;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testEmployees = new Employee[]{ new Employee("Ellis", 40000), new Manager("Kristina", 45000, 2000),
                                    new Employee("Daphne", 50000), new Manager("David", 50000, 5000) };
    
    testBusiness = new Business(testEmployees);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing each employee's name and salary => ")
  public void testToString() {
    message = "Traverse the 1D array employees and concatenate each";
    message += "\\n        employee's name and salary to result";
    message += messageGap;
      
    String expected = getExpectedString();
    String actual = testBusiness.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  public String getExpectedString() {
    String result = "";

    for (Employee current : testEmployees) {
      result += current.getName() + " - $" + current.getSalary() + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing Arrays of Objects (b)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    BankAccount[] accounts = { new BankAccount("Shakila", 500),
                               new BasicAccount("Ali", 200),
                               new BankAccount("Valentina", 100),
                               new BasicAccount("Maria", 800) };

    Bank localBank = new Bank(accounts);

    /* ----------------------------------- TO DO #2 --------------------------------
     * ✅ Call the makeWithdrawals() method and print the Bank object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Bank.java`,text:`/*
 * Represents a bank
 */
public class Bank {

  private BankAccount[] accounts;    // The 1D array of BankAccount objects

  /*
   * Initializes accounts to the 1D array of BankAccount objects
   */
  public Bank(BankAccount[] accounts) {
    this.accounts = accounts;
  }

  /*
   * Returns the 1D array accounts
   */
  public BankAccount[] getAccounts() {
    return accounts;
  }

  /*
   * Withdraws the specified amount from each BankAccount in accounts
   */
  public void makeWithdrawals(double amount) {
    /* ----------------------------------- TO DO #1---------------------------------
     * ✅ Traverse the 1D array accounts and call the withdraw() method on each
     * BankAccount object to withdraw the specified amount.
     * -----------------------------------------------------------------------------
     */



    
  }

  /*
   * Returns a String containing information about each bank account
   */
  public String toString() {
    String result = "";

    for (BankAccount acct : accounts) {
      result += acct + "\\n";
    }

    return result;
  }
  
}`},{path:`BankAccount.java`,text:`/*
 * Represents a bank account
 */
public class BankAccount {

  private String customerName;  // The name of an account holder
  private double balance;       // The starting balance of an account

  /*
   * Sets customerName and balance to the specified values
   */
  public BankAccount(String customerName, double balance) {
    this.customerName = customerName;
    this.balance = balance;
  }

  /*
   * Adds the specified amount to the balance
   */
  public void deposit(double amount) {
    balance += amount;
  }

  /*
   * Subtracts the specified amount from the balance
   */
  public void withdraw(double amount) {
    balance -= amount;
  }

  /*
   * Returns the customer's name
   */
  public String getName() {
    return customerName;
  }

  /*
   * Returns the current balance
   */
  public double getBalance() {
    return balance;
  }

  /*
   * Returns a String containing the customer's name and balance
   */
  public String toString() {
    return customerName + ": $" + balance;
  }
  
}`},{path:`BasicAccount.java`,text:`/*
 * Represents a basic bank account
 */
public class BasicAccount extends BankAccount {

  /*
   * Sets customerName and balance to the specified values
   */
  public BasicAccount(String customerName, double balance) {
    super(customerName, balance);
  }

  /*
   * Subtracts the specified amount from the balance and
   * charges a fee if the withdrawal overdraws the account
   */
  public void withdraw(double amount) {
    super.withdraw(amount);

    if (getBalance() < 0) {
      super.withdraw(30);
    }
  }
  
}`}],validationFiles:[{path:`BankTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Bank.java Test")
public class BankTest {

  private Bank testBank;

  @BeforeEach
  public void setup() {
    // Initialize BankAccount and BasicAccount objects
    BankAccount[] accounts = new BankAccount[] {
      new BankAccount("Shakila", 500.0),
      new BasicAccount("Ali", 200.0),
      new BasicAccount("Valentina", 100.0),
      new BankAccount("Maria", 800.0)
    };
    
    // Create a new Bank object with the accounts
    testBank = new Bank(accounts);
  }

  @Test
  @Order(1)
  @DisplayName("makeWithdrawals() method withdraws the parameter amount from each account balance => ")
  public void testMakeWithdrawals() {
    // Call makeWithdrawals with the value 300
    testBank.makeWithdrawals(300.0);

    // Retrieve the updated account balances
    BankAccount[] accounts = testBank.getAccounts();
    
    // Check each account balance
    assertEquals(200.0, accounts[0].getBalance(), "Shakila's balance should be $200.0 after withdrawal.");
    assertEquals(-130.0, accounts[1].getBalance(), "Ali's balance should be $-130.0 after withdrawal and overdraft fee.");
    assertEquals(-230.0, accounts[2].getBalance(), "Valentina's balance should be $-230.0 after withdrawal and overdraft fee.");
    assertEquals(500.0, accounts[3].getBalance(), "Maria's balance should be $500.0 after withdrawal.");
  }
}
`}],dataFiles:[]},{name:`Practice: Traversing Arrays of Objects (c)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Dessert[] availableDesserts = { new Dessert("vanilla", 2.99, 4),
                                    new Cookie("oatmeal", 1.50, 12, 6),
                                    new Pie("cherry", 4.75, 4, "cherry") };

    FoodTruck dessertTruck = new FoodTruck(availableDesserts);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the FoodTruck object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Cookie.java`,text:`/*
 * Represents a cookie that is sold at a food truck
 */
public class Cookie extends Dessert {

  private int diameter;     // The diameter of a cookie

  /*
   * Sets diameter to 3
   */
  public Cookie() {
    diameter = 3;
  }

  /*
   * Sets flavor, price, quantity, and diameter to the specified values
   */
  public Cookie(String flavor, double price, int quantity, int diameter) {
    super(flavor, price, quantity);
    this.diameter = diameter;
  }

  /*
   * Gets the total price of the cookie order
   */
  public double getTotalPrice() {
    double total = super.getTotalPrice();

    if (getQuantity() > 6) {
      total *= 0.75;
    }

    return total;
  }

  /*
   * Returns a String containing the flavor, price, and diameter of the cookie
   */
  public String toString() {
    return getFlavor() + " (" + diameter + "): " + getPrice();
  }
  
}`},{path:`Dessert.java`,text:`/*
 * Represents a dessert that a food truck sells
 */
public class Dessert {

  private String flavor;  // The flavor of the dessert
  private double price;   // The price of the dessert
  private int quantity;   // The number of desserts

  /*
   * Sets flavor to "plain", price to 0.99, and quantity to 1
   */
  public Dessert() {
    flavor = "plain";
    price = 0.99;
    quantity = 1;
  }

  /*
   * Sets flavor, price, and quantity to the specified values
   */
  public Dessert(String flavor, double price, int quantity) {
    this.flavor = flavor;
    this.price = price;
    this.quantity = quantity;
  }

  /*
   * Returns the flavor of the dessert
   */
  public String getFlavor() {
    return flavor;
  }

  /*
   * Returns the quantity
   */
  public int getQuantity() {
    return quantity;
  }

  /*
   * Returns the price of the dessert
   */
  public double getPrice() {
    return price;
  }

  /*
   * Gets the total price of the dessert order
   */
  public double getTotalPrice() {
    return price * quantity;
  }

  /*
   * Returns a String containing the flavor and price of the dessert
   */
  public String toString() {
    return flavor + ": " + price;
  }
  
}`},{path:`FoodTruck.java`,text:`/*
 * Represents a food truck
 */
public class FoodTruck {

  private Dessert[] availableDesserts;    // The 1D array of Dessert objects

  /*
   * Initializes availableDesserts to the specified 1D array of Dessert objects
   */
  public FoodTruck(Dessert[] availableDesserts) {
    this.availableDesserts = availableDesserts;
  }

  /*
   * Returns a String containing each dessert flavor and total price of the order
   */
  public String toString() {
    String result = "";
    
    /* ------------------------------------------- TO DO -------------------------------------------
     * ✅ Traverse the 1D array availableDesserts and use the getFlavor() and getTotalPrice() methods
     * to concatenate the flavor and price of each Dessert object to result on separate lines.
     * ---------------------------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`},{path:`Pie.java`,text:`/*
 * Represents a pie that is sold at a food truck
 */
public class Pie extends Dessert {

  private String filling;    // The filling flavor of a pie

  /*
   * Sets filling to "none"
   */
  public Pie() {
    filling = "none";
  }

  /*
   * Sets flavor, price, quantity, and filling to the specified values
   */
  public Pie(String flavor, double price, int quantity, String filling) {
    super(flavor, price, quantity);
    this.filling = filling;
  }

  /*
   * Gets the total price of the pie order
   */
  public double getTotalPrice() {
    double total = super.getTotalPrice();

    if (getQuantity() > 2) {
      total *= 0.8;
    }

    return total;
  }

  /*
   * Returns a String containing the flavor, filling, and price of the pie
   */
  public String toString() {
    return getFlavor() + " with " + filling + " filling: " + getPrice();
  }

}`}],validationFiles:[{path:`FoodTruckTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("FoodTruck.java Test")
public class FoodTruckTest {

  Dessert[] testDesserts;
  FoodTruck testFoodTruck;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testDesserts = new Dessert[]{ new Dessert("vanilla", 2.99, 4), new Cookie("oatmeal", 1.50, 12, 6),
                                  new Pie("cherry", 4.75, 4, "cherry") };
    
    testFoodTruck = new FoodTruck(testDesserts);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the flavor and total price for each dessert order => ")
  public void testToString() {
    message = "Traverse the 1D array availableDesserts and concatenate each";
    message += "\\n        flavor and total price to result";
    message += messageGap;
      
    String expected = getExpectedString();
    String actual = testFoodTruck.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  public String getExpectedString() {
    String result = "";

    for (Dessert current : testDesserts) {
      result += current.getFlavor() + " Total Price: $" + current.getTotalPrice() + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing Arrays of Objects (d)`,lesson:`Lesson 9: Polymorphism`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Ticket[] eventTickets = { new Ticket(1), new Walkup(2), new Advance(3, 11), new Student(4, 7) };

    Event schoolEvent = new Event(eventTickets);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the Event object.
     * -----------------------------------------------------------------------------
     */





    
    
  }
}`},{path:`Advance.java`,text:`/*
 * Represents a ticket for an event that
 * is purchased in advance
 */
public class Advance extends Ticket {

  private int numDays;  // The number of days in advance the ticket is being purchased

  /*
   * Sets ticketNumber and numDays to the specified values
   */
  public Advance(int ticketNumber, int numDays) {
    super(ticketNumber);
    this.numDays = numDays;
  }

  /*
   * Gets the price of the ticket based on the
   * number of days in advance it is being purchased
   */
  public double getTicketPrice() {
    if (numDays >= 10) {
      return 30;
    }
    else {
      return 40;
    }
  }
  
}`},{path:`Event.java`,text:`/*
 * Represents an event
 */
public class Event {

  private Ticket[] eventTickets;     // The 1D array of Ticket objects

  /*
   * Initializes eventTickets to the 1D array of Ticket objects
   */
  public Event(Ticket[] eventTickets) {
    this.eventTickets = eventTickets;
  }

  /*
   * Returns a String containing each ticket's information and price
   */
  public String toString() {
    String result = "";
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array eventTickets and concatenate each ticket's
     * information and price to result.
     * -----------------------------------------------------------------------------
     */

    

    return result;
  }
  
}`},{path:`Student.java`,text:`/*
 * Represents a student ticket for an event that
 * is purchased in advance
 */
public class Student extends Advance {

  /*
   * Sets ticketNumber and numDays to the specified values
   */
  public Student(int ticketNumber, int numDays) {
    super(ticketNumber, numDays);
  }

  /*
   * Gets the price of the ticket based on the number of days
   * in advance it is being purchased and discounts it in half
   */
  public double getTicketPrice() {
    return super.getTicketPrice() / 2;
  }

  /*
   * Returns a String with the ticket number and reminder
   * to show their student ID
   */
  public String toString() {
    return super.toString() + " (must show their student ID)";
  }
  
}`},{path:`Ticket.java`,text:`/*
 * Represents a ticket for an event
 */
public class Ticket {

  private int ticketNumber;   // The ticket number

  /*
   * Sets ticketNumber to the specified ticket number
   */
  public Ticket(int ticketNumber) {
    this.ticketNumber = ticketNumber;
  }

  /*
   * Returns the price of the ticket
   */
  public double getTicketPrice() {
    return 0.00;
  }

  /*
   * Returns a String with the ticket number
   */
  public String toString() {
    return "Ticket No " + ticketNumber;
  }
  
}`},{path:`Walkup.java`,text:`/*
 * Represents a ticket for an event
 * that is purchased on the day of the event
 */
public class Walkup extends Ticket {

  /*
   * Sets ticketNumber to the specified ticket number
   */
  public Walkup(int ticketNumber) {
    super(ticketNumber);
  }

  /*
   * Returns the price of the ticket
   */
  public double getTicketPrice() {
    return 50.00;
  }
  
}`}],validationFiles:[{path:`EventTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Event.java Test")
public class EventTest {

  Ticket[] testTickets;
  Event testEvent;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testTickets = new Ticket[]{ new Ticket(1), new Walkup(2), new Advance(3, 11), new Student(4, 7) };
    testEvent = new Event(testTickets);
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing each ticket's information and price => ")
  public void testToString() {
    message = "Traverse the 1D array eventTickets and concatenate each";
    message += "\\n        ticket's information and price to result.";
    message += messageGap;
      
    String expected = getExpectedString();
    String actual = testEvent.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get expected result
   */
  public String getExpectedString() {
    String result = "";

    for (Ticket current : testTickets) {
      result += current + " Price: $" + current.getTicketPrice() + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Enhanced For Loops`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Forbes topCompanies = new Forbes("companies.txt", "profits.txt");
    
    System.out.println(topCompanies);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
  }
}`},{path:`Company.java`,text:`/*
 * Represents a company
 */
public class Company {

  private String name;      // The name of a company
  private double profits;   // The annual profits in billions made by a company

  /*
   * Sets name and profits to the specified values
   */
  public Company(String name, double profits) {
    this.name = name;
    this.profits = profits;
  }

  /*
   * Returns the name of the company
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the profits made by the company
   */
  public double getProfits() {
    return profits;
  }

  /*
   * Returns a String containing the name of
   * the company and its annual profits
   */
  public String toString() {
    return name + " - $" + profits + " billion annual profit";
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
  
}`},{path:`Forbes.java`,text:`/*
 * Manages data about Forbes top companies
 */
public class Forbes {

  private String[] organizations;   // The 1D array containing the company names
  private double[] annualProfits;   // The 1D array containing the annual profits of each company
  private Company[] companies;      // The 1D array of Company objects

  /*
   * 
   */
  public Forbes(String companiesFile, String profitsFile) {
    organizations = FileReader.toStringArray(companiesFile);
    annualProfits = FileReader.toDoubleArray(profitsFile);
    companies = createCompanies();
  }

  /*
   * 
   */
  public Company[] createCompanies() {
    Company[] tempCompanies = new Company[organizations.length];

    for (int index = 0; index < tempCompanies.length; index++) {
      tempCompanies[index] = new Company(organizations[index], annualProfits[index]);
    }

    return tempCompanies;
  }

  /*
   *
   */
  public String getAllProfits() {
    String result = "";

    for (int index = 0; index < annualProfits.length; index++) {
      result += annualProfits[index] + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing each company's information
   */
  public String toString() {
    String result = "Top Companies and Profits\\n----------\\n";

    for (Company org : companies) {
      result += org + "\\n";
    }

    return result;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





  
  
}`}],validationFiles:[],dataFiles:[{path:`companies.txt`,text:`Berkshire Hathaway
JPMorgan Chase
Amazon
Apple
Bank of America
Alphabet
Microsoft
ExxonMobil
Wells Fargo
Verizon Communications
AT&T
UnitedHealth Group
Walmart
Chevron
Citigroup
Comcast
Meta Platforms
Morgan Stanley
Goldman Sachs Group
Johnson & Johnson
CVS Health
Pfizer
Intel
Procter & Gamble
AbbVie
General Motors
MetLife
American Express
Cigna
Prudential Financial
PepsiCo
Merck & Co.
Elevance Health
American International Group
Cisco Systems
Walt Disney
IBM
United Parcel Service
Raytheon Technologies
Capital One
Bristol Myers Squibb
Coca-Cola
Oracle
Thermo Fisher Scientific
Costco Wholesale
Charter Communications
ConocoPhillips
US Bancorp
Abbott Laboratories
Caterpillar`},{path:`profits.txt`,text:`89.8
42.12
33.36
100.56
31
76.03
71.18
23.04
20.58
21.52
17.33
17.45
13.67
15.62
18.23
14.16
39.37
14.58
18.74
19.83
7.91
21.98
19.87
14.6
11.46
10.02
6.55
7.86
5.36
7.61
7.62
13.05
6.24
9.39
11.82
3.08
5.52
12.89
3.86
12.29
6.99
9.77
7.56
7.72
5.51
4.65
8.08
7.21
7.7
6.49`}]},{name:`Investigate and Modify: Enhanced For Loops #1`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Company.java`,text:`/*
 * Represents a company
 */
public class Company {

  private String name;      // The name of a company
  private double profits;   // The annual profits in billions made by a company

  /*
   * Sets name and profits to the specified values
   */
  public Company(String name, double profits) {
    this.name = name;
    this.profits = profits;
  }

  /*
   * Returns the name of the company
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the profits made by the company
   */
  public double getProfits() {
    return profits;
  }

  /*
   * Returns a String containing the name of
   * the company and its annual profits
   */
  public String toString() {
    return name + " - $" + profits + " billion annual profit";
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
  
}`},{path:`Forbes.java`,text:`/*
 * Manages data about Forbes top companies
 */
public class Forbes {

  private String[] organizations;   // The 1D array containing the company names
  private double[] annualProfits;   // The 1D array containing the annual profits of each company
  private Company[] companies;      // The 1D array of Company objects

  /*
   * 
   */
  public Forbes(String companiesFile, String profitsFile) {
    organizations = FileReader.toStringArray(companiesFile);
    annualProfits = FileReader.toDoubleArray(profitsFile);
    companies = createCompanies();
  }

  /*
   * 
   */
  public Company[] createCompanies() {
    Company[] tempCompanies = new Company[organizations.length];

    for (int index = 0; index < tempCompanies.length; index++) {
      tempCompanies[index] = new Company(organizations[index], annualProfits[index]);
    }

    return tempCompanies;
  }

  /*
   *
   */
  public String getAllProfits() {
    String result = "";

    for (int index = 0; index < annualProfits.length; index++) {
      result += annualProfits[index] + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing each company's information
   */
  public String toString() {
    String result = "Top Companies and Profits\\n----------\\n";

    for (Company org : companies) {
      result += org + "\\n";
    }

    return result;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





  
  
}`}],validationFiles:[],dataFiles:[{path:`companies.txt`,text:`Berkshire Hathaway
JPMorgan Chase
Amazon
Apple
Bank of America
Alphabet
Microsoft
ExxonMobil
Wells Fargo
Verizon Communications
AT&T
UnitedHealth Group
Walmart
Chevron
Citigroup
Comcast
Meta Platforms
Morgan Stanley
Goldman Sachs Group
Johnson & Johnson
CVS Health
Pfizer
Intel
Procter & Gamble
AbbVie
General Motors
MetLife
American Express
Cigna
Prudential Financial
PepsiCo
Merck & Co.
Elevance Health
American International Group
Cisco Systems
Walt Disney
IBM
United Parcel Service
Raytheon Technologies
Capital One
Bristol Myers Squibb
Coca-Cola
Oracle
Thermo Fisher Scientific
Costco Wholesale
Charter Communications
ConocoPhillips
US Bancorp
Abbott Laboratories
Caterpillar`},{path:`profits.txt`,text:`89.8
42.12
33.36
100.56
31
76.03
71.18
23.04
20.58
21.52
17.33
17.45
13.67
15.62
18.23
14.16
39.37
14.58
18.74
19.83
7.91
21.98
19.87
14.6
11.46
10.02
6.55
7.86
5.36
7.61
7.62
13.05
6.24
9.39
11.82
3.08
5.52
12.89
3.86
12.29
6.99
9.77
7.56
7.72
5.51
4.65
8.08
7.21
7.7
6.49`}]},{name:`Investigate and Modify: Enhanced For Loops #2`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Company.java`,text:`/*
 * Represents a company
 */
public class Company {

  private String name;      // The name of a company
  private double profits;   // The annual profits in billions made by a company

  /*
   * Sets name and profits to the specified values
   */
  public Company(String name, double profits) {
    this.name = name;
    this.profits = profits;
  }

  /*
   * Returns the name of the company
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the profits made by the company
   */
  public double getProfits() {
    return profits;
  }

  /*
   * Returns a String containing the name of
   * the company and its annual profits
   */
  public String toString() {
    return name + " - $" + profits + " billion annual profit";
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
  
}`},{path:`Forbes.java`,text:`/*
 * Manages data about Forbes top companies
 */
public class Forbes {

  private String[] organizations;   // The 1D array containing the company names
  private double[] annualProfits;   // The 1D array containing the annual profits of each company
  private Company[] companies;      // The 1D array of Company objects

  /*
   * 
   */
  public Forbes(String companiesFile, String profitsFile) {
    organizations = FileReader.toStringArray(companiesFile);
    annualProfits = FileReader.toDoubleArray(profitsFile);
    companies = createCompanies();
  }

  /*
   * 
   */
  public Company[] createCompanies() {
    Company[] tempCompanies = new Company[organizations.length];

    for (int index = 0; index < tempCompanies.length; index++) {
      tempCompanies[index] = new Company(organizations[index], annualProfits[index]);
    }

    return tempCompanies;
  }

  /*
   *
   */
  public String getAllProfits() {
    String result = "";

    for (int index = 0; index < annualProfits.length; index++) {
      result += annualProfits[index] + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing each company's information
   */
  public String toString() {
    String result = "Top Companies and Profits\\n----------\\n";

    for (Company org : companies) {
      result += org + "\\n";
    }

    return result;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





  
  
}`}],validationFiles:[],dataFiles:[{path:`companies.txt`,text:`Berkshire Hathaway
JPMorgan Chase
Amazon
Apple
Bank of America
Alphabet
Microsoft
ExxonMobil
Wells Fargo
Verizon Communications
AT&T
UnitedHealth Group
Walmart
Chevron
Citigroup
Comcast
Meta Platforms
Morgan Stanley
Goldman Sachs Group
Johnson & Johnson
CVS Health
Pfizer
Intel
Procter & Gamble
AbbVie
General Motors
MetLife
American Express
Cigna
Prudential Financial
PepsiCo
Merck & Co.
Elevance Health
American International Group
Cisco Systems
Walt Disney
IBM
United Parcel Service
Raytheon Technologies
Capital One
Bristol Myers Squibb
Coca-Cola
Oracle
Thermo Fisher Scientific
Costco Wholesale
Charter Communications
ConocoPhillips
US Bancorp
Abbott Laboratories
Caterpillar`},{path:`profits.txt`,text:`89.8
42.12
33.36
100.56
31
76.03
71.18
23.04
20.58
21.52
17.33
17.45
13.67
15.62
18.23
14.16
39.37
14.58
18.74
19.83
7.91
21.98
19.87
14.6
11.46
10.02
6.55
7.86
5.36
7.61
7.62
13.05
6.24
9.39
11.82
3.08
5.52
12.89
3.86
12.29
6.99
9.77
7.56
7.72
5.51
4.65
8.08
7.21
7.7
6.49`}]},{name:`Investigate and Modify: Enhanced For Loops #3`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Forbes topCompanies = new Forbes("companies.txt", "profits.txt");
    
    System.out.println(topCompanies);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
  }
}`},{path:`Company.java`,text:`/*
 * Represents a company
 */
public class Company {

  private String name;      // The name of a company
  private double profits;   // The annual profits in billions made by a company

  /*
   * Sets name and profits to the specified values
   */
  public Company(String name, double profits) {
    this.name = name;
    this.profits = profits;
  }

  /*
   * Returns the name of the company
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the profits made by the company
   */
  public double getProfits() {
    return profits;
  }

  /*
   * Returns a String containing the name of
   * the company and its annual profits
   */
  public String toString() {
    return name + " - $" + profits + " billion annual profit";
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
  
}`},{path:`Forbes.java`,text:`/*
 * Manages data about Forbes top companies
 */
public class Forbes {

  private String[] organizations;   // The 1D array containing the company names
  private double[] annualProfits;   // The 1D array containing the annual profits of each company
  private Company[] companies;      // The 1D array of Company objects

  /*
   * 
   */
  public Forbes(String companiesFile, String profitsFile) {
    organizations = FileReader.toStringArray(companiesFile);
    annualProfits = FileReader.toDoubleArray(profitsFile);
    companies = createCompanies();
  }

  /*
   * 
   */
  public Company[] createCompanies() {
    Company[] tempCompanies = new Company[organizations.length];

    for (int index = 0; index < tempCompanies.length; index++) {
      tempCompanies[index] = new Company(organizations[index], annualProfits[index]);
    }

    return tempCompanies;
  }

  /*
   *
   */
  public String getAllProfits() {
    String result = "";

    for (int index = 0; index < annualProfits.length; index++) {
      result += annualProfits[index] + "\\n";
    }

    return result;
  }

  /*
   * Returns a String containing each company's information
   */
  public String toString() {
    String result = "Top Companies and Profits\\n----------\\n";

    for (Company org : companies) {
      result += org + "\\n";
    }

    return result;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





  
  
}`}],validationFiles:[],dataFiles:[{path:`companies.txt`,text:`Berkshire Hathaway
JPMorgan Chase
Amazon
Apple
Bank of America
Alphabet
Microsoft
ExxonMobil
Wells Fargo
Verizon Communications
AT&T
UnitedHealth Group
Walmart
Chevron
Citigroup
Comcast
Meta Platforms
Morgan Stanley
Goldman Sachs Group
Johnson & Johnson
CVS Health
Pfizer
Intel
Procter & Gamble
AbbVie
General Motors
MetLife
American Express
Cigna
Prudential Financial
PepsiCo
Merck & Co.
Elevance Health
American International Group
Cisco Systems
Walt Disney
IBM
United Parcel Service
Raytheon Technologies
Capital One
Bristol Myers Squibb
Coca-Cola
Oracle
Thermo Fisher Scientific
Costco Wholesale
Charter Communications
ConocoPhillips
US Bancorp
Abbott Laboratories
Caterpillar`},{path:`profits.txt`,text:`89.8
42.12
33.36
100.56
31
76.03
71.18
23.04
20.58
21.52
17.33
17.45
13.67
15.62
18.23
14.16
39.37
14.58
18.74
19.83
7.91
21.98
19.87
14.6
11.46
10.02
6.55
7.86
5.36
7.61
7.62
13.05
6.24
9.39
11.82
3.08
5.52
12.89
3.86
12.29
6.99
9.77
7.56
7.72
5.51
4.65
8.08
7.21
7.7
6.49`}]},{name:`Practice: Converting Loops (a)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Bicycle object
    Bicycle fremontBridge = new Bicycle("counts.txt");

    // Calls the calcTotalBicycles() method and prints the result
    System.out.println("Total Bicycles: " + fremontBridge.calcTotalBicycles());
    
  }
}`},{path:`Bicycle.java`,text:`/*
 * Manages data about bicycles that cross the Seattle Fremont Bridge
 */
public class Bicycle {

  // The 1D array of the number of bicycles that crossed the bridge each hour
  private int[] bicycleCounts;

  /*
   * Reads the data from the specified filename to initialize bicycleCounts
   */
  public Bicycle(String filename) {
    this.bicycleCounts = FileReader.toIntArray(filename);
  }

  /*
   * Calculates and returns the total bicycles that crossed the bridge
   */
  public int calcTotalBicycles() {
    int total = 0;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this for loop to use an enhanced for loop.
     * -----------------------------------------------------------------------------
     */

    for (int index = 0; index < bicycleCounts.length; index++) {
      total += bicycleCounts[index];
    }

    return total;
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
  
}`}],validationFiles:[{path:`BicycleTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Bicycle.java Test")
public class BicycleTest {

  int[] testCounts;
  Bicycle testBicycle;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testCounts = FileReader.toIntArray("counts.txt");
    testBicycle = new Bicycle("counts.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the total number of bicycles that crossed the bridge => ")
  public void testCalcTotalBicycles() {
    message = "The method should still perform the same task! When converting to an enhanced for";
    message += "\\n        loop, update the loop header and any references to array elements.";
    message += messageGap;
      
    int expected = getExpectedResult();
    int actual = testBicycle.calcTotalBicycles();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  public int getExpectedResult() {
    int total = 0;

    for (int index = 0; index < testCounts.length; index++) {
      total += testCounts[index];
    }

    return total;
  }
  
}`}],dataFiles:[{path:`counts.txt`,text:`26
20
4
10
14
62
310
704
874
552
236
84
152
180
256
328
630
1166
760
256
160
126
98
34
36
6
18
6
16
52
284
638
836
482
240
216
208
252
238
294
550
1014
756
328
166
140
92
82`}]},{name:`Practice: Converting Loops (b)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Nintendo object
    Nintendo topGames = new Nintendo("titles.txt", "copies.txt");

    // Prints the Nintendo object
    System.out.println(topGames);
    
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
  
}`},{path:`Game.java`,text:`/*
 * Represents a video game
 */
public class Game {

  private String title;    // The title of a game
  private int copies;      // The number of copies sold

  /*
   * Sets title and copies to the specified values
   */
  public Game(String title, int copies) {
    this.title = title;
    this.copies = copies;
  }

  /*
   * Returns the title of the game
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the number of copies sold
   */
  public int getCopies() {
    return copies;
  }

  /*
   * Returns a String containing the title and number of copies sold
   */
  public String toString() {
    return title + ": " + copies + " copies sold";
  }
  
}`},{path:`Nintendo.java`,text:`/*
 * Manages data about best selling video games
 */
public class Nintendo {

  private Game[] bestSellingGames;   // The 1D array of best selling video games

  /*
   * Reads the data from titlesFile and copiesFile to initialize bestSellingGames
   */
  public Nintendo(String titlesFile, String copiesFile) {
    bestSellingGames = createGames(titlesFile, copiesFile);
  }

  /*
   * Returns a 1D array of Game objects using the data from titlesFile and copiesFile
   */
  public Game[] createGames(String titlesFile, String copiesFile) {
    String[] titlesData = FileReader.toStringArray(titlesFile);
    int[] copiesData = FileReader.toIntArray(copiesFile);
    Game[] tempGames = new Game[titlesData.length];

    for (int index = 0; index < tempGames.length; index++) {
      tempGames[index] = new Game(titlesData[index], copiesData[index]);
    }

    return tempGames;
  }

  /*
   * Returns the 1D array of Game objects
   */
  public Game[] getBestSellingGames() {
    return bestSellingGames;
  }

  /*
   * Returns a String containing the information about each Game object
   */
  public String toString() {
    String result = "";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this for loop to an enhanced for loop.
     * -----------------------------------------------------------------------------
     */

    for (int index = 0; index < bestSellingGames.length; index++) {
      result += bestSellingGames[index] + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`NintendoTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Nintendo.java Test")
public class NintendoTest {

  Nintendo testNintendo;
  Game[] testGames;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testNintendo = new Nintendo("titles.txt", "copies.txt");
    testGames = testNintendo.getBestSellingGames();
  }
   
  @Test
  @Order(1)
  @DisplayName("Return a String containing the information for each video game => ")
  public void testToString() {
    message = "The method should still perform the same task! When converting to an enhanced for";
    message += "\\n        loop, update the loop header and any references to array elements.";
    message += messageGap;
      
    String expected = getExpectedResult();
    String actual = testNintendo.toString();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  public String getExpectedResult() {
    String result = "";

    for (int index = 0; index < testGames.length; index++) {
      result += testGames[index] + "\\n";
    }

    return result;
  }
  
}`}],dataFiles:[{path:`copies.txt`,text:`48410000
40170000
29530000
27790000
25370000
24400000
18350000
14920000
14870000
14810000
13910000
13310000
13300000
11430000
10000000
9430000
9070000
8070000
7900000
7890000
7700000
6150000
6080000
5270000
4280000
4220000
4120000
4000000
3980000
3910000
3820000
3630000
3500000
3340000
3310000
3200000
3010000
2900000
2740000
2660000
2440000
2350000
2230000
2170000
2160000
2130000
2010000
2000000
1890000
1890000
1720000
1680000
1680000
1590000
1580000
1550000
1540000
1500000
1420000
1280000
1270000
1260000
1200000
1200000
1060000
1040000
1000000
1000000
1000000
1000000
1000000
1000000
1000000`},{path:`titles.txt`,text:`Mario Kart 8 Deluxe
Animal Crossing: New Horizons
Super Smash Bros. Ultimate
The Legend of Zelda: Breath of the Wild
Pokemon Sword and Shield
Super Mario Odyssey
Super Mario Party
Pokemon Brilliant Diamond and Shining Pearl
Ring Fit Adventure
Pokemon: Let's Go, Pikachu! and Let's Go, Eevee!
Pokemon Legends: Arceus
New Super Mario Bros. U Deluxe
Splatoon 2
Luigi's Mansion 3
Pokemon Scarlet and Violet
Super Mario 3D World + Bowser's Fury
Super Mario 3D All-Stars
Mario Party Superstars
Splatoon 3
Super Mario Maker 2
Monster Hunter Rise
Nintendo Switch Sports
The Legend of Zelda: Link's Awakening
Kirby and the Forgotten Land
Mario Tennis Aces
Clubhouse Games: 51 Worldwide Classics
Donkey Kong Country: Tropical Freeze
Hyrule Warriors: Age of Calamity
Kirby Star Allies
The Legend of Zelda: Skyward Sword HD
Fire Emblem: Three Houses
1-2-Switch
Momotaro Dentetsu: Showa, Heisei, Reiwa Mo Teiban!
Paper Mario: The Origami King
Minecraft
Among Us
Yoshi's Crafted World
Metroid Dread
New Pokemon Snap
Arms
Xenoblade Chronicles 2
Mario Golf: Super Rush
Pikmin 3 Deluxe
Mario Strikers: Battle League
Octopath Traveler
Captain Toad: Treasure Tracker
Dragon Ball FighterZ
Mario + Rabbids Kingdom Battle
Pokemon Mystery Dungeon: Rescue Team DX
Taiko no Tatsujin: Drum 'n' Fun!
Xenoblade Chronicles 3
Miitopia
Xenoblade Chronicles: Definitive Edition
Big Brain Academy: Brain vs. Brain
Mario Kart Live: Home Circuit
Dragon Ball Xenoverse 2
Pokken Tournament DX
Marvel Ultimate Alliance 3: The Black Order
Nintendo Labo Toy-Con 01: Variety Kit
Astral Chain
WarioWare: Get It Together!
Naruto Shippudden: Ultimate Ninja Storm Trilogy
Dr Kawashima's Brain Training for Nintendo Switch
Resident Evil: Revelations Collection
Game Builder Garage
Bayonetta 2
Enter the Gungeon
Fire Emblem Warriors: Three Hopes
Fitness Boxing
Fitness Boxing 2: Rhythm and Exercise
Shin Megami Tensei V
Story of Seasons: Pioneers of Olive Town
Thief Simulator`}]},{name:`Practice: Converting Loops (c)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a WorldCup object
    WorldCup fifa = new WorldCup("hosts.txt", "years.txt", "attend.txt");

    // Calls the calcAverageAttendance() method and prints the result
    System.out.println("Average Attendance at the FIFA World Cup: " + fifa.calcAverageAttendance());

    // Prints the WorldCup object
    System.out.println("\\n" + fifa);
    
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
  
}`},{path:`Tournament.java`,text:`/*
 * Represents a FIFA World Cup tournament
 */
public class Tournament {

  private String host;      // The host of a tournament
  private String year;      // The year of a tournament
  private int attendance;   // The total attendance of a tournament

  /*
   * Sets host, year, and attendance to the specified values
   */
  public Tournament(String host, String year, int attendance) {
    this.host = host;
    this.year = year;
    this.attendance = attendance;
  }

  /*
   * Returns the host of the tournament
   */
  public String getHost() {
    return host;
  }

  /*
   * Returns the year of the tournament
   */
  public String getYear() {
    return year;
  }

  /*
   * Returns the total attendance of the tournament
   */
  public int getAttendance() {
    return attendance;
  }

  /*
   * Returns a String containing the host, year, and attendance
   */
  public String toString() {
    return year + " hosted by " + host + " - " + attendance + " total attendance";
  }
  
}`},{path:`WorldCup.java`,text:`/*
 * Manages data the FIFA World Cup
 */
public class WorldCup {

  private Tournament[] worldCups;   // The 1D array of Tournament objects

  /*
   * Reads the data from hostsFile, yearsFile,
   * and attendanceFile to initialize worldCups
   */
  public WorldCup(String hostsFile, String yearsFile, String attendanceFile) {
    worldCups = createWorldCups(hostsFile, yearsFile, attendanceFile);
  }

  /*
   * Returns a 1D array of Tournament objects using the
   * data from hostsFile, yearsFile, and attendanceFile
   */
  public Tournament[] createWorldCups(String hostsFile, String yearsFile, String attendanceFile) {
    String[] hostsData = FileReader.toStringArray(hostsFile);
    String[] yearsData = FileReader.toStringArray(yearsFile);
    int[] attendanceData = FileReader.toIntArray(attendanceFile);
    Tournament[] tempTournaments = new Tournament[hostsData.length];

    for (int index = 0; index < tempTournaments.length; index++) {
      tempTournaments[index] = new Tournament(hostsData[index], yearsData[index], attendanceData[index]);
    }

    return tempTournaments;
  }

  /*
   * Calculates and returns the average attendance of all World Cup games
   */
  public int calcAverageAttendance() {
    int total = 0;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this while loop to an enhanced for loop.
     * -----------------------------------------------------------------------------
     */
    int index = 0;
    
    while (index < worldCups.length) {
      total += worldCups[index].getAttendance();
      index++;
    }

    return total / worldCups.length;
  }

  /*
   * Returns a String containing the information about each Tournament object
   */
  public String toString() {
    String result = "";

    for (Tournament event : worldCups) {
      result += event + "\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`WorldCupTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("WorldCup.java Test")
public class WorldCupTest {

  int[] testAttendance;
  WorldCup testWorldCup;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testAttendance = FileReader.toIntArray("attend.txt");
    testWorldCup = new WorldCup("hosts.txt", "years.txt", "attend.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average attendance at the FIFA World Cup => ")
  public void testCalcAverageAttendance() {
    message = "The method should still perform the same task! When converting to an enhanced for";
    message += "\\n        loop, update the loop header and any references to array elements.";
    message += messageGap;
      
    int expected = getExpectedResult();
    int actual = testWorldCup.calcAverageAttendance();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  public int getExpectedResult() {
    int total = 0;
    
    for (int index = 0; index < testAttendance.length; index++) {
      total += testAttendance[index];
    }

    return total / testAttendance.length;
  }
  
}`}],dataFiles:[{path:`attend.txt`,text:`590549
363000
375700
1045246
768607
819810
893172
1563135
1603975
1865753
1545791
2109723
2394031
2516215
3587538
2785100
2705197
3359439
3178856
3429873
3031768
3404252`},{path:`hosts.txt`,text:`Uruguay
Italy
France
Brazil
Switzerland
Sweden
Chile
England
Mexico
West Germany
Argentina
Spain
Mexico
Italy
United States
France
South KoreaÂ\xA0Japan
Germany
South Africa
Brazil
Russia
Qatar`},{path:`years.txt`,text:`1930
1934
1938
1950
1954
1958
1962
1966
1970
1974
1978
1982
1986
1990
1994
1998
2002
2006
2010
2014
2018
2022`}]},{name:`Practice: Converting Loops (d)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a MusicSurvey object
    MusicSurvey surveyResults = new MusicSurvey("ages.txt", "hours.txt");

    // Calls the calcAverageHours() method and prints the result
    System.out.print("Average Number of Hours Respondents Listen to Music: ");
    System.out.println(surveyResults.calcAverageHours());
    
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about music survey responses
 */
public class MusicSurvey {

  private Respondent[] answers;   // The 1D array of Respondent objects

  /*
   * Reads the data from agesFile and hoursFile to initialize answers
   */
  public MusicSurvey(String agesFile, String hoursFile) {
    answers = createResponses(agesFile, hoursFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the data from agesFile and hoursFile
   */
  public Respondent[] createResponses(String agesFile, String hoursFile) {
    int[] agesData = FileReader.toIntArray(agesFile);
    double[] hoursData = FileReader.toDoubleArray(hoursFile);
    Respondent[] tempResponses = new Respondent[agesData.length];

    for (int index = 0; index < tempResponses.length; index++) {
      tempResponses[index] = new Respondent(agesData[index], hoursData[index]);
    }

    return tempResponses;
  }

  /*
   * Calculates and returns the average number of hours respondents listen to music
   */
  public double calcAverageHours() {
    double total = 0.0;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Convert this while loop to use an enhanced for loop.
     * -----------------------------------------------------------------------------
     */

    int index = 0;
    
    while (index < answers.length) {
      total += answers[index].getHours();
      index++;
    }

    return total / answers.length;
  }
}`},{path:`Respondent.java`,text:`/*
 * Represents a respondent on a survey
 */
public class Respondent {

  private int age;        // The age of a respondent
  private double hours;   // The number of hours per day a respondent listens to music

  /*
   * Sets age and hours to the specific values
   */
  public Respondent(int age, double hours) {
    this.age = age;
    this.hours = hours;
  }

  /*
   * Returns the age of the respondent
   */
  public int getAge() {
    return age;
  }

  /*
   * Returns the number of hours per day the respondent listens to music
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns a String containing the age and hours for the respondent
   */
  public String toString() {
    return age + " year old respondent - " + hours + " hours per day";
  }
  
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  double[] testHours;
  MusicSurvey testMusicSurvey;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testHours = FileReader.toDoubleArray("hours.txt");
    testMusicSurvey = new MusicSurvey("ages.txt", "hours.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average number of hours each respondent listened to music => ")
  public void testCalcAverageHours() {
    message = "The method should still perform the same task! When converting to an enhanced for";
    message += "\\n        loop, update the loop header and any references to array elements.";
    message += messageGap;
      
    double expected = getExpectedResult();
    double actual = testMusicSurvey.calcAverageHours();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  public double getExpectedResult() {
    double total = 0.0;
    
    for (int index = 0; index < testHours.length; index++) {
      total += testHours[index];
    }

    return total / testHours.length;
  }
  
}`}],dataFiles:[{path:`ages.txt`,text:`18
63
18
61
18
18
18
21
19
18
18
19
17
19
18
17
16
16
17
15
15
17
19
18
16
18
14
18
17
17
20
19
19
17
16
18
21
17
26
20
23
18
37
17
18
17
36
24
18
19
17
16
23
23
22
16
18
42
23
23
19
22
15
25
32
36
24
29
41
36
19
31
16
26
22
16
17
19
18
18
19
19
22
16
19
37
14
33
26
32
30
43
24
36
19
22
20
31
19
15
18
25
17
28
20
21
41
20
34
21
23
16
19
22
19
23
23
22
17
28
23
19
17
21
29
22
18
13
24
35
29
28
32
32
21
18
19
18
16
25
17
16
27
53
25
34
22
17
26
19
21
32
21
25
26
21
17
28
41
18
22
18
49
21
27
19
27
27
19
42
60
19
20
28
16
18
34
18
21
38
17
35
21
25
18
16
19
31
26
16
26
17
22
20
30
26
19
21
33
28
25
22
19
44
22
26
23
17
42
30
35
20
18
21
19
37
31
29
20
36
18
18
25
18
31
17
21
30
40
64
14
26
35
33
30
31
27
53
23
25
16
18
29
31
31
38
17
25
20
15
16
20
23
22
15
17
22
21
19
32
34
18
21
20
19
25
31
21
31
20
19
19
23
24
20
15
17
20
14
24
14
23
28
22
25
27
18
17
23
40
27
18
16
18
23
15
20
16
16
20
33
22
21
21
29
56
17
22
26
30
49
22
32
32
22
17
21
17
25
15
13
38
23
17
59
22
17
27
23
15
22
23
19
32
40
21
21
23
21
35
18
21
22
23
13
19
33
18
17
16
18
19
15
17
25
24
23
16
25
16
20
17
29
20
16
21
16
43
32
10
27
24
18
17
19
16
64
33
32
15
27
20
23
19
12
16
17
14
32
20
54
18
14
13
19
14
14
18
15
24
17
44
23
21
57
23
18
26
13
15
57
12
26
17
17
69
17
32
22
38
18
14
14
19
20
18
30
31
72
73
16
19
16
12
39
15
14
21
38
23
16
21
13
16
18
21
17
24
14
25
22
20
23
61
24
34
49
33
22
38
23
34
40
27
54
16
23
35
22
41
63
28
22
58
58
67
70
60
15
18
58
28
22
49
20
55
48
14
18
25
71
60
18
19
74
36
14
20
15
19
29
24
18
21
19
25
28
27
60
25
60
43
51
63
64
22
24
19
56
20
19
20
60
65
28
18
42
80
22
56
20
15
67
57
22
17
19
50
24
67
18
20
15
17
32
18
18
18
19
18
17
34
18
18
22
18
19
25
17
25
18
21
24
16
18
18
16
68
16
21
53
29
18
29
18
22
19
13
26
14
27
26
59
30
27
18
23
18
40
24
23
56
29
16
18
21
32
22
17
18
18
24
21
46
18
16
21
19
20
21
56
21
20
21
25
33
20
17
24
26
17
16
27
28
48
19
21
21
15
20
21
22
22
31
20
49
42
26
17
30
23
30
31
42
16
34
21
27
18
26
37
26
19
22
43
24
21
28
23
20
17
19
32
18
20
27
15
26
24
20
21
16
21
27
17
21
17
21
17
36
18
18
18
18
18
18
18
18
18
20
19
17
23
18
19
89
20
16
17
16
20
30
37
44
21
19
23
26
18
35
16
19
23
17
29
21
17
23
17
22
17
19
19
16
19
13
18
26
14
21
21
17
18
19
19
29`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3
2
12
6
2
1
5
2
6
4
1
5
4
3
0.5
4
2
5
0.25
3
4
1
1
3
5
3
2
6
1
12
4
2
6
0
7
1
1
1
2
2
5
6
3
4
3
2
3
2
4
2
0.5
5
5
10
2
4
5
3
2
2.5
3
1
7
1
1
3
5
6
4
1
4
10
3
2
4
8
2
0.5
5
1
3
2
3
2
4
3
2
3
4
2
8
4
2
1
3
0.5
2
3
2
1
5
10
4
2
1
6
1
6
7
1
3
2
3
2
3
1.5
4
2
10
7
1
2
6
2
3
8
3
3
3
2
1
2
2
4
4
1
2
2
1
3
10
1
3
1.5
3
2.5
2
5
2
2
2
2
1
1.5
5
0.5
2
0.5
2
2.5
2
2
1
4
5
2
2
2
1
4
1
4
5
5
7
2
3
2
3
3
3
5
6
2
2
1
1
3
3
1
6
4
2
2
3
8
3
2
7
12
9
2
1
1
2
4
2
6
4
1
6
2
5
1.5
5
2
4
2
5
6
1
6
4
2
2
2
4
2
6
3
5
0.5
1
10
2
0.5
1
2
3
3
2
1
8
1
1
1
5
5
4
8
0.7
3
7
1.5
4
4
10
2
2
6
0.5
2
5
3
4.5
3
20
1
2.5
1
2
5
3
3
0.5
8
2
3
3
5
1.5
1
2
1
4
1
2
2
2
1
6
2
4
1.5
3
4
10
3
3
3
2
2
3
2
5
6
3
5
5
6
6
3
10
2
2
3
1.5
10
2
3
2.5
3
0.5
16
6
3
0.5
5
2
2
2
1.5
1
10
2
15
0.5
3
2
4
2
1
24
1
2
2
3
3
2
7
6
3
5
4
3
2
3
2
2
3
2
3
0
1
2
2
6
5
2
4
8
4
1.5
2
5
6
2
2
1
1
2
2
1
14
3
3
2
0.5
6
1
2
4
3
12
3
1
10
5
0.5
5
3
2
13
4
6
3
4
3
2
2
1
6
9
1
1
0.5
1
1.5
6
4
6
4
1.5
2
2
4
3
4
2
2
6
0
1
4
2
3
4
2
3
1
10
10
10
4
4
3
2
1
6
1
1
4
2
2
2
6
3
1
4
0.5
8
15
1.5
2
3
1
0.25
1
1
5
1
1
7
3
3
7
8
2
2
2
2
4
8
4
1
5
2
1
5
8
2
2
1
8
8
1
1
8
2
6
3
4
2
3
4
2
1
1
1
2
7
5
3
1
6
2
3
1.5
4
2
2
2
1
3
3
2
6
3
3
2
2
2
3
4
2
5
2
1
7
3
3
4
3
1
7
1
4
4
3
2
4
5
6
1
1
3
1
8
1
8
1
1
10
10
5
10
2
6
2
3
8
8
8
8
12
6
4
8
2
1
2
3
1
0
2
0.1
2
12
1
1
3
2
2
3
1
3
3
5
8
4
5
3
4
1
4
1
2
6
2
1
4
8
2
2
10
0
6
3
2
0.25
4
2
2
3
6
3
2
5
4
4
0
12
0.5
1
3
12
4
3
1
4
2
3
3
1
2
1
0.5
2
11
10
5
1.5
2
3
2
3
4
1
5
2
7
2
1
1
4
6
3
4
4
1
2
24
5
3
4
9
1
1
2
1.5
2
4
1
1
3
4
8
5
3
8
1
4
1
2
1
3
5
5
4
6
6
1
18
1
7
0.5
2
2
1
6
5
2`}]},{name:`Practice: Traversing 1D Arrays (a)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a ScreenTime object with the "times.txt" file, then call
     * the calcAverageTime() method and print the result.
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
  
}`},{path:`ScreenTime.java`,text:`/*
 * Manages data about screen time
 */
public class ScreenTime {

  private int[] socialNetworkTimes;    // The 1D array of the time spent on social media apps

  /*
   * Reads the data from the specified filename
   * to initialize socialNetworkTimes
   */
  public ScreenTime(String filename) {
    socialNetworkTimes = FileReader.toIntArray(filename);
  }

  /*
   * Calculates and returns the average amount of
   * time spent on social networking apps
   */
  public int calcAverageTime() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use an enhanced for loop to calculate the average of the values
     * stored in the 1D array socialNetworkTimes.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }
  
}`}],validationFiles:[{path:`ScreenTimeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ScreenTime.java Test")
public class ScreenTimeTest {

  int[] testTimes;
  ScreenTime testScreenTime;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testTimes = FileReader.toIntArray("times.txt");
    testScreenTime = new ScreenTime("times.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average amount of time spent on social networking apps => ")
  public void testCalcAverageTime() {
    message = "Use an enhanced for loop to traverse the 1D array socialNetworkTimes.";
    message += "\\n        Add each value to a total, then return the total divided by the length of socialNetworkTimes.";
    message += messageGap;
      
    int expected = getExpectedResult();
    int actual = testScreenTime.calcAverageTime();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  private int getExpectedResult() {
    int total = 0;

    for (int value : testTimes) {
      total += value;
    }

    return total / testTimes.length;
  }

}`}],dataFiles:[{path:`times.txt`,text:`89
78
52
69
35
68
56
98
25
76
75
42
46
40
90
60
64
34
109
81
70
53
42
93
49
28
37
41`}]},{name:`Practice: Traversing 1D Arrays (b)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Countries object with the "names.txt" and "populations.txt"
     * files, then call the calcAveragePopulation() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Countries.java`,text:`/*
 * Manages data about populations in countries
 */
public class Countries {

  private String[] countries;     // The 1D array of country names
  private int[] populations;      // The 1D array of the population of each country

  /*
   * Reads the data from countriesFile and popFile to
   * initialize countries and populations
   */
  public Countries(String countriesFile, String popFile) {
    countries = FileReader.toStringArray(countriesFile);
    populations = FileReader.toIntArray(popFile);
  }

  /*
   * Calculates and returns the average population of each country
   */
  public int calcAveragePopulation() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use an enhanced for loop to calculate the average of the values
     * stored in the 1D array populations.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Returns a String containing the name of each country and its population
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < countries.length; index++) {
      result += countries[index] + " population: " + populations[index] + "\\n";
    }

    return result;
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
  
}`}],validationFiles:[{path:`CountriesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Countries.java Test")
public class CountriesTest {

  int[] testPopulations;
  Countries testCountries;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPopulations = FileReader.toIntArray("populations.txt");
    testCountries = new Countries("names.txt", "populations.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average population of each country => ")
  public void testCalcAveragePopulation() {
    message = "Use an enhanced for loop to traverse the 1D array populations.";
    message += "\\n        Add each value to a total, then return the total divided by the length of populations.";
    message += messageGap;
      
    int expected = getExpectedResult();
    int actual = testCountries.calcAveragePopulation();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  private int getExpectedResult() {
    int total = 0;

    for (int value : testPopulations) {
      total += value;
    }

    return total / testPopulations.length;
  }

}`}],dataFiles:[{path:`names.txt`,text:`China
India
United States
Indonesia
Pakistan
Nigeria
Brazil
Bangladesh
Russia
Mexico
Japan
Philippines
Ethiopia
Egypt
DR Congo
Vietnam
Iran
Turkey
Germany
France
United Kingdom
Thailand
Tanzania
South Africa
Italy`},{path:`populations.txt`,text:`1412600000
1375586000
334200979
275773800
235825000
218541000
215565116
165158616
145100000
128533664
125104000
112869745
105163988
104243583
99010000
98506193
86053807
84680273
84270625
67975000
67026292
66875330
61741120
60604992
58853482`}]},{name:`Practice: Traversing 1D Arrays (c)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Basketball object with the "names.txt" and "points.txt"
     * files, then call the calcAveragePoints() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Basketball.java`,text:`/*
 * Manages data about basketball players
 */
public class Basketball {

  private Player[] players;    // The 1D array of Player objects

  /*
   * Reads the data from namesFile and pointsFile to
   * initialize the 1D array players
   */
  public Basketball(String namesFile, String pointsFile) {
    players = createPlayers(namesFile, pointsFile);
  }

  /*
   * Returns a 1D array of Player objects using the data
   * from namesFile and pointsFile
   */
  public Player[] createPlayers(String namesFile, String pointsFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    int[] pointsData = FileReader.toIntArray(pointsFile);
    Player[] tempPlayers = new Player[namesData.length];

    for (int index = 0; index < tempPlayers.length; index++) {
      tempPlayers[index] = new Player(namesData[index], pointsData[index]);
    }

    return tempPlayers;
  }

  /*
   * Calculates and returns the average points scored by each player
   */
  public int calcAveragePoints() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use an enhanced for loop to calculate the average points scored by
     * each player in the 1D array players.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Returns a String containing each player's information
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < players.length; index++) {
      result += players[index] + "\\n";
    }

    return result;
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
  
}`},{path:`Player.java`,text:`/*
 * Represents a basketball player
 */
public class Player {

  private String name;       // The name of a player
  private int totalPoints;   // The total points a player scored

  /*
   * Sets name and totalPoints to the specified values
   */
  public Player(String name, int totalPoints) {
    this.name = name;
    this.totalPoints = totalPoints;
  }

  /*
   * Returns the name of the player
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the total points the player scored
   */
  public int getTotalPoints() {
    return totalPoints;
  }

  /*
   * Returns a String containing the name of the player
   * and the total points the player scored
   */
  public String toString() {
    return name + ": " + totalPoints + " total points scored";
  }
  
}`}],validationFiles:[{path:`BasketballTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Basketball.java Test")
public class BasketballTest {

  int[] testPoints;
  Basketball testBasketball;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPoints = FileReader.toIntArray("points.txt");
    testBasketball = new Basketball("names.txt", "points.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average points scored by each player => ")
  public void testCalcAveragePoints() {
    message = "Use an enhanced for loop to traverse the 1D array players. Use the getTotalPoints() method in the Player class";
    message += "\\n        to add each player's points to the total, then return the total divided by the length of players.";
    message += messageGap;
      
    int expected = getExpectedResult();
    int actual = testBasketball.calcAveragePoints();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  private int getExpectedResult() {
    int total = 0;

    for (int value : testPoints) {
      total += value;
    }

    return total / testPoints.length;
  }

}`}],dataFiles:[{path:`names.txt`,text:`Kareem Abdul-Jabbar
LeBron James
Karl Malone
Kobe Bryant
Michael Jordan
Dirk Nowitzki
Wilt Chamberlain
Shaquille O'Neal
Carmelo Anthony
Moses Malone
Elvin Hayes
Hakeem Olajuwon
Oscar Robertson
Kevin Durant
Dominique Wilkins
Tim Duncan
Paul Pierce
John Havlicek
Kevin Garnett
Vince Carter
Alex English
Reggie Miller
Jerry West
Patrick Ewing
Ray Allen
Allen Iverson
James Harden
Russell Westbrook
Charles Barkley
Robert Parish
Adrian Dantley
Dwyane Wade
Elgin Baylor
Clyde Drexler
Gary Payton
Larry Bird
Hal Greer
Chris Paul
Walt Bellamy
DeMar DeRozan
Pau Gasol
Bob Pettit
Stephen Curry
David Robinson
George Gervin
LaMarcus Aldridge
Mitch Richmond
Joe Johnson
Tom Chambers
Antawn Jamison`},{path:`points.txt`,text:`38387
37965
36928
33643
32292
31560
31419
28596
28289
27409
27313
26946
26710
26684
26668
26496
26397
26395
26071
25728
25613
25279
25192
24815
24505
24368
24005
23845
23757
23334
23177
23165
23149
22195
21813
21791
21586
21276
20941
20925
20894
20880
20843
20790
20708
20558
20497
20407
20049
20042`}]},{name:`Practice: Traversing 1D Arrays (d)`,lesson:`Lesson 10: Enhanced For Loops`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Congress object with the "names.txt" and "funds.txt" files,
     * then call the calcAverageFundsRaised() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Congress.java`,text:`/*
 * Manages data about members of congress
 */
public class Congress {

  private Member[] members;    // The 1D array of Member objects

  /*
   * Reads the data from namesFile and fundsFile to
   * to initialize the 1D array members
   */
  public Congress(String namesFile, String fundsFile) {
    members = createMembers(namesFile, fundsFile);
  }

  /*
   * Returns a 1D array of Member objects using the data
   * from namesFile and fundsFile
   */
  public Member[] createMembers(String namesFile, String fundsFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    double[] fundsData = FileReader.toDoubleArray(fundsFile);
    Member[] tempMembers = new Member[namesData.length];

    for (int index = 0; index < tempMembers.length; index++) {
      tempMembers[index] = new Member(namesData[index], fundsData[index]);
    }

    return tempMembers;
  }

  /*
   * Calculates and returns the average funds raised by each member
   */
  public double calcAverageFundsRaised() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use an enhanced for loop to calculate the average funds raised by each
     * member in the 1D array members.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Returns a String containing each member's information
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < members.length; index++) {
      result += members[index] + "\\n";
    }

    return result;
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
  
}`},{path:`Member.java`,text:`/*
 * Represents a member of Congress
 */
public class Member {

  private String name;      // The name of the member
  private double raised;    // The amount raised by the member

  /*
   * Sets name and raised to the specified values
   */
  public Member(String name, double raised) {
    this.name = name;
    this.raised = raised;
  }

  /*
   * Returns the name of the member
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the amount raised by the member
   */
  public double getRaised() {
    return raised;
  }

  /*
   * Returns a String containing the name of the member
   * and the amount they raised
   */
  public String toString() {
    return name + ": $" + raised + " raised";
  }
  
}`}],validationFiles:[{path:`CongressTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Congress.java Test")
public class CongressTest {

  double[] testFunds;
  Congress testCongress;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testFunds = FileReader.toDoubleArray("funds.txt");
    testCongress = new Congress("names.txt", "funds.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Calculate and return the average funds raised by each member => ")
  public void testCalcAverageFundsRaised() {
    message = "Use an enhanced for loop to traverse the 1D array members. Use the getRaised() method in the Member class";
    message += "\\n        to add each member's funds to the total, then return the total divided by the length of members.";
    message += messageGap;
      
    double expected = getExpectedResult();
    double actual = testCongress.calcAverageFundsRaised();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected result
   */
  private double getExpectedResult() {
    double total = 0;

    for (double value : testFunds) {
      total += value;
    }

    return total / testFunds.length;
  }

}`}],dataFiles:[{path:`funds.txt`,text:`7719396.00
2192741.00
20993041.00
1211111.00
1617611.00
1178693.00
346571.00
593604.00
181286.00
204340.00
4262971.00
11253771.00
11326505.00
838536.00
54700.00
1355357.00
3234766.00
725565.00
726917.00
3184656.00
3447266.00
1761000.00
1757044.00
6069123.00
2586972.00
6817301.00
81913.00
0.00
3873628.00
4331508.00
1857034.00
448866.00
778998.00
6446790.00
2639948.00
761878.00
1242476.00
1881840.00
717298.00
790735.00
152052.00
858890.00
1097555.00
951050.00
1247753.00
14074831.00
3465317.00
1991061.00
1978660.00
3289974.00
2369794.00
2480390.00
1534981.00
1389395.00
620848.00
893757.00
1289389.00
2370318.00
1580076.00
4120442.00
321762.00
1038921.00
1411641.00
193192.00
758691.00
781122.00
1242835.00
3360517.00
1593208.00
1904466.00
344191.00
1406396.00
2094504.00
875803.00
3991112.00
1447143.00
6007472.00
2328949.00
1656992.00
3232315.00
1854187.00
3149814.00
5225390.00
2016114.00
1084888.00
2746695.00
4353630.00
37993811.00
5808554.00
35899966.00
381761.00
509569.00
167506.00
2071661.00
858263.00
474819.00
4220118.00
1332282.00
996345.00
6167303.00
4418709.00
569007.00
510936.00
7828900.00
6131905.00
806946.00
2989524.00
856339.00
1010336.00
3119825.00
5272240.00
2212505.00
5794554.00
220970.00
1793984.00
14535870.00
4798334.00
853475.00
1979358.00
629343.00
704798.00
570467.00
3632692.00
1115797.00
1191099.00
1838461.00
1800445.00
2111035.00
2367635.00
199991.00
1204897.00
1646710.00
1911503.00
12990741.00
3215917.00
2049518.00
983002.00
1427992.00
1076710.00
3437752.00
1881484.00
1936676.00
2461642.00
1029410.00
1720763.00
9778.00
1028474.00
2588432.00
3367598.00
1993756.00
1836753.00
1260631.00
821439.00
1284829.00
783541.00
465226.00
3163351.00
2447306.00
970454.00
1083660.00
1218880.00
991211.00
904754.00
627622.00
15000.00
8724776.00
217953.00
8417574.00
8700992.00
4754168.00
1042771.00
3390174.00
1388678.00
1871503.00
2803934.00
634266.00
1396865.00
563173.00
2895371.00
217083.00
1733173.00
1113250.00
3667685.00
1941614.00
1210017.00
2412193.00
1423370.00
295443.00
1724862.00
2097733.00
3879556.00
1472022.00
2498238.00
1746079.00
1701888.00
1042952.00
5529626.00
696558.00
5300715.00
481307.00
4269860.00
2828252.00
472779.00
273672.00
4215388.00
2253420.00
3078342.00
2824798.00
6278712.00
1393362.00
1754371.00
1506882.00
3208608.00
403990.00
5429416.00
975577.00
4160925.00
1445030.00
5051130.00
827098.00
2417062.00
2714726.00
854849.00
1020109.00
1536235.00
1558322.00
2816286.00
3369890.00
560343.00
2124293.00
923401.00
538284.00
4880505.00
844278.00
335990.00
2302093.00
561230.00
1727906.00
1520006.00
12417107.00
402848.00
1566445.00
2287480.00
638820.00
1888539.00
-42855.00
1729051.00
7790164.00
241369.00
910221.00
1480419.00
1622189.00
6252176.00
1192842.00
1974574.00
1898707.00
1012953.00
864182.00
2941980.00
1373506.00
1517310.00
28119082.00
1614911.00
1992376.00
1442822.00
710380.00
177476.00
6770674.00
190197.00
10260242.00
2335296.00
1642695.00
1775246.00
2099507.00
7720264.00
6193616.00
11028757.00
2027765.00
905730.00
1275434.00
1998752.00
2620536.00
463256.00
0.00
507962.00
2091623.00
2002236.00
466705.00
1432885.00
2692609.00
21441693.00
1846695.00
1794452.00
1375514.00
3077072.00
273024.00
458054.00
1167360.00
24791538.00
8163575.00
3301708.00
2244429.00
452659.00
7112616.00
814098.00
1296300.00
6175563.00
6227355.00
766924.00
1403194.00
6695582.00
2543199.00
1189244.00
8559805.00
0.00
3412257.00
1131363.00
1407018.00
1117075.00
1812003.00
1174363.00
206591.00
168660.00
5042351.00
1727651.00
3965149.00
30914830.00
1652257.00
36709285.00
2855041.00
686777.00
4516392.00
4225010.00
1449001.00
1718171.00
1584197.00
11571029.00
924763.00
566400.00
1447075.00
73140886.00
1000923.00
1123015.00
2563285.00
4466785.00
2561756.00
1403676.00
1434559.00
1869196.00
4163482.00
4604399.00
5929700.00
1977326.00
1117733.00
3305211.00
640039.00
3211215.00
18086343.00
1159191.00
975508.00
1240555.00
2265809.00
1349285.00
13025.00
2728460.00
1515656.00
6258368.00
1500937.00
1269505.00
2181022.00
4182600.00
1788743.00
166850.00
2940815.00
6128156.00
1276358.00
1180269.00
7703753.00
4448589.00
963237.00
138756.00
1547000.00
2000373.00
5622081.00
5503392.00
474786.00
3148542.00
3013074.00
720632.00
5119356.00
22216583.00
1107829.00
1531504.00
4260780.00
1066976.00
677046.00
543105.00
3352245.00
13254.00
730885.00
812657.00
3471514.00
13377243.00
737101.00
1661023.00
2916586.00
876121.00
2279020.00
0.00
2399884.00
5143410.00
2533189.00
7843935.00
306434.00
22490627.00
2664811.00
921792.00
86581469.00
3105241.00
549117.00
2099148.00
6871179.00
145474.00
20520.00
3163938.00
2846323.00
856257.00
979267.00
1516417.00
4731925.00
4080074.00
5550174.00
-1386356.00
1859691.00
1385034.00
4598365.00
1562228.00
3304702.00
605073.00
1386724.00
1589224.00
1674579.00
27680901.00
587143.00
10024604.00
4442159.00
1895941.00
-367162.00
2153850.00
479167.00
2031545.00
2005290.00
2680786.00
1584377.00
141458.00
919350.00
972575.00
2529397.00
2227355.00
4650448.00
4717097.00
1867651.00
6717662.00
4250064.00
761516.00
1373474.00
707321.00
3672268.00
4522424.00
920408.00
3583458.00
2185504.00
1111888.00
714987.00
1731244.00
509828.00
17940130.00
1404019.00
1179969.00
4254563.00
1048457.00
966641.00
5771483.00
5420136.00
2252066.00
918476.00
836909.00
3943221.00
13588871.00
11030158.00
17471712.00
309658.00
1323092.00
2684930.00
2093425.00
783940.00
973980.00
1314552.00
2239085.00
38334636.00
37743256.00
1375262.00
1418349.00
10055406.00
697627.00
2095503.00
4637570.00
3893173.00
7599512.00
1509014.00
3688187.00
1587079.00
1548007.00
1037497.00
881479.00
1393256.00
3851900.00
1144309.00
959008.00
354993.00
1752943.00
2904436.00
1247447.00
64682883.00
2353035.00
3664655.00
1166742.00
2492127.00
2992372.00
3612075.00
1929971.00
871656.00
892064.00
7900497.00
684268.00
3777731.00
1679466.00`},{path:`names.txt`,text:`Abigail Spanberger
Adam Kinzinger
Adam Schiff
Adam Smith
Adrian Smith
Adriano Espaillat
Al Green
Al Lawson
Alan Lowenthal
Albio Sires
Alex Mooney
Alex Padilla
Alexandria Ocasio-Cortez
Alma Adams
Aumua Amata Radewagen
Ami Bera
Amy Klobuchar
Andre Carson
Andrew Clyde
Andrew Garbarino
Andy Barr
Andy Biggs
Andy Harris
Andy Kim
Andy Levin
Angie Craig
Angus King
Ann Kirkpatrick
Ann Kuster
Ann Wagner
Anna Eshoo
Anthony Brown
Anthony Gonzalez
Ashley Hinson
August Pfluger
Austin Scott
Ayanna Pressley
Barbara Lee
Barry Loudermilk
Barry Moore
Ben Cardin
Ben Cline
Ben Ray LujÃ¡n
Ben Sasse
Bennie Thompson
Bernie Sanders
Beth Van Duyne
Betty McCollum
Bill Cassidy
Bill Foster
Bill Hagerty
Bill Huizenga
Bill Johnson
Bill Pascrell
Bill Keating
Bill Posey
Billy Long
Blaine Luetkemeyer
Blake Moore
Bob Casey Jr.
Bob Gibbs
Bob Good
Bob Latta
Bobby Rush
Bobby Scott
Bonnie Watson Coleman
Brad Finstad
Brad Schneider
Brad Sherman
Brad Wenstrup
Brenda Lawrence
Brendan Boyle
Brett Guthrie
Brian Babin
Brian Fitzpatrick
Brian Higgins
Brian Mast
Brian Schatz
Bruce Westerman
Bryan Steil
Buddy Carter
Burgess Owens
Byron Donalds
Carlos Gimenez
Carol Miller
Carolyn Bourdeaux
Carolyn Maloney
Catherine Cortez Masto
Cathy McMorris Rodgers
Chuck Schumer
Charlie Crist
Chellie Pingree
Cheri Bustos
Chip Roy
Chris Coons
Chris Jacobs
Chris Pappas
Chris Smith
Chris Stewart
Chris Van Hollen
Chrissy Houlahan
Christopher S. Murphy
Chuck Fleischmann
Chuck Grassley
Cindy Axne
Cindy Hyde-Smith
Claudia Tenney
Clay Higgins
Cliff Bentz
Colin Allred
Conor Lamb
Cori Bush
Cory Booker
Cynthia Lummis
Dan Bishop
Dan Crenshaw
Dan Kildee
Dan Meuser
Dan Newhouse
Dan Sullivan
Daniel Webster
Danny K. Davis
Darin LaHood
Darrell Issa
Darren Soto
David Cicilline
David Kustoff
David McKinley
David Joyce
David Price
David Rouzer
David Schweikert
David Scott
David Trone
David Valadao
Dean Phillips
Deb Fischer
Debbie Dingell
Debbie Lesko
Debbie Stabenow
Debbie Wasserman Schultz
Deborah Ross
Derek Kilmer
Diana DeGette
Diana Harshbarger
Dianne Feinstein
Dick Durbin
Dina Titus
Don Bacon
Don Beyer
Don Norcross
Donald Payne Jr.
Donald McEachin
Doris Matsui
Doug LaMalfa
Doug Lamborn
Drew Ferguson
Dusty Johnson
Dutch Ruppersberger
Dwight Evans
Earl Blumenauer
Ed Case
Ed Markey
Ed Perlmutter
Eddie Bernice Johnson
Elaine Luria
Eleanor Holmes Norton
Elise Stefanik
Elissa Slotkin
Elizabeth Warren
Emanuel Cleaver
Eric Swalwell
Frank Lucas
Frank Mrvan
Frank Pallone Jr.
Fred Keller
Fred Upton
Frederica Wilson
French Hill
G.K. Butterfield
Garret Graves
Gary Palmer
Gary Peters
Gerry Connolly
Glenn Grothman
Glenn Thompson
Grace Meng
Grace Napolitano
Greg Murphy
Greg Pence
Greg Stanton
Greg Steube
Gregory Meeks
Gus Bilirakis
Guy Reschenthaler
Gwen Moore
Hakeem Jeffries
Hal Rogers
Haley Stevens
Hank Johnson
Henry Cuellar
Ilhan Omar
Jack Reed
Jackie Speier
Jacky Rosen
Jahana Hayes
Jaime Herrera Beutler
Jake Auchincloss
Jake Ellzey
Jake LaTurner
Jamaal Bowman
James Comer
James Clyburn
Jim Risch
James Lankford
Jim McGovern
Jamie Raskin
Jan Schakowsky
Jared Golden
Jared Huffman
Jason Crow
Jason Smith
Jay Obernolte
Jeanne Shaheen
Jeff Duncan
Jeff Merkley
Jeff Van Drew
Jennifer Wexton
Jenniffer Gonzalez
Jerrold Nadler
Jerry Carl
Jerry McNerney
Jerry Moran
Jesus Garcia
Jim Baird
Jim Banks
Jim Cooper
Jim Costa
Jim Himes
Jim Jordan
Jim Langevin
Jimmy Gomez
Jimmy Panetta
Joaquin Castro
Jodey Arrington
Jody Hice
Joe Courtney
Joe Manchin
Joe Sempolinski
Joe Wilson
John Barrasso
John Bergman
John Boozman
John Carter
John Cornyn
John Curtis
John Garamendi
John Hickenlooper
John Hoeven
John Joyce
John Katko
John Kennedy
John Larson
John Moolenaar
John Rose
John Rutherford
John Sarbanes
John Thune
John Yarmuth
Jon Ossoff
Jon Tester
Joni Ernst
Joseph Morelle
Joe Neguse
Josh Gottheimer
Josh Harder
Josh Hawley
Joyce Beatty
Juan Vargas
Judy Chu
Julia Brownley
Julia Letlow
Kai Kahele
Kamala Harris
Karen Bass
Kat Cammack
Katherine Clark
Kathleen Rice
Kathy Castor
Kathy Manning
Katie Porter
Kay Granger
Kelly Armstrong
Ken Buck
Ken Calvert
Kevin Brady
Kevin Cramer
Kevin Hern
Kevin McCarthy
Kim Schrier
Kirsten Gillibrand
Kurt Schrader
Kweisi Mfume
Kyrsten Sinema
Lance Gooden
Larry Bucshon
Lauren Underwood
Lauren Boebert
Lee Zeldin
Linda SÃ¡nchez
Lindsey Graham
Lisa Blunt Rochester
Lisa McClain
Lisa Murkowski
Liz Cheney
Lizzie Fletcher
Lloyd Doggett
Lloyd Smucker
Lois Frankel
Lori Trahan
Lou Correa
Louis B. Gohmert Jr.
Lucille Roybal-Allard
Lucy McBath
Madeleine Dean
Madison Cawthorn
Maggie Hassan
Marc Veasey
Marco Rubio
Marcy Kaptur
Maria Cantwell
Maria Salazar
Mariannette Miller-Meeks
Marie Newman
Marilyn Strickland
Mario Diaz-Balart
Marjorie Taylor Greene
Mark Amodei
Mark Desaulnier
Mark Green
Mark Kelly
Mark Pocan
Mark Takano
Mark Warner
Markwayne Mullin
Marsha Blackburn
Martin Heinrich
Mary Gay Scanlon
Mary Miller
Mary Peltola
Matt Cartwright
Matt Gaetz
Matt Rosendale
Maxine Waters
Mayra Flores
Mazie K. Hirono
Melanie Stansbury
Michael Bennet
Michael Burgess
Michael Cloud
Michael Guest
Michael McCaul
Michael Turner
Michael San Nicolas
Michael Waltz
Michelle Fischbach
Michelle Steel
Mike Bost
Mike Braun
Mike Carey
Mike Crapo
Mike Rogers
Mike Doyle
Mike Gallagher
Mike Garcia
Mike Johnson
Mike Kelly
Mike Lee
Mike Levin
Mike Quigley
Mike Rounds
Mike Simpson
Mike Thompson
Mikie Sherrill
Mitch McConnell
Mitt Romney
Mo Brooks
Mondaire Jones
Morgan Griffith
Nancy Mace
Nancy Pelosi
Nanette BarragÃ¡n
Neal Dunn
Nicole Malliotakis
Nikema Williams
Norma Torres
Nydia Velazquez
Pat Ryan
Pat Toomey
Pat Fallon
Patrick Leahy
Patrick McHenry
Patty Murray
Paul Gosar
Paul Tonko
Pete Aguilar
Pete Sessions
Pete Stauber
Peter DeFazio
Peter Meijer
Peter Welch
Pramila Jayapal
Raja Krishnamoorthi
Ralph Norman
Rand Paul
Randy Feenstra
Randy Weber
Raphael Warnock
Rashida Tlaib
Raul Grijalva
Raul Ruiz
Richard Blumenthal
Richard Burr
Richard Shelby
Richard Neal
Richard Hudson
Rick Allen
Rick Crawford
Rick Larsen
Rick Scott
Ritchie Torres
Ro Khanna
Rob Portman
Rob Wittman
Robert Aderholt
Bob Menendez
Robin Kelly
Rodney Davis
Roger Marshall
Roger Wicker
Roger Williams
Ron Estes
Ron Johnson
Ron Kind
Ron Wyden
Ronny Jackson
Rosa DeLauro
Roy Blunt
Ruben Gallego
Russ Fulcher
Salud Carbajal
Sam Graves
Sanford Bishop Jr.
Sara Jacobs
Scott Desjarlais
Scott Fitzgerald
Scott Franklin
Scott Perry
Scott Peters
Sean Casten
Sean Maloney
Seth Moulton
Sharice Davids
Sheila Cherfilus-McCormick
Sheila Jackson Lee
Sheldon Whitehouse
Shelley Moore Capito
Sherrod Brown
Shontel Brown
Stacey Plaskett
Steny Hoyer
Stephanie Bice
Stephanie Murphy
Stephen Lynch
Steve Chabot
Steve Cohen
Steve Scalise
Steve Womack
Steve Daines
Steven Horsford
Steven Palazzo
Susan Collins
Susan Wild
Susie Lee
Suzan DelBene
Suzanne Bonamici
Sylvia Garcia
Tammy Baldwin
Tammy Duckworth
Ted Budd
Ted Cruz
Ted Deutch
Ted Lieu
Teresa Leger Fernandez
Terri Sewell
Thom Tillis
Thomas Massie
Tim Burchett
Tim Kaine
Tim Ryan
Tim Scott
Tim Walberg
Tina Smith
Todd Young
Tom Carper
Tom Cole
Tom Cotton
Tom Emmer
Tom Malinowski
Tom McClintock
Tom O'Halleran
Tom Rice
Tom Suozzi
Tom Tiffany
Tommy Tuberville
Tony CÃ¡rdenas
Tony Gonzales
Tracey Mann
Trent Kelly
Trey Hollingsworth
Troy Balderson
Troy Carter
Troy Nehls
Val Demings
Van Taylor
Vernon Buchanan
Veronica Escobar
Vicente Gonzalez
Vicky Hartzler
Victoria Spartz
Virginia Foxx
Warren Davidson
William Timmons
Young Kim
Yvette Clarke
Yvette Herrell
Zoe Lofgren`}]},{name:`Predict and Run: Comparing String Objects`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static boolean[] toBooleanArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    boolean[] data = new boolean[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Boolean.parseBoolean(currentValue);
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about responses to Music & Mental Health Survey
 */
public class MusicSurvey {

  private Respondent[] answers;     // The 1D array of Respondent objects

  /*
   * Reads the data from hoursFile, genreFile, and
   * effectFile to initialize the 1D array answers
   */
  public MusicSurvey(String hoursFile, String genreFile, String effectFile) {
    answers = createRespondents(hoursFile, genreFile, effectFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the
   * data from hoursFile, genreFile, and effectFile
   */
  public Respondent[] createRespondents(String hoursFile, String genreFile, String effectFile) {
    double[] hoursData = FileReader.toDoubleArray(hoursFile);
    String[] genreData = FileReader.toStringArray(genreFile);
    String[] effectData = FileReader.toStringArray(effectFile);
    Respondent[] tempAnswers = new Respondent[hoursData.length];

    for (int index = 0; index < tempAnswers.length; index++) {
      boolean effectResponse = convertToBoolean(effectData[index]);
      tempAnswers[index] = new Respondent(hoursData[index], genreData[index], effectResponse);
    }

    return tempAnswers;
  }

  /*
   * Converts response to a true or false value
   */
  public boolean convertToBoolean(String response) {
    boolean result = false;

    if (response.equals("Improve")) {
      result = true;
    }

    return result;
  }

  /*
   * Returns the Respondent at the specified position from the 1D array answers
   */
  public Respondent getAnswer(int position) {
    return answers[position];
  }

  /*
   * Returns the Respondent that answered targetEffect
   */
  public Respondent getEffectResponse(boolean targetEffect) {
    Respondent result = null;
    
    for (Respondent response : answers) {
      if (response.getEffect() == targetEffect) {
        result = response;
      }
    }

    return result;
  }

  /*
   * Counts the number of times a respondent selected targetGenre
   */
  public int countGenre(String targetGenre) {
    int count = 0;

    for (Respondent response : answers) {
      if (response.getGenre() == targetGenre) {
        count++;
      }
    }

    return count;
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent response : answers) {
      result += response + "\\n----------\\n";
    }

    return result;
  }
}`},{path:`Respondent.java`,text:`/*
 * Represents a survey respondent
 */
public class Respondent {

  private double hours;         // The number of hours per day a respondent listens to music
  private String genre;         // The favorite genre of a respondent
  private boolean effect;       // Whether or not music improves or worsens a respondent's mental health

  /*
   * Sets hours, genre, and effect to the specified values
   */
  public Respondent(double hours, String genre, boolean effect) {
    this.hours = hours;
    this.genre = genre;
    this.effect = effect;
  }

  /*
   * Returns the number of hours per day the respondent listens to music
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns the favorite genre of the respondent
   */
  public String getGenre() {
    return genre;
  }

  /*
   * Returns whether or not music improves or worsens the respondent's mental health
   */
  public boolean getEffect() {
    return effect;
  }

  /*
   * Returns a String containing the respondent's answers
   */
  public String toString() {
    return "Hours Per Day: " + hours + "\\nFavorite Genre: " + genre + "\\nImproves Mental Health: " + effect;
  }
  
}`}],validationFiles:[],dataFiles:[{path:`effect.txt`,text:`No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Worsen
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve`},{path:`genre.txt`,text:`Latin
Rock
Video game music
Jazz
R&B
Jazz
Video game music
K pop
Rock
R&B
Country
EDM
Hip hop
Country
Jazz
Pop
Hip hop
Hip hop
Rap
Hip hop
Hip hop
Rap
R&B
Pop
Rock`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3`}]},{name:`Investigate and Modify: Comparing and Returning #1`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static boolean[] toBooleanArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    boolean[] data = new boolean[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Boolean.parseBoolean(currentValue);
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about responses to Music & Mental Health Survey
 */
public class MusicSurvey {

  private Respondent[] answers;     // The 1D array of Respondent objects

  /*
   * Reads the data from hoursFile, genreFile, and
   * effectFile to initialize the 1D array answers
   */
  public MusicSurvey(String hoursFile, String genreFile, String effectFile) {
    answers = createRespondents(hoursFile, genreFile, effectFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the
   * data from hoursFile, genreFile, and effectFile
   */
  public Respondent[] createRespondents(String hoursFile, String genreFile, String effectFile) {
    double[] hoursData = FileReader.toDoubleArray(hoursFile);
    String[] genreData = FileReader.toStringArray(genreFile);
    String[] effectData = FileReader.toStringArray(effectFile);
    Respondent[] tempAnswers = new Respondent[hoursData.length];

    for (int index = 0; index < tempAnswers.length; index++) {
      boolean effectResponse = convertToBoolean(effectData[index]);
      tempAnswers[index] = new Respondent(hoursData[index], genreData[index], effectResponse);
    }

    return tempAnswers;
  }

  /*
   * Converts response to a true or false value
   */
  public boolean convertToBoolean(String response) {
    boolean result = false;

    if (response.equals("Improve")) {
      result = true;
    }

    return result;
  }

  /*
   * Returns the Respondent at the specified position from the 1D array answers
   */
  public Respondent getAnswer(int position) {
    return answers[position];
  }

  /*
   * Returns the Respondent that answered targetEffect
   */
  public Respondent getEffectResponse(boolean targetEffect) {
    Respondent result = null;
    
    for (Respondent response : answers) {
      if (response.getEffect() == targetEffect) {
        result = response;
      }
    }

    return result;
  }

  /*
   * Counts the number of times a respondent selected targetGenre
   */
  public int countGenre(String targetGenre) {
    int count = 0;

    for (Respondent response : answers) {
      if (response.getGenre() == targetGenre) {
        count++;
      }
    }

    return count;
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent response : answers) {
      result += response + "\\n----------\\n";
    }

    return result;
  }
}`},{path:`Respondent.java`,text:`/*
 * Represents a survey respondent
 */
public class Respondent {

  private double hours;         // The number of hours per day a respondent listens to music
  private String genre;         // The favorite genre of a respondent
  private boolean effect;       // Whether or not music improves or worsens a respondent's mental health

  /*
   * Sets hours, genre, and effect to the specified values
   */
  public Respondent(double hours, String genre, boolean effect) {
    this.hours = hours;
    this.genre = genre;
    this.effect = effect;
  }

  /*
   * Returns the number of hours per day the respondent listens to music
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns the favorite genre of the respondent
   */
  public String getGenre() {
    return genre;
  }

  /*
   * Returns whether or not music improves or worsens the respondent's mental health
   */
  public boolean getEffect() {
    return effect;
  }

  /*
   * Returns a String containing the respondent's answers
   */
  public String toString() {
    return "Hours Per Day: " + hours + "\\nFavorite Genre: " + genre + "\\nImproves Mental Health: " + effect;
  }
  
}`}],validationFiles:[],dataFiles:[{path:`effect.txt`,text:`No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Worsen
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve`},{path:`genre.txt`,text:`Latin
Rock
Video game music
Jazz
R&B
Jazz
Video game music
K pop
Rock
R&B
Country
EDM
Hip hop
Country
Jazz
Pop
Hip hop
Hip hop
Rap
Hip hop
Hip hop
Rap
R&B
Pop
Rock`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3`}]},{name:`Investigate and Modify: Comparing and Returning #2`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static boolean[] toBooleanArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    boolean[] data = new boolean[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Boolean.parseBoolean(currentValue);
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about responses to Music & Mental Health Survey
 */
public class MusicSurvey {

  private Respondent[] answers;     // The 1D array of Respondent objects

  /*
   * Reads the data from hoursFile, genreFile, and
   * effectFile to initialize the 1D array answers
   */
  public MusicSurvey(String hoursFile, String genreFile, String effectFile) {
    answers = createRespondents(hoursFile, genreFile, effectFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the
   * data from hoursFile, genreFile, and effectFile
   */
  public Respondent[] createRespondents(String hoursFile, String genreFile, String effectFile) {
    double[] hoursData = FileReader.toDoubleArray(hoursFile);
    String[] genreData = FileReader.toStringArray(genreFile);
    String[] effectData = FileReader.toStringArray(effectFile);
    Respondent[] tempAnswers = new Respondent[hoursData.length];

    for (int index = 0; index < tempAnswers.length; index++) {
      boolean effectResponse = convertToBoolean(effectData[index]);
      tempAnswers[index] = new Respondent(hoursData[index], genreData[index], effectResponse);
    }

    return tempAnswers;
  }

  /*
   * Converts response to a true or false value
   */
  public boolean convertToBoolean(String response) {
    boolean result = false;

    if (response.equals("Improve")) {
      result = true;
    }

    return result;
  }

  /*
   * Returns the Respondent at the specified position from the 1D array answers
   */
  public Respondent getAnswer(int position) {
    return answers[position];
  }

  /*
   * Returns the Respondent that answered targetEffect
   */
  public Respondent getEffectResponse(boolean targetEffect) {
    Respondent result = null;
    
    for (Respondent response : answers) {
      if (response.getEffect() == targetEffect) {
        result = response;
      }
    }

    return result;
  }

  /*
   * Counts the number of times a respondent selected targetGenre
   */
  public int countGenre(String targetGenre) {
    int count = 0;

    for (Respondent response : answers) {
      if (response.getGenre() == targetGenre) {
        count++;
      }
    }

    return count;
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent response : answers) {
      result += response + "\\n----------\\n";
    }

    return result;
  }
}`},{path:`Respondent.java`,text:`/*
 * Represents a survey respondent
 */
public class Respondent {

  private double hours;         // The number of hours per day a respondent listens to music
  private String genre;         // The favorite genre of a respondent
  private boolean effect;       // Whether or not music improves or worsens a respondent's mental health

  /*
   * Sets hours, genre, and effect to the specified values
   */
  public Respondent(double hours, String genre, boolean effect) {
    this.hours = hours;
    this.genre = genre;
    this.effect = effect;
  }

  /*
   * Returns the number of hours per day the respondent listens to music
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns the favorite genre of the respondent
   */
  public String getGenre() {
    return genre;
  }

  /*
   * Returns whether or not music improves or worsens the respondent's mental health
   */
  public boolean getEffect() {
    return effect;
  }

  /*
   * Returns a String containing the respondent's answers
   */
  public String toString() {
    return "Hours Per Day: " + hours + "\\nFavorite Genre: " + genre + "\\nImproves Mental Health: " + effect;
  }
  
}`}],validationFiles:[],dataFiles:[{path:`effect.txt`,text:`No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Worsen
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve`},{path:`genre.txt`,text:`Latin
Rock
Video game music
Jazz
R&B
Jazz
Video game music
K pop
Rock
R&B
Country
EDM
Hip hop
Country
Jazz
Pop
Hip hop
Hip hop
Rap
Hip hop
Hip hop
Rap
R&B
Pop
Rock`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3`}]},{name:`Investigate and Modify: Comparing and Returning #3`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static boolean[] toBooleanArray(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    boolean[] data = new boolean[temp.size()];

    for (int index = 0; index < data.length; index++) {
      String currentValue = temp.get(index);
      data[index] = Boolean.parseBoolean(currentValue);
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about responses to Music & Mental Health Survey
 */
public class MusicSurvey {

  private Respondent[] answers;     // The 1D array of Respondent objects

  /*
   * Reads the data from hoursFile, genreFile, and
   * effectFile to initialize the 1D array answers
   */
  public MusicSurvey(String hoursFile, String genreFile, String effectFile) {
    answers = createRespondents(hoursFile, genreFile, effectFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the
   * data from hoursFile, genreFile, and effectFile
   */
  public Respondent[] createRespondents(String hoursFile, String genreFile, String effectFile) {
    double[] hoursData = FileReader.toDoubleArray(hoursFile);
    String[] genreData = FileReader.toStringArray(genreFile);
    String[] effectData = FileReader.toStringArray(effectFile);
    Respondent[] tempAnswers = new Respondent[hoursData.length];

    for (int index = 0; index < tempAnswers.length; index++) {
      boolean effectResponse = convertToBoolean(effectData[index]);
      tempAnswers[index] = new Respondent(hoursData[index], genreData[index], effectResponse);
    }

    return tempAnswers;
  }

  /*
   * Converts response to a true or false value
   */
  public boolean convertToBoolean(String response) {
    boolean result = false;

    if (response.equals("Improve")) {
      result = true;
    }

    return result;
  }

  /*
   * Returns the Respondent at the specified position from the 1D array answers
   */
  public Respondent getAnswer(int position) {
    return answers[position];
  }

  /*
   * Returns the Respondent that answered targetEffect
   */
  public Respondent getEffectResponse(boolean targetEffect) {
    Respondent result = null;
    
    for (Respondent response : answers) {
      if (response.getEffect() == targetEffect) {
        result = response;
      }
    }

    return result;
  }

  /*
   * Counts the number of times a respondent selected targetGenre
   */
  public int countGenre(String targetGenre) {
    int count = 0;

    for (Respondent response : answers) {
      if (response.getGenre() == targetGenre) {
        count++;
      }
    }

    return count;
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent response : answers) {
      result += response + "\\n----------\\n";
    }

    return result;
  }
}`},{path:`Respondent.java`,text:`/*
 * Represents a survey respondent
 */
public class Respondent {

  private double hours;         // The number of hours per day a respondent listens to music
  private String genre;         // The favorite genre of a respondent
  private boolean effect;       // Whether or not music improves or worsens a respondent's mental health

  /*
   * Sets hours, genre, and effect to the specified values
   */
  public Respondent(double hours, String genre, boolean effect) {
    this.hours = hours;
    this.genre = genre;
    this.effect = effect;
  }

  /*
   * Returns the number of hours per day the respondent listens to music
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns the favorite genre of the respondent
   */
  public String getGenre() {
    return genre;
  }

  /*
   * Returns whether or not music improves or worsens the respondent's mental health
   */
  public boolean getEffect() {
    return effect;
  }

  /*
   * Returns a String containing the respondent's answers
   */
  public String toString() {
    return "Hours Per Day: " + hours + "\\nFavorite Genre: " + genre + "\\nImproves Mental Health: " + effect;
  }
  
}`}],validationFiles:[],dataFiles:[{path:`effect.txt`,text:`No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Worsen
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve`},{path:`genre.txt`,text:`Latin
Rock
Video game music
Jazz
R&B
Jazz
Video game music
K pop
Rock
R&B
Country
EDM
Hip hop
Country
Jazz
Pop
Hip hop
Hip hop
Rap
Hip hop
Hip hop
Rap
R&B
Pop
Rock`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3`}]},{name:`Practice: Writing Algorithms with 1D Arrays (a)`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    MusicSurvey responses = new MusicSurvey("ages.txt", "streaming.txt");
    System.out.println(responses);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the promptUser() and countStreaming() methods and print the result.
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
  
}`},{path:`MusicSurvey.java`,text:`import java.util.Scanner;

/*
 * Manages data about responses to a survey
 */
public class MusicSurvey {

  private Respondent[] responses;    // The 1D array of Respondent objects

  /*
   * Reads the data from agesFile and streamingFile to initialize responses
   */
  public MusicSurvey(String agesFile, String streamingFile) {
    responses = createResponses(agesFile, streamingFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the data from agesFile and streamingFile
   */
  public Respondent[] createResponses(String agesFile, String streamingFile) {
    int[] agesData = FileReader.toIntArray(agesFile);
    String[] streamingData = FileReader.toStringArray(streamingFile);
    
    Respondent[] tempResponses = new Respondent[streamingData.length];

    for (int index = 0; index < tempResponses.length; index++) {
      tempResponses[index] = new Respondent(agesData[index], streamingData[index]);
    }

    return tempResponses;
  }

  /*
   * Counts the number of times a streaming service appears in the responses
   */
  public int countStreaming(String streamingService) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Count the number of times the parameter streamingService appears in the
     * 1D array responses and return the count.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Prompts the user to enter a streaming service and returns their response
   */
  public String promptUser() {
    /* -------------------------------------- TO DO --------------------------------------
     * ✅ Prompt the user to enter a streaming service to find and return their response.
     * -----------------------------------------------------------------------------------
     */

    
    return "";
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent answer : responses) {
      result += answer + "\\n";
    }

    return result;
  }
  
}`},{path:`Respondent.java`,text:`/*
 * Represents a respondent to a survey
 */
public class Respondent {

  private int age;              // The age of a respondent
  private String streaming;     // The streaming service a respondent uses

  /*
   * Sets age and streaming to the specified values
   */
  public Respondent(int age, String streaming) {
    this.age = age;
    this.streaming = streaming;
  }

  /*
   * Returns the age of the respondent
   */
  public int getAge() {
    return age;
  }

  /*
   * Returns the streaming service the respondent uses
   */
  public String getStreaming() {
    return streaming;
  }

  /*
   * Returns a String containing the age and streaming service of the respondent
   */
  public String toString() {
    return age + " years old: " + streaming;
  }
  
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  String[] testStreaming;
  String[] genres;
  MusicSurvey testMusicSurvey;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testStreaming = FileReader.toStringArray("streaming.txt");
    
    genres = new String[]{"Latin", "Rock", "Video game music", "Jazz", "R&B", "K pop", "Country", "EDM",
                          "Hip hop", "Pop", "Rap", "Classical", "Metal", "Folk", "Lofi", "Gospel"};
    
    testMusicSurvey = new MusicSurvey("ages.txt", "streaming.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Count the number of times the parameter streamingService appears in the survey responses => ")
  public void testCountStreaming() {
    message = "Traverse the 1D array responses and use the getStreaming() method in the Respondent class to check";
    message += "\\n        if the respondent's streaming service matches target. If so, add 1 to a counter variable.";
    message += messageGap;
      
    for (String option : genres) {
      int expected = getExpectedResult(option);
      int actual = testMusicSurvey.countStreaming(option);
      assertEquals(expected, actual, message);
    }
  }

  /*
   * Helper to get the expected result
   */
  private int getExpectedResult(String target) {
    int count = 0;

    for (String value : testStreaming) {
      if (testStreaming.equals(target)) {
        count++;
      }
    }
    
    return count;
  }
  
}`}],dataFiles:[{path:`ages.txt`,text:`18
63
18
61
18
18
18
21
19
18
18
19
17
19
18
17
16
16
17
15
15
17
19
18
16
18
14
18
17
17
20
19
19
17
16
18
21
17
26
20
23
18
37
17
18
17
36
24
18
19
17
16
23
23
22
16
18
42
23
23
19
22
15
25
32
36
24
29
41
36
19
31
16
26
22
16
17
19
18
18
19
19
22
16
19
37
14
33
26
32
30
43
24
36
19
22
20
31
19
15`},{path:`streaming.txt`,text:`Spotify
Pandora
Spotify
YouTube Music
Spotify
Spotify
YouTube Music
Spotify
Spotify
None
Spotify
YouTube Music
Spotify
Spotify
Spotify
Spotify
Spotify
Spotify
Spotify
Spotify
Apple Music
Spotify
None
Spotify
Other
Spotify
Spotify
YouTube Music
Spotify
Apple Music
Apple Music
Spotify
Spotify
Spotify
Spotify
Spotify
Spotify
Other
Other
Spotify
YouTube Music
Spotify
YouTube Music
Spotify
Spotify
None
Spotify
Spotify
Spotify
Spotify
Apple Music
Spotify
Apple Music
Spotify
Spotify
Apple Music
Spotify
None
Spotify
None
YouTube Music
Pandora
Other
Spotify
Spotify
Spotify
Spotify
Spotify
Apple Music
None
YouTube Music
Spotify
Spotify
Spotify
Apple Music
Spotify
Spotify
Spotify
YouTube Music
YouTube Music
Spotify
Spotify
Spotify
Spotify
Spotify
Spotify
Spotify
None
Spotify
Spotify
Spotify
None
Spotify
YouTube Music
YouTube Music
Spotify
Spotify
Spotify
Spotify
Spotify`}]},{name:`Practice: Writing Algorithms with 1D Arrays (b)`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    MusicSurvey responses = new MusicSurvey("hours.txt", "effects.txt");
    System.out.println(responses);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findMinHours() and findMaxHours() methods and print the result.
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about responses to a survey
 */
public class MusicSurvey {

  private Respondent[] responses;    // The 1D array of Respondent objects

  /*
   * Reads the data from hoursFile and effectsFile to initialize responses
   */
  public MusicSurvey(String hoursFile, String effectsFile) {
    responses = createResponses(hoursFile, effectsFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the data from hoursFile and effectsFile
   */
  public Respondent[] createResponses(String hoursFile, String effectsFile) {
    double[] hoursData = FileReader.toDoubleArray(hoursFile);
    String[] effectsData = FileReader.toStringArray(effectsFile);
    
    Respondent[] tempResponses = new Respondent[hoursData.length];

    for (int index = 0; index < tempResponses.length; index++) {
      tempResponses[index] = new Respondent(hoursData[index], effectsData[index]);
    }

    return tempResponses;
  }

  /*
   * Returns the minimum number of hours a respondent listens to music per day
   */
  public double findMinHours() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find and return the smallest number of hours a respondent listens
     * to music each day.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Returns the maximum number of hours a respondent listens to music per day
   */
  public double findMaxHours() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find and return the largest number of hours a respondent listens
     * to music each day.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent answer : responses) {
      result += answer + "\\n";
    }

    return result;
  }
  
}`},{path:`Respondent.java`,text:`/*
 * Represents a respondent to a survey
 */
public class Respondent {

  private double hours;      // The number of hours a respondent listens to music per day
  private String effect;     // The effect music has on a respondent's mental health

  /*
   * Sets hours and effect to the specified values
   */
  public Respondent(double hours, String effect) {
    this.hours = hours;
    this.effect = effect;
  }

  /*
   * Returns the number of hours a respondent listens to music per day
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns the effect music has on the respondent's mental health
   */
  public String getEffect() {
    return effect;
  }

  /*
   * Returns a String containing the hours and effect
   */
  public String toString() {
    return hours + " hours - " + effect;
  }
  
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  double[] testHours;
  MusicSurvey testMusicSurvey;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testHours = FileReader.toDoubleArray("hours.txt");
    testMusicSurvey = new MusicSurvey("hours.txt", "effects.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Find the smallest value for the number of hours a respondent listens to music per day => ")
  public void testFindMinHours() {
    message = "Traverse the 1D array responses and use the getHours() method in the Respondent class to check if the respondent's";
    message += "\\n        number of hours is less than the current minimum value. If so, update the current minimum value to the value found.";
    message += messageGap;
      
    double expected = getExpectedMin();
    double actual = testMusicSurvey.findMinHours();

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Find the largest value for the number of hours a respondent listens to music per day => ")
  public void testFindMaxHours() {
    message = "Traverse the 1D array responses and use the getHours() method in the Respondent class to check if the respondent's";
    message += "\\n        number of hours is greater than the current maximum value. If so, update the current maximum value to the value found.";
    message += messageGap;
      
    double expected = getExpectedMax();
    double actual = testMusicSurvey.findMaxHours();

    assertEquals(expected, actual, message);
  }

  /*
   * Helper to get the expected min
   */
  private double getExpectedMin() {
    double minHours = testHours[0];

    for (double value : testHours) {
      if (value < minHours) {
        minHours = value;
      }
    }
    
    return minHours;
  }

  /*
   * Helper to get the expected max
   */
  private double getExpectedMax() {
    double maxHours = testHours[0];

    for (double value : testHours) {
      if (value > maxHours) {
        maxHours = value;
      }
    }

    return maxHours;
  }
  
}`}],dataFiles:[{path:`effects.txt`,text:`No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Worsen
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
No effect
Improve
Improve
No effect
No effect
Improve
Improve
Improve
No effect
Improve
No effect
Improve
No effect
Improve
Improve
No effect
Improve
Improve
No effect
No effect
Improve
Improve
No effect
No effect
Improve
Improve
Improve
Improve
Improve
No effect
Worsen
Improve
No effect
Improve
Improve
Improve
Improve
No effect
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
No effect
Improve
Improve
No effect
No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
Improve`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3
2
12
6
2
1
5
2
6
4
1
5
4
3
0.5
4
2
5
0.25
3
4
1
1
3
5
3
2
6
1
12
4
2
6
0
7
1
1
1
2
2
5
6
3
4
3
2
3
2
4
2
0.5
5
5
10
2
4
5
3
2
2.5
3
1
7
1
1
3
5
6
4
1
4
10
3
2
4
8`}]},{name:`Practice: Writing Algorithms with 1D Arrays (c)`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    MusicSurvey responses = new MusicSurvey("genres.txt", "effects.txt");
    System.out.println(responses);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the promptUser() and findGenreEffect() methods and print the result.
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
  
}`},{path:`MusicSurvey.java`,text:`import java.util.Scanner;

/*
 * Manages data about responses to a survey
 */
public class MusicSurvey {

  private Respondent[] responses;    // The 1D array of Respondent objects

  /*
   * Reads the data from genreFile and effectFile to initialize responses
   */
  public MusicSurvey(String genreFile, String effectFile) {
    responses = createResponses(genreFile, effectFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the data from genreFile and effectFile
   */
  public Respondent[] createResponses(String genreFile, String effectFile) {
    String[] genreData = FileReader.toStringArray(genreFile);
    String[] effectData = FileReader.toStringArray(effectFile);

    Respondent[] tempResponses = new Respondent[genreData.length];

    for (int index = 0; index < tempResponses.length; index++) {
      tempResponses[index] = new Respondent(genreData[index], effectData[index]);
    }

    return tempResponses;
  }

  /*
   * Finds the genre that matches target and returns true if music
   * improved the respondent's mental health, otherwise false
   */
  public boolean findGenreEffect(String target) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find the genre that matches target and use the getEffectAsBoolean()
     * method to return true if effect is "Improve", otherwise false
     * -----------------------------------------------------------------------------
     */

    
    
    return false;
  }

  /*
   * Prompts the user for a genre and returns their response
   */
  public String promptUser() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Prompt the user to enter a genre and return their response.
     * -----------------------------------------------------------------------------
     */

    
    return "";
  }

  /*
   * Returns true if effect is "Improve", otherwise false
   */
  public boolean getEffectAsBoolean(String effect) {
    boolean result = false;

    if (effect.equals("Improve")) {
      result = true;
    }

    return result;
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent answer : responses) {
      result += answer + "\\n";
    }

    return result;
  }
  
}`},{path:`Respondent.java`,text:`/*
 * Represents a respondent to a survey
 */
public class Respondent {

  private String genre;      // The favorite genre of a respondent
  private String effect;     // The effect music had on a respondent's mental health

  /*
   * Sets genre and effect to the specified values
   */
  public Respondent(String genre, String effect) {
    this.genre = genre;
    this.effect = effect;
  }

  /*
   * Returns the favorite genre of the respondent
   */
  public String getGenre() {
    return genre;
  }

  /*
   * Returns the effect music had on the respondent's mental health
   */
  public String getEffect() {
    return effect;
  }

  /*
   * Returns a String containing the genre and effect
   */
  public String toString() {
    return genre + " - " + effect;
  }
  
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  String[] testGenres;
  String[] testEffects;
  String[] genres;
  MusicSurvey testMusicSurvey;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testGenres = FileReader.toStringArray("genres.txt");
    testEffects = FileReader.toStringArray("effects.txt");
    
    genres = new String[]{"Latin", "Rock", "Video game music", "Jazz", "R&B", "K pop", "Country", "EDM",
                          "Hip hop", "Pop", "Rap", "Classical", "Metal", "Folk", "Lofi", "Gospel"};
    
    testMusicSurvey = new MusicSurvey("genres.txt", "effects.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Find the genre that matches the parameter target and return whether or not music improved mental health => ")
  public void testFindGenreEffect() {
    message = "Traverse the 1D array responses and use the getGenre() method in the Respondent class to check";
    message += "\\n        if the respondent's favorite genre matches target. If so, call the getEffect() method in the Respondent";
    message += "\\n        class inside a call to the getEffectAsBoolean() method in the MusicSurvey class to return true or false.";
    message += messageGap;
      
    for (String option : genres) {
      boolean expected = getExpectedResult(option);
      boolean actual = testMusicSurvey.findGenreEffect(option);
      assertEquals(expected, actual, message);
    }
  }

  /*
   * Helper to get the expected result
   */
  private boolean getExpectedResult(String target) {
    for (int index = 0; index < testGenres.length; index++) {
      if (testGenres[index].equals(target)) {
        return testMusicSurvey.getEffectAsBoolean(testEffects[index]);
      }
    }
    
    return false;
  }
  
}`}],dataFiles:[{path:`effects.txt`,text:`No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Worsen
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
No effect
Improve
Improve
No effect
No effect
Improve
Improve
Improve
No effect
Improve
No effect
Improve
No effect
Improve
Improve
No effect
Improve
Improve
No effect
No effect
Improve
Improve
No effect
No effect
Improve
Improve
Improve
Improve
Improve
No effect
Worsen
Improve
No effect
Improve
Improve
Improve
Improve
No effect
Improve
Improve
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
No effect
Improve
Improve
No effect
Improve
Improve
No effect
No effect
No effect
No effect
Improve
Improve
Improve
Improve
Improve
Improve
Improve
Improve`},{path:`genres.txt`,text:`Latin
Rock
Video game music
Jazz
R&B
Jazz
Video game music
K pop
Rock
R&B
Country
EDM
Hip hop
Country
Jazz
Pop
Hip hop
Hip hop
Rap
Hip hop
Hip hop
Rap
R&B
Pop
Rock
Pop
Rock
Pop
Pop
Pop
Rock
Classical
Metal
Rock
Classical
Pop
Pop
Pop
Rock
EDM
Video game music
Rock
Video game music
Classical
Metal
Folk
Classical
Rock
Metal
Pop
EDM
K pop
Rap
Pop
Rock
Classical
Rock
Rock
Folk
Video game music
K pop
Rock
Rock
Metal
Rock
Metal
Rock
Hip hop
Metal
Rock
Rock
Rock
Rock
Hip hop
Hip hop
Jazz
Metal
R&B
EDM
Hip hop
Rock
Jazz
K pop
Rock
R&B
Rock
Jazz
Metal
R&B
Hip hop
Metal
Video game music
Classical
Rock
Video game music
Metal
EDM
Rock
Metal
Rock`}]},{name:`Practice: Writing Algorithms with 1D Arrays (d)`,lesson:`Lesson 11: Array Algorithms`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    MusicSurvey responses = new MusicSurvey("times.txt", "ages.txt", "hours.txt");
    System.out.println("Original Order\\n--------------------");
    System.out.println(responses);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the reverseResponses() method, then print the MusicSurvey object.
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
  
}`},{path:`MusicSurvey.java`,text:`/*
 * Manages data about responses to a survey
 */
public class MusicSurvey {

  private Respondent[] responses;    // The 1D array of Respondent objects

  /*
   * Reads the data from timesFile, agesFile, and
   * hoursFile to initialize responses
   */
  public MusicSurvey(String timesFile, String agesFile, String hoursFile) {
    responses = createResponses(timesFile, agesFile, hoursFile);
  }

  /*
   * Returns a 1D array of Respondent objects using the
   * data from timesFile, agesFile, and hoursFile
   */
  public Respondent[] createResponses(String timesFile, String agesFile, String hoursFile) {
    String[] timesData = FileReader.toStringArray(timesFile);
    int[] agesData = FileReader.toIntArray(agesFile);
    double[] hoursData = FileReader.toDoubleArray(hoursFile);

    Respondent[] tempResponses = new Respondent[timesData.length];

    for (int index = 0; index < tempResponses.length; index++) {
      tempResponses[index] = new Respondent(timesData[index], agesData[index], hoursData[index]);
    }

    return tempResponses;
  }

  /*
   * Returns the 1D array of Respondent objects
   */
  public Respondent[] getResponses() {
    return responses;
  }

  /*
   * Reverses the elements in the 1D array responses
   */
  public void reverseResponses() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Reverse the elements in the 1D array responses so that the newest
     * response is first and the oldest response is last.
     * -----------------------------------------------------------------------------
     */

    
  }

  /*
   * Returns a String containing each respondent's information
   */
  public String toString() {
    String result = "";

    for (Respondent answer : responses) {
      result += answer + "\\n";
    }

    return result;
  }
  
}`},{path:`Respondent.java`,text:`/*
 * Represents a respondent to a survey
 */
public class Respondent {

  private String time;      // The date and time a respondent answered the survey
  private int age;          // The age of a respondent
  private double hours;     // The number of hours a respondent listens to music each day

  /*
   * Sets time, age, and hours to the specified values
   */
  public Respondent(String time, int age, double hours) {
    this.time = time;
    this.age = age;
    this.hours = hours;
  }

  /*
   * Returns the date and time the respondent answered the survey
   */
  public String getTime() {
    return time;
  }

  /*
   * Returns the age of the respondent
   */
  public int getAge() {
    return age;
  }

  /*
   * Returns the number of hours the respondent listens to music each day
   */
  public double getHours() {
    return hours;
  }

  /*
   * Returns a String containing the date and time, age, and hours
   */
  public String toString() {
    return "[" + time + "] " + age + " years old - " + hours + " hours per day";
  }
}`}],validationFiles:[{path:`MusicSurveyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MusicSurvey.java Test")
public class MusicSurveyTest {

  MusicSurvey testMusicSurvey;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testMusicSurvey = new MusicSurvey("times.txt", "ages.txt", "hours.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Reverse the elements in the 1D array responses => ")
  public void testReverseResponses() {
    message = "Reverse the elements in the 1D array responses by swapping the first element with the last";
    message += "\\n        element, the second element with the next to last element, and so on.";
    message += messageGap;
      
    Respondent[] expected = getExpectedResult();
    testMusicSurvey.reverseResponses();
    Respondent[] actual = testMusicSurvey.getResponses();

    assertEquals(Arrays.toString(expected), Arrays.toString(actual), message);
  }

  /*
   * Helper to get the expected result
   */
  private Respondent[] getExpectedResult() {
    String[] testTimes = FileReader.toStringArray("times.txt");
    int[] testAges = FileReader.toIntArray("ages.txt");
    double[] testHours = FileReader.toDoubleArray("hours.txt");

    Respondent[] temp = new Respondent[testTimes.length];
    int end = temp.length - 1;

    for (int index = 0; index < temp.length; index++) {
      temp[end] = new Respondent(testTimes[index], testAges[index], testHours[index]);
      end--;
    }

    return temp;
  }
  
}`}],dataFiles:[{path:`ages.txt`,text:`18
63
18
61
18
18
18
21
19
18
18
19
17
19
18
17
16
16
17
15
15
17
19
18
16
18
14
18
17
17
20
19
19
17
16
18
21
17
26
20
23
18
37
17
18
17
36
24
18
19
17
16
23
23
22
16
18
42
23
23
19
22
15
25
32
36
24
29
41
36
19
31
16
26
22
16
17
19
18
18
19
19
22
16
19
37
14
33
26
32
30
43
24
36
19
22
20
31
19
15`},{path:`hours.txt`,text:`3
1.5
4
2.5
4
5
3
1
6
1
3
8
3
2
4
2
8
12
24
3
8
4
5
2
3
2
12
6
2
1
5
2
6
4
1
5
4
3
0.5
4
2
5
0.25
3
4
1
1
3
5
3
2
6
1
12
4
2
6
0
7
1
1
1
2
2
5
6
3
4
3
2
3
2
4
2
0.5
5
5
10
2
4
5
3
2
2.5
3
1
7
1
1
3
5
6
4
1
4
10
3
2
4
8`},{path:`times.txt`,text:`08/27 @ 07:29
08/27 @ 07:57
08/27 @ 09:28
08/27 @ 09:40
08/27 @ 09:54
08/27 @ 09:56
08/27 @ 10:00
08/27 @ 10:18
08/27 @ 10:33
08/27 @ 10:44
08/27 @ 10:51
08/27 @ 11:00
08/27 @ 11:04
08/27 @ 11:12
08/27 @ 11:16
08/27 @ 11:19
08/27 @ 11:39
08/27 @ 11:39
08/27 @ 11:40
08/27 @ 11:41
08/27 @ 11:43
08/28 @ 12:28
08/28 @ 01:39
08/28 @ 03:19
08/28 @ 04:13
08/28 @ 04:38
08/28 @ 04:40
08/28 @ 05:05
08/28 @ 05:16
08/28 @ 08:36
08/28 @ 10:30
08/28 @ 10:38
08/28 @ 10:54
08/28 @ 10:59
08/28 @ 11:08
08/28 @ 11:13
08/28 @ 11:25
08/28 @ 11:27
08/28 @ 11:36
08/28 @ 11:39
08/28 @ 11:50
08/28 @ 11:54
08/28 @ 11:55
08/28 @ 11:58
08/28 @ 12:08
08/28 @ 12:12
08/28 @ 12:15
08/28 @ 12:23
08/28 @ 12:30
08/28 @ 12:32
08/28 @ 12:37
08/28 @ 12:39
08/28 @ 12:52
08/28 @ 12:53
08/28 @ 12:54
08/28 @ 01:01
08/28 @ 01:05
08/28 @ 01:08
08/28 @ 01:09
08/28 @ 01:18
08/28 @ 01:22
08/28 @ 01:22
08/28 @ 01:34
08/28 @ 01:34
08/28 @ 01:42
08/28 @ 01:43
08/28 @ 01:46
08/28 @ 01:48
08/28 @ 01:55
08/28 @ 01:57
08/28 @ 01:59
08/28 @ 02:00
08/28 @ 02:01
08/28 @ 02:02
08/28 @ 02:04
08/28 @ 02:04
08/28 @ 02:07
08/28 @ 02:08
08/28 @ 02:10
08/28 @ 02:10
08/28 @ 02:12
08/28 @ 02:14
08/28 @ 02:18
08/28 @ 02:24
08/28 @ 02:25
08/28 @ 02:26
08/28 @ 02:29
08/28 @ 02:30
08/28 @ 02:38
08/28 @ 02:41
08/28 @ 02:45
08/28 @ 02:47
08/28 @ 02:49
08/28 @ 02:51
08/28 @ 02:58
08/28 @ 02:59
08/28 @ 03:02
08/28 @ 03:03
08/28 @ 03:03
08/28 @ 03:07`}]},{name:`Predict and Run: Nested Loops`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Patterns.java`,text:`public class Patterns {

  public void printMessage() {
    for (int outer = 1; outer <= 3; outer++) {
      System.out.println("Countdown " + outer);
      
      for (int inner = 10; inner > 0; inner--) {
        System.out.println(inner);
      }

      System.out.println("I love CS!\\n");
    }
  }

  public void printSquare(int size) {
    System.out.println("---------- " + size + "x" + size + " Square ----------\\n");
    
    for (int outer = 1; outer <= size; outer++) {
      for (int inner = 1; inner <= size; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printTriangle(int height) {
    System.out.println("---------- " + height + " Tall Triangle ----------\\n");
    
    for (int outer = 1; outer <= height; outer++) {
      for (int inner = 1; inner <= outer; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printArray(int[] numbers) {
    for (int index = 0; index < numbers.length; index++) {
      for (int next = index + 1; next < numbers.length; next++) {
        System.out.println("Outer: " + numbers[index] + ", Inner: " + numbers[next]);
      }

      System.out.println();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Nested Loops #1`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Patterns.java`,text:`public class Patterns {

  public void printMessage() {
    for (int outer = 1; outer <= 3; outer++) {
      System.out.println("Countdown " + outer);
      
      for (int inner = 10; inner > 0; inner--) {
        System.out.println(inner);
      }

      System.out.println("I love CS!\\n");
    }
  }

  public void printSquare(int size) {
    System.out.println("---------- " + size + "x" + size + " Square ----------\\n");
    
    for (int outer = 1; outer <= size; outer++) {
      for (int inner = 1; inner <= size; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printTriangle(int height) {
    System.out.println("---------- " + height + " Tall Triangle ----------\\n");
    
    for (int outer = 1; outer <= height; outer++) {
      for (int inner = 1; inner <= outer; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printArray(int[] numbers) {
    for (int index = 0; index < numbers.length; index++) {
      for (int next = index + 1; next < numbers.length; next++) {
        System.out.println("Outer: " + numbers[index] + ", Inner: " + numbers[next]);
      }

      System.out.println();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Nested Loops #2`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Patterns.java`,text:`public class Patterns {

  public void printMessage() {
    for (int outer = 1; outer <= 3; outer++) {
      System.out.println("Countdown " + outer);
      
      for (int inner = 10; inner > 0; inner--) {
        System.out.println(inner);
      }

      System.out.println("I love CS!\\n");
    }
  }

  public void printSquare(int size) {
    System.out.println("---------- " + size + "x" + size + " Square ----------\\n");
    
    for (int outer = 1; outer <= size; outer++) {
      for (int inner = 1; inner <= size; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printTriangle(int height) {
    System.out.println("---------- " + height + " Tall Triangle ----------\\n");
    
    for (int outer = 1; outer <= height; outer++) {
      for (int inner = 1; inner <= outer; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printArray(int[] numbers) {
    for (int index = 0; index < numbers.length; index++) {
      for (int next = index + 1; next < numbers.length; next++) {
        System.out.println("Outer: " + numbers[index] + ", Inner: " + numbers[next]);
      }

      System.out.println();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Nested Loops #3`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Patterns.java`,text:`public class Patterns {

  public void printMessage() {
    for (int outer = 1; outer <= 3; outer++) {
      System.out.println("Countdown " + outer);
      
      for (int inner = 10; inner > 0; inner--) {
        System.out.println(inner);
      }

      System.out.println("I love CS!\\n");
    }
  }

  public void printSquare(int size) {
    System.out.println("---------- " + size + "x" + size + " Square ----------\\n");
    
    for (int outer = 1; outer <= size; outer++) {
      for (int inner = 1; inner <= size; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printTriangle(int height) {
    System.out.println("---------- " + height + " Tall Triangle ----------\\n");
    
    for (int outer = 1; outer <= height; outer++) {
      for (int inner = 1; inner <= outer; inner++) {
        System.out.print("* ");
      }

      System.out.println();
    }
  }

  public void printArray(int[] numbers) {
    for (int index = 0; index < numbers.length; index++) {
      for (int next = index + 1; next < numbers.length; next++) {
        System.out.println("Outer: " + numbers[index] + ", Inner: " + numbers[next]);
      }

      System.out.println();
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Finding Duplicates (a)`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Forbes companies = new Forbes("names.txt", "years.txt");
    System.out.println(companies);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the checkForDuplicates() method and print the result.
     * -----------------------------------------------------------------------------
     */
   
    
    
  }
}`},{path:`Company.java`,text:`/*
 * Represents a company
 */
public class Company {

  private String name;        // The name of a company
  private int yearFounded;    // The year a company was founded

  /*
   * Sets name and yearFounded to the specified values
   */
  public Company(String name, int yearFounded) {
    this.name = name;
    this.yearFounded = yearFounded;
  }

  /*
   * Returns the name of the company
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the year the company was founded
   */
  public int getYearFounded() {
    return yearFounded;
  }

  /*
   * Returns a String containing the name of the
   * company and the year it was founded
   */
  public String toString() {
    return name + " - Founded in " + yearFounded;
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
  
}`},{path:`Forbes.java`,text:`/*
 * Manages data about Forbes 2000 Global Companies
 */
public class Forbes {

  private Company[] companyData;    // The 1D array of Company objects

  /*
   * Reads the data from namesFile and
   * yearsFile to initialize companyData
   */
  public Forbes(String namesFile, String yearsFile) {
    companyData = createCompanies(namesFile, yearsFile);
  }

  /*
   * Returns a 1D array of Company objects using
   * the data from namesFile and yearsFile
   */
  public Company[] createCompanies(String namesFile, String yearsFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    int[] yearsData = FileReader.toIntArray(yearsFile);

    Company[] tempCompanies = new Company[namesData.length];

    for (int index = 0; index < tempCompanies.length; index++) {
      tempCompanies[index] = new Company(namesData[index], yearsData[index]);
    }

    return tempCompanies;
  }

  /*
   * Returns true if any companies in companyData were
   * founded the same year, otherwise returns false
   */
  public boolean checkForDuplicates() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use nested loops to traverse the 1D array companyData and the getYearFounded()
     * method in the Company class to check if there are duplicate years.
     * -----------------------------------------------------------------------------
     */

    
    
    return false;
  }

  /*
   * Returns a String containing each company's information
   */
  public String toString() {
    String result = "";

    for (Company org : companyData) {
      result += org + "\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`ForbesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Forbes.java Test")
public class ForbesTest {

  Forbes testForbes;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testForbes = new Forbes("names.txt", "years.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Return true if there are duplicate years in the 1D array companyData => ")
  public void testCheckForDuplicates() {
    message = "Use nested loops to check if a company was founded the same year as another company";
    message += "\\n        in the 1D array companyData. Return true inside the loop once a duplicate is found.";
    message += messageGap;
      
    assertTrue(testForbes.checkForDuplicates(), message);
  }
   
}`}],dataFiles:[{path:`names.txt`,text:`Berkshire Hathaway
JPMorgan Chase
Amazon
Apple
Bank of America
Alphabet
Microsoft
ExxonMobil
Wells Fargo
Verizon Communications
AT&T
UnitedHealth Group
Walmart
Chevron
Citigroup
Comcast
Meta Platforms
Morgan Stanley
Goldman Sachs Group
Johnson & Johnson
CVS Health
Pfizer
Intel
Procter & Gamble
AbbVie
General Motors
MetLife
American Express
Cigna
Prudential Financial
PepsiCo
Merck & Co.
Elevance Health
American International Group
Cisco Systems
Walt Disney
IBM
United Parcel Service
Raytheon Technologies
Capital One
Bristol Myers Squibb
Coca-Cola
Oracle
Thermo Fisher Scientific
Costco Wholesale
Charter Communications
ConocoPhillips
US Bancorp
Abbott Laboratories
Caterpillar`},{path:`years.txt`,text:`1939
2000
1994
1976
1998
1998
1975
1999
1852
1983
1876
1977
1962
1879
1998
1963
2004
1924
1869
1886
1963
1849
1968
1837
2013
1908
1868
1850
1792
1875
1965
1891
2004
1919
1984
1923
1911
1907
2020
1994
1933
1892
1977
1956
1983
1993
1875
1929
1888
1925`}]},{name:`Practice: Finding Duplicates (b)`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    MovieDatabase movies = new MovieDatabase("names.txt", "runtimes.txt", "ratings.txt");
    System.out.println(movies);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the checkForDuplicates() method and print the result.
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
  
}`},{path:`Movie.java`,text:`/*
 * Represents a movie
 */
public class Movie {

  private String name;     // The name of a movie
  private int runtime;     // The length of a movie in minutes
  private double rating;   // The rating of a movie

  /*
   * Sets name, runtime, and rating to the specified values
   */
  public Movie(String name, int runtime, double rating) {
    this.name = name;
    this.runtime = runtime;
    this.rating = rating;
  }

  /*
   * Returns the name of the movie
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the runtime of the movie
   */
  public int getRuntime() {
    return runtime;
  }

  /*
   * Returns the rating of the movie
   */
  public double getRating() {
    return rating;
  }

  /*
   * Returns a String containing the name, runtime, and rating of the movie
   */
  public String toString() {
    return name + " (" + runtime + " minutes) - " + rating;
  }
}`},{path:`MovieDatabase.java`,text:`/*
 * Manages data about movies from The Movie Database
 */
public class MovieDatabase {

  private Movie[] movieData;    // The 1D array of Movie objects

  /*
   * Reads the data from nameFile, runtimeFile, and
   * ratingFile to initialize movieData
   */
  public MovieDatabase(String nameFile, String runtimeFile, String ratingFile) {
    movieData = createMovies(nameFile, runtimeFile, ratingFile);
  }

  /*
   * Returns a 1D array of Movie objects using the
   * data from nameFile, runtimeFile, and ratingFile
   */
  public Movie[] createMovies(String nameFile, String runtimeFile, String ratingFile) {
    String[] namesData = FileReader.toStringArray(nameFile);
    int[] runtimeData = FileReader.toIntArray(runtimeFile);
    double[] ratingsData = FileReader.toDoubleArray(ratingFile);

    Movie[] tempMovies = new Movie[namesData.length];

    for (int index = 0; index < tempMovies.length; index++) {
      tempMovies[index] = new Movie(namesData[index], runtimeData[index], ratingsData[index]);
    }

    return tempMovies;
  }

  /*
   * Returns true if any movie in movieData have the same runtime, otherwise returns false
   */
  public boolean checkForDuplicates() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use nested loops to traverse the 1D array movieData and the getRuntime()
     * method in the Movie class to check if there are duplicate runtimes.
     * -----------------------------------------------------------------------------
     */

       

    return false;
  }

  /*
   * Returns a String containing each movie's information
   */
  public String toString() {
    String result = "";

    for (Movie theMovie : movieData) {
      result += theMovie + "\\n";
    }

    return result;
  }
}`}],validationFiles:[{path:`MovieDatabaseTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MovieDatabase.java Test")
public class MovieDatabaseTest {

  MovieDatabase testMovieDatabase;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testMovieDatabase = new MovieDatabase("names.txt", "runtimes.txt", "ratings.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Return true if there are duplicate runtimes in the 1D array movieData => ")
  public void testCheckForDuplicates() {
    message = "Use nested loops to check if a movie has the same runtime as another movie";
    message += "\\n        in the 1D array movieData. Return true inside the loop once a duplicate is found.";
    message += messageGap;
      
    assertTrue(testMovieDatabase.checkForDuplicates(), message);
  }
   
}`}],dataFiles:[{path:`names.txt`,text:`Jurassic World
Mad Max: Fury Road
Insurgent
Star Wars: The Force Awakens
Furious 7
The Revenant
Terminator Genisys
The Martian
Minions
Inside Out
Spectre
Jupiter Ascending
Ex Machina
Pixels
Avengers: Age of Ultron
The Hateful Eight
Taken 3
Ant-Man
The Hunger Games: Mockingjay - Part 2
Tomorrowland
San Andreas
The Big Short
Mission: Impossible - Rogue Nation
Kingsman: The Secret Service
Spotlight
Maze Runner: The Scorch Trials
Chappie
Goosebumps
Room
The Good Dinosaur
Brooklyn
Straight Outta Compton
The Last Witch Hunter
The Age of Adaline
Hardcore Henry
Home
The Lobster
Carol
The Intern
Everest
Point Break
Burnt
Self/less
Mortdecai
Blackhat
Fantastic Four
The Witch
Spy
Focus
Hitman: Agent 47`},{path:`ratings.txt`,text:`6.5
7.1
6.3
7.5
7.3
7.2
5.8
7.6
6.5
8
6.2
5.2
7.6
5.8
7.4
7.4
6.1
7
6.5
6.2
6.1
7.3
7.1
7.6
7.8
6.4
6.6
6.2
8
6.6
7.3
7.7
5.6
7.3
5.9
6.9
6.6
7.1
7.1
6.7
5.5
6.3
6.2
5.3
5
4.4
5.8
6.9
6.7
5.5`},{path:`runtimes.txt`,text:`124
120
119
136
137
156
125
141
91
94
148
124
108
105
141
167
109
115
136
130
114
130
131
130
128
132
120
103
117
93
111
147
106
112
97
94
118
118
121
121
114
100
116
106
133
100
92
120
105
96`}]},{name:`Practice: Finding Duplicates (c)`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Pokedex pokemon = new Pokedex("names.txt", "attacks.txt");
    System.out.println(pokemon);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the checkForDuplicates() method and print the result.
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
  
}`},{path:`Pokedex.java`,text:`/*
 * Manages data about Pokemon
 */
public class Pokedex {

  private Pokemon[] pokemonData;   // The 1D array of Pokemon objects

  /*
   * Reads the data from namesFile and attackFile
   * to initialize pokemonData
   */
  public Pokedex(String namesFile, String attackFile) {
    pokemonData = createPokemon(namesFile, attackFile);
  }

  /*
   * Returns a 1D array of Pokemon objects using
   * the data from namesFile and attackFile
   */
  public Pokemon[] createPokemon(String namesFile, String attackFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    int[] attackData = FileReader.toIntArray(attackFile);

    Pokemon[] tempPokemon = new Pokemon[namesData.length];

    for (int index = 0; index < tempPokemon.length; index++) {
      tempPokemon[index] = new Pokemon(namesData[index], attackData[index]);
    }

    return tempPokemon;
  }

  /*
   * Returns true if more than one Pokemon have the same attack points, otherwise returns false
   */
  public boolean checkForDuplicates() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use nested loops to traverse the 1D array pokemonData and the getAttack()
     * method in the Pokemon class to check if there are duplicate attack points.
     * -----------------------------------------------------------------------------
     */

    
    
    return false;
  }

  /*
   * Returns a String containing each Pokemon's information
   */
  public String toString() {
    String result = "";

    for (Pokemon thePokemon : pokemonData) {
      result += thePokemon + "\\n";
    }

    return result;
  }
  
}`},{path:`Pokemon.java`,text:`/*
 * Represents a Pokemon
 */
public class Pokemon {

  private String name;    // The name of a Pokemon
  private int attack;     // The attack of a Pokemon

  /*
   * Sets name and attack to the specified values
   */
  public Pokemon(String name, int attack) {
    this.name = name;
    this.attack = attack;
  }

  /*
   * Returns the name of the Pokemon
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the attack of the Pokemon
   */
  public int getAttack() {
    return attack;
  }

  /*
   * Returns a String containing the name and attack of the Pokemon
   */
  public String toString() {
    return name + ": " + attack;
  }
  
}`}],validationFiles:[{path:`PokedexTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Pokedex.java Test")
public class PokedexTest {

  Pokedex testPokedex;
  String message;
  String messageGap = "\\n       ";
   
  @BeforeEach
  public void setup() {
    testPokedex = new Pokedex("names.txt", "attacks.txt");
  }
   
  @Test
  @Order(1)
  @DisplayName("Return true if there are duplicate attack points in the 1D array pokemonData => ")
  public void testCheckForDuplicates() {
    message = "Use nested loops to check if a Pokemon has the same attack points as another Pokemon";
    message += "\\n        in the 1D array pokemonData. Return true inside the loop once a duplicate is found.";
    message += messageGap;
      
    assertTrue(testPokedex.checkForDuplicates(), message);
  }
   
}`}],dataFiles:[{path:`attacks.txt`,text:`49
62
82
100
52
64
84
104
130
48
63
83
103
30
20
45
35
25
90
150
45
60
80
80
56
56
81
71
60
90
60
95
55
90
85
75
75
100
100
47
62
92
57
72
102
45
70
41
41
76
67
45
70
45
80
50
65
80
70
95
55
65
55
55`},{path:`names.txt`,text:`Bulbasaur
Ivysaur
Venusaur
Mega Venusaur
Charmander
Charmeleon
Charizard
Mega Charizard Y
Mega Charizard X
Squirtle
Wartortle
Blastoise
Mega Blastoise
Caterpie
Metapod
Butterfree
Weedle
Kakuna
Beedrill
Mega Beedrill
Pidgey
Pidgeotto
Pidgeot
Mega Pidgeot
Rattata
Alolan Rattata
Raticate
Alolan Raticate
Spearow
Fearow
Ekans
Arbok
Pikachu
Raichu
Alolan Raichu
Sandshrew
Alolan Sandshrew
Sandslash
Alolan Sandslash
Nidoran Female
Nidorina
Nidoqueen
Nidoran Male
Nidorino
Nidoking
Clefairy
Clefable
Vulpix
Alolan Vulpix
Ninetales
Alolan Ninetales
Jigglypuff
Wigglytuff
Zubat
Golbat
Oddish
Gloom
Vileplume
Paras
Parasect
Venonat
Venomoth
Diglett
Alolan Diglett`}]},{name:`Practice: Finding Duplicates (d)`,lesson:`Lesson 12: Finding Duplicates`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    HeroManager heroes = new HeroManager("names.txt", "publishers.txt");
    System.out.println(heroes);

    /* ----------------------------------- TO DO #2---------------------------------
     * ✅ Call the checkForDuplicates() method and print the result.
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
  
}`},{path:`HeroManager.java`,text:`/*
 * Manages data about superheros
 */
public class HeroManager {

  private Superhero[] heroData;    // The 1D array of Superhero objects

  /*
   * Reads the data from namesFile and
   * publisherFile to initialize heroData
   */
  public HeroManager(String namesFile, String publisherFile) {
    heroData = createSuperheroes(namesFile, publisherFile);
  }

  /*
   * Returns a 1D array of Superhero objects using
   * the data from namesFile and publisherFile
   */
  public Superhero[] createSuperheroes(String namesFile, String publisherFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    String[] publisherData = FileReader.toStringArray(publisherFile);

    Superhero[] tempHeroes = new Superhero[namesData.length];

    for (int index = 0; index < tempHeroes.length; index++) {
      tempHeroes[index] = new Superhero(namesData[index], publisherData[index]);
    }

    return tempHeroes;
  }

  /*
   * Returns true if more than one superhero has the same publisher, otherwise returns false
   */
  public boolean checkForDuplicates() {
    /* ----------------------------------- TO DO #1 --------------------------------
     * ✅ Use nested loops to traverse the 1D array heroData and the getPublisher()
     * method in the Superhero class to check if there are duplicate publishers.
     * -----------------------------------------------------------------------------
     */


    
    
    return false;
  }

  /*
   * Returns a String containing each hero's information
   */
  public String toString() {
    String result = "";

    for (Superhero hero : heroData) {
      result += hero + "\\n";
    }

    return result;
  }
}`},{path:`Superhero.java`,text:`/*
 * Represents a superhero
 */
public class Superhero {

  private String name;        // The name of a superhero
  private String publisher;   // The name of the publisher of a superhero

  /*
   * Sets name and publisher to the specified values
   */
  public Superhero(String name, String publisher) {
    this.name = name;
    this.publisher = publisher;
  }

  /*
   * Returns the name of the superhero
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the publisher of the superhero
   */
  public String getPublisher() {
    return publisher;
  }

  /*
   * Returns a String containing the name and publisher
   */
  public String toString() {
    return name + " (" + publisher + ")";
  }
  
}`}],validationFiles:[{path:`HeroManagerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("HeroManager.java Test")
public class HeroManagerTest {

  private HeroManager heroManagerWithDuplicates;
  String message;
  String messageGap = "\\n       ";

  @BeforeEach
  public void setup() {
    heroManagerWithDuplicates = new HeroManager("testNames.txt", "testPublishersWithDuplicates.txt");
  }

  @Test
  @Order(1)
  @DisplayName("Return true if any superheroes have the same publisher")
  public void testCheckForDuplicatesWithDuplicates() {
    message = "Use nested loops to check if a superhero has the same publisher as another superhero";
    message += "\\n        in the 1D array heroData. Return true inside the loop once a duplicate is found.";
    message += messageGap;
    
    assertTrue(heroManagerWithDuplicates.checkForDuplicates(), message);
  }

}
`}],dataFiles:[{path:`names.txt`,text:`Alien
Killer Frost
Mystique
Nebula
Abe Sapien
Dr Manhattan
Shadow Lass
Beast
Archangel
Copycat
Lobo
Nova
Ardina
Living Tribunal
K-2SO
Poison Ivy
Brainiac
Swamp Thing
Killer Croc
Greedo
Evilhawk
Martian Manhunter
Beast Boy
Yoda
Donatello
Leonardo
Raphael
Michelangelo
Drax the Destroyer
Triton
Fin Fang Foom
Man-Thing
Hulk
Mantis
Gamora
Toad
Darkseid
Tiger Shark
Apocalypse
Godzilla
Century
Starfire
Jar Jar Binks
Kilowog
Blink
Purple Man
Thanos
Gladiator
Trigon
Klaw`},{path:`publishers.txt`,text:`Dark Horse Comics
DC Comics
Marvel Comics
Marvel Comics
Dark Horse Comics
DC Comics
DC Comics
Marvel Comics
Marvel Comics
Marvel Comics
DC Comics
Marvel Comics
Marvel Comics
Marvel Comics
George Lucas
DC Comics
DC Comics
DC Comics
DC Comics
George Lucas
Marvel Comics
DC Comics
DC Comics
George Lucas
IDW Publishing
IDW Publishing
IDW Publishing
IDW Publishing
Marvel Comics
Marvel Comics
Marvel Comics
Marvel Comics
Marvel Comics
Marvel Comics
Marvel Comics
Marvel Comics
DC Comics
Marvel Comics
Marvel Comics
unknown
Marvel Comics
DC Comics
George Lucas
DC Comics
Marvel Comics
Marvel Comics
Marvel Comics
Marvel Comics
DC Comics
Marvel Comics`},{path:`testNames.txt`,text:`Superman
Batman
Spider-Man
Iron Man`},{path:`testPublishersWithDuplicates.txt`,text:`DC
DC
Marvel
Marvel`}]},{name:`Data for Social Good Project #1`,lesson:`Lesson 13a: Data for Social Good Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {





    


    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {







  
}`}],validationFiles:[],dataFiles:[{path:`colors.txt`,text:`Yellow
Blue
Black
Black
White
Brown
Grey
Grey
White
Blue
Blue
Grey
Brown
Black
Green
Brown
Brown
Brown
Black
Brown
Black
White
Brown
Black
Black
Brown
Brown
Yellow
Red
Grey
Yellow
White
Blue
Yellow
Green
Grey
Black
Black
Black
Brown
White
Pink
Grey
Black
Brown
Green
White
Brown
Brown
Blue
Brown
White
Brown
Brown
Yellow
Brown
Red
Brown
Grey
White
Black
Pink
Blue
White
Brown
Grey
White
Brown
Black
White
Grey
Pink
White
Brown
Green
Brown
Red
White
Brown
Grey
Brown
Black
Grey
White
Brown
Black
Brown
Black
Black
Black
Blue
Pink
Blue
White
White
Red
Brown
Black`},{path:`diets.txt`,text:`seeds
frogs
flying insects
squid
fish
fish
fish
fish
fish
nuts
fruit
flies
insects
ants
nectar
insects
flying insects
seeds
seeds
fruit
carrion
larvae
carcasses
seeds
fish
insects
flying insects
insects
insects
shoots
fruit
flying insects
insects
insects
seeds
fish
fruits
fish
fruit
flying insects
fish
plankton
monkeys
fish
leaves
plants
carcasses
snakes
mice
licuri palm nuts
grubs
crustaceans
insects
seeds
Insects
leaves
seeds
insects
pigeons
crustaceans
insects
spiders
pinyon seeds
fish
blood
insects
squid
fruit
eucalyptus nectar
insects
fruits
crustaceans
insects
insects
flower nectar
insects
insects
insects
birds
fish
insects
insects
snails
small mammals
mosquitoes
fruit
large insects
fruit
beetles
fish
seeds
fruit
seeds
aquatic insects
crustaceans
fruit
berries
tree sap`},{path:`names.txt`,text:`American Goldfinch
American Purple Gallinule
American Redstart
Amsterdam Albatross
Arctic Tern
Bald Eagle
Belted Kingfisher
Black-Crowned Night Heron
Blue Footed Booby
Blue Jay
Blue Manakin
Bokikokiko
Brown Creeper
Buff-Spotted Flufftail
Bumblebee Hummingbird
Carolina Wren
Cave Swallow
Common Buttonquail
Common Ostrich
Common Pheasant
Common Raven
Common Sandpiper
Crested Caracara
Dark-Eyed Junco
Double-Crested Cormorant
Eastern Screech Owl
Eastern Whip-Poor-Will
Elegant Sunbird
Elegant Trogon
Emperor Goose
Flame Bowerbird
Fork-Tailed Flycatcher
Gambel's Quail
Golden White-Eye
Gouldian Finch
Great Blue Heron
Great Curassow
Great Frigatebird
Great Hornbill
Great Potoo
Great White Pelican
Greater Flamingo
Harpy Eagle
Hawaiian Stilt
Hoatzin
Kakapo
King Vulture
Laughing Falcon
Laughing Kookaburra
Lear's Macaw
Little Spotted Kiwi
Macaroni Penguin
Mallard
Mourning Dove
Narcissus Flycatcher
Nene
Northern Cardinal
Ovenbird
Peregrine Falcon
Pied Avocet
Pileated Woodpecker
Pink Robin
Pinyon Jay
Razorbill
Red-Billed Oxpecker
Red-Breasted Nuthatch
Red-Tailed Tropicbird
Red-Whiskered Bulbul
Regent Honeyeater
Ring-Billed Gull
Rock Dove
Roseate Spoonbill
Rose-Breasted Gosbeak
Ruby-Crowned Kinglet
Ruby-Throated Hummingbird
Rudd's Lark
Scarlet Tanager
Secretarybird
Sharp-Shinned Hawk
Shoebill
Sierra Madre Sparrow
Smooth-Billed Ani
Snail Kite
Snowy Owl
Spoon-Billed Sandpiper
Superb Bird-of-Paradise
Tawny Frogmouth
Toco Toucan
Tricolored Blackbird
Tufted Puffin
Victoria Crowned Pigeon
Violet-Backed Starling
Vulturine Guineafowl
White Wagtail
Whooping Crane
Wilson's Bird-Of-Paradise
Wood Duck
Yellow Bellied Sapsucker`},{path:`status.txt`,text:`Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Near Threatened
Least Concern
Least Concern
Least Concern
Endangered
Near Threatened
Least Concern
Vulnerable
Least Concern
Near Threatened
Least Concern
Least Concern
Least Concern
Near Threatened
Endangered
Least Concern
Critically Endangered
Least Concern
Least Concern
Least Concern
Endangered
Near Threatened
Vulnerable
Least Concern
Least Concern
Least Concern
Vulnerable
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Vulnerable
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Critically Endangered
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Vulnerable
Least Concern
Vulnerable
Least Concern
Vulnerable
Endangered
Least Concern
Least Concern
Vulnerable
Critically Endangered
Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Near Threatened
Least Concern
Least Concern
Least Concern
Endangered
Near Threatened
Least Concern
Least Concern`}]},{name:`Data for Social Good Project #2`,lesson:`Lesson 13a: Data for Social Good Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    






    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`countries.txt`,text:`Afghanistan
Albania
Algeria
American Samoa
Andorra
Angola
Antigua and Barbuda
Argentina
Armenia
Aruba
Australia
Austria
Azerbaijan
Bahamas, The
Bahrain
Bangladesh
Barbados
Belarus
Belgium
Belize
Benin
Bermuda
Bhutan
Bolivia
Bosnia and Herzegovina
Botswana
Brazil
British Virgin Islands
Brunei Darussalam
Bulgaria
Burkina Faso
Burundi
Cabo Verde
Cambodia
Cameroon
Canada
Cayman Islands
Central African Republic
Chad
Chile
China
Colombia
Comoros
Congo, Dem. Rep.
Congo, Rep.
Costa Rica
Cote d'Ivoire
Croatia
Cuba
Curacao
Cyprus
Czech Republic
Denmark
Djibouti
Dominica
Dominican Republic
Ecuador
Egypt, Arab Rep.
El Salvador
Equatorial Guinea
Eritrea
Estonia
Eswatini
Ethiopia
Faroe Islands
Fiji
Finland
France
French Polynesia
Gabon
Gambia, The
Georgia
Germany
Ghana
Gibraltar
Greece
Greenland
Grenada
Guam
Guatemala
Guinea
Guinea-Bissau
Guyana
Haiti
Honduras
Hong Kong SAR, China
Hungary
Iceland
India
Indonesia
Iran, Islamic Rep.
Iraq
Ireland
Isle of Man
Israel
Italy
Jamaica
Japan
Jordan
Kazakhstan
Kenya
Kiribati
Korea, Dem. People's Rep.
Korea, Rep.
Kuwait
Kyrgyz Republic
Lao PDR
Latvia
Lebanon
Lesotho
Liberia
Libya
Liechtenstein
Lithuania
Luxembourg
Macao SAR, China
Madagascar
Malawi
Malaysia
Maldives
Mali
Malta
Marshall Islands
Mauritania
Mauritius
Mexico
Micronesia, Fed. Sts.
Moldova
Monaco
Mongolia
Montenegro
Morocco
Mozambique
Myanmar
Namibia
Nauru
Nepal
Netherlands
New Caledonia
New Zealand
Nicaragua
Niger
Nigeria
North Macedonia
Northern Mariana Islands
Norway
Oman
Pakistan
Palau
Panama
Papua New Guinea
Paraguay
Peru
Philippines
Poland
Portugal
Puerto Rico
Qatar
Romania
Russian Federation
Rwanda
Samoa
San Marino
Sao Tome and Principe
Saudi Arabia
Senegal
Serbia
Seychelles
Sierra Leone
Singapore
Sint Maarten (Dutch part)
Slovak Republic
Slovenia
Solomon Islands
Somalia
South Africa
South Sudan
Spain
Sri Lanka
St. Kitts and Nevis
St. Lucia
St. Martin (French part)
St. Vincent and the Grenadines
Sudan
Suriname
Sweden
Switzerland
Syrian Arab Republic
Tajikistan
Tanzania
Thailand
Timor-Leste
Togo
Tonga
Trinidad and Tobago
Tunisia
Turkey
Turkmenistan
Turks and Caicos Islands
Tuvalu
Uganda
Ukraine
United Arab Emirates
United Kingdom
United States
Uruguay
Uzbekistan
Vanuatu
Venezuela, RB
Vietnam
Virgin Islands (U.S.)
West Bank and Gaza
Yemen, Rep.
Zambia
Zimbabwe`},{path:`incomes.txt`,text:`Low Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
High Income
Lower Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
Upper Middle Income
Low Income
Low Income
Lower Middle Income
Lower Middle Income
Lower Middle Income
High Income
High Income
Low Income
Low Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Upper Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Low Income
High Income
upper Middle Income
High Income
High Income
High Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Low Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Low Income
High Income
High Income
Lower Middle Income
Lower Middle Income
High Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
High Income
High Income
High Income
Low income
Low Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
High Income
High Income
High Income
Lower Middle Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
High Income
Low Income
High Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
High Income
High Income
Low Income
Low Income
Low Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Lower Middle Income`},{path:`internetpercent.txt`,text:`13.5
71.85
49.04
-1
91.57
14.34
76
74.29
64.74
97.17
86.55
87.48
79.8
85
98.64
15
81.76
79.13
88.66
47.08
20
98.37
48.11
44.29
70.12
47
70.43
77.7
94.87
64.78
16
2.66
57.16
40
23.2
91
81.07
4.34
6.5
82.33
54.3
64.13
8.48
8.62
8.65
74.09
46.82
75.29
57.15
68.13
84.43
80.69
97.32
55.68
69.62
74.82
57.27
46.92
33.82
26.24
1.31
89.36
47
18.62
97.58
49.97
88.89
82.04
72.7
62
19.84
62.72
89.74
39
94.44
72.95
69.48
59.07
80.51
65
18
3.93
37.33
32.47
31.7
90.51
76.07
99.01
34.45
39.9
70
75
84.52
-1
83.73
74.39
55.07
91.28
66.79
78.9
17.83
14.58
-1
96.02
99.6
38
25.51
83.58
78.18
29
7.98
21.76
98.1
79.72
97.06
83.79
9.8
13.78
81.2
63.19
13
81.66
38.7
20.8
58.6
65.77
35.3
76.12
97.05
47.16
71.52
64.8
10
30.68
51
57
34
94.71
82.01
90.81
27.86
5.25
42
79.17
-1
96.49
80.19
15.51
-1
57.87
11.21
64.99
52.54
60.05
77.54
74.66
70.6
99.65
70.68
80.86
21.77
33.61
60.18
29.93
93.31
46
73.36
58.77
9
88.17
-1
80.66
79.75
11.92
2
56.17
7.98
86.11
34.11
80.71
50.82
-1
22.39
30.87
48.95
92.14
89.69
34.25
21.96
25
56.82
27.49
12.36
41.25
77.33
64.19
71.04
21.25
-1
49.32
23.71
62.55
98.45
94.9
87.27
74.77
55.2
25.72
72
70.35
64.38
64.4
26.72
14.3
27.06`},{path:`populations.txt`,text:`37172386
2866376
42228429
55465
77006
30809762
96286
44494502
2951776
105845
24982688
8840521
9939800
385640
1569439
161356039
286641
9483499
11433256
383071
11485048
63973
754394
11353142
3323929
2254126
209469333
29802
428962
7025037
19751535
11175378
543767
16249798
25216237
37057765
64174
4666377
15477751
18729160
1392730000
49648685
832322
84068091
5244363
4999441
25069229
4087843
11338138
159800
1189265
10629928
5793636
958920
71625
10627165
17084357
98423595
6420744
1308974
3213972
1321977
1136191
109224559
48497
883483
5515525
66977107
277679
2119275
2280102
3726549
82905782
29767108
33718
10731726
56025
111454
165768
17247807
12414318
1874309
779004
11123176
9587522
7451000
9775564
352721
1352617328
267663435
81800269
38433600
4867309
84077
8882800
60421760
2934855
126529100
9956011
18272430
51393010
115847
25549819
51606633
4137309
6322800
7061507
1927174
6848925
2108132
4818977
6678567
37910
2801543
607950
631636
26262368
18143315
31528585
515696
19077690
484630
58413
4403319
1265303
126190788
112640
2706049
38682
3170208
622227
36029138
29495962
53708395
2448255
12704
28087871
17231624
284060
4841000
6465513
22442948
195874740
2082958
56882
5311916
4829483
212215030
17907
4176873
8606316
6956071
31989256
106651922
37974750
10283822
3195153
2781677
19466145
144478050
12301939
196130
33785
211028
33699947
15854360
6982604
96762
7650154
5638676
40654
5446771
2073894
652858
15008154
57779622
10975920
46796540
21670000
52441
181889
37264
110210
41801533
575991
10175214
8513227
16906283
9100837
56318348
69428524
1267972
7889094
103197
1389858
11565204
82319724
5850908
37665
11508
42723139
44622516
9630959
66460344
326687501
3449299
32955400
292680
28870195
95540395
106977
4569087
28498687
17351822
14439018`},{path:`unemployment.txt`,text:`11.18
13.75
13.57
9.2
3.7
7.36
8.42
9.22
17.7
8.9
5.16
4.85
4.9
12.7
1.2
4.37
9.72
4.76
5.95
6.6
2.65
8.94
2.45
3.52
15.69
17.86
11.93
2.9
8.7
5.21
6.48
1.57
12.17
0.72
3.53
5.66
4.24
5.6
5.9
7.23
3.8
9.11
8.14
4.49
10
11.49
3.27
8.43
2.4
13.43
8.37
2.24
4.97
5.8
10.96
5.86
3.81
11.74
4.01
8.1
5.1
5.37
22.72
2.25
3.7
4.32
7.36
9.06
11.74
20.39
9.42
12.67
3.38
4.22
1
19.29
9.1
22.9
5.4
2.46
4.55
6
14.02
14.1
5.65
3.09
3.71
2.7
5.33
4.4
12.06
13.02
5.74
2.67
4
10.61
7.72
2.4
15.27
4.9
2.76
9.33
4.7
3.82
2.16
4.54
9.41
7.41
6.35
24.58
3.08
19.03
2.01
6.15
5.59
2
1.79
28.67
3.3
6.12
1.62
3.66
4.74
10.34
6.43
3.48
8.86
2.98
6.33
5.38
15.17
9.3
3.43
0.87
19.88
13.28
11.36
3.83
14.59
4.3
4.52
7.77
8.39
20.74
11.17
3.8
1.8
4.08
1.36
3.89
2.62
6.22
6.43
2.34
3.85
6.99
8.27
0.11
4.19
4.85
15.11
14.47
6.45
13.59
6.04
6.76
12.73
3.53
4.68
4.2
9.9
6.54
5.11
0.69
5.9
28.47
12.15
15.25
4.05
5.12
21.26
12
18.79
17.44
7.22
6.35
4.71
8.61
6.9
2.12
0.77
4.66
3.74
3.07
3.21
15.46
10.89
4
8.3
8.49
9.44
8.8
2.23
4
3.9
8.34
9.3
1.85
6.6
2
7
26.26
13.47
11.63
4.77`}]},{name:`Data for Social Good Project #3`,lesson:`Lesson 13a: Data for Social Good Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    





    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`countries.txt`,text:`Afghanistan
Albania
Algeria
American Samoa
Andorra
Angola
Antigua and Barbuda
Argentina
Armenia
Aruba
Australia
Austria
Azerbaijan
Bahamas, The
Bahrain
Bangladesh
Barbados
Belarus
Belgium
Belize
Benin
Bermuda
Bhutan
Bolivia
Bosnia and Herzegovina
Botswana
Brazil
British Virgin Islands
Brunei Darussalam
Bulgaria
Burkina Faso
Burundi
Cabo Verde
Cambodia
Cameroon
Canada
Cayman Islands
Central African Republic
Chad
Chile
China
Colombia
Comoros
Congo, Dem. Rep.
Congo, Rep.
Costa Rica
Cote d'Ivoire
Croatia
Cuba
Curacao
Cyprus
Czech Republic
Denmark
Djibouti
Dominica
Dominican Republic
Ecuador
Egypt, Arab Rep.
El Salvador
Equatorial Guinea
Eritrea
Estonia
Eswatini
Ethiopia
Faroe Islands
Fiji
Finland
France
French Polynesia
Gabon
Gambia, The
Georgia
Germany
Ghana
Gibraltar
Greece
Greenland
Grenada
Guam
Guatemala
Guinea
Guinea-Bissau
Guyana
Haiti
Honduras
Hong Kong SAR, China
Hungary
Iceland
India
Indonesia
Iran, Islamic Rep.
Iraq
Ireland
Isle of Man
Israel
Italy
Jamaica
Japan
Jordan
Kazakhstan
Kenya
Kiribati
Korea, Dem. People's Rep.
Korea, Rep.
Kuwait
Kyrgyz Republic
Lao PDR
Latvia
Lebanon
Lesotho
Liberia
Libya
Liechtenstein
Lithuania
Luxembourg
Macao SAR, China
Madagascar
Malawi
Malaysia
Maldives
Mali
Malta
Marshall Islands
Mauritania
Mauritius
Mexico
Micronesia, Fed. Sts.
Moldova
Monaco
Mongolia
Montenegro
Morocco
Mozambique
Myanmar
Namibia
Nauru
Nepal
Netherlands
New Caledonia
New Zealand
Nicaragua
Niger
Nigeria
North Macedonia
Northern Mariana Islands
Norway
Oman
Pakistan
Palau
Panama
Papua New Guinea
Paraguay
Peru
Philippines
Poland
Portugal
Puerto Rico
Qatar
Romania
Russian Federation
Rwanda
Samoa
San Marino
Sao Tome and Principe
Saudi Arabia
Senegal
Serbia
Seychelles
Sierra Leone
Singapore
Sint Maarten (Dutch part)
Slovak Republic
Slovenia
Solomon Islands
Somalia
South Africa
South Sudan
Spain
Sri Lanka
St. Kitts and Nevis
St. Lucia
St. Martin (French part)
St. Vincent and the Grenadines
Sudan
Suriname
Sweden
Switzerland
Syrian Arab Republic
Tajikistan
Tanzania
Thailand
Timor-Leste
Togo
Tonga
Trinidad and Tobago
Tunisia
Turkey
Turkmenistan
Turks and Caicos Islands
Tuvalu
Uganda
Ukraine
United Arab Emirates
United Kingdom
United States
Uruguay
Uzbekistan
Vanuatu
Venezuela, RB
Vietnam
Virgin Islands (U.S.)
West Bank and Gaza
Yemen, Rep.
Zambia
Zimbabwe`},{path:`incomes.txt`,text:`Low Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
High Income
Lower Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
Upper Middle Income
Low Income
Low Income
Lower Middle Income
Lower Middle Income
Lower Middle Income
High Income
High Income
Low Income
Low Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Upper Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Low Income
High Income
upper Middle Income
High Income
High Income
High Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Low Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Low Income
High Income
High Income
Lower Middle Income
Lower Middle Income
High Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
High Income
High Income
High Income
Low income
Low Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
High Income
High Income
High Income
Lower Middle Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
High Income
Low Income
High Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
High Income
High Income
Low Income
Low Income
Low Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Lower Middle Income`},{path:`populations.txt`,text:`37172386
2866376
42228429
55465
77006
30809762
96286
44494502
2951776
105845
24982688
8840521
9939800
385640
1569439
161356039
286641
9483499
11433256
383071
11485048
63973
754394
11353142
3323929
2254126
209469333
29802
428962
7025037
19751535
11175378
543767
16249798
25216237
37057765
64174
4666377
15477751
18729160
1392730000
49648685
832322
84068091
5244363
4999441
25069229
4087843
11338138
159800
1189265
10629928
5793636
958920
71625
10627165
17084357
98423595
6420744
1308974
3213972
1321977
1136191
109224559
48497
883483
5515525
66977107
277679
2119275
2280102
3726549
82905782
29767108
33718
10731726
56025
111454
165768
17247807
12414318
1874309
779004
11123176
9587522
7451000
9775564
352721
1352617328
267663435
81800269
38433600
4867309
84077
8882800
60421760
2934855
126529100
9956011
18272430
51393010
115847
25549819
51606633
4137309
6322800
7061507
1927174
6848925
2108132
4818977
6678567
37910
2801543
607950
631636
26262368
18143315
31528585
515696
19077690
484630
58413
4403319
1265303
126190788
112640
2706049
38682
3170208
622227
36029138
29495962
53708395
2448255
12704
28087871
17231624
284060
4841000
6465513
22442948
195874740
2082958
56882
5311916
4829483
212215030
17907
4176873
8606316
6956071
31989256
106651922
37974750
10283822
3195153
2781677
19466145
144478050
12301939
196130
33785
211028
33699947
15854360
6982604
96762
7650154
5638676
40654
5446771
2073894
652858
15008154
57779622
10975920
46796540
21670000
52441
181889
37264
110210
41801533
575991
10175214
8513227
16906283
9100837
56318348
69428524
1267972
7889094
103197
1389858
11565204
82319724
5850908
37665
11508
42723139
44622516
9630959
66460344
326687501
3449299
32955400
292680
28870195
95540395
106977
4569087
28498687
17351822
14439018`},{path:`unemployment.txt`,text:`11.18
13.75
13.57
9.2
3.7
7.36
8.42
9.22
17.7
8.9
5.16
4.85
4.9
12.7
1.2
4.37
9.72
4.76
5.95
6.6
2.65
8.94
2.45
3.52
15.69
17.86
11.93
2.9
8.7
5.21
6.48
1.57
12.17
0.72
3.53
5.66
4.24
5.6
5.9
7.23
3.8
9.11
8.14
4.49
10
11.49
3.27
8.43
2.4
13.43
8.37
2.24
4.97
5.8
10.96
5.86
3.81
11.74
4.01
8.1
5.1
5.37
22.72
2.25
3.7
4.32
7.36
9.06
11.74
20.39
9.42
12.67
3.38
4.22
1
19.29
9.1
22.9
5.4
2.46
4.55
6
14.02
14.1
5.65
3.09
3.71
2.7
5.33
4.4
12.06
13.02
5.74
2.67
4
10.61
7.72
2.4
15.27
4.9
2.76
9.33
4.7
3.82
2.16
4.54
9.41
7.41
6.35
24.58
3.08
19.03
2.01
6.15
5.59
2
1.79
28.67
3.3
6.12
1.62
3.66
4.74
10.34
6.43
3.48
8.86
2.98
6.33
5.38
15.17
9.3
3.43
0.87
19.88
13.28
11.36
3.83
14.59
4.3
4.52
7.77
8.39
20.74
11.17
3.8
1.8
4.08
1.36
3.89
2.62
6.22
6.43
2.34
3.85
6.99
8.27
0.11
4.19
4.85
15.11
14.47
6.45
13.59
6.04
6.76
12.73
3.53
4.68
4.2
9.9
6.54
5.11
0.69
5.9
28.47
12.15
15.25
4.05
5.12
21.26
12
18.79
17.44
7.22
6.35
4.71
8.61
6.9
2.12
0.77
4.66
3.74
3.07
3.21
15.46
10.89
4
8.3
8.49
9.44
8.8
2.23
4
3.9
8.34
9.3
1.85
6.6
2
7
26.26
13.47
11.63
4.77`}]},{name:`Data for Social Good Project #4`,lesson:`Lesson 13a: Data for Social Good Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    




    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`arenas.txt`,text:`TD Garden
Barclays Center
Madison Square Garden
Wells Fargo Center
Scotiabank Arena
United Center
Rocket Mortgage FieldHouse
Little Caesars Arena
Bankers Life Fieldhouse
Fiserv Forum
State Farm Arena
Spectrum Center
American Airlines Arena
Amway Center
Capital One Arena
Pepsi Center
Target Center
Chesapeake Energy Arena
Moda Center
Vivint Smart Home Arena
Chase Center
Staples Center
Staples Center
Talking Stick Resort Arena
Golden 1 Center
American Airlines Center
Toyota Center
FedExForum
Smoothie King Center
AT&T Center`},{path:`capacities.txt`,text:`18624
17732
19812
21600
19800
20917
20562
20491
17923
18717
15711
19077
19600
18846
20356
19520
19356
18203
19441
19911
18064
19060
18997
18055
17500
19200
18055
18119
16867
18418`},{path:`championships.txt`,text:`17
0
2
3
1
6
1
3
0
1
1
0
3
0
1
0
0
1
1
0
6
0
16
0
1
1
2
0
0
5`},{path:`locations.txt`,text:`Boston
New York City
New York City
Philadelphia
Toronto
Chicago
Cleveland
Detroit
Indianapolis
Milwaukee
Atlanta
Charlotte
Miami
Orlando
Washington
Denver
Minneapolis
Oklahoma City
Portland
Salt Lake City
San Francisco
Los Angeles
Los Angeles
Phoenix
Sacramento
Dallas
Houston
Memphis
New Orleans
San Antonio`},{path:`teams.txt`,text:`Boston Celtics
Brooklyn Nets
New York Knicks
Philadelphia 76ers
Toronto Raptors
Chicago Bulls
Cleveland Cavaliers
Detroit Pistons
Indiana Pacers
Milwaukee Bucks
Atlanta Hawks
Charlotte Hornets
Miami Heat
Orlando Magic
Washington Wizards
Denver Nuggets
Minnesota Timberwolves
Oklahoma City Thunder
Portland Trail Blazers
Utah Jazz
Golden State Warriors
Los Angeles Clippers
Los Angeles Lakers
Phoenix Suns
Sacramento Kings
Dallas Mavericks
Houston Rockets
Memphis Grizzlies
New Orleans Pelicans
San Antonio Spurs`}]},{name:`Data for Social Good Project #5`,lesson:`Lesson 13a: Data for Social Good Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {





    

    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`albums.txt`,text:`Sgt. Pepper's Lonely Hearts Club Band
Pet Sounds
Revolver
Highway 61 Revisited
Rubber Soul
What's Going On
Exile on Main St.
London Calling
Blonde on Blonde
The Beatles ("The White Album")
The Sun Sessions
Kind of Blue
The Velvet Underground & Nico
Abbey Road
Are You Experienced
Blood on the Tracks
Nevermind
Born to Run
Astral Weeks
Thriller
The Great Twenty_Eight
The Complete Recordings
John Lennon/Plastic Ono Band
Innervisions
Live at the Apollo, 1962
Rumours
The Joshua Tree
Who's Next
Led Zeppelin
Blue
Bringing It All Back Home
Let It Bleed
Ramones
Music From Big Pink
The Rise and Fall of Ziggy Stardust and the Spiders From Mars
Tapestry
Hotel California
The Anthology
Please Please Me
Forever Changes
Never Mind the Bollocks Here's the Sex Pistols
The Doors
The Dark Side of the Moon
Horses
The Band ("The Brown Album")
Legend: The Best of Bob Marley and The Wailers
A Love Supreme
It Takes a Nation of Millions to Hold Us Back
At Fillmore East
Here's Little Richard
Bridge Over Troubled Water
Greatest Hits
Meet The Beatles!
The Birth of Soul
Electric Ladyland
Elvis Presley
Songs in the Key of Life
Beggars Banquet
Chronicle: The 20 Greatest Hits
Trout Mask Replica
Greatest Hits
Appetite for Destruction
Achtung Baby
Sticky Fingers
Back to Mono (1958-1969)
Moondance
Kid A
Off the Wall
[Led Zeppelin IV]
The Stranger
Graceland
Superfly
Physical Graffiti
After the Gold Rush
Star Time
Purple Rain
Back in Black
Otis Blue: Otis Redding Sings Soul
Led Zeppelin II
Imagine
The Clash
Harvest
Axis: Bold as Love
I Never Loved a Man the Way I Love You
Lady Soul
Born in the U.S.A.
The Wall
At Folsom Prison
Dusty in Memphis
Talking Book
Goodbye Yellow Brick Road
20 Golden Greats
Sign "Peace" the Times
40 Greatest Hits
Brew
Tommy
The Freewheelin' Bob Dylan
This Year's Model
There's a Riot Goin' On
Odessey and Oracle
In the Wee Small Hours
Fresh Cream
Giant Steps
Sweet Baby James
Modern Sounds in Country and Western Music
Rocket to Russia
Portrait of a Legend 1951-1964
Hunky Dory
Aftermath
Loaded
The Bends
If You Can Believe Your Eyes and Ears
Court and Spark
Disraeli Gears
The Who Sell Out
Out of Our Heads
Layla and Other Assorted Love Songs
Late Registration
At Last!
Sweetheart of the Rodeo
Stand!
The Harder They Come
Raising Hell
Moby Grape
Pearl
Catch a Fire
Younger Than Yesterday
Raw Power
Remain in Light
Marquee Moon
Paranoid
Saturday Night Fever: The Original Movie Sound Track
The Wild, the Innocent & the E Street Shuffle
Ready to Die
Slanted and Enchanted
Greatest Hits
Tim
The Chronic
Rejuvenation
Parallel Lines
Live at the Regal
A Christmas Gift for You From Phil Spector
Gris-Gris
Straight Outta Compton
Aja
Surrealistic Pillow
Deja vu
Houses of the Holy
Santana
Darkness on the Edge of Town
Funeral
The B 52's / Play Loud
The Low End Theory
Moanin' in the Moonlight
Pretenders
Paul's Boutique
Closer
Captain Fantastic and the Brown Dirt Cowboy
Alive!
Electric Warrior
The Dock of the Bay
OK Computer
1999
The Very Best of Linda Ronstadt
Let's Get It On
Imperial Bedroom
Master of Puppets
My Aim Is True
Exodus
Live at Leeds
The Notorious Byrd Brothers
Every Picture Tells a Story
Something/Anything?
Desire
Close to You
Rocks
One Nation Under a Groove
The Anthology: 1961-1977
The Definitive Collection
The Rolling Stones, Now!
Natty Dread
Fleetwood Mac
Red Headed Stranger
The Immaculate Collection
The Stooges
Fresh
So
Buffalo Springfield Again
Happy Trails
From Elvis in Memphis
Fun House
The Gilded Palace of Sin
Dookie
Transformer
Blues Breakers With Eric Clapton ("The Beano Album")
Nuggets: Original Artyfacts From the First Psychedelic Era, 1965-1968
Murmur
The Best of Little Walter
Is This It
Highway to Hell
The Downward Spiral
Parsley, Sage, Rosemary and Thyme
Bad
Modern Times
Wheels of Fire
Dirty Mind
Abraxas
Tea for the Tillerman
Ten
Everybody Knows This Is Nowhere
Wish You Were Here
Crooked Rain Crooked Rain
Tattoo You
Proud Mary: The Best of Ike and Tina Turner
New York Dolls
Bo Diddley / Go Bo Diddley
Two Steps From the Blues
The Queen Is Dead
Licensed to Ill
Look-Ka Py Py
Loveless
New Orleans Piano
War
The Neil Diamond Collection
American Idiot
Nebraska
Doolittle
Paid in Full
Toys in the Attic
Nick of Time
A Night at the Opera
The Kink Kronikles
Mr. Tambourine Man
Bookends
The Ultimate Collection
Mr. Excitement!
My Generation
Howlin' Wolf
Like a Prayer
Can't Buy a Thrill
Let It Be
Run D.M.C.
Black Sabbath
The Marshall Mathers LP
All Killer No Filler! The Jerry Lee Lewis Anthology
Freak Out!
Live/Dead
The Shape of Jazz to Come
Automatic for the People
Reasonable Doubt
Low
The Blueprint
The River
Complete & Unbelievable: The Otis Redding Dictionary of Soul
Metallica ("The Black Album")
Trans Europa Express
Whitney Houston
The Kinks Are The Village Green Preservation Society
The Velvet Rope
Stardust
American Beauty
Crosby, Stills & Nash
Tracy Chapman
Workingman's Dead
The Genius of Ray Charles
Child Is Father to the Man
Quadrophenia
Paul Simon
Psychocandy
Some Girls
The Beach Boys Today!
Dig Me Out
Going to a Go-Go
Nightbirds
The Slim Shady LP
Mothership Connection
Rhythm Nation 1814
Anthology of American Folk Music
Aladdin Sane
All That You Can't Leave Behind
My Life
Folk Singer
Can't Get Enough
The Cars
Music of My Mind
I'm Still in Love With You
Los Angeles
Anthem of the Sun
Something Else by The Kinks
Call Me
Talking Heads: 77
The Basement Tapes
White Light/White Heat
Kick Out the Jams
Songs of Love and Hate
Meat Is Murder
We're Only in It for the Money
The College Dropout
Weezer (Blue Album)
Master of Reality
Coat of Many Colors
Fear of a Black Planet
John Wesley Harding
Grace
Car Wheels on a Gravel Road
Odelay
A Hard Day's Night
Songs for Swingin' Lovers!
Willy and the Poor Boys
Blood Sugar Sex Magik
The Sun Records Collection
Nothing's Shocking
MTV Unplugged in New York
The Miseducation of Lauryn Hill
Damn the Torpedoes
The Velvet Underground
Surfer Rosa
Back Stabbers
Burnin'
Amnesiac
Pink Moon
Sail Away
Ghost in the Machine
Station to Station
Slowhand
Disintegration
Exile in Guyville
Daydream Nation
In the Jungle Groove
Tonight's the Night
Help!
Shoot Out the Lights
Wild Gift
Squeezing Out Sparks
Superunknown
In Rainbows
Aqualung
Cheap Thrills
The Heart of Saturday Night
Damaged
Play
Violator
Bat Out of Hell
Berlin
Stop Making Sense
3 Feet High and Rising
The Piper at the Gates of Dawn
Muddy Waters at Newport 1960
The Black Album
Roger the Engineer
Rust Never Sleeps
Brothers in Arms
My Beautiful Dark Twisted Fantasy
52nd Street
Having a Rave Up
12 Songs
Between the Buttons
Sketches of Spain
Honky Chateau
Singles Going Steady
Stankonia
Siamese Dream
Substance 1987
L.A. Woman
Rage Against the Machine
American Recordings
Ray of Light
Eagles
Louder Than Bombs
Mott
Whatever People Say I Am, That's What I'm Not
Reggatta de Blanc
Volunteers
Siren
Late for the Sky
Post
The Ultimate Collection: 1948-1990
(What's the Story) Morning Glory?
CrazySexyCool
Funky Kingston
The Smile Sessions
The Modern Lovers
More Songs About Buildings and Food
A Quick One
Love and Theft
Pretzel Logic
Enter the Wu_Tang: 36 Chambers
The Indestructible Beat of Soweto
The End of the Innocence
Elephant
The Pretender
Let It Be
Kala
Good Old Boys
Sound of Silver
For Your Pleasure
Blue Lines
Eliminator
Rain Dogs
Anthology: The Best of The Temptations
Californication
Illmatic
(pronounced 'leh-'nerd 'skin-'nerd)
Dr. John's Gumbo
Radio City
Rid of Me
Sandinista!
I Do Not Want What I Haven't Got
Strange Days
Time Out of Mind
461 Ocean Boulevard
Pink Flag
Double Nickels on the Dime
Beauty and the Beat
Van Halen
Mule Variations
Boy
Band on the Run
Dummy
The "Chirping" Crickets
The Best of the Girl Groups, Volume 1
Presenting the Fabulous Ronettes Featuring Veronica
Anthology
The Rising
Grievous Angel
Cheap Trick at Budokan
Sleepless
Outlandos d'Amour
Another Green World
Vampire Weekend
Stories From the City, Stories From the Sea
Here Come the Warm Jets
All Things Must Pass
#1 Record
In Utero
Sea Change
Tha Carter III
Boys Don't Cry
Live at the Harlem Square Club, 1963
Q: Are We Not Men? A: We Are Devo!
In Color
The World Is a Ghetto
Fly Like an Eagle
Back in the USA
Getz / Gilberto
Synchronicity
Third/Sister Lovers
For Everyman
Back to Black
John Prine
Strictly Business
Love It to Death
How Will the Wolf Survive?
Here, My Dear
Z
Tumbleweed Connection
The Drifters' Golden Hits
Live Through This
Metal Box
Document
Heaven Up Here
Hysteria
69 Love Songs
A Rush of Blood to the Head
Tunnel of Love
The Paul Butterfield Blues Band
The Score
Radio
I Want to See the Bright Lights Tonight
Faith
The Smiths
Proxima estacion: Esperanza
Armed Forces
Life After Death
Down Every Road
All Time Greatest Hits
Maggot Brain
Only Built 4 Cuban Linx
Voodoo
Guitar Town
Entertainment!
All the Young Dudes
Vitalogy
That's the Way of the World
She's So Unusual
New Day Rising
Destroyer
Tres hombres
Born Under a Bad Sign
Touch
Yankee Hotel Foxtrot
Oracular Spectacular
Give It Up
Boz Scaggs
White Blood Cells
The Stone Roses
Live in Cook County Jail
Aquemini`},{path:`artists.txt`,text:`The Beatles
The Beach Boys
The Beatles
Bob Dylan
The Beatles
Marvin Gaye
The Rolling Stones
The Clash
Bob Dylan
The Beatles
Elvis Presley
Miles Davis
The Velvet Underground
The Beatles
The Jimi Hendrix Experience
Bob Dylan
Nirvana
Bruce Springsteen
Van Morrison
Michael Jackson
Chuck Berry
Robert Johnson
John Lennon / Plastic Ono Band
Stevie Wonder
James Brown
Fleetwood Mac
U2
The Who
Led Zeppelin
Joni Mitchell
Bob Dylan
The Rolling Stones
Ramones
The Band
David Bowie
Carole King
Eagles
Muddy Waters
The Beatles
Love
Sex Pistols
The Doors
Pink Floyd
Patti Smith
The Band
Bob Marley & The Wailers
John Coltrane
Public Enemy
The Allman Brothers Band
Little Richard
Simon & Garfunkel
Al Green
The Beatles
Ray Charles
The Jimi Hendrix Experience
Elvis Presley
Stevie Wonder
The Rolling Stones
Creedence Clearwater Revival
Captain Beefheart & His Magic Band
Sly & The Family Stone
Guns N' Roses
U2
The Rolling Stones
Phil Spector
Van Morrison
Radiohead
Michael Jackson
Led Zeppelin
Billy Joel
Paul Simon
Curtis Mayfield
Led Zeppelin
Neil Young
James Brown
Prince and the Revolution
AC/DC
Otis Redding
Led Zeppelin
John Lennon
The Clash
Neil Young
The Jimi Hendrix Experience
Aretha Franklin
Aretha Franklin
Bruce Springsteen
Pink Floyd
Johnny Cash
Dusty Springfield
Stevie Wonder
Elton John
Buddy Holly
Prince
Hank Williams
Miles Davis
The Who
Bob Dylan
Elvis Costello
Sly & The Family Stone
The Zombies
Frank Sinatra
Cream
John Coltrane
James Taylor
Ray Charles
Ramones
Sam Cooke
David Bowie
The Rolling Stones
The Velvet Underground
Radiohead
The Mamas and the Papas
Joni Mitchell
Cream
The Who
The Rolling Stones
Derek and the Dominos
Kanye West
Etta James
The Byrds
Sly & The Family Stone
Various Artists
Run D.M.C.
Moby Grape
Janis Joplin
The Wailers
The Byrds
Iggy and The Stooges
Talking Heads
Television
Black Sabbath
Various Artists
Bruce Springsteen
The Notorious B.I.G.
Pavement
Elton John
The Replacements
Dr. Dre
The Meters
Blondie
B.B. King
Phil Spector
Dr. John, the Night Tripper
N.W.A
Steely Dan
Jefferson Airplane
Crosby, Stills, Nash & Young
Led Zeppelin
Santana
Bruce Springsteen
Arcade Fire
The B 52's
A Tribe Called Quest
Howlin' Wolf
Pretenders
Beastie Boys
Joy Division
Elton John
KISS
T. Rex
Otis Redding
Radiohead
Prince
Linda Ronstadt
Marvin Gaye
Elvis Costello & The Attractions
Metallica
Elvis Costello
Bob Marley & The Wailers
The Who
The Byrds
Rod Stewart
Todd Rundgren
Bob Dylan
Carpenters
Aerosmith
Funkadelic
Curtis Mayfield and The Impressions
ABBA
The Rolling Stones
Bob Marley & The Wailers
Fleetwood Mac
Willie Nelson
Madonna
The Stooges
Sly & The Family Stone
Peter Gabriel
Buffalo Springfield
Quicksilver Messenger Service
Elvis Presley
The Stooges
The Flying Burrito Brothers
Green Day
Lou Reed
John Mayall & The Bluesbreakers
Various Artists
R.E.M.
Little Walter
The Strokes
AC/DC
Nine Inch Nails
Simon & Garfunkel
Michael Jackson
Bob Dylan
Cream
Prince
Santana
Cat Stevens
Pearl Jam
Neil Young & Crazy Horse
Pink Floyd
Pavement
The Rolling Stones
Ike & Tina Turner
New York Dolls
Bo Diddley
Bobby "Blue" Bland
The Smiths
Beastie Boys
The Meters
My Bloody Valentine
Professor Longhair
U2
Neil Diamond
Green Day
Bruce Springsteen
Pixies
Eric B. & Rakim
Aerosmith
Bonnie Raitt
Queen
The Kinks
The Byrds
Simon & Garfunkel
Patsy Cline
Jackie Wilson
The Who
Howlin' Wolf
Madonna
Steely Dan
The Replacements
Run D.M.C.
Black Sabbath
Eminem
Jerry Lee Lewis
The Mothers of Invention
The Grateful Dead
Ornette Coleman
R.E.M.
Jay Z
David Bowie
Jay Z
Bruce Springsteen
Otis Redding
Metallica
Kraftwerk
Whitney Houston
The Kinks
Janet
Willie Nelson
Grateful Dead
Crosby, Stills & Nash
Tracy Chapman
Grateful Dead
Ray Charles
Blood, Sweat & Tears
The Who
Paul Simon
The Jesus and Mary Chain
The Rolling Stones
The Beach Boys
Sleater Kinney
Smokey Robinson & The Miracles
LaBelle
Eminem
Parliament
Janet Jackson
Various
David Bowie
U2
Mary J. Blige
Muddy Waters
Barry White
The Cars
Stevie Wonder
Al Green
X
Grateful Dead
The Kinks
Al Green
Talking Heads
Bob Dylan and the Band
The Velvet Underground
MC5
Leonard Cohen
The Smiths
The Mothers of Invention
Kanye West
Weezer
Black Sabbath
Dolly Parton
Public Enemy
Bob Dylan
Jeff Buckley
Lucinda Williams
Beck
The Beatles
Frank Sinatra
Creedence Clearwater Revival
Red Hot Chili Peppers
Various
Jane's Addiction
Nirvana
Lauryn Hill
Tom Petty and the Heartbreakers
The Velvet Underground
Pixies
The O'Jays
The Wailers
Radiohead
Nick Drake
Randy Newman
The Police
David Bowie
Eric Clapton
The Cure
Liz Phair
Sonic Youth
James Brown
Neil Young
The Beatles
Richard & Linda Thompson
X
Graham Parker & The Rumour
Soundgarden
Radiohead
Jethro Tull
Big Brother & the Holding Company
Tom Waits
Black Flag
Moby
Depeche Mode
Meat Loaf
Lou Reed
Talking Heads
De La Soul
Pink Floyd
Muddy Waters
Jay Z
The Yardbirds
Neil Young & Crazy Horse
Dire Straits
Kanye West
Billy Joel
The Yardbirds
Randy Newman
The Rolling Stones
Miles Davis
Elton John
Buzzcocks
OutKast
The Smashing Pumpkins
New Order
The Doors
Rage Against the Machine
Johnny Cash
Madonna
Eagles
The Smiths
Mott the Hoople
Arctic Monkeys
The Police
Jefferson Airplane
Roxy Music
Jackson Browne
Bjork
John Lee Hooker
Oasis
TLC
Toots & The Maytals
The Beach Boys
The Modern Lovers
Talking Heads
The Who
Bob Dylan
Steely Dan
Wu Tang Clan
Various Artists
Don Henley
The White Stripes
Jackson Browne
The Beatles
M.I.A.
Randy Newman
LCD Soundsystem
Roxy Music
Massive Attack
ZZ Top
Tom Waits
The Temptations
Red Hot Chili Peppers
Nas
Lynyrd Skynyrd
Dr. John
Big Star
PJ Harvey
The Clash
Sinead O'Connor
The Doors
Bob Dylan
Eric Clapton
Wire
Minutemen
The Go Go's
Van Halen
Tom Waits
U2
Paul McCartney & Wings
Portishead
The Crickets
Various Artists
The Ronettes
Diana Ross & The Supremes
Bruce Springsteen
Gram Parsons
Cheap Trick
Peter Wolf
The Police
Brian Eno
Vampire Weekend
PJ Harvey
Brian Eno
George Harrison
Big Star
Nirvana
Beck
Lil Wayne
The Cure
Sam Cooke
DEVO
Cheap Trick
War
Steve Miller Band
MC5
Stan Getz
The Police
Big Star
Jackson Browne
Amy Winehouse
John Prine
EPMD
Alice Cooper
Los Lobos
Marvin Gaye
My Morning Jacket
Elton John
The Drifters
Hole
Public Image Ltd.
R.E.M.
Echo and The Bunnymen
Def Leppard
The Magnetic Fields
Coldplay
Bruce Springsteen
The Paul Butterfield Blues Band
Fugees
L.L. Cool J
Richard & Linda Thompson
George Michael
The Smiths
Manu Chao
Elvis Costello & The Attractions
The Notorious B.I.G.
Merle Haggard
Loretta Lynn
Funkadelic
Raekwon
D'Angelo
Steve Earle
Gang of Four
Mott the Hoople
Pearl Jam
Earth, Wind & Fire
Cyndi Lauper
Husker Du
KISS
ZZ Top
Albert King
Eurythmics
Wilco
MGMT
Bonnie Raitt
Boz Scaggs
The White Stripes
The Stone Roses
B.B. King
OutKast`},{path:`genres.txt`,text:`Rock
Rock
Rock
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Jazz
Rock
Rock
Rock
Rock
Rock
Rock
Jazz
Funk 
Rock
Blues
Rock
Funk 
Funk 
Rock
Rock
Rock
Rock
Pop
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Folk
Rock
Rock
Rock
Rock
Rock
Rock
Classical
Reggae
Jazz
Hip Hop
Rock
Rock
Rock
Funk 
Rock
Jazz
Rock
Rock
Funk 
Rock
Rock
Rock
Funk 
Rock
Electronic
Rock
Rock
Jazz
Electronic
Funk 
Rock
Rock
Jazz
Funk 
Rock
Rock
Funk 
Electronic
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Funk 
Funk 
Rock
Rock
Folk
Rock
Funk 
Rock
Rock
Rock
Folk
Jazz
Rock
Folk
Rock
Funk 
Rock
Jazz
Rock
Jazz
Rock
Funk 
Rock
Latin
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Hip Hop
Funk 
Rock
Funk 
Reggae
Hip Hop
Rock
Rock
Reggae
Rock
Rock
Electronic
Rock
Rock
Electronic
Rock
Hip Hop
Rock
Rock
Rock
Hip Hop
Funk 
Electronic
Blues
Rock
Jazz
Hip Hop
Jazz
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Hip Hop
Blues
Rock
Hip Hop
Rock
Rock
Rock
Rock
Funk 
Electronic
Funk 
Rock
Funk 
Rock
Rock
Rock
Reggae
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Funk 
Funk 
Electronic
Rock
Reggae
Rock
Folk
Electronic
Rock
Funk 
Electronic
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Blues
Rock
Rock
Electronic
Rock
Funk 
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Funk 
Rock
Rock
Funk 
Rock
Hip Hop
Funk 
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Hip Hop
Rock
Rock
Rock
Rock
Rock
Rock
Folk
Funk 
Rock
Blues
Electronic
Rock
Rock
Hip Hop
Rock
Hip Hop
Rock
Electronic
Rock
Jazz
Rock
Hip Hop
Electronic
Hip Hop
Rock
Funk 
Rock
Electronic
Funk 
Rock
Electronic
Pop
Rock
Rock
Rock
Rock
Jazz
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Funk 
Funk 
Hip Hop
Funk 
Electronic
Blues
Rock
Rock
Hip Hop
Blues
Funk 
Electronic
Funk 
Funk 
Rock
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Folk
Rock
Electronic
Hip Hop
Rock
Rock
Folk
Hip Hop
Rock
Rock
Folk
Electronic
Rock
Jazz
Rock
Rock
Rock
Rock
Rock
Hip Hop
Rock
Rock
Rock
Funk 
Reggae
Electronic
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Jazz
Rock
Electronic
Electronic
Rock
Rock
Rock
Hip Hop
Rock
Rock
Hip Hop
Rock
Rock
Rock
Hip Hop
Jazz
Rock
Rock
Rock
Jazz
Rock
Rock
Hip Hop
Rock
Electronic
Rock
Hip Hop
Folk
Electronic
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Blues
Rock
Electronic
Reggae
Rock
Rock
Rock
Rock
Rock
Jazz
Hip Hop
Funk
Rock
Rock
Rock
Rock
Electronic
Rock
Electronic
Rock
Electronic
Rock
Rock
Electronic
Rock
Hip Hop
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Electronic
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Hip Hop
Rock
Funk 
Rock
Rock
Funk 
Rock
Rock
Jazz
Rock
Rock
Rock
Funk 
Folk
Hip Hop
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Hip Hop
Hip Hop
Rock
Electronic
Rock
Rock
Rock
Hip Hop
Folk
Folk
Rock
Hip Hop
Hip Hop
Rock
Rock
Rock
Rock
Funk 
Electronic
Rock
Rock
Rock
Funk 
Electronic
Rock
Electronic
Rock
Rock
Rock
Rock
Blues
Hip Hop`}]},{name:`Data for Social Good Project #6`,lesson:`Lesson 13a: Data for Social Good Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {





    


    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {







  
}`}],validationFiles:[],dataFiles:[]},{name:`Data for Social Good Project #1`,lesson:`Lesson 13b: Data for Social Good Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {





    


    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {







  
}`}],validationFiles:[],dataFiles:[{path:`colors.txt`,text:`Yellow
Blue
Black
Black
White
Brown
Grey
Grey
White
Blue
Blue
Grey
Brown
Black
Green
Brown
Brown
Brown
Black
Brown
Black
White
Brown
Black
Black
Brown
Brown
Yellow
Red
Grey
Yellow
White
Blue
Yellow
Green
Grey
Black
Black
Black
Brown
White
Pink
Grey
Black
Brown
Green
White
Brown
Brown
Blue
Brown
White
Brown
Brown
Yellow
Brown
Red
Brown
Grey
White
Black
Pink
Blue
White
Brown
Grey
White
Brown
Black
White
Grey
Pink
White
Brown
Green
Brown
Red
White
Brown
Grey
Brown
Black
Grey
White
Brown
Black
Brown
Black
Black
Black
Blue
Pink
Blue
White
White
Red
Brown
Black`},{path:`diets.txt`,text:`seeds
frogs
flying insects
squid
fish
fish
fish
fish
fish
nuts
fruit
flies
insects
ants
nectar
insects
flying insects
seeds
seeds
fruit
carrion
larvae
carcasses
seeds
fish
insects
flying insects
insects
insects
shoots
fruit
flying insects
insects
insects
seeds
fish
fruits
fish
fruit
flying insects
fish
plankton
monkeys
fish
leaves
plants
carcasses
snakes
mice
licuri palm nuts
grubs
crustaceans
insects
seeds
Insects
leaves
seeds
insects
pigeons
crustaceans
insects
spiders
pinyon seeds
fish
blood
insects
squid
fruit
eucalyptus nectar
insects
fruits
crustaceans
insects
insects
flower nectar
insects
insects
insects
birds
fish
insects
insects
snails
small mammals
mosquitoes
fruit
large insects
fruit
beetles
fish
seeds
fruit
seeds
aquatic insects
crustaceans
fruit
berries
tree sap`},{path:`names.txt`,text:`American Goldfinch
American Purple Gallinule
American Redstart
Amsterdam Albatross
Arctic Tern
Bald Eagle
Belted Kingfisher
Black-Crowned Night Heron
Blue Footed Booby
Blue Jay
Blue Manakin
Bokikokiko
Brown Creeper
Buff-Spotted Flufftail
Bumblebee Hummingbird
Carolina Wren
Cave Swallow
Common Buttonquail
Common Ostrich
Common Pheasant
Common Raven
Common Sandpiper
Crested Caracara
Dark-Eyed Junco
Double-Crested Cormorant
Eastern Screech Owl
Eastern Whip-Poor-Will
Elegant Sunbird
Elegant Trogon
Emperor Goose
Flame Bowerbird
Fork-Tailed Flycatcher
Gambel's Quail
Golden White-Eye
Gouldian Finch
Great Blue Heron
Great Curassow
Great Frigatebird
Great Hornbill
Great Potoo
Great White Pelican
Greater Flamingo
Harpy Eagle
Hawaiian Stilt
Hoatzin
Kakapo
King Vulture
Laughing Falcon
Laughing Kookaburra
Lear's Macaw
Little Spotted Kiwi
Macaroni Penguin
Mallard
Mourning Dove
Narcissus Flycatcher
Nene
Northern Cardinal
Ovenbird
Peregrine Falcon
Pied Avocet
Pileated Woodpecker
Pink Robin
Pinyon Jay
Razorbill
Red-Billed Oxpecker
Red-Breasted Nuthatch
Red-Tailed Tropicbird
Red-Whiskered Bulbul
Regent Honeyeater
Ring-Billed Gull
Rock Dove
Roseate Spoonbill
Rose-Breasted Gosbeak
Ruby-Crowned Kinglet
Ruby-Throated Hummingbird
Rudd's Lark
Scarlet Tanager
Secretarybird
Sharp-Shinned Hawk
Shoebill
Sierra Madre Sparrow
Smooth-Billed Ani
Snail Kite
Snowy Owl
Spoon-Billed Sandpiper
Superb Bird-of-Paradise
Tawny Frogmouth
Toco Toucan
Tricolored Blackbird
Tufted Puffin
Victoria Crowned Pigeon
Violet-Backed Starling
Vulturine Guineafowl
White Wagtail
Whooping Crane
Wilson's Bird-Of-Paradise
Wood Duck
Yellow Bellied Sapsucker`},{path:`status.txt`,text:`Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Near Threatened
Least Concern
Least Concern
Least Concern
Endangered
Near Threatened
Least Concern
Vulnerable
Least Concern
Near Threatened
Least Concern
Least Concern
Least Concern
Near Threatened
Endangered
Least Concern
Critically Endangered
Least Concern
Least Concern
Least Concern
Endangered
Near Threatened
Vulnerable
Least Concern
Least Concern
Least Concern
Vulnerable
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Vulnerable
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Critically Endangered
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Least Concern
Vulnerable
Least Concern
Vulnerable
Least Concern
Vulnerable
Endangered
Least Concern
Least Concern
Vulnerable
Critically Endangered
Least Concern
Least Concern
Least Concern
Endangered
Least Concern
Near Threatened
Least Concern
Least Concern
Least Concern
Endangered
Near Threatened
Least Concern
Least Concern`}]},{name:`Data for Social Good Project #2`,lesson:`Lesson 13b: Data for Social Good Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    






    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`countries.txt`,text:`Afghanistan
Albania
Algeria
American Samoa
Andorra
Angola
Antigua and Barbuda
Argentina
Armenia
Aruba
Australia
Austria
Azerbaijan
Bahamas, The
Bahrain
Bangladesh
Barbados
Belarus
Belgium
Belize
Benin
Bermuda
Bhutan
Bolivia
Bosnia and Herzegovina
Botswana
Brazil
British Virgin Islands
Brunei Darussalam
Bulgaria
Burkina Faso
Burundi
Cabo Verde
Cambodia
Cameroon
Canada
Cayman Islands
Central African Republic
Chad
Chile
China
Colombia
Comoros
Congo, Dem. Rep.
Congo, Rep.
Costa Rica
Cote d'Ivoire
Croatia
Cuba
Curacao
Cyprus
Czech Republic
Denmark
Djibouti
Dominica
Dominican Republic
Ecuador
Egypt, Arab Rep.
El Salvador
Equatorial Guinea
Eritrea
Estonia
Eswatini
Ethiopia
Faroe Islands
Fiji
Finland
France
French Polynesia
Gabon
Gambia, The
Georgia
Germany
Ghana
Gibraltar
Greece
Greenland
Grenada
Guam
Guatemala
Guinea
Guinea-Bissau
Guyana
Haiti
Honduras
Hong Kong SAR, China
Hungary
Iceland
India
Indonesia
Iran, Islamic Rep.
Iraq
Ireland
Isle of Man
Israel
Italy
Jamaica
Japan
Jordan
Kazakhstan
Kenya
Kiribati
Korea, Dem. People's Rep.
Korea, Rep.
Kuwait
Kyrgyz Republic
Lao PDR
Latvia
Lebanon
Lesotho
Liberia
Libya
Liechtenstein
Lithuania
Luxembourg
Macao SAR, China
Madagascar
Malawi
Malaysia
Maldives
Mali
Malta
Marshall Islands
Mauritania
Mauritius
Mexico
Micronesia, Fed. Sts.
Moldova
Monaco
Mongolia
Montenegro
Morocco
Mozambique
Myanmar
Namibia
Nauru
Nepal
Netherlands
New Caledonia
New Zealand
Nicaragua
Niger
Nigeria
North Macedonia
Northern Mariana Islands
Norway
Oman
Pakistan
Palau
Panama
Papua New Guinea
Paraguay
Peru
Philippines
Poland
Portugal
Puerto Rico
Qatar
Romania
Russian Federation
Rwanda
Samoa
San Marino
Sao Tome and Principe
Saudi Arabia
Senegal
Serbia
Seychelles
Sierra Leone
Singapore
Sint Maarten (Dutch part)
Slovak Republic
Slovenia
Solomon Islands
Somalia
South Africa
South Sudan
Spain
Sri Lanka
St. Kitts and Nevis
St. Lucia
St. Martin (French part)
St. Vincent and the Grenadines
Sudan
Suriname
Sweden
Switzerland
Syrian Arab Republic
Tajikistan
Tanzania
Thailand
Timor-Leste
Togo
Tonga
Trinidad and Tobago
Tunisia
Turkey
Turkmenistan
Turks and Caicos Islands
Tuvalu
Uganda
Ukraine
United Arab Emirates
United Kingdom
United States
Uruguay
Uzbekistan
Vanuatu
Venezuela, RB
Vietnam
Virgin Islands (U.S.)
West Bank and Gaza
Yemen, Rep.
Zambia
Zimbabwe`},{path:`incomes.txt`,text:`Low Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
High Income
Lower Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
Upper Middle Income
Low Income
Low Income
Lower Middle Income
Lower Middle Income
Lower Middle Income
High Income
High Income
Low Income
Low Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Upper Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Low Income
High Income
upper Middle Income
High Income
High Income
High Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Low Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Low Income
High Income
High Income
Lower Middle Income
Lower Middle Income
High Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
High Income
High Income
High Income
Low income
Low Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
High Income
High Income
High Income
Lower Middle Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
High Income
Low Income
High Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
High Income
High Income
Low Income
Low Income
Low Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Lower Middle Income`},{path:`internetpercent.txt`,text:`13.5
71.85
49.04
-1
91.57
14.34
76
74.29
64.74
97.17
86.55
87.48
79.8
85
98.64
15
81.76
79.13
88.66
47.08
20
98.37
48.11
44.29
70.12
47
70.43
77.7
94.87
64.78
16
2.66
57.16
40
23.2
91
81.07
4.34
6.5
82.33
54.3
64.13
8.48
8.62
8.65
74.09
46.82
75.29
57.15
68.13
84.43
80.69
97.32
55.68
69.62
74.82
57.27
46.92
33.82
26.24
1.31
89.36
47
18.62
97.58
49.97
88.89
82.04
72.7
62
19.84
62.72
89.74
39
94.44
72.95
69.48
59.07
80.51
65
18
3.93
37.33
32.47
31.7
90.51
76.07
99.01
34.45
39.9
70
75
84.52
-1
83.73
74.39
55.07
91.28
66.79
78.9
17.83
14.58
-1
96.02
99.6
38
25.51
83.58
78.18
29
7.98
21.76
98.1
79.72
97.06
83.79
9.8
13.78
81.2
63.19
13
81.66
38.7
20.8
58.6
65.77
35.3
76.12
97.05
47.16
71.52
64.8
10
30.68
51
57
34
94.71
82.01
90.81
27.86
5.25
42
79.17
-1
96.49
80.19
15.51
-1
57.87
11.21
64.99
52.54
60.05
77.54
74.66
70.6
99.65
70.68
80.86
21.77
33.61
60.18
29.93
93.31
46
73.36
58.77
9
88.17
-1
80.66
79.75
11.92
2
56.17
7.98
86.11
34.11
80.71
50.82
-1
22.39
30.87
48.95
92.14
89.69
34.25
21.96
25
56.82
27.49
12.36
41.25
77.33
64.19
71.04
21.25
-1
49.32
23.71
62.55
98.45
94.9
87.27
74.77
55.2
25.72
72
70.35
64.38
64.4
26.72
14.3
27.06`},{path:`populations.txt`,text:`37172386
2866376
42228429
55465
77006
30809762
96286
44494502
2951776
105845
24982688
8840521
9939800
385640
1569439
161356039
286641
9483499
11433256
383071
11485048
63973
754394
11353142
3323929
2254126
209469333
29802
428962
7025037
19751535
11175378
543767
16249798
25216237
37057765
64174
4666377
15477751
18729160
1392730000
49648685
832322
84068091
5244363
4999441
25069229
4087843
11338138
159800
1189265
10629928
5793636
958920
71625
10627165
17084357
98423595
6420744
1308974
3213972
1321977
1136191
109224559
48497
883483
5515525
66977107
277679
2119275
2280102
3726549
82905782
29767108
33718
10731726
56025
111454
165768
17247807
12414318
1874309
779004
11123176
9587522
7451000
9775564
352721
1352617328
267663435
81800269
38433600
4867309
84077
8882800
60421760
2934855
126529100
9956011
18272430
51393010
115847
25549819
51606633
4137309
6322800
7061507
1927174
6848925
2108132
4818977
6678567
37910
2801543
607950
631636
26262368
18143315
31528585
515696
19077690
484630
58413
4403319
1265303
126190788
112640
2706049
38682
3170208
622227
36029138
29495962
53708395
2448255
12704
28087871
17231624
284060
4841000
6465513
22442948
195874740
2082958
56882
5311916
4829483
212215030
17907
4176873
8606316
6956071
31989256
106651922
37974750
10283822
3195153
2781677
19466145
144478050
12301939
196130
33785
211028
33699947
15854360
6982604
96762
7650154
5638676
40654
5446771
2073894
652858
15008154
57779622
10975920
46796540
21670000
52441
181889
37264
110210
41801533
575991
10175214
8513227
16906283
9100837
56318348
69428524
1267972
7889094
103197
1389858
11565204
82319724
5850908
37665
11508
42723139
44622516
9630959
66460344
326687501
3449299
32955400
292680
28870195
95540395
106977
4569087
28498687
17351822
14439018`},{path:`unemployment.txt`,text:`11.18
13.75
13.57
9.2
3.7
7.36
8.42
9.22
17.7
8.9
5.16
4.85
4.9
12.7
1.2
4.37
9.72
4.76
5.95
6.6
2.65
8.94
2.45
3.52
15.69
17.86
11.93
2.9
8.7
5.21
6.48
1.57
12.17
0.72
3.53
5.66
4.24
5.6
5.9
7.23
3.8
9.11
8.14
4.49
10
11.49
3.27
8.43
2.4
13.43
8.37
2.24
4.97
5.8
10.96
5.86
3.81
11.74
4.01
8.1
5.1
5.37
22.72
2.25
3.7
4.32
7.36
9.06
11.74
20.39
9.42
12.67
3.38
4.22
1
19.29
9.1
22.9
5.4
2.46
4.55
6
14.02
14.1
5.65
3.09
3.71
2.7
5.33
4.4
12.06
13.02
5.74
2.67
4
10.61
7.72
2.4
15.27
4.9
2.76
9.33
4.7
3.82
2.16
4.54
9.41
7.41
6.35
24.58
3.08
19.03
2.01
6.15
5.59
2
1.79
28.67
3.3
6.12
1.62
3.66
4.74
10.34
6.43
3.48
8.86
2.98
6.33
5.38
15.17
9.3
3.43
0.87
19.88
13.28
11.36
3.83
14.59
4.3
4.52
7.77
8.39
20.74
11.17
3.8
1.8
4.08
1.36
3.89
2.62
6.22
6.43
2.34
3.85
6.99
8.27
0.11
4.19
4.85
15.11
14.47
6.45
13.59
6.04
6.76
12.73
3.53
4.68
4.2
9.9
6.54
5.11
0.69
5.9
28.47
12.15
15.25
4.05
5.12
21.26
12
18.79
17.44
7.22
6.35
4.71
8.61
6.9
2.12
0.77
4.66
3.74
3.07
3.21
15.46
10.89
4
8.3
8.49
9.44
8.8
2.23
4
3.9
8.34
9.3
1.85
6.6
2
7
26.26
13.47
11.63
4.77`}]},{name:`Data for Social Good Project #3`,lesson:`Lesson 13b: Data for Social Good Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    





    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`countries.txt`,text:`Afghanistan
Albania
Algeria
American Samoa
Andorra
Angola
Antigua and Barbuda
Argentina
Armenia
Aruba
Australia
Austria
Azerbaijan
Bahamas, The
Bahrain
Bangladesh
Barbados
Belarus
Belgium
Belize
Benin
Bermuda
Bhutan
Bolivia
Bosnia and Herzegovina
Botswana
Brazil
British Virgin Islands
Brunei Darussalam
Bulgaria
Burkina Faso
Burundi
Cabo Verde
Cambodia
Cameroon
Canada
Cayman Islands
Central African Republic
Chad
Chile
China
Colombia
Comoros
Congo, Dem. Rep.
Congo, Rep.
Costa Rica
Cote d'Ivoire
Croatia
Cuba
Curacao
Cyprus
Czech Republic
Denmark
Djibouti
Dominica
Dominican Republic
Ecuador
Egypt, Arab Rep.
El Salvador
Equatorial Guinea
Eritrea
Estonia
Eswatini
Ethiopia
Faroe Islands
Fiji
Finland
France
French Polynesia
Gabon
Gambia, The
Georgia
Germany
Ghana
Gibraltar
Greece
Greenland
Grenada
Guam
Guatemala
Guinea
Guinea-Bissau
Guyana
Haiti
Honduras
Hong Kong SAR, China
Hungary
Iceland
India
Indonesia
Iran, Islamic Rep.
Iraq
Ireland
Isle of Man
Israel
Italy
Jamaica
Japan
Jordan
Kazakhstan
Kenya
Kiribati
Korea, Dem. People's Rep.
Korea, Rep.
Kuwait
Kyrgyz Republic
Lao PDR
Latvia
Lebanon
Lesotho
Liberia
Libya
Liechtenstein
Lithuania
Luxembourg
Macao SAR, China
Madagascar
Malawi
Malaysia
Maldives
Mali
Malta
Marshall Islands
Mauritania
Mauritius
Mexico
Micronesia, Fed. Sts.
Moldova
Monaco
Mongolia
Montenegro
Morocco
Mozambique
Myanmar
Namibia
Nauru
Nepal
Netherlands
New Caledonia
New Zealand
Nicaragua
Niger
Nigeria
North Macedonia
Northern Mariana Islands
Norway
Oman
Pakistan
Palau
Panama
Papua New Guinea
Paraguay
Peru
Philippines
Poland
Portugal
Puerto Rico
Qatar
Romania
Russian Federation
Rwanda
Samoa
San Marino
Sao Tome and Principe
Saudi Arabia
Senegal
Serbia
Seychelles
Sierra Leone
Singapore
Sint Maarten (Dutch part)
Slovak Republic
Slovenia
Solomon Islands
Somalia
South Africa
South Sudan
Spain
Sri Lanka
St. Kitts and Nevis
St. Lucia
St. Martin (French part)
St. Vincent and the Grenadines
Sudan
Suriname
Sweden
Switzerland
Syrian Arab Republic
Tajikistan
Tanzania
Thailand
Timor-Leste
Togo
Tonga
Trinidad and Tobago
Tunisia
Turkey
Turkmenistan
Turks and Caicos Islands
Tuvalu
Uganda
Ukraine
United Arab Emirates
United Kingdom
United States
Uruguay
Uzbekistan
Vanuatu
Venezuela, RB
Vietnam
Virgin Islands (U.S.)
West Bank and Gaza
Yemen, Rep.
Zambia
Zimbabwe`},{path:`incomes.txt`,text:`Low Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
High Income
Lower Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
Upper Middle Income
Low Income
Low Income
Lower Middle Income
Lower Middle Income
Lower Middle Income
High Income
High Income
Low Income
Low Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Upper Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Low Income
High Income
Lower Middle Income
Low Income
High Income
upper Middle Income
High Income
High Income
High Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Low Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
Low Income
High Income
High Income
Lower Middle Income
Lower Middle Income
High Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
High Income
High Income
High Income
Low income
Low Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Low Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Upper Middle Income
High Income
High Income
High Income
Lower Middle Income
High Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
Lower Middle Income
High Income
High Income
High Income
High Income
Upper Middle Income
Upper Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
High Income
Lower Middle Income
Upper Middle Income
High Income
Low Income
High Income
High Income
High Income
High Income
Lower Middle Income
Low Income
Upper Middle Income
Low Income
High Income
Upper Middle Income
High Income
Upper Middle Income
High Income
Upper Middle Income
Lower Middle Income
Upper Middle Income
High Income
High Income
Low Income
Low Income
Low Income
Upper Middle Income
Lower Middle Income
Low Income
Upper Middle Income
High Income
Lower Middle Income
Upper Middle Income
Upper Middle Income
High Income
Upper Middle Income
Low Income
Lower Middle Income
High Income
High Income
High Income
High Income
Lower Middle Income
Lower Middle Income
Upper Middle Income
Lower Middle Income
High Income
Lower Middle Income
Low Income
Lower Middle Income
Lower Middle Income`},{path:`populations.txt`,text:`37172386
2866376
42228429
55465
77006
30809762
96286
44494502
2951776
105845
24982688
8840521
9939800
385640
1569439
161356039
286641
9483499
11433256
383071
11485048
63973
754394
11353142
3323929
2254126
209469333
29802
428962
7025037
19751535
11175378
543767
16249798
25216237
37057765
64174
4666377
15477751
18729160
1392730000
49648685
832322
84068091
5244363
4999441
25069229
4087843
11338138
159800
1189265
10629928
5793636
958920
71625
10627165
17084357
98423595
6420744
1308974
3213972
1321977
1136191
109224559
48497
883483
5515525
66977107
277679
2119275
2280102
3726549
82905782
29767108
33718
10731726
56025
111454
165768
17247807
12414318
1874309
779004
11123176
9587522
7451000
9775564
352721
1352617328
267663435
81800269
38433600
4867309
84077
8882800
60421760
2934855
126529100
9956011
18272430
51393010
115847
25549819
51606633
4137309
6322800
7061507
1927174
6848925
2108132
4818977
6678567
37910
2801543
607950
631636
26262368
18143315
31528585
515696
19077690
484630
58413
4403319
1265303
126190788
112640
2706049
38682
3170208
622227
36029138
29495962
53708395
2448255
12704
28087871
17231624
284060
4841000
6465513
22442948
195874740
2082958
56882
5311916
4829483
212215030
17907
4176873
8606316
6956071
31989256
106651922
37974750
10283822
3195153
2781677
19466145
144478050
12301939
196130
33785
211028
33699947
15854360
6982604
96762
7650154
5638676
40654
5446771
2073894
652858
15008154
57779622
10975920
46796540
21670000
52441
181889
37264
110210
41801533
575991
10175214
8513227
16906283
9100837
56318348
69428524
1267972
7889094
103197
1389858
11565204
82319724
5850908
37665
11508
42723139
44622516
9630959
66460344
326687501
3449299
32955400
292680
28870195
95540395
106977
4569087
28498687
17351822
14439018`},{path:`unemployment.txt`,text:`11.18
13.75
13.57
9.2
3.7
7.36
8.42
9.22
17.7
8.9
5.16
4.85
4.9
12.7
1.2
4.37
9.72
4.76
5.95
6.6
2.65
8.94
2.45
3.52
15.69
17.86
11.93
2.9
8.7
5.21
6.48
1.57
12.17
0.72
3.53
5.66
4.24
5.6
5.9
7.23
3.8
9.11
8.14
4.49
10
11.49
3.27
8.43
2.4
13.43
8.37
2.24
4.97
5.8
10.96
5.86
3.81
11.74
4.01
8.1
5.1
5.37
22.72
2.25
3.7
4.32
7.36
9.06
11.74
20.39
9.42
12.67
3.38
4.22
1
19.29
9.1
22.9
5.4
2.46
4.55
6
14.02
14.1
5.65
3.09
3.71
2.7
5.33
4.4
12.06
13.02
5.74
2.67
4
10.61
7.72
2.4
15.27
4.9
2.76
9.33
4.7
3.82
2.16
4.54
9.41
7.41
6.35
24.58
3.08
19.03
2.01
6.15
5.59
2
1.79
28.67
3.3
6.12
1.62
3.66
4.74
10.34
6.43
3.48
8.86
2.98
6.33
5.38
15.17
9.3
3.43
0.87
19.88
13.28
11.36
3.83
14.59
4.3
4.52
7.77
8.39
20.74
11.17
3.8
1.8
4.08
1.36
3.89
2.62
6.22
6.43
2.34
3.85
6.99
8.27
0.11
4.19
4.85
15.11
14.47
6.45
13.59
6.04
6.76
12.73
3.53
4.68
4.2
9.9
6.54
5.11
0.69
5.9
28.47
12.15
15.25
4.05
5.12
21.26
12
18.79
17.44
7.22
6.35
4.71
8.61
6.9
2.12
0.77
4.66
3.74
3.07
3.21
15.46
10.89
4
8.3
8.49
9.44
8.8
2.23
4
3.9
8.34
9.3
1.85
6.6
2
7
26.26
13.47
11.63
4.77`}]},{name:`Data for Social Good Project #4`,lesson:`Lesson 13b: Data for Social Good Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    




    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`arenas.txt`,text:`TD Garden
Barclays Center
Madison Square Garden
Wells Fargo Center
Scotiabank Arena
United Center
Rocket Mortgage FieldHouse
Little Caesars Arena
Bankers Life Fieldhouse
Fiserv Forum
State Farm Arena
Spectrum Center
American Airlines Arena
Amway Center
Capital One Arena
Pepsi Center
Target Center
Chesapeake Energy Arena
Moda Center
Vivint Smart Home Arena
Chase Center
Staples Center
Staples Center
Talking Stick Resort Arena
Golden 1 Center
American Airlines Center
Toyota Center
FedExForum
Smoothie King Center
AT&T Center`},{path:`capacities.txt`,text:`18624
17732
19812
21600
19800
20917
20562
20491
17923
18717
15711
19077
19600
18846
20356
19520
19356
18203
19441
19911
18064
19060
18997
18055
17500
19200
18055
18119
16867
18418`},{path:`championships.txt`,text:`17
0
2
3
1
6
1
3
0
1
1
0
3
0
1
0
0
1
1
0
6
0
16
0
1
1
2
0
0
5`},{path:`locations.txt`,text:`Boston
New York City
New York City
Philadelphia
Toronto
Chicago
Cleveland
Detroit
Indianapolis
Milwaukee
Atlanta
Charlotte
Miami
Orlando
Washington
Denver
Minneapolis
Oklahoma City
Portland
Salt Lake City
San Francisco
Los Angeles
Los Angeles
Phoenix
Sacramento
Dallas
Houston
Memphis
New Orleans
San Antonio`},{path:`teams.txt`,text:`Boston Celtics
Brooklyn Nets
New York Knicks
Philadelphia 76ers
Toronto Raptors
Chicago Bulls
Cleveland Cavaliers
Detroit Pistons
Indiana Pacers
Milwaukee Bucks
Atlanta Hawks
Charlotte Hornets
Miami Heat
Orlando Magic
Washington Wizards
Denver Nuggets
Minnesota Timberwolves
Oklahoma City Thunder
Portland Trail Blazers
Utah Jazz
Golden State Warriors
Los Angeles Clippers
Los Angeles Lakers
Phoenix Suns
Sacramento Kings
Dallas Mavericks
Houston Rockets
Memphis Grizzlies
New Orleans Pelicans
San Antonio Spurs`}]},{name:`Data for Social Good Project #5`,lesson:`Lesson 13b: Data for Social Good Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {





    

    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {





  
}`}],validationFiles:[],dataFiles:[{path:`albums.txt`,text:`Sgt. Pepper's Lonely Hearts Club Band
Pet Sounds
Revolver
Highway 61 Revisited
Rubber Soul
What's Going On
Exile on Main St.
London Calling
Blonde on Blonde
The Beatles ("The White Album")
The Sun Sessions
Kind of Blue
The Velvet Underground & Nico
Abbey Road
Are You Experienced
Blood on the Tracks
Nevermind
Born to Run
Astral Weeks
Thriller
The Great Twenty_Eight
The Complete Recordings
John Lennon/Plastic Ono Band
Innervisions
Live at the Apollo, 1962
Rumours
The Joshua Tree
Who's Next
Led Zeppelin
Blue
Bringing It All Back Home
Let It Bleed
Ramones
Music From Big Pink
The Rise and Fall of Ziggy Stardust and the Spiders From Mars
Tapestry
Hotel California
The Anthology
Please Please Me
Forever Changes
Never Mind the Bollocks Here's the Sex Pistols
The Doors
The Dark Side of the Moon
Horses
The Band ("The Brown Album")
Legend: The Best of Bob Marley and The Wailers
A Love Supreme
It Takes a Nation of Millions to Hold Us Back
At Fillmore East
Here's Little Richard
Bridge Over Troubled Water
Greatest Hits
Meet The Beatles!
The Birth of Soul
Electric Ladyland
Elvis Presley
Songs in the Key of Life
Beggars Banquet
Chronicle: The 20 Greatest Hits
Trout Mask Replica
Greatest Hits
Appetite for Destruction
Achtung Baby
Sticky Fingers
Back to Mono (1958-1969)
Moondance
Kid A
Off the Wall
[Led Zeppelin IV]
The Stranger
Graceland
Superfly
Physical Graffiti
After the Gold Rush
Star Time
Purple Rain
Back in Black
Otis Blue: Otis Redding Sings Soul
Led Zeppelin II
Imagine
The Clash
Harvest
Axis: Bold as Love
I Never Loved a Man the Way I Love You
Lady Soul
Born in the U.S.A.
The Wall
At Folsom Prison
Dusty in Memphis
Talking Book
Goodbye Yellow Brick Road
20 Golden Greats
Sign "Peace" the Times
40 Greatest Hits
Brew
Tommy
The Freewheelin' Bob Dylan
This Year's Model
There's a Riot Goin' On
Odessey and Oracle
In the Wee Small Hours
Fresh Cream
Giant Steps
Sweet Baby James
Modern Sounds in Country and Western Music
Rocket to Russia
Portrait of a Legend 1951-1964
Hunky Dory
Aftermath
Loaded
The Bends
If You Can Believe Your Eyes and Ears
Court and Spark
Disraeli Gears
The Who Sell Out
Out of Our Heads
Layla and Other Assorted Love Songs
Late Registration
At Last!
Sweetheart of the Rodeo
Stand!
The Harder They Come
Raising Hell
Moby Grape
Pearl
Catch a Fire
Younger Than Yesterday
Raw Power
Remain in Light
Marquee Moon
Paranoid
Saturday Night Fever: The Original Movie Sound Track
The Wild, the Innocent & the E Street Shuffle
Ready to Die
Slanted and Enchanted
Greatest Hits
Tim
The Chronic
Rejuvenation
Parallel Lines
Live at the Regal
A Christmas Gift for You From Phil Spector
Gris-Gris
Straight Outta Compton
Aja
Surrealistic Pillow
Deja vu
Houses of the Holy
Santana
Darkness on the Edge of Town
Funeral
The B 52's / Play Loud
The Low End Theory
Moanin' in the Moonlight
Pretenders
Paul's Boutique
Closer
Captain Fantastic and the Brown Dirt Cowboy
Alive!
Electric Warrior
The Dock of the Bay
OK Computer
1999
The Very Best of Linda Ronstadt
Let's Get It On
Imperial Bedroom
Master of Puppets
My Aim Is True
Exodus
Live at Leeds
The Notorious Byrd Brothers
Every Picture Tells a Story
Something/Anything?
Desire
Close to You
Rocks
One Nation Under a Groove
The Anthology: 1961-1977
The Definitive Collection
The Rolling Stones, Now!
Natty Dread
Fleetwood Mac
Red Headed Stranger
The Immaculate Collection
The Stooges
Fresh
So
Buffalo Springfield Again
Happy Trails
From Elvis in Memphis
Fun House
The Gilded Palace of Sin
Dookie
Transformer
Blues Breakers With Eric Clapton ("The Beano Album")
Nuggets: Original Artyfacts From the First Psychedelic Era, 1965-1968
Murmur
The Best of Little Walter
Is This It
Highway to Hell
The Downward Spiral
Parsley, Sage, Rosemary and Thyme
Bad
Modern Times
Wheels of Fire
Dirty Mind
Abraxas
Tea for the Tillerman
Ten
Everybody Knows This Is Nowhere
Wish You Were Here
Crooked Rain Crooked Rain
Tattoo You
Proud Mary: The Best of Ike and Tina Turner
New York Dolls
Bo Diddley / Go Bo Diddley
Two Steps From the Blues
The Queen Is Dead
Licensed to Ill
Look-Ka Py Py
Loveless
New Orleans Piano
War
The Neil Diamond Collection
American Idiot
Nebraska
Doolittle
Paid in Full
Toys in the Attic
Nick of Time
A Night at the Opera
The Kink Kronikles
Mr. Tambourine Man
Bookends
The Ultimate Collection
Mr. Excitement!
My Generation
Howlin' Wolf
Like a Prayer
Can't Buy a Thrill
Let It Be
Run D.M.C.
Black Sabbath
The Marshall Mathers LP
All Killer No Filler! The Jerry Lee Lewis Anthology
Freak Out!
Live/Dead
The Shape of Jazz to Come
Automatic for the People
Reasonable Doubt
Low
The Blueprint
The River
Complete & Unbelievable: The Otis Redding Dictionary of Soul
Metallica ("The Black Album")
Trans Europa Express
Whitney Houston
The Kinks Are The Village Green Preservation Society
The Velvet Rope
Stardust
American Beauty
Crosby, Stills & Nash
Tracy Chapman
Workingman's Dead
The Genius of Ray Charles
Child Is Father to the Man
Quadrophenia
Paul Simon
Psychocandy
Some Girls
The Beach Boys Today!
Dig Me Out
Going to a Go-Go
Nightbirds
The Slim Shady LP
Mothership Connection
Rhythm Nation 1814
Anthology of American Folk Music
Aladdin Sane
All That You Can't Leave Behind
My Life
Folk Singer
Can't Get Enough
The Cars
Music of My Mind
I'm Still in Love With You
Los Angeles
Anthem of the Sun
Something Else by The Kinks
Call Me
Talking Heads: 77
The Basement Tapes
White Light/White Heat
Kick Out the Jams
Songs of Love and Hate
Meat Is Murder
We're Only in It for the Money
The College Dropout
Weezer (Blue Album)
Master of Reality
Coat of Many Colors
Fear of a Black Planet
John Wesley Harding
Grace
Car Wheels on a Gravel Road
Odelay
A Hard Day's Night
Songs for Swingin' Lovers!
Willy and the Poor Boys
Blood Sugar Sex Magik
The Sun Records Collection
Nothing's Shocking
MTV Unplugged in New York
The Miseducation of Lauryn Hill
Damn the Torpedoes
The Velvet Underground
Surfer Rosa
Back Stabbers
Burnin'
Amnesiac
Pink Moon
Sail Away
Ghost in the Machine
Station to Station
Slowhand
Disintegration
Exile in Guyville
Daydream Nation
In the Jungle Groove
Tonight's the Night
Help!
Shoot Out the Lights
Wild Gift
Squeezing Out Sparks
Superunknown
In Rainbows
Aqualung
Cheap Thrills
The Heart of Saturday Night
Damaged
Play
Violator
Bat Out of Hell
Berlin
Stop Making Sense
3 Feet High and Rising
The Piper at the Gates of Dawn
Muddy Waters at Newport 1960
The Black Album
Roger the Engineer
Rust Never Sleeps
Brothers in Arms
My Beautiful Dark Twisted Fantasy
52nd Street
Having a Rave Up
12 Songs
Between the Buttons
Sketches of Spain
Honky Chateau
Singles Going Steady
Stankonia
Siamese Dream
Substance 1987
L.A. Woman
Rage Against the Machine
American Recordings
Ray of Light
Eagles
Louder Than Bombs
Mott
Whatever People Say I Am, That's What I'm Not
Reggatta de Blanc
Volunteers
Siren
Late for the Sky
Post
The Ultimate Collection: 1948-1990
(What's the Story) Morning Glory?
CrazySexyCool
Funky Kingston
The Smile Sessions
The Modern Lovers
More Songs About Buildings and Food
A Quick One
Love and Theft
Pretzel Logic
Enter the Wu_Tang: 36 Chambers
The Indestructible Beat of Soweto
The End of the Innocence
Elephant
The Pretender
Let It Be
Kala
Good Old Boys
Sound of Silver
For Your Pleasure
Blue Lines
Eliminator
Rain Dogs
Anthology: The Best of The Temptations
Californication
Illmatic
(pronounced 'leh-'nerd 'skin-'nerd)
Dr. John's Gumbo
Radio City
Rid of Me
Sandinista!
I Do Not Want What I Haven't Got
Strange Days
Time Out of Mind
461 Ocean Boulevard
Pink Flag
Double Nickels on the Dime
Beauty and the Beat
Van Halen
Mule Variations
Boy
Band on the Run
Dummy
The "Chirping" Crickets
The Best of the Girl Groups, Volume 1
Presenting the Fabulous Ronettes Featuring Veronica
Anthology
The Rising
Grievous Angel
Cheap Trick at Budokan
Sleepless
Outlandos d'Amour
Another Green World
Vampire Weekend
Stories From the City, Stories From the Sea
Here Come the Warm Jets
All Things Must Pass
#1 Record
In Utero
Sea Change
Tha Carter III
Boys Don't Cry
Live at the Harlem Square Club, 1963
Q: Are We Not Men? A: We Are Devo!
In Color
The World Is a Ghetto
Fly Like an Eagle
Back in the USA
Getz / Gilberto
Synchronicity
Third/Sister Lovers
For Everyman
Back to Black
John Prine
Strictly Business
Love It to Death
How Will the Wolf Survive?
Here, My Dear
Z
Tumbleweed Connection
The Drifters' Golden Hits
Live Through This
Metal Box
Document
Heaven Up Here
Hysteria
69 Love Songs
A Rush of Blood to the Head
Tunnel of Love
The Paul Butterfield Blues Band
The Score
Radio
I Want to See the Bright Lights Tonight
Faith
The Smiths
Proxima estacion: Esperanza
Armed Forces
Life After Death
Down Every Road
All Time Greatest Hits
Maggot Brain
Only Built 4 Cuban Linx
Voodoo
Guitar Town
Entertainment!
All the Young Dudes
Vitalogy
That's the Way of the World
She's So Unusual
New Day Rising
Destroyer
Tres hombres
Born Under a Bad Sign
Touch
Yankee Hotel Foxtrot
Oracular Spectacular
Give It Up
Boz Scaggs
White Blood Cells
The Stone Roses
Live in Cook County Jail
Aquemini`},{path:`artists.txt`,text:`The Beatles
The Beach Boys
The Beatles
Bob Dylan
The Beatles
Marvin Gaye
The Rolling Stones
The Clash
Bob Dylan
The Beatles
Elvis Presley
Miles Davis
The Velvet Underground
The Beatles
The Jimi Hendrix Experience
Bob Dylan
Nirvana
Bruce Springsteen
Van Morrison
Michael Jackson
Chuck Berry
Robert Johnson
John Lennon / Plastic Ono Band
Stevie Wonder
James Brown
Fleetwood Mac
U2
The Who
Led Zeppelin
Joni Mitchell
Bob Dylan
The Rolling Stones
Ramones
The Band
David Bowie
Carole King
Eagles
Muddy Waters
The Beatles
Love
Sex Pistols
The Doors
Pink Floyd
Patti Smith
The Band
Bob Marley & The Wailers
John Coltrane
Public Enemy
The Allman Brothers Band
Little Richard
Simon & Garfunkel
Al Green
The Beatles
Ray Charles
The Jimi Hendrix Experience
Elvis Presley
Stevie Wonder
The Rolling Stones
Creedence Clearwater Revival
Captain Beefheart & His Magic Band
Sly & The Family Stone
Guns N' Roses
U2
The Rolling Stones
Phil Spector
Van Morrison
Radiohead
Michael Jackson
Led Zeppelin
Billy Joel
Paul Simon
Curtis Mayfield
Led Zeppelin
Neil Young
James Brown
Prince and the Revolution
AC/DC
Otis Redding
Led Zeppelin
John Lennon
The Clash
Neil Young
The Jimi Hendrix Experience
Aretha Franklin
Aretha Franklin
Bruce Springsteen
Pink Floyd
Johnny Cash
Dusty Springfield
Stevie Wonder
Elton John
Buddy Holly
Prince
Hank Williams
Miles Davis
The Who
Bob Dylan
Elvis Costello
Sly & The Family Stone
The Zombies
Frank Sinatra
Cream
John Coltrane
James Taylor
Ray Charles
Ramones
Sam Cooke
David Bowie
The Rolling Stones
The Velvet Underground
Radiohead
The Mamas and the Papas
Joni Mitchell
Cream
The Who
The Rolling Stones
Derek and the Dominos
Kanye West
Etta James
The Byrds
Sly & The Family Stone
Various Artists
Run D.M.C.
Moby Grape
Janis Joplin
The Wailers
The Byrds
Iggy and The Stooges
Talking Heads
Television
Black Sabbath
Various Artists
Bruce Springsteen
The Notorious B.I.G.
Pavement
Elton John
The Replacements
Dr. Dre
The Meters
Blondie
B.B. King
Phil Spector
Dr. John, the Night Tripper
N.W.A
Steely Dan
Jefferson Airplane
Crosby, Stills, Nash & Young
Led Zeppelin
Santana
Bruce Springsteen
Arcade Fire
The B 52's
A Tribe Called Quest
Howlin' Wolf
Pretenders
Beastie Boys
Joy Division
Elton John
KISS
T. Rex
Otis Redding
Radiohead
Prince
Linda Ronstadt
Marvin Gaye
Elvis Costello & The Attractions
Metallica
Elvis Costello
Bob Marley & The Wailers
The Who
The Byrds
Rod Stewart
Todd Rundgren
Bob Dylan
Carpenters
Aerosmith
Funkadelic
Curtis Mayfield and The Impressions
ABBA
The Rolling Stones
Bob Marley & The Wailers
Fleetwood Mac
Willie Nelson
Madonna
The Stooges
Sly & The Family Stone
Peter Gabriel
Buffalo Springfield
Quicksilver Messenger Service
Elvis Presley
The Stooges
The Flying Burrito Brothers
Green Day
Lou Reed
John Mayall & The Bluesbreakers
Various Artists
R.E.M.
Little Walter
The Strokes
AC/DC
Nine Inch Nails
Simon & Garfunkel
Michael Jackson
Bob Dylan
Cream
Prince
Santana
Cat Stevens
Pearl Jam
Neil Young & Crazy Horse
Pink Floyd
Pavement
The Rolling Stones
Ike & Tina Turner
New York Dolls
Bo Diddley
Bobby "Blue" Bland
The Smiths
Beastie Boys
The Meters
My Bloody Valentine
Professor Longhair
U2
Neil Diamond
Green Day
Bruce Springsteen
Pixies
Eric B. & Rakim
Aerosmith
Bonnie Raitt
Queen
The Kinks
The Byrds
Simon & Garfunkel
Patsy Cline
Jackie Wilson
The Who
Howlin' Wolf
Madonna
Steely Dan
The Replacements
Run D.M.C.
Black Sabbath
Eminem
Jerry Lee Lewis
The Mothers of Invention
The Grateful Dead
Ornette Coleman
R.E.M.
Jay Z
David Bowie
Jay Z
Bruce Springsteen
Otis Redding
Metallica
Kraftwerk
Whitney Houston
The Kinks
Janet
Willie Nelson
Grateful Dead
Crosby, Stills & Nash
Tracy Chapman
Grateful Dead
Ray Charles
Blood, Sweat & Tears
The Who
Paul Simon
The Jesus and Mary Chain
The Rolling Stones
The Beach Boys
Sleater Kinney
Smokey Robinson & The Miracles
LaBelle
Eminem
Parliament
Janet Jackson
Various
David Bowie
U2
Mary J. Blige
Muddy Waters
Barry White
The Cars
Stevie Wonder
Al Green
X
Grateful Dead
The Kinks
Al Green
Talking Heads
Bob Dylan and the Band
The Velvet Underground
MC5
Leonard Cohen
The Smiths
The Mothers of Invention
Kanye West
Weezer
Black Sabbath
Dolly Parton
Public Enemy
Bob Dylan
Jeff Buckley
Lucinda Williams
Beck
The Beatles
Frank Sinatra
Creedence Clearwater Revival
Red Hot Chili Peppers
Various
Jane's Addiction
Nirvana
Lauryn Hill
Tom Petty and the Heartbreakers
The Velvet Underground
Pixies
The O'Jays
The Wailers
Radiohead
Nick Drake
Randy Newman
The Police
David Bowie
Eric Clapton
The Cure
Liz Phair
Sonic Youth
James Brown
Neil Young
The Beatles
Richard & Linda Thompson
X
Graham Parker & The Rumour
Soundgarden
Radiohead
Jethro Tull
Big Brother & the Holding Company
Tom Waits
Black Flag
Moby
Depeche Mode
Meat Loaf
Lou Reed
Talking Heads
De La Soul
Pink Floyd
Muddy Waters
Jay Z
The Yardbirds
Neil Young & Crazy Horse
Dire Straits
Kanye West
Billy Joel
The Yardbirds
Randy Newman
The Rolling Stones
Miles Davis
Elton John
Buzzcocks
OutKast
The Smashing Pumpkins
New Order
The Doors
Rage Against the Machine
Johnny Cash
Madonna
Eagles
The Smiths
Mott the Hoople
Arctic Monkeys
The Police
Jefferson Airplane
Roxy Music
Jackson Browne
Bjork
John Lee Hooker
Oasis
TLC
Toots & The Maytals
The Beach Boys
The Modern Lovers
Talking Heads
The Who
Bob Dylan
Steely Dan
Wu Tang Clan
Various Artists
Don Henley
The White Stripes
Jackson Browne
The Beatles
M.I.A.
Randy Newman
LCD Soundsystem
Roxy Music
Massive Attack
ZZ Top
Tom Waits
The Temptations
Red Hot Chili Peppers
Nas
Lynyrd Skynyrd
Dr. John
Big Star
PJ Harvey
The Clash
Sinead O'Connor
The Doors
Bob Dylan
Eric Clapton
Wire
Minutemen
The Go Go's
Van Halen
Tom Waits
U2
Paul McCartney & Wings
Portishead
The Crickets
Various Artists
The Ronettes
Diana Ross & The Supremes
Bruce Springsteen
Gram Parsons
Cheap Trick
Peter Wolf
The Police
Brian Eno
Vampire Weekend
PJ Harvey
Brian Eno
George Harrison
Big Star
Nirvana
Beck
Lil Wayne
The Cure
Sam Cooke
DEVO
Cheap Trick
War
Steve Miller Band
MC5
Stan Getz
The Police
Big Star
Jackson Browne
Amy Winehouse
John Prine
EPMD
Alice Cooper
Los Lobos
Marvin Gaye
My Morning Jacket
Elton John
The Drifters
Hole
Public Image Ltd.
R.E.M.
Echo and The Bunnymen
Def Leppard
The Magnetic Fields
Coldplay
Bruce Springsteen
The Paul Butterfield Blues Band
Fugees
L.L. Cool J
Richard & Linda Thompson
George Michael
The Smiths
Manu Chao
Elvis Costello & The Attractions
The Notorious B.I.G.
Merle Haggard
Loretta Lynn
Funkadelic
Raekwon
D'Angelo
Steve Earle
Gang of Four
Mott the Hoople
Pearl Jam
Earth, Wind & Fire
Cyndi Lauper
Husker Du
KISS
ZZ Top
Albert King
Eurythmics
Wilco
MGMT
Bonnie Raitt
Boz Scaggs
The White Stripes
The Stone Roses
B.B. King
OutKast`},{path:`genres.txt`,text:`Rock
Rock
Rock
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Jazz
Rock
Rock
Rock
Rock
Rock
Rock
Jazz
Funk 
Rock
Blues
Rock
Funk 
Funk 
Rock
Rock
Rock
Rock
Pop
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Folk
Rock
Rock
Rock
Rock
Rock
Rock
Classical
Reggae
Jazz
Hip Hop
Rock
Rock
Rock
Funk 
Rock
Jazz
Rock
Rock
Funk 
Rock
Rock
Rock
Funk 
Rock
Electronic
Rock
Rock
Jazz
Electronic
Funk 
Rock
Rock
Jazz
Funk 
Rock
Rock
Funk 
Electronic
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Funk 
Funk 
Rock
Rock
Folk
Rock
Funk 
Rock
Rock
Rock
Folk
Jazz
Rock
Folk
Rock
Funk 
Rock
Jazz
Rock
Jazz
Rock
Funk 
Rock
Latin
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Hip Hop
Funk 
Rock
Funk 
Reggae
Hip Hop
Rock
Rock
Reggae
Rock
Rock
Electronic
Rock
Rock
Electronic
Rock
Hip Hop
Rock
Rock
Rock
Hip Hop
Funk 
Electronic
Blues
Rock
Jazz
Hip Hop
Jazz
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Hip Hop
Blues
Rock
Hip Hop
Rock
Rock
Rock
Rock
Funk 
Electronic
Funk 
Rock
Funk 
Rock
Rock
Rock
Reggae
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Funk 
Funk 
Electronic
Rock
Reggae
Rock
Folk
Electronic
Rock
Funk 
Electronic
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Blues
Rock
Rock
Electronic
Rock
Funk 
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Funk 
Rock
Rock
Funk 
Rock
Hip Hop
Funk 
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Hip Hop
Rock
Rock
Rock
Rock
Rock
Rock
Folk
Funk 
Rock
Blues
Electronic
Rock
Rock
Hip Hop
Rock
Hip Hop
Rock
Electronic
Rock
Jazz
Rock
Hip Hop
Electronic
Hip Hop
Rock
Funk 
Rock
Electronic
Funk 
Rock
Electronic
Pop
Rock
Rock
Rock
Rock
Jazz
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Funk 
Funk 
Hip Hop
Funk 
Electronic
Blues
Rock
Rock
Hip Hop
Blues
Funk 
Electronic
Funk 
Funk 
Rock
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Folk
Rock
Electronic
Hip Hop
Rock
Rock
Folk
Hip Hop
Rock
Rock
Folk
Electronic
Rock
Jazz
Rock
Rock
Rock
Rock
Rock
Hip Hop
Rock
Rock
Rock
Funk 
Reggae
Electronic
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Jazz
Rock
Electronic
Electronic
Rock
Rock
Rock
Hip Hop
Rock
Rock
Hip Hop
Rock
Rock
Rock
Hip Hop
Jazz
Rock
Rock
Rock
Jazz
Rock
Rock
Hip Hop
Rock
Electronic
Rock
Hip Hop
Folk
Electronic
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Blues
Rock
Electronic
Reggae
Rock
Rock
Rock
Rock
Rock
Jazz
Hip Hop
Funk
Rock
Rock
Rock
Rock
Electronic
Rock
Electronic
Rock
Electronic
Rock
Rock
Electronic
Rock
Hip Hop
Rock
Funk 
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Electronic
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Rock
Rock
Rock
Rock
Hip Hop
Rock
Funk 
Rock
Rock
Funk 
Rock
Rock
Jazz
Rock
Rock
Rock
Funk 
Folk
Hip Hop
Rock
Rock
Funk 
Rock
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Electronic
Rock
Rock
Rock
Hip Hop
Hip Hop
Rock
Electronic
Rock
Rock
Rock
Hip Hop
Folk
Folk
Rock
Hip Hop
Hip Hop
Rock
Rock
Rock
Rock
Funk 
Electronic
Rock
Rock
Rock
Funk 
Electronic
Rock
Electronic
Rock
Rock
Rock
Rock
Blues
Hip Hop`}]},{name:`Data for Social Good Project #6`,lesson:`Lesson 13b: Data for Social Good Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {





    


    
    
    
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
  
}`},{path:`UserStory.java`,text:`import java.util.Scanner;

public class UserStory {







  
}`}],validationFiles:[],dataFiles:[]},{name:`CSA 2023 Console Sandbox_pilot_oo2025`,lesson:`Sandbox: Console`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`CSA 2023 Theater Sandbox_pilot_oo2025`,lesson:`Sandbox: The Theater`,view:`theater`,grid:`1,0 1,10 1,0 1,0 1,0 1,0 1,0 1,0
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