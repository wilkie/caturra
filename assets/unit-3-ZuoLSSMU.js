var e=[{name:`Predict and Run: The Theater`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    Scene houseScene = new Scene();

    houseScene.playSound("birds.wav");

    houseScene.clear("blue");

    houseScene.setFillColor("green");
    houseScene.removeStrokeColor();
    houseScene.drawRectangle(0, 350, houseScene.getWidth(), 50);

    houseScene.pause(1);
    
    houseScene.drawImage("house.png", 200, 150, 200);

    houseScene.pause(2);

    houseScene.setFillColor("yellow");
    houseScene.drawEllipse(75, 75, 100, 100);

    Theater.playScenes(houseScene);
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The Theater`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    String[] emojiImages = {"smiling.png", "surprised.png", "cool.png"};
    EmojiScene emojis = new EmojiScene(emojiImages);

    emojis.drawScene();
    
    Theater.playScenes(emojis);
    
  }
}`},{path:`EmojiScene.java`,text:`import org.code.theater.*;

public class EmojiScene extends Scene {

  private String[] images;        // The 1D array of image file names
  private String guitarSound;     // The guitar sound effect
  private String background;      // The background color of the scene
  private String clearColor;      // The color to clear the scene with

  /*
   * Initializes images to the specified 1D array of image file names,
   * sets guitarSound to the guitar sound file, background to "orange",
   * and clearColor to "white"
   */
  public EmojiScene(String[] images) {
    this.images = images;
    this.guitarSound = "guitar.wav";
    this.background = "orange";
    this.clearColor = "white";
  }

  /*
   * Draws the scene
   */
  public void drawScene() {
    drawBackground();
    drawAllEmojis();
    playSong();
  }

  /*
   * Draws and resizes an emoji while its size is less than the scene's width
   */
  public void drawEmoji(String filename) {
    playSound(guitarSound);

    for (int size = 100; size < getWidth(); size += 100) {
      drawImage(filename, 50, 50, size);
      pause(0.5);
      clear(clearColor);
    }
  }

  /*
   * Draws each emoji in the 1D array images
   */
  public void drawAllEmojis() {
    for (int index = 0; index < images.length; index++) {
      drawEmoji(images[index]);
    }
  }

  /*
   * Draws the background for the scene
   */
  public void drawBackground() {
    setFillColor(background);
    removeStrokeColor();

    for (int yPos = 0; yPos < getHeight(); yPos += 100) {
      for (int xPos = 25; xPos < getWidth(); xPos += 100) {
        drawEllipse(xPos, yPos, 50, 50);
      }

      pause(0.3);
      clear(clearColor);
    }
  }

  /*
   * Plays each note from a 1D array of notes
   */
  public void playSong() {
    int[] notes = {69, 69, 65, 62, 62, 67, 67, 67, 71, 71, 72, 74,
                   72, 72, 72, 67, 65, 69, 69, 69, 67, 67, 69, 67};

    for (int verses = 0; verses < 2; verses++) {
      for (int index = 0; index < notes.length; index++) {
        playNoteAndPause(notes[index], 0.2);
      }
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Drawing Images (a)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    String[] emojis = {"laughing.png", "smiling.png", "surprised.png", "cool.png"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an ImageScene object, then call the drawScene() method.
     * -----------------------------------------------------------------------------
     */




    
    
    
  }
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene of images
 */
public class ImageScene extends Scene {

  private String[] images;      // The 1D array of images to display in the scene

  /*
   * Initializes images to the specified 1D array of images
   */
  public ImageScene(String[] images) {
    this.images = images;
  }

  /*
   * Draws each images in the 1D array images in the scene
   */
  public void drawScene() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw each image in the 1D array images in the scene. You choose where
     * you want to draw the images and the size of the images.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Drawing Images (b)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    String[] sports = {"basketball.jpg", "baseball.jpg", "football.jpg", "soccer.jpg"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an ImageScene object, then call the drawScene() method.
     * -----------------------------------------------------------------------------
     */




    
    
    
  }
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene of images
 */
public class ImageScene extends Scene {

  private String[] images;      // The 1D array of images to display in the scene

  /*
   * Initializes images to the specified 1D array of images
   */
  public ImageScene(String[] images) {
    this.images = images;
  }

  /*
   * Draws each images in the 1D array images in the scene
   */
  public void drawScene() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw each image in the 1D array images in the scene. You choose where
     * you want to draw the images and the size of the images.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Drawing Images (c)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    String[] food = {"pizza.png", "burger.png", "sushi.png", "curry.png"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an ImageScene object, then call the drawScene() method.
     * -----------------------------------------------------------------------------
     */





    
    
  }
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene of images
 */
public class ImageScene extends Scene {

  private String[] images;      // The 1D array of images to display in the scene

  /*
   * Initializes images to the specified 1D array of images
   */
  public ImageScene(String[] images) {
    this.images = images;
  }

  /*
   * Draws each images in the 1D array images in the scene
   */
  public void drawScene() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw each image in the 1D array images in the scene. You choose where
     * you want to draw the images and the size of the images.
     * -----------------------------------------------------------------------------
     */

    
    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Drawing Images (d)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create a 1D array to store the images you uploaded. Instantiate an
     * ImageScene object with the images, then call the drawScene() method.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene of images
 */
public class ImageScene extends Scene {

  private String[] images;      // The 1D array of images to display in the scene

  /*
   * Initializes images to the specified 1D array of images
   */
  public ImageScene(String[] images) {
    this.images = images;
  }

  /*
   * Draws each image in the 1D array images in the scene
   */
  public void drawScene() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw each image in the 1D array images in the scene. You choose where
     * to draw the images and the size of the images.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creating Drawings (a)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a TreeScene object
    TreeScene tree = new TreeScene();

    // Calls the drawTree() method
    tree.drawTree("brown", "green");

    // Plays the tree scene
    Theater.playScenes(tree);
    
  }
}`},{path:`TreeScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene with a drawing of a tree
 */
public class TreeScene extends Scene {

  /*
   * Draws a tree in the scene
   */
  public void drawTree(String trunkColor, String leavesColor) {
    removeStrokeColor();
    drawTrunk(trunkColor);
    drawLeaves(leavesColor);
  }

  /*
   * Draws the trunk of a tree
   */
  public void drawTrunk(String trunkColor) {
    /* ------------------------------------- TO DO -------------------------------------
     * ✅ Draw the trunk of the tree at (150, 200) with a width of 50 and height of 150.
     * ---------------------------------------------------------------------------------
     */

    
    
  }

  /*
   * Draws the leaves of a tree
   */
  public void drawLeaves(String leavesColor) {
    /* ------------------------------------- TO DO -------------------------------------
     * ✅ Draw the leaves of the tree with a width of 100 and height of 100. The first
     * circle should be at (90, 170), the second circle at (160, 170), and the third
     * circle at (125, 110).
     * ---------------------------------------------------------------------------------
     */

    
    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creating Drawings (b)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a SmileyScene object
    SmileyScene face = new SmileyScene();

    // Calls the drawSmileyFace() method
    face.drawSmileyFace("yellow", "black", "black");

    // Plays the face scene
    Theater.playScenes(face);
    
  }
}`},{path:`SmileyScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene with a drawing of a smiley face
 */
public class SmileyScene extends Scene {

  /*
   * Draws a smiley face in the scene
   */
  public void drawSmileyFace(String faceColor, String eyeColor, String mouthColor) {
    drawFace(faceColor);
    drawEyes(eyeColor);
    drawMouth(mouthColor);
  }

  /*
   * Draws the face
   */
  public void drawFace(String faceColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw a face at (50, 50) with a width of 300 and a height of 300.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Draws the eyes of a smiley face
   */
  public void drawEyes(String eyesColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw the eyes for the smiley face at (120, 120) and (230, 120) with a
     * width of 50 and a height of 50.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Draws the mouth of a smiley face
   */
  public void drawMouth(String mouthColor, String faceColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw the mouth for the smiley face at (130, 200) with a width of 150 and
     * a height of 100. Draw a rectangle to cover the top half of the mouth at
     * (130, 200) with a width of 160 and height of 50.
     * -----------------------------------------------------------------------------
     */


    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Creating Drawings (c)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a HouseScene object
    HouseScene house = new HouseScene();

    // Calls the drawHouse() method
    house.drawHouse("yellow", "brown", "red", "blue");

    // Plays the house scene
    Theater.playScenes(house);
    
  }
}`},{path:`HouseScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene with a drawing of a house
 */
public class HouseScene extends Scene {

  /*
   * Draws a house in the scene
   */
  public void drawHouse(String houseColor, String roofColor, String doorColor, String windowColor) {
    drawMainHouse(houseColor);
    drawRoof(roofColor);
    drawDoor(doorColor);
    drawWindow(windowColor);
  }

  /*
   * Draws the main portion of a house
   */
  public void drawMainHouse(String houseColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw the main part of the house at (100, 150) with a width of 200 and
     * a height of 200.
     * -----------------------------------------------------------------------------
     */

    
    
  }

  /*
   * Draws a roof for a house
   */
  public void drawRoof(String roofColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use an int array containing the points {200, 50, 300, 150, 100, 150} to
     * draw the roof of the house.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Draws a door for a house
   */
  public void drawDoor(String doorColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw the door at (150, 250) with a width of 50 and a height of 100.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Draws a window for a house
   */
  public void drawWindow(String windowColor) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Draw the window at (220, 180) with a width of 50 and a height of 50.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creating Drawings (d)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Instantiates a MuralScene object
    MuralScene mural = new MuralScene();

    // Calls the makeDrawing() method
    mural.makeDrawing();
    
    // Plays the mural scene
    Theater.playScenes(mural);
    
  }
}`},{path:`MuralScene.java`,text:`import org.code.theater.*;


/*
 * Represents a scene with a drawing
 */
public class MuralScene extends Scene {

  /*
   * Creates a drawing in the scene
   */
  public void makeDrawing() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use the drawing methods in the Scene class to create your drawing.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Playing Sounds (a)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Instantiates a FashionScene object
    FashionScene fashion = new FashionScene("fashion.jpg", "happymusic.wav");

    // Calls the drawScene() method
    fashion.drawScene();
    
    // Plays the fashion scene
    Theater.playScenes(fashion);
    
  }
}`},{path:`FashionScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that plays music
 */
public class FashionScene extends Scene {

  private String backgroundImage;     // The image to display in the scene
  private String soundFile;           // The sound file to play

  /*
   * Sets backgroundImages to the specified image and
   * soundFile to the specified sound file
   */
  public FashionScene(String backgroundImage, String soundFile) {
    this.backgroundImage = backgroundImage;
    this.soundFile = soundFile;
  }

  /*
   * Draws backgroundImage in the scene and plays soundFile
   */
  public void drawScene() {
    drawImage(backgroundImage, 0, 0, getWidth());

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use one of the methods in the Scene class to play the sound file.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Playing Sounds (b)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Instantiates a NatureScene object
    NatureScene nature = new NatureScene("nature.jpg", "forestsounds.wav");

    // Calls the drawScene() method
    nature.drawScene();
    
    // Plays the nature scene
    Theater.playScenes(nature);
    
  }
}`},{path:`NatureScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that plays music
 */
public class NatureScene extends Scene {

  private String backgroundImage;     // The image to display in the scene
  private String soundFile;           // The sound file to play

  /*
   * Sets backgroundImages to the specified image and
   * soundFile to the specified sound file
   */
  public NatureScene(String backgroundImage, String soundFile) {
    this.backgroundImage = backgroundImage;
    this.soundFile = soundFile;
  }

  /*
   * Draws backgroundImage in the scene and plays soundFile
   */
  public void drawScene() {
    drawImage(backgroundImage, 0, 0, getWidth());

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use one of the methods in the Scene class to play the sound file.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Playing Sounds (c)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Instantiates a GamingScene object
    GamingScene games = new GamingScene("games.jpg", "gamemusic.wav");

    // Calls the drawScene() method
    games.drawScene();
    
    // Plays the games scene
    Theater.playScenes(games);
    
  }
}`},{path:`GamingScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that plays music
 */
public class GamingScene extends Scene {

  private String backgroundImage;     // The image to display in the scene
  private String soundFile;           // The sound file to play

  /*
   * Sets backgroundImages to the specified image and
   * soundFile to the specified sound file
   */
  public GamingScene(String backgroundImage, String soundFile) {
    this.backgroundImage = backgroundImage;
    this.soundFile = soundFile;
  }

  /*
   * Draws backgroundImage in the scene and plays soundFile
   */
  public void drawScene() {
    drawImage(backgroundImage, 0, 0, getWidth());

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use one of the methods in the Scene class to play the sound file.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Playing Sounds (d)`,lesson:`Lesson 1: The Theater`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a MusicScene object, then call the drawScene() method with
     * the sound file you uploaded to the Asset Manager.
     * -----------------------------------------------------------------------------
     */


    
  }
}`},{path:`MusicScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that plays music
 */
public class MusicScene extends Scene {

  /*
   * Draws the image and plays a sound file
   */
  public void drawScene(String soundFile) {
    drawImage("motivation.jpg", 0, 0, getWidth());

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use one of the methods in the Scene class to play the sound file.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Static Variables and Methods`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArtClubMember.java`,text:`/*
 * Represents a member of an art club
 */
public class ArtClubMember {

  private String name;                 // The name of the club member
  public static int numMembers = 0;   // The number of club members

  /*
   * Sets name to the specified name
   */
  public ArtClubMember(String name) {
    this.name = name;
    numMembers++;
  }

  /*
   * Returns the name of the club member
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the number of club members
   */
  public static String getNumMembers() {
    return "The Art Club has " + numMembers + " members.";
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The static Keyword #1`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArtClubMember.java`,text:`/*
 * Represents a member of an art club
 */
public class ArtClubMember {

  private String name;                 // The name of the club member
  public static int numMembers = 0;   // The number of club members

  /*
   * Sets name to the specified name
   */
  public ArtClubMember(String name) {
    this.name = name;
    numMembers++;
  }

  /*
   * Returns the name of the club member
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the number of club members
   */
  public static String getNumMembers() {
    return "The Art Club has " + numMembers + " members.";
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The static Keyword #2`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArtClubMember.java`,text:`/*
 * Represents a member of an art club
 */
public class ArtClubMember {

  private String name;                 // The name of the club member
  public static int numMembers = 0;   // The number of club members

  /*
   * Sets name to the specified name
   */
  public ArtClubMember(String name) {
    this.name = name;
    numMembers++;
  }

  /*
   * Returns the name of the club member
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the number of club members
   */
  public static String getNumMembers() {
    return "The Art Club has " + numMembers + " members.";
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The static Keyword #3`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`ArtClubMember.java`,text:`/*
 * Represents a member of an art club
 */
public class ArtClubMember {

  private String name;                 // The name of the club member
  public static int numMembers = 0;   // The number of club members

  /*
   * Sets name to the specified name
   */
  public ArtClubMember(String name) {
    this.name = name;
    numMembers++;
  }

  /*
   * Returns the name of the club member
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the number of club members
   */
  public static String getNumMembers() {
    return "The Art Club has " + numMembers + " members.";
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Using Static Variables (a)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    FoodTruck rosiesFoodTruck = new FoodTruck("Rosie");
    FoodTruck erinsFoodTruck = new FoodTruck("Erin");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the value of businessName for each FoodTruck object. Then change
     * the value of businessName to a different name, and print the new value of
     * businessName for each FoodTruck object.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`FoodTruck.java`,text:`/*
 * Represents a single Food Truck at the
 * Project Mercury Pastries Food Truck business
 */
public class FoodTruck {

  private String owner;        // The owner of the Food Truck

  /* ---------------------------------------- TO DO ----------------------------------------
   * ✅ Declare and initialize a static variable called businessName to "Joyful Pastries"
   * ---------------------------------------------------------------------------------------
   */

  

  /*
   * Sets owner to the specified name of the owner
   */
  public FoodTruck(String owner) {
    this.owner = owner;
  }

  /*
   * Returns the owner of the food truck
   */
  public String getOwner() {
    return owner;
  }

}`}],validationFiles:[{path:`FoodTruckTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("FoodTruck.java Test")
public class FoodTruckTest {

  String message;
  String messageGap = "\\n       ";
  FoodTruck testFoodTruck;
  Class foodTruckClass;
   
  @BeforeEach
  public void setup() {
    testFoodTruck = new FoodTruck("A Test Food Truck");
    foodTruckClass = testFoodTruck.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The FoodTruck class has a static variable called businessName => ")
  public void testFoodTruckBusinessName() {
    String variableNotExists = "The FoodTruck class does not have a variable called businessName." + messageGap;
    String variableNotStatic = "The businessName variable is not static." + messageGap;

    Field businessNameField = null;
    String fieldName = "";
    int modifierValue = 0;
    
    try {
      businessNameField = foodTruckClass.getDeclaredField("businessName");
      fieldName = businessNameField.getName();
      modifierValue = businessNameField.getModifiers();
    } catch (Exception e) {
      fail("The FoodTruck class does not have a static variable called businessName." + messageGap);
    }

    assertEquals("businessName", fieldName, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable businessName is initialized to \\"Joyful Pastries\\" => ")
  public void testFoodTruckBusinessNameInitialValue() {
    message = "The static variable businessName is initialized to a different value than \\"Joyful Pastries\\"." + messageGap;

    Field businessNameField = null;
    String actualValue = "";
    
    try {
      businessNameField = foodTruckClass.getDeclaredField("businessName");
      actualValue = (String) businessNameField.get(testFoodTruck);
    } catch (Exception e) {
      fail("The FoodTruck class does not have a static variable called businessName." + messageGap);
    }
    
    assertEquals("Joyful Pastries", actualValue, message);
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables (b)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    State texas = new State("Texas");
    State newYork = new State("New York");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the value of countryName for each State object. Then change the
     * value of countryName to a different name, and print the new value of
     * countryName for each State object.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`State.java`,text:`/*
 * Represents a state in a country
 */
public class State {

  private String name;      // The name of the state

  /* ---------------------------------------- TO DO ----------------------------------------
   * ✅ Declare and initialize a static variable called countryName to "United States"
   * ---------------------------------------------------------------------------------------
   */

    

  /*
   * Sets name to the specified name of the state
   */
  public State(String name) {
    this.name = name;
  }

  /*
   * Returns the name of the state
   */
  public String getName() {
    return name;
  }
  
}`}],validationFiles:[{path:`StateTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("State.java Test")
public class StateTest {

  String message;
  String messageGap = "\\n       ";
  State testState;
  Class stateClass;
   
  @BeforeEach
  public void setup() {
    testState = new State("A Test State");
    stateClass = testState.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The State class has a static variable called countryName => ")
  public void testStateCountryName() {
    String variableNotExists = "The State class does not have a variable called countryName." + messageGap;
    String variableNotStatic = "The countryName variable is not static." + messageGap;

    Field countryNameField = null;
    String fieldName = "";
    int modifierValue = 0;
    
    try {
      countryNameField = stateClass.getDeclaredField("countryName");
      fieldName = countryNameField.getName();
      modifierValue = countryNameField.getModifiers();
    } catch (Exception e) {
      fail("The State class does not have a static variable called countryName." + messageGap);
    }

    assertEquals("countryName", fieldName, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable countryName is initialized to \\"United States\\" => ")
  public void testStateCountryNameInitialValue() {
    message = "The static variable countryName is initialized to a different value than \\"United States\\"." + messageGap;

    Field countryNameField = null;
    String actualValue = "";
    
    try {
      countryNameField = stateClass.getDeclaredField("countryName");
      actualValue = (String) countryNameField.get(testState);
    } catch (Exception e) {
      fail("The State class does not have a static variable called countryName." + messageGap);
    }
    
    assertEquals("United States", actualValue, message);
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables (c)`,lesson:`Lesson 2: Static Variables and Methods`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
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
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.Painter;

public class Main {
  public static void main(String[] args) {

    PainterPlus firstPainter = new PainterPlus(0, 5, "south", 10);
    PainterPlus secondPainter = new PainterPlus(4, 7, "east", 10);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the value of totalPainters. Create more PainterPlus objects, then
     * print the new value of totalPainters.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.*;

/*
 * Creates a PainterPlus
 * PainterPlus is a subclass of Painter
 */
public class PainterPlus extends Painter {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a static variable called totalPainters to 0.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Constructor to create a PainterPlus object
   */
  public PainterPlus() {
    super();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Increase the value of totalPainters by 1.
     * -----------------------------------------------------------------------------
     */

    
  }

  /*
   * Unit 2 Lesson 4
   * Constructor to create a PainterPlus object
   */
  public PainterPlus(int x, int y, String direction, int paint) {
    super(x, y, direction, paint);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Increase the value of totalPainters by 1.
     * -----------------------------------------------------------------------------
     */

    
  }

  /*
   * Turns a PainterPlus object to the right
   * by turning left three times
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }
  
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeAll;
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
  static PainterPlus testPainterPlus;
  static Class painterPlusClass;
   
  @BeforeAll
  public static void setup() {
    testPainterPlus = new PainterPlus();
    painterPlusClass = testPainterPlus.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The PainterPlus class has a static variable called totalPainters => ")
  public void testPainterPlusTotalPainters() {
    String variableNotExists = "The PainterPlus class does not have a variable called totalPainters." + messageGap;
    String variableNotStatic = "The totalPainters variable is not static." + messageGap;

    Field totalPaintersField = null;
    String fieldName = "";
    int modifierValue = 0;
    
    try {
      totalPaintersField = painterPlusClass.getDeclaredField("totalPainters");
      fieldName = totalPaintersField.getName();
      modifierValue = totalPaintersField.getModifiers();
    } catch (Exception e) {
      fail("The PainterPlus class does not have a static variable called totalPainters." + messageGap);
    }

    assertEquals("totalPainters", fieldName, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable totalPainters is initialized to 0 => ")
  public void testPainterPlusTotalPaintersInitialValue() {
    message = "The static variable totalPainters is initialized to a different value than 0." + messageGap;

    Field totalPaintersField = null;
    int actualValue = -1;
    
    try {
      totalPaintersField = painterPlusClass.getDeclaredField("totalPainters");
      actualValue = totalPaintersField.getInt(testPainterPlus);
    } catch (Exception e) {
      fail("The PainterPlus class does not have a static variable called totalPainters." + messageGap);
    }
    
    assertTrue(actualValue == 0 || actualValue == 1, message);
  }

  @Test
  @Order(3)
  @DisplayName("The static variable totalPainters is incremented by 1 in the no-argument constructor => ")
  public void testPainterPlusNoArgConstructor() {
    message = "The static variable totalPainters does not increase by 1 when a default PainterPlus is created." + messageGap;

    Field totalPaintersField = null;
    int actualValue = -1;

    PainterPlus newPainterPlus = new PainterPlus();
    
    try {
      totalPaintersField = painterPlusClass.getDeclaredField("totalPainters");
      actualValue = totalPaintersField.getInt(testPainterPlus);
    } catch (Exception e) {
      fail("The PainterPlus class does not have a static variable called totalPainters." + messageGap);
    }
    
    assertTrue(actualValue == 1 || actualValue == 2, message);
  }

  @Test
  @Order(4)
  @DisplayName("The static variable totalPainters is incremented by 1 in the parameterized constructor => ")
  public void testPainterPlusParameterizedConstructor() {
    message = "The static variable totalPainters does not increase by 1 when a PainterPlus is created with the parameterized constructor." + messageGap;

    Field totalPaintersField = null;
    int actualValue = -1;

    PainterPlus newPainterPlus = new PainterPlus(0, 0, "north", 10);
    
    try {
      totalPaintersField = painterPlusClass.getDeclaredField("totalPainters");
      actualValue = totalPaintersField.getInt(testPainterPlus);
    } catch (Exception e) {
      fail("The PainterPlus class does not have a static variable called totalPainters." + messageGap);
    }
    
    assertTrue(actualValue == 2 || actualValue == 3, message);
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables (d)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Customer aimee = new Customer("Aimee");
    Customer javier = new Customer("Javier");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the value of newCustomers. Create more Customer objects, then
     * print the new value of newCustomers.
     * -----------------------------------------------------------------------------
     */





    
    
  }
}`},{path:`Customer.java`,text:`/*
 * Represents a customer at a store
 */
public class Customer {

  private String name;     // The name of the customer

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a static variable called newCustomers to 0.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Sets name to the specified name of a customer
   */
  public Customer(String name) {
    this.name = name;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Increase the value of newCustomers by 1.
     * -----------------------------------------------------------------------------
     */

    
    
  }

  /*
   * Returns the name of the customer
   */
  public String getName() {
    return name;
  }
  
}`}],validationFiles:[{path:`CustomerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Customer.java Test")
public class CustomerTest {

  String message;
  String messageGap = "\\n       ";
  static Customer testCustomer;
  static Class customerClass;
   
  @BeforeAll
  public static void setup() {
    testCustomer = new Customer("some customer");
    customerClass = testCustomer.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The Customer class has a static variable called newCustomers => ")
  public void testCustomerNewCustomers() {
    String variableNotExists = "The Customer class does not have a variable called newCustomers." + messageGap;
    String variableNotStatic = "The newCustomers variable is not static." + messageGap;

    Field newCustomersField = null;
    String fieldName = "";
    int modifierValue = 0;
    
    try {
      newCustomersField = customerClass.getDeclaredField("newCustomers");
      fieldName = newCustomersField.getName();
      modifierValue = newCustomersField.getModifiers();
    } catch (Exception e) {
      fail("The Customer class does not have a static variable called newCustomers." + messageGap);
    }

    assertEquals("newCustomers", fieldName, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable newCustomers is initialized to 0 => ")
  public void testCustomerNewCustomersInitialValue() {
    message = "The static variable newCustomers is initialized to a different value than 0." + messageGap;

    Field newCustomersField = null;
    int actualValue = -1;
    
    try {
      newCustomersField = customerClass.getDeclaredField("newCustomers");
      actualValue = newCustomersField.getInt(testCustomer);
    } catch (Exception e) {
      fail("The Customer class does not have a static variable called newCustomers." + messageGap);
    }
    
    assertTrue(actualValue == 0 || actualValue == 1, message);
  }
  
  @Test
  @Order(3)
  @DisplayName("The static variable newCustomers is incremented by 1 in the constructor => ")
  public void testCustomerConstructor() {
    message = "The static variable newCustomers does not increase by 1 when a Customers is created." + messageGap;

    Field newCustomersField = null;
    int actualValue = -1;

    Customer newCustomerObject = new Customer("another customer");
    
    try {
      newCustomersField = customerClass.getDeclaredField("newCustomers");
      actualValue = newCustomersField.getInt(newCustomerObject);
    } catch (Exception e) {
      fail("The Customer class does not have a static variable called newCustomers." + messageGap);
    }
    
    assertTrue(actualValue == 2 || actualValue == 3, message);
  }

}`}],dataFiles:[]},{name:`Practice: Writing Static Methods (a)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcEngineersPay() method and print the result.
     * -----------------------------------------------------------------------------
     */


  
    
  }
}`},{path:`ProjectManager.java`,text:`/*
 * Manages information about a project
 */
public class ProjectManager {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a static method called calcEngineersPay() with parameters totalBudget
   * and numEngineers that returns the totalBudget divided by numEngineers.
   * -----------------------------------------------------------------------------
   */

    
  
}`}],validationFiles:[{path:`ProjectManagerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ProjectManager.java Test")
public class ProjectManagerTest {

  String message;
  String failMessage;
  String messageGap = "\\n       ";
  ProjectManager testProjectManager;
  Class projectManagerClass;
  Method staticMethod;
  Class[] methodFields;
   
  @BeforeEach
  @SuppressWarnings("unchecked")
  public void setup() {
    failMessage = "The ProjectManager class does not have a method called calcEngineersPay or the parameters are in a different order." + messageGap;
      
    testProjectManager = new ProjectManager();
    projectManagerClass = testProjectManager.getClass();

    methodFields = new Class[]{double.class, int.class};

    try {
      staticMethod = projectManagerClass.getMethod("calcEngineersPay", methodFields);
    } catch (Exception e) {
      fail(failMessage);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("The ProjectManager class has a calcEngineersPay method with parameters double totalBudget and int numEngineers => ")
  public void testProjectManagerCalcEngineersPay() {
    message = "calcEngineersPay is missing or does not have the expected parameters in the correct order (double totalBudget, int numEngineers)." + messageGap;

    String staticMethodName = staticMethod.getName();
    String expectedMethodName = "calcEngineersPay";
    
    assertEquals(expectedMethodName, staticMethodName, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("calcEngineersPay is a static method => ")
  public void testCalcEngineersPayIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;
    int methodModifiers = staticMethod.getModifiers();
    assertTrue(Modifier.isStatic(methodModifiers), message);
  }
  
  @Test
  @Order(3)
  @DisplayName("calcEngineersPay returns the salary for each engineer on the project => ")
  public void testCalcEngineersPayResult() {
    message = "Return the result of totalBudget divided by numEngineers." + messageGap;

    double randomTotalBudget = (Math.random() * 20000) + 5000;
    int randomNumEngineers = (int)(Math.random() * 10) + 2;

    double expected = randomTotalBudget / randomNumEngineers;
    double actual = -1;

    try {
      actual = (Double) staticMethod.invoke(testProjectManager, randomTotalBudget, randomNumEngineers);
    } catch (Exception e) {
      fail(failMessage);
    }

    assertEquals(expected, actual, message);
  }

}`}],dataFiles:[]},{name:`Practice: Writing Static Methods (b)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[] oliviaGrades = {75, 95, 86, 93, 84};
    int[] ameliaGrades = {91, 96, 86, 88, 72, 90, 80, 82, 84, 73};
    int[] javonGrades = {97, 91, 80, 79, 86, 71, 85};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverageGrade() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`GradeCalculator.java`,text:`/*
 * Manages student grades
 */
public class GradeCalculator {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a static method called calcAverageGrade() with a parameter
   * int[] grades that returns the average of the values in grades.
   * -----------------------------------------------------------------------------
   */


  

}`}],validationFiles:[{path:`GradeCalculatorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("GradeCalculator.java Test")
public class GradeCalculatorTest {

  String message;
  String failMessage;
  String messageGap = "\\n       ";
  GradeCalculator testGradeCalculator;
  Class gradeCalculatorClass;
  Method staticMethod;
  Class[] methodFields;
   
  @BeforeEach
  @SuppressWarnings("unchecked")
  public void setup() {
    failMessage = "The GradeCalculator class does not have a method called calcAverageGrade or the parameter is missing." + messageGap;
      
    testGradeCalculator = new GradeCalculator();
    gradeCalculatorClass = testGradeCalculator.getClass();

    methodFields = new Class[]{int[].class};

    try {
      staticMethod = gradeCalculatorClass.getMethod("calcAverageGrade", methodFields);
    } catch (Exception e) {
      fail(failMessage);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("The GradeCalculator class has a calcAverageGrade method with a parameter int[] grades => ")
  public void testGradeCalculatorCalcAverageGrade() {
    message = "calcAverageGrade is missing or does not have the expected parameter (int[] grades)." + messageGap;

    String staticMethodName = staticMethod.getName();
    String expectedMethodName = "calcAverageGrade";
    
    assertEquals(expectedMethodName, staticMethodName, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("calcAverageGrade is a static method => ")
  public void testCalcAverageGradeIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;
    int methodModifiers = staticMethod.getModifiers();
    assertTrue(Modifier.isStatic(methodModifiers), message);
  }
  
  @Test
  @Order(3)
  @DisplayName("calcAverageGrade returns the average of the values in the parameter grades => ")
  public void testCalcAverageGradeResult() {
    message = "Return the average of the values in the parameter grades." + messageGap;

    int randomNumGrades = (int)(Math.random() * 10) + 2;
    int[] testGrades = new int[randomNumGrades];
    int total = 0;

    for (int index = 0; index < testGrades.length; index++) {
      int randomNum = (int)(Math.random() * 100) + 20;
      testGrades[index] = randomNum;
      total += randomNum;
    }

    int expected = total / randomNumGrades;
    int actual = -1;

    try {
      actual = (Integer) staticMethod.invoke(testGradeCalculator, testGrades);
    } catch (Exception e) {
      fail(failMessage);
    }

    assertEquals(expected, actual, message);
  }

}`}],dataFiles:[]},{name:`Practice: Writing Static Methods (c)`,lesson:`Lesson 2: Static Variables and Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use SoundLoader.read() to extract the samples from the "welcome.wav" and
     * "beat.wav" sound files into 1D double arrays. Then call the combineSounds()
     * method and store the sound returned in a 1D double array called newSound.
     * -----------------------------------------------------------------------------
     */

    

    // Creates a Scene object
    Scene soundScene = new Scene();

    // Plays newSound in the scene
    soundScene.playSound(newSound);

    // Plays the scene
    Theater.playScenes(soundScene);
    
  }
}`},{path:`SoundEditor.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Edits sound samples to create effects
 */
public class SoundEditor {

  /* -------------------------------------- TO DO --------------------------------------
   * ✅ Write a static method called combineSounds() with parameters double[] firstSound
   * and double[] secondSound that returns the first sound combined with the second sound.
   * -----------------------------------------------------------------------------------
   */

  

}`}],validationFiles:[{path:`SoundEditorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SoundEditor.java Test")
public class SoundEditorTest {

  String message;
  String failMessage;
  String messageGap = "\\n       ";
  SoundEditor testSoundEditor;
  Class soundEditorClass;
  Method staticMethod;
  Class[] methodFields;
   
  @BeforeEach
  @SuppressWarnings("unchecked")
  public void setup() {
    failMessage = "The SoundEditor class does not have a method called combineSounds or the parameters are missing." + messageGap;
      
    testSoundEditor = new SoundEditor();
    soundEditorClass = testSoundEditor.getClass();

    methodFields = new Class[]{double[].class, double[].class};

    try {
      staticMethod = soundEditorClass.getMethod("combineSounds", methodFields);
    } catch (Exception e) {
      fail(failMessage);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("The SoundEditor class has a combineSounds method with parameters double[] firstSound and double[] secondSound => ")
  public void testSoundEditorCombineSounds() {
    message = "combineSounds is missing or does not have the expected parameters (double[] firstSound, double[] secondSecond)." + messageGap;

    String staticMethodName = staticMethod.getName();
    String expectedMethodName = "combineSounds";
    
    assertEquals(expectedMethodName, staticMethodName, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("combineSounds is a static method => ")
  public void testCombineSoundsIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;
    int methodModifiers = staticMethod.getModifiers();
    assertTrue(Modifier.isStatic(methodModifiers), message);
  }
  
  @Test
  @Order(3)
  @DisplayName("combineSounds returns a new sound with firstSound followed by secondSound => ")
  public void testCombineSoundsResult() {
    message = "Return a new double array that contains the values in firstSound followed by secondSound." + messageGap;

    double[] testFirst = new double[10];
    double[] testSecond = new double[10];
    double[] testCombined = new double[20];

    int combinedIndex = 0;

    for (int index = 0; index < testFirst.length; index++) {
      testCombined[combinedIndex] = testFirst[index];
      combinedIndex++;
    }

    for (int index = 0; index < testSecond.length; index++) {
      testCombined[combinedIndex] = testSecond[index];
      combinedIndex++;
    }

    double[] actual = null;

    try {
      actual = (double[]) staticMethod.invoke(staticMethod, testFirst, testSecond);
    } catch (Exception e) {
      fail(failMessage);
    }

    assertArrayEquals(testCombined, actual, message);
  }

}`}],dataFiles:[]},{name:`Practice: Writing Static Methods (d)`,lesson:`Lesson 2: Static Variables and Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use SoundLoader.read() to extract the samples from the "retrobeat.wav"
     * sound file into a 1D double array. Then call the createClip() method and
     * store the sound returned in a 1D double array called newSound.
     * -----------------------------------------------------------------------------
     */

    
    

    // Creates a Scene object
    Scene soundScene = new Scene();

    // Plays newSound in the scene
    soundScene.playSound(newSound);

    // Plays the scene
    Theater.playScenes(soundScene);
    
  }
}`},{path:`SoundEditor.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Edits sound samples to create effects
 */
public class SoundEditor {

  /* -------------------------------------- TO DO --------------------------------------
   * ✅ Write a static method called createClip() with parameters double[] sound,
   * int start, and int end that returns the samples from sound from start to end.
   * -----------------------------------------------------------------------------------
   */

  

}`}],validationFiles:[{path:`SoundEditorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import org.code.media.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SoundEditor.java Test")
public class SoundEditorTest {

  String message;
  String failMessage;
  String messageGap = "\\n       ";
  SoundEditor testSoundEditor;
  Class soundEditorClass;
  Method staticMethod;
  Class[] methodFields;
   
  @BeforeEach
  @SuppressWarnings("unchecked")
  public void setup() { 
    failMessage = "The SoundEditor class does not have a method called createClip or the parameters are missing." + messageGap;
      
    testSoundEditor = new SoundEditor();
    soundEditorClass = testSoundEditor.getClass();

    methodFields = new Class[]{double[].class, int.class, int.class};

    try {
      staticMethod = soundEditorClass.getMethod("createClip", methodFields);
    } catch (Exception e) {
      fail(failMessage);
    }
  }
   
  @Test
  @Order(1)
  @DisplayName("The SoundEditor class has a createClip method with parameters double[] sound, int start, and int end => ")
  public void testSoundEditorCreateClip() {
    message = "createClip is missing or does not have the expected parameters (double[] sound, int start, int end)." + messageGap;

    String staticMethodName = staticMethod.getName();
    String expectedMethodName = "createClip";
    
    assertEquals(expectedMethodName, staticMethodName, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("createClip is a static method => ")
  public void testCreateClipIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;
    int methodModifiers = staticMethod.getModifiers();
    assertTrue(Modifier.isStatic(methodModifiers), message);
  }
  
  @Test
  @Order(3)
  @DisplayName("createClip returns a new sound that contains the samples in sound from start to end => ")
  public void testCreateClipResult() {
    message = "Return a new double array that contains the samples in sound from start to end." + messageGap;

    // double[] testSound = SoundLoader.read("retrobeat.wav");
    // int randomStart = (int)(Math.random() * (testSound.length / 2));
    // int randomEnd = (int)(Math.random() * (testSound.length / 2)) + (testSound.length / 2);

    double[] testSound = SoundLoader.read("retrobeat.wav");
    int totalSeconds = testSound.length / 44100;
    
    // Pick small numbers representing SECONDS
    int randomStart = (int)(Math.random() * (totalSeconds / 2)); 
    int randomEnd = (int)(Math.random() * (totalSeconds / 2)) + (totalSeconds / 2) + 1;

    
    double[] expected = clipSound(testSound, randomStart, randomEnd);
    double[] actual = null;

    try {
      // actual = (double[]) staticMethod.invoke(staticMethod, randomStart, randomEnd);
      actual = (double[]) staticMethod.invoke(null, testSound, randomStart, randomEnd);
    } catch (Exception e) {
      fail(failMessage);
    }

    assertArrayEquals(expected, actual, message);
  }

  private double[] clipSound(double[] sound, int start, int end) {
    int startIndex = start * 44100;
    int endIndex = end * 44100;
    
    double[] newSound = new double[endIndex - startIndex];

    int index = 0;
    
    while (startIndex < endIndex) {
      newSound[index] = sound[startIndex];
      startIndex++;
      index++;
    }

    return newSound;
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables and Methods (a)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate several Project objects, then call the getNumProjects()
     * method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Project.java`,text:`/*
 * Represents a project on Kickstarter
 */
public class Project {

  private String name;       // The name of a project
  private int numBackers;    // The number of people who have backed a project
  private String category;   // The category of a project

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a static variable called numProjects to 0.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Sets name to the specified name, numBackers to the specified
   * number of backers, and category to the specified category
   */
  public Project(String name, int numBackers, String category) {
    this.name = name;
    this.numBackers = numBackers;
    this.category = category;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Increment numProjects by 1.
     * -----------------------------------------------------------------------------
     */

    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a static method called getNumProjects() that returns a String
   * containing the text "There are {numProjects} on Kickstarter!"
   * -----------------------------------------------------------------------------
   */
  
  

  

  /*
   * Returns a String containing the information about the project
   */
  public String toString() {
    return name + "\\nCategory: " + category + "\\nBackers: " + numBackers;
  }
  
}`}],validationFiles:[{path:`ProjectTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Project.java Test")
public class ProjectTest {

  String message;
  String messageGap = "\\n       ";
  static Project testObject;
  static Class objectClass;
  Field staticField;
  Method staticMethod;
   
  @BeforeAll
  public static void setup() {   
    testObject = new Project("some project", 1000, "some category");
    objectClass = testObject.getClass();
  }

  @Test
  @Order(1)
  @DisplayName("The Project class has a static variable called numProjects => ")
  public void testObjectNumProjectsExists() {
    String variableNotExists = "The Project class does not have a variable called numProjects." + messageGap;
    String variableNotStatic = "The numProjects variable is not static." + messageGap;

    String expected = "numProjects";
    String actual = "MISSING";

    setStaticField(expected);
    int modifierValue = 0;
    
    try {
      actual = staticField.getName();
      modifierValue = staticField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable numProjects is initialized to 0 => ")
  public void testObjectNumProjectIsZero() {
    message = "The static variable numProjects is initialized to a different value than 0." + messageGap;

    setStaticField("numProjects");
    int actual = -1;
    
    try {
      actual = (int) staticField.get(testObject);
    } catch (Exception e) { }
    
    assertEquals(0, actual, message);
  }
   
  @Test
  @Order(3)
  @DisplayName("The Project class has a getNumProjects method => ")
  public void testObjectGetNumProjectsExists() {
    message = "The method getNumProjects() is missing." + messageGap;

    String expectedMethodName = "getNumProjects";
    setStaticMethod(expectedMethodName);
    String staticMethodName = "";

    try {
      staticMethodName = staticMethod.getName();
      assertEquals(expectedMethodName, staticMethodName, message);
    } catch (Exception e) {
      fail("The method getNumProjects() is missing from the Project class." + messageGap);
    }
  }
   
  @Test
  @Order(4)
  @DisplayName("getNumProjects is a static method => ")
  public void testGetNumProjectsIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;

    setStaticMethod("getNumProjects");
    int methodModifiers = -1;

    try {
      methodModifiers = staticMethod.getModifiers();
      assertTrue(Modifier.isStatic(methodModifiers), message);
    } catch(Exception e) {
      fail("The getNumProjects() method is missing from the Project class or is not a static method." + messageGap);
    }
  }
  
  @Test
  @Order(5)
  @DisplayName("getNumProjects returns a String containing the text \\"There are {numProjects} projects on Kickstarter!\\" => ")
  public void testGetNumProjectsResult() {
    message = "Return a String containing the text \\"There are {numProjects} projects on Kickstarter!\\", where";
    message += "\\n        numProjects is the number of Project objects that have been instantiated." + messageGap;

    setStaticMethod("getNumProjects");
    int numObjectsCreated = setRandomObjects();

    String expected = "There are " + numObjectsCreated + " projects on Kickstarter!";
    String actual = "MISSING";

    try {
      actual = (String) staticMethod.invoke(testObject);
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  private void setStaticField(String fieldName) {
    try {
      staticField = objectClass.getDeclaredField(fieldName);
      staticField.setAccessible(true);
      staticField.setInt(testObject, 0);
    } catch (Exception e) { }
  }

  @SuppressWarnings("unchecked")
  private void setStaticMethod(String methodName) {
    try {
     staticMethod = objectClass.getMethod(methodName);
    } catch (Exception e) { }
  }

  private int setRandomObjects() {
    int randomObjects = (int)(Math.random() * 10) + 2;
    
    String testName = "some project";
    int testBackers = 1000;
    String testCategory = "some category";

    for (int count = 0; count < randomObjects; count++) {
      Project aProject = new Project(testName, testBackers, testCategory);
    }

    return randomObjects;
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables and Methods (b)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate several Startup objects, then call the getNumStartups()
     * method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Startup.java`,text:`/*
 * Represents a startup business that appeared on Shark Tank
 */
public class Startup {

  private String name;          // The name of a startup company
  private String description;   // A description of a company's product
  private boolean gotDeal;      // Whether or not a startup company got the deal

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a static variable called numStartups to 0.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Sets name to the specified name, description to the specified
   * description, and gotDeal to the specified status
   */
  public Startup(String name, String description, boolean gotDeal) {
    this.name = name;
    this.description = description;
    this.gotDeal = gotDeal;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Increment numStartups by 1.
     * -----------------------------------------------------------------------------
     */

    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a static method called getNumStartups() that returns a String containing
   * the text "There have been {numStartups} startup pitches on Shark Tank!", where
   * numStartups is the number of Startup objects that have been instantiated.
   * -----------------------------------------------------------------------------
   */

  

  

  /*
   * Returns a String containing information about the startup
   */
  public String toString() {
    String result = name + " - " + description + "\\n";

    if (gotDeal) {
      result += "Got the deal!";
    }
    else {
      result += "No deal :(";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`StartupTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Startup.java Test")
public class StartupTest {

  String message;
  String messageGap = "\\n       ";
  static Startup testObject;
  static Class objectClass;
  Field staticField;
  Method staticMethod;
   
  @BeforeAll
  public static void setup() {
    testObject = new Startup("some company", "some description", true);
    objectClass = testObject.getClass();
  }

  @Test
  @Order(1)
  @DisplayName("The Startup class has a static variable called numStartups => ")
  public void testObjectNumStartupsExists() {
    String variableNotExists = "The Startup class does not have a variable called numStartups." + messageGap;
    String variableNotStatic = "The numStartups variable is not static." + messageGap;

    String expected = "numStartups";
    String actual = "MISSING";

    setStaticField(expected);
    int modifierValue = 0;
    
    try {
      actual = staticField.getName();
      modifierValue = staticField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable numStartups is initialized to 0 => ")
  public void testObjectsNumStartupsIsZero() {
    message = "The static variable numStartups is initialized to a different value than 0." + messageGap;

    setStaticField("numStartups");
    int actual = -1;
    
    try {
      actual = (int) staticField.get(testObject);
    } catch (Exception e) { }

    assertEquals(0, actual, message);
  }
   
  @Test
  @Order(3)
  @DisplayName("The Startup class has a getNumStartups method => ")
  public void testObjectGetNumStartupsExists() {
    message = "The method getNumStartups() is missing." + messageGap;

    String expectedMethodName = "getNumStartups";
    setStaticMethod(expectedMethodName);
    String staticMethodName = "";

    try {
      staticMethodName = staticMethod.getName();
      assertEquals(expectedMethodName, staticMethodName, message);
    } catch (Exception e) {
      fail("The method getNumStartups() is missing from the Startup class." + messageGap);
    }
  }
   
  @Test
  @Order(4)
  @DisplayName("getNumStartups is a static method => ")
  public void testGetNumStartupsIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;

    setStaticMethod("getNumStartups");
    int methodModifiers = -1;
    
    try {
      methodModifiers = staticMethod.getModifiers();
      assertTrue(Modifier.isStatic(methodModifiers), message);
    } catch(Exception e) {
      fail("The getNumStartups() method is missing from the Startup class or is not a static method." + messageGap);
    }
  }
  
  @Test
  @Order(5)
  @DisplayName("getNumStartups returns a String containing the text \\"There have been {numStartups} startup pitches on Shark Tank!\\" => ")
  public void testGetNumStartupsResult() {
    message = "Return a String containing the text \\"There have been {numStartups} startup pitches on Shark Tank!\\", where";
    message += "\\n        numStartups is the number of Startup objects that have been instantiated." + messageGap;

    setStaticMethod("getNumStartups");
    int numObjectsCreated = setRandomObjects();
    
    String expected = "There have been " + numObjectsCreated + " startup pitches on Shark Tank!";
    String actual = "MISSING";

    try {
      actual = (String) staticMethod.invoke(testObject);
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  private void setStaticField(String fieldName) {
    try {
      staticField = objectClass.getDeclaredField(fieldName);
      staticField.setAccessible(true);
      staticField.setInt(testObject, 0);
    } catch (Exception e) { }
  }

  @SuppressWarnings("unchecked")
  private void setStaticMethod(String methodName) {
    try {
     staticMethod = objectClass.getMethod(methodName);
    } catch (Exception e) { }
  }

  private int setRandomObjects() {
    int randomObjects = (int)(Math.random() * 10) + 1;
    
    String testName = "some company";
    String testDescription = "some description";
    boolean testDeal = true;

    for (int count = 0; count < randomObjects; count++) {
      Startup aStartup = new Startup(testName, testDescription, testDeal);
    }

    return randomObjects;
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables and Methods (c)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate several Solver objects, then call the getFastestTime()
     * method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Solver.java`,text:`/*
 * Represents a person that participated in
 * the World Cube Association competition
 */
public class Solver {

  private String name;    // The name of a person
  private int time;       // The time in centiseconds it took a person to solve a Rubik's Cube

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a static variable called fastestTime to 600.
   *    (Start with a large number so that smaller times can replace it.)
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Sets name to the specified name and time
   * to the specified time in centiseconds
   */
  public Solver(String name, int time) {
    this.name = name;
    this.time = time;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Check if the parameter time is smaller than fastestTime.
     *    If this is true, update fastestTime to the value passed to the parameter time.
     * -----------------------------------------------------------------------------
     */

    
    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a static method called getFastestTime() that returns a String
   *    containing the text "The fastest time so far is {fastestTime}", where
   *    fastestTime is the fastest time of all Solver objects.
   * -----------------------------------------------------------------------------
   */

  

  

  /*
   * Returns a String containing information about the solver
   */
  public String toString() {
    return name + ": " + time + " centiseconds";
  }
  
}
`}],validationFiles:[{path:`SolverTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Solver.java Test")
public class SolverTest {

  String message;
  String messageGap = "\\n       ";
  static Solver testObject;
  static Class objectClass;
  Field staticField;
  Method staticMethod;
   
  @BeforeAll
  public static void setup() {
    testObject = new Solver("some person", 200);
    objectClass = testObject.getClass();
  }

  @Test
  @Order(1)
  @DisplayName("The Solver class has a static variable called fastestTime => ")
  public void testObjectFastestTimeExists() {
    String variableNotExists = "The Solver class does not have a variable called fastestTime." + messageGap;
    String variableNotStatic = "The fastestTime variable is not static." + messageGap;

    String expected = "fastestTime";
    String actual = "MISSING";

    setStaticField(expected);
    int modifierValue = 0;
    
    try {
      actual = staticField.getName();
      modifierValue = staticField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable fastestTime is initialized to 0 => ")
  public void testObjectsFastestTimeIsZero() {
    message = "The static variable fastestTime is initialized to a different value than 0." + messageGap;

    setStaticField("fastestTime");
    int actual = -1;
    
    try {
      actual = (int) staticField.get(testObject);
    } catch (Exception e) { }

    assertEquals(0, actual, message);
  }
   
  @Test
  @Order(3)
  @DisplayName("The Solver class has a getFastestTime method => ")
  public void testObjectGetFastestTimeExists() {
    message = "The method getFastestTime() is missing." + messageGap;

    String expectedMethodName = "getFastestTime";
    setStaticMethod(expectedMethodName);
    String staticMethodName = "";

    try {
      staticMethodName = staticMethod.getName();
      assertEquals(expectedMethodName, staticMethodName, message);
    } catch (Exception e) {
      fail("The method getFastestTime() is missing from the Solver class." + messageGap);
    }
  }
   
  @Test
  @Order(4)
  @DisplayName("getFastestTime is a static method => ")
  public void testGetFastestTimeIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;

    setStaticMethod("getFastestTime");
    int methodModifiers = -1;
    
    try {
      methodModifiers = staticMethod.getModifiers();
      assertTrue(Modifier.isStatic(methodModifiers), message);
    } catch(Exception e) {
      fail("The getFastestTime() method is missing from the Solver class or is not a static method." + messageGap);
    }
  }
  
  @Test
  @Order(5)
  @DisplayName("getFastestTime returns a String containing the text \\"The fastest time so far is {fastestTime}\\" => ")
  public void testGetFastestTimeResult() {
    message = "Return a String containing the text \\"The fastest time so far is {fastestTime}\\", where";
    message += "\\n        fastestTime is the fastest time of all Solver objects." + messageGap;

    setStaticMethod("getFastestTime");
    int[] times = getTimes();
    setRandomObjects(times);
    int fastestTime = getFastestTime(times);
    
    String expected = "The fastest time so far is " + fastestTime;
    String actual = "MISSING";

    try {
      actual = (String) staticMethod.invoke(testObject);
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  private void setStaticField(String fieldName) {
    try {
      staticField = objectClass.getDeclaredField(fieldName);
      staticField.setAccessible(true);
      staticField.setInt(testObject, 0);
    } catch (Exception e) { }
  }

  @SuppressWarnings("unchecked")
  private void setStaticMethod(String methodName) {
    try {
     staticMethod = objectClass.getMethod(methodName);
    } catch (Exception e) { }
  }

  private int getRandomNumObjects() {
    int randomObjects = (int)(Math.random() * 10) + 1;
    return randomObjects;
  }

  private int[] getTimes() {
    int numObjects = getRandomNumObjects();
    int[] times = new int[numObjects];

    for (int index = 0; index < times.length; index++) {
      times[index] = (int)(Math.random() * 500) + 100;
    }

    return times;    
  }

  private int getFastestTime(int[] times) {
    int fastest = times[0];

    for (int value : times) {
      if (value < fastest) {
        fastest = value;
      }
    }

    return fastest;
  }

  private void setRandomObjects(int[] times) {
    String testName = "some name";
    Solver[] testSolvers = new Solver[times.length];

    for (int index = 0; index < testSolvers.length; index++) {
      testSolvers[index] = new Solver(testName, times[index]);
    }
  }

}`}],dataFiles:[]},{name:`Practice: Using Static Variables and Methods (d)`,lesson:`Lesson 2: Static Variables and Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate several Concert objects, then call the getMaxAttendance()
     * method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Concert.java`,text:`/*
 * Represents a band's concert during a tour
 */
public class Concert {

  private String location;     // The location of a concert
  private int attendance;      // The number of people that attended a concert

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a static variable called maxAttendance to 0.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Sets location to the specified location and
   * attendance to the specified attendance
   */
  public Concert(String location, int attendance) {
    this.location = location;
    this.attendance = attendance;

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Check if the parameter attendance is greater than maxAttendance. If this
     * is true, update the maxAttendance to the value passed to the parameter attendance.
     * -----------------------------------------------------------------------------
     */

    
    
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a static method called getMaxAttendance() that returns a String
   * containing the text "The max attendance at an event is {maxAttendance}", 
   * where maxAttendance is the largest attendance at an event.
   * -----------------------------------------------------------------------------
   */

  

  

  /*
   * Returns a String containing information about the concert
   */
  public String toString() {
    return location + ": " + attendance + " people in attendance";
  }
  
}`}],validationFiles:[{path:`ConcertTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Concert.java Test")
public class ConcertTest {

  String message;
  String messageGap = "\\n       ";
  static Concert testObject;
  static Class objectClass;
  Field staticField;
  Method staticMethod;
   
  @BeforeAll
  public static void setup() {
    testObject = new Concert("some location", 2000);
    objectClass = testObject.getClass();
  }

  @Test
  @Order(1)
  @DisplayName("The Concert class has a static variable called maxAttendance => ")
  public void testObjectMaxAttendanceExists() {
    String variableNotExists = "The Concert class does not have a variable called maxAttendance." + messageGap;
    String variableNotStatic = "The maxAttendance variable is not static." + messageGap;

    String expected = "maxAttendance";
    String actual = "MISSING";

    setStaticField(expected);
    int modifierValue = 0;
    
    try {
      actual = staticField.getName();
      modifierValue = staticField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, variableNotExists);
    assertTrue(Modifier.isStatic(modifierValue), variableNotStatic);
  }
   
  @Test
  @Order(2)
  @DisplayName("The static variable maxAttendance is initialized to 0 => ")
  public void testObjectsMaxAttendanceIsZero() {
    message = "The static variable maxAttendance is initialized to a different value than 0." + messageGap;

    setStaticField("maxAttendance");
    int actual = -1;
    
    try {
      actual = (int) staticField.get(testObject);
    } catch (Exception e) { }

    assertEquals(0, actual, message);
  }
   
  @Test
  @Order(3)
  @DisplayName("The Concert class has a getMaxAttendance method => ")
  public void testObjectGetMaxAttendanceExists() {
    message = "The method getMaxAttendance() is missing." + messageGap;

    String expectedMethodName = "getMaxAttendance";
    setStaticMethod(expectedMethodName);
    String staticMethodName = "";

    try {
      staticMethodName = staticMethod.getName();
      assertEquals(expectedMethodName, staticMethodName, message);
    } catch (Exception e) {
      fail("The method getMaxAttendance() is missing from the Concert class." + messageGap);
    }
  }
   
  @Test
  @Order(4)
  @DisplayName("getMaxAttendance is a static method => ")
  public void testGetMaxAttendanceIsStatic() {
    message = "Make sure to include the keyword static in the method signature." + messageGap;

    setStaticMethod("getMaxAttendance");
    int methodModifiers = -1;
    
    try {
      methodModifiers = staticMethod.getModifiers();
      assertTrue(Modifier.isStatic(methodModifiers), message);
    } catch(Exception e) {
      fail("The getMaxAttendance() method is missing from the Concert class or is not a static method." + messageGap);
    }
  }
  
  @Test
  @Order(5)
  @DisplayName("getMaxAttendance returns a String containing the text \\"The max attendance at an event is {maxAttendance}!\\" => ")
  public void testGetMaxAttendanceResult() {
    message = "Return a String containing the text \\"The max attendance at an event is {maxAttendance}!\\", where";
    message += "\\n        maxAttendance is the largest attendance at an event." + messageGap;

    setStaticMethod("getMaxAttendance");
    int[] randomAttendance = getAttendance();
    setRandomObjects(randomAttendance);
    int testMax = getTestMaxAttendance(randomAttendance);
    
    String expected = "The max attendance at an event is " + testMax + "!";
    String actual = "MISSING";

    try {
      actual = (String) staticMethod.invoke(testObject);
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  private void setStaticField(String fieldName) {
    try {
      staticField = objectClass.getDeclaredField(fieldName);
      staticField.setAccessible(true);
      staticField.setInt(testObject, 0);
    } catch (Exception e) { }
  }

  @SuppressWarnings("unchecked")
  private void setStaticMethod(String methodName) {
    try {
     staticMethod = objectClass.getMethod(methodName);
    } catch (Exception e) { }
  }

  private int getRandomNumObjects() {
    int randomObjects = (int)(Math.random() * 10) + 1;
    return randomObjects;
  }

  private int[] getAttendance() {
    int numObjects = getRandomNumObjects();
    int[] randomAttendance = new int[numObjects];

    for (int index = 0; index < randomAttendance.length; index++) {
      randomAttendance[index] = (int)(Math.random() * 50000) + 20000;
    }

    return randomAttendance;    
  }

  private int getTestMaxAttendance(int[] randomAttendance) {
    int max = randomAttendance[0];

    for (int value : randomAttendance) {
      if (value > max) {
        max = value;
      }
    }

    return max;
  }

  private void setRandomObjects(int[] randomAttendance) {
    String testLocation = "some location";
    Concert[] testConcerts = new Concert[randomAttendance.length];

    for (int index = 0; index < testConcerts.length; index++) {
      testConcerts[index] = new Concert(testLocation, randomAttendance[index]);
    }
  }

}`}],dataFiles:[]},{name:`Predict and Run: The Math Class`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int number = 25;
    double firstResult = Math.sqrt(number);
    System.out.println("The result of Math.sqrt(" + number + ") is " + firstResult);

    number = 3;
    double secondResult = Math.pow(number, 2);
    System.out.println("The result of Math.pow(" + number + ") is " + secondResult);

    number = -16;
    double thirdResult = Math.abs(number);
    System.out.println("The result of Math.abs(" + number + ") is " + thirdResult);
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The Math Class`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {





    


    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Using Constants (a)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Rollercoaster objects
    Rollercoaster[] coasters = { new Rollercoaster("Shamu Express", 23), new Rollercoaster("Quicksilver Express", 51),
                               new Rollercoaster("Tig'rr Coaster", 50), new Rollercoaster("Pony Express", 62),
                               new Rollercoaster("Coastersaurus", 32), new Rollercoaster("Cobra", 35)};

    // Instantiates a TravelAgency with the array of rollercoasters
    TravelAgency agency = new TravelAgency(coasters);

    // Calls the getChildFriendly() method and prints the result
    System.out.println(agency.getChildFriendly());    
    
  }
}`},{path:`Rollercoaster.java`,text:`/*
 * Represents a roller coaster
 */
public class Rollercoaster {

  private String name;
  private int speed;

  /*
   * Sets name to the specified name and speed to
   * the specified speed to the roller coaster
   */
  public Rollercoaster(String name, int speed) {
    this.name = name;
    this.speed = speed;
  }

  /*
   * Returns the speed of the roller coaster
   */
  public int getSpeed() {
    return speed;
  }

  /*
   * Returns a String containing information about the roller coaster
   */
  public String toString() {
    return name + ": " + speed + " mph";
  }
  
}`},{path:`TravelAgency.java`,text:`/*
 * Manages data about roller coasters
 */
public class TravelAgency {

  private Rollercoaster[] rollercoasters;    // The 1D array of rollercoasters

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a constant called CHILD_MAX_SPEED to 35.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Initializes rollercoasters to the specified 1D array of rollercoasters
   */
  public TravelAgency(Rollercoaster[] rollercoasters) {
    this.rollercoasters = rollercoasters;
  }

  /*
   * Returns a String containing rollercoasters that are
   * considered child-friendly (under 35 mph)
   */
  public String getChildFriendly() {
    String result = "Child-Friendly Roller Coasters\\n---------------\\n";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Update the condition to use the constant.
     * -----------------------------------------------------------------------------
     */

    for (Rollercoaster coaster : rollercoasters) {
      if (coaster.getSpeed() <= 35) {
        result += coaster + "\\n";
      }
    }

    return result;
  }
  
}`}],validationFiles:[{path:`TravelAgencyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TravelAgency.java Test")
public class TravelAgencyTest {

  String message;
  String messageGap = "\\n       ";
  TravelAgency testObject;
  Class testClass;
  Field constantField;
   
  @BeforeEach
  public void setup() {
    Rollercoaster[] testCoasters = getRandomCoasters();
    testObject = new TravelAgency(testCoasters);
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The TravelAgency class has a constant called CHILD_MAX_SPEED => ")
  public void testConstantExists() {
    String constantNotExists = "The TravelAgency class does not have a variable called CHILD_MAX_SPEED." + messageGap;
    String constantNotStatic = "The CHILD_MAX_SPEED variable is not static." + messageGap;
    String constantNotFinal = "The CHILD_MAX_SPEED variable is not final." + messageGap;

    String expected = "CHILD_MAX_SPEED";
    String actual = "MISSING";

    setConstantField(expected);
    int modifierValue = 0;

    try {
      actual = constantField.getName();
      modifierValue = constantField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, constantNotExists);
    assertTrue(Modifier.isStatic(modifierValue), constantNotStatic);
    assertTrue(Modifier.isFinal(modifierValue), constantNotFinal);
  }
   
  @Test
  @Order(2)
  @DisplayName("The constant CHILD_MAX_SPEED is initialized to 35 => ")
  public void testConstantInitialValueIsZero() {
    message = "The constant CHILD_MAX_SPEED is initialized to a different value than 35." + messageGap;

    int expected = 35;
    setConstantField("CHILD_MAX_SPEED");
    int actual = -1;
    
    try {
      actual = (int) constantField.get(testObject);
    } catch (Exception e) { }
    
    assertEquals(expected, actual, message);
  }

  private Rollercoaster[] getRandomCoasters() {
    int numObjects = (int)(Math.random() * 10) + 1;
    Rollercoaster[] temp = new Rollercoaster[numObjects];

    for (int index = 0; index < temp.length; index++) {
      int randomSpeed = (int)(Math.random() * 50) + 10;
      temp[index] = new Rollercoaster("some coaster", randomSpeed);
    }

    return temp;
  }

  private void setConstantField(String fieldName) {
    try {
      constantField = testClass.getDeclaredField(fieldName);
      constantField.setAccessible(true);
    } catch (Exception e) { }
  }

}`}],dataFiles:[]},{name:`Practice: Using Constants (b)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Country objects
    Country[] countries = {new Country("Grenada", 22), new Country("India", 332), new Country("Chile", 51), new Country("Netherlands", 29),
                           new Country("Turkey", 180), new Country("Costa Rica", 13), new Country("Egypt", 86), new Country("Mexico", 162)};

    // Instantiates an AirQuality with the array of countries
    AirQuality data = new AirQuality(countries);

    // Calls the getUnsatisfactory() method and prints the result
    System.out.println(data.getUnsatisfactory());
    
  }
}`},{path:`AirQuality.java`,text:`/*
 * Manages data about air quality in countries
 */
public class AirQuality {

  private Country[] countries;    // The 1D array of countries

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a constant called MAX_AQI_INDEX to 100.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Initializes countries to the specified 1D array of countries
   */
  public AirQuality(Country[] countries) {
    this.countries = countries;
  }

  /*
   * Returns a String containing the name and air quality index of
   * each country that is greater than the maximum acceptable index
   */
  public String getUnsatisfactory() {
    String result = "Countries Over the Maximum Acceptable Index\\n---------------\\n";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Update the condition to use the constant.
     * -----------------------------------------------------------------------------
     */

    for (Country country : countries) {
      if (country.getIndex() > 100) {
        result += country + "\\n";
      }
    }

    return result;
  }
  
}`},{path:`Country.java`,text:`/*
 * Represents a country
 */
public class Country {

  private String name;    // The name of a country
  private int index;      // The air quality index of a country

  /*
   * Sets name and index to the specified name and index
   */
  public Country(String name, int index) {
    this.name = name;
    this.index = index;
  }

  /*
   * Returns the index of the country
   */
  public int getIndex() {
    return index;
  }

  /*
   * Returns a String containing the name and index of the country
   */
  public String toString() {
    return name + ": " + index + " AQI";
  }
  
}`}],validationFiles:[{path:`AirQualityTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("AirQuality.java Test")
public class AirQualityTest {

  String message;
  String messageGap = "\\n       ";
  AirQuality testObject;
  Class testClass;
  Field constantField;
   
  @BeforeEach
  public void setup() {
    Country[] testCountries = getRandomCountries();
    testObject = new AirQuality(testCountries);
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The AirQuality class has a constant called MAX_AQI_INDEX => ")
  public void testConstantExists() {
    String constantNotExists = "The AirQuality class does not have a variable called MAX_AQI_INDEX." + messageGap;
    String constantNotStatic = "The MAX_AQI_INDEX variable is not static." + messageGap;
    String constantNotFinal = "The MAX_AQI_INDEX variable is not final." + messageGap;

    String expected = "MAX_AQI_INDEX";
    String actual = "MISSING";

    setConstantField(expected);
    int modifierValue = 0;

    try {
      actual = constantField.getName();
      modifierValue = constantField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, constantNotExists);
    assertTrue(Modifier.isStatic(modifierValue), constantNotStatic);
    assertTrue(Modifier.isFinal(modifierValue), constantNotFinal);
  }
   
  @Test
  @Order(2)
  @DisplayName("The constant MAX_AQI_INDEX is initialized to 100 => ")
  public void testConstantInitialValueIsOneHundred() {
    message = "The constant MAX_AQI_INDEX is initialized to a different value than 100." + messageGap;

    int expected = 100;
    setConstantField("MAX_AQI_INDEX");
    int actual = -1;
    
    try {
      actual = (int) constantField.get(testObject);
    } catch (Exception e) { }
    
    assertEquals(expected, actual, message);
  }

  private Country[] getRandomCountries() {
    int numObjects = (int)(Math.random() * 20) + 10;
    Country[] temp = new Country[numObjects];

    for (int index = 0; index < temp.length; index++) {
      int randomIndex = (int)(Math.random() * 300) + 20;
      temp[index] = new Country("some country", randomIndex);
    }

    return temp;
  }

  private void setConstantField(String fieldName) {
    try {
      constantField = testClass.getDeclaredField(fieldName);
      constantField.setAccessible(true);
    } catch (Exception e) { }
  }

}`}],dataFiles:[]},{name:`Practice: Using Constants (c)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Country objects
    Country[] countries = {new Country("Greece", 6.7), new Country("Denmark", 2.89), new Country("Italy", 4.42), new Country("Dominica", 28.47),
                           new Country("Guatemala", 20.88), new Country("Japan", 12.99), new Country("Philippines", 26.7)};

    // Instantiates a ClimateRisk with the array of countries
    ClimateRisk data = new ClimateRisk(countries);

    // Calls the getHighRiskCountries() method and prints the result
    System.out.println(data.getHighRiskCountries());
    
  }
}`},{path:`ClimateRisk.java`,text:`/*
 * Manages data about climate risk index for countries
 */
public class ClimateRisk {

  private Country[] countries;    // The 1D array of countries

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a constant called MAX_HIGH_RISK to 20.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Initializes countries to the specified 1D array of countries
   */
  public ClimateRisk(Country[] countries) {
    this.countries = countries;
  }

  /*
   * Returns a String containing the name and climate risk index of each country
   * that is less than or equal to the maximum index for high risk countries
   */
  public String getHighRiskCountries() {
    String result = "Countries Less Than the Maximum High Risk Index\\n---------------\\n";

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Update the condition to use the constant.
     * -----------------------------------------------------------------------------
     */

    for (Country country : countries) {
      if (country.getIndex() <= 20) {
        result += country + "\\n";
      }
    }

    return result;
  }
  
}`},{path:`Country.java`,text:`/*
 * Represents a country
 */
public class Country {

  private String name;       // The name of a country
  private double index;      // The climate risk index of a country

  /*
   * Sets name and index to the specified name and index
   */
  public Country(String name, double index) {
    this.name = name;
    this.index = index;
  }

  /*
   * Returns the index of the country
   */
  public double getIndex() {
    return index;
  }

  /*
   * Returns a String containing the name and index of the country
   */
  public String toString() {
    return name + ": " + index + " Climate Risk Index";
  }
  
}`}],validationFiles:[{path:`ClimateRiskTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ClimateRisk.java Test")
public class ClimateRiskTest {

  String message;
  String messageGap = "\\n       ";
  ClimateRisk testObject;
  Class testClass;
  Field constantField;
   
  @BeforeEach
  public void setup() {
    Country[] testCountries = getRandomCountries();
    testObject = new ClimateRisk(testCountries);
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The ClimateRisk class has a constant called MAX_HIGH_RISK => ")
  public void testConstantExists() {
    String constantNotExists = "The ClimateRisk class does not have a variable called MAX_HIGH_RISK." + messageGap;
    String constantNotStatic = "The MAX_HIGH_RISK variable is not static." + messageGap;
    String constantNotFinal = "The MAX_HIGH_RISK variable is not final." + messageGap;

    String expected = "MAX_HIGH_RISK";
    String actual = "MISSING";

    setConstantField(expected);
    int modifierValue = 0;

    try {
      actual = constantField.getName();
      modifierValue = constantField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, constantNotExists);
    assertTrue(Modifier.isStatic(modifierValue), constantNotStatic);
    assertTrue(Modifier.isFinal(modifierValue), constantNotFinal);
  }
   
  @Test
  @Order(2)
  @DisplayName("The constant MAX_HIGH_RISK is initialized to 20 => ")
  public void testConstantInitialValueIsTwenty() {
    message = "The constant MAX_HIGH_RISK is initialized to a different value than 20." + messageGap;

    double expected = 20;
    setConstantField("MAX_HIGH_RISK");
    double actual = -1;
    
    try {
      actual = (double) constantField.get(testObject);
    } catch (Exception e) { }
    
    assertEquals(expected, actual, message);
  }

  private Country[] getRandomCountries() {
    int numObjects = (int)(Math.random() * 20) + 10;
    Country[] temp = new Country[numObjects];

    for (int index = 0; index < temp.length; index++) {
      double randomIndex = Math.random() * 30 + 1;
      temp[index] = new Country("some country", randomIndex);
    }

    return temp;
  }

  private void setConstantField(String fieldName) {
    try {
      constantField = testClass.getDeclaredField(fieldName);
      constantField.setAccessible(true);
    } catch (Exception e) { }
  }

}`}],dataFiles:[]},{name:`Practice: Using Constants (d)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Instantiates a Person object
    Person person = new Person(54, 25);

    // Instantiates a TransitCalculator with the person
    TransitCalculator calc = new TransitCalculator(person);

    // Calls the getWeeklyPricePerRide() method and prints the result
    System.out.println("The price per ride using the weekly pass is " + calc.getWeeklyPricePerRide());
    
  }
}`},{path:`Person.java`,text:`/*
 * Represents a person using the NYC transit system
 */
public class Person {

  private int numDays;    // The number of days a person will be using the transit system
  private int numRides;   // The number of individual rides the person expects to take in that time

  /*
   * Sets numDays and numRides to the specified number of days and rides
   */
  public Person(int numDays, int numRides) {
    this.numDays = numDays;
    this.numRides = numRides;
  }

  /*
   * Returns the number of days
   */
  public int getNumDays() {
    return numDays;
  }

  /*
   * Returns the number of rides
   */
  public int getNumRides() {
    return numRides;
  }
  
}`},{path:`TransitCalculator.java`,text:`/*
 * Performs fare calculations
 */
public class TransitCalculator {

  private Person person;    // The person using the NYC transit system

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare and initialize a constant called PRICE_PER_WEEK to 33.00.
   * -----------------------------------------------------------------------------
   */
  
  

  /*
   * Sets person to the specified person
   */
  public TransitCalculator(Person person) {
    this.person = person;
  }

  /*
   * Returns the number of weekly passes needed for the person
   */
  public int getNumWeeklyPasses() {
    int numWeekPasses = person.getNumDays() / 7;

    if (numWeekPasses % person.getNumDays() > 0) {
      numWeekPasses++;
    }

    return numWeekPasses;
  }

  /*
   * Returns the overall price per ride using the weekly option
   */
  public double getWeeklyPricePerRide() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate the total price for the number of weekly prices needed and
     * the cost per ride, then return the cost per ride.
     * -----------------------------------------------------------------------------
     */

    return 0;
  }

}`}],validationFiles:[{path:`TransitCalculatorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Field;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TransitCalculator.java Test")
public class TransitCalculatorTest {

  String message;
  String messageGap = "\\n       ";
  Person testPerson;
  TransitCalculator testObject;
  Class testClass;
  Field constantField;
   
  @BeforeEach
  public void setup() {
    testPerson = getRandomPerson();
    testObject = new TransitCalculator(testPerson);
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("The TransitCalculator class has a constant called PRICE_PER_WEEK => ")
  public void testConstantExists() {
    String constantNotExists = "The TransitCalculator class does not have a variable called PRICE_PER_WEEK." + messageGap;
    String constantNotStatic = "The PRICE_PER_WEEK variable is not static." + messageGap;
    String constantNotFinal = "The PRICE_PER_WEEK variable is not final." + messageGap;

    String expected = "PRICE_PER_WEEK";
    String actual = "MISSING";

    setConstantField(expected);
    int modifierValue = 0;

    try {
      actual = constantField.getName();
      modifierValue = constantField.getModifiers();
    } catch (Exception e) { }

    assertEquals(expected, actual, constantNotExists);
    assertTrue(Modifier.isStatic(modifierValue), constantNotStatic);
    assertTrue(Modifier.isFinal(modifierValue), constantNotFinal);
  }
   
  @Test
  @Order(2)
  @DisplayName("The constant PRICE_PER_WEEK is initialized to 33.00 => ")
  public void testConstantInitialValueIsThirtyThree() {
    message = "The constant PRICE_PER_WEEK is initialized to a different value than 33.00." + messageGap;

    double expected = 33.00;
    setConstantField("PRICE_PER_WEEK");
    double actual = -1;
    
    try {
      actual = (double) constantField.get(testObject);
    } catch (Exception e) { }
    
    assertEquals(expected, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("getWeeklyPricePerRide() returns the total cost per ride for a Person => ")
  public void testGetWeeklyPricePerRide() {
    message = "getWeeklyPricePerRide() should return the total price of weekly passes needed divided by the number of rides." + messageGap;

    double expected = expectedWeeklyPricePerRide();
    double actual = testObject.getWeeklyPricePerRide();

    assertEquals(expected, actual, message);
  }

  private Person getRandomPerson() {
    int randomDays = (int)(Math.random() * 60) + 1;
    int randomRides = (int)(Math.random() * 60) + 1;
    
    Person temp = new Person(randomDays, randomRides);

    return temp;
  }

  private void setConstantField(String fieldName) {
    try {
      constantField = testClass.getDeclaredField(fieldName);
      constantField.setAccessible(true);
    } catch (Exception e) { }
  }

  private double expectedWeeklyPricePerRide() {
    int numWeekPasses = testObject.getNumWeeklyPasses();
    double totalWeekCost = numWeekPasses * 33.0;
    return totalWeekCost / testPerson.getNumRides();
  }

}`}],dataFiles:[]},{name:`Practice: Using Math.pow() (a)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use the promptUser() method in the Calculator class to get two numbers
     * from the user, then call the calcExponent() method with the two numbers
     * provided and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Calculator.java`,text:`import java.util.Scanner;

/*
 * Represents a calculator that performs calculations
 */
public class Calculator {

  public static String calcExponent(double firstNumber, double secondNumber) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate the result of the parameter firstNumber raised to the power of
     * the parameter secondNumber, then return a String containing the text
     * "{firstNumber} raised to the power of {secondNumber} is {result}"
     * -----------------------------------------------------------------------------
     */


    return "";
  }

  /*
   * Prompts a user for a number and returns the number entered
   */
  public static double promptUser() {
    Scanner input = new Scanner(System.in);
    System.out.print("Enter a number: ");
    double number = input.nextDouble();
    input.close();
    return number;
  }
  
}`}],validationFiles:[{path:`CalculatorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Calculator.java Test")
public class CalculatorTest {

  String messageGap = "\\n       ";
  double[] randomFirstNumbers;
  double[] randomSecondNumbers;
   
  @BeforeEach
  public void setup() {
    randomFirstNumbers = getRandomNumbers();
    randomSecondNumbers = getRandomNumbers();
  }
   
  @Test
  @Order(1)
  @DisplayName("calcExponent() returns the result of firstNumber raised to secondNumber => ")
  public void testCalcExponent() {
    String message = "Pass the parameter firstNumber as the first argument to Math.pow() and the parameter secondNumber as the second argument." + messageGap;
      
    for (int index = 0; index < randomFirstNumbers.length; index++) {
      double expectedResult = Math.pow(randomFirstNumbers[index], randomSecondNumbers[index]);
      String actualString = Calculator.calcExponent(randomFirstNumbers[index], randomSecondNumbers[index]);
      String expectedString = randomFirstNumbers[index] + " raised to the power of " + randomSecondNumbers[index] + " is " + expectedResult;
      assertEquals(expectedString, actualString, message);
    }
  }

  private double[] getRandomNumbers() {
    double[] temp = new double[10];

    for (int index = 0; index < temp.length; index++) {
      double random = Math.random() * 20 + 5;
      temp[index] = random;
    }

    return temp;
  }
 
}`}],dataFiles:[]},{name:`Practice: Using Math.pow() (b)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a City object
    City manhattan = new City("Manhattan", 250.3);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calculateGrowth() method to get the estimated growth,
     * and print the current and estimated new population.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`City.java`,text:`/*
 * Represents a city
 */
public class City {

  private String name;                 // The name of a city
  private double currentPopulation;    // The current population of a city

  /*
   * Sets name to the specified name and  current population to the specified value
   */
  public City(String name, double currentPopulation) {
    this.name = name;
    this.currentPopulation = currentPopulation;
  }

  /*
   * Returns the current population
   */
  public double getCurrentPopulation() {
    return currentPopulation;
  }

  /*
   * Returns the new population
   */
  public double calculateGrowth(double growthRate, double time) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate the estimated new population using the researcher's formula
     * and return the result.
     * -----------------------------------------------------------------------------
     */
    
    return 0;
  }

  /*
   * Returns a String containing the name of the city and the current population
   */
  public String toString() {
    return name + " - Current Population: " + currentPopulation;
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
  City testCity;
   
  @BeforeEach
  public void setup() {
    int randomValue = (int)(Math.random() * 200) + 50;
    testCity = new City("some city", randomValue);
  }
   
  @Test
  @Order(1)
  @DisplayName("calculateResult() returns the estimated new population using the researcher's formula => ")
  public void testCalculateResult() {
    String message = "Use Math.pow() to raise 2.7 to the power of (growthRate * time).\\n";
    message += "        Multiply the result of this by the current population." + messageGap;
      
    double expected = testCity.getCurrentPopulation() * Math.pow(2.7, (2 * 5));
    double actual = testCity.calculateGrowth(2, 5);

    assertEquals(expected, actual, message);
  }
 
}`}],dataFiles:[]},{name:`Practice: Using Math.pow() (c)`,lesson:`Lesson 3: The Math Class`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a ComputingScene object
    ComputingScene growth = new ComputingScene(1960, 151);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcGrowth() method and pass the result to a call to the
     * drawResult() method.
     * -----------------------------------------------------------------------------
     */




    
    

    // Plays the scene
    Theater.playScenes(growth);
    
  }
}`},{path:`ComputingScene.java`,text:`import org.code.theater.*;

public class ComputingScene extends Scene {

  private int year;                                  // The year to start the scene
  private double startPower;                         // The calculations per second of the year
  public static final double GROWTH_RATE = 1.2;      // The average growth rate of the calculations per second

  /*
   * Sets year to the specified year and startPower to the specified value
   */
  public ComputingScene(int year, double startPower) {
    this.year = year;
    this.startPower = startPower;
  }

  /*
   * Returns the year to start the scene
   */
  public int getYear() {
    return year;
  }

  /*
   * Returns the starting calculations per second for the year
   */
  public double getStartPower() {
    return startPower;
  }

  /*
   * Returns the estimated computing power in numYears
   */
  public double calcGrowth(int numYears) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate the estimated growth of computing power using the formula.
     * -----------------------------------------------------------------------------
     */
    
    return 0;
  }

  /*
   * Draws an image of a processor in the scene
   */
  public void drawImage() {
    int imageWidth = 150;
    int xPosition = (getWidth() / 2) - (imageWidth / 2);
    drawImage("processor.png", xPosition, 110, imageWidth);
  }

  /*
   * Draws the result in the scene
   */
  public void drawResult(double result, int numYears) {
    double difference = result - startPower;
    
    drawText("Computing Power in " + year, 75, 65);
    drawText("was " + startPower, 75, 90);
    pause(0.5);

    drawImage();
    pause(0.5);

    drawText("In " + (year + numYears) + ", it could increase", 75, 300);
    pause(0.5);
    drawText("by " + difference + " to ", 75, 325);
    pause(0.5);
    drawText(result + "", 75, 350);
  }
  
}`}],validationFiles:[{path:`ComputingSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ComputingScene.java Test")
public class ComputingSceneTest {

  String messageGap = "\\n       ";
  ComputingScene testScene;
   
  @BeforeEach
  public void setup() {
    int randomYear = (int)(Math.random() * 124) + 1900;
    double randomStartPower = Math.random() * 300;
    testScene = new ComputingScene(randomYear, randomStartPower);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcGrowth() returns the estimated growth of computing power => ")
  public void testCalcGrowth() {
    String message = "Use Math.pow() to raise the GROWTH_RATE to the power of numYears.\\n";
    message += "        Multiply the result of this by the starting computing power." + messageGap;
      
    double expected = testScene.getStartPower() * Math.pow(ComputingScene.GROWTH_RATE, 10);
    double actual = testScene.calcGrowth(10);

    assertEquals(expected, actual, message);
  }
 
}`}],dataFiles:[]},{name:`Practice: Using Math.pow() (d)`,lesson:`Lesson 3: The Math Class`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Creator object
    Creator reach = new Creator(10);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcReach() method, then call the drawResults() method with
     * the 1D array returned.
     * -----------------------------------------------------------------------------
     */





    

    // Plays the scene
    Theater.playScenes(reach);
    
  }
}`},{path:`Creator.java`,text:`import org.code.theater.*;

public class Creator extends Scene {

  private int startNum;      // The starting number of people

  /*
   * Sets startNum to the specified value
   */
  public Creator(int startNum) {
    this.startNum = startNum;
  }

  /*
   * Returns the starting number of people
   */
  public int getStartNum() {
    return startNum;
  }

  public double[] calcReach(int numLevels) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return a 1D array containing the number of people a post
     * could potentially reach.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return null;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(double[] results) {
    int xLocation = 50;
    int yLocation = 50;
    
    for (int index = 0; index < results.length; index++) {
      drawImage("message.png", xLocation, yLocation, 50);
      drawText(results[index] + " people reached", xLocation + 60, yLocation + 30);
      yLocation += 50;
      pause(0.1);
    }
  }
  
}`}],validationFiles:[{path:`CreatorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Creator.java Test")
public class CreatorTest {

  String messageGap = "\\n       ";
  Creator testCreator;
   
  @BeforeEach
  public void setup() {
    int randomStart = (int)(Math.random() * 10) + 5;
    testCreator = new Creator(randomStart);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcReach() returns a 1D array containing the number of people a post could reach => ")
  public void testCalcReach() {
    String message = "Use Math.pow() to raise the value at the previous index to the power of the index + 1." + messageGap;
      
    double[] expected = getExpectedResults(3);
    double[] actual = testCreator.calcReach(3);

    assertArrayEquals(expected, actual, message);
  }

  private double[] getExpectedResults(int numLevels) {
    double[] results = new double[numLevels];
    results[0] = testCreator.getStartNum();

    for (int index = 1; index < results.length; index++) {
      results[index] = Math.pow(results[index - 1], index + 1);
    }
    
    return results;
  }
 
}`}],dataFiles:[]},{name:`Practice: Using Math.abs() (a)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of sales amounts
    double[] sales = {-4.05, 5.06, 3.99, -3.99, -1.99, 2.05, 10.50, -7.85, 6.73, -8.45};

    // Creates a Sales object
    Sales business = new Sales(sales);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the Sales.salesToString() method with the 1D array returned from
     * calling the cleanData() method.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Sales.java`,text:`/*
 * Represents sales made at a business
 */
public class Sales {

  private double[] sales;   // The 1D array of sales

  /*
   * Initializes sales to the specified 1D array of sales
   */
  public Sales(double[] sales) {
    this.sales = sales;
  }

  /*
   * Returns the 1D array of sales
   */
  public double[] getSales() {
    return sales;
  }

  /*
   * Returns a new 1D array containing the corrected sales values
   */
  public double[] cleanData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the sales array and add the absolute value of each element to
     * a new 1D array. Return the new 1D array.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return null;
  }

  /*
   * Returns a String containing the sales values
   */
  public static String salesToString(double[] values) {
    String result = "";

    for (double amt : values) {
      result += "$" + amt + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`SalesTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Sales.java Test")
public class SalesTest {

  String message;
  String messageGap = "\\n       ";
  double[] testData;
  Sales testSales;
   
  @BeforeEach
  public void setup() {
    testData = new double[]{-100.0, 50.0, -25.0, 75.0};
    testSales = new Sales(testData);
  }
   
  @Test
  @Order(1)
  @DisplayName("cleanData() returns a new 1D array containing the corrected sales values => ")
  public void testCleanData() {
    message = "Traverse the sales array and add the absolute value of each element to a new 1D array." + messageGap;
      
    double[] expected = {100.0, 50.0, 25.0, 75.0};
    double[] actual = testSales.cleanData();

    assertArrayEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Using Math.abs() (b)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Patient objects
    Patient[] patients = {new Patient("Olivia", 98.2), new Patient("Liam", 98.9), new Patient("Ava", 102.3),
                          new Patient("Ethan", 98.4), new Patient("Isabella", 99.6)};

    // Creates a Physician object
    Physician doctor = new Physician(patients);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getSickPatients() method and print the result.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Patient.java`,text:`/*
 * Represents a patient
 */
public class Patient {

  private String name;    // The name of a patient
  private double temp;    // The temperature of a patient

  /*
   * Sets name and temp to the specified values
   */
  public Patient(String name, double temp) {
    this.name = name;
    this.temp = temp;
  }

  /*
   * Returns the name of the patient
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the temperature of the patient
   */
  public double getTemp() {
    return temp;
  }

  /*
   * Returns a String containing the patient's name and temperature
   */
  public String toString() {
    return name + " - " + temp;
  }
  
}`},{path:`Physician.java`,text:`/*
 * Represents a physician
 */
public class Physician {

  private Patient[] patients;     // A physician's patients

  /*
   * Initializes patients to the specified 1D array of patients
   */
  public Physician(Patient[] patients) {
    this.patients = patients;
  }

  /*
   * Returns the 1D array of patients
   */
  public Patient[] getPatients() {
    return patients;
  }

  /*
   * Finds and returns the number of patients with temperatures
   * that are more than 0.5 degrees from the norm
   */
  public int getSickPatients() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the patients array and check if their temperature minus 98.6 is
     * greater than 0.5. If this is true, update the count. Return the number of
     * patients with temperatures that are more than 0.5 degrees from normal.
     * -----------------------------------------------------------------------------
     */
    
    

    return 0;
  }
  
}`}],validationFiles:[{path:`PhysicianTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Physician.java Test")
public class PhysicianTest {

  String messageGap = "\\n       ";
  Patient[] testPatients;
  Physician testPhysician;
   
  @BeforeEach
  public void setup() {
    testPatients = getRandomPatients();
    testPhysician = new Physician(testPatients);
  }
   
  @Test
  @Order(1)
  @DisplayName("getSickPatients() finds and returns the number of patients with temps more than 0.5 from the norm => ")
  public void testGetSickPatients() {
    String message = "The error message to display." + messageGap;
      
    int expected = getExpectedResult();
    int actual = testPhysician.getSickPatients();

    assertEquals(expected, actual, message);
  }

  private Patient[] getRandomPatients() {
    Patient[] temp = new Patient[10];

    for (int index = 0; index < temp.length; index++) {
      double randomTemp = (Math.random() * 3) + 98;
      temp[index] = new Patient("some patient", randomTemp);
    }

    return temp;
  }

  private int getExpectedResult() {
    int count = 0;

    for (int index = 0; index < testPatients.length; index++) {
      if (Math.abs(testPatients[index].getTemp() - 98.6) > 0.5) {
        count++;
      }
    }

    return count;
  }
  
}`}],dataFiles:[]},{name:`Practice: Using Math.abs() (c)`,lesson:`Lesson 3: The Math Class`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // The 1D array of home team scores
    int[] home = {75, 82, 68, 90, 71};

    // The 1D array of visiting team scores
    int[] visitor = {64, 79, 83, 70, 88};

    // Creates a BasketballScene object with the home and team scores
    BasketballScene scene = new BasketballScene(home, visitor);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawResults() method with the 1D array returned from calling
     * the getScoreDiff() method.
     * -----------------------------------------------------------------------------
     */




    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`BasketballScene.java`,text:`import org.code.theater.*;

/*
 * Creates a visualization to show the difference
 * between home and visiting team scores
 */
public class BasketballScene extends Scene {

  private int[] homeScores;        // The 1D array containing a home team's scores
  private int[] visitingScores;    // The 1D array containing a visiting team's scores

  /*
   * Initializes homeScores and visitingScores to the specified 1D arrays
   */
  public BasketballScene(int[] homeScores, int[] visitingScores) {
    this.homeScores = homeScores;
    this.visitingScores = visitingScores;
  }

  /*
   * Returns a 1D array containing the differences between the home team
   * scores and the visiting team scores
   */
  public int[] getScoreDiff() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a 1D array containing the difference between each home team
     * score and corresponding visiting team score.
     * -----------------------------------------------------------------------------
     */
    
    

    return null;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(int[] results) {
    for (int index = 0; index < results.length; index++) {
      int winner = getWinner(homeScores[index], visitingScores[index]);
      drawHomeText(homeScores[index]);
      
      drawVisitingText(visitingScores[index]);
      drawWinnerText(winner, results[index]);

      int size = 150;
      int xLocation = (getWidth() / 2) - (size / 2);
      drawImage("basketball.png", xLocation, 120, size);
      
      pause(1.5);
      clear("white");
    }
  }

  /*
   * Returns 0 if the home team won or 1 if the visiting team won
   */
  public int getWinner(int homeScore, int visitingScore) {
    if (homeScore > visitingScore) {
      return 0;
    }
    else {
      return 1;
    }
  }

  /*
   * Draws the home team's score
   */
  public void drawHomeText(int score) {
    drawText("Home", 75, 75);
    drawText(score + "", 75, 100);
  }

  /*
   * Draws the visiting team's score
   */
  public void drawVisitingText(int score) {
    drawText("Visitor", 250, 75);
    drawText(score + "", 250, 100);
  }

  /*
   * Draws text in the scene based on who won
   */
  public void drawWinnerText(int winner, int difference) {
    String winnerText = "";
    
    if (winner == 0) {
      winnerText = "Home";
    }
    else {
      winnerText = "Visiting";
    }

    drawText(winnerText + " team won by " + difference + " points", 50, 325);
  }
  
}`}],validationFiles:[{path:`BasketballSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BasketballScene.java Test")
public class BasketballSceneTest {

  String messageGap = "\\n       ";
  int[] testHome;
  int[] testVisitor;
  BasketballScene testScene;
   
  @BeforeEach
  public void setup() {
    testHome = getRandomScores();
    testVisitor = getRandomScores();
    testScene = new BasketballScene(testHome, testVisitor);
  }
   
  @Test
  @Order(1)
  @DisplayName("getScoreDiff() returns a 1D array containing the differences in scores => ")
  public void testGetScoreDiff() {
    String message = "Add the absolute value of the home score minus the visitor score to a new 1D array." + messageGap;
      
    int[] expected = getExpectedResults();
    int[] actual = testScene.getScoreDiff();

    assertArrayEquals(expected, actual, message);
  }

  private int[] getRandomScores() {
    int[] temp = new int[10];

    for (int index = 0; index < temp.length; index++) {
      int randomScore = (int)(Math.random() * 100) + 50;
      temp[index] = randomScore;
    }

    return temp;
  }

  private int[] getExpectedResults() {
    int[] differences = new int[testHome.length];

    for (int index = 0; index < differences.length; index++) {
      differences[index] = Math.abs(testHome[index] - testVisitor[index]);
    }

    return differences;
  }
  
}`}],dataFiles:[]},{name:`Practice: Using Math.abs() (d)`,lesson:`Lesson 3: The Math Class`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of SadBee objects
    SadBee[] bees = {new SadBee(20, 20), new SadBee(200, 20), new SadBee(200, 40), new SadBee(80, 100),
                     new SadBee(100, 45), new SadBee(200, 75), new SadBee(220, 150), new SadBee(60, 35)};

    // Creates a SadBeeScene object
    SadBeeScene scene = new SadBeeScene(bees);

    // Calls the drawResults() method
    scene.drawResults();

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`SadBee.java`,text:`import org.code.theater.*;

/*
 * Represents a sad bee avatar
 */
public class SadBee {

  private int xLocation;
  private int yLocation;
  public static final int SIZE = 150;
  public static final String SAD_BEE_IMAGE = "sadbee.png";

  /*
   * Sets the xLocation and yLocation to the specified values
   */
  public SadBee(int xLocation, int yLocation) {
    this.xLocation = xLocation;
    this.yLocation = yLocation;
  }

  /*
   * Returns the x location of the sad bee avatar
   */
  public int getXLocation() {
    return xLocation;
  }

  /*
   * Returns the y location of the sad bee avatar
   */
  public int getYLocation() {
    return yLocation;
  }

  /*
   * Sets the x location to the newXLocation
   */
  public void setXLocation(int newXLocation) {
    xLocation = newXLocation;
  }

  /*
   * Sets the y location to the newYLocation
   */
  public void setYLocation(int newYLocation) {
    yLocation = newYLocation;
  }
  
}`},{path:`SadBeeScene.java`,text:`import org.code.theater.*;

/*
 * Creates a scene with sad bee avatars and displays
 * their distance from each other as they move
 */
public class SadBeeScene extends Scene {

  private SadBee[] sadbees;     // The 1D array of SadBee objects

  /*
   * Initializes sadbees to the specified 1D array
   */
  public SadBeeScene(SadBee[] sadbees) {
    this.sadbees = sadbees;
  }

  /*
   * Returns the 1D array of sadbees
   */
  public SadBee[] getSadbees() {
    return sadbees;
  }

  /*
   * Returns the distance between the first SadBee and the second SadBee
   */
  public int getDistance(SadBee first, SadBee second) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return the absolute value of the second bee's x location minus the 
     * first bee's x location.
     * -----------------------------------------------------------------------------
     */
    
    return 0;
  }

  /*
   * Draws the sad bee avatars two at a time in the scene
   */
  public void drawResults() {
    for (int index = 0; index < sadbees.length - 1; index += 2) {
      drawImage(SadBee.SAD_BEE_IMAGE, sadbees[index].getXLocation(), sadbees[index].getYLocation(), SadBee.SIZE);
      drawImage(SadBee.SAD_BEE_IMAGE, sadbees[index + 1].getXLocation(), sadbees[index + 1].getYLocation(), SadBee.SIZE);
      drawText("Distance between the sad bees is " + getDistance(sadbees[index], sadbees[index + 1]), 40, 330);
      
      pause(1);
      clear("white");
    }
  }
  
}`}],validationFiles:[{path:`SadBeeSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SadBeeScene.java Test")
public class SadBeeSceneTest {

  String messageGap = "\\n       ";
  SadBee[] testBees;
  SadBeeScene testScene;
   
  @BeforeEach
  public void setup() {
    testBees = new SadBee[]{new SadBee(getRandomLocation(), getRandomLocation()), new SadBee(getRandomLocation(), getRandomLocation())};
    testScene = new SadBeeScene(testBees);
  }
   
  @Test
  @Order(1)
  @DisplayName("getDistance() returns the distance between the first sad bee and the second sad bee => ")
  public void testGetDistance() {
    String message = "Return the absolute value of the second sad bee's x location minus the first sad bee's x location." + messageGap;
      
    int expected = Math.abs(testBees[1].getXLocation() - testBees[0].getXLocation());
    int actual = testScene.getDistance(testBees[0], testBees[1]);

    assertEquals(expected, actual, message);
  }

  private int getRandomLocation() {
    return (int)(Math.random() * 400);
  }

}`}],dataFiles:[]},{name:`Practice: Using Math.sqrt() (a)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates an Investment object
    Investment savings = new Investment(35.89);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcReturn() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`Investment.java`,text:`/*
 * Represents an investment in an asset
 */
public class Investment {

  private double purchaseAmount;     // The amount paid for an asset

  /*
   * Sets purchaseAmount to the specified purchase amount
   */
  public Investment(double purchaseAmount) {
    this.purchaseAmount = purchaseAmount;
  }

  /*
   * Returns the amount paid for the asset
   */
  public double getPurchaseAmount() {
    return purchaseAmount;
  }

  /*
   * Calculates and returns the rate of return on an asset
   */
  public double calcReturn(double soldAmount) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate the square root of soldAmount minus purchaseAmount, then
     * subtract 1 from the result.
     * -----------------------------------------------------------------------------
     */
    
    return 0;
  }
  
}`}],validationFiles:[{path:`InvestmentTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Investment.java Test")
public class InvestmentTest {

  String messageGap = "\\n       ";
  Investment testInvestment;
   
  @BeforeEach
  public void setup() {
    double randomAmount = (Math.random() * 100) + 25;
    testInvestment = new Investment(randomAmount);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcReturn() returns the rate of return on an asset => ")
  public void testCalcReturn() {
    String message = "Get the square root of the soldAmount minus the purchaseAmount, then subtract 1 from the result." + messageGap;

    double soldAmount = testInvestment.getPurchaseAmount() + 50;
    double expected = Math.sqrt(soldAmount - testInvestment.getPurchaseAmount()) - 1;
    double actual = testInvestment.calcReturn(soldAmount);

    assertEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Using Math.sqrt() (b)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of city names
    String[] names = {"New York", "Tokyo", "Chicago", "Atlanta", "Philadelphia", "Boston",
                      "Los Angeles", "Dallas/Fort Worth", "Houston", "Detroit"};

    // Creates a 1D array of the land areas of each city
    double[] cityAreas = {8683, 6993, 5498, 5083, 4661, 4497, 4320, 3644, 3355, 3267};

    // Creates a Research object
    Research cities = new Research(names, cityAreas);

    /* ------------------------------------------- TO DO -------------------------------------------
     * ✅ Call the calculateSizes() method and print the results using the sizesToString() method.
     * ---------------------------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Research.java`,text:`/*
 * Analyzes city data
 */
public class Research {

  private String[] names;    // The 1D array of the names of each city
  private double[] cities;   // The 1D array of the land area of each city

  /*
   * Initializes names and cities to the specified 1D arrays
   */
  public Research(String[] names, double[] cities) {
    this.names = names;
    this.cities = cities;
  }

  /*
   * Returns the 1D array of names
   */
  public String[] getNames() {
    return names;
  }

  /*
   * Returns the 1D array of cities
   */
  public double[] getCities() {
    return cities;
  }

  /*
   * Returns a 1D array containing the approximate
   * width of each city in cities
   */
  public double[] calculateSizes() {
    /* ------------------------------------------- TO DO -------------------------------------------
     * ✅ Get the square root of each city value divided by Math.PI, then multiply the result
     * by 2. Add the result to a new 1D array and return the array.
     * ---------------------------------------------------------------------------------------------
     */
    
    
    
    return null;
  }

  /*
   * Returns a String containing the values in the 1D array
   */
  public String sizesToString(double[] values) {
    String result = "";

    for (int index = 0; index < values.length; index++) {
      result += names[index] + ": " + values[index] + "\\n";
    }

    return result;
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
  double[] testAreas;
  Research testResearch;
   
  @BeforeEach
  public void setup() {
    testAreas = getRandomAreas();
    String[] testNames = new String[10];
    testResearch = new Research(testNames, testAreas);
  }
   
  @Test
  @Order(1)
  @DisplayName("calculateSizes() returns a 1D array containing the approximate width of each city => ")
  public void testCalculateSizes() {
    String message = "Calculate the square root of each land area times Math.PI, then multiply the result by 2." + messageGap;

    double[] expected = getExpectedResults();
    double[] actual = testResearch.calculateSizes();

    assertArrayEquals(expected, actual, message);
  }

  private double[] getRandomAreas() {
    double[] temp = new double[10];

    for (int index = 0; index < temp.length; index++) {
      temp[index] = Math.random() * 1000 + 50;
    }

    return temp;
  }

  private double[] getExpectedResults() {
    double[] results = new double[testAreas.length];

    for (int index = 0; index < results.length; index++) {
      results[index] = Math.sqrt(testAreas[index] * Math.PI) * 2;
    }
    
    return results;
  }
   
}`}],dataFiles:[]},{name:`Practice: Using Math.sqrt() (c)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcTime() method and print the result.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Drone.java`,text:`/*
 * Represents a drone
 */
public class Drone {

  public static double calcTime(int height) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Get the square root of height and divide the result by 4;
     * -----------------------------------------------------------------------------
     */
    
    return 0;
  }
  
}`}],validationFiles:[{path:`DroneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

@DisplayName("Drone.java Test")
public class DroneTest {

  String messageGap = "\\n       ";
   
  @Test
  @DisplayName("calcTime() returns the number of seconds it will take a package to reach the ground => ")
  public void testCalcTime() {
    String message = "Calculate the square root of height, then divide the result by 4." + messageGap;
      
    int randomHeight = (int)(Math.random() * 500) + 100;
    double expected = Math.sqrt(randomHeight) / 4;
    double actual = Drone.calcTime(randomHeight);

    assertEquals(expected, actual, message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Using Math.sqrt() (d)`,lesson:`Lesson 3: The Math Class`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of city names
    String[] names = FileReader.toStringArray("cities.txt");

    // Creates a 1D array of salaries
    int[] salaries = FileReader.toIntArray("salaries.txt");

    // Creates a Salary object
    Salary data = new Salary(names, salaries);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcDeviation() method and print the result.
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
  
}`},{path:`Salary.java`,text:`/*
 * Analyzes data about software engineering salaries
 */
public class Salary {

  private String[] cities;      // The 1D array of city names
  private int[] salaries;       // The 1D array of salaries for each city

  /*
   * Initializes cities and salaries to the specified 1D arrays
   */
  public Salary(String[] cities, int[] salaries) {
    this.cities = cities;
    this.salaries = salaries;
  }

  /*
   * Returns the 1D array of city names
   */
  public String[] getCities() {
    return cities;
  }

  /*
   * Returns the 1D array of salaries for each city
   */
  public int[] getSalaries() {
    return salaries;
  }

  /*
   * Returns the amount each salary is from the average salary
   */
  public double calcDeviation() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the amount each salary is from the average salary.
     * -----------------------------------------------------------------------------
     */
    
    
    return 0;
  }

  /*
   * Calculates and returns the average salary
   */
  public double calcAverage() {
    double sum = 0;

    for (int amount : salaries) {
      sum += amount;
    }

    return sum / salaries.length;
  }
}`}],validationFiles:[{path:`SalaryTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Salary.java Test")
public class SalaryTest {

  String messageGap = "\\n       ";
  int[] testSalaries;
  Salary testSalary;
   
  @BeforeEach
  public void setup() {
    testSalaries = getRandomSalaries();
    String[] testNames = new String[10];
    testSalary = new Salary(testNames, testSalaries);
  }
   
  @Test
  @Order(1)
  @DisplayName("calcDeviation() returns the amount each salary is from the average salary => ")
  public void testCalcDeviation() {
    String message = "Traverse the salaries array and add the result of each salary minus the average salary raised to the power of 2.\\n";
    message += "        Add the result to a variance variable. After the loop, divide the variance by the number of salaries,\\n";
    message += "        then return the square root of the variance." + messageGap;
      
    double expected = getExpectedResult();
    double actual = testSalary.calcDeviation();

    assertEquals(expected, actual, message);
  }

  private int[] getRandomSalaries() {
    int[] temp = new int[10];

    for (int index = 0; index < temp.length; index++) {
      temp[index] = (int)(Math.random() * 100000) + 50000;
    }

    return temp;
  }

  private double getExpectedResult() {
    double averageSalary = testSalary.calcAverage();
    double variance = 0;

    for (int index = 0; index < testSalaries.length; index++) {
      variance += Math.pow(testSalaries[index] - averageSalary, 2);
    }

    variance /= testSalaries.length;
    return Math.sqrt(variance);
  }
   
}`}],dataFiles:[{path:`cities.txt`,text:`Columbus, OH
Seattle-Tacoma-Bellevue, WA
Charlotte-Concord-Gastonia, NC-SC
Colorado Springs, CO
Dayton, OH
Greensboro-High Point, NC
San Francisco-Oakland-Hayward, CA
San Francisco-Oakland-Hayward, CA
San Antonio-New Braunfels, TX
Dallas-Fort Worth-Arlington, TX
Dallas-Fort Worth-Arlington, TX
Durham-Chapel Hill, NC
Atlanta-Sandy Springs-Roswell, GA
Cincinnati, OH-KY-IN
Austin-Round Rock, TX
Raleigh, NC
Houston-The Woodlands-Sugar Land, TX
Baltimore-Columbia-Towson, MD
Las Vegas-Henderson-Paradise, NV
Tucson, AZ
Salt Lake City, UT
Richmond, VA
Lexington-Fayette, KY
Denver-Aurora-Lakewood, CO
San Jose-Sunnyvale-Santa Clara, CA
San Jose-Sunnyvale-Santa Clara, CA
Portland-Vancouver-Hillsboro, OR-WA
Portland-Vancouver-Hillsboro, OR-WA
Wichita, KS
Phoenix-Mesa-Scottsdale, AZ
Phoenix-Mesa-Scottsdale, AZ
Santa Maria-Santa Barbara, CA
Los Angeles-Long Beach-Anaheim, CA
Los Angeles-Long Beach-Anaheim, CA
Boston-Cambridge, MA
Orlando-Kissimmee-Sanford, FL
Tampa-St. Petersburg-Clearwater, FL
Birmingham-Hoover, AL
Syracuse, NY
Washington-Arlington-Alexandria, DC-VA-MD-WV
Sacramento--Roseville--Arden-Arcade, CA
Philadelphia-Camden-Wilmington, PA-NJ-DE-MD
Des Moines-West Des Moines, IA
Omaha-Council Bluffs, NE-IA
Minneapolis-St. Paul-Bloomington, MN-WI
Chicago-Naperville-Elgin, IL-IN-WI
Nashville-Davidson--Murfreesboro--Franklin, TN
Oklahoma City, OK
Detroit-Warren-Dearborn, MI
San Diego-Carlsbad, CA
Pittsburgh, PA
New York-Newark-Jersey City, NY-NJ-PA
New York-Newark-Jersey City, NY-NJ-PA
Milwaukee-Waukesha-West Allis, WI
Indianapolis-Carmel-Anderson, IN
Fort Collins, CO
Charleston-North Charleston, SC
Kansas City, MO-KS
Tulsa, OK
Rochester, NY
Louisville/Jefferson County, KY-IN
Albany-Schenectady-Troy, NY
Buffalo-Cheektowaga-Niagara Falls, NY
Little Rock-North Little Rock-Conway, AR
Boise City, ID
Ann Arbor, MI
Akron, OH
Jacksonville, FL
Albuquerque, NM
Spokane-Spokane Valley, WA
Madison, WI
Memphis, TN-MS-AR
Cleveland-Elyria, OH
Miami-Fort Lauderdale-West Palm Beach, FL
Miami-Fort Lauderdale-West Palm Beach, FL
Eugene, OR
Urban Honolulu, HI`},{path:`salaries.txt`,text:`117552
117323
114122
112118
111616
111050
111017
111017
110898
110891
110891
110313
109849
109013
108990
107676
107672
107282
107175
107170
106291
105258
105248
105197
104964
104964
104335
104335
103690
103177
103177
102781
102188
102188
102171
102093
101733
101598
101256
101188
100785
99955
99878
99826
99228
98726
98623
98617
98156
97974
97043
96686
96686
96580
96448
95969
95609
95308
95187
94819
94320
94051
93735
93194
93112
92683
92207
91568
91010
90269
90238
89921
89040
88249
88249
85912
72811`}]},{name:`Investigate and Modify: Casting`,lesson:`Lesson 4: Casting and Rounding`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {



    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Division`,lesson:`Lesson 4: Casting and Rounding`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {



    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Rounding`,lesson:`Lesson 4: Casting and Rounding`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {



    
    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Random Numbers`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    System.out.println("First random number: " + Math.random());
    System.out.println("Second random number: " + Math.random());
    System.out.println("Third random number: " + Math.random());

  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Random Numbers #1`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Random Numbers #2`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Generating Random Numbers (a)`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Player object
    Player player = new Player("Amelia");
    
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getRandomHealth() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`Player.java`,text:`/*
 * Represents a player in the game
 */
public class Player {

  private String name;       // The name of a player
  private int health;        // The health of a player
  private int level;         // The level of a player

  /*
   * Sets name to the specified name, health to 0, and level to 0
   */
  public Player(String name) {
    this.name = name;
    health = 0;
    level = 0;
  }

  /*
   * Assigns a random value to health
   */
  public int getRandomHealth() {
    /* ------------------------------ TO DO ------------------------------
     * ✅ Generate a random amount of health for the player to start with.
     * -------------------------------------------------------------------
     */
    
    
    return -1;
  }

  /*
   * Returns the current value assigned to level
   */
  public int getLevel() {
    return level;
  }
  
}`}],validationFiles:[{path:`PlayerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Player.java Test")
public class PlayerTest {
    
  String messageGap = "\\n       ";
  Player testPlayer;
  
  @BeforeEach
  public void setup() {
    testPlayer = new Player("some player");
  }
  
  @Test
  @Order(1)
  @DisplayName("getRandomHealth() generates a random number between 100 and 200 => ")
  public void testGetRandomHealth() {
    String randomMessage = "health should be a random number between 100 and 200." + messageGap;
    
    int randomHealth = testPlayer.getRandomHealth();
    assertTrue(randomHealth >= 100 && randomHealth <= 200, randomMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("getRandomHealth() sets the level based on the health => ")
  public void testGetRandomHealthLevel() {
    String levelMessage = "If the health is less than 150, set level to 1. Else, set level to 2." + messageGap;
    
    int randomHealth = testPlayer.getRandomHealth();
    int level = testPlayer.getLevel();

    if (randomHealth < 150) {
      assertEquals(level, 1, levelMessage);
    }
    else if (randomHealth >= 150) {
      assertEquals(level, 2, levelMessage);
    }
  }
}`}],dataFiles:[]},{name:`Practice: Generating Random Numbers (b)`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Declare and initialize a 1D int array that stores the array returned from
     * calling the generateNumCars() method.
     * -----------------------------------------------------------------------------
     */
    


    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the calcAverage() method with the 1D array and print the result.
     * -----------------------------------------------------------------------------
     */

    

  }
}`},{path:`Simulation.java`,text:`/*
 * Represents a simulation of a new bridge
 */
public class Simulation {

  /*
   * Returns a 1D array containing the number of cars that could
   * cross the bridge each day of a month
   */
  public static int[] generateNumCars() {
    /* ------------------------------ TO DO ------------------------------
     * ✅ Return a 1D array containing the number of cars that could cross
     *  the bridge each day of a month.
     * -------------------------------------------------------------------
     */

    return null;
  }

  /*
   * Returns the average of the values in the 1D array
   */
  public static int calcAverage(int[] cars) {
    int sum = 0;

    for (int value : cars) {
      sum += value;
    }

    return sum / cars.length;
  }
  
}`}],validationFiles:[{path:`SimulationTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Simluation.java Test")
public class SimulationTest {
    
  String messageGap = "\\n       ";
  
  @Test
  @Order(1)
  @DisplayName("generateNumCars() returns a 1D array of length 30 => ")
  public void testGenerateNumCars() {
    String arrayLengthMessage = "The 1D array should be of length 30." + messageGap;
    
    int[] testArr = Simulation.generateNumCars();
    assertEquals(testArr.length, 30, arrayLengthMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("generateNumCars() returns a 1D array containing values between 200 and 500 => ")
  public void testGenerateNumCarsValues() {
    String randomMessage = "Each number should be a random number between 200 and 500." + messageGap;
    
    int[] testArr = Simulation.generateNumCars();

    for (int val : testArr) {
      assertTrue(val >= 200 && val <= 500, randomMessage);
    }
  }
}`}],dataFiles:[]},{name:`Practice: Generating Random Numbers (c)`,lesson:`Lesson 5: Random`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a WeatherScene object
    WeatherScene scene = new WeatherScene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getRandomTemp() method and pass the value returned to a call
     * to the drawResult() method.
     * -----------------------------------------------------------------------------
     */


    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`WeatherScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene to simulate the weather
 */
public class WeatherScene extends Scene {

  /*
   * Returns a random temperature
   */
  public int getRandomTemp() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Generate and return a random value between 20 and 80.
     * -----------------------------------------------------------------------------
     */
    
    return -1;
  }

  /*
   * Draws the snowflake if the temperature is less than 40, and draws
   * the sun if the temperature is greater than or equal to 40
   */
  public void drawResult(int temp) {
    if (temp < 40) {
      drawImage("snowflake.png", 50, 50, 300);
    }
    else {
      drawImage("sun.png", 50, 50, 300);
    }

    drawText(temp + " degrees", 250, 350);
  }
  
}`}],validationFiles:[{path:`WeatherSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("WeatherScene.java Test")
public class WeatherSceneTest {

  String messageGap = "\\n       ";
  WeatherScene testScene;
   
  @BeforeEach
  public void setup() {
    testScene = new WeatherScene();
  }
   
  @Test
  @Order(1)
  @DisplayName("getRandomTemp() returns a random value between 20 and 80 => ")
  public void testGetRandomTemp() {
    String message = "Multiply Math.random() by the range (max - min) then add the minimum value." + messageGap;
      
    int actual = testScene.getRandomTemp();

    assertTrue(actual >= 20 && actual <= 80, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Generating Random Numbers (d)`,lesson:`Lesson 5: Random`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a DonutScene object
    DonutScene donutScene = new DonutScene();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getRandomDonuts() method and pass the value returned to a call
     * to the getDiscount() method. Call the drawResults() method with the values
     * returned from the getRandomDonuts() and getDiscount() methods.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(donutScene);

  }
}`},{path:`DonutScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that visualizes donuts
 */
public class DonutScene extends Scene {
  
  /*
   * Returns a random number of donuts
   */
  public int getRandomDonuts() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a random number between 1 and 24.
     * -----------------------------------------------------------------------------
     */

    return -1;
  }

  /*
   * Returns a String containing the discount based on the number of donuts
   */
  public String getDiscount(int numDonuts) {
    if (numDonuts > 12) {
      return "You're eligible for a 20% discount if you order " + numDonuts + " donuts!";
    }
    else {
      return "Sorry, no discount is available for " + numDonuts + " donuts!";
    }
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(int numDonuts, String message) {
    while (numDonuts > 0) {
      int randomX = (int)(Math.random() * getWidth());
      int randomY = (int)(Math.random() * getHeight());
      drawImage("donut.png", randomX, randomY, 50);
      numDonuts--;
      pause(0.1);
    }

    pause(0.5);
    setTextHeight(18);
    drawText(message, 10, 50);
  }
  
}`}],validationFiles:[{path:`DonutTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Donut.java Test")
public class DonutTest {
    
  String messageGap = "\\n       ";
  DonutScene testDonutScene;
  
  @BeforeEach
  public void setup() {
    testDonutScene = new DonutScene();
  }
  
  @Test
  @Order(1)
  @DisplayName("getRandomDonuts() returns a random number between 1 and 24 => ")
  public void testGetRandomDonuts() {
    String donutMessage = "The number of donuts should be a random number between 1 and 24." + messageGap;
  
    int actual = testDonutScene.getRandomDonuts();

    assertTrue(actual >= 1 && actual <= 24, donutMessage);
  }

}`}],dataFiles:[]},{name:`Practice: Accessing Random Elements from a 1D Array (a)`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Planet objects
    Planet[] worlds = { new Planet(), new Planet("HAT-P-26b", "Neptunian"), new Planet("TRAPPIST-1 e", "Terrestrial") };

    // Creates a Galaxy object with the 1D array
    Galaxy theGalaxy = new Galaxy(worlds);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getRandomPlanet() method and print the result
     * -----------------------------------------------------------------------------
     */



    

  }
}`},{path:`Galaxy.java`,text:`/*
 * Represents the galaxy
 */
public class Galaxy {

  private Planet[] worlds;    // The 1D array of Planet objects

  /*
   * Initializes worlds to the 1D array of Planet objects
   */
  public Galaxy(Planet[] worlds) {
    this.worlds = worlds;
  }

  /*
   * Returns a random Planet
   */
  public Planet getRandomPlanet() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a random planet from the 1D array of Planet objects
     * -----------------------------------------------------------------------------
     */

    return null;
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
}`}],validationFiles:[{path:`GalaxyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Galaxy.java Test")
public class GalaxyTest {
    
  String messageGap = "\\n       ";
  Galaxy testGalaxy;
  Planet[] worlds = { new Planet(), new Planet("HAT-P-26b", "Neptunian"), new Planet("TRAPPIST-1 e", "Terrestrial") };
  
  @BeforeEach
  public void setup() {
    testGalaxy = new Galaxy(worlds);
  }
  
  @Test
  @Order(1)
  @DisplayName("Return a random planet => ")
  public void testGetRandomHealth() {
    String randomMessage = "Return a random planet from a 1D array of Planet objects worlds" + messageGap;
    
    Planet randomPlanet = testGalaxy.getRandomPlanet();
    assertTrue(Arrays.asList(worlds).contains(randomPlanet), randomMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Accessing Random Elements from a 1D Array (b)`,lesson:`Lesson 5: Random`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of letters
    String[] wordToEncrypt = {"s", "e", "c", "r", "e", "t"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the encryptWord() method on wordToEncrypt and print the result.
     * -----------------------------------------------------------------------------
     */


    

  }
}`},{path:`Encryption.java`,text:`/*
 * Represents a encryption algorithm
 */
public class Encryption {

  private static String[] availableLetters = {"i", "j", "k", "m", "n", "x", "y", "z"};

  /*
   * Encrypts the word
   */  
  public static String encryptWord(String[] letters) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the letters array and replace each letter with a randomly chosen
     * letter from the availableLetters array. Use the lettersToWord() method to
     * return the encrypted word as a String.
     * -----------------------------------------------------------------------------
     */

    
    
  }

  /*
   * Converts an array of Strings to a String
   */  
  public static String lettersToWord(String[] letters) {
    String word = "";

    for (String s : letters) {
      word += s;
    }

    return word;
  }

  /*
   * Returns the available letters
   */
  public static String[] getAvailableLetters() {
    return availableLetters;
  }
  
}`}],validationFiles:[{path:`EncryptionTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import java.util.List;
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
  
  @BeforeEach
  public void setup() {
  }
  
  @Test
  @Order(1)
  @DisplayName("Encrypt a word => ")
  public void testEncrypt() {
    String randomMessage = "Each letter in the encrypted word should be a randomly chosen letter from the availableLetters array." + messageGap;

    String[] wordToEncrypt = {"r", "a", "n", "d", "o", "m"};
    String encrypted = Encryption.encryptWord(wordToEncrypt);

    assertEquals(encrypted.length(), wordToEncrypt.length, randomMessage);

    String[] encryptedArr = encrypted.split("");
    List<String> availableLetters = Arrays.asList(Encryption.getAvailableLetters());
    for (String s : encryptedArr) {
      assertTrue(availableLetters.contains(s), randomMessage);
    }
  }
}`}],dataFiles:[]},{name:`Practice: Accessing Random Elements from a 1D Array (c)`,lesson:`Lesson 5: Random`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of colors
    String[] colors = {"red", "blue", "orange", "green", "purple"};

    // Creates a ShapeScene object
    ShapeScene scene = new ShapeScene(colors);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the chooseRandomColor() method and pass the value returned to the
     * drawScene() method
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`ShapeScene.java`,text:`import org.code.theater.*;

/*
 * Represents a shape scene
 */
public class ShapeScene extends Scene {
  
  private String[] colors;     // The 1D array of colors to use for the scene

  /*
   * Initializes colors to the specified 1D array of colors
   */
  public ShapeScene(String[] colors) {
    this.colors = colors;
  }
  
  /*
   * Returns a random color from the 1D array of colors
   */
  public String chooseRandomColor() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Choose and return a random color from the 1D array of colors.
     * -----------------------------------------------------------------------------
     */

    return "";
  }
  
  /*
   * Returns the 1D array of colors
   */
  public String[] getColors() {
    return colors;
  }

  /*
   * Draws shapes in the scene using the selected color
   */
  public void drawScene(String color, int numShapes) {
    setFillColor(color);
    
    for(int i = 0; i < numShapes; i++) {
      int randomNum = (int)(Math.random() * 2);
      
      int x = (int)(Math.random() * this.getWidth());
      int y = (int)(Math.random() * this.getHeight());
      int width = 50;
      int height = 50;
      
      if (randomNum == 0) {
        this.drawEllipse(x, y, width, height);
      } else {
        this.drawRectangle(x, y, width, height);
      }
    }
  }
  
}`}],validationFiles:[{path:`ShapeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ShapeScene.java Test")
public class ShapeTest {

  String[] colors = {"red", "blue", "orange", "green", "purple"};
  ShapeScene testShapeScene;
  String messageGap = "\\n       ";
  
  @BeforeEach
  public void setup() {
    testShapeScene = new ShapeScene(colors);
  }
  
  @Test
  @Order(1)
  @DisplayName("chooseRandomColor() returns a random color => ")
  public void testChooseRandomColor() {
    String randomMessage = "The color should be a randomly chosen color from the colors array." + messageGap;

    String testColor = testShapeScene.chooseRandomColor();
    assertTrue(Arrays.asList(testShapeScene.getColors()).contains(testColor), randomMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Accessing Random Elements from a 1D Array (d)`,lesson:`Lesson 5: Random`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of image files
    String[] images = {"fashion.jpg", "nature.jpg", "games.jpg", "astronaut.jpg", "guitar.jpg"};

    // Creates an ImageScene object
    ImageScene scene = new ImageScene(images);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the chooseRandomImage() method and pass the image returned to a
     * call to the drawScene() method.
     * -----------------------------------------------------------------------------
     */   
    
    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene to display images
 */
public class ImageScene extends Scene {

  private String[] images;     // The 1D array of images

  /*
   * Initializes images to the specified 1D array of images
   */
  public ImageScene(String[] images) {
    this.images = images;
  }

  /*
   * Returns the 1D array of images
   */
  public String[] getImages() {
    return images;
  }

  /*
   * Returns a random image from the 1D array of images
   */
  public String chooseRandomImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a random image from the images array.
     * -----------------------------------------------------------------------------
     */

    return "";
  }

  /*
   * Draws the image in the scene
   */
  public void drawScene(String image) {
    drawImage(image, 0, 0, getWidth());
  }
  
}
`}],validationFiles:[{path:`ImageSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ImageScene.java Test")
public class ImageSceneTest {

  String[] images = {"fashion.jpg", "nature.jpg", "games.jpg", "astronaut.jpg", "guitar.jpg"};
  ImageScene testScene;
  String messageGap = "\\n       ";
  
  @BeforeEach
  public void setup() {
    testScene = new ImageScene(images);
  }
  
  @Test
  @Order(1)
  @DisplayName("chooseRandomImage() returns a random image => ")
  public void testChooseRandomImage() {
    String randomMessage = "The image should be a randomly chosen image from the images array." + messageGap;

    String testImage = testScene.chooseRandomImage();
    assertTrue(Arrays.asList(testScene.getImages()).contains(testImage), randomMessage);
  }
}`}],dataFiles:[]},{name:`Predict and Run: Comparing Objects`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Event.java`,text:`/*
 * Represents an event
 */
public class Event {

  private String eventName;    // The name of an event
  private int capacity;        // The number of people attending an event

  /*
   * Sets eventName and capacity to the specified values
   */
  public Event(String eventName, int capacity) {
    this.eventName = eventName;
    this.capacity = capacity;
  }

  /*
   * Returns the name of the event
   */
  public String getEventName() {
    return eventName;
  }

  /*
   * Returns the capacity of the event
   */
  public int getCapacity() {
    return capacity;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Comparing Objects #1`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Event.java`,text:`/*
 * Represents an event
 */
public class Event {

  private String eventName;    // The name of an event
  private int capacity;        // The number of people attending an event

  /*
   * Sets eventName and capacity to the specified values
   */
  public Event(String eventName, int capacity) {
    this.eventName = eventName;
    this.capacity = capacity;
  }

  /*
   * Returns the name of the event
   */
  public String getEventName() {
    return eventName;
  }

  /*
   * Returns the capacity of the event
   */
  public int getCapacity() {
    return capacity;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Comparing Objects #2`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Event.java`,text:`/*
 * Represents an event
 */
public class Event {

  private String eventName;    // The name of an event
  private int capacity;        // The number of people attending an event

  /*
   * Sets eventName and capacity to the specified values
   */
  public Event(String eventName, int capacity) {
    this.eventName = eventName;
    this.capacity = capacity;
  }

  /*
   * Returns the name of the event
   */
  public String getEventName() {
    return eventName;
  }

  /*
   * Returns the capacity of the event
   */
  public int getCapacity() {
    return capacity;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Comparing Objects #3`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Event.java`,text:`/*
 * Represents an event
 */
public class Event {

  private String eventName;    // The name of an event
  private int capacity;        // The number of people attending an event

  /*
   * Sets eventName and capacity to the specified values
   */
  public Event(String eventName, int capacity) {
    this.eventName = eventName;
    this.capacity = capacity;
  }

  /*
   * Returns the name of the event
   */
  public String getEventName() {
    return eventName;
  }

  /*
   * Returns the capacity of the event
   */
  public int getCapacity() {
    return capacity;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Comparing Objects (a)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Dog objects
    Dog[] dogs = {new Dog("Max", 3), new Dog("Bella", 2), new Dog("Duke", 5), new Dog("Luna", 4), new Dog("Rocky", 1)};

    // Creates a PetStore object
    PetStore pets = new PetStore(dogs);

    // Creates two Dog objects to find
    Dog firstDog = new Dog("Charlie", 4);
    Dog secondDog = new Dog("Daisy", 10);

    // Calls the findMatchingDog() method and prints the result
    System.out.println("Match for Charlie: " + pets.findMatchingDog(firstDog) + "\\n");
    System.out.println("Match for Daisy: " + pets.findMatchingDog(secondDog));
    
  }
}`},{path:`Dog.java`,text:`/*
 * Represents a dog
 */
public class Dog {

  private String name;   // The name of a dog
  private int age;       // The age of a dog

  /*
   * Sets name to the specified name and age to the specified age
   */
  public Dog(String name, int age) {
    this.name = name;
    this.age = age;
  }

  /*
   * Returns the name of the dog
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the age of the dog
   */
  public int getAge() {
    return age;
  }

  /*
   * Returns true if the ages are the same, otherwise returns false
   */
  public boolean equals(Object other) {
    if (other == this) {
      return true;
    }

    if (!(other instanceof Dog)) {
      return false;
    }

    Dog otherDog = (Dog) other;
    return this.age == otherDog.age;
  }

  /*
   * Returns a String containing the dog's name and age
   */
  public String toString() {
    return name + ": " + age + " years old";
  }
  
}`},{path:`PetStore.java`,text:`/*
 * Represents a pet store
 */
public class PetStore {

  private Dog[] dogs;     // The 1D array of Dog objects

  /*
   * Initializes dogs to the specified 1D array of Dog objects
   */
  public PetStore(Dog[] dogs) {
    this.dogs = dogs;
  }

  /*
   * Returns a String containing information about the Dog
   * object that has the same age as the parameter dogToFind
   */
  public String findMatchingDog(Dog dogToFind) {
    String result = "";

    for (Dog pet : dogs) {
      if (pet == dogToFind) {
        result += "Matching dog found!\\n" + pet;
      }
    }

    if (result.equals("")) {
      result += "No matching dog found.";
    }

    return result;
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

  String messageGap = "\\n       ";
  Dog[] testDogs;
  PetStore testPetStore;
   
  @BeforeEach
  public void setup() {
    testDogs = new Dog[]{new Dog("Max", 3), new Dog("Bella", 2), new Dog("Duke", 5), new Dog("Luna", 4), new Dog("Rocky", 1)};
    testPetStore = new PetStore(testDogs);
  }
   
  @Test
  @Order(1)
  @DisplayName("findMatchingDog() returns a String containing information about the Dog object that has the same age as the parameter dogToFind => ")
  public void testFindMatchingDogFindsMatch() {
    String message = "Check the condition used in the findMatchingDog() method. Is it comparing aliases or instance variables?" + messageGap;
      
    int randomAge = (int)(Math.random() * 5) + 1;
    Dog testDogToFind = new Dog("some dog", randomAge);
    String actual = testPetStore.findMatchingDog(testDogToFind);

    assertTrue(actual.contains("Matching dog found!"), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("findMatchingDog() returns \\"No matching dog found.\\" if no Dog object has the same age as the parameter dogToFind => ")
  public void testFindMatchingDogDoesNotFindMatch() {
    String message = "Check the condition used in the findMatchingDog() method. Is it comparing aliases or instance variables?" + messageGap;
      
    int randomAge = (int)(Math.random() * 10) + 5;
    Dog testDogToFind = new Dog("some dog", randomAge);
    String actual = testPetStore.findMatchingDog(testDogToFind);

    assertEquals("No matching dog found.", actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Comparing Objects (b)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Video objects
    Video[] videos = {new Video("The Lion King", 1994), new Video("Beauty and the Beast", 1991), new Video("Aladdin", 1992),
                      new Video("Frozen", 2013), new Video("Moana", 2016)};

    // Creates a Catalog object
    Catalog videoCatalog = new Catalog(videos);

    // Creates two Video objects to find
    Video frozen = new Video("Frozen", 2013);
    Video toyStory = new Video("Toy Story", 1995);

    // Calls the findMatchingVideo() method and prints the result
    System.out.println(videoCatalog.findMatchingVideo(frozen) + "\\n");
    System.out.println(videoCatalog.findMatchingVideo(toyStory));
    
  }
}`},{path:`Catalog.java`,text:`/*
 * Represents a catalog of videos
 */
public class Catalog {

  private Video[] videos;      // The 1D array of Video objects

  /*
   * Initializes videos to the specified 1D array of Video objects
   */
  public Catalog(Video[] videos) {
    this.videos = videos;
  }

  /*
   * Returns a String containing information about the Video
   * object that has the same title as the parameter videoToFind
   */
  public String findMatchingVideo(Video videoToFind) {
    String result = "";

    for (Video item : videos) {
      if (item == videoToFind) {
        result += "Matching video found!\\n" + item;
      }
    }

    if (result.equals("")) {
      result += "No matching video found.";
    }

    return result;
  }
  
}`},{path:`Video.java`,text:`/*
 * Represents a video
 */
public class Video {

  private String title;     // The title of a video
  private int year;         // The year the video was released

  /*
   * Sets title to the specified title and year to the specified year
   */
  public Video(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Returns the title of the video
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the year of the video
   */
  public int getYear() {
    return year;
  }

  /*
   * Returns true if the title of other is the same as this Video
   */
  public boolean equals(Object other) {
    if (other == this) {
      return true;
    }

    if (!(other instanceof Video)) {
      return false;
    }

    Video otherVideo = (Video) other;
    return this.title.equals(otherVideo.title);
  }

  /*
   * Returns a String containing the video's title
   */
  public String toString() {
    return title + "(" + year + ")";
  }
  
}`}],validationFiles:[{path:`CatalogTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Catalog.java Test")
public class CatalogTest {

  String messageGap = "\\n       ";
  Video[] testVideos;
  Catalog testCatalog;
   
  @BeforeEach
  public void setup() {
    testVideos = new Video[]{new Video("The Lion King", 1994), new Video("Beauty and the Beast", 1991), new Video("Aladdin", 1992),
                             new Video("Frozen", 2013), new Video("Moana", 2016)};
    testCatalog = new Catalog(testVideos);
  }
   
  @Test
  @Order(1)
  @DisplayName("findMatchingVideo() returns a String containing information about the Video object that has the same title as the parameter videoToFind => ")
  public void testFindMatchingVideoFindsMatch() {
    String message = "Check the condition used in the findMatchingVideo() method. Is it comparing aliases or instance variables?" + messageGap;
      
    Video randomVideo = testVideos[(int)(Math.random() * testVideos.length)];
    Video testVideoToFind = new Video(randomVideo.getTitle(), randomVideo.getYear());
    String actual = testCatalog.findMatchingVideo(testVideoToFind);

    assertTrue(actual.contains("Matching video found!"), message);
  }
   
  @Test
  @Order(2)
  @DisplayName("findMatchingVideo() returns \\"No matching video found.\\" if no Video object has the same title as the parameter videoToFind => ")
  public void testFindMatchingVideoDoesNotFindMatch() {
    String message = "Check the condition used in the findMatchingVideo() method. Is it comparing aliases or instance variables?" + messageGap;
      
    Video testVideoToFind = new Video("The Little Mermaid", 1989);
    String actual = testCatalog.findMatchingVideo(testVideoToFind);

    assertEquals("No matching video found.", actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Comparing Objects (c)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Song objects
    Song[] songs = {new Song("Eye of the Tiger"), new Song("I Will Survive"), new Song("Don't Stop Believin'"), new Song("Stronger"), new Song("Hero")};

    // Creates a Playlist object
    Playlist inspire = new Playlist(songs);

    // Creates first test array of Song objects and Playlist
    Song[] firstTestSongs = {new Song("Eye of the Tiger"), new Song("I Will Survive"), new Song("Don't Stop Believin'"), new Song("Stronger"), new Song("Hero")};
    Playlist first = new Playlist(firstTestSongs);

    // Creates second test array of Song objects and Playlist
    Song[] secondTestSongs = {new Song("Beautiful"), new Song("I Will Survive"), new Song("Hall of Fame"), new Song("Stronger"), new Song("We Are the Champions")};
    Playlist second = new Playlist(secondTestSongs);

    // Calls the findMatchingPlaylist() method and prints the result
    System.out.println(inspire.findMatchingPlaylist(first) + "\\n");
    System.out.println(inspire.findMatchingPlaylist(second));
    
  }
}`},{path:`Playlist.java`,text:`/*
 * Represents a playlist
 */
public class Playlist {

  private Song[] songs;    // The 1D array of Song objects

  /*
   * Initializes songs to the specified 1D array of Song objects
   */
  public Playlist(Song[] songs) {
    this.songs = songs;
  }

  /*
   * Returns the 1D array of Song objects
   */
  public Song[] getSongs() {
    return songs;
  }

  /*
   * Returns a String containing the text "All of the songs are the same!"
   * if all Song objects in the playlist have the same title
   */
  public String findMatchingPlaylist(Playlist other) {
    boolean allSame = true;
    
    for (int index = 0; index < songs.length; index++) {
      if (this.songs[index] != other.songs[index]) {
        allSame = false;
      }
    }

    if (allSame) {
      return "All of the songs are the same!";
    }
    else {
      return "Not all of the songs are the same.";
    }
  }
  
}`},{path:`Song.java`,text:`/*
 * Represents a song
 */
public class Song {

  private String title;     // The title of a song

  /*
   * Sets title to the specified title
   */
  public Song(String title) {
    this.title = title;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns true if the title of other is the same as this Song
   */
  public boolean equals(Object other) {
    if (other == this) {
      return true;
    }

    if (!(other instanceof Song)) {
      return false;
    }

    Song otherSong = (Song) other;
    return this.title.equals(otherSong.title);
  }
  
}`}],validationFiles:[{path:`PlaylistTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Playlist.java Test")
public class PlaylistTest {

  String messageGap = "\\n       ";
  Song[] testSongs;
  Playlist testPlaylist;
   
  @BeforeEach
  public void setup() {
    testSongs = new Song[]{new Song("Title1"), new Song("Title2")};
    testPlaylist = new Playlist(testSongs);
  }
   
  @Test
  @Order(1)
  @DisplayName("findMatchingPlaylist() returns \\"All of the songs are the same!\\" if the songs in both playlists have the same title => ")
  public void testFindMatchingPlaylistWithMatchingPlaylist() {
    String message = "Check the condition used in the findMatchingPlaylist() method. Is it comparing aliases or instance variables?" + messageGap;
      
    Song[] songs2 = {new Song("Title1"), new Song("Title2")};
    Playlist playlist2 = new Playlist(songs2);

    String result = testPlaylist.findMatchingPlaylist(playlist2);
    assertEquals("All of the songs are the same!", result, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("findMatchingPlaylist() returns \\"Not all of the songs are the same\\" if any songs in both playlists don't have the same title => ")
  public void testFindMatchingPlaylistWithNonMatchingPlaylist() {
    String message = "Check the condition used in the findMatchingPlaylist() method. Is it comparing aliases or instance variables?" + messageGap;

    Song[] songs2 = {new Song("Title3"), new Song("Title4")};
    Playlist playlist2 = new Playlist(songs2);

    String result = testPlaylist.findMatchingPlaylist(playlist2);
    assertEquals("Not all of the songs are the same.", result, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Comparing Objects (d)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of MenuItem objects
    MenuItem[] items = {new MenuItem("burger"), new MenuItem("curry"), new MenuItem("pizza"), new MenuItem("sushi"), new MenuItem("samosa")};

    // Creates a Menu object
    Menu foods = new Menu(items);

    // Creates first test array of MenuItem objects and Menu
    MenuItem[] firstTestItems = {new MenuItem("burger"), new MenuItem("curry"), new MenuItem("pizza"), new MenuItem("sushi"), new MenuItem("samosa")};
    Menu first = new Menu(firstTestItems);

    // Creates second test array of MenuItem objects and Menu
    MenuItem[] secondTestItems = {new MenuItem("burger"), new MenuItem("moussaka"), new MenuItem("pizza"), new MenuItem("sushi"), new MenuItem("paella")};
    Menu second = new Menu(secondTestItems);

    // Calls the findMatchingMeal() method and prints the result
    System.out.println(foods.findMatchingMeal(first) + "\\n");
    System.out.println(foods.findMatchingMeal(second));
    
  }
}`},{path:`Menu.java`,text:`/*
 * Represents a restaurant menu
 */
public class Menu {

  private MenuItem[] items;     // The 1D array of MenuItem objects

  /*
   * Initializes items to the specified 1D array of MenuItem objects
   */
  public Menu(MenuItem[] items) {
    this.items = items;
  }

  /*
   * Returns the 1D array of MenuItem objects
   */
  public MenuItem[] getItems() {
    return items;
  }

  /*
   * Returns a String containing the text "All of the options are the same!"
   * if all MenuItem objects in the menu have the same type
   */
  public String findMatchingMeal(Menu other) {
    boolean allSame = true;

    for (int index = 0; index < items.length; index++) {
      if (this.items[index] != other.items[index]) {
        allSame = false;
      }
    }

    if (allSame) {
      return "All of the options are the same!";
    }
    else {
      return "Not all of the options are the same.";
    }
  }
  
}`},{path:`MenuItem.java`,text:`/*
 * Represents an item on a menu
 */
public class MenuItem {

  private String type;     // The type of menu item

  /*
   * Sets type to the specified type
   */
  public MenuItem(String type) {
    this.type = type;
  }

  /*
   * Returns the type of the menu item
   */
  public String getType() {
    return type;
  }

  /*
   * Returns true if the type of other is the same as this MenuItem
   */
  public boolean equals(Object other) {
    if (other == this) {
      return true;
    }

    if (!(other instanceof MenuItem)) {
      return false;
    }

    MenuItem otherItem = (MenuItem) other;
    return this.type.equals(otherItem.type);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Overriding the equals() Method (a)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates 1D arrays containing movie titles
    String[] tinkerBellMovies = {"Peter Pan", "Return to Neverland", "Tinker Bell"};
    String[] elsaMovies = {"Frozen", "Frozen 2"};
    String[] simbaMovies = {"The Lion King", "The Lion King II: Simba's Pride"};

    // Creates a Character object called tinkerBell
    Character tinkerBell = new Character("Tinker Bell", tinkerBellMovies);

    // Creates a Character object called elsa
    Character elsa = new Character("Elsa", elsaMovies);

    // Creates a 1D array of Character objects
    Character[] characters = {tinkerBell, elsa};

    // Creates a Disney object
    Disney catalog = new Disney(characters);

    // Creates additional Character objects
    Character newTinkerBell = new Character("Tinker Bell", tinkerBellMovies);
    Character simba = new Character("Simba", simbaMovies);

    // Calls the getCharacterFilms() method and prints the results
    System.out.println(catalog.getCharacterFilms(newTinkerBell) + "\\n");
    System.out.println(catalog.getCharacterFilms(simba));
    
  }
}`},{path:`Character.java`,text:`/*
 * Represents a character
 */
public class Character {

  private String name;        // The name of a character
  private String[] movies;    // The 1D array of movies that a character appeared in

  /*
   * Sets name to the specified name and initializes
   * movies to the specified 1D array of movies
   */
  public Character(String name, String[] movies) {
    this.name = name;
    this.movies = movies;
  }

  /*
   * Returns the name of the character
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the 1D array of movies the character appeared in
   */
  public String[] getMovies() {
    return movies;
  }

  /*
   * Returns true if the other Character has the same name as this Character
   */
  public boolean equals(Object other) {
    /* ----------------------------------- TO DO -----------------------------------
      * ✅ Return true if this character has the same name as the other character.
      * If the object is not a Character, return false.
     * -----------------------------------------------------------------------------
     */
    

    return false;
  }

  /*
   * Returns a String containing the name of the character
   * and the movies the character appeared in
   */
  public String toString() {
    String result = name + "\\n----------\\n";

    for (String title : movies) {
      result += title + "\\n";
    }

    return result;
  }
  
}`},{path:`Disney.java`,text:`/*
 * Represents a catalog of Disney characters
 */
public class Disney {

  private Character[] characters;    // The 1D array of Character objects

  /*
   * Initializes characters to the specified 1D array of characters
   */
  public Disney(Character[] characters) {
    this.characters = characters;
  }

  /*
   *
   */
  public Character[] getCharacters() {
    return characters;
  }

  /*
   * Returns the information for the Character from
   * characters that has the same name as other
   */
  public String getCharacterFilms(Character other) {
    String result = "No character found";

    for (Character item : characters) {
      if (item.equals(other)) {
        result = "Character found!\\n" + item;
      }
    }

    return result;
  }
  
}`}],validationFiles:[{path:`DisneyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Character.java Test")
public class DisneyTest {

  String messageGap = "\\n       ";
  Character someCharacter;
  Disney testDisney;
   
  @BeforeEach
  public void setup() {
    String[] testMovies = new String[3];
    someCharacter = new Character("Character1", testMovies);

    Character[] testCharacters = new Character[2];
    testDisney = new Disney(testCharacters);
  }

  @Test
  @Order(1)
  @DisplayName("equals() returns true if this Character points to the same object as other => ")
  public void testGetCharacterFilmsReturnsTrueIfAlias() {
    String message = "Use an if statement to check if other points to the same object as this object." + messageGap;
    Character testCharacter = someCharacter;
    assertTrue(someCharacter.equals(testCharacter), message);
  }

  @Test
  @Order(2)
  @DisplayName("equals() returns false if other is not a Character => ")
  public void testGetCharacterFilmsReturnsFalseIfNotCharacter() {
    String message = "Use an if statement to check if other is not an instance of a Character" + messageGap;
    assertFalse(someCharacter.equals(testDisney), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("equals() method true if two Character objects have the same name => ")
  public void testCharacterEqualsReturnsTrue() {
    String message = "Cast other as a Character and check if this object's name is the same as the other object's name." + messageGap;
      
    String[] testMovies = new String[3];
    Character testCharacter = new Character("Character1", testMovies);

    assertTrue(someCharacter.equals(testCharacter), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("equals() returns false if two Character objects don't have the same name => ")
  public void testCharacterEqualsReturnsFalse() {
    String message = "Cast other as a Character and check if this object's name is NOT the same as the other object's name." + messageGap;
      
    String[] testMovies = new String[3];
    Character testCharacter = new Character("Character3", testMovies);

    assertFalse(someCharacter.equals(testCharacter), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Overriding the equals() Method (b)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Poem objects
    Poem[] poems = {new Poem("T.S. Eliot", "The Waste Land"), new Poem("Edgar Allan Poe", "The Raven"),
                    new Poem("T.S. Eliot", "The Love Song of J. Alfred Prufrock"), new Poem("John Keats", "Ode to a Nightingale"),
                    new Poem("William Wordsworth", "I Wandered Lonely as a Cloud")};

    // Creates a Library object
    Library catalog = new Library(poems);

    // Creates a new Poem object
    Poem another = new Poem("T.S. Eliot", "Four Quartets");

    // Calls the getSimilarPoems() method and prints the results
    System.out.println(catalog.getSimilarPoems(another));
    
  }
}`},{path:`Library.java`,text:`/*
 * Represents a library
 */
public class Library {

  private Poem[] poems;    // The 1D array of Poem objects

  /*
   * Initializes poems to the specified 1D array of Poem objects
   */
  public Library(Poem[] poems) {
    this.poems = poems;
  }

  /*
   * Returns the 1D array of Poem objects
   */
  public Poem[] getPoems() {
    return poems;
  }

  /*
   * Returns a String containing the title of each poem by
   * the same author as other
   */
  public String getSimilarPoems(Poem other) {
    String result = "";

    for (int index = 0; index < poems.length; index++) {
      if (poems[index].equals(other)) {
        result += poems[index] + "\\n";
      }
    }

    if (result.equals("")) {
      result = "No poems found.";
    }

    return result;
  }
  
}`},{path:`Poem.java`,text:`/*
 * Represents a poem
 */
public class Poem {

  private String author;      // The author of a poem
  private String title;       // The title of a poem

  /*
   * Sets author and title to the specified values
   */
  public Poem(String author, String title) {
    this.author = author;
    this.title = title;
  }

  /*
   * Returns the author of the poem
   */
  public String getAuthor() {
    return author;
  }

  /*
   * Returns the title of the poem
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns true if the other Poem has the same author as this Poem
   */
  public boolean equals(Object other) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return true if this poem has the same author as the other poem.
     * If the object is not a Poem, return false.
     * -----------------------------------------------------------------------------
     */

    return false;
  }

  /*
   * Returns a String containing the title and author of the poem
   */
  public String toString() {
    return title + " by " + author;
  }
  
}`}],validationFiles:[{path:`PoemTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Poem.java Test")
public class PoemTest {

  String messageGap = "\\n       ";
  Poem testPoem;
   
  @BeforeEach
  public void setup() {
    testPoem = new Poem("Author1", "Title1");
  }

  @Test
  @Order(1)
  @DisplayName("equals() returns true if this Poem points to the same object as other => ")
  public void testEqualsReturnsTrueIfAlias() {
    String message = "Use an if statement to check if other points to the same object as this object." + messageGap;
    Poem otherPoem = testPoem;
    assertTrue(testPoem.equals(otherPoem), message);
  }

  @Test
  @Order(2)
  @DisplayName("equals() returns false if other is not a Poem => ")
  public void testEqualsReturnsFalseIfNotPoem() {
    String message = "Use an if statement to check if other is not an instance of a Poem" + messageGap;

    Poem[] testPoems = new Poem[2];
    Library testLibrary = new Library(testPoems);
    
    assertFalse(testPoem.equals(testLibrary), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("equals() method true if two Poem objects have the same author => ")
  public void testEqualsReturnsTrue() {
    String message = "Cast other as a Poem and check if this object's author is the same as the other object's author." + messageGap;
    Poem otherPoem = new Poem("Author1", "Title1");
    assertTrue(testPoem.equals(otherPoem), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("equals() returns false if two Poem objects don't have the same author => ")
  public void testEqualsReturnsFalse() {
    String message = "Cast other as a Poem and check if this object's author is NOT the same as the other object's author." + messageGap;
    Poem otherPoem = new Poem("Author2", "Title2");
    assertFalse(testPoem.equals(otherPoem), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Overriding the equals() Method (c)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Player objects
    Player[] players = {new Player("Mike Trout", "Center Fielder"), new Player("Clayton Kershaw", "Starting Pitcher"),
                        new Player("Aaron Judge", "Right Fielder"), new Player("Mookie Betts", "Right Fielder"),
                        new Player("Salvador Perez", "Catcher")};

    // Creates a Team object
    Team baseball = new Team(players);

    // Creates a new Player object
    Player max = new Player("Max Scherzer", "Starting Pitcher");

    // Calls the getSamePositions() method and prints the results
    System.out.println(baseball.getSamePositions(max));
    
  }
}`},{path:`Player.java`,text:`import org.code.theater.*;

/*
 * Represents a baseball player
 */
public class Player {

  private String name;      // The name of a player
  private String position;  // The position of a player

  /*
   * Sets name to the specified name and position
   * to the specified position
   */
  public Player(String name, String position) {
    this.name = name;
    this.position = position;
  }

  /*
   * Returns the name of the player
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the position of the player
   */
  public String getPosition() {
    return position;
  }

  /*
   * Returns true if this Player has the same position as other
   */
  public boolean equals(Object other) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return true if this player has the same position as the other player.
     * If the object is not a Player, return false.
     * -----------------------------------------------------------------------------
     */

    return false;
  }

  /*
   * Returns a String containing the name and position of the player
   */
  public String toString() {
    return name + ": " + position;
  }
  
}`},{path:`Team.java`,text:`/*
 * Represents a team
 */
public class Team {

  private Player[] players;     // The 1D array of Player objects

  /*
   * Initializes players to the specified 1D array of Player objects
   */
  public Team(Player[] players) {
    this.players = players;
  }

  /*
   * Returns the 1D array of players
   */
  public Player[] getPlayers() {
    return players;
  }

  /*
   * Returns a String containing the names of each player
   * that has the same position as other
   */
  public String getSamePositions(Player other) {
    String result = "";

    for (int index = 0; index < players.length; index++) {
      if (players[index].equals(other)) {
        result += players[index] + "\\n";
      }
    }

    if (result.equals("")) {
      result = "No players found.";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`PlayerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Player.java Test")
public class PlayerTest {

  String messageGap = "\\n       ";
  Player testPlayer;
   
  @BeforeEach
  public void setup() {
    testPlayer = new Player("Player1", "Position1");
  }

  @Test
  @Order(1)
  @DisplayName("equals() returns true if this Player points to the same object as other => ")
  public void testEqualsReturnsTrueIfAlias() {
    String message = "Use an if statement to check if other points to the same object as this object." + messageGap;
    Player otherPlayer = testPlayer;
    assertTrue(testPlayer.equals(otherPlayer), message);
  }

  @Test
  @Order(2)
  @DisplayName("equals() returns false if other is not a Player => ")
  public void testEqualsReturnsFalseIfNotPoem() {
    String message = "Use an if statement to check if other is not an instance of a Player" + messageGap;

    Player[] testPlayers = new Player[2];
    Team testTeam = new Team(testPlayers);
    
    assertFalse(testPlayer.equals(testTeam), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("equals() method true if two Player objects have the same position => ")
  public void testEqualsReturnsTrue() {
    String message = "Cast other as a Player and check if this object's position is the same as the other object's position." + messageGap;
    Player otherPlayer = new Player("Player1", "Position1");
    assertTrue(testPlayer.equals(otherPlayer), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("equals() returns false if two Player objects don't have the same position => ")
  public void testEqualsReturnsFalse() {
    String message = "Cast other as a Player and check if this object's position is NOT the same as the other object's position." + messageGap;
    Player otherPlayer = new Player("Player2", "Position2");
    assertFalse(testPlayer.equals(otherPlayer), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Overriding the equals() Method (d)`,lesson:`Lesson 6: Object Aliases and Equality`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates several Color objects
    Color teal = new Color(0, 128, 128);
    Color maroon = new Color(128, 0, 0);
    Color otherColor = new Color(0, 128, 128);

    // Compares the Color objects and prints the results
    System.out.println(teal.equals(maroon));
    System.out.println(teal.equals(otherColor));
    
  }
}`},{path:`Color.java`,text:`/*
 * Represents a custom color
 */
public class Color {

  private int red;      // The red value of a color
  private int green;    // The green value of a color
  private int blue;     // The blue value of a color

  /*
   * Sets red, green, and blue to the specified values
   */
  public Color(int red, int green, int blue) {
    this.red = red;
    this.green = green;
    this.blue = blue;
  }

  /*
   * Returns the red value of the color
   */
  public int getRed() {
    return red;
  }

  /*
   * Returns the green value of the color
   */
  public int getGreen() {
    return green;
  }

  /*
   * Returns the blue value of the color
   */
  public int getBlue() {
    return blue;
  }

  /*
   * Returns true if this CustomColor has the same red,
   * green, and blue values as other
   */
  public boolean equals(Object other) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return true if this color has the same red, green, and blue values
     * as the other color. If the object is not a Color, return false.
     * -----------------------------------------------------------------------------
     */

    return false;
  }

  /*
   * Returns a String containing the red, green, and blue values
   */
  public String toString() {
    return "Red: " + red + ", Green: " + green + ", Blue: " + blue;
  }
  
}`}],validationFiles:[{path:`ColorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Color.java Test")
public class ColorTest {

  String messageGap = "\\n       ";
  Color testColor;
   
  @BeforeEach
  public void setup() {
    testColor = new Color(230, 230, 250);
  }

  @Test
  @Order(1)
  @DisplayName("equals() returns true if this Color points to the same object as other => ")
  public void testEqualsReturnsTrueIfAlias() {
    String message = "Use an if statement to check if other points to the same object as this object." + messageGap;
    Color otherColor = testColor;
    assertTrue(testColor.equals(otherColor), message);
  }

  @Test
  @Order(2)
  @DisplayName("equals() returns false if other is not a Color => ")
  public void testEqualsReturnsFalseIfNotColor() {
    String message = "Use an if statement to check if other is not an instance of a Color" + messageGap;
    Object testObject = new Object();
    assertFalse(testColor.equals(testObject), message);
  }
   
  @Test
  @Order(3)
  @DisplayName("equals() method true if two Color objects have the same red, green, and blue values => ")
  public void testEqualsReturnsTrue() {
    String message = "Cast other as a Color and check if this object's values are the same as the other object's values." + messageGap;
    Color otherColor = new Color(230, 230, 250);
    assertTrue(testColor.equals(otherColor), message);
  }
   
  @Test
  @Order(4)
  @DisplayName("equals() returns false if two Color objects don't have the same color values => ")
  public void testEqualsReturnsFalse() {
    String message = "Cast other as a Color and check if this object's values are NOT the same as the other object's values." + messageGap;
    Color otherColor = new Color(255, 127, 80);
    assertFalse(testColor.equals(otherColor), message);
  }
  
}`}],dataFiles:[]},{name:`Predict and Run: Nested if Statements`,lesson:`Lesson 7: Nested If Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`AgeChecker.java`,text:`public class AgeChecker {

  public String checkAge(int age) {
    if (age <= 18) {
      if (age <= 13) {
        return "Child";
      }
      else {
        return "Teenager";
      }
    }
    else {
      return "Adult";
    }
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Nested if Statements #1`,lesson:`Lesson 7: Nested If Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`AgeChecker.java`,text:`public class AgeChecker {

  public String checkAge(int age) {
    if (age <= 18) {
      if (age <= 13) {
        return "Child";
      }
      else {
        return "Teenager";
      }
    }
    else {
      return "Adult";
    }
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Nested if Statements #2`,lesson:`Lesson 7: Nested If Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`AgeChecker.java`,text:`public class AgeChecker {

  public String checkAge(int age) {
    if (age <= 18) {
      if (age <= 13) {
        return "Child";
      }
      else {
        return "Teenager";
      }
    }
    else {
      return "Adult";
    }
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Nested if Statements (a) — Theater`,lesson:`Lesson 7: Nested If Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Reads the data from the names.txt and causes.txt files into a 1D array of Fire objects
    Fire[] fires = Fire.createFires("names.txt", "causes.txt");

    // Creates a FireData object
    FireData scene = new FireData(fires);
    

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getNumFires() method then pass the result to a call to the 
     * drawResult() method.
     * -----------------------------------------------------------------------------
     */



    
    
    // Plays the scene
    Theater.playScenes(scene);
    
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
  
}`},{path:`Fire.java`,text:`/*
 * Represents a fire event in a city
 */
public class Fire {

  private String name;      // The name of a fire
  private String cause;     // The cause of a fire

  /*
   * Sets name to the specified name and cause
   * to the specified cause
   */
  public Fire(String name, String cause) {
    this.name = name;
    this.cause = cause;
  }

  /*
   * Returns the name of the fire
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the cause of the fire
   */
  public String getCause() {
    return cause;
  }

  /*
   * Returns a 1D array of Fire objects using the data in the namesFile and causesFile
   */
  public static Fire[] createFires(String namesFile, String causesFile) {
    String[] fireNames = FileReader.toStringArray(namesFile);
    String[] causes = FileReader.toStringArray(causesFile);
    Fire[] firesData = new Fire[fireNames.length];

    for (int index = 0; index < firesData.length; index++) {
      firesData[index] = new Fire(fireNames[index], causes[index]);
    }

    return firesData;
  }

  /*
   * Returns a String containing the name of the fire and its cause
   */
  public String toString() {
    return name + " - caused by " + cause;
  }
  
}`},{path:`FireData.java`,text:`import org.code.theater.*;

/*
 * Creates a visualization for fire data
 */
public class FireData extends Scene {

  private Fire[] fires;     // The 1D array of Fire objects

  /*
   * Initializes fires to the specified 1D array of Fire objects
   */
  public FireData(Fire[] fires) {
    this.fires = fires;
  }

  /*
   * Returns the 1D array of Fire objects
   */
  public Fire[] getFires() {
    return fires;
  }

  /*
   * Returns the number of fires caused by the parameter cause
   * where the length of the name of the fire is greater than 6
   */
  public int getNumFires(String cause) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Count and return the number of fires causes by the parameter cause where
     * the length of the name of the fire is greater than 6.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Draws the result in the scene
   */
  public void drawResult(String cause, int count) {
    int numFires = count;
    while (numFires > 0) {
      int randomX = (int)(Math.random() * getWidth());
      int randomY = (int)(Math.random() * getHeight());
      drawImage("fire.png", randomX, randomY, 100);
      pause(0.1);
      numFires--;
    }

    pause(0.5);
    drawText("There have been " + count + " fires", 50, 100);
    drawText("caused by " + cause, 50, 125);
    drawText("where the length of the name", 50, 150);
    drawText("is greater than 6.", 50, 175);
  }
  
}`}],validationFiles:[{path:`FireDataTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("FireData.java Test")
public class FireDataTest {

  String messageGap = "\\n       ";
  Fire[] testFires;
  FireData testScene;
   
  @BeforeEach
  public void setup() {
    testFires = new Fire[]{new Fire("--Name1--", "Cause1"), new Fire("Name1", "Cause1"), new Fire("--Name2--", "Cause2")};
    testScene = new FireData(testFires);
  }
   
  @Test
  @Order(1)
  @DisplayName("getNumFires() returns the number of fires caused by the parameter cause => ")
  public void testGetNumFiresMatchesCause() {
    String message = "Check if the cause of the fire is the same as the parameter cause using the equals() method." + messageGap;
    assertEquals(1, testScene.getNumFires("Cause1"), message);
  }

  @Test
  @Order(2)
  @DisplayName("getNumFires() only counts fires where the length of the name of the fire is greater than 6 => ")
  public void testGetNumFiresNameLength() {
    String message = "Use a nested if statement to check if the length of the fire's name is greater than 6." + messageGap;
    assertEquals(1, testScene.getNumFires("Cause1"), message);
  }
   
}`}],dataFiles:[{path:`causes.txt`,text:`Lightning
Motorist
Motorist
Lightning
Lightning
Other-Public
Public Utility
Motorist
Recreationist
Lightning
Ruralist-Paying
Lightning
Hunter
Motorist
Lightning
Lightning
Motorist
Lightning
Recreationist
Recreationist
Lightning
Lightning
Lightning
Other-Public
Other-Landowner Related`},{path:`names.txt`,text:`Woodley
QUEENS BRANCH
WREN
Ritner Creek
Big Tamarack
COIDC 918
RAYMOND ROAD
BLISS
North Canine
LONG CREEK
7498 Upper Applegate
Mehl Cr.
Corkey
I-5 MP 75
Boomer Hill
Kelly Creek
Pengra Car Fire
GP 632
Smith Road #1
FIRE WINNEY
Doubleday
Vandervert
CB7 AGAIN
Ritter
Mile Post 6 Fire`}]},{name:`Practice: Nested if Statements (b) — Theater`,lesson:`Lesson 7: Nested If Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Reads the data from the text files into a 1D array of Home objects
    Home[] homes = Home.createHomes("towns.txt", "types.txt", "prices.txt");

    // Creates a RealEstate object
    RealEstate scene = new RealEstate(homes);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getHomesByType() method then pass the result to a call to the
     * drawResult() method.
     * -----------------------------------------------------------------------------
     */


    
    
    // Plays the scene
    Theater.playScenes(scene);
    
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
  
}`},{path:`Home.java`,text:`/*
 * Represents a home sold
 */
public class Home {

  private String town;     // The name of the town a home is in
  private String type;     // The type of a home
  private int price;       // The sale amount a home was sold at

  /*
   * Sets town, type, and price to the specified values
   */
  public Home(String town, String type, int price) {
    this.town = town;
    this.type = type;
    this.price = price;
  }

  /*
   * Returns the name of the town
   */
  public String getTown() {
    return town;
  }

  /*
   * Returns the type of the home
   */
  public String getType() {
    return type;
  }

  /*
   * Returns sale amount of the home
   */
  public int getPrice() {
    return price;
  }

  /*
   * Returns a 1D array of Home objects using the data in the specified files
   */
  public static Home[] createHomes(String townsFile, String typesFile, String pricesFile) {
    String[] townsData = FileReader.toStringArray(townsFile);
    String[] typesData = FileReader.toStringArray(typesFile);
    int[] pricesData = FileReader.toIntArray(pricesFile);
    Home[] homesData = new Home[townsData.length];

    for (int index = 0; index < homesData.length; index++) {
      homesData[index] = new Home(townsData[index], typesData[index], pricesData[index]);
    }

    return homesData;
  }

  /*
   * Returns a String containing information about the home
   */
  public String toString() {
    return town + " - " + type + " ($" + price + ")";
  }
  
}`},{path:`RealEstate.java`,text:`import org.code.theater.*;

/*
 * Creates a visualization for real estate data
 */
public class RealEstate extends Scene {

  private Home[] sales;     // The 1D array of Home objects

  /*
   * Initializes sales to the 1D array of Home objects
   */
  public RealEstate(Home[] sales) {
    this.sales = sales;
  }

  /*
   * Returns the 1D array of Home objects
   */
  public Home[] getSales() {
    return sales;
  }

  /*
   * Returns the number of homes sold of the parameter type where
   * the length of the name of the town is greater than 4
   */
  public int getHomesByType(String type) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Count and return the number of homes sold of the parameter type where
     * the length of the name of the town is greater than 4.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Draws the result in the scene
   */
  public void drawResult(String type, int count) {
    int numHomes = count;
    while (numHomes > 0) {
      int randomX = (int)(Math.random() * getWidth());
      int randomY = (int)(Math.random() * getHeight());
      drawImage("house.png", randomX, randomY, 100);
      pause(0.1);
      numHomes--;
    }

    pause(0.5);
    drawText("There were " + count + " homes", 50, 100);
    drawText("sold of type " + type, 50, 125);
    drawText("where the length of the town's name", 50, 150);
    drawText("is greater than 4.", 50, 175);
  }
  
}`}],validationFiles:[{path:`RealEstateTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("RealEstate.java Test")
public class RealEstateTest {

  String messageGap = "\\n       ";
  Home[] testHomes;
  RealEstate testScene;
   
  @BeforeEach
  public void setup() {
    testHomes = new Home[]{new Home("Town1", "Type1", 100), new Home("Town2", "Type2", 100), new Home("Twn", "Type1", 100)};
    testScene = new RealEstate(testHomes);
  }
   
  @Test
  @Order(1)
  @DisplayName("getHomesByType() returns the number of homes that are the same type as the parameter type => ")
  public void testGetHomesByTypeMatchesCause() {
    String message = "Check if the type of the home is the same as the parameter cause using the equals() method." + messageGap;
    assertEquals(1, testScene.getHomesByType("Type1"), message);
  }

  @Test
  @Order(2)
  @DisplayName("getHomesByType() only counts homes where the length of the town's name is greater than 4 => ")
  public void testGetHomesByTypeNameLength() {
    String message = "Use a nested if statement to check if the length of the town's name is greater than 4." + messageGap;
    assertEquals(1, testScene.getHomesByType("Type1"), message);
  }
   
}`}],dataFiles:[{path:`prices.txt`,text:`48000000
45000000
42175000
27927556
26000000
25000000
24000000
22250000
22000000
21750000
21750000
21500000
21000000
20500000
20377000
20000000
19907000
19250000
19000000
18600000
18000000
18000000
17900000
17500000
17300000
17100000
17000000
17000000
17000000
16000000
16000000
16000000
15850000
15088000
15000000
14900000
14875000
14820000
14685000
14500000
14500000
14500000
14400000
14325000
14100000
14000000
14000000
13978066
13900000
13666125`},{path:`towns.txt`,text:`Greenwich
Greenwich
Greenwich
Beacon Falls
Greenwich
Greenwich
Greenwich
Avon
Greenwich
Greenwich
Greenwich
Branford
Greenwich
Darien
Greenwich
Greenwich
Greenwich
Greenwich
Greenwich
Greenwich
Darien
Westport
Stamford
Greenwich
Greenwich
Greenwich
Greenwich
Greenwich
Greenwich
Westport
Fairfield
Greenwich
Greenwich
Greenwich
Greenwich
Greenwich
Greenwich
East Hartford
Greenwich
Greenwich
Greenwich
Brookfield
Westport
Hartford
Greenwich
Greenwich
Greenwich
Stamford
Fairfield
Greenwich`},{path:`types.txt`,text:`Single Family
Single Family
Single Family
Condo
Single Family
Single Family
Single Family
Condo
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Two Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Single Family
Condo
Single Family
Condo
Single Family
Single Family
Single Family
Condo
Single Family
Single Family`}]},{name:`Practice: Nested if Statements (c) #1`,lesson:`Lesson 7: Nested If Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Reads the data from the text files into a 1D array of Meteorite objects
    Meteorite[] meteors = Meteorite.createMeteors("names.txt", "classes.txt", "years.txt");

    // Creates a Landings object
    Landings scene = new Landings(meteors);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getMeteoritesByYear() method then pass the result to a call to
     * the drawResult() method.
     * -----------------------------------------------------------------------------
     */


    
    
    // Plays the scene
    Theater.playScenes(scene);
    
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
  
}`},{path:`Landings.java`,text:`import org.code.theater.*;

/*
 * Creates a visualization for meteorite data
 */
public class Landings extends Scene {

  private Meteorite[] meteors;    // The 1D array of Meteorite objects

  /*
   * Initializes meteors to the specified 1D array of Meteorite objects
   */
  public Landings(Meteorite[] meteors) {
    this.meteors = meteors;
  }

  /*
   * Returns the 1D array of Meteorite objects
   */
  public Meteorite[] getMeteors() {
    return meteors;
  }

  /*
   * Returns the number of meteorites that landed in the parameter year
   * where the length of its recommended classification is greater than 3
   */
  public int getMeteoritesByYear(int year) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Count and return the number of meteorites that landed in the parameter
     * year where the length of its recommended classification is greater than 3.
     * -----------------------------------------------------------------------------
     */

    
    
    return -1;
  }

  /*
   * Draws the result in the scene
   */
  public void drawResult(int year, int count) {
    int numMeteors = count;
    while (numMeteors > 0) {
      int randomX = (int)(Math.random() * getWidth());
      int randomY = (int)(Math.random() * getHeight());
      drawImage("meteorite.png", randomX, randomY, 100);
      pause(0.1);
      numMeteors--;
    }

    pause(0.5);
    drawText("There have been " + count + " meteorites", 50, 100);
    drawText("that landed since " + year, 50, 125);
    drawText("where the length of its class", 50, 150);
    drawText("is greater than 3.", 50, 175);
  }
  
}`},{path:`Meteorite.java`,text:`/*
 * Represents a meteorite landing
 */
public class Meteorite {

  private String name;       // The name of a meteorite
  private String recClass;   // The recommended classification of a meteorite
  private int year;          // The year a meteorite landed

  /*
   * Sets name, class, and year to the specified values
   */
  public Meteorite(String name, String recClass, int year) {
    this.name = name;
    this.recClass = recClass;
    this.year = year;
  }

  /*
   * Returns the name of the meteorite
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the recommended classification of the meteorite
   */
  public String getRecClass() {
    return recClass;
  }

  /*
   * Returns the year the meteorite landed
   */
  public int getYear() {
    return year;
  }

  /*
   * Returns a 1D array of Meteorite objects using the data in the specified text files
   */
  public static Meteorite[] createMeteors(String namesFile, String classesFile, String yearsFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    String[] classesData = FileReader.toStringArray(classesFile);
    int[] yearsData = FileReader.toIntArray(yearsFile);
    Meteorite[] meteorsData = new Meteorite[namesData.length];

    for (int index = 0; index < meteorsData.length; index++) {
      meteorsData[index] = new Meteorite(namesData[index], classesData[index], yearsData[index]);
    }

    return meteorsData;
  }

  /*
   * Returns a String containing information about the meteorite
   */
  public String toString() {
    return name + " (" + year + ") - " + recClass;
  }
  
}`}],validationFiles:[{path:`LandingsTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Landings.java Test")
public class LandingsTest {

  String messageGap = "\\n       ";
  Meteorite[] testMeteorites;
  Landings testScene;
   
  @BeforeEach
  public void setup() {
    testMeteorites = new Meteorite[]{new Meteorite("Meteor1", "Class1", 1900), new Meteorite("Meteor2", "Class2", 2000), new Meteorite("Meteor3", "Cls", 1900)};
    testScene = new Landings(testMeteorites);
  }
   
  @Test
  @Order(1)
  @DisplayName("getMeteoritesByYear() returns the number of meteorites that have landed since the parameter year => ")
  public void testGetMeteoritesByYearMatchesCause() {
    String message = "Check if the year of the Meteorite is greater than or equal to the parameter year." + messageGap;
    assertEquals(2, testScene.getMeteoritesByYear(1900), message);
  }

  @Test
  @Order(2)
  @DisplayName("getMeteoritesByYear() only counts meteorites where the length of the class of the meteorite is greater than 3 => ")
  public void testGetMeteoritesByYearClassLength() {
    String message = "Use a nested if statement to check if the length of the meteorite's class is greater than 3." + messageGap;
    assertEquals(2, testScene.getMeteoritesByYear(1900), message);
  }
   
}`}],dataFiles:[{path:`classes.txt`,text:`H4
H6
H5
R4
L3.7
H4
H~4
Diogenite
H6
L6
L6
L5
L6
LL6
H5
CM2
H6
Eucrite
H5
L5
H4
H5
H5
L6
H6
L6
H5
L6
H6
L4
CO3
H3
Mesosiderite
H5
H4
L4
H4
L6
H4-5
L5
H4
H4
H3
L6
LL6
H5
L5
L6
L6
L4`},{path:`names.txt`,text:`Hammadah al Hamra 038
Frontier Mountain 03067
Aiken
LaPaz Icefield 03805
Northwest Africa 529
Kuleschovka
Yamato 790432
Andryushki
Frontier Mountain 90156
Northwest Africa 7695
Yamato 983432
Grove Mountains 022638
Miller Range 07329
Lewis Cliff 86070
Yamato 981929
Grove Mountains 052449
Northwest Africa 763
Acfer 318
Dar al Gani 995
Ksar Ghilane 011
Dar al Gani 124
Miller Range 090009
Allan Hills 83005
Elephant Moraine 96118
Elephant Moraine 96089
Acfer 398
Miller Range 090245
Dar al Gani 041
Jiddat al Harasis 697
Queen Alexandra Range 90259
Yamato 790476
Yabrin 002
Allan Hills A77024
Sayh al Uhaymir 501
Dar al Gani 375
MacAlpine Hills 041031
Northwest Africa 5721
Grosvenor Mountains 03029
Dhofar 1221
Elephant Moraine 92170
Queen Alexandra Range 99222
Yamato 794028
Northwest Africa 1522
Northwest Africa 4218
Meteorite Hills 00626
Yamato 86309
Yamato 86698
Dhofar 455
Miller Range 07403
Miller Range 07092`},{path:`years.txt`,text:`1997
2008
1979
1990
2006
2009
2009
2011
1980
1994
1993
2003
1982
1982
2003
2001
2002
1997
2003
1939
2006
2006
1990
1992
1986
1979
1988
1996
1993
2001
1979
1999
1979
1995
1991
2003
2006
2001
2002
2001
2001
2002
2003
2010
2000
1986
2003
1996
1984
2009`}]},{name:`Practice: Nested if Statements (d) #1`,lesson:`Lesson 7: Nested If Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Battle objects using the data from the main.txt and enemy.txt files
    Battle[] battles = Battle.createBattles("main.txt", "enemy.txt");

    // Creates a BattleScene object
    BattleScene scene = new BattleScene(battles);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getBattles() method then pass the 1D array returned to a call
     * to the drawResults() method.
     * -----------------------------------------------------------------------------
     */


    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`Battle.java`,text:`/*
 * Represents a character battle
 */
public class Battle {

  private String mainCharacter;      // The name of the main character
  private String mainImage;          // The image file of the main character
  private String enemyCharacter;     // The name of the enemy character
  private String enemyImage;         // The image file of the enemy character

  /*
   * Sets mainCharacter and enemyCharacter to the specified values
   */
  public Battle(String mainCharacter, String enemyCharacter) {
    this.mainCharacter = mainCharacter;
    this.enemyCharacter = enemyCharacter;
    this.mainImage = getImageFile(mainCharacter);
    this.enemyImage = getImageFile(enemyCharacter);
  }

  /*
   * Returns the image file to use based on the character's name
   */
  public String getImageFile(String characterName) {
    String image = "";
    
    if (characterName.equals("Batman")) {
      image = "batman-battle.png";
    }
    
    if (characterName.equals("Superman")) {
      image = "superman-battle.png";
    }

    if (characterName.equals("Wonder Woman")) {
      image = "wonder-woman-battle.png";
    }

    if (characterName.equals("Supergirl")) {
      image = "supergirl.png";
    }

    if (characterName.equals("The Joker")) {
      image = "joker.png";
    }
    
    if (characterName.equals("Lex Luthor")) {
      image = "lexluther.png";
    }

    if (characterName.equals("Ares")) {
      image = "ares.png";
    }

    if (characterName.equals("Reign")) {
      image = "reign.jpg";
    }

    if (characterName.equals("Cyborg Superman")) {
      image = "cyborg.png";
    }

    return image;
  }

  /*
   * Returns the name of the main character
   */
  public String getMainCharacter() {
    return mainCharacter;
  }

  /*
   * Returns the name of the enemy character
   */
  public String getEnemyCharacter() {
    return enemyCharacter;
  }

  /*
   * Returns the image for the main character
   */
  public String getMainImage() {
    return mainImage;
  }

  /*
   * Returns the image for the enemy character
   */
  public String getEnemyImage() {
    return enemyImage;
  }

  /*
   * Returns a 1D array of Battle objects using the data in the mainFile and enemyFile
   */
  public static Battle[] createBattles(String mainFile, String enemyFile) {
    String[] mainData = FileReader.toStringArray(mainFile);
    String[] enemyData = FileReader.toStringArray(enemyFile);
    Battle[] battleData = new Battle[mainData.length];

    for (int index = 0; index < battleData.length; index++) {
      battleData[index] = new Battle(mainData[index], enemyData[index]);
    }

    return battleData;
  }

  /*
   * Returns a String containing the names of the main and enemy characters
   */
  public String toString() {
    return mainCharacter + " vs " + enemyCharacter;
  }
}`},{path:`BattleScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Creates a visualization of character battles
 */
public class BattleScene extends Scene {

  private Battle[] battles;      // The 1D array of Battle objects

  /*
   * Initializes battles to the specified 1D array of Battle objects
   */
  public BattleScene(Battle[] battles) {
    this.battles = battles;
  }

  /*
   * Returns the 1D array of Battle objects
   */
  public Battle[] getAllBattles() {
    return battles;
  }

  /*
   * Finds and returns a 1D array containing all Battle objects where
   * the mainCharacter matches the parameter firstCharacter and the
   * enemeyCharacter matches the parameter secondCharacter
   */
  public Battle[] getBattles(String firstCharacter, String secondCharacter) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Count the number of Battle objects where the mainCharacter matches the
     * parameter firstCharacter and the enemyCharacter matches the parameter
     * secondCharacter. Then create a 1D array and store each matching Battle object
     * in the array and return the array.
     * -----------------------------------------------------------------------------
     */
    

    

    return null;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(Battle[] results) {
    setTextColor("white");
    setTextHeight(20);
    setTextStyle(Font.SANS, FontStyle.BOLD);
    
    drawImage(results[0].getMainImage(), 0, 0, getWidth());
    pause(0.2);
    drawText(results[0].getMainCharacter() + "", 150, 300);

    pause(2);
    clear("black");
    drawText(results.length + " battles against", 100, 150);

    pause(0.5);
    drawImage(results[0].getEnemyImage(), 0, 0, getWidth());
    pause(0.2);
    drawText(results[0].getEnemyCharacter() + "", 150, 300);
  }
}
`},{path:`FileReader.java`,text:`import java.util.Scanner;
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
  
}`}],validationFiles:[{path:`BattleSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BattleScene.java Test")
public class BattleSceneTest {

  String messageGap = "\\n       ";
  Battle[] testBattles;
  BattleScene testScene;
   
  @BeforeEach
  public void setup() {
    testBattles = new Battle[]{
      new Battle("Batman", "Joker"),
      new Battle("Superman", "Lex Luthor"),
      new Battle("Wonder Woman", "Ares"),
      new Battle("Batman", "Reign"),
      new Battle("Superman", "Joker")
    };
    
    testScene = new BattleScene(testBattles);
  }
   
  @Test
  @Order(1)
  @DisplayName("getBattles() returns all battles with specified main and enemy characters => ")
  public void testGetBattlesMatchingCharacters() {
    String message = "Check if the battles have the correct main and enemy characters." + messageGap;
    Battle[] expectedBattles = {testBattles[0]};
    Battle[] actualBattles = testScene.getBattles("Batman", "Joker");
    assertArrayEquals(expectedBattles, actualBattles, message);
  }

  @Test
  @Order(2)
  @DisplayName("getBattles() returns an empty array when no battles match => ")
  public void testGetBattlesNoMatches() {
    String message = "Check if no battles are returned when there are no matches." + messageGap;
    Battle[] expectedBattles = new Battle[0];
    Battle[] actualBattles = testScene.getBattles("Wonder Woman", "Joker");
    assertArrayEquals(expectedBattles, actualBattles, message);
  }
  
}`}],dataFiles:[{path:`enemy.txt`,text:`The Joker
The Joker
The Joker
The Joker
The Joker
The Joker
The Joker
The Joker
The Joker
The Joker
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Lex Luthor
Ares
Ares
Ares
Ares
Ares
Ares
Reign
Reign
Cyborg Superman
Cyborg Superman
Cyborg Superman`},{path:`main.txt`,text:`Batman
Batman
Batman
Batman
Batman
Batman
Batman
Batman
Batman
Batman
Superman
Superman
Superman
Superman
Superman
Superman
Superman
Superman
Superman
Superman
Wonder Woman
Wonder Woman
Wonder Woman
Wonder Woman
Wonder Woman
Wonder Woman
Supergirl
Supergirl
Supergirl
Supergirl
Supergirl`}]},{name:`Practice: Nested if Statements (a) — Neighborhood`,lesson:`Lesson 7: Nested If Statements`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0
1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 0,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
1,0 1,0 0,0 0,0 0,0 0,0 0,0 0,0 1,0 0,0 0,0 0,0
1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 0,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a TrafficPainter object at (0, 0) facing "east" and call
     * the getToStore() method.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`PainterPlus.java`,text:`import org.code.neighborhood.Painter;

/*
 * Represents a Painter that can turn right
 */
public class PainterPlus extends Painter {

  /*
   * Sets the x and y direction to the specified starting location,
   * direction to the specified starting direction, and numPaint
   * to the specified amount of paint to start with
   */
  public PainterPlus(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Turns the PainterPlus object to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }
  
}`},{path:`TrafficPainter.java`,text:`import org.code.neighborhood.*;

/*
 * Represents a Painter that can navigate traffic in The Neighborhood
 */
public class TrafficPainter extends PainterPlus {

  /*
   * Sets the x and y direction to the specified starting location,
   * direction to the specified starting direction, and numPaint
   * to the specified amount of paint to start with
   */
  public TrafficPainter(int x, int y, String direction, int numPaint) {
    super(x, y, direction, numPaint);
  }

  /*
   * Navigates a TrafficPainter object through
   * traffic to reach the store
   */
  public void getToStore() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use nested if statements to navigate a TrafficPainter object from
     * (0, 0) to the store.
     * -----------------------------------------------------------------------------
     */

    
  }
  
}`}],validationFiles:[{path:`NeighborhoodRunnerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@DisplayName("TrafficPainter.java Test")
@TestMethodOrder(OrderAnnotation.class)
public class NeighborhoodRunnerTest {

  TrafficPainter painter;
  NeighborhoodLog neighborhood;
  PainterLog[] painterLogs;
  PainterLog primaryPainterLog;
  String message;

  @BeforeEach
  public void setup() {
    message = "There is an error in your MyNeighborhood.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += "\\n       ";

    painter = partialMockBuilder(TrafficPainter.class)
      .withConstructor(0, 0, "east", 0)
      .addMockedMethod("canMove", new Class[]{})
      .addMockedMethod("canMove", String.class)
      .addMockedMethod("isFacingEast")
      .addMockedMethod("isFacingSouth")
      .addMockedMethod("turnRight")
      .addMockedMethod("turnLeft")
      .createMock();

    getNeighborhoodResults(message);
  }
  
  @Test
  @Order(1)
  @DisplayName("TrafficPainter turns right if facing east and can move south => ")
  public void testGetToStoreTurnsRight() {
    expect(painter.canMove()).andReturn(true);
    expect(painter.isFacingEast()).andReturn(true);
    expect(painter.canMove("south")).andReturn(true);
    painter.turnRight();
    expect(painter.isFacingSouth()).andReturn(false);
    expect(painter.canMove()).andReturn(false);
    
    replay(painter);

    painter.getToStore();
    verify(painter);
  }

  @Test
  @Order(2)
  @DisplayName("TrafficPainter turns left if facing south and can move east => ")
  public void testGetToStoreTurnsLeft() {
    expect(painter.canMove()).andReturn(true);
    expect(painter.isFacingEast()).andReturn(false);
    expect(painter.isFacingSouth()).andReturn(true);
    expect(painter.canMove("east")).andReturn(true);
    painter.turnLeft();
    expect(painter.canMove()).andReturn(false);
    
    replay(painter);

    painter.getToStore();
    verify(painter);
  }

  @Test
  @Order(3)
  @DisplayName("TrafficPainter object instantiated at (0, 0) facing east => ")
  public void testTrafficPainterInstantiated() {
    message = "Instantiate a TrafficPainter object at (0, 0) facing east.";
    message += "\\n       ";

    int startingXLocation = primaryPainterLog.getStartingPosition().getX();
    int startingYLocation = primaryPainterLog.getStartingPosition().getY();
    String startingDirection = primaryPainterLog.getStartingPosition().getDirection();

    assertEquals(0, startingXLocation, message);
    assertEquals(0, startingYLocation, message);
    assertEquals("east", startingDirection, message);
  }

  @Test
  @Order(4)
  @DisplayName("TrafficPainter navigates to the store (x: 8, y: 8) => ")
  public void testTrafficPainterNavigatesToStore() {
    message = "Call the getToStore() method to nevigate to the store. The method should ";
    message += "\\n        move forward while can move, turn right if facing east and can move ";
    message += "\\n        south, and turn left if facing south and can move east. ";
    message += "\\n       ";

    int endingXLocation = primaryPainterLog.getEndingPosition().getX();
    int endingYLocation = primaryPainterLog.getEndingPosition().getY();

    assertEquals(8, endingXLocation, message);
    assertEquals(8, endingYLocation, message);
  }

  private void getNeighborhoodResults(String message) {
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Nested if Statements (b) — Neighborhood`,lesson:`Lesson 7: Nested If Statements`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a RowPainter object and call the paintArea() method.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`RowPainter.java`,text:`import org.code.neighborhood.*;

/*
 * A Painter that paints rows in The Neighborhood
 */
public class RowPainter extends Painter {

  /*
   * Sets the x and y values to the starting x and y location,
   * direction to the specified starting direction, and paint
   * to the specified starting paint amount
   */
  public RowPainter(int x, int y, String direction, int paint) {
    super(x, y, direction, paint);
  }

  /*
   * Turns the RowPainter object to the right
   */
  public void turnRight() {
    turnLeft();
    turnLeft();
    turnLeft();
  }

  /*
   * Moves the RowPainter object forward in the direction it is facing if it can move
   */
  public void moveIfCanMove() {
    if (canMove()) {
      move();
    }
  }

  /*
   * Paints a row in The Neighborhood with firstColor
   * then moves to the next row with secondColor
   */
  public void paintArea(String firstColor, String secondColor) {
    while (canMove()) {
      paintEast(firstColor);
      paintWest(secondColor);
      moveIfCanMove();

      if (!canMove()) {
        paintEast(firstColor);
        paintWest(secondColor);
        moveToNextRow();
      }
    }
  }

  /*
   * Paints a row if facing east and y location is even
   */
  public void paintEast(String color) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ If the RowPainter object is facing east and its y location is an even
     * number, paint with the parameter color.
     * -----------------------------------------------------------------------------
     */


    
    
  }

  /*
   * Paints a row if facing west and y location is odd
   */
  public void paintWest(String color) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ If the RowPainter object is facing west and its y location is an odd
     * number, paint with the parameter color.
     * -----------------------------------------------------------------------------
     */


    
    
  }

  /*
   * Moves to the next row if unable to move forward
   */
  public void moveToNextRow() {
    if (canMove("south")) {
      if (facingEast()) {
        turnRight();
        moveIfCanMove();
        turnRight();
      }
      else {
        turnLeft();
        moveIfCanMove();
        turnLeft();
      }
    }
  }
  
}`}],validationFiles:[{path:`RowPainterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import org.code.neighborhood.*;
import org.code.validation.*;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@DisplayName("RowPainter.java Test")
@TestMethodOrder(OrderAnnotation.class)
public class RowPainterTest {

  RowPainter painter;
  NeighborhoodLog neighborhood;
  PainterLog[] painterLogs;
  PainterLog primaryPainterLog;
  String message;

  @BeforeEach
  public void setup() {
    message = "There is an error in your MyNeighborhood.";
    message += "\\n        The Painter might not be instantiated or tried to move off the grid.";
    message += "\\n       ";

    painter = partialMockBuilder(RowPainter.class)
      .withConstructor(1, 4, "east", 40)
      .addMockedMethod("isFacingEast")
      .addMockedMethod("isFacingWest")
      .addMockedMethod("getY")
      .addMockedMethod("paint")
      .createMock();

    getNeighborhoodResults(message);
  }
  
  @Test
  @Order(1)
  @DisplayName("RowPainter paints using the parameter \\"color\\" if facing east and y location is even => ")
  public void testPaintEast() {
    expect(painter.isFacingEast()).andReturn(true);
    expect(painter.getY()).andReturn(4);
    painter.paint("red");
    
    replay(painter);

    painter.paintEast("red");
    verify(painter);
  }

  @Test
  @Order(2)
  @DisplayName("RowPainter paints using parameter \\"color\\" if facing west and y location is odd => ")
  public void testPaintWest() {
    expect(painter.isFacingWest()).andReturn(true);
    expect(painter.getY()).andReturn(3);
    painter.paint("orange");

    replay(painter);

    painter.paintWest("orange");
    verify(painter);
  }

  @Test
  @Order(3)
  @DisplayName("RowPainter object instantiated at (1, 4) facing east with 40 units of paint => ")
  public void testRowPlusInstantiated() {
    message = "Instantiate a RowPainter object at (1, 4) facing east with 40 units of paint.";
    message += "\\n       ";

    int startingXLocation = primaryPainterLog.getStartingPosition().getX();
    int startingYLocation = primaryPainterLog.getStartingPosition().getY();
    String startingDirection = primaryPainterLog.getStartingPosition().getDirection();
    int startingPaint = primaryPainterLog.getStartingPaintCount();

    assertEquals(1, startingXLocation, message);
    assertEquals(4, startingYLocation, message);
    assertEquals("east", startingDirection, message);
    assertEquals(40, startingPaint, message);
  }

  @Test
  @Order(4)
  @DisplayName("RowPainter paints the area within the walls => ")
  public void testRowPainterPaintsArea() {
    message = "Call the paintArea() method to paint the area within the walls. The method should ";
    message += "\\n        move forward while can move, paint using \\"firstColor\\" if facing east and y location ";
    message += "\\n        is even, and paint using \\"secondColor\\" if facing west and y location is odd. ";
    message += "\\n       ";

    boolean[][] expected = new boolean[12][12];

    expected[1] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[2] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[3] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[4] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[5] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[6] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[7] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[8] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[9] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};
    expected[10] = new boolean[]{false, false, false, false, true, true, true, true, false, false, false, false};

    assertTrue(neighborhood.finalOutputContainsPaint(expected), message);
  }

  private void getNeighborhoodResults(String message) {
    try {
      neighborhood = NeighborhoodTestRunner.run();
      painterLogs = neighborhood.getPainterLogs();
      primaryPainterLog = painterLogs[0];
    } catch (Exception e) {
      fail(message);
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Nested if Statements (c) #2`,lesson:`Lesson 7: Nested If Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a SocialMediaScene object
    SocialMediaScene scene = new SocialMediaScene(1358, 10841, 4134);

    /* ------------------------------------- TO DO -------------------------------------
     * ✅ Call the getImage() method then call the drawResults() method with the result.
     * ---------------------------------------------------------------------------------
     */


    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`SocialMediaScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that illustrates
 * which social media app had the most users
 */
public class SocialMediaScene extends Scene {

  private int twitter;     // The number of users on Twitter
  private int facebook;    // The number of users on Facebook
  private int instagram;   // The number of users on Instagram

  /*
   * Sets twitter, facebook, and instagram to the specified values
   */
  public SocialMediaScene(int twitter, int facebook, int instagram) {
    this.twitter = twitter;
    this.facebook = facebook;
    this.instagram = instagram;
  }

  /*
   * Sets the number of users on Twitter
   */
  public void setTwitter(int newUsers) {
    this.twitter = newUsers;
  }

  /*
   * Sets the number of users on Facebook
   */
  public void setFacebook(int newUsers) {
    this.facebook = newUsers;
  }

  /*
   * Sets the number of users on Instagram
   */
  public void setInstagram(int newUsers) {
    this.instagram = newUsers;
  }

  /*
   * Returns the icon to draw for the social
   * media app that has the most users
   */
  public String getImage() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use nested if statements to determine which social media app has the
     * most users and return its icon image.
     * -----------------------------------------------------------------------------
     */
    
    
    return "";
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(String imageFile) {
    drawImage(imageFile, 50, 50, 200);
  }
  
}`}],validationFiles:[{path:`SocialMediaSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@DisplayName("SocialMediaScene.java Test")
@TestMethodOrder(OrderAnnotation.class)
public class SocialMediaSceneTest {

  SocialMediaScene testScene;
  String message;

  @BeforeEach
  public void setup() {
    testScene = new SocialMediaScene(0, 0, 0);
  }
  
  @Test
  @Order(1)
  @DisplayName("Return \\"twitter.png\\" if \\"twitter\\" is greater than \\"facebook\\" and \\"instagram\\" => ")
  public void testGetImageReturnsTwitter() {
    message = "Check your conditional statements in the getImage() method. Make sure to check if \\"twitter\\" is ";
    message += "\\n        greater than \\"facebook\\" and greater than \\"instagram\\" and return \\"twitter.png\\" if true.";
    message += "\\n       ";

    testScene.setTwitter(10);
    testScene.setFacebook(0);
    testScene.setInstagram(0);
    String actual = testScene.getImage();

    assertEquals("twitter.png", actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Return \\"facebook.png\\" if \\"facebook\\" is greater than \\"twitter\\" and \\"instagram\\" => ")
  public void testGetImageReturnsFacebook() {
    message = "Check your conditional statements in the getImage() method. Make sure to check if \\"facebook\\" is ";
    message += "\\n        greater than \\"twitter\\" and greater than \\"instagram\\" and return \\"facebook.png\\" if true.";
    message += "\\n       ";

    testScene.setTwitter(0);
    testScene.setFacebook(10);
    testScene.setInstagram(0);
    String actual = testScene.getImage();

    assertEquals("facebook.png", actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("Return \\"instagram.png\\" if \\"instagram\\" is greater than \\"facebook\\" and \\"twitter\\" => ")
  public void testGetImageReturnsInstagram() {
    message = "Check your conditional statements in the getImage() method. Make sure to check if \\"instagram\\" is ";
    message += "\\n        greater than \\"instagram\\" and greater than \\"facebook\\" and return \\"instagram.png\\" if true.";
    message += "\\n       ";

    testScene.setTwitter(0);
    testScene.setFacebook(0);
    testScene.setInstagram(10);
    String actual = testScene.getImage();

    assertEquals("instagram.png", actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Nested if Statements (d) #2`,lesson:`Lesson 7: Nested If Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array containing the SAT scores
    int[] scores = FileReader.toIntArray("satscores.txt");

    // Creates a SATScene object
    SATScene scene = new SATScene(scores);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the analyzeData() method then call the drawResults() method to
     * display the results.
     * -----------------------------------------------------------------------------
     */


    

    // Plays the scene
    Theater.playScenes(scene);
    
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
  
}`},{path:`SATScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that illustrates SAT score ranges
 */
public class SATScene extends Scene {

  private int[] satScores;    // The 1D array of SAT scores
  private int topScores;      // The number of SAT scores in the top range (1340-1600)
  private int competitive;    // The number of SAT scores in the competitive range (1200-1320)
  private int aboveAverage;   // The number of SAT scores in the above average range (1030-1180)
  private int belowAverage;   // The number of SAT scores in the below average range (400-1010)

  /*
   * Initializes satScores with the specified 1D array
   * of SAT scores and sets the counts for each range to 0
   */
  public SATScene(int[] satScores) {
    this.satScores = satScores;
    
    topScores = 0;
    competitive = 0;
    aboveAverage = 0;
    belowAverage = 0;
  }

  /*
   * Returns the current value of topScores
   */
  public int getTopScores() {
    return topScores;
  }

  /*
   * Returns the current value of competitive
   */
  public int getCompetitive() {
    return competitive;
  }

  /*
   * Returns the current value of aboveAverage
   */
  public int getAboveAverage() {
    return aboveAverage;
  }

  /*
   * Returns the current value of belowAverage
   */
  public int getBelowAverage() {
    return belowAverage;
  }

  /*
   * Analyzes the values in the satScores array to count the
   * number of scores in each range
   */
  public void analyzeData() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the satScores array and use nested if statements to count the
     * number of scores in each range
     * -----------------------------------------------------------------------------
     */
    
    
    
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults() {
    setFillColor("green");
    drawEllipse(50, 50, topScores * 2, topScores * 2);

    setFillColor("blue");
    drawEllipse(300, 100, competitive * 2, competitive * 2);

    setFillColor("orange");
    drawEllipse(150, 100, aboveAverage * 2, aboveAverage * 2);

    setFillColor("red");
    drawEllipse(150, 200, belowAverage * 2, belowAverage * 2);
  }
  
}`}],validationFiles:[{path:`SATSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@DisplayName("SATScene.java Test")
@TestMethodOrder(OrderAnnotation.class)
public class SATSceneTest {

  SATScene testScene;
  int[] studentScores;
  int[] testScores;
  int testTopScores = 0;
  int testCompetitive = 0;
  int testAboveAverage = 0;
  int testBelowAverage = 0;
  String message;

  @BeforeEach
  public void setup() {
    studentScores = FileReader.toIntArray("satscores.txt");
    testScene = new SATScene(studentScores);

    testScores = FileReader.toIntArray("satscores.txt");
    analyzeData();
  }
  
  @Test
  @Order(1)
  @DisplayName("Add to \\"topScores\\" if less than or equal to 1600 and greater than or equal to 1340 => ")
  public void testTotalTopScores() {
    message = "Check your conditional statements in the analyzeData() method. Make sure to add 1 to \\"topScores\\" ";
    message += "\\n        if the value is less than or equal to 1600 and greater than or equal to 1340. ";
    message += "\\n       ";
    
    testScene.analyzeData();
    int actual = testScene.getTopScores();

    assertEquals(testTopScores, actual, message);
  }

  @Test
  @Order(2)
  @DisplayName("Add to \\"competitive\\" if less than or equal to 1320 and greater than or equal to 1200 => ")
  public void testTotalCompetitiveScores() {
    message = "Check your conditional statements in the getImage() method. Make sure to add 1 to \\"competitive\\" ";
    message += "\\n        if the value is less than or equal to 1320 and greater than or equal to 1200.";
    message += "\\n       ";

    testScene.analyzeData();
    int actual = testScene.getCompetitive();

    assertEquals(testCompetitive, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("Add to \\"aboveAverage\\" if less than or equal to 1180 and greater than or equal to 1030 => ")
  public void testTotalAboveAverageScores() {
    message = "Check your conditional statements in the getImage() method. Make sure to add 1 to \\"aboveAverage\\" ";
    message += "\\n        if the value is less than or equal to 1180 and greater than or equal to 1030.";
    message += "\\n       ";

    testScene.analyzeData();
    int actual = testScene.getAboveAverage();

    assertEquals(testAboveAverage, actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("Add to \\"belowAverage\\" if less than or equal to 1010 and greater than or equal to 400 => ")
  public void testTotalBelowAverageScores() {
    message = "Check your conditional statements in the getImage() method. Make sure to add 1 to \\"belowAverage\\" ";
    message += "\\n        if the value is less than or equal to 1010 and greater than or equal to 400.";
    message += "\\n       ";

    testScene.analyzeData();
    int actual = testScene.getBelowAverage();

    assertEquals(testBelowAverage, actual, message);
  }

  private void analyzeData() {    
    for (int index = 0; index < testScores.length; index++) {
      if (testScores[index] <= 1600) {
        if (testScores[index] >= 1340) {
          testTopScores++;
        }
      }

      if (testScores[index] <= 1320) {
        if (testScores[index] >= 1200) {
          testCompetitive++;
        }
      }

      if (testScores[index] <= 1180) {
        if (testScores[index] >= 1030) {
          testAboveAverage++;
        }
      }

      if (testScores[index] <= 1010) {
        if (testScores[index] >= 400) {
          testBelowAverage++;
        }
      }
    }
  }
  
}`}],dataFiles:[{path:`satscores.txt`,text:`1600
1593
1587
1580
1573
1567
1560
1553
1547
1540
1533
1527
1520
1513
1507
1500
1493
1487
1480
1473
1467
1460
1453
1447
1440
1433
1427
1420
1413
1407
1400
1393
1387
1380
1373
1367
1360
1353
1347
1340
1333
1327
1320
1313
1307
1300
1293
1287
1280
1273
1267
1260
1253
1247
1240
1233
1227
1220
1213
1207
1200
1193
1187
1180
1173
1167
1160
1153
1147
1140
1133
1127
1120
1113
1107
1100
1093
1087
1080
1073
1067
1060
1053
1047
1040
1033
1027
1020
1013
1007
1000
993
987
980
973
967
960
953
947
940
933
927
920
913
907
900
893
887
880
873
867
860
853
847
840
833
827
820
813
807
800
793
787
780
773
767
760
753
747
740
733
727
720
713
707
700
693
687
680
673
667
660
653
647
640
633
627
620
613
607
600
593
587
580
573
567
560
553
547
540
533
527
520
513
507
500
493
487
480
473
467
460
453
447
440
433
427
420
413
407
400`}]},{name:`Predict and Run: Logical Operators`,lesson:`Lesson 8: Logical Operators`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`import java.util.Scanner;

/*
 * Represents a course at a school
 */ 
public class Course {

  private String teacher;    // The name of the teacher

  /*
   * Sets teacher to the given name of the teacher
   */
  public Course(String teacher) {
    this.teacher = teacher;
  }

  /*
   * Prompts the user to enter grades while the input is not -1
   * and prints the score entered
   */
  public void promptForGrade() {
    Scanner input = new Scanner(System.in);
    int score = 0;

    while (score != -1) {
      System.out.print("Enter the student's score (or -1 to quit): ");
      score = input.nextInt();
      printResult(score);
    }

    input.close();
  }

  /*
   * Prints the score provided by the user
   */
  public void printResult(int enteredScore) {
    if (isValidScore(enteredScore)) {
      System.out.println("Score is: " + enteredScore);
    }
    else {
      if (enteredScore == -1) {
        System.out.println("Goodbye!");
      }
      else {
        System.out.println("The score is invalid. Please try again.");
      }
    }
  }

  /*
   * Returns true if the score is valid, otherwise returns false
   */
  public boolean isValidScore(int score) {
    if (score >= 50 && score <= 100) {
      return true;
    }
    else {
      return false;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Logical Operators #1`,lesson:`Lesson 8: Logical Operators`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`import java.util.Scanner;

/*
 * Represents a course at a school
 */ 
public class Course {

  private String teacher;    // The name of the teacher

  /*
   * Sets teacher to the given name of the teacher
   */
  public Course(String teacher) {
    this.teacher = teacher;
  }

  /*
   * Prompts the user to enter grades while the input is not -1
   * and prints the score entered
   */
  public void promptForGrade() {
    Scanner input = new Scanner(System.in);
    int score = 0;

    while (score != -1) {
      System.out.print("Enter the student's score (or -1 to quit): ");
      score = input.nextInt();
      printResult(score);
    }

    input.close();
  }

  /*
   * Prints the score provided by the user
   */
  public void printResult(int enteredScore) {
    if (isValidScore(enteredScore)) {
      System.out.println("Score is: " + enteredScore);
    }
    else {
      if (enteredScore == -1) {
        System.out.println("Goodbye!");
      }
      else {
        System.out.println("The score is invalid. Please try again.");
      }
    }
  }

  /*
   * Returns true if the score is valid, otherwise returns false
   */
  public boolean isValidScore(int score) {
    if (score >= 50 && score <= 100) {
      return true;
    }
    else {
      return false;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Logical Operators #2`,lesson:`Lesson 8: Logical Operators`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`import java.util.Scanner;

/*
 * Represents a course at a school
 */ 
public class Course {

  private String teacher;    // The name of the teacher

  /*
   * Sets teacher to the given name of the teacher
   */
  public Course(String teacher) {
    this.teacher = teacher;
  }

  /*
   * Prompts the user to enter grades while the input is not -1
   * and prints the score entered
   */
  public void promptForGrade() {
    Scanner input = new Scanner(System.in);
    int score = 0;

    while (score != -1) {
      System.out.print("Enter the student's score (or -1 to quit): ");
      score = input.nextInt();
      printResult(score);
    }

    input.close();
  }

  /*
   * Prints the score provided by the user
   */
  public void printResult(int enteredScore) {
    if (isValidScore(enteredScore)) {
      System.out.println("Score is: " + enteredScore);
    }
    else {
      if (enteredScore == -1) {
        System.out.println("Goodbye!");
      }
      else {
        System.out.println("The score is invalid. Please try again.");
      }
    }
  }

  /*
   * Returns true if the score is valid, otherwise returns false
   */
  public boolean isValidScore(int score) {
    if (score >= 50 && score <= 100) {
      return true;
    }
    else {
      return false;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Logical Operators #3`,lesson:`Lesson 8: Logical Operators`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`import java.util.Scanner;

/*
 * Represents a course at a school
 */ 
public class Course {

  private String teacher;    // The name of the teacher

  /*
   * Sets teacher to the given name of the teacher
   */
  public Course(String teacher) {
    this.teacher = teacher;
  }

  /*
   * Prompts the user to enter grades while the input is not -1
   * and prints the score entered
   */
  public void promptForGrade() {
    Scanner input = new Scanner(System.in);
    int score = 0;

    while (score != -1) {
      System.out.print("Enter the student's score (or -1 to quit): ");
      score = input.nextInt();
      printResult(score);
    }

    input.close();
  }

  /*
   * Prints the score provided by the user
   */
  public void printResult(int enteredScore) {
    if (isValidScore(enteredScore)) {
      System.out.println("Score is: " + enteredScore);
    }
    else {
      if (enteredScore == -1) {
        System.out.println("Goodbye!");
      }
      else {
        System.out.println("The score is invalid. Please try again.");
      }
    }
  }

  /*
   * Returns true if the score is valid, otherwise returns false
   */
  public boolean isValidScore(int score) {
    if (score >= 50 && score <= 100) {
      return true;
    }
    else {
      return false;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Multi-Selection Statements`,lesson:`Lesson 10: Multi-Selection Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Order.java`,text:`/*
 * Represents a customer's order
 */
public class Order {

  private boolean isFilled;     // Whether or not an order has been filled
  private double cost;          // The cost of an order
  private String shipping;      // The shipping method to send an order

  /*
   * Sets isFilled to the specified status, cost to the specified
   * cost and shipping to the specified shipping method
   */
  public Order(boolean isFilled, double cost, String shipping) {
    this.isFilled = isFilled;
    this.cost = cost;
    this.shipping = shipping;
  }

  /*
   * Returns a String containing the shipping cost if the order is filled,
   * other returns a String indicating that the order is not ready
   */
  public String ship() {
    if (isFilled) {
      return "Shipping cost: " + calculateShipping();
    }
    else {
      return "Order not ready.";
    }
  }

  /*
   * Returns the shipping cost based on the shipping method
   */
  public double calculateShipping() {
    if (shipping.equals("Regular")) {
      return 0;
    }
    else if (shipping.equals("Express")) {
      return 1.75;
    }
    else {
      return 0.50;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Multi-Selection Statements #1`,lesson:`Lesson 10: Multi-Selection Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Order.java`,text:`/*
 * Represents a customer's order
 */
public class Order {

  private boolean isFilled;     // Whether or not an order has been filled
  private double cost;          // The cost of an order
  private String shipping;      // The shipping method to send an order

  /*
   * Sets isFilled to the specified status, cost to the specified
   * cost and shipping to the specified shipping method
   */
  public Order(boolean isFilled, double cost, String shipping) {
    this.isFilled = isFilled;
    this.cost = cost;
    this.shipping = shipping;
  }

  /*
   * Returns a String containing the shipping cost if the order is filled,
   * other returns a String indicating that the order is not ready
   */
  public String ship() {
    if (isFilled) {
      return "Shipping cost: " + calculateShipping();
    }
    else {
      return "Order not ready.";
    }
  }

  /*
   * Returns the shipping cost based on the shipping method
   */
  public double calculateShipping() {
    if (shipping.equals("Regular")) {
      return 0;
    }
    else if (shipping.equals("Express")) {
      return 1.75;
    }
    else {
      return 0.50;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Multi-Selection Statements #2`,lesson:`Lesson 10: Multi-Selection Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Order.java`,text:`/*
 * Represents a customer's order
 */
public class Order {

  private boolean isFilled;     // Whether or not an order has been filled
  private double cost;          // The cost of an order
  private String shipping;      // The shipping method to send an order

  /*
   * Sets isFilled to the specified status, cost to the specified
   * cost and shipping to the specified shipping method
   */
  public Order(boolean isFilled, double cost, String shipping) {
    this.isFilled = isFilled;
    this.cost = cost;
    this.shipping = shipping;
  }

  /*
   * Returns a String containing the shipping cost if the order is filled,
   * other returns a String indicating that the order is not ready
   */
  public String ship() {
    if (isFilled) {
      return "Shipping cost: " + calculateShipping();
    }
    else {
      return "Order not ready.";
    }
  }

  /*
   * Returns the shipping cost based on the shipping method
   */
  public double calculateShipping() {
    if (shipping.equals("Regular")) {
      return 0;
    }
    else if (shipping.equals("Express")) {
      return 1.75;
    }
    else {
      return 0.50;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Multi-Selection Statements #3`,lesson:`Lesson 10: Multi-Selection Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Order.java`,text:`/*
 * Represents a customer's order
 */
public class Order {

  private boolean isFilled;     // Whether or not an order has been filled
  private double cost;          // The cost of an order
  private String shipping;      // The shipping method to send an order

  /*
   * Sets isFilled to the specified status, cost to the specified
   * cost and shipping to the specified shipping method
   */
  public Order(boolean isFilled, double cost, String shipping) {
    this.isFilled = isFilled;
    this.cost = cost;
    this.shipping = shipping;
  }

  /*
   * Returns a String containing the shipping cost if the order is filled,
   * other returns a String indicating that the order is not ready
   */
  public String ship() {
    if (isFilled) {
      return "Shipping cost: " + calculateShipping();
    }
    else {
      return "Order not ready.";
    }
  }

  /*
   * Returns the shipping cost based on the shipping method
   */
  public double calculateShipping() {
    if (shipping.equals("Regular")) {
      return 0;
    }
    else if (shipping.equals("Express")) {
      return 1.75;
    }
    else {
      return 0.50;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Writing Algorithms (a)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Brand objects using data from text files
    Brand[] brands = Brand.createBrands("names.txt", "genders.txt");

    // Creates a Fashion object
    Fashion scene = new Fashion(brands);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the countByBrands() method then call the drawResults() method
     * with the results.
     * -----------------------------------------------------------------------------
     */


    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`Brand.java`,text:`/*
 * Represents a brand of clothing
 */
public class Brand {

  private String name;       // The name of a brand of clothing that was purchased
  private String gender;     // The person's gender that purchased the clothing

  /*
   * Sets name and gender to the specified values
   */
  public Brand(String name, String gender) {
    this.name = name;
    this.gender = gender;
  }

  /*
   * Returns the name of the brand of clothing that was purchased
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the person's gender that purchased the clothing
   */
  public String getGender() {
    return gender;
  }

  /*
   * Returns a 1D array of Brand objects using the data from the specified text files
   */
  public static Brand[] createBrands(String namesFile, String gendersFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    String[] gendersData = FileReader.toStringArray(gendersFile);
    Brand[] brandsData = new Brand[namesData.length];

    for (int index = 0; index < brandsData.length; index++) {
      brandsData[index] = new Brand(namesData[index], gendersData[index]);
    }

    return brandsData;
  }

}`},{path:`Fashion.java`,text:`import org.code.theater.*;

/*
 * Creates a visual to show popular brands
 */
public class Fashion extends Scene {

  private Brand[] brands;     // The 1D array of Brand objects

  /*
   * Initializes brands to the specified 1D array of Brand objects
   */
  public Fashion(Brand[] brands) {
    this.brands = brands;
  }

  /*
   * Returns the 1D array of Brand objects
   */
  public Brand[] getBrands() {
    return brands;
  }

  /*
   * Returns a 1D array containing the number of "US Polo" purchases at index 0,
   * the number of "GAP" purchases at index 1, the number of "Kenneth Cole"
   * purchases at index 2, and other brands at index 3 made by the parameter gender
   */
  public int[] countByBrands(String gender) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ For each brand, check if its gender matches the parameter gender and
     * increment the appropriate index in a 1D array of int values based on
     * the name of the brand. Return the 1D array.
     * -----------------------------------------------------------------------------
     */

    
    
    return null;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(String gender, int[] results) {
    String[] colors = {"red", "purple", "green", "orange", "blue"};
    drawText("Brands purchased by " + gender, 55, 40);

    int yLocation = 90;
    
    for (int index = 0; index < results.length; index++) {
      int randomIndex = (int)(Math.random() * colors.length);
      setFillColor(colors[randomIndex]);
      
      drawRectangle(0, yLocation, results[index] * 20, 25);
      pause(0.2);
      yLocation += 40;
    }
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
  
}`}],validationFiles:[{path:`FashionTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Fashion.java Test")
public class FashionTest {

  String messageGap = "\\n       ";
  Brand[] testBrands;
  Fashion testScene;
   
  @BeforeEach
  public void setup() {
    testBrands = new Brand[]{new Brand("US Polo", "Women"), new Brand("US Polo", "Men"),
                             new Brand("GAP", "Women"), new Brand("GAP", "Men"),
                             new Brand("Kenneth Cole", "Women"), new Brand("Kenneth Cole", "Men"),
                             new Brand("Puma", "Women"), new Brand("Puma", "Men")};
    
    testScene = new Fashion(testBrands);
  }
   
  @Test
  @Order(1)
  @DisplayName("Increments the first index if the brands's gender matches the parameter gender and name is \\"US Polo\\" => ")
  public void testCountByBrandsUSPolo() {
    String message = "Check if the brand's gender matches the parameter gender and name equals \\"US Polo\\"." + messageGap;
    int[] actual = testScene.countByBrands("Women");
    assertEquals(1, actual[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("Increments the second index if the brands's gender matches the parameter gender and name is \\"GAP\\" => ")
  public void testCountByBrandsGAP() {
    String message = "Check if the brand's gender matches the parameter gender and name equals \\"GAP\\"." + messageGap;
    int[] actual = testScene.countByBrands("Women");
    assertEquals(1, actual[1], message);
  }

  @Test
  @Order(3)
  @DisplayName("Increments the first index if the brands's gender matches the parameter gender and name is \\"Kenneth Cole\\" => ")
  public void testCountByBrandsKennethCole() {
    String message = "Check if the brand's gender matches the parameter gender and name equals \\"Kenneth Cole\\"." + messageGap;
    int[] actual = testScene.countByBrands("Women");
    assertEquals(1, actual[2], message);
  }

  @Test
  @Order(4)
  @DisplayName("Increments the first index if the brands's gender matches the parameter gender but name doesn't match other names => ")
  public void testCountByBrandsElse() {
    String message = "Check if the brand's gender matches the parameter gender and name doesn't equal the other names." + messageGap;
    int[] actual = testScene.countByBrands("Women");
    assertEquals(1, actual[3], message);
  }

}`}],dataFiles:[{path:`genders.txt`,text:`Boys
Girls
Girls
Girls
Girls
Boys
Boys
Girls
Men
Women
Men
Women
Women
Women
Women
Women
Men
Men
Men
Men
Men
Men
Men
Women
Women
Women
Women
Women
Boys
Girls
Boys
Girls
Boys
Girls
Girls
Boys
Men
Men
Men
Men
Women
Women
Women
Women
Men
Women
Women
Men
Women
Men
Men
Women
Unisex
Men
Men
Unisex
Unisex
Men
Women
Men
Men
Men
Men
Men
Men
Men
Men
Men
Men
Unisex
Unisex
Men`},{path:`names.txt`,text:`US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
US Polo
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
GAP
Kenneth Cole
Kenneth Cole
Kenneth Cole
Kenneth Cole
Kenneth Cole
Kenneth Cole
Kenneth Cole
Kenneth Cole
Kenneth Cole
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma
Puma`}]},{name:`Practice: Writing Algorithms (b)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Country objects from data in text files
    Country[] countries = Country.createCountries("names.txt", "groups.txt");

    // Creates an Internet object
    Internet scene = new Internet(countries);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getCountsByGroup() method then call the drawResults() method
     * with the results.
     * -----------------------------------------------------------------------------
     */


    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`Country.java`,text:`/*
 * Represents a country
 */
public class Country {

  private String name;        // The name of a country
  private String group;       // The income group of a country

  /*
   * Sets name and group to the specified values
   */
  public Country(String name, String group) {
    this.name = name;
    this.group = group;
  }

  /*
   * Returns the name of the country
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the income group of the country
   */
  public String getGroup() {
    return group;
  }

  /*
   * Returns a 1D array of Country objects using data from text files
   */
  public static Country[] createCountries(String namesFile, String groupsFile) {
    String[] namesData = FileReader.toStringArray(namesFile);
    String[] groupsData = FileReader.toStringArray(groupsFile);
    Country[] data = new Country[namesData.length];

    for (int index = 0; index < data.length; index++) {
      data[index] = new Country(namesData[index], groupsData[index]);
    }

    return data;
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
  
}`},{path:`Internet.java`,text:`import org.code.theater.*;

/*
 * Creates a visualization of Internet usage by income groups
 */
public class Internet extends Scene {

  private Country[] countries;     // The 1D array of Country objects

  /*
   * Initializes countries to the specified 1D array of Country objects
   */
  public Internet(Country[] countries) {
    this.countries = countries;
  }

  /*
   * Returns the 1D array of Country objects
   */
  public Country[] getCountries() {
    return countries;
  }

  /*
   * Returns a 1D array containing the number of countries that are classified
   * as high income at index 0, lower middle income at index 1, upper middle
   * income at index 2, and others at index 3
   */
  public int[] getCountsByGroup() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array countries. If the country's group equals "High income",
     * increment the value at index 0. Else if the country's group equals "Lower
     * middle income", increment the value at index 1. Else if the country's group
     * equals "Upper middle income", increment the value at index 2. Otherwise,
     * increment the value at index 3.
     * -----------------------------------------------------------------------------
     */

    

    return null;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults(int[] results) {
    String[] colors = {"red", "purple", "green", "orange", "blue"};

    int yLocation = 50;
    
    for (int index = 0; index < results.length; index++) {
      int randomIndex = (int)(Math.random() * colors.length);
      setFillColor(colors[randomIndex]);
      
      drawRectangle(0, yLocation, results[index] * 2, 25);
      pause(0.2);
      yLocation += 40;
    }
  }
  
}`}],validationFiles:[{path:`InternetTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Internet.java Test")
public class InternetTest {

  String messageGap = "\\n       ";
  Country[] testCountries;
  Internet testScene;
   
  @BeforeEach
  public void setup() {
    testCountries = new Country[]{new Country("Name1", "High income"), new Country("Name2", "Lower middle income"),
                                new Country("Name3", "Upper middle income"), new Country("Name4", "Low income")};
    
    testScene = new Internet(testCountries);
  }
   
  @Test
  @Order(1)
  @DisplayName("Increments the first index if the country's group is \\"High income\\" => ")
  public void testGetCountsByGroupHighIncome() {
    String message = "Check if the country's group is \\"High income\\" and increment index 0 if this is true." + messageGap;
    int[] actual = testScene.getCountsByGroup();
    assertEquals(1, actual[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("Increments the second index if the country's group is \\"Lower middle income\\" => ")
  public void testGetCountsByGroupLowerMiddle() {
    String message = "Check if the country's group is \\"Lower middle income\\" and increment index 1 if this is true." + messageGap;
    int[] actual = testScene.getCountsByGroup();
    assertEquals(1, actual[1], message);
  }

  @Test
  @Order(3)
  @DisplayName("Increments the third index if the country's group is \\"Upper middle income\\" => ")
  public void testGetCountsByGroupUpperMiddle() {
    String message = "Check if the country's group is \\"Upper middle income\\" and increment index 2 if this is true." + messageGap;
    int[] actual = testScene.getCountsByGroup();
    assertEquals(1, actual[2], message);
  }

  @Test
  @Order(4)
  @DisplayName("Increments the fourth index if the country's group doesn't match the other groups => ")
  public void testGetCountsByGroupElse() {
    String message = "Check if the country's group doesn't match the other groups and increment index 3 if this is true" + messageGap;
    int[] actual = testScene.getCountsByGroup();
    assertEquals(1, actual[3], message);
  }

}`}],dataFiles:[{path:`groups.txt`,text:`High income
Low income
Lower middle income
Upper middle income
High income
High income
Upper middle income
Upper middle income
Upper middle income
High income
High income
High income
Upper middle income
Low income
High income
Lower middle income
Low income
Lower middle income
Upper middle income
High income
High income
Upper middle income
Upper middle income
Upper middle income
High income
Lower middle income
Upper middle income
High income
High income
Lower middle income
Upper middle income
Low income
High income
High income
High income
High income
Upper middle income
Lower middle income
Lower middle income
Low income
Lower middle income
Upper middle income
Lower middle income
Lower middle income
Upper middle income
Upper middle income
High income
High income
High income
High income
High income
Lower middle income
Upper middle income
High income
Upper middle income
Lower middle income
Upper middle income
Lower middle income
Low income
High income
High income
Low income
High income
Upper middle income
High income
High income
Lower middle income
Upper middle income
High income
Upper middle income
Lower middle income
High income
Low income
Low income
Low income
Upper middle income
High income
Upper middle income
High income
Upper middle income
High income
Upper middle income
High income
Lower middle income
High income
Lower middle income
High income
Lower middle income
High income
Lower middle income
High income
Lower middle income
Upper middle income
High income
High income
High income
Upper middle income
Upper middle income
High income
Upper middle income
Lower middle income
Lower middle income
Lower middle income
Lower middle income
High income
High income
High income
Lower middle income
Lower middle income
Low income
Upper middle income
Upper middle income
High income
Lower middle income
Lower middle income
High income
High income
High income
High income
High income
Lower middle income
High income
Upper middle income
Low income
Upper middle income
Upper middle income
Upper middle income
Upper middle income
Low income
High income
Lower middle income
Upper middle income
Lower middle income
High income
Low income
Lower middle income
Upper middle income
Low income
Upper middle income
Upper middle income
High income
Low income
Lower middle income
Lower middle income
High income
High income
Lower middle income
High income
High income
High income
Lower middle income
High income
Upper middle income
Lower middle income
Upper middle income
Lower middle income
High income
High income
Low income
High income
Upper middle income
Lower middle income
High income
High income
High income
Upper middle income
Low income
High income
Low income
Lower middle income
High income
Lower middle income
Low income
Lower middle income
High income
Low income
Upper middle income
Low income
Lower middle income
Upper middle income
High income
High income
High income
Lower middle income
High income
High income
Low income
High income
Low income
Low income
Upper middle income
Lower middle income
Upper middle income
Lower middle income
Upper middle income
High income
Lower middle income
Upper middle income
Upper middle income
Lower middle income
Low income
Lower middle income
High income
High income
Lower middle income
Upper middle income
High income
High income
Lower middle income
Lower middle income
Lower middle income
Upper middle income
Low income
Upper middle income
Low income
Lower middle income`},{path:`names.txt`,text:`Aruba
Afghanistan
Angola
Albania
Andorra
United Arab Emirates
Argentina
Armenia
American Samoa
Antigua and Barbuda
Australia
Austria
Azerbaijan
Burundi
Belgium
Benin
Burkina Faso
Bangladesh
Bulgaria
Bahrain
Bahamas, The
Bosnia and Herzegovina
Belarus
Belize
Bermuda
Bolivia
Brazil
Barbados
Brunei Darussalam
Bhutan
Botswana
Central African Republic
Canada
Switzerland
Channel Islands
Chile
China
Côte d'Ivoire
Cameroon
Congo, Dem. Rep.
Congo, Rep.
Colombia
Comoros
Cabo Verde
Costa Rica
Cuba
Curaçao
Cayman Islands
Cyprus
Czechia
Germany
Djibouti
Dominica
Denmark
Dominican Republic
Algeria
Ecuador
Egypt, Arab Rep.
Eritrea
Spain
Estonia
Ethiopia
Finland
Fiji
France
Faroe Islands
Micronesia, Fed. Sts.
Gabon
United Kingdom
Georgia
Ghana
Gibraltar
Guinea
Gambia, The
Guinea-Bissau
Equatorial Guinea
Greece
Grenada
Greenland
Guatemala
Guam
Guyana
Hong Kong SAR, China
Honduras
Croatia
Haiti
Hungary
Indonesia
Isle of Man
India
Ireland
Iran, Islamic Rep.
Iraq
Iceland
Israel
Italy
Jamaica
Jordan
Japan
Kazakhstan
Kenya
Kyrgyz Republic
Cambodia
Kiribati
St. Kitts and Nevis
Korea, Rep.
Kuwait
Lao PDR
Lebanon
Liberia
Libya
St. Lucia
Liechtenstein
Sri Lanka
Lesotho
Lithuania
Luxembourg
Latvia
Macao SAR, China
St. Martin (French part)
Morocco
Monaco
Moldova
Madagascar
Maldives
Mexico
Marshall Islands
North Macedonia
Mali
Malta
Myanmar
Montenegro
Mongolia
Northern Mariana Islands
Mozambique
Mauritania
Mauritius
Malawi
Malaysia
Namibia
New Caledonia
Niger
Nigeria
Nicaragua
Netherlands
Norway
Nepal
Nauru
New Zealand
Oman
Pakistan
Panama
Peru
Philippines
Palau
Papua New Guinea
Poland
Puerto Rico
Korea, Dem. People's Rep.
Portugal
Paraguay
West Bank and Gaza
French Polynesia
Qatar
Romania
Russian Federation
Rwanda
Saudi Arabia
Sudan
Senegal
Singapore
Solomon Islands
Sierra Leone
El Salvador
San Marino
Somalia
Serbia
South Sudan
São Tomé and Principe
Suriname
Slovak Republic
Slovenia
Sweden
Eswatini
Sint Maarten (Dutch part)
Seychelles
Syrian Arab Republic
Turks and Caicos Islands
Chad
Togo
Thailand
Tajikistan
Turkmenistan
Timor-Leste
Tonga
Trinidad and Tobago
Tunisia
Türkiye
Tuvalu
Tanzania
Uganda
Ukraine
Uruguay
United States
Uzbekistan
St. Vincent and the Grenadines
British Virgin Islands
Virgin Islands (U.S.)
Vietnam
Vanuatu
Samoa
Kosovo
Yemen, Rep.
South Africa
Zambia
Zimbabwe`}]},{name:`Practice: Writing Algorithms (c)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Company object
    Company cardCompany = new Company();

    // Creates several Card objects
    int[] validCardNo = {1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 0};
    Card validCard = new Card(validCardNo);

    int[] invalidCardNo = {4, 6, 2, 4, 7, 4, 8, 2, 3, 3, 2, 4, 9, 0, 8, 0};
    Card invalidCard = new Card(invalidCardNo);

    // Calls the validateNumber() method on each Card object and prints the result
    System.out.println(validCard + " is valid? " + cardCompany.validateNumber(validCard));
    System.out.println(invalidCard + " is valid? " + cardCompany.validateNumber(invalidCard));
    
  }
}`},{path:`Card.java`,text:`/*
 * Represents a credit card
 */
public class Card {

  private int[] digits;      // The 1D array of digits that make up the card number

  /*
   * Initializes digits to the specified 1D array of digits
   */
  public Card(int[] digits) {
    this.digits = digits;
  }

  /*
   * Returns the 1D array of digits
   */
  public int[] getDigits() {
    return digits;
  }

  /*
   * Returns a String representing the credit card number
   */
  public String toString() {
    String result = "";

    for (int index = 0; index < digits.length; index++) {
      result += digits[index];

      if (index % 4 == 3) {
        result += " ";
      }
    }

    return result;
  }
  
}`},{path:`Company.java`,text:`/*
 * Represents a credit card company
 */
public class Company {

  /*
   * Returns true if the digits in creditCard are valid,
   * otherwise returns false
   */
  public boolean validateNumber(Card creditCard) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement the Luhn algorithm to check if the digits in creditCard are
     * valid. Return true if valid, otherwise return false.
     * -----------------------------------------------------------------------------
     */

    

    return false;
  }
  
}`}],validationFiles:[{path:`CompanyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Company.java Test")
public class CompanyTest {

  String messageGap = "\\n       ";
  Company testCompany;
   
  @BeforeEach
  public void setup() {
    testCompany = new Company();
  }
   
  @Test
  @Order(1)
  @DisplayName("validateNumber() returns true if the card number is valid => ")
  public void testValidateNumberReturnsTrue() {
    String message = "Be sure the correct code is executed under the correct conditions." + messageGap;
    int[] testDigits = {4, 5, 5, 6, 7, 3, 7, 5, 8, 6, 8, 9, 9, 8, 5, 5};
    Card testCard = new Card(testDigits);
    assertTrue(testCompany.validateNumber(testCard));
  }
   
  @Test
  @Order(2)
  @DisplayName("validateNumber() returns false if the card number is invalid => ")
  public void testValidateNumberReturnsFalse() {
    String message = "Be sure the correct code is executed under the correct conditions." + messageGap;
    int[] testDigits = {4, 0, 2, 4, 0, 0, 7, 1, 0, 9, 0, 2, 2, 1, 4, 3};
    Card testCard = new Card(testDigits);
    assertFalse(testCompany.validateNumber(testCard));
  }
  
}`}],dataFiles:[]},{name:`Practice: Writing Algorithms (d)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Lender object
    Lender auto = new Lender();

    // Creates several Customer objects
    Customer firstCustomer = new Customer(10000, 5, 4.5, 1000);
    Customer secondCustomer = new Customer(20000, 10, 0, 2000);
    Customer thirdCustomer = new Customer(15000, 3, 5.2, 15000);

    // Calls the calculateMonthlyPayment() method and prints the result
    double firstResult = auto.calculateMonthlyPayment(firstCustomer);
    System.out.println("First customer: " + auto.getResult(firstResult));

    double secondResult = auto.calculateMonthlyPayment(secondCustomer);
    System.out.println("Second customer: " + auto.getResult(secondResult));

    double thirdResult = auto.calculateMonthlyPayment(thirdCustomer);
    System.out.println("Third customer: " + auto.getResult(thirdResult));
    
  }
}`},{path:`Customer.java`,text:`/*
 * Represents a customer
 */
public class Customer {

  private double amount;        // The requested amount for the loan
  private int length;           // The requested length of the loan in years
  private double rate;          // The requested interest rate for the loan
  private double downPayment;   // The down payment a customer will make

  /*
   * Sets amount, length, rate, and downPayment to the specified values
   */
  public Customer(double amount, int length, double rate, double downPayment) {
    this.amount = amount;
    this.length = length;
    this.rate = rate;
    this.downPayment = downPayment;
  }

  /*
   * Returns the requested amount for the loan
   */
  public double getAmount() {
    return amount;
  }

  /*
   * Returns the requested length for the loan
   */
  public int getLength() {
    return length;
  }

  /*
   * Returns the requested interest rate for the loan
   */
  public double getRate() {
    return rate;
  }

  /*
   * Returns the down payment the customer will make
   */
  public double getDownPayment() {
    return downPayment;
  }
}`},{path:`Lender.java`,text:`/*
 * Represents an auto loan lender
 */
public class Lender {

  /*
   * Returns -1 if the customer's loan length is less than or equal to 0 OR the
   * interest rate is less than or equal to 0, returns 1 if the down payment is
   * greater than or equal to the loan amount, or returns the result of a call
   * to the getPaymentAmount() method
   */
  public double calculateMonthlyPayment(Customer customer) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use if/else-if/else statements to check each condition and return
     * the expected result.
     * -----------------------------------------------------------------------------
     */
    
    return -1;
  }

  /*
   * Calculates and returns the payment amount
   */
  public double getPaymentAmount(Customer customer) {
    int months = customer.getLength() * 12;
    double remainingBalance = customer.getAmount() - customer.getDownPayment();
    double monthlyBalance = remainingBalance / months;
    double interest = (monthlyBalance * customer.getRate()) / 100;
    double monthlyPayment = monthlyBalance + interest;
    return monthlyPayment;
  }

  /*
   * Returns "Error! You must take out a valid car loan." if
   * result is -1, "The car can be paid in full." if result is
   * 1, or "Your monthly payment will be {payment}."
   */
  public String getResult(double payment) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use if/else-if/else statements to check each condition and return
     * the expected result.
     * -----------------------------------------------------------------------------
     */
    
    return "";
  }
  
}`}],validationFiles:[{path:`LenderTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Lender.java Test")
public class LenderTest {

  String messageGap = "\\n       ";
  Lender testLender;
   
  @BeforeEach
  public void setup() {
    testLender = new Lender();
  }
   
  @Test
  @Order(1)
  @DisplayName("calculateMonthlyPayment() returns -1 if loan length is less than or equal to zero => ")
  public void testCalculateMonthlyPaymentLoanLength() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(10000, -1, 4.5, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    assertEquals(-1, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("calculateMonthlyPayment() returns -1 if interest rate is less than or equal to 0 => ")
  public void testCalculateMonthlyPaymentInterestRate() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(10000, 5, -1, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    assertEquals(-1, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("calculateMonthlyPayment() returns 1 if down payment is greater than or equal to loan amount => ")
  public void testCalculateMonthlyPaymentDownPayment() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(1000, 5, 4.5, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    assertEquals(1, actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("calculateMonthlyPayment() returns the result of a call to getPaymentAmount() => ")
  public void testCalculateMonthlyPaymentReturnPayment() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(10000, 5, 4.5, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    assertTrue(actual != 1 && actual != -1, message);
  }

  @Test
  @Order(5)
  @DisplayName("getResult() returns error message if payment equals -1 => ")
  public void testGetResultReturnsError() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(10000, -1, 4.5, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    String result = testLender.getResult(actual);
    assertTrue(result.contains("Error! You must take out a valid car loan"), message);
  }

  @Test
  @Order(6)
  @DisplayName("getResult() returns pay in full message if payment equals 1 => ")
  public void testGetResultReturnsPayInFull() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(1000, 5, 4.5, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    String result = testLender.getResult(actual);
    assertTrue(result.contains("The car can be paid in full"), message);
  }

  @Test
  @Order(7)
  @DisplayName("getResult() returns the monthly payment => ")
  public void testGetResultReturnsPayment() {
    String message = "Use if/else-if/else statements to check each condition." + messageGap;
    Customer testCustomer = new Customer(10000, 5, 4.5, 1000);
    double actual = testLender.calculateMonthlyPayment(testCustomer);
    String result = testLender.getResult(actual);
    assertTrue(result.contains("Your monthly payment will be"), message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Multi-Selection with The Theater (a)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of park names from a text file
    String[] names = FileReader.toStringArray("parks.txt");

    // Creates a 1D array containing the areas from a text file
    int[] acres = FileReader.toIntArray("acres.txt");

    // Creates a ParkScene object
    ParkScene scene = new ParkScene(names, acres);

    // Calls the drawResult() method
    scene.drawResult();

    // Plays the scene
    Theater.playScenes(scene);
    
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
  
}`},{path:`ParkScene.java`,text:`import org.code.theater.*;

/*
 * A scene visualizing the size of each national park
 */
public class ParkScene extends Scene {

  private String[] parks;     // The 1D array of national park names
  private int[] acres;        // The 1D array containing the areas of each park in acres

  /*
   * Initializes parks and acres to the specified 1D array of
   * park names and areas of each park in acres
   */
  public ParkScene(String[] parks, int[] acres) {
    this.parks = parks;
    this.acres = acres;
  }

  /*
   * Returns the size to draw each image
   */
  public int chooseSize(int acres) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ If the acres is greater than 1000000, return 100. Else if the acres is
     * greater than or equal to 100000, return 50. Otherwise, return 25.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Creates the scene by drawing the park image based on
   * the area of each park in acres
   */
  public void drawResult() {    
    for (int i = 0; i < acres.length; i++) {
      int x = (int) (Math.random() * getWidth());
      int y = (int) (Math.random() * getHeight());
      int size = chooseSize(acres[i]);
      drawImage("park.png", x, y, size);
    }
  }
  
}`}],validationFiles:[{path:`ParkSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ParkScene.java Test")
public class ParkSceneTest {

  String messageGap = "\\n       ";
  int acres;
  ParkScene testScene;
   
  @BeforeEach
  public void setup() {
    String[] testNames = new String[3];
    int[] testAcres = new int[3];
    testScene = new ParkScene(testNames, testAcres);
  }
   
  @Test
  @Order(1)
  @DisplayName("chooseSize() returns 100 when acres is greater than 1000000 => ")
  public void testChooseSizeReturns100() {
    String message = "Use an if statement to check if acres is greater than 1000000. Return 100 if this is true." + messageGap;
    acres = (int)(Math.random() * (1100000 - 1000000 + 1)) + 1000000;
    int actual = testScene.chooseSize(acres);
    assertEquals(100, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("chooseSize() returns 50 when acres is greater than or equal to 100000 => ")
  public void testChooseSizeReturns50() {
    String message = "Use an else if statement to check if acres is greater than or equal to 100000. Return 50 if this is true." + messageGap;
    acres = (int)(Math.random() * (110000 - 100000 + 1)) + 100000;
    int actual = testScene.chooseSize(acres);
    assertEquals(50, actual, message);
  }
  
  @Test
  @Order(3)
  @DisplayName("chooseSize() returns 25 when acres is NOT greater than 1000000 or greater than or equal to 100000 => ")
  public void testChooseSizeReturns25() {
    String message = "Use an else statement to return 25 if the if and else if conditions are false." + messageGap;
    acres = (int)(Math.random() * (1000 - 500 + 1)) + 500;
    int actual = testScene.chooseSize(acres);
    assertEquals(25, actual, message);
  }
}`}],dataFiles:[{path:`acres.txt`,text:`49077
8257
76679
242756
801163
172971
30780
35835
337598
241905
46766
249561
26476
183224
32572
3408407
4740911
64701
1508939
7523897
193
1013126
3223383
1201647
310044
77180
107342
522427
86367
33265
325605
5554
15349
571790
795156
3674529
669650
461901
1750716
2619816
106589
54012
52485
236382
504781
922649
221390
26686
138999
265807
91716
404063
199224
70447
15053
218222
146344
33971
8323146
2219791
761748
147243`},{path:`parks.txt`,text:`Acadia
American Samoa
Arches
Badlands
Big Bend
Biscayne
Black Canyon of the Gunnison
Bryce Canyon
Canyonlands
Capitol Reef
Carlsbad Caverns
Channel Islands
Congaree
Crater Lake
Cuyahoga Valley
Death Valley
Denali
Dry Tortugas
Everglades
Gates of the Arctic
Gateway Arch
Glacier
Glacier Bay
Grand Canyon
Grand Teton
Great Basin
Great Sand Dunes
Great Smoky Mountains
Guadalupe Mountains
Haleakala
Hawaiʻi Volcanoes
Hot Springs
Indiana Dunes
Isle Royale
Joshua Tree
Katmai
Kenai Fjords
Kings Canyon
Kobuk Valley
Lake Clark
Lassen Volcanic
Mammoth Cave
Mesa Verde
Mount Rainier
North Cascades
Olympic
Petrified Forest
Pinnacles
Redwood
Rocky Mountain
Saguaro
Sequoia
Shenandoah
Theodore Roosevelt
Virgin Islands
Voyageurs
White Sands
Wind Cave
Wrangell–St.\xA0Elias
Yellowstone
Yosemite
Zion`}]},{name:`Practice: Multi-Selection with The Theater (b)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of image file names from a text file
    String[] logos = FileReader.toStringArray("logos.txt");

    // Creates a 1D array containing the number of wins from a text file
    int[] wins = FileReader.toIntArray("wins.txt");

    // Creates a BasketballScene object
    BasketballScene scene = new BasketballScene(logos, wins);

    // Calls the drawResults() method
    scene.drawResults();

    // Plays the scene
    Theater.playScenes(scene);    
    
  }
}`},{path:`BasketballScene.java`,text:`import org.code.theater.*;

/*
 * A scene visualizing the number of championship wins
 * for each basketball team
 */
public class BasketballScene extends Scene {

  private String[] logos;             // The 1D array of basketball team logos
  private int[] championships;        // The 1D array of championship wins

  /*
   * Initializes logos and championships to the specified 1D arrays
   */
  public BasketballScene(String[] logos, int[] championships) {
    this.logos = logos;
    this.championships = championships;
  }

  /*
   * Return the size to draw the image based on
   * the number of championship wins
   */
  public int chooseSize(int numWins) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ If the number of wins is greater than 10, return 150. Else if the
     * number of wins is greater than or equal to 5, return 100. Otherwise,
     * return 50.
     * -----------------------------------------------------------------------------
     */

    

    return -1;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults() {    
    for (int i = 0; i < championships.length; i++) {
      int x = (int) (Math.random() * getWidth());
      int y = (int) (Math.random() * getHeight());
      int size = chooseSize(championships[i]);
      drawImage(logos[i], x, y, size);
    }
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
  
}`}],validationFiles:[{path:`BasketballSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BasketballScene.java Test")
public class BasketballSceneTest {

  String messageGap = "\\n       ";
  int numWins;
  BasketballScene testScene;
   
  @BeforeEach
  public void setup() {
    String[] testLogos = new String[3];
    int[] testWins = new int[3];
    testScene = new BasketballScene(testLogos, testWins);
  }
   
  @Test
  @Order(1)
  @DisplayName("chooseSize() returns 150 when numWins is greater than 10 => ")
  public void testChooseSizeReturns150() {
    String message = "Use an if statement to check if numWins is greater than 10. Return 150 if this is true." + messageGap;
    numWins = (int)(Math.random() * (20 - 15 + 1)) + 15;
    int actual = testScene.chooseSize(numWins);
    assertEquals(150, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("chooseSize() returns 100 when numWins is greater than or equal to 5 => ")
  public void testChooseSizeReturns100() {
    String message = "Use an else if statement to check if numWins is greater than or equal to 5. Return 100 if this is true." + messageGap;
    numWins = (int)(Math.random() * (10 - 5 + 1)) + 5;
    int actual = testScene.chooseSize(numWins);
    assertEquals(100, actual, message);
  }
  
  @Test
  @Order(3)
  @DisplayName("chooseSize() returns 50 when numWins is NOT greater than 10 or greater than or equal to 5 => ")
  public void testChooseSizeReturns50() {
    String message = "Use an else statement to return 50 if the if and else if conditions are false." + messageGap;
    numWins = (int)(Math.random() * (4 - 1 + 1)) + 1;
    int actual = testScene.chooseSize(numWins);
    assertEquals(50, actual, message);
  }
}`}],dataFiles:[{path:`logos.txt`,text:`bostonceltics.png
brooklynnets.png
newyorkknicks.png
philadelphia76ers.png
torontoraptors.png
chicagobulls.png
clevelandcavaliers.png
detroitpistons.png
indianapacers.png
milwaukeebucks.png
atlantahawks.png
charlottehornets.png
miamiheat.png
orlandomagic.png
washingtonwizards.png
denvernuggets.png
minnesotatimberwolves.png
okcthunder.png
portlandtrailblazers.png
utahjazz.png
goldenstatewarriors.png
losangelesclippers.png
lalakers.png
phoenixsuns.png
sacramentokings.png
dallasmavericks.png
houstonrockets.png
memphisgrizzlies.png
neworleanspelicans.png
sanantoniospurs.png`},{path:`wins.txt`,text:`17
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
5`}]},{name:`Practice: Multi-Selection with The Theater (c)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of planet image files from a text file
    String[] planets = FileReader.toStringArray("planets.txt");

    // Creates a 1D array containing the length of days from a text file
    int[] days = FileReader.toIntArray("days.txt");

    // Creates a PlanetScene object
    PlanetScene scene = new PlanetScene(planets, days);

    // Calls the drawResults() method
    scene.drawResults();

    // Plays the scene
    Theater.playScenes(scene);
    
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
  
}`},{path:`PlanetScene.java`,text:`import org.code.theater.*;

/*
 * A scene visualizing the length of a day for each planet
 */
public class PlanetScene extends Scene {

  private String[] planets;     // The 1D array of planet images
  private int[] dayLengths;     // The 1D array of the length of days for each planet

  /*
   * Initializes planets and dayLengths to the specified 1D arrays
   */
  public PlanetScene(String[] planets, int[] dayLengths) {
    this.planets = planets;
    this.dayLengths = dayLengths;
  }

  /*
   * Returns the size to draw the image based on the length of the day
   */
  public int chooseSize(int dayLength) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ If dayLength is greater than 100, return 150. Else if dayLength is
     * greater than or equal to 15, return 100. Otherwise, return 50.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return -1;
  }

  /*
   * Draws the results in the scene
   */
  public void drawResults() {
    for (int i = 0; i < dayLengths.length; i++) {
      int x = (int) (Math.random() * getWidth());
      int y = (int) (Math.random() * getHeight());
      int size = chooseSize(dayLengths[i]);
      drawImage(planets[i], x, y, size);
    }
  }
  
}`}],validationFiles:[{path:`PlanetSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PlanetScene.java Test")
public class PlanetSceneTest {

  String messageGap = "\\n       ";
  int dayLength;
  PlanetScene testScene;
   
  @BeforeEach
  public void setup() {
    String[] testImages = new String[3];
    int[] testDays = new int[3];
    testScene = new PlanetScene(testImages, testDays);
  }
   
  @Test
  @Order(1)
  @DisplayName("chooseSize() returns 150 when dayLength is greater than 100 => ")
  public void testChooseSizeReturns150() {
    String message = "Use an if statement to check if dayLength is greater than 100. Return 150 if this is true." + messageGap;
    dayLength = (int)(Math.random() * (200 - 150 + 1)) + 150;
    int actual = testScene.chooseSize(dayLength);
    assertEquals(150, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("chooseSize() returns 100 when dayLength is greater than or equal to 15 => ")
  public void testChooseSizeReturns100() {
    String message = "Use an else if statement to check if dayLength is greater than or equal to 15. Return 100 if this is true." + messageGap;
    dayLength = (int)(Math.random() * (50 - 15 + 1)) + 15;
    int actual = testScene.chooseSize(dayLength);
    assertEquals(100, actual, message);
  }
  
  @Test
  @Order(3)
  @DisplayName("chooseSize() returns 50 when dayLength is NOT greater than 100 or greater than or equal to 15 => ")
  public void testChooseSizeReturns50() {
    String message = "Use an else statement to return 50 if the if and else if conditions are false." + messageGap;
    dayLength = (int)(Math.random() * (10 - 5 + 1)) + 5;
    int actual = testScene.chooseSize(dayLength);
    assertEquals(50, actual, message);
  }
}`}],dataFiles:[{path:`days.txt`,text:`4222
2802
24
24
9
10
17
16
153`},{path:`planets.txt`,text:`mercury.png
venus.png
earth.png
mars.png
jupiter.png
saturn.png
uranus.png
neptune.png
pluto.png`}]},{name:`Practice: Multi-Selection with The Theater (d)`,lesson:`Lesson 10: Multi-Selection Statements`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of cat breeds
    String[] breeds = FileReader.toStringArray("breeds.txt");

    // Creates a 1D array of country names
    String[] countries = FileReader.toStringArray("countries.txt");

    // Creates a CatScene object
    CatScene scene = new CatScene(breeds, countries);

    // Calls the analyzeCountries() method
    int[] results = scene.analyzeCountries();

    // Calls the drawResults() method
    scene.drawResults(results);

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`CatScene.java`,text:`import org.code.theater.*;

/*
 * A scene visualizing the number of cat
 * breeds that originated in each country
 */
public class CatScene extends Scene {

  private String[] breeds;     // The 1D array of cat breeds
  private String[] countries;  // The 1D array of originating countries

  /*
   * Initializes breeds and countries to the specified 1D arrays
   */
  public CatScene(String[] breeds, String[] countries) {
    this.breeds = breeds;
    this.countries = countries;
  }

  /*
   * Returns an array containing the number
   * of cat breeds for each country
   */
  public int[] analyzeCountries() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the countries array to count the number of breeds that
     * originated in each country.
     * -----------------------------------------------------------------------------
     */


    return null;
  }

  /*
   * Creates the scene by drawing each country flag at a random location
   * and the size based on the number of originating cat breeds
   */
  public void drawResults(int[] counts) {
    String[] countryFlags = {"egypt.png", "russia.png", "thailand.png", "uk.png", "usa.png"};

    for (int index = 0; index < countryFlags.length; index++) {
      int randomX = (int)(Math.random() * 200);
      int randomY = (int)(Math.random() * 200);

      drawImage(countryFlags[index], randomX, randomY, counts[index] * 5);
    }
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
  
}`}],validationFiles:[{path:`CatSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CatScene.java Test")
public class CatSceneTest {

  String messageGap = "\\n       ";
  CatScene testScene;
   
  @BeforeEach
  public void setup() {
    String[] testBreeds = new String[3];
    String[] testCountries = {"Egypt", "Russia", "Thailand", "United Kingdom", "United States"};
    testScene = new CatScene(testBreeds, testCountries);
  }
   
  @Test
  @Order(1)
  @DisplayName("analyzeCountries() counts the number of countries equal to \\"Egypt\\" => ")
  public void testAnalyzeCountriesCountsEgypt() {
    String message = "Use an if statement to check if a country equals \\"Egypt\\". Increment the value at index 0 if this is true." + messageGap;
    int[] actual = testScene.analyzeCountries();
    assertEquals(1, actual[0], message);
  }

  @Test
  @Order(2)
  @DisplayName("analyzeCountries() counts the number of countries equal to \\"Russia\\" => ")
  public void testAnalyzeCountriesCountsRussia() {
    String message = "Use an if statement to check if a country equals \\"Russia\\". Increment the value at index 1 if this is true." + messageGap;
    int[] actual = testScene.analyzeCountries();
    assertEquals(1, actual[1], message);
  }

  @Test
  @Order(3)
  @DisplayName("analyzeCountries() counts the number of countries equal to \\"Thailand\\" => ")
  public void testAnalyzeCountriesCountsThailand() {
    String message = "Use an if statement to check if a country equals \\"Thailand\\". Increment the value at index 2 if this is true." + messageGap;
    int[] actual = testScene.analyzeCountries();
    assertEquals(1, actual[2], message);
  }

  @Test
  @Order(4)
  @DisplayName("analyzeCountries() counts the number of countries equal to \\"United Kingdom\\" => ")
  public void testAnalyzeCountriesCountsUnitedKingdom() {
    String message = "Use an if statement to check if a country equals \\"United Kingdom\\". Increment the value at index 3 if this is true." + messageGap;
    int[] actual = testScene.analyzeCountries();
    assertEquals(1, actual[3], message);
  }

  @Test
  @Order(5)
  @DisplayName("analyzeCountries() counts the number of countries equal to \\"United States\\" => ")
  public void testAnalyzeCountriesCountsUnitedStates() {
    String message = "Use an if statement to check if a country equals \\"United States\\". Increment the value at index 4 if this is true." + messageGap;
    int[] actual = testScene.analyzeCountries();
    assertEquals(1, actual[4], message);
  }
   
}`}],dataFiles:[{path:`breeds.txt`,text:`Abyssinian
American Bobtail
American Curl
American Shorthair
American Wirehair
Balinese
Bambino
Bengal
Bombay
British Longhair
British Shorthair
Burmilla
California Spangled
Chantilly-Tiffany
Chausie
Cheetoh
Colorpoint Shorthair
Cornish Rex
Devon Rex
Donskoy
Egyptian Mau
Exotic Shorthair
Havana Brown
Himalayan
Javanese
Khao Manee
Korat
Kurilian
LaPerm
Maine Coon
Malayan
Munchkin
Nebelung
Ocicat
Oriental
Pixie-bob
Ragamuffin
Ragdoll
Russian Blue
Savannah
Scottish Fold
Selkirk Rex
Siamese
Siberian
Snowshoe
Toyger
York Chocolate`},{path:`countries.txt`,text:`Egypt
United States
United States
United States
United States
United States
United States
United States
United States
United Kingdom
United Kingdom
United Kingdom
United States
United States
Egypt
United States
United States
United Kingdom
United Kingdom
Russia
Egypt
United States
United Kingdom
United States
United States
Thailand
Thailand
Russia
Thailand
United States
United Kingdom
United States
United States
United States
United States
United States
United States
United States
Russia
United States
United Kingdom
United States
Thailand
Russia
United States
United States
United States`}]},{name:`Abstract Data Art Project`,lesson:`Lesson 11a: Abstract Data Art Project`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {


    
    
  }
}`},{path:`DataScene.java`,text:`import org.code.theater.*;

public class DataScene extends Scene {





  
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
  
}`}],validationFiles:[],dataFiles:[{path:`data1.txt`,text:``},{path:`data2.txt`,text:``}]},{name:`Abstract Data Art Project`,lesson:`Lesson 11b: Abstract Data Art Project [1-Day Version]`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {


    
    
  }
}`},{path:`DataScene.java`,text:`import org.code.theater.*;

public class DataScene extends Scene {





  
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
  
}`}],validationFiles:[],dataFiles:[{path:`data1.txt`,text:``},{path:`data2.txt`,text:``}]},{name:`CSA 2023 Console Sandbox_2025`,lesson:`Sandbox: Console`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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