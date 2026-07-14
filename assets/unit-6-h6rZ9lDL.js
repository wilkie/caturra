var e=[{name:`Creative Coding with The Theater Example Projects (a)`,lesson:`Lesson 1: Project Planning`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    Can[] cans = { new BrandCan("Coke", "coke.png"), new BrandCan("Dr.Pepper", "drPepper.png"), new GenericCan()};

    RecycleScene myScene = new RecycleScene(cans);

    myScene.createScene();

    Theater.playScenes(myScene);

  }
}`},{path:`BrandCan.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class BrandCan extends Can {

  private String brand;

  public BrandCan(String brand, String image) {
    super(image);
    this.brand = brand;
  }
  
  public String getBrand() {
    return brand;
  }
  
}`},{path:`Can.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Can {

  private String image;

  public Can(String image) {
    this.image = image;
  }
  
  public String getImage() {
    return image;
  }

  public String getBrand() {
    return "";
  }

  public int getY() {
    return 150;
  }
  
  public int getSize() {
    return 190;
  }
  
}`},{path:`GenericCan.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class GenericCan extends Can {

  public GenericCan() {
    super("cans.png");
  }
  
  public String getBrand() {
    return "generic";
  }

  public int getY() {
    return 250;
  }

  public int getSize() {
    return 19;
  }
  
}`},{path:`RecycleScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class RecycleScene extends Scene {

  private Can[] cans;
  private int qty;
  private double value;

  public RecycleScene(Can[] cans) {
    this.cans = cans;
    this.qty = 0;
    this.value = 0.0;
  }

  public void drawCans() {
    for (int index = 0; index < cans.length; index++) {
      for (int x = 0; x < 300; x += 20) {
        drawImage(cans[index].getImage(), x, cans[index].getY(), cans[index].getSize(), 90); 
        drawBackgroundQTYandSum();
      } 
      
      qty++;
      value += 0.10;
      
      drawBackgroundQTYandSum(); 
      playSound("coinSound.wav"); 
    }     
  }

  public void drawReceipt() {
    for (int h = 0; h < 100; h += 10) {
      for (int y = 320; y < 360; y += 10) {
        drawReceiptPaper(h);
        drawReceiptText(y);
      }
    }
  }
  
  public void drawBackground() {
    pause(0.2);
    
    drawImage("Machine.png", 0, 0, 400);
    
    drawRectangles();
  
    drawImage("cans.png", 250, 247, 13);
    
    setTextHeight(9);
    drawText("Sum:", 290, 255);
    drawText("x", 270, 270);
    drawText("Can-Bottle   $0.10", 240, 225);
  }
  
  public void createScene() {
    pause(1);
    drawCans();
    drawReceipt();
  }

  private void drawRectangles() {
    int[][] rectangleValues = { {232, 210, 95, 80}, {236, 215, 85, 15}, {236, 240, 85, 40} };

    for (int row = 0; row < rectangleValues.length; row++) {

      if (row == 0) {
        setFillColor("white");
      }
      else {
        removeStrokeColor();
        setFillColor("gray");
      }
      
      int[] currentValues = rectangleValues[row];
      drawRectangle(currentValues[0], currentValues[1], currentValues[2], currentValues[3]);
    }
  }

  private void drawReceiptText(int y) {
    for (int index = 0; index < cans.length; index++) {
      setTextHeight(8);
      drawText("" + cans[index].getBrand(), 295, y);
      y += 20;
    }
  }

  private void drawReceiptPaper(int height) {
    setFillColor("white");
    drawRectangle(290, 305, 40, height);
    height += 20;
  }

  private void showSum() {
    drawText(String.format("%.2f", value), 300, 270);
  }
  
  private void showQuantity() {
    drawText("" + qty, 278, 270);
  }

  private void drawBackgroundQTYandSum() {
    drawBackground();
    showQuantity();
    showSum();
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with The Theater Example Projects (b)`,lesson:`Lesson 1: Project Planning`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    MovingSoundCharacter[] characters = {
        new MovingSoundCharacter("spongebobSound.wav", "frontSpongebob.png", "backSpongebob.png"), 
        new MovingSoundCharacter("patrickSound.wav", "frontPatrick.png", "backPatrick.png"),
        new MovingSoundCharacter("sandySound.wav", "frontSandy.png", "backSandy.png"),
        new MovingSoundCharacter("garySound.wav", "frontGary.png", "backGary.png"),
        new MovingSoundCharacter("purpleSnailSound.wav","purpleSnail.png", "purpleSnail.png") };

    Seat[] seats = { new Seat(80, -20, 260), new Seat(80, 67, 260), new Seat(85, 20, 250),
                     new Seat(120, 337, 265), new Seat(100, 210, 250) };

    SoundCharacter mrKrabs = new SoundCharacter("mrKrabsSound.wav", "MrKrabs.png");
    OpeningScene opening = new OpeningScene(mrKrabs, "krustyKrabs.png");
    opening.createScene();
    
    SoundCharacter squidWard = new SoundCharacter("squidwardSound.wav", "squidWard1.png");
    KrustyKrabsScene mainScene = new KrustyKrabsScene(characters, seats, "krustyKrabs.jpg", squidWard);
    mainScene.createScene();

    Theater.playScenes(opening, mainScene);

  }
}`},{path:`Character.java`,text:`public class Character {
  private String image;

  public Character(String image) {
    this.image = image;
  }
  
  public String getImage() {
    return image;
  }
  
}`},{path:`KrustyKrabsScene.java`,text:`import org.code.theater.*;
import org.code.media.*;
import java.util.ArrayList;

public class KrustyKrabsScene extends Scene {

  private MovingSoundCharacter[] characters;
  private Seat[] seats;
  private String theInside;
  private SoundCharacter server;
  private ArrayList<MovingSoundCharacter> seatedCharacters;

  public KrustyKrabsScene(MovingSoundCharacter[] characters, Seat[] seats, String theInside, SoundCharacter server) {
    this.characters = characters;
    this.seats = seats;
    this.theInside = theInside;
    this.server = server;
    
    this.seatedCharacters = new ArrayList<MovingSoundCharacter>();
  }
  
  public void createScene() {
    for(int index = 0; index < characters.length; index++) {
      MovingSoundCharacter currentCharacter = characters[index];
      showCharacter(currentCharacter);
      seatedCharacters.add(characters[index]);
      pause(1);
      drawBackground(); 
    }
    
    playSound(server.getSoundFile());
  }
  
  public void drawSeatedCharacters() {
    for (int index = 0; index < seatedCharacters.size(); index++) {
      MovingSoundCharacter currentCharacter = seatedCharacters.get(index);
      drawImage(currentCharacter.getImage(), seats[index].getSeatXPos(), seats[index].getSeatYPos(), seats[index].getImageSize());
    }  
  }
  
  public void drawBackground() {
    clear("black");
    drawImage(theInside, -40, 0, 500);
    drawImage(server.getImage(), 240, 202, 130);
    drawSeatedCharacters();
  }

  private void showCharacter(MovingSoundCharacter currentCharacter) {
    playSound(currentCharacter.getSoundFile());
    
    for (int x = 90; x < 230; x += 40) {
        drawBackground();
        drawImage(currentCharacter.getImage(true), x, 235, 80);   
        pause(0.2);
    }
  }
  
}`},{path:`MovingSoundCharacter.java`,text:`public class MovingSoundCharacter extends SoundCharacter {
  private String backImage;

  public MovingSoundCharacter(String soundFile, String image, String backImage) {
    super(soundFile, image);
    this.backImage = backImage;
  }
  
  public String getImage(boolean isBackImage) {
    if (isBackImage) {
      return backImage;
    }
    return getImage();
  }
  
}`},{path:`OpeningScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class OpeningScene extends Scene {

  private SoundCharacter owner;
  private String krustyKrabs;

  public OpeningScene(SoundCharacter owner, String krustyKrabs) {
    this.owner = owner;
    this.krustyKrabs = krustyKrabs;
  }

  public void drawOneBubble(int numBubbles) {
    if (numBubbles == 0) { 
      playSound("bubblesSound.wav");
      return;
    }
    
    drawRandomBubble();
    drawOneBubble(numBubbles - 1);
  }

  public void drawMrKrabs() {
    drawOwner();
    drawSpeech();
    playSound(owner.getSoundFile()); 
  }

  public void createScene() {
    drawImage(krustyKrabs, 0, 0, 400);
    drawMrKrabs();
    drawOneBubble(25);
    pause(2);
  }

  private void drawRandomBubble() {
    int x = (int)(Math.random() * 400);    // x position
    int y = (int)(Math.random() * 200);    // y position
    int z = (int)(Math.random() * 60);     // size
    
    drawImage("1bubble.png", x + 10, y - 20, z);
  }

  private void drawOwner() {
    drawImage(owner.getImage(), 170, 245, 200);
    pause(0.25);
  }

  private void drawSpeech() {
    drawImage("screamSpeechBubble.png", 80, 130, 270);
    setTextHeight(8);
    drawText("Hello! I like money", 185, 250);
  }
  
}`},{path:`Seat.java`,text:`public class Seat {

  private int imageSize;
  private int seatXPos;
  private int seatYPos;
  
  public Seat(int imageSize, int seatXPos, int seatYPos) {
    this.imageSize = imageSize;
    this.seatXPos = seatXPos;
    this.seatYPos = seatYPos;
  }
  
  public int getImageSize() {
    return imageSize;
  }
  
  public int getSeatXPos() {
    return seatXPos;
  }
  
  public int getSeatYPos() {
    return seatYPos;
  }
  
}`},{path:`SoundCharacter.java`,text:`public class SoundCharacter extends Character {
  private String soundFile;

  public SoundCharacter(String soundFile, String image) {
    super(image);
    this.soundFile = soundFile;
  }
  
  public String getSoundFile() {
    return soundFile;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with The Theater Example Projects (c)`,lesson:`Lesson 1: Project Planning`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    MenuItem bananaSmoothie = new MenuItem("Banana Smoothie", "bSmoothie.png", null, 6, "drink");
    MenuItem strawberrySmoothie = new MenuItem("Strawberry Smoothie", "sbSmoothie.png", null, 6, "drink");
    MenuItem bananas = new MenuItem("Bananas", "threeBananas.png", null, 3, "food");

    String[] sandwichIngredients = {"plate.png", "bread.png", "lettuce.png", "cheese.png", "tomatoes.png", "bread.png"};
    MenuItem sandwich = new MenuItem("Sandwich", "sandwhich.png", sandwichIngredients, 9, "food");

    MenuItem[] fullMenu = {bananaSmoothie, strawberrySmoothie, bananas, sandwich};

    Minion[] minions = { new Minion("minion2.png", bananas), new Minion("minions3.png", bananaSmoothie),
                          new Minion("minions4.png", sandwich), new Minion("minions5.png", strawberrySmoothie) };

    MinionScene cafe = new MinionScene(minions, "gru.png", fullMenu);
    Kitchen theKitchen = new Kitchen(fullMenu, "kitchen.jpg", "backOfGru.png");
    ServeFood serveFood = new ServeFood(fullMenu, "gru.png");
   
    cafe.createScene();
    theKitchen.createScene();
    serveFood.createScene();

    Theater.playScenes(cafe, theKitchen, serveFood);

  }
}`},{path:`BananaCafe.java`,text:`import org.code.theater.*;
import org.code.media.*;
import java.util.ArrayList;

public class BananaCafe extends Scene {

  private MenuItem[] fullMenu;
  private String server;
  public static final Color cafeColor = new Color(185, 136, 92);

  public BananaCafe(MenuItem[] fullMenu, String server) {
    this.fullMenu = fullMenu;
    this.server = server;
  }

  public MenuItem[] getFullMenu() {
    return fullMenu;
  }

  public String getServer() {
    return server;
  }
  
  public void drawBar() {
    setFillColor("black");
    drawRectangle(100, 50, 50, 100);
    drawRectangle(150, 100, 150, 50);
  }
  
  public void drawFloors() {
    for (int z = 300; z > -500; z -= 100) {
      drawFloorTile(z, 0);
    }    
  }
  
  public void drawTables() {
    setFillColor(cafeColor);
    
    for (int x = 15; x < 320; x += 150) {
      drawEllipse(x, 210, 75, 75);
    }
    
    for (int y = 90; y < 245; y += 150) {
      drawEllipse(y, 360, 75, 75);
    }  
  }
  
  public void drawWall() {
    setFillColor("white");
    
    drawRectangle(0, 0, 400, 50);
    
    // shelf
    drawLine(75, 25, 325, 25);
    drawImage("fruit_basket.png", 55, -40, 100);
    drawImage("fruit_basket.png", 95, -40, 100);
    drawImage("door.png", 170, -10, 60);
    drawImage("fruit_basket.png", 215, -40, 100);
    drawImage("fruit_basket.png", 255, -40, 100);
  }
  
  public void drawFloorTile(int x, int y) {
    if (y == getHeight()) {
      return;
    }
    
    setFillColor("yellow");
    drawRectangle(x, y, 50, 50);
    setFillColor("white");
    drawRectangle(x + 50, y, 50, 50);
    
    drawFloorTile(x + 50, y + 50);
  }
  
  public void drawThreeChair(int x, int y) {
    if (x < 0) {
      return;
    }
    
    int size = 30;
    setFillColor(cafeColor);
    drawEllipse(x, y, size, size);
    
    drawThreeChair(x - 150, y);
  }
  
  public void drawAllChairs() {
    int x = 300;
    int y = 190;
    
    drawThreeChair(x, y);
    drawThreeChair(x + 72, y);
    drawThreeChair(x, y + 100);
    drawThreeChair(x + 72, y + 100);
    drawThreeChair(x + 35, y + 200);
  }
  
  public void drawMenuPoster(int x_rectangle) {
    setStrokeWidth(3);
    setFillColor("brown");
    drawRectangle(x_rectangle, 0, 65, 40);
    setStrokeWidth(0);
  }
  
  public void drawDrinksMenu() {
    drawMenuPoster(10);
    setTextHeight(8);
    setTextColor("black");
    drawText("Drinks Menu:", 15, 10);
    drawMenuText("drink");
  }
  
  public void drawFoodMenu() {
    drawMenuPoster(330);
    setTextColor("black");
    drawText("Food Menu:", 335, 10);
    drawMenuText("food");
  }

  public void drawMenuText(String type) {
    int yPos = 20;
    ArrayList<MenuItem> items = getMenuItemsByType(type);

    for (int index = 0; index < items.size(); index++) {
      MenuItem currentItem = items.get(index);
      
      setTextHeight(8);
      drawText(currentItem.getItemName(), 15, yPos);

      if (type.equals("food") && currentItem.getIngredients() != null) {
        setTextHeight(6);
        yPos = drawMenuIngredients(currentItem, yPos);
      }
      
      yPos += 10;
    }
  }

  public int drawMenuIngredients(MenuItem currentItem, int yPos) {
    String[] ingredients = currentItem.getIngredients();

    for (int index = 0; index < ingredients.length; index++) {
      yPos += 5;
      drawText(ingredients[index], 335, yPos);
    }

    return yPos;
  }

  public ArrayList<MenuItem> getMenuItemsByType(String type) {
    ArrayList<MenuItem> items = new ArrayList<MenuItem>();

    for (int index = 0; index < fullMenu.length; index++) {
      String currentItemType = fullMenu[index].getType();

      if (currentItemType.equals(type)) {
        items.add(fullMenu[index]);
      }
    }

    return items;
  }
  
  public void drawBananaTree() {
    drawImage("bananaTree.png", -30, 25, 150);
    drawImage("bananaTree.png", 290, 25, 150);
  }

  public void displayMinions() {
    int[][] sizeAndPosition = { {210, 165, 40}, {140, 165, 50}, {140, 240, 55}, {210, 250, 60} };
    drawImage(getServer(), 180, 130, 40);

    Minion[] minions = MinionScene.getMinions();

    for (int index = 0; index < minions.length; index++) {
      drawImage(minions[index].getImage(), sizeAndPosition[index][0], sizeAndPosition[index][1], sizeAndPosition[index][2]);
    }
  }
  
  public void drawBackground() {
    drawFloors();
    drawBar();
    drawWall();
    drawTables();
    drawDrinksMenu();
    drawFoodMenu();
    drawBananaTree();
    drawAllChairs();
    displayMinions();
  }
  
  public void createScene() {
    drawBackground();
  }
  
}`},{path:`Kitchen.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Kitchen extends Scene {

  private MenuItem[] fullMenu;
  private String kitchenImage;
  private String chefImage;

  public Kitchen(MenuItem[] fullMenu, String kitchenImage, String chefImage) {
    this.fullMenu = fullMenu;
    this.kitchenImage = kitchenImage;
    this.chefImage = chefImage;
  }

  public void createScene() {
    drawBackground();
    makeSandwich();
  }
 
  public void drawBackground() {
    drawImage(kitchenImage, 0, 0, getWidth());
    drawImage(chefImage, 50, 150, 110, 10);
    pause(1);
  }

  public void makeSandwich() {
    String[] sandwichIngredients = getSandwichIngredients();
    
    for (int index = 0; index < sandwichIngredients.length; index++) {
      drawImage(sandwichIngredients[index], 160, 210, 100);
      pause(0.5);
    }
  }
  
  private String[] getSandwichIngredients() {
    MenuItem sandwichOrder = getSandwich();

    if (sandwichOrder != null) {
      return sandwichOrder.getIngredients();
    }

    return null;
  }

  private MenuItem getSandwich() {
    for (int index = 0; index < fullMenu.length; index++) {
      MenuItem currentItem = fullMenu[index];
      if (currentItem.getItemName().equals("Sandwich")) {
        return fullMenu[index];
      }
    }

    return null;
  }
  
}`},{path:`MenuItem.java`,text:`public class MenuItem {

  private String itemName;
  private String image;
  private String[] ingredients;
  private int price;
  private String type;

  public MenuItem(String itemName, String image, String[] ingredients, int price, String type) {
    this.itemName = itemName;
    this.image = image;
    this.ingredients = ingredients;
    this.price = price;
    this.type = type;
  }

  public String getItemName() {
    return itemName;
  }

  public String getImage() {
    return image;
  }

  public String[] getIngredients() {
    return ingredients;
  }

  public int getPrice() {
    return price;
  }

  public String getType() {
    return type;
  }

  public static int calculateTotal(Minion[] minionsOrder) {
    int total = 0;

    for (int index = 0; index < minionsOrder.length; index++) {
      MenuItem currentOrder = minionsOrder[index].getOrder();
      total += currentOrder.getPrice();
    }

    return total;
  }
  
}`},{path:`Minion.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Minion {
  
  private String image;
  private MenuItem order;

  public Minion(String image, MenuItem order) {
    this.image = image;
    this.order = order;
  }
  
  public MenuItem getOrder() {
    return order;
  }
  
  public String getImage() {
    return image;
  }
  
}`},{path:`MinionScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class MinionScene extends BananaCafe {

  private static Minion[] minions;

  public MinionScene(Minion[] minions, String server, MenuItem[] fullMenu) {
    super(fullMenu, server);
    this.minions = minions;
  }

  public static Minion[] getMinions() {
    return minions;
  }

  public void createScene() {
    super.createScene();
    displayGru();
    placeOrder(90, 190, 120);
    pause(1);
  }

  public void displayGru() {
    drawImage("RightSpeechBubble.png", 73, 25, 180);
    setTextHeight(7);
    drawText("What can I get for you?", 125, 110);
  }
  
  public void placeOrder(int y, int x, int size) {
    for (int index = 0; index < minions.length; index++) {
      formatText();
      showSpeechBubble(index, x, y, size);
      showOrder(index);
      showSubtitles(220, minions[index].getOrder().getItemName());
    }
  }

  public void showSpeechBubble(int currentOrder, int x, int y, int size) {
    int[][] speechBubblePosition = { {x, y}, {x, y * 2}, {x - 110, y}, {x - 110, y * 2} };
    String speechBubble = getSpeechBubble(currentOrder);
    drawImage(speechBubble, speechBubblePosition[currentOrder][0], speechBubblePosition[currentOrder][1], size);
  }

  public void showOrder(int currentOrder) {
    String[][] orderText = { {"Ba-na-na!", null}, {"Strewberryy", "smooooti"}, {"Ba-na-na", "smooooti"}, {"ssaannwich", null} };
    int[] orderPosition = getOrderPosition(currentOrder);

    for (int index = 0; index < orderPosition.length; index++) {
      drawText(orderText[currentOrder][0], orderPosition[0], orderPosition[1]);
    
      if (orderText[currentOrder][1] != null) {
        drawText(orderText[currentOrder][1], orderPosition[2], orderPosition[3]);
      }
    }
  }

  public void showSubtitles(int length, String orderName) {
    setFillColor("black");
    drawRectangle(130, 335, length, 20);
    setTextHeight(15);
    setTextColor("white");
    drawText("subtitles: " + orderName, 150, 350);
  }

  private int[] getOrderPosition(int currentOrder) {
    int[][] orderPosition = { {230, 145, 0, 0}, {230, 230, 230, 240}, {120, 145, 120, 155}, {120, 230, 0, 0} };
    return orderPosition[currentOrder];
  }

  private String getSpeechBubble(int row) {
    String speechBubble = "";
    
    if (row == 0 || row == 1) {
      speechBubble = "LeftSpeechBubble.png";
    }
    else {
      speechBubble = "RightSpeechBubble.png";
    }

    return speechBubble;
  }

  private void formatText() {
    pause(1);
    setTextColor("black");
    setTextHeight(7);
  }
  
}`},{path:`ServeFood.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class ServeFood extends BananaCafe {

  public ServeFood(MenuItem[] fullMenu, String server) {
    super(fullMenu, server);
  }

  public void createScene() {
    super.createScene();
    drawOrders();
    displayBill();
    pause(1);
  }

  public void drawOrders() {
    int[][] sizeAndPosition = { {200, 210, 30}, {150, 180, 70}, {15, 250, 70}, {180, 220, 70} };
    Minion[] minions = MinionScene.getMinions();

    for (int index = 0; index < minions.length; index++) {
      MenuItem minionOrder = minions[index].getOrder();
      int[] formatValues = sizeAndPosition[index];
      drawImage(minionOrder.getImage(), formatValues[0], formatValues[1], formatValues[2]);
    }
  }
  
  private void displayBill() {
    drawImage("RightSpeechBubble.png", 73, 25, 180);
    setTextHeight(7);
    int total = MenuItem.calculateTotal(MinionScene.getMinions());
    drawText("You owe me $ " + total, 135, 110);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with The Theater Example Projects (d)`,lesson:`Lesson 1: Project Planning`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {

    String[] dogImages = {"sitting.png", "scratching.png", "lookingBack.png"};
    double[] barking = SoundLoader.read("barking.wav");

    Dog theDog = new Dog("running.png", dogImages, barking, 180, 200, GardenScene.size);
    Butterfly theButterfly = new Butterfly("butterflyImage.png", 280, 10, 45, GardenScene.size);

    GardenScene garden = new GardenScene(theDog, theButterfly, "garden.jpg");
    garden.createScene();
    
    Theater.playScenes(garden);

  }
}`},{path:`Animal.java`,text:`public class Animal {

  private String image;
  private int xPos;
  private int yPos;
  private int size;
  
  public Animal(String image, int xPos, int yPos, int size) {
    this.image = image;
    this.xPos = xPos;
    this.yPos = yPos;
    this.size = size;
  }
  
  public String getImage() {
    return image;
  }
  
  public int getXPos() {
    return xPos;
  }
  
  public int getYPos() {
    return yPos;
  }
  
  public void setXPos(int xPos) {
    this.xPos = xPos;
  }

  public void setYPos(int yPos) {
    this.yPos = yPos;
  }

  public int getSize() {
    return size;
  }

  public void updatePosition(int xPos, int yPos) {
    setXPos(xPos);
    setYPos(yPos);
  }
}`},{path:`Butterfly.java`,text:`public class Butterfly extends Animal {

  private int rotation;
  
  public Butterfly(String image, int xPos, int yPos, int rotation, int size) {
    super(image, xPos, yPos, size);
    this.rotation = rotation;
  }
  
  public int getRotation() {
    return rotation;
  }
  
  public void setRotation(int rotation) {
    this.rotation = rotation;
  }

  public void updatePosition(int xPos, int yPos, int getRotation) {
    updatePosition(xPos, yPos);
    setRotation(rotation);
  }
}`},{path:`Dog.java`,text:`public class Dog extends Animal {

  private String[] dogImages;
  private double[] barkingSound;
  
  public Dog(String image, String[] dogImages, double[] barkingSound, int xPos, int yPos, int size) {
    super(image, xPos, yPos, size);
    this.dogImages = dogImages;
    this.barkingSound = barkingSound;
  }
  
  public String[] getDogImages() {
    return dogImages;
  }

  public double[] getBarkingSound() {
    return barkingSound;
  }

  public String getImage(String position) {
    String currentMovement = dogImages[0];
    
    for (int index = 0; index < dogImages.length; index++) {
      if (dogImages[index].indexOf(position) >= 0) {
        currentMovement = dogImages[index];
      }
    }

    return currentMovement;
  }

}`},{path:`GardenScene.java`,text:`import org.code.theater.*;
import org.code.media.*;
import java.util.ArrayList;

public class GardenScene extends Scene {
  
  private Dog theDog;
  private Butterfly theButterfly;
  private String backgroundImage;
  public static final int size = 400;

  public GardenScene(Dog theDog, Butterfly theButterfly, String backgroundImage) {
    this.theDog = theDog;
    this.theButterfly = theButterfly;
    this.backgroundImage = backgroundImage;
  }
  
  public void drawBackground() {
    drawImage(backgroundImage, 0, 0, 420);
  }

  public void drawDogSittingAndScratching() {
    drawDogSitting();
    drawBackground();
    drawDogScratching();
    drawBackground();
    drawDogSitting();
  }
   
  public void chaseButterfly(int count, double increment) {
    if (count == 0) { 
      return;
    }
    
    drawBackground();
    drawImage(theDog.getImage(), theDog.getXPos(), theDog.getYPos(), theDog.getSize());
    drawImage(theButterfly.getImage(), theButterfly.getXPos(), theButterfly.getYPos(), theButterfly.getSize(), theButterfly.getRotation());
    playBarking(increment);
    pause(0.5);
    
    theButterfly.updatePosition(theButterfly.getXPos() - 20, theButterfly.getYPos() - 10, theButterfly.getRotation() + 4);
    theDog.setXPos(theDog.getXPos() - 20);
    chaseButterfly(count - 1, increment * 2);
  }
  
  public void createScene() {
    drawBackground();
    drawDogSittingAndScratching();
    chaseButterfly(16, 0);
    drawBackground();
    drawDogLookingBack();
  }

  private void drawDogSitting() {
    String dogSit = theDog.getImage("sitting");
    drawImage(dogSit, theDog.getXPos(), theDog.getYPos(), theDog.getSize());
    pause(0.5);
  }

  private void drawDogScratching() {
    String dogScratch = theDog.getImage("scratching");
    drawImage(dogScratch, theDog.getXPos(), theDog.getYPos(), theDog.getSize());
    pause(0.5);
  }
  
  private void drawDogLookingBack() {
    String dogLookBack = theDog.getImage("lookingBack");
    drawImage(dogLookBack, -80, theDog.getYPos(), theDog.getSize());
    pause(0.5);
  }

  private void playBarking(double increment) {
    double[] newSound = SoundEffects.increaseVolume(theDog.getBarkingSound(), increment);
    playSound(newSound);
  }
  
}`},{path:`SoundEffects.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class SoundEffects {

  public static double[] increaseVolume(double[] sound, double amount) {
    double[] louderBark = new double[sound.length];
    
    for (int i = 0; i < sound.length; i++) {
      louderBark[i] = sound[i] * amount;
    }
    
    return louderBark;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Object References as Parameters`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`public class Course {

  private String name;
  private boolean status;

  public Course(String name, boolean status) {
    this.name = name;
    this.status = status;
  }

  public void setStatus(boolean newStatus) {
    status = newStatus;
  }

  public String toString() {
    String text = name + ": ";

    if (status) {
      text = text + "Enrolled";
    }
    else {
      text = text + "Dropped";
    }

    return text;
  }
  
}`},{path:`Student.java`,text:`public class Student {

  private String name;
  private Course newCourse;

  public Student(String name, Course newCourse) {
    this.name = name;
    this.newCourse = newCourse;
  }

  public void dropCourse(Course theCourse) {
    theCourse.setStatus(false);
  }

  public String toString() {
    return name + "\\n" + newCourse;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Object References as Parameters #1`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`public class Course {

  private String name;
  private boolean status;

  public Course(String name, boolean status) {
    this.name = name;
    this.status = status;
  }

  public void setStatus(boolean newStatus) {
    status = newStatus;
  }

  public String toString() {
    String text = name + ": ";

    if (status) {
      text = text + "Enrolled";
    }
    else {
      text = text + "Dropped";
    }

    return text;
  }
  
}`},{path:`Student.java`,text:`public class Student {

  private String name;
  private Course newCourse;

  public Student(String name, Course newCourse) {
    this.name = name;
    this.newCourse = newCourse;
  }

  public void dropCourse(Course theCourse) {
    theCourse.setStatus(false);
  }

  public String toString() {
    return name + "\\n" + newCourse;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Object References as Parameters #2`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`public class Course {

  private String name;
  private boolean status;

  public Course(String name, boolean status) {
    this.name = name;
    this.status = status;
  }

  public void setStatus(boolean newStatus) {
    status = newStatus;
  }

  public String toString() {
    String text = name + ": ";

    if (status) {
      text = text + "Enrolled";
    }
    else {
      text = text + "Dropped";
    }

    return text;
  }
  
}`},{path:`Student.java`,text:`public class Student {

  private String name;
  private Course newCourse;

  public Student(String name, Course newCourse) {
    this.name = name;
    this.newCourse = newCourse;
  }

  public void dropCourse(Course theCourse) {
    theCourse.setStatus(false);
  }

  public String toString() {
    return name + "\\n" + newCourse;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Object References as Parameters #3`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Course.java`,text:`public class Course {

  private String name;
  private boolean status;

  public Course(String name, boolean status) {
    this.name = name;
    this.status = status;
  }

  public void setStatus(boolean newStatus) {
    status = newStatus;
  }

  public String toString() {
    String text = name + ": ";

    if (status) {
      text = text + "Enrolled";
    }
    else {
      text = text + "Dropped";
    }

    return text;
  }
  
}`},{path:`Student.java`,text:`public class Student {

  private String name;
  private Course newCourse;

  public Student(String name, Course newCourse) {
    this.name = name;
    this.newCourse = newCourse;
  }

  public void dropCourse(Course theCourse) {
    theCourse.setStatus(false);
  }

  public String toString() {
    return name + "\\n" + newCourse;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Object References as Parameters (a) #1`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates several ClubMember objects
    ClubMember naomi = new ClubMember("Naomi", 10);
    ClubMember dominic = new ClubMember("Dominic", 15);
    ClubMember ariana = new ClubMember("Ariana", 12);

    // Creates a 1D array of ClubMember objects
    ClubMember[] members = {naomi, dominic, ariana};

    // Creates a ClubSponsor object
    ClubSponsor sponsor = new ClubSponsor("Ms. Easley", members);

    // Prints the ClubSponsor object
    System.out.println(sponsor);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the addServiceHours() method, then print the updated ClubSponsor object.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`ClubMember.java`,text:`/*
 * Represents a member of a school club
 */
public class ClubMember {

  private String name;        // The name of a club member
  private int serviceHours;   // The number of community service hours

  /*
   * Sets name and serviceHours to the specified values
   */
  public ClubMember(String name, int serviceHours) {
    this.name = name;
    this.serviceHours = serviceHours;
  }

  /*
   * Returns the name of the club member
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the number of community service hours
   */
  public int getServiceHours() {
    return serviceHours;
  }

  /*
   * Updates the number of community service hours
   * to the newServiceHours
   */
  public void setServiceHours(int newServiceHours) {
    serviceHours = newServiceHours;
  }

  /*
   * Returns a String containing information about the club member
   */
  public String toString() {
    return name + ": " + serviceHours + " community service hours";
  }
  
}`},{path:`ClubSponsor.java`,text:`/*
 * Represents a sponsor of a school club
 */
public class ClubSponsor {

  private String sponsor;         // The name of a club sponsor
  private ClubMember[] members;   // The 1D array containing the club members

  /*
   * Sets sponsor to the specified name of a club sponsor and
   * initializes members to the specified 1D array of members
   */
  public ClubSponsor(String sponsor, ClubMember[] members) {
    this.sponsor = sponsor;
    this.members = members;
  }

  /*
   * Returns the name of the club sponsor
   */
  public String getSponsor() {
    return sponsor;
  }

  /*
   * Returns the 1D array of club members
   */
  public ClubMember[] getMembers() {
    return members;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the addServiceHours() method to add a specified number of hours to
   * a club member. The method should have a parameter for a ClubMember object and
   * the number of hours to add, and it should add the hours to the ClubMember
   * object's current service hours.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns a String containing information about the club sponsor
   */
  public String toString() {
    String text = "Club Sponsor: " + sponsor + "\\n";

    for (ClubMember student : members) {
      text = text + student + "\\n";
    }

    return text;
  }
  
}`}],validationFiles:[{path:`ClubSponsorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Parameter;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ClubSponsor.java Test")
public class ClubSponsorTest {

  String messageGap = "\\n       ";
  ClubMember[] testMembers;
  ClubSponsor testObject;
  Class testClass;
   
  @BeforeEach
  public void setup() {
    testMembers = new ClubMember[]{new ClubMember("Naomi", 10), new ClubMember("Dominic", 15), new ClubMember("Ariana", 12)};
    testObject = new ClubSponsor("some sponsor", testMembers);
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("addServiceHours() has a parameter for a ClubMember object => ")
  public void testAddServiceHoursClubMemberParameter() {
    String message = "Inside the parentheses for the method signature, add a ClubMember parameter for a ClubMember object." + messageGap;
      
    Method actualMethod = getExpectedMethod("addServiceHours");
    String actualParameter = getExpectedParameter(actualMethod, "ClubMember");

    assertEquals("ClubMember", actualParameter, message);
  }

  @Test
  @Order(2)
  @DisplayName("addServiceHours() has a parameter for the number of hours to add => ")
  public void testAddServiceHoursNumHoursParameter() {
    String message = "Inside the parentheses for the method signature, add an int parameter for the number of hours." + messageGap;
      
    Method actualMethod = getExpectedMethod("addServiceHours");
    String actualParameter = getExpectedParameter(actualMethod, "int");

    assertEquals("int", actualParameter, message);
  }

  @Test
  @Order(3)
  @DisplayName("addServiceHours() has a void return type => ")
  public void testAddServiceHoursReturnType() {
    String message = "The method does not return a value, so it should have a void return type in the method signature." + messageGap;
      
    Method actualMethod = getExpectedMethod("addServiceHours");
    String actualReturn = getExpectedReturnType(actualMethod, "void");

    assertEquals("void", actualReturn, message);
  }

  @Test
  @Order(4)
  @DisplayName("addServiceHours() adds the specified number of hours to the ClubMember object's service hours => ")
  public void testAddServiceHoursUpdatesServiceHours() {
    String message = "Call the getServiceHours() method in the ClubMember class to get the current service hours. Add the";
    message += "\\n        parameter for the number of hours to the current service hours and update the ClubMember object's";
    message += "\\n        serviceHours instance variable with the new value using the setServiceHours() method." + messageGap;
      
    int randomIndex = (int)(Math.random() * testMembers.length);
    int randomHours = (int)(Math.random() * 20) + 5;
    ClubMember randomMember = testObject.getMembers()[randomIndex];
  
    int expected = randomMember.getServiceHours() + randomHours;
    Method actualMethod = getExpectedMethod("addServiceHours");
  
    if (actualMethod != null) {
      try {
        actualMethod.invoke(testObject, randomMember, randomHours); // FIXED
      } catch (Exception e) {
        fail("Exception occurred while invoking addServiceHours: " + e.getMessage());
      }
    }
  
    int actual = randomMember.getServiceHours();
    assertEquals(expected, actual, message);
  }


  private Method getExpectedMethod(String methodName) {
    Method[] classMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : classMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }

  private String getExpectedParameter(Method method, String parameterName) {
    Class<?>[] parameters = null;
    String actualParameter = "";
    
    if (method != null) {
      parameters = method.getParameterTypes();
    }

    if (parameters != null) {
      for (Class param : parameters) {
        if (param.getName().equals(parameterName)) {
          actualParameter = parameterName;
        }
      }
    }

    return actualParameter;
  }

  private String getExpectedReturnType(Method method, String typeName) {
    Class<?> returnType = null;
    String actualReturnType = "";

    if (method != null) {
      returnType = method.getReturnType();
    }

    if (returnType != null && returnType.getName().equals(typeName)) {
      actualReturnType = typeName;
    }

    return actualReturnType;
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (b) #1`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Creates several Song objects
    Song firstSong = new Song("Let It Go", "Disney");
    Song secondSong = new Song("Happy", "Pharrell Williams");
    Song thirdSong = new Song("Count On Me", "Bruno Mars");
    Song fourthSong = new Song("Hakuna Matata", "Disney");
    Song fifthSong = new Song("Can't Stop the Feeling", "Justin Timberlake");

    // Creates a User object
    User natalia = new User();

    // Adds songs to the user's listening history
    natalia.addSong(firstSong);
    natalia.addSong(secondSong);
    natalia.addSong(thirdSong);
    natalia.addSong(fourthSong);
    natalia.addSong(fifthSong);

    // Likes some of the songs
    natalia.likeSong(secondSong);
    natalia.likeSong(thirdSong);
    natalia.likeSong(fifthSong);

    // Creates a Playlist object
    Playlist nataliaPlaylist = new Playlist();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calls the createPlaylist() method and prints the results using the
     * playlistToString() method.
     * -----------------------------------------------------------------------------
     */

    


    
    
  }
}`},{path:`Playlist.java`,text:`import java.util.ArrayList;

/*
 * Represents a playlist on a music streaming app
 */
public class Playlist {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createPlaylist() method that takes a User object as a parameter
   * and returns an ArrayList containing the songs the user liked in their
   * listening history.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns a String containing each song in the playlist
   */
  public String playlistToString(ArrayList<Song> playlist) {
    String result = "";

    for (Song song : playlist) {
      result += song + "\\n";
    }

    return result;
  }
  
}`},{path:`Song.java`,text:`/*
 * Represents a song on a music streaming app
 */
public class Song {

  private String title;      // The title of a song
  private String artist;     // The name of the artist of a song
  private boolean isLiked;   // Whether or not a user liked a song

  /*
   * Sets title and artist to the specified values
   * and isLiked to false
   */
  public Song(String title, String artist) {
    this.title = title;
    this.artist = artist;
    this.isLiked = false;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the artist of the song
   */
  public String getArtist() {
    return artist;
  }

  /*
   * Returns true if a user liked the song, otherwise false
   */
  public boolean getIsLiked() {
    return isLiked;
  }

  /*
   * Sets isLiked to newStatus
   */
  public void setIsLiked(boolean newStatus) {
    this.isLiked = newStatus;
  }

  /*
   * Returns a String containing the song's information
   */
  public String toString() {
    String result = artist + " by " + artist;

    if (isLiked) {
      result += " (Liked)";
    }
    else {
      result += " (Not Liked)";
    }

    return result;
  }
  
}`},{path:`User.java`,text:`import java.util.ArrayList;

/*
 * Represents a user on a music streaming app
 */
public class User {

  private ArrayList<Song> listeningHistory;    // The list of songs a user has listened to

  /*
   * Initializes listeningHistory to an empty list
   */
  public User() {
    this.listeningHistory = new ArrayList<Song>();
  }

  /*
   * Returns the user's listening history
   */
  public ArrayList<Song> getListeningHistory() {
    return listeningHistory;
  }

  /*
   * Adds a song to the user's listening history
   */
  public void addSong(Song newSong) {
    listeningHistory.add(newSong);
  }

  /*
   * Sets the Song status to true if the user liked the song
   */
  public void likeSong(Song song) {
    song.setIsLiked(true);
  }

  /*
   * Returns a String containing each song in the user's listening history
   */
  public String toString() {
    String result = "";

    for (Song item : listeningHistory) {
      result += item + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`PlaylistTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;

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
  User testUser;
  Playlist testObject;
  Class testClass;
   
  @BeforeEach
  public void setup() {
    testUser = new User();
    testObject = new Playlist();
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("createPlaylist() has a parameter for a User object => ")
  public void testCreatePlaylistUserParameter() {
    String message = "Inside the parentheses for the method signature, add a User parameter for a User object." + messageGap;
      
    Method actualMethod = getExpectedMethod("createPlaylist");
    String actualParameter = getExpectedParameter(actualMethod, "User");

    assertEquals("User", actualParameter, message);
  }

  @Test
  @Order(2)
  @DisplayName("createPlaylist() has a return type for an ArrayList<Song> => ")
  public void testCreatePlaylistReturnsArrayList() {
    String message = "The method does not return a value, so it should have return type for an ArrayList in the method signature." + messageGap;
      
    Method actualMethod = getExpectedMethod("createPlaylist");
    String actualReturn = getExpectedReturnType(actualMethod, "java.util.ArrayList");

    assertEquals("java.util.ArrayList", actualReturn, message);
  }

  @Test
  @Order(3)
  @DisplayName("createPlaylist() returns an ArrayList of Song objects that a user liked => ")
  public void testCreatePlaylistReturnsLikedSongs() {
    String message = "Call the createPlaylist() method in the Playlist class to get the user's listening";
    message += "\\n        history and traverse their list of songs. If they song isLiked, add the song to a";
    message += "\\n        new ArrayList containing the songs they liked and return the ArrayList." + messageGap;

    addSongsToUsers();
    ArrayList<Song> expectedList = getExpectedList();
    Method actualMethod = getExpectedMethod("createPlaylist");
    Object actualList = null;

    if (actualMethod != null) {
      try {
        actualList = actualMethod.invoke(testObject, testUser);
      } catch (Exception e) { }
    }

    assertEquals(expectedList, actualList, message);
  }

  private void addSongsToUsers() {
    Song firstSong = new Song("Let It Go", "Disney");
    Song secondSong = new Song("Happy", "Pharrell Williams");
    Song thirdSong = new Song("Count On Me", "Bruno Mars");
    Song fourthSong = new Song("Hakuna Matata", "Disney");
    Song fifthSong = new Song("Can't Stop the Feeling", "Justin Timberlake");

    testUser.addSong(firstSong);
    testUser.addSong(secondSong);
    testUser.addSong(thirdSong);
    testUser.addSong(fourthSong);
    testUser.addSong(fifthSong);
  }

  private ArrayList<Song> getExpectedList() {
    ArrayList<Song> userSongs = testUser.getListeningHistory();
    ArrayList<Song> playlist = new ArrayList<Song>();

    for (int count = 0; count < 3; count++) {
      int randomIndex = (int)(Math.random() * userSongs.size());
      Song currentSong = userSongs.get(randomIndex);
      currentSong.setIsLiked(true);
    }

    for (Song song : userSongs) {
      if (song.getIsLiked()) {
        playlist.add(song);
      }
    }

    return playlist;
  }

  private Method getExpectedMethod(String methodName) {
    Method[] classMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : classMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }

  private String getExpectedParameter(Method method, String parameterName) {
    Class<?>[] parameters = null;
    String actualParameter = "";
    
    if (method != null) {
      parameters = method.getParameterTypes();
    }

    if (parameters != null) {
      for (Class param : parameters) {
        if (param.getName().equals(parameterName)) {
          actualParameter = parameterName;
        }
      }
    }

    return actualParameter;
  }

  private String getExpectedReturnType(Method method, String typeName) {
    Class<?> returnType = null;
    String actualReturnType = "";

    if (method != null) {
      returnType = method.getReturnType();
    }

    if (returnType != null && returnType.getName().equals(typeName)) {
      actualReturnType = typeName;
    }

    return actualReturnType;
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (c) #1`,lesson:`Lesson 2: Object References as Parameters`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a MessageScene object
    MessageScene scrollingMessage = new MessageScene("background.jpg");

    // Creates Message objects
    Message firstMessage = new Message("Big journeys begin with small steps", 0, 100);
    Message secondMessage = new Message("Mindset is everything", 50, 150);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawScene() method with the Message objects.
     * -----------------------------------------------------------------------------
     */




    

    // Plays the scene
    Theater.playScenes(scrollingMessage);

  }
}`},{path:`Message.java`,text:`/*
 * Represents a message that is displayed in a scene
 */
public class Message {

  private String text;   // The text in a message
  private int xPos;      // The starting x position of a message
  private int yPos;      // The starting y position of a message

  /*
   * Sets the text of a message to the specified text,
   * and the x and y positions to the specified values
   */
  public Message(String text, int xPos, int yPos) {
    this.text = text;
    this.xPos = xPos;
    this.yPos = yPos;
  }

  /*
   * Returns the text in the message
   */
  public String getText() {
    return text;
  }

  /*
   * Returns the x position of the message
   */
  public int getXPos() {
    return xPos;
  }

  /*
   * Returns the y position of the message
   */
  public int getYPos() {
    return yPos;
  }

  /*
   * Changes the x position of the message to newXPos
   */
  public void setXPos(int newXPos) {
    xPos = newXPos;
  }

  /*
   * Changes the y position of the message to newYPos
   */
  public void setYPos(int newYPos) {
    xPos = newYPos;
  }
  
}`},{path:`MessageScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a scene with a message that appears
 * to scroll across the scene
 */
public class MessageScene extends Scene {

  private String backgroundImage;    // The filename for the background image

  /*
   * Sets backgroundImage to the specified filename for the background image
   */
  public MessageScene(String backgroundImage) {
    this.backgroundImage = backgroundImage;
  }

  /*
   * Draws the background image in the scene
   */
  public void drawBackground() {
    drawImage(backgroundImage, 0, 0, getWidth());
  }

  /*
   * Formats the text in the scene
   */
  public void formatText() {
    setTextStyle(Font.SANS, FontStyle.BOLD);
    setTextColor("white");
    setTextHeight(20);
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ While the x position of theMessage is less than the width of the scene,
   * draw the background and format the text using the provided methods. Increase
   * the x position of theMessage by some amount (like 10), then draw theMessage
   * text at its x and y positions. Be sure to add a pause to create the animation.
   * -----------------------------------------------------------------------------
   */




  
  
}`}],validationFiles:[{path:`MessageSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MessageScene.java Test")
public class MessageSceneTest {

  String messageGap = "\\n       ";
  MessageScene testObject;
  Class testClass;
   
  @BeforeEach
  public void setup() {
    testObject = new MessageScene("background.jpg");
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("drawScene() has a parameter for a Message object => ")
  public void testDrawSceneMessageParameter() {
    String message = "Inside the parentheses for the method signature, add a Message parameter for a Message object." + messageGap;
      
    Method actualMethod = getExpectedMethod("drawScene");
    String actualParameter = getExpectedParameter(actualMethod, "Message");

    assertEquals("Message", actualParameter, message);
  }

  @Test
  @Order(2)
  @DisplayName("drawScene() has a void return type => ")
  public void testDrawSceneReturnType() {
    String message = "The method does not return a value, so it should have a void return type in the method signature." + messageGap;
      
    Method actualMethod = getExpectedMethod("drawScene");
    String actualReturn = getExpectedReturnType(actualMethod, "void");

    assertEquals("void", actualReturn, message);
  }

  @Test
  @Order(3)
  @DisplayName("drawScene() moves the text of the Message object across the scene => ")
  public void testDrawSceneMovesText() {
    String message = "Increase the x position of the Message object to move it across the scene." + messageGap;

    Method actualMethod = getExpectedMethod("drawScene");
    Message testMessage = new Message("Big journeys begin with small steps", 0, 100);

    if (actualMethod != null) {
      try {
        actualMethod.invoke(testObject, testMessage);
      } catch (Exception e) { }
    }
    
    int actual = testMessage.getXPos();

    assertTrue(actual >= 400, message);
  }

  private Method getExpectedMethod(String methodName) {
    Method[] classMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : classMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }

  private String getExpectedParameter(Method method, String parameterName) {
    Class<?>[] parameters = null;
    String actualParameter = "";
    
    if (method != null) {
      parameters = method.getParameterTypes();
    }

    if (parameters != null) {
      for (Class param : parameters) {
        if (param.getName().equals(parameterName)) {
          actualParameter = parameterName;
        }
      }
    }

    return actualParameter;
  }

  private String getExpectedReturnType(Method method, String typeName) {
    Class<?> returnType = null;
    String actualReturnType = "";

    if (method != null) {
      returnType = method.getReturnType();
    }

    if (returnType != null && returnType.getName().equals(typeName)) {
      actualReturnType = typeName;
    }

    return actualReturnType;
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (d) #1`,lesson:`Lesson 2: Object References as Parameters`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates several Mandala objects
    Mandala firstMandala = new Mandala("mandala1.png", 0);
    Mandala secondMandala = new Mandala("mandala2.png", 0);
    Mandala thirdMandala = new Mandala("mandala3.png", 0);
    Mandala fourthMandala = new Mandala("mandala4.png", 0);
    Mandala fifthMandala = new Mandala("mandala5.png", 0);

    // Creates a 1D array of Mandala objects
    Mandala[] theMandalas = {firstMandala, secondMandala, thirdMandala, fourthMandala};

    // Creates a MandalaScene object
    MandalaScene scene = new MandalaScene("background.jpg");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawMandalas() method with theMandalas array.
     * -----------------------------------------------------------------------------
     */



    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`Mandala.java`,text:`/*
 * Represents an image of a mandala
 */
public class Mandala {

  private String imageFile;   // The filename of a mandala image
  private int rotation;       // The amount of rotation of an image

  /*
   * Sets the imageFile to the specified image file and
   * rotation to the specified starting rotation
   */
  public Mandala(String imageFile, int rotation) {
    this.imageFile = imageFile;
    this.rotation = rotation;
  }

  /*
   * Returns the image file of the mandala
   */
  public String getImage() {
    return imageFile;
  }

  /*
   * Returns the amount the mandala is rotated
   */
  public int getRotation() {
    return rotation;
  }

  /*
   * Sets the amount the mandala is rotated to newRotation
   */
  public void setRotation(int newRotation) {
    rotation = newRotation;
  }
  
}`},{path:`MandalaScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that draws images of mandalas
 */
public class MandalaScene extends Scene {

  private String backgroundImage;   // The filename for the background image

  /*
   * Sets backgroundImage to the specified file name for the background image
   */
  public MandalaScene(String backgroundImage) {
    this.backgroundImage = backgroundImage;
  }

  /*
   * Draws each Mandala object in mandalaList in the scene
   */
  public void drawMandalas(Mandala[] mandalaList) {
    for (Mandala theMandala : mandalaList) {
      drawScene(theMandala);
    }
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the drawScene() method with a parameter for a Mandala object.
   * While the rotation of theMandala is less than or equal to 360, draw the
   * backgroundImage, draw theMandala object's image with its current rotation,
   * and increase the rotation by some amount (like 50). Add a pause to animate
   * the drawing and rotation.
   * -----------------------------------------------------------------------------
   */


  
  
}`}],validationFiles:[{path:`MandalaSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.theater.*;
import java.lang.reflect.Method;
import java.lang.reflect.Parameter;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("MandalaScene.java Test")
public class MandalaSceneTest {

  String messageGap = "\\n       ";
  MandalaScene testObject;
  Class testClass;
   
  @BeforeEach
  public void setup() {
    testObject = new MandalaScene("background.jpg");
    testClass = testObject.getClass();
  }
   
  @Test
  @Order(1)
  @DisplayName("drawScene() has a parameter for a Mandala object => ")
  public void testDrawSceneMandalaParameter() {
    String message = "Inside the parentheses for the method signature, add a Mandala parameter for a Mandala object." + messageGap;
      
    Method actualMethod = getExpectedMethod("drawScene");
    String actualParameter = getExpectedParameter(actualMethod, "Mandala");

    assertEquals("Mandala", actualParameter, message);
  }

  @Test
  @Order(2)
  @DisplayName("drawScene() has a void return type => ")
  public void testDrawSceneReturnType() {
    String message = "The method does not return a value, so it should have a void return type in the method signature." + messageGap;
      
    Method actualMethod = getExpectedMethod("drawScene");
    String actualReturn = getExpectedReturnType(actualMethod, "void");

    assertEquals("void", actualReturn, message);
  }

  @Test
  @Order(3)
  @DisplayName("drawScene() increases the rotation of the Mandala object around the scene => ")
  public void testDrawSceneIncreasesRotation() {
    String message = "Increase the rotation of the Mandala object to move it across the scene." + messageGap;

    Method actualMethod = getExpectedMethod("drawScene");
    Mandala testMandala = new Mandala("mandala1.png", 0);

    if (actualMethod != null) {
      try {
        actualMethod.invoke(testObject, testMandala);
      } catch (Exception e) { }
    }
    
    int actual = testMandala.getRotation();

    assertTrue(actual >= 360, message);
  }

  private Method getExpectedMethod(String methodName) {
    Method[] classMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : classMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }

  private String getExpectedParameter(Method method, String parameterName) {
    Class<?>[] parameters = null;
    String actualParameter = "";
    
    if (method != null) {
      parameters = method.getParameterTypes();
    }

    if (parameters != null) {
      for (Class param : parameters) {
        if (param.getName().equals(parameterName)) {
          actualParameter = parameterName;
        }
      }
    }

    return actualParameter;
  }

  private String getExpectedReturnType(Method method, String typeName) {
    Class<?> returnType = null;
    String actualReturnType = "";

    if (method != null) {
      returnType = method.getReturnType();
    }

    if (returnType != null && returnType.getName().equals(typeName)) {
      actualReturnType = typeName;
    }

    return actualReturnType;
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (a) #2`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Reservation object
    Reservation vacation = new Reservation("March 15", 4);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Guest object and print the object. Then call either the
     * changeCheckIn() or changeNumNights() methods and print the updated Guest
     * object. Print the Reservation object to confirm it was not modified.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Guest.java`,text:`/*
 * Represents a guest at a hotel
 */
public class Guest {

  private String name;                // The name of a guest
  private Reservation reservation;    // The reservation for a guest

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the Guest constructor with a parameter for the name of a guest and a
   * parameter for a Reservation object. Instantiate the reservation object to a new
   * Reservation object using the values assigned to the parameter's instance variables.
   * -----------------------------------------------------------------------------
   */


  

  /*
   * Returns the name of the guest
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the reservation
   */
  public Reservation getReservation() {
    return reservation;
  }

  /*
   * Changes the reservation's check-in date
   */
  public void changeCheckIn(String newCheckIn) {
    reservation.setCheckInDate(newCheckIn);
  }

  /*
   * Changes the reservation's number of nights
   */
  public void changeNumNights(int newNumNights) {
    reservation.setNumNights(newNumNights);
  }

  /*
   * Returns a String containing the guest's name and reservation information
   */
  public String toString() {
    return name + "\\n" + reservation;
  }
  
}`},{path:`Reservation.java`,text:`/*
 * Represents a hotel reservation
 */
public class Reservation {
  
  private String checkInDate;     // The check-in date for a reservation
  private int numNights;          // The number of nights for a reservation

  /*
   * Sets checkInDate and numNights to the specified values
   */
  public Reservation(String checkInDate, int numNights) {
    this.checkInDate = checkInDate;
    this.numNights = numNights;
  }

  /*
   * Returns the checkInDate for the reservation
   */
  public String getCheckInDate() {
    return checkInDate;
  }

  /*
   * Returns the number of nights for the reservation
   */
  public int getNumNights() {
    return numNights;
  }

  /*
   * Changes the check-in date for the reservation to newDate
   */
  public void setCheckInDate(String newDate) {
    checkInDate = newDate;
  }

  /*
   * Sets the number of nights for the reservation to newNumNights
   */
  public void setNumNights(int newNumNights) {
    numNights = newNumNights;
  }

  /*
   * Returns a String containing the reservation information
   */
  public String toString() {
    return "Check in: " + checkInDate + "\\nNumber of Nights: " + numNights;
  }
  
}`}],validationFiles:[{path:`GuestTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.lang.reflect.Constructor;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Guest.java Test")
public class GuestTest {

  String messageGap = "\\n       ";
  Reservation testReservation;
  Guest testObject;
  Class testClass;
  ArrayList<String> classConstructors;
   
  @BeforeEach
  public void setup() {
    testReservation = new Reservation("some date", 3);
    testObject = new Guest("some guest", testReservation);
    testClass = testObject.getClass();
    classConstructors = ConstructorsHelper.getClassConstructorsList(testClass, "Guest");
  }
   
  @Test
  @Order(1)
  @DisplayName("The Guest constructor takes a String for the name of the guest => ")
  public void testGuestConstructorStringParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a String parameter for the name of the guest." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public Guest", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), parameterMessage);
  }

  @Test
  @Order(2)
  @DisplayName("The Guest constructor takes a Reservation object => ")
  public void testGuestConstructorReservationParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a Reservation parameter for the Reservation object." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public Guest", actualName, constructorMessage);
    assertTrue(actualParameters.contains("Reservation"), parameterMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Guest constructor instantiates the reservation instance variable to a new Reservation object => ")
  public void testGuestConstructorInstantiatesNewReservation() {
    String message = "The Guest constructor should create a new instance of the Reservation class using the current";
    message += "\\n        values of the parameter reservation object's instance variables." + messageGap;
    Reservation actual = testObject.getReservation();
    assertNotEquals(testReservation, actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("changeCheckIn() and changeNumNights() do not modify the original Reservation object => ")
  public void testOriginalObjectNotModified() {
    String message = "The Guest constructor should create a new instance of the Reservation class using the current";
    message += "\\n        values of the parameter reservation object's instance variables." + messageGap;
      
    testObject.changeNumNights(20);
    assertNotEquals(testReservation.getNumNights(), testObject.getReservation().getNumNights(), message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (b) #2`,lesson:`Lesson 2: Object References as Parameters`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a Brand object
    Brand gibson = new Brand("Gibson", 575.99);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an Instrument object and print the object. Then call the
     * changePrice() method and print the updated Instrument object. Print the
     * Brand object to confirm it was not modified.
     * -----------------------------------------------------------------------------
     */




    
    
  }
}`},{path:`Brand.java`,text:`/*
 * Represents a brand of an instrument
 */
public class Brand {
  
  private String name;     // The name of a brand
  private double price;    // The price of a brand

  /*
   * Sets name and price to the specified values
   */
  public Brand(String name, double price) {
    this.name = name;
    this.price = price;
  }

  /*
   * Returns the name of the brand
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the price of the brand
   */
  public double getPrice() {
    return price;
  }

  /*
   * Sets the price to newPrice
   */
  public void setPrice(double newPrice) {
    price = newPrice;
  }

  /*
   * Returns a String containing the brand's information
   */
  public String toString() {
    return name + " - $" + price;
  }
  
}`},{path:`Instrument.java`,text:`/*
 * Represents an instrument in a music store
 */
public class Instrument {
      
  private Brand brand;    // The Brand of an instrument
  private String type;    // The type of an instrument

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the Instrument constructor with a parameter for a Brand object and
   * the type of an instrument. Instantiate the brand object to a new Brand object
   * using the values assigned to the parameter's instance variables.
   * -----------------------------------------------------------------------------
   */
  



  

  /*
   * Returns the instrument's brand
   */
  public Brand getBrand() {
    return brand;
  }
  
  /*
   * Returns the type of the instrument
   */
  public String getType() {
    return type;
  }

  /*
   * Changes the price of the instrument
   */
  public void changePrice(double newPrice) {
    brand.setPrice(newPrice);
  }

  /*
   * Returns a String containing the brand information and type
   */
  public String toString() {
    return type + "\\n" + brand;
  }
  
}`}],validationFiles:[{path:`InstrumentTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.lang.reflect.Constructor;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Instrument.java Test")
public class InstrumentTest {

  String messageGap = "\\n       ";
  Brand testBrand;
  Instrument testObject;
  Class testClass;
  ArrayList<String> classConstructors;
   
  @BeforeEach
  public void setup() {
    testBrand = new Brand("some brand", 573.99);
    testObject = new Instrument(testBrand, "guitar");
    testClass = testObject.getClass();
    classConstructors = ConstructorsHelper.getClassConstructorsList(testClass, "Instrument");
  }
   
  @Test
  @Order(1)
  @DisplayName("The Instrument constructor takes a Brand object => ")
  public void testInstrumentConstructorBrandParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a Brand parameter for the Brand object." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public Instrument", actualName, constructorMessage);
    assertTrue(actualParameters.contains("Brand"), parameterMessage);
  }

  @Test
  @Order(2)
  @DisplayName("The Instrument constructor has a parameter for the type of instrument => ")
  public void testInstrumentConstructorStringParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a parameter for the type of the instrument." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public Instrument", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), parameterMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Instrument constructor instantiates the brand instance variable to a new Brand object => ")
  public void testInstrumentConstructorInstantiatesNewBrand() {
    String message = "The Instrument constructor should create a new instance of the Brand class using the current";
    message += "\\n        values of the parameter brand object's instance variables." + messageGap;
    Brand actual = testObject.getBrand();
    assertNotEquals(testBrand, actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("changePrice() does not modify the original Brand object => ")
  public void testOriginalObjectNotModified() {
    String message = "The Instrument constructor should create a new instance of the Brand class using the current";
    message += "\\n        values of the parameter brand object's instance variables." + messageGap;
      
    testObject.changePrice(964.99);
    assertNotEquals(testBrand.getPrice(), testObject.getBrand().getPrice(), message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (c) #2`,lesson:`Lesson 2: Object References as Parameters`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Customer object
    Customer amirah = new Customer("Amirah");

    // Creates an Item object
    Item burger = new Item("Cheeseburger", 9.99);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an OrderScene object and call the drawScene() method. Then call
     * the changePrice() method and call the drawScene() method again. Print the
     * Item object to confirm it was not modified.
     * -----------------------------------------------------------------------------
     */



    
    // Plays the scene
    Theater.playScenes(meal);
    
  }
}`},{path:`Customer.java`,text:`/*
 * Represents a customer
 */
public class Customer {
  
  private String name;    // The name of a customer

  /*
   * Sets name to the specified name
   */
  public Customer(String name) {
    this.name = name;
  }

  /*
   * Returns the customer's name
   */
  public String getName() {
    return name;
  }
  
}`},{path:`Item.java`,text:`/*
 * Represents a menu item
 */
public class Item {
  
  private String type;    // The type of item
  private double price;   // The price of an item
  
  /*
   * Sets type to the specified type and price to the specified price
   */
  public Item(String type, double price) {
    this.type = type;
    this.price = price;
  }

  /*
   * Returns the type of the item
   */
  public String getType() {
    return type;
  }

  /*
   * Returns the price of the item
   */
  public double getPrice() {
    return price;
  }

  /*
   * Sets the price of the item to newPrice
   */
  public void setPrice(double newPrice) {
    price = newPrice;
  }

  /*
   * Returns a String containing the item's information
   */
  public String toString() {
    return type + " - $" + price;
  }
  
}`},{path:`OrderScene.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents an order
 */
public class OrderScene extends Scene {
  
  private Customer customer;   // The customer the order is for
  private Item item;           // The item the customer has selected

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the OrderScene constructor with a parameter for a Customer object,
   * and a parameter for an Item object. Instantiate the Customer and Item objects
   * using the values assigned to their instance variables.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns the customer
   */
  public Customer getCustomer() {
    return customer;
  }

  /*
   * Returns the item the customer has selected
   */
  public Item getItem() {
    return item;
  }

  /*
   * Changes the price of the item
   */
  public void changePrice(double newPrice) {
    item.setPrice(newPrice);
  }

  /*
   * Draws the order information in the scene
   */
  public void drawScene() {
    drawImage("background.jpg", 0, 0, getWidth());
    drawImage("plate.png", 50, 50, 100);
    formatText(30, Color.MAROON);
    drawText(customer.getName() + "", 180, 100);
    formatText(24, Color.WHITE);
    drawText(item + "", 50, 225);
    pause(2);
  }

  /*
   * Formats the text
   */
  private void formatText(int size, Color color) {
    setTextStyle(Font.SANS, FontStyle.BOLD);
    setTextHeight(size);
    setTextColor(color);
  }
  
}`}],validationFiles:[{path:`OrderTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Order.java Test")
public class OrderTest {

  String messageGap = "\\n       ";
  Customer testCustomer;
  Item testItem;
  OrderScene testObject;
  Class testClass;
  ArrayList<String> classConstructors;
   
  @BeforeEach
  public void setup() {
    testCustomer = new Customer("some customer");
    testItem = new Item("some item", 2.99);
    testObject = new OrderScene(testCustomer, testItem);
    testClass = testObject.getClass();
    classConstructors = ConstructorsHelper.getClassConstructorsList(testClass, "OrderScene");
  }
   
  @Test
  @Order(1)
  @DisplayName("The OrderScene constructor takes a Customer object => ")
  public void testOrderSceneConstructorCustomerParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a Customer parameter for the Customer object." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public OrderScene", actualName, constructorMessage);
    assertTrue(actualParameters.contains("Customer"), parameterMessage);
  }

  @Test
  @Order(2)
  @DisplayName("The OrderScene constructor takes an Item object => ")
  public void testOrderSceneConstructorItemParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add an Item parameter for the Item object." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public OrderScene", actualName, constructorMessage);
    assertTrue(actualParameters.contains("Item"), parameterMessage);
  }

  @Test
  @Order(3)
  @DisplayName("OrderScene constructor instantiates the customer instance variable to a new Customer object => ")
  public void testOrderSceneConstructorInstantiatesNewCustomer() {
    String message = "The OrderScene constructor should create a new instance of the Customer class using the current";
    message += "\\n        values of the parameter customer object's instance variables." + messageGap;
    Customer actual = testObject.getCustomer();
    assertNotEquals(testCustomer, actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("OrderScene constructor instantiates the item instance variable to a new Item object => ")
  public void testOrderSceneConstructorInstantiatesNewItem() {
    String message = "The OrderScene constructor should create a new instance of the Item class using the current";
    message += "\\n        values of the parameter item object's instance variables." + messageGap;
    Item actual = testObject.getItem();
    assertNotEquals(testItem, actual, message);
  }

  @Test
  @Order(5)
  @DisplayName("changePrice() does not modify the original Item object => ")
  public void testOriginalObjectNotModified() {
    String message = "The OrderScene constructor should create a new instance of the Item class using the current";
    message += "\\n        values of the parameter item object's instance variables." + messageGap;
      
    testObject.changePrice(15.99);
    assertNotEquals(testItem.getPrice(), testObject.getItem().getPrice(), message);
  }
   
}`}],dataFiles:[]},{name:`Practice: Object References as Parameters (d) #2`,lesson:`Lesson 2: Object References as Parameters`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a Destination object
    Destination newYork = new Destination("Kayla Jackson", "123 Main Street, Brooklyn, NY");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Shipment object and call the drawScene() method. Then
     * call the changeAddress() method and call the drawScene() method again. Print
     * the Destination object to confirm it was not modified.
     * -----------------------------------------------------------------------------
     */




    

    // Plays the scene
    Theater.playScenes(birthdayGift);
    
  }
}`},{path:`Destination.java`,text:`/*
 * Represents a destination for a shipment
 */
public class Destination {
  
  private String name;        // The name of a destination
  private String address;     // The address of a destination

  /*
   * Sets name and address to the specified values
   */
  public Destination(String name, String address) {
    this.name = name;
    this.address = address;
  }

  /*
   * Returns the name of the destination
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the address of the destination
   */
  public String getAddress() {
    return address;
  }

  /*
   * Sets the address to newAddress
   */
  public void setAddress(String newAddress) {
    address = newAddress;
  }

  /*
   * Returns a String containing the name and address of the destination
   */
  public String toString() {
    return name + ", " + address;
  }
  
}`},{path:`Shipment.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class Shipment extends Scene {
  
  private Destination destination;    // The destination for the shipment
  private double weight;              // The weight of the shipment

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the Shipment constructor with a parameter for a Destination object
   * and a parameter for the weight. Instantiate the Destination object using the
   * values assigned to the parameter's instance variables.
   * -----------------------------------------------------------------------------
   */
  



  

  /*
   * Returns the destination
   */
  public Destination getDestination() {
    return destination;
  }

  /*
   * Returns the weight
   */
  public double getWeight() {
    return weight;
  }

  /*
   * Changes the address for the destination
   */
  public void changeAddress(String newAddress) {
    destination.setAddress(newAddress);
  }

  /*
   * Draws the shipment information in the scene
   */
  public void drawScene() {
    drawPlane();
    drawImage("world.png", 0, 0, getWidth());
    formatText(30, Color.NAVY);
    drawText(destination.getName() + "", 80, 170);
    formatText(20, Color.BLACK);
    drawText(destination.getAddress() + "", 80, 250);
    pause(2);
    clear(Color.WHITE);
  }

  /*
   * Draws the plane moving across the scene
   */
  private void drawPlane() {
    int xPos = 30;
    int yPos = 350;

    while (xPos < getWidth()) {
      drawImage("world.png", 0, 0, getWidth());
      drawImage("airplane.png", xPos, yPos, 75);
      pause(0.2);
      clear(Color.WHITE);
      xPos += 50;
      yPos -= 50;
    }
  }

  /*
   * Formats the text
   */
  private void formatText(int size, Color color) {
    setTextStyle(Font.SANS, FontStyle.BOLD);
    setTextHeight(size);
    setTextColor(color);
  }

}`}],validationFiles:[{path:`ShipmentTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Shipment.java Test")
public class ShipmentTest {

  String messageGap = "\\n       ";
  Destination testDestination;
  Shipment testObject;
  Class testClass;
  ArrayList<String> classConstructors;
   
  @BeforeEach
  public void setup() {
    testDestination = new Destination("some name", "some address");
    testObject = new Shipment(testDestination, 12.5);
    testClass = testObject.getClass();
    classConstructors = ConstructorsHelper.getClassConstructorsList(testClass, "Shipment");
  }
   
  @Test
  @Order(1)
  @DisplayName("The Shipment constructor takes a Destination object => ")
  public void testShipmentConstructorDestinationParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a Destination parameter for the Destination object." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public Shipment", actualName, constructorMessage);
    assertTrue(actualParameters.contains("Destination"), parameterMessage);
  }

  @Test
  @Order(2)
  @DisplayName("The Shipment constructor takes a parameter for the weight => ")
  public void testShipmentConstructorWeightParameter() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String parameterMessage = "Inside the parentheses for the method signature, add a double parameter for the weight." + messageGap;
      
    String actualConstructor = classConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);

    assertEquals("public Shipment", actualName, constructorMessage);
    assertTrue(actualParameters.contains("double"), parameterMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Shipment constructor instantiates the destination instance variable to a new Destination object => ")
  public void testShipmentConstructorInstantiatesNewDestination() {
    String message = "The Shipment constructor should create a new instance of the Destination class using the current";
    message += "\\n        values of the parameter destination object's instance variables." + messageGap;
    Destination actual = testObject.getDestination();
    assertNotEquals(testDestination, actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("changeAddress() does not modify the original Destination object => ")
  public void testOriginalObjectNotModified() {
    String message = "The Shipment constructor should create a new instance of the Destination class using the current";
    message += "\\n        values of the parameter destination object's instance variables." + messageGap;
      
    testObject.changeAddress("some new address");
    assertNotEquals(testDestination.getAddress(), testObject.getDestination().getAddress(), message);
  }
   
}`}],dataFiles:[]},{name:`Predict and Run: Overloaded Methods`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`MusicPlayer.java`,text:`/*
 * Represents a music player
 */
public class MusicPlayer {

  /*
   * Returns a String containing the text "Now playing: "
   * and the song's information
   */
  public String play(Song song) {
    return "Now playing: " + song;
  }

  /* 
   * Returns a String containig each song in playlist
   */
  public String play(Playlist playlist) {
    String result = "First up: " + playlist.getSong(0) + "\\n";

    for (int index = 1; index < playlist.getNumSongs(); index++) {
      result += "Next up: " + playlist.getSong(index) + "\\n";
    }

    return result;
  }
  
}`},{path:`Playlist.java`,text:`import java.util.ArrayList;

/*
 * Represents a playlist of songs
 */
public class Playlist {

  private ArrayList<Song> songs;    // The list of Song objects

  /*
   * Initializes songs to an empty list
   */
  public Playlist() {
    this.songs = new ArrayList<Song>();
  }

  /*
   * Returns the list of songs
   */
  public ArrayList<Song> getSongs() {
    return songs;
  }

  /*
   * Returns the number of songs in the playlist
   */
  public int getNumSongs() {
    return songs.size();
  }

  /*
   * Returns the song at the specified index
   */
  public Song getSong(int index) {
    return songs.get(index);
  }

  /*
   * Adds a song to the list of songs
   */
  public void addSong(Song newSong) {
    songs.add(new Song(newSong.getTitle(), newSong.getArtist()));
  }

  /*
   * Returns a String containing each song in the playlist
   */
  public String toString() {
    String result = "";

    for (Song song : songs) {
      result += song + "\\n";
    }

    return result;
  }
  
}`},{path:`Song.java`,text:`/*
 * Represents a song
 */
public class Song {

  private String title;     // The title of a song
  private String artist;    // The name of the artist of a song

  /*
   * Sets title and artist to the specified values
   */
  public Song(String title, String artist) {
    this.title = title;
    this.artist = artist;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the artist of the song
   */
  public String getArtist() {
    return artist;
  }

  /*
   * Returns a String containing the song information
   */
  public String toString() {
    return title + " by " + artist;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Overloaded Methods #1`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`MusicPlayer.java`,text:`/*
 * Represents a music player
 */
public class MusicPlayer {

  /*
   * Returns a String containing the text "Now playing: "
   * and the song's information
   */
  public String play(Song song) {
    return "Now playing: " + song;
  }

  /* 
   * Returns a String containig each song in playlist
   */
  public String play(Playlist playlist) {
    String result = "First up: " + playlist.getSong(0) + "\\n";

    for (int index = 1; index < playlist.getNumSongs(); index++) {
      result += "Next up: " + playlist.getSong(index) + "\\n";
    }

    return result;
  }
  
}`},{path:`Playlist.java`,text:`import java.util.ArrayList;

/*
 * Represents a playlist of songs
 */
public class Playlist {

  private ArrayList<Song> songs;    // The list of Song objects

  /*
   * Initializes songs to an empty list
   */
  public Playlist() {
    this.songs = new ArrayList<Song>();
  }

  /*
   * Returns the list of songs
   */
  public ArrayList<Song> getSongs() {
    return songs;
  }

  /*
   * Returns the number of songs in the playlist
   */
  public int getNumSongs() {
    return songs.size();
  }

  /*
   * Returns the song at the specified index
   */
  public Song getSong(int index) {
    return songs.get(index);
  }

  /*
   * Adds a song to the list of songs
   */
  public void addSong(Song newSong) {
    songs.add(new Song(newSong.getTitle(), newSong.getArtist()));
  }

  /*
   * Returns a String containing each song in the playlist
   */
  public String toString() {
    String result = "";

    for (Song song : songs) {
      result += song + "\\n";
    }

    return result;
  }
  
}`},{path:`Song.java`,text:`/*
 * Represents a song
 */
public class Song {

  private String title;     // The title of a song
  private String artist;    // The name of the artist of a song

  /*
   * Sets title and artist to the specified values
   */
  public Song(String title, String artist) {
    this.title = title;
    this.artist = artist;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the artist of the song
   */
  public String getArtist() {
    return artist;
  }

  /*
   * Returns a String containing the song information
   */
  public String toString() {
    return title + " by " + artist;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Overloaded Methods #2`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`MusicPlayer.java`,text:`/*
 * Represents a music player
 */
public class MusicPlayer {

  /*
   * Returns a String containing the text "Now playing: "
   * and the song's information
   */
  public String play(Song song) {
    return "Now playing: " + song;
  }

  /* 
   * Returns a String containig each song in playlist
   */
  public String play(Playlist playlist) {
    String result = "First up: " + playlist.getSong(0) + "\\n";

    for (int index = 1; index < playlist.getNumSongs(); index++) {
      result += "Next up: " + playlist.getSong(index) + "\\n";
    }

    return result;
  }
  
}`},{path:`Playlist.java`,text:`import java.util.ArrayList;

/*
 * Represents a playlist of songs
 */
public class Playlist {

  private ArrayList<Song> songs;    // The list of Song objects

  /*
   * Initializes songs to an empty list
   */
  public Playlist() {
    this.songs = new ArrayList<Song>();
  }

  /*
   * Returns the list of songs
   */
  public ArrayList<Song> getSongs() {
    return songs;
  }

  /*
   * Returns the number of songs in the playlist
   */
  public int getNumSongs() {
    return songs.size();
  }

  /*
   * Returns the song at the specified index
   */
  public Song getSong(int index) {
    return songs.get(index);
  }

  /*
   * Adds a song to the list of songs
   */
  public void addSong(Song newSong) {
    songs.add(new Song(newSong.getTitle(), newSong.getArtist()));
  }

  /*
   * Returns a String containing each song in the playlist
   */
  public String toString() {
    String result = "";

    for (Song song : songs) {
      result += song + "\\n";
    }

    return result;
  }
  
}`},{path:`Song.java`,text:`/*
 * Represents a song
 */
public class Song {

  private String title;     // The title of a song
  private String artist;    // The name of the artist of a song

  /*
   * Sets title and artist to the specified values
   */
  public Song(String title, String artist) {
    this.title = title;
    this.artist = artist;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the artist of the song
   */
  public String getArtist() {
    return artist;
  }

  /*
   * Returns a String containing the song information
   */
  public String toString() {
    return title + " by " + artist;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Overloaded Methods #3`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`MusicPlayer.java`,text:`/*
 * Represents a music player
 */
public class MusicPlayer {

  /*
   * Returns a String containing the text "Now playing: "
   * and the song's information
   */
  public String play(Song song) {
    return "Now playing: " + song;
  }

  /* 
   * Returns a String containig each song in playlist
   */
  public String play(Playlist playlist) {
    String result = "First up: " + playlist.getSong(0) + "\\n";

    for (int index = 1; index < playlist.getNumSongs(); index++) {
      result += "Next up: " + playlist.getSong(index) + "\\n";
    }

    return result;
  }
  
}`},{path:`Playlist.java`,text:`import java.util.ArrayList;

/*
 * Represents a playlist of songs
 */
public class Playlist {

  private ArrayList<Song> songs;    // The list of Song objects

  /*
   * Initializes songs to an empty list
   */
  public Playlist() {
    this.songs = new ArrayList<Song>();
  }

  /*
   * Returns the list of songs
   */
  public ArrayList<Song> getSongs() {
    return songs;
  }

  /*
   * Returns the number of songs in the playlist
   */
  public int getNumSongs() {
    return songs.size();
  }

  /*
   * Returns the song at the specified index
   */
  public Song getSong(int index) {
    return songs.get(index);
  }

  /*
   * Adds a song to the list of songs
   */
  public void addSong(Song newSong) {
    songs.add(new Song(newSong.getTitle(), newSong.getArtist()));
  }

  /*
   * Returns a String containing each song in the playlist
   */
  public String toString() {
    String result = "";

    for (Song song : songs) {
      result += song + "\\n";
    }

    return result;
  }
  
}`},{path:`Song.java`,text:`/*
 * Represents a song
 */
public class Song {

  private String title;     // The title of a song
  private String artist;    // The name of the artist of a song

  /*
   * Sets title and artist to the specified values
   */
  public Song(String title, String artist) {
    this.title = title;
    this.artist = artist;
  }

  /*
   * Returns the title of the song
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the artist of the song
   */
  public String getArtist() {
    return artist;
  }

  /*
   * Returns a String containing the song information
   */
  public String toString() {
    return title + " by " + artist;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Overloaded Methods (a) — Console`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {
    
    // Create a VideoPlatform object
    VideoPlatform myPlatform = new VideoPlatform();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call both versions of the createAccount() method and print the User objects.
     * -----------------------------------------------------------------------------
     */
  

    
    
  }
}`},{path:`User.java`,text:`/*
 * Represents a User
 */
public class User {
  
  private String username; // username
  private String password; // password
  private String email;    // email

  public User(String username, String password, String email) {
    this.username = username;
    this.password = password;
    this.email = email;
  }

  public User(String username, String password) {
    this(username, password, "");
  }

  /*
   * Returns a String containing the user's information
   */
  public String toString() {
    return "username: " + username + " email: " + email;
  }

}`},{path:`VideoPlatform.java`,text:`/*
 * Represents a video platform
 */
public class VideoPlatform {
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createAccount() method. The method should take a username,
   * password, and email address as parameters, and should return a new User object.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createAccount() method. The method should take a username and
   * password as parameters, and should return a new User object.
   * -----------------------------------------------------------------------------
   */


  


}`}],validationFiles:[{path:`VideoPlatformTest.java`,text:`import java.util.ArrayList;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("VideoPlatform.java Test")
public class VideoPlatformTest {

  String messageGap = "\\n       ";
  VideoPlatform testPlatform;
   
  @BeforeEach
  public void setup() {
    testPlatform = new VideoPlatform();
  }

  @Test
  @Order(1)
  @DisplayName("Overload createAccount => ")
  public void testCreateAccount() {
    String message = "Overloaded methods have the same name." + messageGap;

    String username =  "user" + (int) (Math.random() * 100);
    String username2 =  "user" + (int) (Math.random() * 100);
    String email = (int) (Math.random() * 100) + "@code.org";
    User user = testPlatform.createAccount(username, "notsecure", email);
    User user2 = testPlatform.createAccount(username2, "weakpassword");

    String expected = "username: " + username + " email: " + email;
    String expected2 = "username: " + username2 + " email: ";

    assertEquals(expected, user.toString(), message);
    assertEquals(expected2, user2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (b) — Console`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {
    
    ArrayList<Product> products = new ArrayList<Product>();
    products.add(new Product("1", "AP CSA textbook"));
    products.add(new Product("20", "running shoes"));
    products.add(new Product("10", "smartphone"));

    // Create a ShoppingPlatform object
    ShoppingPlatform myPlatform = new ShoppingPlatform(products);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call both versions of the search() method and print the Product objects
     * -----------------------------------------------------------------------------
     */
    



    
    
  }
}`},{path:`Product.java`,text:`/*
 * Represents a Product
 */
public class Product {
  
  private String ID;   // ID
  private String name; // name

  public Product(String ID, String name) {
    this.ID = ID;
    this.name = name;
  }

  /*
   * Returns the ID
   */
  public String getID() {
    return ID;
  }

  /*
   * Returns the name
   */
  public String getName() {
    return name;
  }
  
  /*
   * Returns a String containing the products's information
   */
  public String toString() {
    return "ID: " + ID + " name: " + name;
  }

}`},{path:`ShoppingPlatform.java`,text:`import java.util.ArrayList;

/*
 * Represents a video platform
 */
public class ShoppingPlatform {
  
  private ArrayList<Product> products; // list of products

  public ShoppingPlatform(ArrayList<Product> products) {
    this.products = products;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the search() method. The method should take a product name and ID
   * as parameters, and return a Product. If no product is found, return null.
   * -----------------------------------------------------------------------------
   */






  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the search() method. The method should take a product ID
   * as parameter, and return a Product. If no product is found, return null.
   * -----------------------------------------------------------------------------
   */




  

}`}],validationFiles:[{path:`ShoppingPlatformTest.java`,text:`import java.util.ArrayList;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ShoppingPlatform.java Test")
public class ShoppingPlatformTest {

  String messageGap = "\\n       ";
  ShoppingPlatform testPlatform;
  Product product;
  Product product2;
  Product product3;
   
  @BeforeEach
  public void setup() {
    String ID =  (int) (Math.random() * 100) + "";
    String ID2 =  (int) (Math.random() * 100) + "";
    String ID3 =  (int) (Math.random() * 100) + "";
    String name = "product" + ID;
    String name2 = "product" + ID2;
    String name3 = "product" + ID3;
    product = new Product(ID, name);
    product2 = new Product(ID2, name2);
    product3 = new Product(ID3, name3);

    ArrayList<Product> products = new ArrayList<Product>();
    products.add(product);
    products.add(product2);
    products.add(new Product(ID3, name3));
    
    testPlatform = new ShoppingPlatform(products);
  }

  @Test
  @Order(1)
  @DisplayName("Overload search => ")
  public void testSearch() {
    String message = "Overloaded methods have the same name." + messageGap;

    Product searchProduct = testPlatform.search(product.getID(), product.getName());
    Product searchProduct2 = testPlatform.search(product2.getID());

    assertEquals(product, searchProduct, message);
    assertEquals(product2, searchProduct2, message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (c) — Console`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {
    
    // Creates a Platform object
    Platform myPlatform = new Platform();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call both versions of the createUser() method and print the User objects
     * -----------------------------------------------------------------------------
     */
    
   


    
  }
}`},{path:`Platform.java`,text:`/*
 * Represents a platform
 */
public class Platform {
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createUser() method. The method should take a username,
   * password, and email address as parameters, and should return a new User object.
   * -----------------------------------------------------------------------------
   */





  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createUser() method. The method should take a username and
   * password as parameters, and should return a new User object.
   * -----------------------------------------------------------------------------
   */




  

}`},{path:`User.java`,text:`/*
 * Represents a User
 */
public class User {
  
  private String username; // username
  private String password; // password
  private String email;    // email

  public User(String username, String password, String email) {
    this.username = username;
    this.password = password;
    this.email = email;
  }

  public User(String username, String password) {
    this(username, password, "");
  }

  /*
   * Returns a String containing the User's information
   */
  public String toString() {
    return "username: " + username + " email: " + email;
  }

}`}],validationFiles:[{path:`PlatformTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Platform.java Test")
public class PlatformTest {

  String messageGap = "\\n       ";
  Platform testPlatform;
   
  @BeforeEach
  public void setup() {
    testPlatform = new Platform();
  }

  @Test
  @Order(1)
  @DisplayName("Overload createUser => ")
  public void testCreateUser() {
    String message = "Overloaded methods have the same name." + messageGap;

    String username =  "user" + (int) (Math.random() * 100);
    String username2 =  "user" + (int) (Math.random() * 100);
    String email = (int) (Math.random() * 100) + "@code.org";
    User user = testPlatform.createUser(username, "notsecure", email);
    User user2 = testPlatform.createUser(username2, "weakpassword");

    String expected = "username: " + username + " email: " + email;
    String expected2 = "username: " + username2 + " email: ";

    assertEquals(expected, user.toString(), message);
    assertEquals(expected2, user2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (d) — Console`,lesson:`Lesson 3: Overloaded Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {
    
    // Creates a League object
    League myLeague = new League();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call both versions of the createGame() method and print the Game objects
     * -----------------------------------------------------------------------------
     */
 




    
  }
}`},{path:`Game.java`,text:`/*
 * Represents a Game
 */
public class Game {
  
  private String ID;   // ID
  private String date; // date
  private String time; // time
  private String name; // name

  public Game(String ID, String date, String time, String name) {
    this.ID = ID;
    this.date = date;
    this.time = time;
    this.name = name;
  }

  public Game(String ID, String date, String time) {
    this(ID, date, time, "");
  }

  /*
   * Returns a String containing the game's information
   */
  public String toString() {
    return "ID: " + ID + " date: " + date + " time: " + time + " name: " + name;
  }

}`},{path:`League.java`,text:`/*
 * Represents a league
 */
public class League {
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createGame() method. The method should take a game ID,
   * date, time, and team name as parameters, and should return a new Game object.
   * -----------------------------------------------------------------------------
   */





  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createGame() method. The method should take a game ID,
   * date, and time as parameters, and should return a new Game object.
   * -----------------------------------------------------------------------------
   */




  
  
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
  League testLeague;
   
  @BeforeEach
  public void setup() {
    testLeague = new League();
  }

  @Test
  @Order(1)
  @DisplayName("Overload createGame => ")
  public void testCreateGame() {
    String message = "Overloaded methods have the same name." + messageGap;

    String ID =  "game" + (int) (Math.random() * 100);
    String ID2 =  "game" + (int) (Math.random() * 100);
    Game game = testLeague.createGame(ID, "05/03/2023", "12 PM", "championship");
    Game game2 = testLeague.createGame(ID2, "05/04/2022", "12 PM");

    String expected = "ID: " + ID + " date: 05/03/2023 time: 12 PM name: championship";
    String expected2 = "ID: " + ID2 + " date: 05/04/2022 time: 12 PM name: ";

    assertEquals(expected, game.toString(), message);
    assertEquals(expected2, game2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (a) — Theater`,lesson:`Lesson 3: Overloaded Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {
    
    // Creates a School object
    School scene = new School();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call both versions of the createCourse() method
     * -----------------------------------------------------------------------------
     */

    scene.drawResult();

    Theater.playScenes(scene);
  }
}`},{path:`Course.java`,text:`import org.code.theater.*;
import org.code.media.*;

/*
 * Represents a Course
 */
public class Course {
  
  private String ID;   // ID
  private String name; // name

  public Course(String ID, String name) {
    this.ID = ID;
    this.name = name;
  }

  public Course(String ID) {
    this(ID, "");
  }

  /*
   * Returns a String containing the course's information
   */
  public String toString() {
    return "ID: " + ID + " name: " + name;
  }

}`},{path:`School.java`,text:`import java.util.ArrayList;

import org.code.theater.*;

/*
 * Represents a school
 */
public class School extends Scene {
  private ArrayList<Course> courses = new ArrayList<Course>();
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createCourse() method. The method should take a course ID and
   * name as parameters, and should return a new Course object. Add the new Course
   * to the courses ArrayList.
   * -----------------------------------------------------------------------------
   */




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createCourse() method. The method should take a course ID as a 
   * parameter, and should return a new Course object. Add the new Course
   * to the courses ArrayList.
   * -----------------------------------------------------------------------------
   */




  

  /*
   * Draws the result in the scene
   */
  public void drawResult() {
    drawImage("whiteboard.png", 0, 0, getWidth());
    int y = 100;
    
    for (Course c : courses) {
      drawText(c.toString(), 50, y);
      y += 20;
      pause(0.5);
    }
  }

}`}],validationFiles:[{path:`SchoolTest.java`,text:`import java.util.ArrayList;

import static org.junit.jupiter.api.Assertions.*;

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
  School testSchool;
   
  @BeforeEach
  public void setup() {
    testSchool = new School();
  }

  @Test
  @Order(1)
  @DisplayName("Overload createCourse => ")
  public void testCreateCourse() {
    String message = "Overloaded methods have the same name." + messageGap;

    String ID =  "id" + (int) (Math.random() * 100);
    String ID2 =  "id" + (int) (Math.random() * 100);
    String name = "name " + (int) (Math.random() * 100);
    Course course = testSchool.createCourse(ID, name);
    Course course2 = testSchool.createCourse(ID2);

    String expected = "ID: " + ID + " name: " + name;
    String expected2 = "ID: " + ID2 + " name: ";

    assertEquals(expected, course.toString(), message);
    assertEquals(expected2, course2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (b) — Theater`,lesson:`Lesson 3: Overloaded Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Create a TravelApp object
    TravelApp scene = new TravelApp();

   /* ----------------------------------- TO DO -----------------------------------
   * ✅ Call both versions of the createBooking() method
   * -----------------------------------------------------------------------------
   */

    scene.drawResult();

    Theater.playScenes(scene);
  }
}`},{path:`Booking.java`,text:`/*
 * Represents a Booking
 */
public class Booking {
  
  private int num;     // booking number
  private double cost; // cost
  private String name; // traveler name

  public Booking(int num, double cost, String name) {
    this.num = num;
    this.cost = cost;
    this.name = name;
  }

  public Booking(int num) {
    this(num, 0.0, "");
  }

  /*
   * Returns a String containing the booking's information
   */
  public String toString() {
    return "booking number: " + num + " cost: " + cost + " name: " + name;
  }

}`},{path:`TravelApp.java`,text:`import java.util.ArrayList;

import org.code.theater.*;

/*
 * Represents a travel app
 */
public class TravelApp extends Scene {
  
  private ArrayList<Booking> bookings = new ArrayList<Booking>();
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createBooking() method. The method should take booking number,
   * total cost, and traveler information as parameters. and should return a new
   * Booking object. Add the new Booking to the bookings ArrayList.
   * -----------------------------------------------------------------------------
   */




  


  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the createBooking() method. The method should take a booking number as a 
   * parameter, and should return a new Booking object. Add the new Booking
   * to the bookings ArrayList.
   * -----------------------------------------------------------------------------
   */


  /*
   * Draws the result in the scene
   */
  public void drawResult() {
    drawImage("worldwide.png", 0, 0, getWidth());
    int y = 100;
    for (Booking b : bookings) {
      drawText(b.toString(), 5, y);
      y += 20;
      pause(0.5);
    }
  }

}`}],validationFiles:[{path:`TravelAppTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TravelApp.java Test")
public class TravelAppTest {

  String messageGap = "\\n       ";
  TravelApp testTravelApp;
   
  @BeforeEach
  public void setup() {
    testTravelApp = new TravelApp();
  }

  @Test
  @Order(1)
  @DisplayName("Overload createBooking => ")
  public void testCreateBooking() {
    String message = "Overloaded methods have the same name." + messageGap;

    int num = (int) (Math.random() * 100);
    int num2 =  (int) (Math.random() * 100);
    double cost = (Math.random() * 100);
    String name = "name " + (int) (Math.random() * 100);
    Booking booking = testTravelApp.createBooking(num, cost, name);
    Booking booking2 = testTravelApp.createBooking(num2);

    String expected = "booking number: " + num + " cost: " + cost + " name: " + name;
    String expected2 = "booking number: " + num2 + " cost: 0.0 name: ";

    assertEquals(expected, booking.toString(), message);
    assertEquals(expected2, booking2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (c) — Theater`,lesson:`Lesson 3: Overloaded Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Create a RentalStore object
    RentalStore scene = new RentalStore();

   /* ----------------------------------- TO DO -----------------------------------
   * ✅ Call both versions of the addMovie() method
   * -----------------------------------------------------------------------------
   */

    scene.drawResult();

    Theater.playScenes(scene);
  }
}`},{path:`Movie.java`,text:`/*
 * Represents a Movie
 */
public class Movie {
  
  private int num;     // quantity
  private String name; // movie title

  public Movie(int num, String name) {
    this.num = num;
    this.name = name;
  }

  public Movie(int num) {
    this(num, "");
  }

  /*
   * Returns a String containing the movies's information
   */
  public String toString() {
    return "number: " + num + " name: " + name;
  }

}`},{path:`RentalStore.java`,text:`import java.util.ArrayList;

import org.code.theater.*;

/*
 * Represents a rental store
 */
public class RentalStore extends Scene {
  
  private ArrayList<Movie> movies = new ArrayList<Movie>();
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the addMovie() method. The method should take a quantity and movie
   * title as parameters. and should return a new Movie object. Add the new Movie
   * to the movies ArrayList.
   * -----------------------------------------------------------------------------
   */




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the addMovie() method. The method should take a quantity as a 
   * parameter, and should return a new Movie object. Add the new Movie
   * to the movies ArrayList.
   * -----------------------------------------------------------------------------
   */





  
  /*
   * Draws the result in the scene
   */
  public void drawResult() {
    drawImage("shelf.png", 0, 0, getWidth());
    int y = 100;
    for (Movie m : movies) {
      drawText(m.toString(), 5, y);
      y += 20;
      pause(0.5);
    }
  }

}`}],validationFiles:[{path:`RentalStoreTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("RentalStore.java Test")
public class RentalStoreTest {

  String messageGap = "\\n       ";
  RentalStore testRentalStore;
   
  @BeforeEach
  public void setup() {
    testRentalStore = new RentalStore();
  }

  @Test
  @Order(1)
  @DisplayName("Overload addMovie => ")
  public void testAddMovie() {
    String message = "Overloaded methods have the same name." + messageGap;

    int num = (int) (Math.random() * 100);
    int num2 =  (int) (Math.random() * 100);
    String name = "name " + (int) (Math.random() * 100);
    Movie movie = testRentalStore.addMovie(num, name);
    Movie movie2 = testRentalStore.addMovie(num2);

    String expected = "number: " + num + " name: " + name;
    String expected2 = "number: " + num2 + " name: ";

    assertEquals(expected, movie.toString(), message);
    assertEquals(expected2, movie2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Practice: Overloaded Methods (d) — Theater`,lesson:`Lesson 3: Overloaded Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Create a new Agency object
    Agency scene = new Agency();

   /* ----------------------------------- TO DO -----------------------------------
   * ✅ Call both versions of the addProperty() method
   * -----------------------------------------------------------------------------
   */


    scene.drawResult();

    Theater.playScenes(scene);
  }
}`},{path:`Agency.java`,text:`import java.util.ArrayList;

import org.code.theater.*;

/*
 * Represents a agency
 */
public class Agency extends Scene {
  
  private ArrayList<Property> properties = new ArrayList<Property>();
  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the addProperty() method. The method should take an address and type 
   * as parameters, and should return a new Property object. Add the new Property
   * to the properties ArrayList.
   * -----------------------------------------------------------------------------
   */




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the addProperty() method. The method should take an address as a 
   * parameter, and should return a new Property object. Add the new Property
   * to the properties ArrayList.
   * -----------------------------------------------------------------------------
   */





  
  /*
   * Draws the result in the scene
   */
  public void drawResult() {
    drawImage("blanksign.png", 0, 0, getWidth());
    int y = 120;
    setTextHeight(12);
    for (Property p : properties) {
      drawText(p.toString(), 30, y);
      y += 20;
      pause(0.5);
    }
  }

}`},{path:`Property.java`,text:`/*
 * Represents a Property
 */
public class Property {
  
  private String address;  // address
  private String type;     // type

  public Property(String address, String type) {
    this.address = address;
    this.type = type;
  }

  public Property(String address) {
    this(address, "");
  }

  /*
   * Returns a String containing the property's information
   */
  public String toString() {
    return "address: " + address + " type: " + type;
  }

}`}],validationFiles:[{path:`AgencyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Agency.java Test")
public class AgencyTest {

  String messageGap = "\\n       ";
  Agency testAgency;
   
  @BeforeEach
  public void setup() {
    testAgency = new Agency();
  }

  @Test
  @Order(1)
  @DisplayName("Overload addProperty => ")
  public void testAddProperty() {
    String message = "Overloaded methods have the same name." + messageGap;

    String address = (int) (Math.random() * 100) + " street";
    String address2 =  (int) (Math.random() * 100) + " street";
    String type = "type " + (int) (Math.random() * 100);
    Property property = testAgency.addProperty(address, type);
    Property property2 = testAgency.addProperty(address2);

    String expected = "address: " + address + " type: " + type;
    String expected2 = "address: " + address2 + " type: ";

    assertEquals(expected, property.toString(), message);
    assertEquals(expected2, property2.toString(), message);
  }
}`}],dataFiles:[]},{name:`Investigate and Modify: Private Methods #1`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Recipe.java`,text:`public class Recipe {

  private String name;
  private String diet;

  public Recipe(String name, String diet) {
    this.name = name;
    this.diet = diet;
  }

  public String getName() {
    return name;
  }

  public String getDiet() {
    return diet;
  }

  public String toString() {
    return name + " (" + diet + ")";
  }
  
}`},{path:`RecipeBlog.java`,text:`import java.util.ArrayList;

public class RecipeBlog {
  
  private ArrayList<Recipe> recipeList;
  
  public RecipeBlog(ArrayList<Recipe> recipeList) {
    this.recipeList = recipeList;
  }
  
  public String getRecommendedRecipes(String dietaryPreference) {
    ArrayList<Recipe> filteredRecipes = filterByDiet(recipeList, dietaryPreference);
    return recipesToString(filteredRecipes);
  }

  private ArrayList<Recipe> filterByDiet(ArrayList<Recipe> recipes, String dietaryPreference) {
    ArrayList<Recipe> filteredRecipes = new ArrayList<Recipe>();
    
    for (Recipe recipe : recipes) {
      if (recipe.getDiet().equals(dietaryPreference)) {
        filteredRecipes.add(recipe);
      }
    }
    
    return filteredRecipes;
  }

  public String recipesToString(ArrayList<Recipe> recipes) {
    String result = "";

    for (Recipe recipe : recipes) {
      result += recipe + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Private Methods #2`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Recipe.java`,text:`public class Recipe {

  private String name;
  private String diet;

  public Recipe(String name, String diet) {
    this.name = name;
    this.diet = diet;
  }

  public String getName() {
    return name;
  }

  public String getDiet() {
    return diet;
  }

  public String toString() {
    return name + " (" + diet + ")";
  }
  
}`},{path:`RecipeBlog.java`,text:`import java.util.ArrayList;

public class RecipeBlog {
  
  private ArrayList<Recipe> recipeList;
  
  public RecipeBlog(ArrayList<Recipe> recipeList) {
    this.recipeList = recipeList;
  }
  
  public String getRecommendedRecipes(String dietaryPreference) {
    ArrayList<Recipe> filteredRecipes = filterByDiet(recipeList, dietaryPreference);
    return recipesToString(filteredRecipes);
  }

  private ArrayList<Recipe> filterByDiet(ArrayList<Recipe> recipes, String dietaryPreference) {
    ArrayList<Recipe> filteredRecipes = new ArrayList<Recipe>();
    
    for (Recipe recipe : recipes) {
      if (recipe.getDiet().equals(dietaryPreference)) {
        filteredRecipes.add(recipe);
      }
    }
    
    return filteredRecipes;
  }

  public String recipesToString(ArrayList<Recipe> recipes) {
    String result = "";

    for (Recipe recipe : recipes) {
      result += recipe + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Private Methods #3`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Recipe.java`,text:`public class Recipe {

  private String name;
  private String diet;

  public Recipe(String name, String diet) {
    this.name = name;
    this.diet = diet;
  }

  public String getName() {
    return name;
  }

  public String getDiet() {
    return diet;
  }

  public String toString() {
    return name + " (" + diet + ")";
  }
  
}`},{path:`RecipeBlog.java`,text:`import java.util.ArrayList;

public class RecipeBlog {
  
  private ArrayList<Recipe> recipeList;
  
  public RecipeBlog(ArrayList<Recipe> recipeList) {
    this.recipeList = recipeList;
  }
  
  public String getRecommendedRecipes(String dietaryPreference) {
    ArrayList<Recipe> filteredRecipes = filterByDiet(recipeList, dietaryPreference);
    return recipesToString(filteredRecipes);
  }

  private ArrayList<Recipe> filterByDiet(ArrayList<Recipe> recipes, String dietaryPreference) {
    ArrayList<Recipe> filteredRecipes = new ArrayList<Recipe>();
    
    for (Recipe recipe : recipes) {
      if (recipe.getDiet().equals(dietaryPreference)) {
        filteredRecipes.add(recipe);
      }
    }
    
    return filteredRecipes;
  }

  public String recipesToString(ArrayList<Recipe> recipes) {
    String result = "";

    for (Recipe recipe : recipes) {
      result += recipe + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Private Methods (a) — Console`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of Soda objects
    Soda[] sodaList = {new Soda("Coca Cola", 5), new Soda("Sprite", 4), new Soda("Dr. Pepper", 2), new Soda("Spindrift", 3)};

    // Creates a SodaMachine object
    SodaMachine machine = new SodaMachine(sodaList, 1.50);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getTotalCost() method and print the result.
     * -----------------------------------------------------------------------------
     */



    

  }
}`},{path:`Soda.java`,text:`/*
 * Represents a soda
 */
public class Soda {

  private String name;    // The name of a soda
  private int quantity;   // The number of sodas in inventory

  /*
   * Sets name and quantity to the specified values
   */
  public Soda(String name, int quantity) {
    this.name = name;
    this.quantity = quantity;
  }

  /*
   * Returns the name of the soda
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the quantity of sodas
   */
  public int getQuantity() {
    return quantity;
  }

  /*
   * Returns a String containing the name of the soda and quantity
   */
  public String toString() {
    return name + " - " + quantity;
  }
  
}`},{path:`SodaMachine.java`,text:`/*
 * Represents a soda machine
 */
public class SodaMachine {

  private Soda[] inventory;    // The 1D array of sodas in the machine
  private double price;        // The price of each soda in the machine

  /*
   * Initializes inventory to the specified 1D array of sodas
   * in the machine and sets price to the specified price;
   */
  public SodaMachine(Soda[] inventory, double price) {
    this.inventory = inventory;
    this.price = price;
  }

  /*
   * Returns the 1D array of sodas in the machine
   */
  public Soda[] getInventory() {
    return inventory;
  }

  /*
   * Returns the price of sodas in the machine
   */
  public double getPrice() {
    return price;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method getTotalSodas() to get the total number of
   * sodas that are in the soda machine.
   * -----------------------------------------------------------------------------
   */




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method getTotalCost() to calculate and return the total
   * cost of all sodas in the machine.
   * -----------------------------------------------------------------------------
   */
  


  
  
}`}],validationFiles:[{path:`SodaMachineTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SodaMachine.java Test")
public class SodaMachineTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("getTotalSodas() is a private method => ")
  public void testPrivateMethod() {
    String message = "getTotalSodas() should be a private method." + messageGap;
      
    Soda[] testSodas = new Soda[]{new Soda("Coca Cola", 5), new Soda("Sprite", 4), new Soda("Dr. Pepper", 2), new Soda("Spindrift", 3)};
    SodaMachine testSodaMachine = new SodaMachine(testSodas, 1.50);
    Method actualMethod = getExpectedMethod(testSodaMachine, "getTotalSodas");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("getTotalSodas() returns the total number of sodas available in the inventory array => ")
  public void testGetTotalSodas() {
    String message = "getTotalSodas() should return the sum of all quantities of each soda in the inventory array." + messageGap;

    Soda[] testSodas = new Soda[]{new Soda("Coca Cola", 5), new Soda("Sprite", 4), new Soda("Dr. Pepper", 2), new Soda("Spindrift", 3)};
    SodaMachine testSodaMachine = new SodaMachine(testSodas, 1.50);
    Method actualMethod = getExpectedMethod(testSodaMachine, "getTotalSodas");

    int expected = 5 + 4 + 2 + 3;
    int actual = 0;

    try {
      actualMethod.setAccessible(true);
      actual = (int) actualMethod.invoke(testSodaMachine);
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("getTotalCost() is a public method => ")
  public void testPublicMethod() {
    String message = "getTotalCost() should be a public method." + messageGap;
      
    Soda[] testSodas = new Soda[]{new Soda("Coca Cola", 5), new Soda("Sprite", 4), new Soda("Dr. Pepper", 2), new Soda("Spindrift", 3)};
    SodaMachine testSodaMachine = new SodaMachine(testSodas, 1.50);
    Method actualMethod = getExpectedMethod(testSodaMachine, "getTotalCost");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  @Test
  @Order(4)
  @DisplayName("getTotalCost() calls the private method getTotalSodas() => ")
  public void testGetTotalCost() {
    String message = "The getTotalCost() method should call the getTotalSodas() method to calculate the total cost of all sodas in the machine.";
    message += "\\n        Return the result of the value returned from calling the getTotalSodas() method multiplied by the instance variable price."+ messageGap;
    
    Soda mockSoda = createMock(Soda.class);
    expect(mockSoda.getQuantity()).andReturn(2);
    expect(mockSoda.getQuantity()).andReturn(1);
    expect(mockSoda.getQuantity()).andReturn(3);
    replay(mockSoda);

    Soda[] testInventory = {mockSoda, mockSoda, mockSoda};
    SodaMachine testSodaMachine = new SodaMachine(testInventory, 1.25);
    double actual = testSodaMachine.getTotalCost();

    verify(mockSoda);

    double expected = (2 + 1 + 3) * 1.25;

    assertEquals(expected, actual, message);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Practice: Private Methods (b) — Console`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates several User objects
    User reparo = new User("scholarlyreparo", true);
    User protego = new User("inventiveprotego", false);
    User portus = new User("stylishportus", false);
    User lumos = new User("uniquelumos", true);

    // Creates a 1D array of User objects
    User[] appUsers = {reparo, protego, portus, lumos};

    // Creates a SocialMedia object
    SocialMedia app = new SocialMedia(appUsers);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the createPost() method to create posts for some of the User objects,
     * then print the publicPosts and hiddenPosts lists.
     * -----------------------------------------------------------------------------
     */




    

  }
}`},{path:`SocialMedia.java`,text:`import java.util.ArrayList;

/*
 * Represents a social media platform
 */
public class SocialMedia {

  private User[] profiles;                 // The 1D array of users on the social media service
  private ArrayList<String> publicPosts;   // The list of public posts
  private ArrayList<String> hiddenPosts;   // The list of hidden posts

  /*
   * Initializes profiles to the specified 1D array of users and
   * initializes publicPosts and hiddenPosts to empty lists
   */
  public SocialMedia(User[] profiles) {
    this.profiles = profiles;

    publicPosts = new ArrayList<String>();
    hiddenPosts = new ArrayList<String>();
  }

  /* 
   * Returns the 1D array of users
   */
  public User[] getProfiles() {
    return profiles;
  }

  /*
   * Returns the list of public posts as a String
   */
  public String getPublicPosts() {
    String result = "\\n----------PUBLIC POSTS----------\\n";
    
    for (String post : publicPosts) {
      result += post + "\\n";
    }

    return result;
  }

  /*
   * Returns the list of hidden posts as a String
   */
  public String getHiddenPosts() {
    String result = "\\n----------PRIVATE POSTS----------\\n";

    for (String post : hiddenPosts) {
      result += post + "\\n";
    }

    return result;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method markPost() to return a String with the post
   * containing the user's username and "[PRIVATE]" if their profile is private.
   * -----------------------------------------------------------------------------
   */
  



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method createPost() to add the marked post to publicPosts
   * if the user's profile is public or hiddenPosts if the user's profile is private.
   * -----------------------------------------------------------------------------
   */
  



  
  
}`},{path:`User.java`,text:`/*
 * Represents a user on a social media platform
 */
public class User {

  private String username;    // The user's username
  private boolean isPrivate;  // Whether or not the user's profile is private

  /*
   * Sets username and isPrivate to the specified values
   */
  public User(String username, boolean isPrivate) {
    this.username = username;
    this.isPrivate = isPrivate;
  }

  /*
   * Returns the user's username
   */
  public String getUsername() {
    return username;
  }

  /*
   * Returns the privacy status
   */
  public boolean getIsPrivate() {
    return isPrivate;
  }
  
}`}],validationFiles:[{path:`SocialMediaTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SocialMedia.java Test")
public class SocialMediaTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("markPost() is a private method => ")
  public void testPrivateMethod() {
    String message = "markPost() should be a private method." + messageGap;
    
    User[] testUsers = {new User("scholarlyreparo", true), new User("inventiveprotego", false),
                        new User("stylishportus", false), new User("uniquelumos", true)};
    SocialMedia testSocialMedia = new SocialMedia(testUsers);
    Method actualMethod = getExpectedMethod(testSocialMedia, "markPost");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("markPost() returns a String containing the username and message if their profile is public => ")
  public void testMarkPostPublicProfile() {
    String message = "markPost() should return a String containing the username and message if their profile is public." + messageGap;

    User[] testUsers = {new User("scholarlyreparo", true), new User("inventiveprotego", false),
                        new User("stylishportus", false), new User("uniquelumos", true)};
    SocialMedia testSocialMedia = new SocialMedia(testUsers);
    Method actualMethod = getExpectedMethod(testSocialMedia, "markPost");

    String expected = "inventiveprotego: some message";
    String actual = "";

    try {
      actualMethod.setAccessible(true);
      actual = (String) actualMethod.invoke(testSocialMedia, testUsers[1], "some message");
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("createPost() is a public method => ")
  public void testPublicMethod() {
    String message = "createPost() should be a public method." + messageGap;
      
    User[] testUsers = {new User("scholarlyreparo", true), new User("inventiveprotego", false),
                        new User("stylishportus", false), new User("uniquelumos", true)};
    SocialMedia testSocialMedia = new SocialMedia(testUsers);
    Method actualMethod = getExpectedMethod(testSocialMedia, "createPost");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  @Test
  @Order(4)
  @DisplayName("createPost() calls the private method markPost() and adds private posts to hiddenPosts => ")
  public void testCreatePostPrivatePosts() {
    String message = "The createPost() method should call the markPost() method to format the post with the username and whether or not it";
    message += "\\n        is a private post. If the user profile is private, the post should be added to hiddenPosts." + messageGap;
    
    User mockUser = createMock(User.class);
    expect(mockUser.getUsername()).andReturn("some user");
    expect(mockUser.getIsPrivate()).andReturn(true);
    expect(mockUser.getIsPrivate()).andReturn(true);
    replay(mockUser);

    User[] testUsers = {mockUser};
    SocialMedia testSocialMedia = new SocialMedia(testUsers);
    testSocialMedia.createPost(testUsers[0], "some message");

    verify(mockUser);

    String actual = testSocialMedia.getHiddenPosts();

    assertTrue(actual.contains("[PRIVATE]"), message);
  }

  @Test
  @Order(5)
  @DisplayName("createPost() calls the private method markPost() and adds public posts to publicPosts => ")
  public void testCreatePostPublicPosts() {
    String message = "The createPost() method should call the markPost() method to format the post with the username and whether or not it";
    message += "\\n        is a private post. If the user profile is public, the post should be added to publicPosts." + messageGap;
    
    User mockUser = createMock(User.class);
    expect(mockUser.getUsername()).andReturn("some user");
    expect(mockUser.getIsPrivate()).andReturn(false);
    expect(mockUser.getIsPrivate()).andReturn(false);
    replay(mockUser);

    User[] testUsers = {mockUser};
    SocialMedia testSocialMedia = new SocialMedia(testUsers);
    testSocialMedia.createPost(testUsers[0], "some message");

    verify(mockUser);

    String actual = testSocialMedia.getHiddenPosts();

    assertFalse(actual.contains("[PRIVATE]"), message);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Practice: Private Methods (c) — Console`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Creates an ArrayList of recipes
    ArrayList<Recipe> allRecipes = new ArrayList<Recipe>();
    allRecipes.add(new Recipe("Spaghetti Bolognese", FileReader.toStringList("recipe1.txt")));
    allRecipes.add(new Recipe("Chicken Curry", FileReader.toStringList("recipe2.txt")));
    allRecipes.add(new Recipe("Beef Stroganoff", FileReader.toStringList("recipe3.txt")));
    allRecipes.add(new Recipe("Vegetable Stir-Fry", FileReader.toStringList("recipe4.txt")));
    allRecipes.add(new Recipe("Salmon with Asparagus", FileReader.toStringList("recipe5.txt")));

    // Creates an ArrayList of preferred ingredients
    ArrayList<String> preferred = new ArrayList<String>();
    preferred.add("chicken");
    preferred.add("curry powder");
    preferred.add("coconut milk");

    // Creates a Website object
    Website recipeWebsite = new Website(allRecipes);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getRecommendedRecipes() method and print the results.
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

  public static ArrayList<String> toStringList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    return temp;
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

  public static ArrayList<Integer> toIntList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Integer> data = new ArrayList<Integer>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Integer.parseInt(currentValue));
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Double> data = new ArrayList<Double>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Double.parseDouble(currentValue));
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
  
}`},{path:`Recipe.java`,text:`import java.util.ArrayList;

/*
 * Represents a recipe
 */
public class Recipe {
  
  private String name;                        // The name of a recipe
  private ArrayList<String> ingredients;      // The list of ingredients

  /*
   * Sets name to the specified name and initializes
   * ingredients to the specified list of ingredients
   */
  public Recipe(String name, ArrayList<String> ingredients) {
    this.name = name;
    this.ingredients = ingredients;
  }

  /*
   * Returns the name of the recipe
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the list of ingredients
   */
  public ArrayList<String> getIngredients() {
    return ingredients;
  }

  /*
   * Returns a String containing the recipe's information
   */
  public String toString() {
    String result = name + "\\n----------\\n";

    for (String ingred : ingredients) {
      result += ingred + "\\n";
    }

    return result;
  }
  
}`},{path:`Website.java`,text:`import java.util.ArrayList;

/*
 * Represents a recipe website
 */
public class Website {

  private ArrayList<Recipe> recipes;    // The list of recipes

  /*
   * Initializes recipes to the specified list of recipes
   */
  public Website(ArrayList<Recipe> recipes) {
    this.recipes = recipes;
  }

  /*
   * Returns the list of recipes
   */
  public ArrayList<Recipe> getRecipes() {
    return recipes;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method resultsToString() with a parameter for an
   * ArrayList of Recipe objects. The method should return each recipe in the list
   * on separate lines.
   * -----------------------------------------------------------------------------
   */
  



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method getRecommendedRecipes() with a parameter for an
   * ArrayList of preferred ingredients. Find all recipes that contain a
   * preferred ingredient and add each recipe that matches to a new ArrayList.
   * Return a String containing the results using the resultsToString() method.
   * -----------------------------------------------------------------------------
   */
  


  
  
}`}],validationFiles:[{path:`WebsiteTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Website.java Test")
public class WebsiteTest {

  String messageGap = "\\n       ";
  ArrayList<Recipe> testRecipes;
  Website testWebsite;

  @BeforeEach
  public void setup() {
    testRecipes = new ArrayList<Recipe>();
    testRecipes.add(new Recipe("Spaghetti Bolognese", FileReader.toStringList("recipe1.txt")));
    testRecipes.add(new Recipe("Chicken Curry", FileReader.toStringList("recipe2.txt")));
    testRecipes.add(new Recipe("Beef Stroganoff", FileReader.toStringList("recipe3.txt")));
    testRecipes.add(new Recipe("Vegetable Stir-Fry", FileReader.toStringList("recipe4.txt")));
    testRecipes.add(new Recipe("Salmon with Asparagus", FileReader.toStringList("recipe5.txt")));
    testWebsite = new Website(testRecipes);
  }
   
  @Test
  @Order(1)
  @DisplayName("resultsToString() is a private method => ")
  public void testPrivateMethod() {
    String message = "resultsToString() should be a private method." + messageGap;
    Method actualMethod = getExpectedMethod(testWebsite, "resultsToString");
    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("resultsToString() returns a String containing each Recipe object's information on new lines => ")
  public void testResultsToString() {
    String message = "resultsToString() should traverse the ArrayList parameter and concatenate each Recipe object on a new line." + messageGap;
    Method actualMethod = getExpectedMethod(testWebsite, "resultsToString");

    String expected = getExpectedString(testRecipes);
    String actual = "";

    try {
      actualMethod.setAccessible(true);
      actual = (String) actualMethod.invoke(testWebsite, testRecipes);
    } catch (Exception e) { }

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("getRecommendedRecipes() is a public method => ")
  public void testPublicMethod() {
    String message = "getRecommendedRecipes() should be a public method." + messageGap;
    Method actualMethod = getExpectedMethod(testWebsite, "getRecommendedRecipes");
    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  @Test
  @Order(4)
  @DisplayName("getRecommendedRecipes() calls the private method resultsToString() => ")
  public void testGetRecommendedRecipes() {
    String message = "The getRecommendedRecipes() should find all recipes that contain ingredients that are the same as the";
    message += "\\n        an ingredient in the preferred ingredients list provided as the parameter. It should call the";
    message += "\\n        resultsToString() method to return the recipes found as a String." + messageGap;

    ArrayList<String> testPreferred = new ArrayList<String>();
    testPreferred.add("mushrooms");
    testPreferred.add("onion");
    
    String expected = getExpectedResults(testRecipes, testPreferred);
    String actual = testWebsite.getRecommendedRecipes(testPreferred);

    assertEquals(expected, actual, message);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }

  private String getExpectedString(ArrayList<Recipe> testList) {
    String result = "";

    for (Recipe recipe : testList) {
      result += recipe + "\\n";
    }

    return result;
  }

  private String getExpectedResults(ArrayList<Recipe> recipes, ArrayList<String> preferredIngredients) {
    ArrayList<Recipe> filteredRecipes = new ArrayList<Recipe>();
    
    for (Recipe recipe : recipes) {
      ArrayList<String> recipeIngredients = recipe.getIngredients();
      boolean containsPreferredIngredients = false;
      
      for (String preferred : preferredIngredients) {
        for (String ingredient : recipeIngredients) {
          if (preferred.equals(ingredient)) {
            containsPreferredIngredients = true;
          }
        }
      }
      
      if (containsPreferredIngredients) {
        filteredRecipes.add(recipe);
      }
    }
    
    return getExpectedString(filteredRecipes);
  }
  
}`}],dataFiles:[{path:`recipe1.txt`,text:`spaghetti
ground beef
tomato sauce
onion
garlic`},{path:`recipe2.txt`,text:`chicken
curry powder
coconut milk
onion
garlic`},{path:`recipe3.txt`,text:`beef
sour cream
mushrooms
onion
noodles`},{path:`recipe4.txt`,text:`vegetables
soy sauce
ginger
garlic`},{path:`recipe5.txt`,text:`salmon
asparagus
lemon
garlic`}]},{name:`Practice: Private Methods (d) — Console`,lesson:`Lesson 4: Private Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates a User object
    User amelia = new User();

    // Creates a Video object
    Video innovators = new Video("The Innovators: Women in Tech", 3360);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the addViewing() method and print the User object.
     * -----------------------------------------------------------------------------
     */
    


    

  }
}`},{path:`User.java`,text:`import java.util.ArrayList;

/*
 * Represents a user on a video streaming website
 */
public class User {

  private ArrayList<Video> history;     // The list of videos a user has viewed

  /*
   * Initializes history to an empty list
   */
  public User() {
    this.history = new ArrayList<Video>();
  }

  /*
   * Returns the list of videos the user has viewed
   */
  public ArrayList<Video> getHistory() {
    return history;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method hasWatchedMinimum() with a parameter for the
   * number of seconds that returns true if the number of seconds is greater than
   * 30, otherwise returns false.
   * -----------------------------------------------------------------------------
   */
  


  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method addViewing with a parameter for a Video object that
   * adds a new Video object to the history list if hasWatchedMinimum() is true.
   * -----------------------------------------------------------------------------
   */
  


  

  /*
   * Returns a String containing the videos the user has viewed
   */
  public String toString() {
    String result = "";

    for (Video video : history) {
      result += video + "\\n";
    }

    return result;
  }
  
}`},{path:`Video.java`,text:`/*
 * Represents a video on a video streaming website
 */
public class Video {

  private String title;          // The title of a video
  private int length;            // The length of a video in seconds
  private int watched;           // The number of seconds a video was watched

  /*
   * Sets title and length to the specified values,
   * and sets watched to 0
   */
  public Video(String title, int length) {
    this.title = title;
    this.length = length;
  }

  /*
   * Returns the title of the video
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the length of the video in seconds
   */
  public int getLength() {
    return length;
  }

  /*
   * Returns the number of seconds the video was watched
   */
  public int getWatched() {
    return watched;
  }

  /*
   * Sets the number of seconds the video was watched
   */
  public void setWatched(int numSeconds) {
    watched = numSeconds;
  }

  /*
   * Returns a String containing the video's information
   */
  public String toString() {
    return title + " - " + length + " seconds (" + watched + " seconds watched)";
  }
  
}`}],validationFiles:[{path:`UserTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("User.java Test")
public class UserTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("hasWatchedMinimum() is a private method => ")
  public void testPrivateMethod() {
    String message = "hasWatchedMinimum() should be a private method." + messageGap;

    User testUser = new User();
    Method actualMethod = getExpectedMethod(testUser, "hasWatchedMinimum");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("hasWatchedMinimum() returns true if the number of seconds is greater than 30 => ")
  public void testHasWatchedMinimumReturnsTrue() {
    String message = "hasWatchedMinimum() should return true if the parameter for the number of seconds is greater than 30." + messageGap;

    User testUser = new User();
    Method actualMethod = getExpectedMethod(testUser, "hasWatchedMinimum");
    boolean actual = false;

    try {
      actualMethod.setAccessible(true);
      actual = (boolean) actualMethod.invoke(testUser, 40);
    } catch (Exception e) { }

    assertTrue(actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("hasWatchedMinimum() returns false if the number of seconds is less than 30 => ")
  public void testHasWatchedMinimumReturnsFalse() {
    String message = "hasWatchedMinimum() should return false if the parameter for the number of seconds is less than 30." + messageGap;

    User testUser = new User();
    Method actualMethod = getExpectedMethod(testUser, "hasWatchedMinimum");
    boolean actual = false;

    try {
      actualMethod.setAccessible(true);
      actual = (boolean) actualMethod.invoke(testUser, 10);
    } catch (Exception e) { }

    assertFalse(actual, message);
  }

  @Test
  @Order(4)
  @DisplayName("addViewing() is a public method => ")
  public void testPublicMethod() {
    String message = "addViewing() should be a public method." + messageGap;
    
    User testUser = new User();
    Method actualMethod = getExpectedMethod(testUser, "addViewing");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  @Test
  @Order(5)
  @DisplayName("addViewing() calls the private method hasWatchedMinimum() => ")
  public void testAddViewing() {
    String message = "The addViewing() method should call the hasWatchedMinimum() method to determine if the number of seconds is greater than";
    message += "\\n        30. If so, create a new Video object and set its watched instance variable to the number of seconds. Add the";
    message += "\\n        new Video object to the history list." + messageGap;

    User testUser = new User();
    Video testVideo = new Video("The Innovators: Women in Tech", 3360);
    testUser.addViewing(testVideo, 40);
    ArrayList<Video> actual = testUser.getHistory();

    assertEquals("The Innovators: Women in Tech", actual.get(0).getTitle(), message);
    assertEquals(3360, actual.get(0).getLength(), message);
    assertEquals(40, actual.get(0).getWatched(), message);
    assertNotEquals(testVideo, actual.get(0), message);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Practice: Private Methods (a) — Theater`,lesson:`Lesson 4: Private Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of quote images
    String[] quotes = {"make-it-happen.png", "make-today-amazing.png",
                       "stay-positive.png", "youve-got-this.png"};

    // Creates a QuoteScene object
    QuoteScene scene = new QuoteScene(quotes);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawScene() method.
     * -----------------------------------------------------------------------------
     */




    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`QuoteScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that displays motivational quotes
 */
public class QuoteScene extends Scene {

  private String[] quotes;   // The 1D array of quote images

  /*
   * Initializes quotes to the specified 1D array of quote images
   */
  public QuoteScene(String[] quotes) {
    this.quotes = quotes;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method animateQuote() to animate a quote image by
   * increasing its size by a set amount.
   * -----------------------------------------------------------------------------
   */
  




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method drawScene() to traverse the quotes array and
   * animate each quote image.
   * -----------------------------------------------------------------------------
   */
  




  
  
}`}],validationFiles:[{path:`QuoteSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("QuoteScene.java Test")
public class QuoteSceneTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("animateQuote() is a private method => ")
  public void testPrivateMethod() {
    String message = "animateQuote() should be a private method." + messageGap;
      
    String[] testQuotes = {"make-it-happen.png", "make-today-amazing.png", "stay-positive.png", "youve-got-this.png"};
    QuoteScene testScene = new QuoteScene(testQuotes);
    Method actualMethod = getExpectedMethod(testScene, "animateQuote");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("drawScene() is a public method => ")
  public void testPublicMethod() {
    String message = "drawScene() should be a public method." + messageGap;
      
    String[] testQuotes = {"make-it-happen.png", "make-today-amazing.png", "stay-positive.png", "youve-got-this.png"};
    QuoteScene testScene = new QuoteScene(testQuotes);
    Method actualMethod = getExpectedMethod(testScene, "drawScene");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Practice: Private Methods (b) — Theater`,lesson:`Lesson 4: Private Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 1D array of image files
    String[] images = {"laptop.jpg", "dog.jpg", "nature.jpg", "paintbrushes.jpg"};

    // Creates an ImageScene object
    ImageScene scene = new ImageScene(images);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the drawScene() method.
     * -----------------------------------------------------------------------------
     */




    
    
    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`ImageScene.java`,text:`import org.code.theater.*;

/*
 * Represents a scene that displays images
 */
public class ImageScene extends Scene {

  private String[] images;   // The 1D array of image files

  /*
   * Sets images to the specified 1D array of image files
   */
  public ImageScene(String[] images) {
    this.images = images;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method drawTransition() to animate a drawing of a circle.
   * -----------------------------------------------------------------------------
   */
  




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method drawScene() to draw each image with a transition
   * in between each image.
   * -----------------------------------------------------------------------------
   */
  




  

}`}],validationFiles:[{path:`ImageSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("ImageScene.java Test")
public class ImageSceneTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("drawTransition() is a private method => ")
  public void testPrivateMethod() {
    String message = "drawTransition() should be a private method." + messageGap;
      
    String[] testImages = {"laptop.jpg", "dog.jpg", "nature.jpg", "paintbrushes.jpg"};
    ImageScene testScene = new ImageScene(testImages);
    Method actualMethod = getExpectedMethod(testScene, "drawTransition");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("drawScene() is a public method => ")
  public void testPublicMethod() {
    String message = "drawScene() should be a public method." + messageGap;
      
    String[] testImages = {"laptop.jpg", "dog.jpg", "nature.jpg", "paintbrushes.jpg"};
    ImageScene testScene = new ImageScene(testImages);
    Method actualMethod = getExpectedMethod(testScene, "drawScene");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Practice: Private Methods (c) — Theater`,lesson:`Lesson 4: Private Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;

public class Main {
  public static void main(String[] args) {

    // Creates a 2D array of chords
    int[][] chords = { {62, 66, 69}, {64, 68, 71}, {65, 69, 72}, {67, 71, 74},
                       {69, 73, 76}, {71, 75, 78}, {62, 65, 69}, {64, 67, 71} };

    // Creates a SongGenerator object
    SongGenerator scene = new SongGenerator(chords);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the generateSong() method.
     * -----------------------------------------------------------------------------
     */

    



    

    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`SongGenerator.java`,text:`import org.code.theater.*;
import org.code.media.*;

public class SongGenerator extends Scene {
  
  private int[][] chords;    // The 2D array of chords

  /*
   * Initializes chords to the specified 2D array of chords
   */
  public SongGenerator(int[][] chords) {
    this.chords = chords;
  }

  /*
   * Returns the 2D array of chords
   */
  public int[][] getChords() {
    return chords;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method generateChord() to return a random row from
   * the 2D array of chords.
   * -----------------------------------------------------------------------------
   */
  


  

  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method generateSong() to play five random chords.
   * -----------------------------------------------------------------------------
   */
  




  
  
}
`}],validationFiles:[{path:`SongGeneratorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SongGenerator.java Test")
public class SongGeneratorTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("generateChord() is a private method => ")
  public void testPrivateMethod() {
    String message = "generateChord() should be a private method." + messageGap;
      
    int[][] testChords = { {62, 66, 69}, {64, 68, 71}, {65, 69, 72}, {67, 71, 74},
                       {69, 73, 76}, {71, 75, 78}, {62, 65, 69}, {64, 67, 71} };
    SongGenerator testScene = new SongGenerator(testChords);
    Method actualMethod = getExpectedMethod(testScene, "generateChord");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("generateSong() is a public method => ")
  public void testPublicMethod() {
    String message = "generateSong() should be a public method." + messageGap;
      
    int[][] testChords = { {62, 66, 69}, {64, 68, 71}, {65, 69, 72}, {67, 71, 74},
                       {69, 73, 76}, {71, 75, 78}, {62, 65, 69}, {64, 67, 71} };
    SongGenerator testScene = new SongGenerator(testChords);
    Method actualMethod = getExpectedMethod(testScene, "generateSong");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Practice: Private Methods (d) — Theater`,lesson:`Lesson 4: Private Methods`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`import org.code.theater.*;
import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Creates an ArrayList of characters
    ArrayList<Character> characters = new ArrayList<Character>();
    characters.add(new Character("Adventurer", "zelda.jpg"));
    characters.add(new Character("Warrior", "kratos.png"));
    characters.add(new Character("Soldier", "masterchief.jpg"));
    characters.add(new Character("Princess", "princesspeach.jpg"));
    characters.add(new Character("Hunter", "aloy.png"));
    characters.add(new Character("Survivor", "ellie.jpg"));

    // Creates a CharacterScene object
    CharacterScene scene = new CharacterScene(characters);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the getRecommendedCharacters() method.
     * -----------------------------------------------------------------------------
     */




    
    
    // Plays the scene
    Theater.playScenes(scene);
    
  }
}`},{path:`Character.java`,text:`/*
 * Represents a character
 */
public class Character {

  private String characterClass;    // The class of a character
  private String image;             // The image file for a character

  /*
   * Sets characterClass and image to the specified values
   */
  public Character(String characterClass, String image) {
    this.characterClass = characterClass;
    this.image = image;
  }

  /*
   * Returns the class of the character
   */
  public String getCharacterClass() {
    return characterClass;
  }

  /*
   * Returns the image file for the character
   */
  public String getImage() {
    return image;
  }
  
}`},{path:`CharacterScene.java`,text:`import org.code.theater.*;
import java.util.ArrayList;

/*
 * Creates a scene to display characters that match a player's style
 */
public class CharacterScene extends Scene {
  
  private ArrayList<Character> characters;    // The list of available characters

  /*
   * Initializes characters to the specified list of available characters
   */
  public CharacterScene(ArrayList<Character> characters) {
    this.characters = characters;
  }

  /*
   * Returns the list of available characters
   */
  public ArrayList<Character> getCharacters() {
    return characters;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the private method filterByPlayStyle() to return a list of characters
   * whose character class matches the first play style or the second play style.
   * -----------------------------------------------------------------------------
   */
  




  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write the public method getRecommendedCharacters() to call the
   * filterByPlayStyle() method and draw each character's image and character class
   * in the scene.
   * -----------------------------------------------------------------------------
   */
  




  

}`}],validationFiles:[{path:`CharacterSceneTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("CharacterScene.java Test")
public class CharacterSceneTest {

  String messageGap = "\\n       ";
   
  @Test
  @Order(1)
  @DisplayName("filterByPlayStyle() is a private method => ")
  public void testPrivateMethod() {
    String message = "filterByPlayStyle() should be a private method." + messageGap;
      
    ArrayList<Character> testCharacters = new ArrayList<Character>();
    testCharacters.add(new Character("Adventurer", "zelda.jpg"));
    testCharacters.add(new Character("Warrior", "kratos.png"));
    testCharacters.add(new Character("Soldier", "masterchief.jpg"));
    testCharacters.add(new Character("Princess", "princesspeach.jpg"));
    testCharacters.add(new Character("Hunter", "aloy.png"));
    testCharacters.add(new Character("Survivor", "ellie.jpg"));

    CharacterScene testScene = new CharacterScene(testCharacters);
    Method actualMethod = getExpectedMethod(testScene, "filterByPlayStyle");

    assertTrue(actualMethod.getModifiers() == Modifier.PRIVATE);
  }

  @Test
  @Order(2)
  @DisplayName("getRecommendedCharacters() is a public method => ")
  public void testPublicMethod() {
    String message = "getRecommendedCharacters() should be a public method." + messageGap;
      
    ArrayList<Character> testCharacters = new ArrayList<Character>();
    testCharacters.add(new Character("Adventurer", "zelda.jpg"));
    testCharacters.add(new Character("Warrior", "kratos.png"));
    testCharacters.add(new Character("Soldier", "masterchief.jpg"));
    testCharacters.add(new Character("Princess", "princesspeach.jpg"));
    testCharacters.add(new Character("Hunter", "aloy.png"));
    testCharacters.add(new Character("Survivor", "ellie.jpg"));

    CharacterScene testScene = new CharacterScene(testCharacters);
    Method actualMethod = getExpectedMethod(testScene, "getRecommendedCharacters");

    assertTrue(actualMethod.getModifiers() == Modifier.PUBLIC);
  }

  private Method getExpectedMethod(Object testObject, String methodName) {
    Class testClass = testObject.getClass();
    Method[] testClassMethods = testClass.getDeclaredMethods();
    Method actualMethod = null;

    for (Method method : testClassMethods) {
      if (method.getName().equals(methodName)) {
        actualMethod = method;
      }
    }

    return actualMethod;
  }
  
}`}],dataFiles:[]},{name:`Creative Coding with The Theater Project`,lesson:`Lesson 7: Project Development`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static ArrayList<String> toStringList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    return temp;
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

  public static ArrayList<Integer> toIntList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Integer> data = new ArrayList<Integer>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Integer.parseInt(currentValue));
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Double> data = new ArrayList<Double>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Double.parseDouble(currentValue));
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
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with The Theater Project`,lesson:`Lesson 9a: Creative Coding with The Theater Project`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static ArrayList<String> toStringList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    return temp;
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

  public static ArrayList<Integer> toIntList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Integer> data = new ArrayList<Integer>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Integer.parseInt(currentValue));
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Double> data = new ArrayList<Double>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Double.parseDouble(currentValue));
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
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with The Theater Project`,lesson:`Lesson 9b: Creative Coding with The Theater Project [1-Day Version]`,view:`theater`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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

  public static ArrayList<String> toStringList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    return temp;
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

  public static ArrayList<Integer> toIntList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Integer> data = new ArrayList<Integer>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Integer.parseInt(currentValue));
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> temp = readDataFromFile(filename);
    ArrayList<Double> data = new ArrayList<Double>();

    for (int index = 0; index < temp.size(); index++) {
      String currentValue = temp.get(index);
      data.add(Double.parseDouble(currentValue));
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