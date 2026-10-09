var e=[{name:`Creative Coding with the Console Example Projects (a)`,lesson:`Lesson 1: Project Planning`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Customer myCustomer = new Customer();
    Chipotle myChipotle = new Chipotle(myCustomer, "menu.txt");
    
    myChipotle.serveCustomer();
    
  }
}`},{path:`BowlOrder.java`,text:`public class BowlOrder extends Order {  
  public String getType() {
    return "Burrito Bowl";
  }
}`},{path:`Chipotle.java`,text:`import java.util.ArrayList;

public class Chipotle {

  private Customer currentCustomer;
  private String menuFile;
  private static ArrayList<String> menuOptions;
  private ArrayList<String> optionTypes;

  public Chipotle(Customer currentCustomer, String menuFile) {
    this.currentCustomer = currentCustomer;
    this.menuFile = menuFile;
    menuOptions = FileReader.toStringList(menuFile);
    this.optionTypes = setOptionTypes();
  }

  public String getMenuFile() {
    return menuFile;
  }

  public void setMenuFile(String newMenuFile) {
    this.menuFile = newMenuFile;
  }

  public static ArrayList<String> getMenuOptions() {
    return menuOptions;
  }

  public void greetCustomer() {
    System.out.print("Welcome to Chipotle! What is your name? ");
    String customerName = currentCustomer.giveName();
    System.out.println("\\nWelcome " + customerName + "!");
  }

  public void createCustomerOrder() {
    System.out.print("\\nWould you like a burrito (1) or a burrito bowl (2)? ");
    String item = currentCustomer.getItemSelection();
    Order tempOrder;
    if (item.equals("bowl")) {
        tempOrder = new BowlOrder();
    }
    else if (item.equals("discount")) {
        tempOrder = new DiscountOrder();
    }
    else {
        tempOrder = new Order();
    }
    currentCustomer.setOrder(tempOrder);
  }

  public void getCustomerSelection() {
    for (String currentType : optionTypes) {
      System.out.println("\\nChoose your " + currentType + ":");
      showChoices(currentType);

      if (currentType.equals("protein") && currentCustomer.getOrder().canAddProtein()) {
        System.out.println("\\nYou can choose up to 2 options for protein.");
        System.out.print("Would you like to add an additional protein? ");
        getAdditionalChoice(currentType);
      }

      if (currentType.equals("toppings")) {
        System.out.print("\\nAny additional toppings? ");
        getAdditionalChoice(currentType);
      }
    }
  }

  public void showChoices(String currentType) {
    String choicesAndPrices = Option.getChoicesAndPrices(currentType);
    System.out.println(choicesAndPrices);
    System.out.print("Which " + currentType + " would you like? ");
    getChoice(choicesAndPrices, currentType);
  }

  public void getChoice(String choicesAndPrices, String currentType) {
    String selection = currentCustomer.makeOptionSelection(currentType);
    verifySelection(choicesAndPrices, selection, currentType);
  }

  public void getAdditionalChoice(String currentType) {
    boolean wantMore = currentCustomer.getAnswer();

    if (wantMore && currentType.equals("protein")) {
      currentCustomer.getOrder().addProtein();
      showChoices(currentType);
    }
    
    while (wantMore && currentType.equals("toppings")) {
      showChoices(currentType);
      System.out.print("Any additional toppings? ");
      wantMore = currentCustomer.getAnswer();
    }
  }

  public void getCharitySelection(double currentTotal) {
    System.out.print("\\nWould you like to round up for charity? ");
    boolean roundUp = currentCustomer.getAnswer();

    if (roundUp) {
      double newTotal = currentCustomer.getOrder().calculateNewTotal(currentTotal);
      System.out.println("Your new total is " + String.format("%.2f", newTotal) + ".");
    }
  }

  public void displayOrder() {
    System.out.print("\\n" + currentCustomer.getName() + "'s " + currentCustomer.getOrder());
    double customerTotal = currentCustomer.getOrder().calculateTotal();
    System.out.println("\\nYour total is " + String.format("%.2f", customerTotal) + ".");
    getCharitySelection(customerTotal);
  }

  public void serveCustomer() {
    greetCustomer();
    createCustomerOrder();
    getCustomerSelection();
    displayOrder();
  }

  private void verifySelection(String choicesAndPrices, String selection, String type) {
    if (choicesAndPrices.indexOf(selection) < 0) {
      System.out.println("Sorry! Your selection was invalid. Please try again.");
    }
    else {
      double selectionPrice = Option.getPriceForChoice(selection, type);
      Option tempOption = new Option(selection, selectionPrice);
      currentCustomer.getOrder().addToOrder(tempOption);
    }
  }

  private ArrayList<String> setOptionTypes() {
    ArrayList<String> tempTypesList = new ArrayList<String>();

    for (String currentOption : menuOptions) {
      String currentType = currentOption.substring(0, currentOption.indexOf("/"));
      tempTypesList.add(currentType);
    }

    removeDuplicates(tempTypesList);
    return tempTypesList;
  }

  private void removeDuplicates(ArrayList<String> originalList) {
    for (int start = 0; start < originalList.size(); start++) {
      String currentItem = originalList.get(start);

      for (int next = start + 1; next < originalList.size(); next++) {
        String nextItem = originalList.get(next);

        if (currentItem.equals(nextItem)) {
          originalList.remove(nextItem);
          next--;
        }
      }
    }
  }
  
}`},{path:`Customer.java`,text:`import java.util.Scanner;

public class Customer {

  private String name;
  private Order customerOrder;
  private Scanner input;

  public Customer() {
    this.name = null;
    this.customerOrder = null;
    this.input = new Scanner(System.in);
  }

  public String getName() {
    return name;
  }

  public String giveName() {
      this.name = input.nextLine();
      return name;
  }

  public Order getOrder() {
    return customerOrder;
  }

  public void setOrder(Order newOrder) {
    this.customerOrder = newOrder;
  }

  public String getItemSelection() {
    String item = input.nextLine();
    String result = "";

    if (item.equals("burrito bowl") || item.equals("2")) {
      result = "bowl";
    } else if (item.equals("COUPONCODE")) {
      result = "discount";
    }

    return result;
  }

  public String makeOptionSelection(String currentType) {
    String selection = input.nextLine();
    return selection;
  }

  public boolean getAnswer() {
    String selection = input.nextLine();
    boolean result = false;
  
    if (selection.toLowerCase().equals("yes") || selection.toLowerCase().equals("y")) {
      result = true;
    }
  
    return result;
  }

}`},{path:`DiscountOrder.java`,text:`public class DiscountOrder extends Order {  
  public double calculateTotal() {
    return super.calculateTotal() * .5;
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
  
}`},{path:`Option.java`,text:`import java.util.ArrayList;

public class Option {

  private String selectedChoice;
  private double selectedPrice;

  public Option(String selectedChoice, double selectedPrice) {
    this.selectedChoice = selectedChoice;
    this.selectedPrice = selectedPrice;
  }

  public String getSelectedChoice() {
    return selectedChoice;
  }

  public double getSelectedPrice() {
    return selectedPrice;
  }

  public static String getChoicesAndPrices(String typeToFind) {
    ArrayList<String> choices = getChoiceNames(typeToFind);
    ArrayList<Double> prices = getChoicePrices(typeToFind);
    
    String result = "";

    for (int index = 0; index < choices.size(); index++) {
      result += choices.get(index);

      if (prices.get(index) > 0) {
        result += " - " + prices.get(index);
      }

      result += "\\n";
    }

    return result;
  }

  public static double getPriceForChoice(String currentChoice, String typeToFind) {
    ArrayList<String> choiceNames = getChoiceNames(typeToFind);
    ArrayList<Double> choicePrices = getChoicePrices(typeToFind);
    
    double currentPrice = 0.00;

    for (int index = 0; index < choiceNames.size(); index++) {
      if (currentChoice.equals(choiceNames.get(index))) {
        currentPrice = choicePrices.get(index);
      }
    }

    return currentPrice;
  }

  private static ArrayList<String> getChoiceNames(String typeToFind) {
    ArrayList<String> menu = Chipotle.getMenuOptions();
    ArrayList<String> temp = new ArrayList<String>();

    for (int index = 0; index < menu.size(); index++) {
      String currentChoice = menu.get(index);

      if (isMatchingChoice(typeToFind, currentChoice)) {
        String currentName = currentChoice.substring(currentChoice.indexOf("/") + 1, currentChoice.indexOf(","));
        temp.add(currentName);
      }
    }

    return temp;
  }

  private static ArrayList<Double> getChoicePrices(String typeToFind) {
    ArrayList<String> menu = Chipotle.getMenuOptions();
    ArrayList<Double> temp = new ArrayList<Double>();

    for (int index = 0; index < menu.size(); index++) {
      String currentChoice = menu.get(index);

      if (isMatchingChoice(typeToFind, currentChoice)) {
        String currentPrice = currentChoice.substring(currentChoice.indexOf(",") + 1);
        temp.add(Double.parseDouble(currentPrice));
      }
    }

    return temp;
  }

  private static boolean isMatchingChoice(String typeToFind, String currentChoice) {
    boolean isMatching = false;
    String currentType = currentChoice.substring(0, currentChoice.indexOf("/"));

    if (currentType.equals(typeToFind)) {
      isMatching = true;
    }

    return isMatching;
  }
  
}`},{path:`Order.java`,text:`import java.util.ArrayList;

public class Order {

  private ArrayList<Option> selectedOptions;
  public static final int MAX_PROTEIN = 2;
  public static int currentProtein;

  public Order() {
    this.selectedOptions = new ArrayList<Option>();
    currentProtein = 0;
  }

  public void addProtein() {
    if (currentProtein < MAX_PROTEIN) {
      currentProtein++;
    }
  }

  public boolean canAddProtein() {
    return currentProtein < MAX_PROTEIN;
  }

  public void addToOrder(Option currentOption) {
    selectedOptions.add(currentOption);
  }

  public double calculateTotal() {
    double total = 0.00;

    for (Option currentOption : selectedOptions) {
        total += currentOption.getSelectedPrice();
    }

    total += getTax(total);

    return total;
  }

  public double getTax(double subtotal) {
    double taxRate = 0.06;
    return subtotal * taxRate;
  }

  public double calculateNewTotal(double currentTotal) {
    double newTotal = (int)(currentTotal + 0.5);
    return newTotal;
  }

  public String getType() {
    return "Burrito";
  }

  public String toString() {
    String result = getType();

    result += "\\n--------------------\\n";

    for (Option currentOption : selectedOptions) {
        result += currentOption.getSelectedChoice() + " (" + currentOption.getSelectedPrice() + ")" + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[{path:`menu.txt`,text:`protein/chicken,8.45
protein/steak,10.20
protein/barbacoa,10.20
protein/carnitas,9.10
protein/sofritas,8.45
protein/veggie,8.45
rice/white rice,0.00
rice/brown rice,0.00
rice/none,0.00
beans/black beans,0.00
beans/pinto beans,0.00
beans/none,0.00
toppings/guacamole,2.65
toppings/tomato salsa,0.00
toppings/roasted chili-corn salsa,0.00
toppings/tomatillo-green chili salsa,0.00
toppings/tomatillo-red chili salsa,0.00
toppings/sour cream,0.00
toppings/fajita veggies,0.00
toppings/cheese,0.00
toppings/romaine lettuce,0.00
toppings/queso blanco,1.55
toppings/none,0.00`}]},{name:`Creative Coding with the Console Example Projects (b)`,lesson:`Lesson 1: Project Planning`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    BandArtist oneDirection = new BandArtist("One Direction", 40098648,
      new String[]{"Zayn", "Niall", "Louis", "Liam", "Harry"});
    SoloArtist zayn = new SoloArtist("Zayn Malik", 19218530);
    SoloArtist niall = new SoloArtist("Niall Horan", 14067863);
    SoloArtist louis = new SoloArtist("Louis Tomlinson", 3660858);
    SoloArtist liam = new SoloArtist("Liam Payne", 6067392);
    SoloArtist harry = new SoloArtist("Harry Styles", 70632017);
    
    Artist[] allArtists = {oneDirection, zayn, niall, louis, liam, harry};
    

    System.out.println("---------- Before Sort ----------");
    for (Artist a : allArtists) {
      System.out.println(a);
    }

    System.out.println("---------- After Sort ----------");
    Artist.sortMonthlyListeners(allArtists);
    for (Artist a : allArtists) {
      System.out.println(a);
    }
    
  }
}`},{path:`Artist.java`,text:`public class Artist {

  private String name;
  private int monthlylisteners;

  public Artist(String name, int monthlylisteners) {
    this.name = name;
    this.monthlylisteners = monthlylisteners;
  }

  public String getName() {
    return name;
  }

  public int getMonthlyListeners() {
    return monthlylisteners;
  }
  
  public String toString() {
    return name + "'s monthly listeners on Spotify: " +  String.format("%,d", monthlylisteners);
  }

  public static void sortMonthlyListeners(Artist[] artistArray) {
    for (int index = 0; index < artistArray.length; index++) {
      int current = index;
      int currentMonthlyListeners = artistArray[current].getMonthlyListeners();

      for (int next = index + 1; next < artistArray.length; next++) {
        int nextMonthlyListeners = artistArray[next].getMonthlyListeners();

        if (nextMonthlyListeners > currentMonthlyListeners) {
          current = next;
          currentMonthlyListeners = artistArray[current].getMonthlyListeners();
        }
      }

      Artist temp = artistArray[current];
      artistArray[current] = artistArray[index];
      artistArray[index] = temp;
    }
  }
  
}`},{path:`BandArtist.java`,text:`public class BandArtist extends Artist {

  private String[] members;

  public BandArtist(String name, int monthlylisteners, String[] members) {
    super(name, monthlylisteners);
    this.members = members;
  }

  public String[] getMembers() {
    return members;
  }

  public String toString() {
    String result = "";
    
    if (members.length > 0) {
      result += members[0];
    }
    for (int i = 1; i < members.length; i++) {
      result += ", " + members[i];
    }

    return getName() + "'s (" + result + ") monthly listeners on Spotify: " +  String.format("%,d", getMonthlyListeners());
  }
  
}`},{path:`SoloArtist.java`,text:`public class SoloArtist extends Artist {

  public SoloArtist(String name, int monthlylisteners) {
    super(name, monthlylisteners);
  }
  
  public String toString() {
    return getName() + "'s (solo) monthly listeners on Spotify: " +  String.format("%,d", getMonthlyListeners());
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with the Console Example Projects (c)`,lesson:`Lesson 1: Project Planning`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Game myGame = new Game();
    myGame.playGame();
    
  }
}`},{path:`Enemy.java`,text:`import java.util.ArrayList;

public class Enemy extends Player {

  public Enemy(String name, int health) {
    super(name, health);
  }

  public void move(Player mainPlayer) {
    System.out.println("Computer's turn . . .\\n");
    String randomMove = getRandomMove();
    System.out.println(makeMove(randomMove, mainPlayer));
  }

  private String getRandomMove() {
    ArrayList<String> moves = getMoves();
    int randomIndex = (int)(Math.random() * moves.size());
    return moves.get(randomIndex);
  }
  
}`},{path:`Game.java`,text:`import java.util.Scanner;

public class Game {

  private Player mainPlayer;
  private Enemy enemyPlayer;
  private Player[] players;
  private Enemy[] enemies;

  public Game() {
    this.players = createPlayerOptions();
    this.enemies = createEnemies();

    this.mainPlayer = choosePlayer();
    this.enemyPlayer = enemies[0];
  }

  public Player[] createPlayerOptions() {
    Player[] playerOptions = {new Spiderman(100), new Jedi(100), new Pikachu(100)};
    return playerOptions;
  }

  public String showPlayers() {
    String playersText = "";

    for (Player playerOption : players) {
      playersText += playerOption.getName() + "\\n";
    }

    return playersText;
  }

  public Player choosePlayer() {
    String selection = promptForPlayer();
    Player selectedPlayer = null;

    while (selectedPlayer == null) {
      selectedPlayer = getSelectedPlayer(selection);
      
      if (!isValidPlayer(selectedPlayer)) {
        selection = promptForPlayer();
      }
    }

    return selectedPlayer;
  }

  public Player getSelectedPlayer(String selection) {
    Player chosenPlayer = null;

    for (Player playerOption : players) {
      if (selection.toLowerCase().equals(playerOption.getName().toLowerCase())) {
        chosenPlayer = playerOption;
      }
    }

    return chosenPlayer;
  }

  public Enemy[] createEnemies() {
    Enemy[] enemyOptions = {new Voldemort(100), new Thanos(100)};
    return enemyOptions;
  }

  public void printGameStatus() {
    System.out.println("\\n--------------------\\n");
    System.out.println(mainPlayer);
    System.out.println(enemyPlayer);
    System.out.println("-------------------\\n");
  }

  public void playGame() {
    System.out.println("\\nIt's " + mainPlayer.getName() + " against " + enemyPlayer.getName() + "!\\n");

    while (mainPlayer.getHealth() > 0 && enemyPlayer.getHealth() > 0) {
      playerMove();
      enemyMove();
    }

    checkWinner();
  }

  public void playerMove() {
    if (canMove(mainPlayer)) {
      mainPlayer.move(enemyPlayer);
      printGameStatus();
    }
  }

  public void enemyMove() {
    if (enemyPlayer.getHealth() > 0 && canMove(enemyPlayer)) {
      enemyPlayer.move(mainPlayer);
      printGameStatus();
    }
  }

  public boolean canMove(Player currentPlayer) {
    boolean status = true;

    if (currentPlayer.isStunned()) {
      status = false;
      System.out.println(currentPlayer.getName() + " cannot move.\\n");
      currentPlayer.setStun(false);
    }

    return status;
  }

  public void checkWinner() {
    if (mainPlayer.getHealth() < 0) {
      System.out.println(enemyPlayer.getName() + " wins");
    }
    else {
      System.out.println(mainPlayer.getName() + " wins!");
      nextLevel();
    }
  }

  public void nextLevel() {
    mainPlayer.increaseLevel();

    if (mainPlayer.getLevel() == 2) {
      System.out.println("Leveling up . . .");
      enemyPlayer = enemies[1];
      playGame();
    }
    else {
      System.out.println("You defeated all of the enemies!");
    }
  }

  private String promptForPlayer() {
    Scanner input = new Scanner(System.in);
    System.out.println("Which character do you want to play as? ");
    System.out.print(showPlayers());
    System.out.print("> ");
    String selection = input.nextLine();
    return selection;
  }

  private boolean isValidPlayer(Player selectedPlayer) {
    boolean result = true;

    if (selectedPlayer == null) {
      System.out.println("\\nNot a valid player! Try again.\\n");
      result = false;
    }

    return result;
  }
  
}`},{path:`Jedi.java`,text:`public class Jedi extends Player {

  private int specialMove;
  
  public Jedi(int health) {
    super("Jedi", health);
    this.specialMove = 2;
    createJediMoves();
  }

  public void createJediMoves() {
    removeMove("attack");
    addMove("lightsaber slash");
    addMove("force whirlwind");
    addMove("battlemind");
  }

  public String lightsaberSlash(Player otherPlayer) {
    return attack(otherPlayer, 40);
  }

  public String forceWhirlwind(Player otherPlayer) {
    String moveMessage = "";
    moveMessage = attack(otherPlayer, 60);
    specialMove--;

    if (specialMove > 0) {
      moveMessage += "\\n" + specialMove + " time(s) left to use this move.";
    }
    else {
      removeMove("force whirlwind");
    }

    return moveMessage;
  }

  public String battlemind() {
    String moveMessage = "";

    setEnergy(getEnergy() + 0.2);
    addHealth(getHealth() + 10);

    moveMessage += "Meditation increases the concentration and willpower of " + getName() + ".\\n";
    moveMessage += getName() + " has increased energy for attacking and healing.";

    if (getEnergy() >= 1.8) {
      removeMove("battlemind");
    }

    return moveMessage;
  }

  public String makeMove(String selection, Player otherPlayer) {
    String moveMessage = "";

    moveMessage = super.makeMove(selection, otherPlayer);

    if (selection.equals("lightsaber slash")) {
      moveMessage = lightsaberSlash(otherPlayer);
    }
    
    if (selection.equals("force whirlwind")) {
      moveMessage = forceWhirlwind(otherPlayer);
    }

    if (selection.equals("battlemind")) {
      moveMessage = battlemind();
    }

    return moveMessage;
  }
  
}`},{path:`Pikachu.java`,text:`public class Pikachu extends Player {
  
  public Pikachu(int health) {
    super("Pikachu", health);
    createPikachuMoves();
  }

  public void createPikachuMoves() {
    removeMove("attack");
    addMove("thundershock");
    addMove("power up");
    addMove("discharge");
    addMove("quick attack");
  }

  public String quickAttack(Player otherPlayer) {
    return attack(otherPlayer);
  }

  public String thundershock(Player otherPlayer) {
    int damage = (int)(Math.random() * 40 + 10);
    return attack(otherPlayer, damage);
  }

  public String powerUp(Player otherPlayer) {
    String moveMessage = "";

    if (getEnergy() >= 1.8) {
      moveMessage = "Maximum energy reached.\\n";
    }
    else {
      moveMessage = "Pikachu powers up, gaining more energy!\\n";
    }

    setEnergy(getEnergy() + 0.5);
    moveMessage += getName() + " energy level: " + getEnergy();
    return moveMessage;
  }

  public String discharge(Player otherPlayer) {
    String moveMessage = "";
    int damage = (int)(Math.random() * 20 + 10);
    moveMessage = attack(otherPlayer, damage);
    moveMessage += getName() + "\\n's electric discharge paralyzed the enemy!";
    otherPlayer.setStun(true);
    return moveMessage;
  }

  public String makeMove(String selection, Player otherPlayer) {
    String moveMessage = "";

    moveMessage = super.makeMove(selection, otherPlayer);

    if (selection.equals("quick attack")) {
      moveMessage = quickAttack(otherPlayer);
    }
    
    if (selection.equals("thundershock")) {
      moveMessage = thundershock(otherPlayer);
    }

    if (selection.equals("power up")) {
      moveMessage = powerUp(otherPlayer);
    }

    if (selection.equals("discharge")) {
      moveMessage = discharge(otherPlayer);
    }

    return moveMessage;
  }
  
}`},{path:`Player.java`,text:`import java.util.ArrayList;
import java.util.Scanner;

public class Player {

  private String name;
  private int health;
  private double energy;
  private int level;
  private boolean stun;
  private ArrayList<String> moves;

  public Player(String name, int health) {
    this.name = name;
    this.health = health;
    this.energy = 1;
    this.level = 1;
    this.stun = false;
    this.moves = createMovesList();
  }

  public String getName() {
    return name;
  }

  public int getHealth() {
    return health;
  }

  public double getEnergy() {
    return energy;
  }

  public int getLevel() {
    return level;
  }

  public boolean isStunned() {
    return stun;
  }

  public ArrayList<String> getMoves() {
    return moves;
  }

  public void addMove(String newMove) {
    moves.add(newMove);
  }

  public void removeMove(String move) {
    moves.remove(move);
  }

  public void removeHealth(int damage) {
    this.health = this.health - damage;
  }

  public void addHealth(int healthToAdd) {
    this.health += healthToAdd;
  }

  public void setEnergy(double newEnergy) {
    this.energy = newEnergy;
  }

  public void increaseLevel() {
    this.level++;
  }

  public void setStun(boolean status) {
    this.stun = status;
  }

  public String showMoves() {
    String movesText = "";

    for (String move : moves) {
      movesText += move + "\\n";
    }

    return movesText;
  }

  public void move(Enemy enemyPlayer) {
    System.out.println("Choose one of " + name + "'s moves:");
    System.out.println(showMoves());
    String selection = promptForMove();
    
    while (!isValidMove(selection)) {
      System.out.println("Invalid move, try again!");
      selection = promptForMove();
    }

    System.out.println("\\n" + name + " : " + selection.toUpperCase());
    System.out.println(makeMove(selection, enemyPlayer));
  }

  public String makeMove(String selection, Player otherPlayer) {
    String moveMessage = "";

    if (selection.equals("attack")) {
      moveMessage = attack(otherPlayer);
    }
    
    if (selection.equals("heal")) {
      moveMessage = heal();
    }

    return moveMessage;
  }

  public String attack(Player otherPlayer) {
    int damage = (int)(Math.random() * 20 + 20);
    damage *= energy;
    otherPlayer.removeHealth(damage);
    return this.name + "'s attack did " + damage + " damage to " + otherPlayer.getName();
  }

  public String attack(Player otherPlayer, int damage) {
    damage *= energy;
    otherPlayer.removeHealth(damage);
    return this.name + "'s attack did " + damage + " damage to " + otherPlayer.getName();
  }

  public String heal() {
    int healthToAdd = (int)(Math.random() * 20 + 20);
    healthToAdd *= energy;
    this.health += healthToAdd;

    if (health > 100) {
      this.health = 100;
    }
    
    return this.name + " healed " + healthToAdd;
  }

  private ArrayList<String> createMovesList() {
    ArrayList<String> tempList = new ArrayList<String>();

    tempList.add("attack");
    tempList.add("heal");

    return tempList;
  }

  public String toString() {
    return name + " health: " + health;
  }

  private boolean isValidMove(String selection) {
    boolean status = false;

    for (String move : moves) {
      if (selection.equals(move)) {
        status = true;
      }
    }

    return status;
  }

  private String promptForMove() {
    Scanner input = new Scanner(System.in);
    System.out.print("> ");
    String selection = input.nextLine();
    return selection;
  }
  
}`},{path:`Spiderman.java`,text:`public class Spiderman extends Player {

  private int specialMove;
  
  public Spiderman(int health) {
    super("Spiderman", health);
    this.specialMove = 3;
    createSpidermanMoves();
  }

  public void createSpidermanMoves() {
    removeMove("attack");
    addMove("web throw");
    addMove("web shooter");
    addMove("web attack");
  }

  public String webShooter(Player otherPlayer) {
    int damage = (int)(Math.random() * 10 + 5);
    String moveMessage = attack(otherPlayer, damage);
    moveMessage += "\\nSpiderman glued " + otherPlayer.getName() + " to the ground with web.";
    moveMessage += "\\n" + otherPlayer.getName() + " misses the next turn.";
    otherPlayer.setStun(true);
    return moveMessage;
  }

  public String webThrow(Player otherPlayer) {
    String moveMessage = "Spiderman yanks and throws " + otherPlayer.getName() + " into the ground.\\n";
    moveMessage += attack(otherPlayer, 40);
    specialMove--;
    moveMessage += "\\n" + specialMove + " time(s) left to use this move.";

    if (specialMove == 0) {
      removeMove("web throw");
    }

    return moveMessage;
  }

  public String webAttack(Player otherPlayer) {
    return attack(otherPlayer);
  }

  public String makeMove(String selection, Player otherPlayer) {
    String moveMessage = "";

    moveMessage = super.makeMove(selection, otherPlayer);

    if (selection.equals("web throw")) {
      moveMessage = webThrow(otherPlayer);
    }

    if (selection.equals("web shooter")) {
      moveMessage = webShooter(otherPlayer);
    }

    if (selection.equals("web attack")) {
      moveMessage = webAttack(otherPlayer);
    }

    return moveMessage;
  }
  
}`},{path:`Thanos.java`,text:`public class Thanos extends Enemy {
  
  public Thanos(int health) {
    super("Thanos", 100);
    createThanosMoves();
  }

  public void createThanosMoves() {
    removeMove("attack");
    removeMove("heal");
    addMove("smash");
    addMove("Thanos Snap");
  }

  public String smash(Player otherPlayer) {
    int damage = (int)(Math.random() * 30 + 30);
    return attack(otherPlayer, damage);
  }

  public String fingerSnap(Player otherPlayer) {
    String moveMessage = getName() + "' snap takes away half the health of " + otherPlayer.getName() + "!\\n";
    int damage = otherPlayer.getHealth() / 2;
    moveMessage += attack(otherPlayer, damage);
    return moveMessage;
  }

  public String makeMove(String selection, Player otherPlayer) {
    String moveMessage = "";

    moveMessage = super.makeMove(selection, otherPlayer);

    if (selection.equals("smash")) {
      moveMessage = smash(otherPlayer);
    }
    
    if (selection.equals("finger snap")) {
      moveMessage = fingerSnap(otherPlayer);
    }

    return moveMessage;
  }

}`},{path:`Voldemort.java`,text:`public class Voldemort extends Enemy {
  
  public Voldemort(int health) {
    super("Voldemort", 100);
    createVoldemortMoves();
  }

  public void createVoldemortMoves() {
    removeMove("attack");
    removeMove("heal");
    addMove("killing curse");
    addMove("regeneration");
    addMove("crucio");
  }

  public String killingCurse(Player otherPlayer) {
    String moveMessage = "AVADA KEDAVRA!\\n";
    int damage = (int)(Math.random() * 40 + 10);
    moveMessage += attack(otherPlayer, damage);
    return moveMessage;
  }

  public String regeneration() {
    return heal();
  }

  public String crucio(Player otherPlayer) {
    int damage = (int)(Math.random() + 30 + 10);
    return attack(otherPlayer, damage);
  }

  public String makeMove(String selection, Player otherPlayer) {
    String moveMessage = "";

    moveMessage = super.makeMove(selection, otherPlayer);

    if (selection.equals("killing curse")) {
      moveMessage = killingCurse(otherPlayer);
    }
    
    if (selection.equals("regeneration")) {
      moveMessage = regeneration();
    }

    if (selection.equals("crucio")) {
      moveMessage = crucio(otherPlayer);
    }

    return moveMessage;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with the Console Example Projects (d)`,lesson:`Lesson 1: Project Planning`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    Player redPlayer = new Player("red");
    Player yellowPlayer;

    Scanner input = new Scanner(System.in);
    System.out.println("Do you want to play against?");
    System.out.println("1: another player");
    System.out.println("2: a random computer player");
    System.out.println("3: a pattern computer player");
    int choice = input.nextInt();

    if (choice == 1) {
        yellowPlayer = new Player("yellow");
    }
    else if (choice == 2) {
        yellowPlayer = new RandomComputerPlayer("yellow");
    }
    else {
        yellowPlayer = new PatternComputerPlayer("yellow");
    }

    ConnectFour game = new ConnectFour(redPlayer, yellowPlayer);
    game.playGame();
    
  }
}`},{path:`Board.java`,text:`public class Board {

  private String[][] grid;

  public Board() {
    this.grid = createGrid();
  }

  public String[][] getGrid() {
    return grid;
  }

  public String[][] createGrid() {
    String[][] tempGrid = new String[7][15];

    for (int row = 0; row < tempGrid.length; row++) {
      for (int col = 0; col < tempGrid[0].length; col++) {
        if (col % 2 == 0) {
          tempGrid[row][col] = "|";
        }
        else {
          tempGrid[row][col] = " ";
        }

        if (row == 6) {
          tempGrid[row][col] = "-";
        }
      }
    }

    return tempGrid;
  }

  public void showGrid() {
    for (int row = 0; row < grid.length; row++) {
      for (int col = 0; col < grid[0].length; col++) {
        System.out.print(grid[row][col]);
      }

      System.out.println();
    }
  }

  public void addDisk(int playerSelection, String playerLetter) {
    for (int position = 5; position >= 0; position--) {
      if (grid[position][playerSelection] == " ") {
        grid[position][playerSelection] = playerLetter;
        break;
      }
    }
  }

  public String checkWinner() {
    String horizontalResult = checkHorizontalRow();
    String verticalResult = checkVerticalRow();
    String diagonalResult = checkDiagonalRow();
    String reverseResult = checkReverseVertical();

    String result;

    if (horizontalResult != null) {
      result = horizontalResult;
    }
    else if (verticalResult != null) {
      result = verticalResult;
    }
    else if (diagonalResult != null) {
      result = diagonalResult;
    }
    else if (reverseResult != null) {
      result = reverseResult;
    }
    else {
      result = null;
    }

    return result;
  }

  private String checkHorizontalRow() {
    String winner = null;

    for (int row = 0; row < 6; row ++) {
      for (int col = 0; col < 7; col += 2) {
        if ((grid[row][col+1] != " ") && (grid[row][col+3] != " ") && (grid[row][col+5] != " ") && (grid[row][col+7] != " ")
            && ((grid[row][col+1].equals(grid[row][col+3])) && (grid[row][col+3].equals(grid[row][col+5])) && (grid[row][col+5].equals(grid[row][col+7])))) {

          winner = grid[row][col+1];

        }
      }
    }

    return winner;
  }

  private String checkVerticalRow() {
    String winner = null;

    for (int row = 1; row < 15; row += 2) {
      for (int col = 0; col < 3; col++) {
        if ((grid[col][row] != " ") && (grid[col+1][row] != " ") && (grid[col+2][row] != " ") && (grid[col+3][row] != " ")
            && ((grid[col][row].equals(grid[col+1][row])) && (grid[col+1][row].equals(grid[col+2][row])) && (grid[col+2][row].equals(grid[col+3][row])))) {

          winner = grid[col][row];

        }
      }
    }

    return winner;
  }

  private String checkDiagonalRow() {
    String winner = null;

    for (int row = 0; row < 3; row++) {
      for (int col = 1; col < 9; col += 2) {
        if ((grid[row][col] != " ") && (grid[row+1][col+2] != " ") && (grid[row+2][col+4] != " ") && (grid[row+3][col+6] != " ")
            && ((grid[row][col].equals(grid[row+1][col+2])) && (grid[row+1][col+2].equals(grid[row+2][col+4])) && (grid[row+2][col+4].equals(grid[row+3][col+6])))) {

          winner = grid[row][col];
        }
      }
    }

    return winner;
  }

  private String checkReverseVertical() {
    String winner = null;

    for (int row = 0; row < 3; row++) {
      for (int col = 7; col < 15; col += 2) {
        if ((grid[row][col] != " ") && (grid[row+1][col-2] != " ") && (grid[row+2][col-4] != " ") && (grid[row+3][col-6] != " ")
            && ((grid[row][col].equals(grid[row+1][col-2])) && (grid[row+1][col-2].equals(grid[row+2][col-4])) && (grid[row+2][col-4].equals(grid[row+3][col-6])))) {

              winner = grid[row][col];
        }
      }
    }

    return winner;
  }
    
}`},{path:`ConnectFour.java`,text:`public class ConnectFour {

  private Board gameBoard;
  private Player redPlayer;
  private Player yellowPlayer;
  
  public ConnectFour(Player redPlayer, Player yellowPlayer) {
    this.redPlayer = redPlayer;
    this.yellowPlayer = yellowPlayer;
  }
  
  public void dropDisk(Player currentPlayer) {
    int selection = currentPlayer.getPosition();
    gameBoard.addDisk(selection, currentPlayer.getLetter());
  }
  
  public String displayWinner(String winner) {
    String result = null;

    if (winner != null) {
      if (winner.equals(redPlayer.getLetter())) {
        result = "The red player won.";
      }

      if (winner.equals(yellowPlayer.getLetter())) {
        result = "The yellow player won.";
      }
    }

    return result;
  }
  
  public Player choosePlayer(int turn) {
    Player currentPlayer = null;

    if (turn % 2 == 0) {
      currentPlayer = redPlayer;
    }
    else {
      currentPlayer = yellowPlayer;
    }

    return currentPlayer;
  }
  
  public void playGame() {
    int turn = 0;
    String winner = null;

    gameBoard = new Board();
    gameBoard.showGrid();

    while (winner == null) {
      Player currentPlayer = choosePlayer(turn);
      dropDisk(currentPlayer);

      turn++;
      gameBoard.showGrid();
      winner = gameBoard.checkWinner();
      endGame(currentPlayer, winner);
    }
  }
  
  public void endGame(Player currentPlayer, String winner) {
    if (winner != null) {
      String winningMessage = displayWinner(winner);
      System.out.println(winningMessage);

      if (currentPlayer.playAgain()) {
        playGame();
      }
    }
  }

}`},{path:`PatternComputerPlayer.java`,text:`public class PatternComputerPlayer extends Player {
  private int position = 0;

  public PatternComputerPlayer(String color) {
    super(color);
  }

  public int getPosition() {
    position += 2;
    position %= 7;
    return 2 * position + 1;
  }
}`},{path:`Player.java`,text:`import java.util.Scanner;

public class Player {
    
  private String color;

  public Player(String color) {
    this.color = color;
  }

  public String getColor() {
    return color;
  }

  public String getLetter() {
    return color.substring(0, 1).toUpperCase();
  }

  public int getPosition() {
    Scanner input = new Scanner(System.in);

    System.out.print("Drop a " + color + " disk at column 0-6: ");
    int position = input.nextInt();
    position = 2 * position + 1;

    return position;
  }

  public boolean playAgain() {
    boolean result = false;
    Scanner input = new Scanner(System.in);

    System.out.print("Do you want to play again? ");
    String answer = input.nextLine();

    if (answer.equals("yes") || answer.equals("y")) {
      result = true;
    }

    return result;
  }
}`},{path:`RandomComputerPlayer.java`,text:`public class RandomComputerPlayer extends Player {
  public RandomComputerPlayer(String color) {
    super(color);
  }

  public int getPosition() {
    int position = (int) (Math.random() * 7);
    position = 2 * position + 1;

    return position;
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Linear Search #1`,lesson:`Lesson 2: Searching`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int searchList(int[] numbers, int target) {
    int itemsChecked = 0;

    for (int index = 0; index < numbers.length; index++) {
      System.out.println("Number of items checked: " + itemsChecked);

      if (numbers[index] == target) {
        System.out.println(numbers[index] + ": is target");
        return index;
      }
      else {
        System.out.println(numbers[index] + ": not target");
      }

      itemsChecked++;
    }

    return -1;
  }

  /*
   * Returns the index that target is found in the ArrayList numbers,
   * or returns -1 if the target is not found
   */
  public static int searchList(ArrayList<Integer> numbers, int target) {
    int itemsChecked = 0;

    for (int index = 0; index < numbers.size(); index++) {
      System.out.println("Number of items checked: " + itemsChecked);

      if (numbers.get(index) == target) {
        System.out.println(numbers.get(index) + ": is target");
        return index;
      }
      else {
        System.out.println(numbers.get(index) + ": not target");
      }

      itemsChecked++;
    }

    return -1;
  }

  /*
   * Returns an ArrayList of size numItems containing random numbers
   */
  public static ArrayList<Integer> generateRandomList(int numItems) {
    ArrayList<Integer> temp = new ArrayList<Integer>();

    while (numItems > 0) {
      int randomNumber = (int)(Math.random() * 200);
      temp.add(randomNumber);
      numItems--;
    }

    return temp;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Linear Search #2`,lesson:`Lesson 2: Searching`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int searchList(int[] numbers, int target) {
    int itemsChecked = 0;

    for (int index = 0; index < numbers.length; index++) {
      System.out.println("Number of items checked: " + itemsChecked);

      if (numbers[index] == target) {
        System.out.println(numbers[index] + ": is target");
        return index;
      }
      else {
        System.out.println(numbers[index] + ": not target");
      }

      itemsChecked++;
    }

    return -1;
  }

  /*
   * Returns the index that target is found in the ArrayList numbers,
   * or returns -1 if the target is not found
   */
  public static int searchList(ArrayList<Integer> numbers, int target) {
    int itemsChecked = 0;

    for (int index = 0; index < numbers.size(); index++) {
      System.out.println("Number of items checked: " + itemsChecked);

      if (numbers.get(index) == target) {
        System.out.println(numbers.get(index) + ": is target");
        return index;
      }
      else {
        System.out.println(numbers.get(index) + ": not target");
      }

      itemsChecked++;
    }

    return -1;
  }

  /*
   * Returns an ArrayList of size numItems containing random numbers
   */
  public static ArrayList<Integer> generateRandomList(int numItems) {
    ArrayList<Integer> temp = new ArrayList<Integer>();

    while (numItems > 0) {
      int randomNumber = (int)(Math.random() * 200);
      temp.add(randomNumber);
      numItems--;
    }

    return temp;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Linear Search #3`,lesson:`Lesson 2: Searching`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int searchList(int[] numbers, int target) {
    int itemsChecked = 0;

    for (int index = 0; index < numbers.length; index++) {
      System.out.println("Number of items checked: " + itemsChecked);

      if (numbers[index] == target) {
        System.out.println(numbers[index] + ": is target");
        return index;
      }
      else {
        System.out.println(numbers[index] + ": not target");
      }

      itemsChecked++;
    }

    return -1;
  }

  /*
   * Returns the index that target is found in the ArrayList numbers,
   * or returns -1 if the target is not found
   */
  public static int searchList(ArrayList<Integer> numbers, int target) {
    int itemsChecked = 0;

    for (int index = 0; index < numbers.size(); index++) {
      System.out.println("Number of items checked: " + itemsChecked);

      if (numbers.get(index) == target) {
        System.out.println(numbers.get(index) + ": is target");
        return index;
      }
      else {
        System.out.println(numbers.get(index) + ": not target");
      }

      itemsChecked++;
    }

    return -1;
  }

  /*
   * Returns an ArrayList of size numItems containing random numbers
   */
  public static ArrayList<Integer> generateRandomList(int numItems) {
    ArrayList<Integer> temp = new ArrayList<Integer>();

    while (numItems > 0) {
      int randomNumber = (int)(Math.random() * 200);
      temp.add(randomNumber);
      numItems--;
    }

    return temp;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Binary Search`,lesson:`Lesson 3: Binary Search`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[] firstSet = {10, 20, 30, 40, 50, 60};
    int[] secondSet = {67, 45, 72, 23, 38, 81};

    int firstLocation = Searcher.binarySearch(firstSet, 50);
    System.out.println("First set: " + firstLocation);
    
  }
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int binarySearch(int[] numbers, int target) {
    int left = 0;
    int right = numbers.length - 1;
    int totalChecks = 0;

    while (left <= right) {
      totalChecks++;
      System.out.println("Total checks: " + totalChecks);

      int mid = (left + right) / 2;
      System.out.println("Middle value: " + numbers[mid]);

      if (numbers[mid] < target) {
        left = mid + 1;
      }
      else if (numbers[mid] > target) {
        right = mid + 1;
      }
      else {
        return mid;
      }
    }

    return -1;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Recursive Binary Search #1`,lesson:`Lesson 3: Binary Search`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int binarySearch(int[] numbers, int target) {
    int left = 0;
    int right = numbers.length - 1;
    int totalChecks = 0;

    while (left <= right) {
      totalChecks++;
      System.out.println("Total checks: " + totalChecks);

      int mid = (left + right) / 2;
      System.out.println("Middle value: " + numbers[mid]);

      if (numbers[mid] < target) {
        left = mid + 1;
      }
      else if (numbers[mid] > target) {
        right = mid - 1;
      }
      else {
        return mid;
      }
    }

    return -1;
  }

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int recursiveBinarySearch(int[] numbers, int left, int right, int target) {
    System.out.println("Number checked!");
    
    if (left > right) {
      return -1;
    }

    int middle = (left + right) / 2;

    if (target < numbers[middle]) {
      return recursiveBinarySearch(numbers, left, middle - 1, target);
    }
    else if (target > numbers[middle]) {
      return recursiveBinarySearch(numbers, middle + 1, right, target);
    }
    else {
      return middle;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Recursive Binary Search #2`,lesson:`Lesson 3: Binary Search`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int binarySearch(int[] numbers, int target) {
    int left = 0;
    int right = numbers.length - 1;
    int totalChecks = 0;

    while (left <= right) {
      totalChecks++;
      System.out.println("Total checks: " + totalChecks);

      int mid = (left + right) / 2;
      System.out.println("Middle value: " + numbers[mid]);

      if (numbers[mid] < target) {
        left = mid + 1;
      }
      else if (numbers[mid] > target) {
        right = mid - 1;
      }
      else {
        return mid;
      }
    }

    return -1;
  }

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int recursiveBinarySearch(int[] numbers, int left, int right, int target) {
    System.out.println("Number checked!");
    
    if (left > right) {
      return -1;
    }

    int middle = (left + right) / 2;

    if (target < numbers[middle]) {
      return recursiveBinarySearch(numbers, left, middle - 1, target);
    }
    else if (target > numbers[middle]) {
      return recursiveBinarySearch(numbers, middle + 1, right, target);
    }
    else {
      return middle;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Recursive Binary Search #3`,lesson:`Lesson 3: Binary Search`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Searcher.java`,text:`import java.util.ArrayList;

/*
 * Analyzes lists of data
 */
public class Searcher {

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int binarySearch(int[] numbers, int target) {
    int left = 0;
    int right = numbers.length - 1;
    int totalChecks = 0;

    while (left <= right) {
      totalChecks++;
      System.out.println("Total checks: " + totalChecks);

      int mid = (left + right) / 2;
      System.out.println("Middle value: " + numbers[mid]);

      if (numbers[mid] < target) {
        left = mid + 1;
      }
      else if (numbers[mid] > target) {
        right = mid - 1;
      }
      else {
        return mid;
      }
    }

    return -1;
  }

  /*
   * Returns the index that target is found in the 1D array numbers,
   * or returns -1 if the target is not found
   */
  public static int recursiveBinarySearch(int[] numbers, int left, int right, int target) {
    System.out.println("Number checked!");
    
    if (left > right) {
      return -1;
    }

    int middle = (left + right) / 2;

    if (target < numbers[middle]) {
      return recursiveBinarySearch(numbers, left, middle - 1, target);
    }
    else if (target > numbers[middle]) {
      return recursiveBinarySearch(numbers, middle + 1, right, target);
    }
    else {
      return middle;
    }
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Selection Sort #1`,lesson:`Lesson 4: Selection Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Selection Sort #2`,lesson:`Lesson 4: Selection Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Selection Sort #3`,lesson:`Lesson 4: Selection Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Insertion Sort #1`,lesson:`Lesson 5: Insertion Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  /*
   * Uses the insertion sort algorithm to sort the elements in the 1D array data
   */
  public static void insertionSort(int[] data) {
    int numCompares = 0;

    for (int i = 1; i < data.length; i++) {
      int temp = data[i];
      int index = i;

      numCompares++;

      
      while (index > 0 && data[index - 1] > temp) {
        int value = data[index - 1];
        data[index] = data[index - 1];
        index--;

        if (index > 0) {
          numCompares++;
        }
      }

      data[index] = temp;
      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /*
   * Uses the selection sort algorithm to sort the elements in the 1D array data
   */
  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        System.out.println("Comparison #" + numCompares + " -- checking " + data[m] + " and " + data[k]);
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Insertion Sort #2`,lesson:`Lesson 5: Insertion Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  /*
   * Uses the insertion sort algorithm to sort the elements in the 1D array data
   */
  public static void insertionSort(int[] data) {
    int numCompares = 0;

    for (int i = 1; i < data.length; i++) {
      int temp = data[i];
      int index = i;

      numCompares++;

      
      while (index > 0 && data[index - 1] > temp) {
        int value = data[index - 1];
        data[index] = data[index - 1];
        index--;

        if (index > 0) {
          numCompares++;
        }
      }

      data[index] = temp;
      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /*
   * Uses the selection sort algorithm to sort the elements in the 1D array data
   */
  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        System.out.println("Comparison #" + numCompares + " -- checking " + data[m] + " and " + data[k]);
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Insertion Sort #3`,lesson:`Lesson 5: Insertion Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  /*
   * Uses the insertion sort algorithm to sort the elements in the 1D array data
   */
  public static void insertionSort(int[] data) {
    int numCompares = 0;

    for (int i = 1; i < data.length; i++) {
      int temp = data[i];
      int index = i;

      numCompares++;

      
      while (index > 0 && data[index - 1] > temp) {
        int value = data[index - 1];
        data[index] = data[index - 1];
        index--;

        if (index > 0) {
          numCompares++;
        }
      }

      data[index] = temp;
      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /*
   * Uses the selection sort algorithm to sort the elements in the 1D array data
   */
  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        System.out.println("Comparison #" + numCompares + " -- checking " + data[m] + " and " + data[k]);
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Merge Sort #1`,lesson:`Lesson 6: Merge Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[] numbers = {12, 11, 13, 5, 6, 7};
    Sorter.mergeSort(numbers, 0, numbers.length - 1);
    System.out.println("numbers after merge sort -> " + Sorter.arrayToString(numbers));

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



    
    
  }
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {

  /*
   * Uses the merge sort algorithm to sort the elements in the 1D array numList
   */
  public static void mergeSort(int[] numList, int left, int right) {
    if (left < right) {
      int middle = (left + right) / 2;

      mergeSort(numList, left, middle);
      mergeSort(numList, middle + 1, right);

      merge(numList, left, middle, right);
    }
  }

  /*
   * Splits the 1D array numList into two sub-arrays and merges them together in order
   */
  public static void merge(int[] numList, int left, int middle, int right) {
    // create temporary arrays
    int[] leftList = new int[middle - left + 1];
    int[] rightList = new int[right - middle];

    // copy numList into the temporary arrays
    for (int index = 0; index < leftList.length; index++) {
      leftList[index] = numList[left + index];
    }

    for (int index = 0; index < rightList.length; index++) {
      rightList[index] = numList[middle + index + 1];
    }

    // current indexes of temporary arrays
    int leftIndex = 0;
    int rightIndex = 0;
    
    // copy from leftList and rightList back into numList
    for (int index = left; index <= right; index++) {
      // if there are still uncopied values in leftList and rightList, copy the smallest value of the two
      if (leftIndex < leftList.length && rightIndex < rightList.length) {
        if (leftList[leftIndex] < rightList[rightIndex]) {
          numList[index] = leftList[leftIndex];
          leftIndex++;
        }
        else {
          numList[index] = rightList[rightIndex];
          rightIndex++;
        }
      }
      else if (leftIndex < leftList.length) {
        // if all values have been copied from rightList, copy the rest of leftList
        numList[index] = leftList[leftIndex];
        leftIndex++;
      }
      else if (rightIndex < rightList.length) {
        // if all values have been copied from leftList, copy the rest of rightList
        numList[index] = rightList[rightIndex];
        rightIndex++;
      }
    }
  }

  /*
   * Uses the insertion sort algorithm to sort the elements in the 1D array data
   */
  public static void insertionSort(int[] data) {
    int numCompares = 0;

    for (int i = 1; i < data.length; i++) {
      int temp = data[i];
      int index = i;

      numCompares++;
      
      while (index > 0 && data[index - 1] > temp) {
        int value = data[index - 1];
        data[index] = data[index - 1];
        index--;

        if (index > 0) {
          numCompares++;
        }
      }

      data[index] = temp;
      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /*
   * Uses the selection sort algorithm to sort the elements in the 1D array data
   */
  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        System.out.println("Comparison #" + numCompares + " -- checking " + data[m] + " and " + data[k]);
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Merge Sort #2`,lesson:`Lesson 6: Merge Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {
  
  static int MERGE_COPIES = 0;
  
  /*
   * Uses the merge sort algorithm to sort the elements in the 1D array numList
   */
  public static void mergeSort(int[] numList, int left, int right) {
    if (left == 0 && right == numList.length - 1) MERGE_COPIES = 0;
    
    if (left < right) {
      int middle = (left + right) / 2;

      mergeSort(numList, left, middle);
      mergeSort(numList, middle + 1, right);

      merge(numList, left, middle, right);
    }
    
    if (left == 0 && right == numList.length - 1) {
      System.out.println("MergeSort copies: " + MERGE_COPIES);
    }
  }

  /*
   * Splits the 1D array numList into two sub-arrays and merges them together in order
   */
  public static void merge(int[] numList, int left, int middle, int right) {
    // create temporary arrays
    int[] leftList = new int[middle - left + 1];
    int[] rightList = new int[right - middle];

    // copy numList into the temporary arrays
    for (int index = 0; index < leftList.length; index++) {
      leftList[index] = numList[left + index];
    }

    for (int index = 0; index < rightList.length; index++) {
      rightList[index] = numList[middle + index + 1];
    }

    // current indexes of temporary arrays
    int leftIndex = 0;
    int rightIndex = 0;
    
    // copy from leftList and rightList back into numList
    for (int index = left; index <= right; index++) {
      // if there are still uncopied values in leftList and rightList, copy the smallest value of the two
      if (leftIndex < leftList.length && rightIndex < rightList.length) {
        if (leftList[leftIndex] < rightList[rightIndex]) {
          numList[index] = leftList[leftIndex];
          leftIndex++;
          MERGE_COPIES++;
        }
        else {
          numList[index] = rightList[rightIndex];
          rightIndex++;
          MERGE_COPIES++;
        }
      }
      else if (leftIndex < leftList.length) {
        // if all values have been copied from rightList, copy the rest of leftList
        numList[index] = leftList[leftIndex];
        leftIndex++;
        MERGE_COPIES++;
      }
      else if (rightIndex < rightList.length) {
        // if all values have been copied from leftList, copy the rest of rightList
        numList[index] = rightList[rightIndex];
        rightIndex++;
        MERGE_COPIES++;
      }
    }
  }

  /*
   * Uses the insertion sort algorithm to sort the elements in the 1D array data
   */
  public static void insertionSort(int[] data) {
    int numSwaps = 0;
    int numCompares = 0;
    System.out.println("Insertion Sort: "+ arrayToString(data));
    for (int i = 1; i < data.length; i++) {
      int temp = data[i];
      int index = i;

      numCompares++;
      
      while (index > 0 && data[index - 1] > temp) {
        int value = data[index - 1];
        data[index] = data[index - 1];
        numSwaps++;  
        index--;

        if (index > 0) {
          numCompares++;
        }
      }

      data[index] = temp;
      System.out.println("Updated array: " + arrayToString(data));
    }
    System.out.println("InsertionSort comparisons = " + numCompares);
    System.out.println("InsertionSort swaps = " + numSwaps);
  }

  /*
   * Uses the selection sort algorithm to sort the elements in the 1D array data
   */
  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        System.out.println("Comparison #" + numCompares + " -- checking " + data[m] + " and " + data[k]);
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Merge Sort #3`,lesson:`Lesson 6: Merge Sort`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Sorter.java`,text:`/*
 * Analyzes data in a list
 */
public class Sorter {
  
  static int MERGE_COPIES = 0;
  
  /*
   * Uses the merge sort algorithm to sort the elements in the 1D array numList
   */
  public static void mergeSort(int[] numList, int left, int right) {
    if (left == 0 && right == numList.length - 1) MERGE_COPIES = 0;
    
    if (left < right) {
      int middle = (left + right) / 2;

      mergeSort(numList, left, middle);
      mergeSort(numList, middle + 1, right);

      merge(numList, left, middle, right);
    }
    
    if (left == 0 && right == numList.length - 1) {
      System.out.println("MergeSort copies: " + MERGE_COPIES);
    }
  }

  /*
   * Splits the 1D array numList into two sub-arrays and merges them together in order
   */
  public static void merge(int[] numList, int left, int middle, int right) {
    // create temporary arrays
    int[] leftList = new int[middle - left + 1];
    int[] rightList = new int[right - middle];

    // copy numList into the temporary arrays
    for (int index = 0; index < leftList.length; index++) {
      leftList[index] = numList[left + index];
    }

    for (int index = 0; index < rightList.length; index++) {
      rightList[index] = numList[middle + index + 1];
    }

    // current indexes of temporary arrays
    int leftIndex = 0;
    int rightIndex = 0;
    
    // copy from leftList and rightList back into numList
    for (int index = left; index <= right; index++) {
      // if there are still uncopied values in leftList and rightList, copy the smallest value of the two
      if (leftIndex < leftList.length && rightIndex < rightList.length) {
        if (leftList[leftIndex] < rightList[rightIndex]) {
          numList[index] = leftList[leftIndex];
          leftIndex++;
          MERGE_COPIES++;
        }
        else {
          numList[index] = rightList[rightIndex];
          rightIndex++;
          MERGE_COPIES++;
        }
      }
      else if (leftIndex < leftList.length) {
        // if all values have been copied from rightList, copy the rest of leftList
        numList[index] = leftList[leftIndex];
        leftIndex++;
        MERGE_COPIES++;
      }
      else if (rightIndex < rightList.length) {
        // if all values have been copied from leftList, copy the rest of rightList
        numList[index] = rightList[rightIndex];
        rightIndex++;
        MERGE_COPIES++;
      }
    }
  }

  /*
   * Uses the insertion sort algorithm to sort the elements in the 1D array data
   */
  public static void insertionSort(int[] data) {
    int numSwaps = 0;
    int numCompares = 0;
    System.out.println("Insertion Sort: "+ arrayToString(data));
    for (int i = 1; i < data.length; i++) {
      int temp = data[i];
      int index = i;

      numCompares++;
      
      while (index > 0 && data[index - 1] > temp) {
        int value = data[index - 1];
        data[index] = data[index - 1];
        numSwaps++;  
        index--;

        if (index > 0) {
          numCompares++;
        }
      }

      data[index] = temp;
      System.out.println("Updated array: " + arrayToString(data));
    }
    System.out.println("InsertionSort comparisons = " + numCompares);
    System.out.println("InsertionSort swaps = " + numSwaps);
  }

  /*
   * Uses the selection sort algorithm to sort the elements in the 1D array data
   */
  public static void selectionSort(int[] data) {
    int numCompares = 0;
    
    for (int j = 0; j < data.length - 1; j++) {
      int m = j;

      for (int k = j + 1; k < data.length; k++) {
        numCompares++;
        System.out.println("Comparison #" + numCompares + " -- checking " + data[m] + " and " + data[k]);
        
        if (data[k] < data[m]) {
          m = k;
        }
      }

      int temp = data[m];
      data[m] = data[j];
      data[j] = temp;

      System.out.println("Current array: " + arrayToString(data));
    }
  }

  /* 
   * Returns a String containing each element in the 1D array data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Multiple Lists`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Reads data from text files into ArrayLists
    ArrayList<Double> dallas = FileReader.toDoubleList("dallastemps.txt");
    ArrayList<Double> seattle = FileReader.toDoubleList("seattletemps.txt");

    // Instantiates a CityTemps object with one of the ArrayLists
    CityTemps dallasTemps = new CityTemps(dallas);

    // Calls the compareTemps() method
    int numDays = dallasTemps.compareTemps(seattle);

    // Prints the result
    System.out.println("Dallas had " + numDays + " days warmer than Seattle in January 2020.");
    
  }
}`},{path:`CityTemps.java`,text:`import java.util.ArrayList;

/*
 * Analyzes data about temperatures in cities
 */
public class CityTemps {

  private ArrayList<Double> cityTemps;      // The list of temperatures in a city

  /*
   * Initializes cityTemps to the specified ArrayList of temperatures
   */
  public CityTemps(ArrayList<Double> cityTemps) {
    this.cityTemps = cityTemps;
  }

  /*
   * Returns the number of values in cityTemps that are greater than otherCity
   */
  public int compareTemps(ArrayList<Double> otherCity) {
    int count = 0;

    for (int index = 0; index < cityTemps.size(); index++) {
      Double currentCityTemp = cityTemps.get(index);
      Double otherCityTemp = otherCity.get(index);

      if (currentCityTemp > otherCityTemp) {
        count++;
      }
    }

    return count;
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
  
}`}],validationFiles:[],dataFiles:[{path:`dallastemps.txt`,text:`47.3
54.5
51.1
47.3
51.3
50.4
49.9
49.7
61.1
67.4
42.7
38.8
47.9
58.6
67.6
54.9
49.7
55.8
42.8
45.3
43.8
41.8
47.2
48.2
49.6
57.0
50.9
51.7
41.5
41.7
45.5`},{path:`detroittemps.txt`,text:`32.0
42.1
41.5
34.5
34.5
38.5
35.4
24.4
32.0
46.6
44.4
27.0
33.1
36.5
34.0
30.6
22.5
33.1
23.5
20.5
24.4
26.1
32.0
35.4
36.0
35.4
35.1
33.1
30.6
31.5
29.5`},{path:`seattletemps.txt`,text:`51.0
45.1
48.1
47.0
44.3
44.8
50.9
45.2
35.9
38.2
44.2
41.4
35.3
31.0
35.0
41.1
37.3
41.7
48.2
50.2
48.7
45.3
50.9
50.9
48.1
49.4
47.6
46.0
47.9
46.3
51.3`}]},{name:`Investigate and Modify: Multiple Lists #1`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`CityTemps.java`,text:`import java.util.ArrayList;

/*
 * Analyzes data about temperatures in cities
 */
public class CityTemps {

  private ArrayList<Double> cityTemps;      // The list of temperatures in a city

  /*
   * Initializes cityTemps to the specified ArrayList of temperatures
   */
  public CityTemps(ArrayList<Double> cityTemps) {
    this.cityTemps = cityTemps;
  }

  /*
   * Returns the number of values in cityTemps that are greater than otherCity
   */
  public int compareTemps(ArrayList<Double> otherCity) {
    int count = 0;

    for (int index = 0; index < cityTemps.size(); index++) {
      Double currentCityTemp = cityTemps.get(index);
      Double otherCityTemp = otherCity.get(index);

      if (currentCityTemp > otherCityTemp) {
        count++;
      }
    }

    return count;
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> tempList = readDataFromFile(filename);
    ArrayList<Double> returnList = new ArrayList<Double>();
    for (String temp : tempList) {
      returnList.add(Double.parseDouble(temp));
    }

    return returnList;
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
  
}`}],validationFiles:[],dataFiles:[{path:`dallastemps.txt`,text:`47.3
54.5
51.1
47.3
51.3
50.4
49.9
49.7
61.1
67.4
42.7
38.8
47.9
58.6
67.6
54.9
49.7
55.8
42.8
45.3
43.8
41.8
47.2
48.2
49.6
57.0
50.9
51.7
41.5
41.7
45.5`},{path:`detroittemps.txt`,text:`32.0
42.1
41.5
34.5
34.5
38.5
35.4
24.4
32.0
46.6
44.4
27.0
33.1
36.5
34.0
30.6
22.5
33.1
23.5
20.5
24.4
26.1
32.0
35.4
36.0
35.4
35.1
33.1
30.6
31.5
29.5`},{path:`seattletemps.txt`,text:`51.0
45.1
48.1
47.0
44.3
44.8
50.9
45.2
35.9
38.2
44.2
41.4
35.3
31.0
35.0
41.1
37.3
41.7
48.2
50.2
48.7
45.3
50.9
50.9
48.1
49.4
47.6
46.0
47.9
46.3
51.3`}]},{name:`Investigate and Modify: Multiple Lists #2`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`CityTemps.java`,text:`import java.util.ArrayList;

/*
 * Analyzes data about temperatures in cities
 */
public class CityTemps {

  private ArrayList<Double> cityTemps;      // The list of temperatures in a city

  /*
   * Initializes cityTemps to the specified ArrayList of temperatures
   */
  public CityTemps(ArrayList<Double> cityTemps) {
    this.cityTemps = cityTemps;
  }

  /*
   * Returns the number of values in cityTemps that are greater than otherCity
   */
  public int compareTemps(ArrayList<Double> otherCity) {
    int count = 0;

    for (int index = 0; index < cityTemps.size(); index++) {
      Double currentCityTemp = cityTemps.get(index);
      Double otherCityTemp = otherCity.get(index);

      if (currentCityTemp > otherCityTemp) {
        count++;
      }
    }

    return count;
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> tempList = readDataFromFile(filename);
    ArrayList<Double> returnList = new ArrayList<Double>();
    for (String temp : tempList) {
      returnList.add(Double.parseDouble(temp));
    }

    return returnList;
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
  
}`}],validationFiles:[],dataFiles:[{path:`dallastemps.txt`,text:`47.3
54.5
51.1
47.3
51.3
50.4
49.9
49.7
61.1
67.4
42.7
38.8
47.9
58.6
67.6
54.9
49.7
55.8
42.8
45.3
43.8
41.8
47.2
48.2
49.6
57.0
50.9
51.7
41.5
41.7
45.5`},{path:`detroittemps.txt`,text:`32.0
42.1
41.5
34.5
34.5
38.5
35.4
24.4
32.0
46.6
44.4
27.0
33.1
36.5
34.0
30.6
22.5
33.1
23.5
20.5
24.4
26.1
32.0
35.4
36.0
35.4
35.1
33.1
30.6
31.5
29.5`},{path:`seattletemps.txt`,text:`51.0
45.1
48.1
47.0
44.3
44.8
50.9
45.2
35.9
38.2
44.2
41.4
35.3
31.0
35.0
41.1
37.3
41.7
48.2
50.2
48.7
45.3
50.9
50.9
48.1
49.4
47.6
46.0
47.9
46.3
51.3`}]},{name:`Investigate and Modify: Multiple Lists #3`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`CityTemps.java`,text:`import java.util.ArrayList;

/*
 * Analyzes data about temperatures in cities
 */
public class CityTemps {

  private ArrayList<Double> cityTemps;      // The list of temperatures in a city

  /*
   * Initializes cityTemps to the specified ArrayList of temperatures
   */
  public CityTemps(ArrayList<Double> cityTemps) {
    this.cityTemps = cityTemps;
  }

  /*
   * Returns the number of values in cityTemps that are greater than otherCity
   */
  public int compareTemps(ArrayList<Double> otherCity) {
    int count = 0;

    for (int index = 0; index < cityTemps.size(); index++) {
      Double currentCityTemp = cityTemps.get(index);
      Double otherCityTemp = otherCity.get(index);

      if (currentCityTemp > otherCityTemp) {
        count++;
      }
    }

    return count;
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

  public static ArrayList<Double> toDoubleList(String filename) {
    ArrayList<String> tempList = readDataFromFile(filename);
    ArrayList<Double> returnList = new ArrayList<Double>();
    for (String temp : tempList) {
      returnList.add(Double.parseDouble(temp));
    }

    return returnList;
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
  
}`}],validationFiles:[],dataFiles:[{path:`dallastemps.txt`,text:`47.3
54.5
51.1
47.3
51.3
50.4
49.9
49.7
61.1
67.4
42.7
38.8
47.9
58.6
67.6
54.9
49.7
55.8
42.8
45.3
43.8
41.8
47.2
48.2
49.6
57.0
50.9
51.7
41.5
41.7
45.5`},{path:`detroittemps.txt`,text:`32.0
42.1
41.5
34.5
34.5
38.5
35.4
24.4
32.0
46.6
44.4
27.0
33.1
36.5
34.0
30.6
22.5
33.1
23.5
20.5
24.4
26.1
32.0
35.4
36.0
35.4
35.1
33.1
30.6
31.5
29.5`},{path:`seattletemps.txt`,text:`51.0
45.1
48.1
47.0
44.3
44.8
50.9
45.2
35.9
38.2
44.2
41.4
35.3
31.0
35.0
41.1
37.3
41.7
48.2
50.2
48.7
45.3
50.9
50.9
48.1
49.4
47.6
46.0
47.9
46.3
51.3`}]},{name:`Practice: Traversing Multiple Lists (a)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Creates ArrayLists to contain the sales for each week
    ArrayList<Product> firstWeek = Marketing.getSales("firstweek.txt");
    ArrayList<Product> secondWeek = Marketing.getSales("secondweek.txt");

    // Prints the values in each list
    System.out.println("First Week Sales\\n" + Marketing.salesToString(firstWeek));
    System.out.println("Second Week Sales\\n" + Marketing.salesToString(secondWeek));

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the combineWeeks() and combineSales() methods and print the new list.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Marketing.java`,text:`import java.util.ArrayList;
import java.util.Scanner;
import java.io.File;
import java.io.FileNotFoundException;

/*
 * Manages and analyzes data about product sales
 */
public class Marketing {

  /*
   * Reads the product sales information from a file
   * and returns an ArrayList of Products
   */
  public static ArrayList<Product> getSales(String filename) {
    File myFile = new File(filename);
    Scanner fileReader = createScanner(myFile);

    ArrayList<Product> tempProducts = new ArrayList<Product>();

    while (fileReader.hasNextLine()) {
      String currentLine = fileReader.nextLine();
      Product currentProduct = createProduct(currentLine);
      tempProducts.add(currentProduct);
    }

    fileReader.close();
    return tempProducts;
  }

  /*
   * Returns a Scanner object to read a specified File
   */
  public static Scanner createScanner(File theFile) {
    Scanner tempScanner = null;

    try {
      tempScanner = new Scanner(theFile);
    } catch(FileNotFoundException error) {
      System.out.println("File not found.");
    }

    return tempScanner;
  }

  /*
   * Gets the product name and total quantity sold from
   * the currentLine and returns a Product object
   */
  public static Product createProduct(String currentLine) {
    int comma = currentLine.indexOf(",");
    
    String productName = currentLine.substring(0, comma);
    String totalSold = currentLine.substring(comma + 1);
    
    Product temp = new Product(productName, Integer.parseInt(totalSold));
    return temp;
  }

  /*
   * Returns a String containing the product sales for a specified week
   */
  public static String salesToString(ArrayList<Product> week) {
    String text = "";
    
    for (Product item : week) {
      text = text + item + "\\n";
    }

    return text;
  }

  /*
   * Returns a new ArrayList that combines the product sales for the first week and the second week
   */
  public static ArrayList<Product> combineWeeks(ArrayList<Product> firstWeek, ArrayList<Product> secondWeek) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Create and return a new list that combines the sales for firstWeek
     * followed by the sales for secondWeek.
     * -----------------------------------------------------------------------------
     */

    
    
    return null;
  }

  /*
   * Combines duplicate products and their quantities
   */
  public static void combineSales(ArrayList<Product> combined) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find duplicate products and combine their total quantities sold. Remove
     * the duplicate products from the combined list.
     * -----------------------------------------------------------------------------
     */


    
  }
  
}`},{path:`Product.java`,text:`/*
 * Represents a product sold on an e-commerce website
 */
public class Product {

  private String name;     // The name of a product
  private int quantity;    // The total quantity sold

  /*
   * Sets name and quantity to the specified values
   */
  public Product(String name, int quantity) {
    this.name = name;
    this.quantity = quantity;
  }

  /*
   * Returns the name of the product
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the total quantity sold
   */
  public int getQuantity() {
    return quantity;
  }

  /*
   * Changes the quantity to newQuantity
   */
  public void setQuantity(int newQuantity) {
    quantity = newQuantity;
  }

  /*
   * Returns true if this Product has the same name as otherProduct
   */
  public boolean equals(Object other) {
    if (other == this) {
      return true;
    }

    if (!(other instanceof Product)) {
      return false;
    }

    Product otherProduct = (Product) other;
    return this.name.equals(otherProduct.name);
  }

  /*
   * Returns a String containing the product information
   */
  public String toString() {
    return name + ": " + quantity + " sold";
  }
  
}`}],validationFiles:[{path:`MarketingTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

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
  ArrayList<Product> testFirst;
  ArrayList<Product> testSecond;
  ArrayList<Product> testCombined;
   
  @BeforeEach
  public void setup() {
    testFirst = getRandomProducts();
    testSecond = getRandomProducts();

    testCombined = new ArrayList<Product>();
    testCombined.addAll(testFirst);
    testCombined.addAll(testSecond);
  }
   
  @Test
  @Order(1)
  @DisplayName("combineWeeks() combines the two lists into one ArrayList and returns the combined list => ")
  public void testCombineWeeks() {
    String message = "Create a new ArrayList and add each element in firstWeek to the new list. Then add each element in";
    message += "\\n        secondWeek to the new list right after the firstWeek elements." + messageGap;
    ArrayList<Product> actual = Marketing.combineWeeks(testFirst, testSecond);
    assertEquals(testCombined, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("combineSales() finds duplicate products, combines their total quantities sold, and removes the duplicate products => ")
  public void testCombineSalesCombinesQuantities() {
    String message = "Traverse the combined list to find duplicate products and combine their total quantities sold.";
    message += "\\n        Remove the duplicate product from the list after combining the quantities." + messageGap;
    
    getExpectedCombinedSales(testCombined);
    ArrayList<Product> actual = Marketing.combineWeeks(testFirst, testSecond);
    Marketing.combineSales(actual);

    assertEquals(testCombined, actual, message);
  }

  private ArrayList<Product> getRandomProducts() {
    ArrayList<Product> temp = new ArrayList<Product>();

    for (int index = 0; index < 5; index++) {
      int randomQuantity = (int)(Math.random() * 10) + 5;
      temp.add(new Product("some product", randomQuantity));
    }

    return temp;
  }

  private void getExpectedCombinedSales(ArrayList<Product> combined) {
    for (int current = 0; current < combined.size(); current++) {
      Product currentProduct = combined.get(current);

      for (int next = current + 1; next < combined.size(); next++) {
        Product nextProduct = combined.get(next);

        if (currentProduct.equals(nextProduct)) {
          currentProduct.setQuantity(currentProduct.getQuantity() + nextProduct.getQuantity());
          combined.remove(next);
          next--;
        }
      }
    }
  }
  
}`}],dataFiles:[{path:`firstweek.txt`,text:`shoes,12
book,18
laptop,23
yoga mat,6
phone case,31
charger,25
headphones,9
air fryer,4
water bottle,15
board game,3`},{path:`secondweek.txt`,text:`board game,17
pet food,7
headphones,26
shoes,14
cat bed,16
waffle maker,9
crochet kit,3
smart bulb,12
charger,4
robot vacuum,5`}]},{name:`Practice: Traversing Multiple Lists (b)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    // Creates 1D array of food truck sales
    double[] firstTruckSales = {266.74, 196.93, 214.67, 165.01, 109.93, 129.77, 196.37};
    double[] secondTruckSales = {132.56, 246.78, 286.68, 248.09, 247.48, 279.93, 210.09};

    // Creates FoodTruck objects with the sales
    FoodTruck firstTruck = new FoodTruck("Charlotte", firstTruckSales);
    FoodTruck secondTruck = new FoodTruck("Evelyn", secondTruckSales);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findMostSales() method and print the result.
     * -----------------------------------------------------------------------------
     */

    
    
  }
}`},{path:`FoodTruck.java`,text:`/*
 * Represents a food truck
 */
public class FoodTruck {

  private String name;   // The name of the food truck
  private double[] sales;   // The list of daily sales

  /*
   * Sets name to the specified name and initializes
   * sales to the specified 1D array of sales
   */
  public FoodTruck(String name, double[] sales) {
    this.name = name;
    this.sales = sales;
  }

  /*
   * Returns the name of the FoodTruck
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the 1D array of sales
   */
  public double[] getSales() {
    return sales;
  }

  /*
   * Calculates and returns the total sales
   */
  public double totalSales() {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Calculate and return the total sales.
     * -----------------------------------------------------------------------------
     */

    return -1;
  }
  
}`},{path:`Owner.java`,text:`/*
 * Represents an owner of a food truck business
 */
public class Owner {

  /*
   * Returns the name of the food truck that has the most sales
   */
  public static String findMostSales(FoodTruck firstTruck, FoodTruck secondTruck) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Determine which food truck has the most sales and return the name
     * of the food truck.
     * -----------------------------------------------------------------------------
     */

    return "";
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Traversing Multiple Lists (c)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Creates an ArrayList containing attendees for an event
    ArrayList<Attendee> attendees = new ArrayList<Attendee>();
    attendees.add(new Attendee("Alice", 1));
    attendees.add(new Attendee("Bob", 2));
    attendees.add(new Attendee("Charlie", 3));
    attendees.add(new Attendee("David", 4));
    attendees.add(new Attendee("Eve", 5));

    // Creates an Event object
    Event meetup = new Event("Book Club Meetup", attendees);

    // Creates a list of cancelled attendees
    ArrayList<Attendee> cancelledAttendees = new ArrayList<Attendee>();
    cancelledAttendees.add(new Attendee("Charlie", 3));
    cancelledAttendees.add(new Attendee("Eve", 5));

    // Prints the information about the event
    System.out.println(meetup);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the removeCancelledAttendees() method and print the updated list
     * of attendees by printing the Event object.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`Attendee.java`,text:`/*
 * Represents an attendee at an event
 */
public class Attendee {
  
  private String name;     // The name of an attendee
  private int id;          // The id of an attendee

  /*
   * Sets name and id to the specified values
   */
  public Attendee(String name, int id) {
    this.name = name;
    this.id = id;
  }

  /*
   * Returns the name of the attendee
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the id of the attendee
   */
  public int getId() {
    return id;
  }

  /*
   * Returns true if this attendee has the same name and id as other
   */
  public boolean equals(Object other) {
    if (this == other) {
      return true;
    }

    if (!(other instanceof Attendee)) {
      return false;
    }
    
    Attendee otherAttendee = (Attendee) other;
    return this.name.equals(otherAttendee.name) && this.id == otherAttendee.id;
  }
  
}`},{path:`Event.java`,text:`import java.util.ArrayList;

/*
 * Represents an event on a social networking platform
 */
public class Event {

  private String name;                       // The name of an event
  private ArrayList<Attendee> attendees;     // The list of attendees

  /*
   * Sets name to the specified name and initializes attendees
   * to the specified ArrayList of Attendee objects
   */
  public Event(String name, ArrayList<Attendee> attendees) {
    this.name = name;
    this.attendees = attendees;
  }

  /*
   * Returns the name of the event
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the list of attendees
   */
  public ArrayList<Attendee> getAttendees() {
    return attendees;
  }

  /*
   * Removes the cancelled attendees from the list of attendees
   */
  public void removeCancelledAttendees(ArrayList<Attendee> cancelledAttendees) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Removes the cancelled attendees from the list of attendees.
     * -----------------------------------------------------------------------------
     */



    
  }

  /*
   * Returns a String containing the name of the event
   * and each attendee in the list of attendees
   */
  public String toString() {
    String result = name + " Attendees\\n----------\\n";

    for (Attendee person : attendees) {
      result += person.getName() + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`EventTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Event.java Test")
public class EventTest {

  String messageGap = "\\n       ";
  ArrayList<Attendee> testAttendees;
  ArrayList<Attendee> testCancelled;
  ArrayList<Attendee> testResult;
  Event testEvent;
   
  @BeforeEach
  public void setup() {
    testAttendees = getTestAttendees();
    testCancelled = getRandomCancelled();
    testResult = getTestAttendees();
    testEvent = new Event("some event", testAttendees);
  }
   
  @Test
  @Order(1)
  @DisplayName("Removes all the attendees in the cancelled attendees list from the main list of attendees => ")
  public void testRemoveCancelledAttendees() {
    String message = "Traverse the list of cancelled attendees and check if they are in the main list of attendees.";
    message += "\\n        If a match is found, remove the attendee from the main list of attendees." + messageGap;
      
    getExpectedResult(testCancelled);
    testEvent.removeCancelledAttendees(testCancelled);

    assertEquals(testResult, testAttendees, message);
  }

  private ArrayList<Attendee> getTestAttendees() {
    ArrayList<Attendee> temp = new ArrayList<Attendee>();
    
    temp.add(new Attendee("Alice", 1));
    temp.add(new Attendee("Bob", 2));
    temp.add(new Attendee("Charlie", 3));
    temp.add(new Attendee("David", 4));
    temp.add(new Attendee("Eve", 5));

    return temp;
  }

  private ArrayList<Attendee> getRandomCancelled() {
    ArrayList<Attendee> temp = new ArrayList<Attendee>();

    for (int count = 0; count < 2; count++) {
      int randomIndex = (int)(Math.random() * testAttendees.size());
      Attendee attendee = testAttendees.get(randomIndex);
      temp.add(new Attendee(attendee.getName(), attendee.getId()));
    }

    return temp;
  }

  private void getExpectedResult(ArrayList<Attendee> cancelledAttendees) {
    for (int cancelledIndex = 0; cancelledIndex < cancelledAttendees.size(); cancelledIndex++) {
      Attendee cancelledAttendee = cancelledAttendees.get(cancelledIndex);
      
      for (int attendeeIndex = 0; attendeeIndex < testResult.size(); attendeeIndex++) {
        Attendee attendee = testResult.get(attendeeIndex);

        if (cancelledAttendee.equals(attendee)) {
          testResult.remove(attendeeIndex);
          attendeeIndex--;
        }
      }
    }
  }
  
}`}],dataFiles:[]},{name:`Practice: Traversing Multiple Lists (d)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    // Creates a list of books
    ArrayList<Book> books = new ArrayList<Book>();
    books.add(new Book("The Great Gatsby", "F. Scott Fitzgerald", "978-0743273565"));
    books.add(new Book("To Kill a Mockingbird", "Harper Lee", "978-0446310789"));
    books.add(new Book("The Catcher in the Rye", "J.D. Salinger", "978-0316769488"));
    books.add(new Book("Pride and Prejudice", "Jane Austen", "978-0141439518"));

    // Creates a SearchEngine object
    SearchEngine search = new SearchEngine(books);

    // Creates a list of search queries
    ArrayList<String> searchQueries = new ArrayList<String>();
    searchQueries.add("Catcher");
    searchQueries.add("Hamlet");
    searchQueries.add("Prejudice");
    searchQueries.add("Gatsby");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findSearchResults() method and print the results using the
     * resultsToString() method.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`Book.java`,text:`/*
 * Represents a book
 */
public class Book {
  
  private String title;      // The title of a book
  private String author;     // The author of a book
  private String isbn;       // The ISBN of a book

  /*
   * Sets title, author, and isbn to the specified values
   */
  public Book(String title, String author, String isbn) {
    this.title = title;
    this.author = author;
    this.isbn = isbn;
  }

  /*
   * Returns the title of the book
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the author of the book
   */
  public String getAuthor() {
    return author;
  }

  /*
   * Returns the ISBN of the book
   */
  public String getIsbn() {
    return isbn;
  }

  /*
   * Returns a String containing the information about the book
   */
  public String toString() {
    return title + " by " + author + " [ISBN: " + isbn + "]";
  }
  
}`},{path:`SearchEngine.java`,text:`import java.util.ArrayList;

/*
 * Represents a search engine for a collection of books
 */
public class SearchEngine {

  private ArrayList<Book> books;         // The list of books in the collection

  /*
   * Initializes books to the specified list of Book objects
   */
  public SearchEngine(ArrayList<Book> books) {
    this.books = books;
  }

  /*
   * Returns an ArrayList containing the location of the first occurence of
   * each search query in the searchQueries list
   */
  public ArrayList<Integer> findSearchResults(ArrayList<String> searchQueries) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Find the first occurence of each search query in the list of books and
     * add the index where the Book object was found to a new ArrayList. Return the 
     * new ArrayList containing the locations of each search query.
     * -----------------------------------------------------------------------------
     */
    
    
    
    return null;
  }

  /*
   * Returns true if the text contains the query
   */
  public boolean matchQuery(String text, String query) {
    int space = text.indexOf(" ");

    while (space >= 0) {
      String word = text.substring(0, space);

      if (word.equals(query)) {
        return true;
      }

      text = text.substring(space + 1);
      space = text.indexOf(" ");
    }

    if (text.equals(query)) {
      return true;
    }

    return false;
  }

  /*
   * Returns a String containing the list of results
   */
  public String resultsToString(ArrayList<Integer> results) {
    String text = "";

    for (int index = 0; index < results.size(); index++) {
      int currentIndex = results.get(index);

      if (currentIndex == -1) {
        text += "No result found.\\n";
      }
      else {
        Book currentBook = books.get(currentIndex);
        text += "(" + currentIndex + ") " + currentBook + "\\n";
      }
    }

    return text;
  }
  
}`}],validationFiles:[{path:`SearchEngineTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("SearchEngine.java Test")
public class SearchEngineTest {

  String messageGap = "\\n       ";
  ArrayList<Book> testBooks;
  SearchEngine testSearch;
   
  @BeforeEach
  public void setup() {
    testBooks = new ArrayList<Book>();

    testBooks.add(new Book("The Great Gatsby", "F. Scott Fitzgerald", "9780743273565"));
    testBooks.add(new Book("To Kill a Mockingbird", "Harper Lee", "9780446310789"));
    testBooks.add(new Book("1984", "George Orwell", "9780451524935"));
    testBooks.add(new Book("Pride and Prejudice", "Jane Austen", "9780141439518"));
    testBooks.add(new Book("The Hobbit", "J.R.R. Tolkien", "9780261103283"));

    testSearch = new SearchEngine(testBooks);
  }
   
  @Test
  @Order(1)
  @DisplayName("findSearchResults() returns an ArrayList containing the indexes where each matching Book title is found => ")
  public void testFindSearchResultsMatchesTitle() {
    String message = "Traverse the list of search queries and use the matchQuery() method to check if a Book title";
    message += "\\n        contains the search query text. If this is true, add the location of the Book object in the new list."+ messageGap;
      
    ArrayList<String> testQueries = new ArrayList<String>();
    testQueries.add("Gatsby");
    testQueries.add("Mockingbird");
    testQueries.add("Pride");

    ArrayList<Integer> expected = new ArrayList<Integer>();
    expected.add(0);
    expected.add(1);
    expected.add(3);

    ArrayList<Integer> actual = testSearch.findSearchResults(testQueries);

    assertEquals(expected, actual, message);
  }
   
  @Test
  @Order(2)
  @DisplayName("findSearchResults() returns an ArrayList containing the indexes where each matching Book author is found => ")
  public void testFindSearchResultsMatchesAuthor() {
    String message = "Traverse the list of search queries and use the matchQuery() method to check if a Book author";
    message += "\\n        contains the search query text. If this is true, add the location of the Book object in the new list."+ messageGap;
      
    ArrayList<String> testQueries = new ArrayList<String>();
    testQueries.add("Scott");
    testQueries.add("Austen");
    testQueries.add("Harper");

    ArrayList<Integer> expected = new ArrayList<Integer>();
    expected.add(0);
    expected.add(3);
    expected.add(1);

    ArrayList<Integer> actual = testSearch.findSearchResults(testQueries);

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("findSearchResults() returns an ArrayList containing the indexes where each matching Book ISBN is found => ")
  public void testFindSearchResultsMatchesIsbn() {
    String message = "Traverse the list of search queries and use the matchQuery() method to check if a Book ISBN";
    message += "\\n        contains the search query text. If this is true, add the location of the Book object in the new list."+ messageGap;
      
    ArrayList<String> testQueries = new ArrayList<String>();
    testQueries.add("9780446310789");
    testQueries.add("9780743273565");
    testQueries.add("9780261103283");

    ArrayList<Integer> expected = new ArrayList<Integer>();
    expected.add(1);
    expected.add(0);
    expected.add(4);

    ArrayList<Integer> actual = testSearch.findSearchResults(testQueries);

    assertEquals(expected, actual, message);
  }

  @Test
  @Order(3)
  @DisplayName("findSearchResults() returns an ArrayList that contains -1 if a query does not match a Book object => ")
  public void testFindSearchResultsAddsNegativeOne() {
    String message = "Traverse the list of search queries and use the matchQuery() method to check if a Book title, author,";
    message += "\\n        or ISBN contains the search query text. If this is false, add -1 to the new list."+ messageGap;
      
    ArrayList<String> testQueries = new ArrayList<String>();
    testQueries.add("some text");
    testQueries.add("some other text");
    testQueries.add("different text");

    ArrayList<Integer> expected = new ArrayList<Integer>();
    expected.add(-1);
    expected.add(-1);
    expected.add(-1);

    ArrayList<Integer> actual = testSearch.findSearchResults(testQueries);

    assertEquals(expected, actual, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Searching Algorithms (a)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    GuessingGame guess = new GuessingGame();

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the guessNumber() method.
     * -----------------------------------------------------------------------------
     */
    
    
    
  }
}`},{path:`GuessingGame.java`,text:`import java.util.Scanner;

/*
 * Represents a guessing game
 */
public class GuessingGame {

  private Scanner input;      // The Scanner to read the user input

  /*
   * Initializes the Scanner input
   */
  public GuessingGame() {
    input = new Scanner(System.in);
  }

  /*
   * Uses the binary search algorithm to guess the user's number
   */
  public boolean guessNumber(int start, int end) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement the binary search algorithm to guess the user's number.
     * -----------------------------------------------------------------------------
     */
    
    
    return false;
  }

  /*
   * Asks the user if their number equals guess
   * and returns the response from the user
   */
  public String promptUser(int guess) {
    System.out.print("Is the number " + guess + "? ");
    String answer = input.nextLine();
    return answer;
  }

  /*
   * Asks the user if their number is greater than guess
   * and returns the response from the user
   */
  public String isGreaterThan(int guess) {
    System.out.print("Is the number greater than " + guess + "? ");
    String answer = input.nextLine();
    return answer;
  }
  
}`}],validationFiles:[{path:`GuessingGameTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;
import static org.easymock.EasyMock.*;

import java.util.Scanner;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("GuessingGame.java Test")
public class GuessingGameTest {

  String messageGap = "\\n       ";
  GuessingGame testGame;
  Scanner testScanner;
   
  @BeforeEach
  public void setup() {
    testGame = partialMockBuilder(GuessingGame.class)
      .addMockedMethod("promptUser")
      .createMock();
  }
   
  @Test
  @Order(1)
  @DisplayName("guessNumber() guesses a user's number => ")
  public void testGuessNumberFindsNumber() {
    String message = "The error message to display." + messageGap;

    expect(testGame.promptUser(5)).andReturn("yes");
    replay(testGame);

    boolean result = testGame.guessNumber(1, 10);

    verify(testGame);
    assertTrue(result, message);
  }
  
}`}],dataFiles:[]},{name:`Practice: Searching Algorithms (b)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] students = {"Blake", "Claire", "Dominic", "Elias", "Leah", "Natalie", "Sophia"};

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the findPartner() method and print the result.
     * -----------------------------------------------------------------------------
     */
    

    
    
  }
}`},{path:`PairFinder.java`,text:`/*
 * Manages a list of students
 */
public class PairFinder {

  /*
   * Finds the closest matching student that comes after target
   */
  public static String findPartner(String[] students, String target) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement the binary search algorithm to find and return the name that
     * is the closest and comes after the name to find.
     * -----------------------------------------------------------------------------
     */

    
    return "";
  }
  
}`}],validationFiles:[{path:`PairFinderTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import java.util.Random;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("PairFinder.java Test")
public class PairFinderTest {

  String messageGap = "\\n       ";
  String[] testStudents;
  
  @BeforeEach
  public void setup() {
    testStudents = generateRandomStrings();
  }
   
  @Test
  @Order(1)
  @DisplayName("findPartner() returns the name that is closest and comes after the name to find => ")
  public void testFindPartnerFindsClosestName() {
    String message = "Use a binary search to find and return the student that is closest to the target student." + messageGap;
      
    String randomStudent = testStudents[(int)(Math.random() * testStudents.length)];
    String expected = getExpectedResult(testStudents, randomStudent);
    String actual = PairFinder.findPartner(testStudents, randomStudent);

    assertEquals(expected, actual, message);
  }
  
  private String[] generateRandomStrings() {
    String alphabet = "abcdefghijklmnopqrstuvwxyz";
    String[] randomStrings = new String[10];
    Random rand = new Random();
    
    for (int i = 0; i < randomStrings.length; i++) {
      String randomString = "";
        
      for (int j = 0; j < 3; j++) {
        int index = rand.nextInt(alphabet.length());
        randomString += alphabet.charAt(index);
      }
        
      randomStrings[i] = randomString;
    }

    Arrays.sort(randomStrings);
    return randomStrings;
  }

  private String getExpectedResult(String[] students, String target) {
    int low = 0;
    int high = students.length;

    while (low < high) {
      int mid = (low + high) / 2;

      if (students[mid].compareTo(target) < 0) {
        low = mid + 1;
      }
      else {
        high = mid;
      }
    }

    return students[low % students.length];
  }
  
}`}],dataFiles:[]},{name:`Practice: Searching Algorithms (c)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    ArrayList<Integer> ticketNumbers = FileReader.toIntList("ticketnumbers.txt");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the missingSmallest() method and print the result.
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
  
}`},{path:`TicketGenerator.java`,text:`import java.util.ArrayList;

/*
 * Manages a list of ticket numbers
 */
public class TicketGenerator {

  /*
   * Finds and returns the smallest missing value in tickets
   */
  public static int missingSmallest(ArrayList<Integer> tickets, int start, int end) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement the binary search algorithm to find the smallest missing value.
     * -----------------------------------------------------------------------------
     */

    
    return -1;
  }
  
}`}],validationFiles:[{path:`TicketGeneratorTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;
import java.util.Collections;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("TicketGenerator.java Test")
public class TicketGeneratorTest {

  String messageGap = "\\n       ";
  ArrayList<Integer> testNumbers;
   
  @BeforeEach
  public void setup() {
    testNumbers = generateRandomNumbers();
  }
   
  @Test
  @Order(1)
  @DisplayName("missingSmallest() finds and returns the smallest missing number in tickets => ")
  public void testMissingSmallest() {
    String message = "Use a binary search to find and return the smallest missing number." + messageGap;

    int expected = getExpectedResult(testNumbers, 0, testNumbers.size());
    int actual = TicketGenerator.missingSmallest(testNumbers, 0, testNumbers.size());

    assertEquals(expected, actual, message);
  }

  private ArrayList<Integer> generateRandomNumbers() {
    ArrayList<Integer> temp = new ArrayList<Integer>();

    for (int count = 0; count < 10; count++) {
      temp.add((int)(Math.random() * 20) + 1);
    }

    Collections.sort(temp);
    return temp;
  }

  private int getExpectedResult(ArrayList<Integer> numbers, int start, int end) {
    int mid = (start + end) / 2;

    if (start > end) {
      return start;
    }

    if (numbers.get(mid) == mid) {
      return getExpectedResult(numbers, mid + 1, end);
    }
    else {
      return getExpectedResult(numbers, start, mid - 1);
    }
    
  }
   
}`}],dataFiles:[{path:`tickets.txt`,text:`0
1
2
3
4
6
7
8
9
10`}]},{name:`Practice: Searching Algorithms (d)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    ArrayList<VideoGame> games = GamesManager.getVideoGames("games.txt");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the gamesByPublisher() method and print the result.
     * -----------------------------------------------------------------------------
     */
    

    
    
  }
}`},{path:`GamesManager.java`,text:`import java.util.ArrayList;
import java.util.Scanner;
import java.io.File;
import java.io.FileNotFoundException;

/*
 * Analyzes information about video games
 */
public class GamesManager {

  /*
   * Finds the first or last occurrence of a target publisher
   */
  public static int gamesByPublisher(ArrayList<VideoGame> games, String target, boolean searchFirst) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement the binary search algorithm to count the number of video
     * games by the target publisher.
     * -----------------------------------------------------------------------------
     */
    
    
    return -1;
  }

  /*
   * Reads the video game information from a file
   * and returns an ArrayList of VideoGames
   */
  public static ArrayList<VideoGame> getVideoGames(String filename) {
    File myFile = new File(filename);
    Scanner fileReader = createScanner(myFile);

    ArrayList<VideoGame> tempGames = new ArrayList<VideoGame>();

    while (fileReader.hasNextLine()) {
      String currentLine = fileReader.nextLine();
      VideoGame currentGame = createVideoGame(currentLine);
      tempGames.add(currentGame);
    }

    fileReader.close();
    return tempGames;
  }

  /*
   * Returns a Scanner object to read a specified File
   */
  public static Scanner createScanner(File theFile) {
    Scanner tempScanner = null;

    try {
      tempScanner = new Scanner(theFile);
    } catch(FileNotFoundException error) {
      System.out.println("File not found.");
    }

    return tempScanner;
  }

  /*
   * Gets the video game title and publisher from the
   * currentLine and returns a VideoGame object
   */
  public static VideoGame createVideoGame(String currentLine) {
    int comma = currentLine.indexOf(",");

    String gameTitle = currentLine.substring(0, comma);
    String gamePublisher = currentLine.substring(comma + 1);

    VideoGame temp = new VideoGame(gameTitle, gamePublisher);
    return temp;
  }
  
}`},{path:`VideoGame.java`,text:`/*
 * Represents a video game
 */
public class VideoGame {

  private String title;        // The title of a game
  private String publisher;    // The publisher of a game

  /*
   * Sets title and publisher to the specified values
   */
  public VideoGame(String title, String publisher) {
    this.title = title;
    this.publisher = publisher;
  }

  /*
   * Returns the title of the game
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the publisher of the game
   */
  public String getPublisher() {
    return publisher;
  }
  
}`}],validationFiles:[{path:`GamesManagerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("GamesManager.java Test")
public class GamesManagerTest {

  String messageGap = "\\n       ";
  ArrayList<VideoGame> testGames;
   
  @BeforeEach
  public void setup() {
    testGames = new ArrayList<VideoGame>();

    testGames.add(new VideoGame("Minecraft", "Mojang Studios"));
    testGames.add(new VideoGame("Tetris", "Electronic Arts"));
    testGames.add(new VideoGame("Wii Sports", "Nintendo"));
    testGames.add(new VideoGame("Super Mario Bros.", "Nintendo"));
    testGames.add(new VideoGame("Pac-Man", "Namco"));
    testGames.add(new VideoGame("Mario Kart Wii", "Nintendo"));
    testGames.add(new VideoGame("The Elder Scrolls V: Skyrim", "Bethesda Softworks"));
  }
   
  @Test
  @Order(1)
  @DisplayName("gamesByPublisher() returns the index location of the first occurence => ")
  public void testGamesByPublisherFirstOccurrence() {
    String message = "Use a binary search to find the first occurrence of a video game";
    message += "\\n        by a target publisher in games." + messageGap;
      
    int result = GamesManager.gamesByPublisher(testGames, "Nintendo", true);

    assertEquals(2, result, message);
  }

  @Test
  @Order(1)
  @DisplayName("gamesByPublisher() returns the index location of the last occurence => ")
  public void testGamesByPublisherLastOccurrence() {
    String message = "Use a binary search to find the last occurrence of a video game";
    message += "\\n        by a target publisher in games." + messageGap;
      
    int result = GamesManager.gamesByPublisher(testGames, "Nintendo", false);

    assertEquals(5, result, message);
  }
   
}`}],dataFiles:[{path:`games.txt`,text:`The Elder Scrolls V: Skyrim,Bethesda Softworks
Tetris,Electronic Arts
Minecraft,Mojang Studios
Pac-Man,Namco
Mario Kart Wii,Nintendo
Super Mario Bros.,Nintendo
Wii Sports,Nintendo`}]},{name:`Practice: Sorting Algorithms (a)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int[] firstSet = {8, 4, 10, 2, 6};
    int[] secondSet = {-3, 5, 0, -6, 3};

    System.out.println("----------First Set----------");
    System.out.println(Sorter.arrayToString(firstSet));

    System.out.println("\\n----------Second Set----------");
    System.out.println(Sorter.arrayToString(secondSet));

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the squareList() and sortList() methods and print the updated arrays.
     * -----------------------------------------------------------------------------
     */
    


    
    
  }
}`},{path:`Sorter.java`,text:`/*
 * Manages and sorts lists
 */
public class Sorter {

  /*
   * Returns an array containing the squares of each value in data.
   */
  public static int[] squareList(int[] data) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Return a new array containing the squares of each value in data.
     * -----------------------------------------------------------------------------
     */


    return null;
  }

  /*
   * Sorts an array from smallest to largest value
   */
  public static void sortList(int[] data) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Implement a sorting algorithm to sort the 1D array data.
     * -----------------------------------------------------------------------------
     */
    

    
  }

  /*
   * Returns a String containing each element in data
   */
  public static String arrayToString(int[] data) {
    String result = "";
    
    for (int value : data) {
      result += value + " ";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`SorterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Sorter.java Test")
public class SorterTest {

  String messageGap = "\\n       ";
  int[] testNumbers;
  int[] testSquared;
  int[] testSorted;
  
  @BeforeEach
    public void setup() {
      testNumbers = new int[10];
      testSquared = new int[10];
      testSorted = new int[10];
      setupArrays();
    }

  @Test
  @Order(1)
  @DisplayName("squareList() returns a new 1D array containing the squares of each value in data => ")
  public void testSquaredTrue() {
    String message = "squareList() did not returns the expected 1D array containing the squares of a sample array\\n";
    int[] result = Sorter.squareList(testNumbers);
    assertArrayEquals(testSquared, result, message);  
  }

  @Test
  @Order(2)
  @DisplayName("sortList() sorts the 1D array data from the smallest to the largest value => ")
  public void testSortedTrue() {
    String message = "sortList() did not sort a sample array from the expected smallest to the largest values\\n";
    Sorter.sortList(testNumbers);
    assertArrayEquals(testSorted, testNumbers, message);  
  }

  private void setupArrays() {
    for (int index = 0; index < testNumbers.length; index++) {
      int randomNumber = (int)(Math.random() * 30) + 5;
      testNumbers[index] = randomNumber;
      testSorted[index] = randomNumber;
    }

    for (int index = 0; index < testSquared.length; index++) {
      testSquared[index] = (int)(Math.pow(testNumbers[index], 2));
    }

    Arrays.sort(testSorted);
  }
  
}`}],dataFiles:[]},{name:`Practice: Sorting Algorithms (b)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] attendees = {"Kaia", "Payton", "Amira", "Kaia", "Kaleb",
                          "Juliana", "Noelle", "Kaia", "Sara", "Presley"};
    
    System.out.println("----------Unsorted List of Attendees----------");
    System.out.println(EventManager.arrayToString(attendees));

    System.out.println("\\n----------Sorted List of Attendees----------");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the sortAttendees() method and print the updated list.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`EventManager.java`,text:`/*
 * Manages lists of attendees that have attended events
 */
public class EventManager {

  /*
   * Sorts the list of attendees in alphabetical (A to Z) order
   */
  public static void sortAttendees(String[] attendees) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Sort the 1D array of attendees using a selection, insertion, or merge sort.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns a String containing each attendee in attendees
   */
  public static String arrayToString(String[] attendees) {
    String result = "";
    
    for (String person : attendees) {
      result += person + " ";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`EventManagerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Arrays;
import java.util.Random;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("EventManager.java Test")
public class EventManagerTest {

  String messageGap = "\\n       ";
  String[] testAttendees;
  String[] testResult;
  
  @BeforeEach
  public void setup() {
    testAttendees = generateRandomStrings();
    testResult = copyTestStrings();
  }

  @Test
  @Order(1)
  @DisplayName("sortAttendees() sorts the 1D array of attendees in alphabetical (A to Z) order => ")
  public void testSortAttendees() {
    String message = "Use a selection, insertion, or merge sort to sort the attendees in alphabetical order." + messageGap;
    
    Arrays.sort(testResult);
    EventManager.sortAttendees(testAttendees);
    
    assertArrayEquals(testResult, testAttendees, message);  
  }

  private String[] generateRandomStrings() {
    String alphabet = "abcdefghijklmnopqrstuvwxyz";
    String[] randomStrings = new String[10];
    Random rand = new Random();
    
    for (int i = 0; i < randomStrings.length; i++) {
      String randomString = "";
        
      for (int j = 0; j < 3; j++) {
        int index = rand.nextInt(alphabet.length());
        randomString += alphabet.charAt(index);
      }
        
      randomStrings[i] = randomString;
    }
    
    return randomStrings;
  }

  private String[] copyTestStrings() {
    String[] temp = new String[testAttendees.length];

    for (int index = 0; index < temp.length; index++) {
      temp[index] = testAttendees[index];
    }

    return temp;
  }
  
}`}],dataFiles:[]},{name:`Practice: Sorting Algorithms (c)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    String[] words = {"is2", "sentence4", "This1", "a3"};

    System.out.println("----------Unsorted Words----------");
    System.out.println(WordSorter.wordsToString(words));

    System.out.println("\\n----------Sorted Into Sentence----------");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the sortWords() method and print the updated array. Then call the
     * createSentence() method and print the result.
     * -----------------------------------------------------------------------------
     */
    

    
    
  }
}`},{path:`WordSorter.java`,text:`/*
 * Manages lists of words
 */
public class WordSorter {

  /*
   * Sorts the wordList in order according to the number
   * at the end of each word
   */
  public static void sortWords(String[] wordList) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Use a selection, insertion, or merge sort to sort the words in the
     * 1D array wordList in order based on the number at the end of each word.
     * -----------------------------------------------------------------------------
     */


    
  }

  /*
   * Returns the number at the end of the String
   */
  public static int getNumber(String currentWord) {
    String number = currentWord.substring(currentWord.length() - 1);
    return Integer.parseInt(number);
  }

  /*
   * Returns a String containing each word in wordList without the 
   * number that appears at the end of each String
   */
  public static String createSentence(String[] wordList) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Traverse the 1D array wordList and obtain a substring of each word
     * without the number at the end. Concatenate each substring to a new String
     * and return the String.
     * -----------------------------------------------------------------------------
     */

    
    return "";
  }

  /*
   * Returns a String containing each word in wordList
   */
  public static String wordsToString(String[] wordList) {
    String result = "";
    
    for (String word : wordList) {
      result += word + " ";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`WordSorterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.Random;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("WordSorter.java Test")
public class WordSorterTest {

  String messageGap = "\\n       ";
  String[] testWords;
  String[] testResult;
  
  @BeforeEach
  public void setup() {
    testWords = generateRandomStrings();
    testResult = copyTestStrings();
  }

  @Test
  @Order(1)
  @DisplayName("sortWords() method sorts the words in wordList in order based on the number at the end of each word => ")
  public void testSortWordsEquals() {
    String message = "The sortWords() method did not sort as expected given the sample array." + messageGap;

    getExpectedResult(testResult);
    WordSorter.sortWords(testWords);  
    
    assertArrayEquals(testResult, testWords, message);  
  }

  @Test
  @Order(2)
  @DisplayName("createSentence() method returns a String of the words without the numbers at the end of each word => ")
  public void testCreateSentenceEquals() {
    String message = "The createSentence() method did not return the expected sentence given a sample array." + messageGap;

    getExpectedResult(testResult);
    String expected = getExpectedSentence(testResult);
    WordSorter.sortWords(testWords);   
    String actual = WordSorter.createSentence(testWords);
    
    assertEquals(expected, actual, message);  
  }

  private String[] generateRandomStrings() {
    String alphabet = "abcdefghijklmnopqrstuvwxyz";
    String numbers = "1234567890";
    
    String[] randomStrings = new String[10];
    Random rand = new Random();
    
    for (int i = 0; i < randomStrings.length; i++) {
      String randomString = "";
        
      for (int j = 0; j < 3; j++) {
        int randLetterIndex = rand.nextInt(alphabet.length());
        randomString += alphabet.charAt(randLetterIndex);
      }

      int randNumberIndex = rand.nextInt(numbers.length());
      randomString += numbers.charAt(randNumberIndex);
        
      randomStrings[i] = randomString;
    }
    
    return randomStrings;
  }

  private String[] copyTestStrings() {
    String[] temp = new String[testWords.length];

    for (int index = 0; index < temp.length; index++) {
      temp[index] = testWords[index];
    }

    return temp;
  }

  private void getExpectedResult(String[] wordList) {
    for (int index = 1; index < wordList.length; index++) {
      String currentWord = wordList[index];
      int currentValue = WordSorter.getNumber(currentWord);
      int next = index - 1;

      while ((next >= 0) && (WordSorter.getNumber(wordList[next]) > currentValue)) {
        wordList[next + 1] = wordList[next];
        next--;
      }

      wordList[next + 1] = currentWord;
    }
  }

  public String getExpectedSentence(String[] wordList) {
    String sentence = "";

    for (int index = 0; index < wordList.length; index++) {
      String currentWord = wordList[index];
      sentence = sentence + currentWord.substring(0, currentWord.length() - 1) + " ";
    }

    return sentence;
  }
  
}`}],dataFiles:[]},{name:`Practice: Sorting Algorithms (d)`,lesson:`Lesson 7: Searching and Sorting`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.ArrayList;

public class Main {
  public static void main(String[] args) {

    ArrayList<String> stateData = FileReader.toStringList("populations.txt");

    ArrayList<String> stateNames = StateSorter.getStates(stateData);
    ArrayList<Integer> populations = StateSorter.getData(stateData);

    System.out.println("----------Unsorted List of States and Populations----------");
    System.out.println(StateSorter.dataToString(stateNames, populations));

    System.out.println("\\n----------Sorted List of States and Populations----------");

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the sortStates() method and print the updated list.
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
  
}`},{path:`StateSorter.java`,text:`import java.util.ArrayList;

/*
 * Manages lists of data about states
 */
public class StateSorter {

  /*
   * Sorts the list of state names and data in increasing order
   */
  public static void sortStates(ArrayList<String> stateNames, ArrayList<Integer> data) {
    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Sort stateNames and data in increasing order based on the values in data.
     * -----------------------------------------------------------------------------
     */
    

    
  }

  /*
   * Returns an ArrayList containing the name of the states
   */
  public static ArrayList<String> getStates(ArrayList<String> statesData) {
    ArrayList<String> stateNames = new ArrayList<String>();

    for (int index = 0; index < statesData.size(); index++) {
      String currentData = statesData.get(index);
      int comma = currentData.indexOf(",");
      String currentState = currentData.substring(0, comma);
      stateNames.add(currentState);
    }
    
    return stateNames;
  }

  /*
   * Returns an ArrayList containing the data for each state
   */
  public static ArrayList<Integer> getData(ArrayList<String> statesData) {
    ArrayList<Integer> stateData = new ArrayList<Integer>();

    for (int index = 0; index < statesData.size(); index++) {
      String currentData = statesData.get(index);
      int comma = currentData.indexOf(",");
      int currentValue = Integer.parseInt(currentData.substring(comma + 1));
      stateData.add(currentValue);
    }

    return stateData;
  }

  /*
   * Returns a String containing each state name and their population data
   */
  public static String dataToString(ArrayList<String> stateNames, ArrayList<Integer> data) {
    String result = "";
    
    for (int index = 0; index < stateNames.size(); index++) {
      result += stateNames.get(index) + ": " + data.get(index) + "\\n";
    }

    return result;
  }
  
}`}],validationFiles:[{path:`StateSorterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("StateSorter.java Test")
public class StateSorterTest {

  ArrayList<String> states = new ArrayList<String>();
  ArrayList<Integer> populations = new ArrayList<Integer>();
  ArrayList<String> stateSort = new ArrayList<String>();
  ArrayList<Integer> populationSort = new ArrayList<Integer>();
  String messageGap = "\\n       ";

  @BeforeEach
    public void setup() {
      states.add("Alabama");
      states.add("Alaska");
      states.add("Arizona");
      states.add("Arkansas");
      states.add("California");
      populations.add(4833722);
      populations.add(735132);
      populations.add(6626624);
      populations.add(2959373);
      populations.add(38332521);
      stateSort.add("Alaska");
      stateSort.add("Arkansas");
      stateSort.add("Alabama");
      stateSort.add("Arizona");
      stateSort.add("California");
      populationSort.add(735132);
      populationSort.add(2959373);
      populationSort.add(4833722);
      populationSort.add(6626624);
      populationSort.add(38332521);
    }

  @Test
  @Order(1)
  @DisplayName("sortStates() sorts stateNames in increasing order by the values in population data => ")
  public void testSortStatesEqual() {
    String message = "StateSorter.sortStates() does not sort the states by population as expected for a sample array." + messageGap;
    StateSorter.sortStates(states, populations);
    assertEquals(stateSort, states, message);  
  }

  @Test
  @Order(2)
  @DisplayName("sortStates() method sorts population data in increasing order by the values in data => ")
  public void testSortPopulationEqual() {
    String message = "StateSorter.sortStates() does not sort population data in increasing order as expected for a sample array." + messageGap;
    StateSorter.sortStates(states, populations);
    assertEquals(populationSort, populations, message);  
  }  

}`}],dataFiles:[{path:`populations.txt`,text:`Alabama,4833722
Alaska,735132
Arizona,6626624
Arkansas,2959373
California,38332521
Colorado,5268367
Connecticut,3596080
Delaware,925749
Florida,19552860
Georgia,9992167
Hawaii,1404054
Idaho,1612136
Illinois,12882135
Indiana,6570902
Iowa,3090416
Kansas,2893957
Kentucky,4395295
Louisiana,4625470
Maine,1328302
Maryland,5928814
Massachusetts,6692824
Michigan,9895622
Minnesota,5420380
Mississippi,2991207
Missouri,6044171
Montana,1015165
Nebraska,1868516
Nevada,2790136
New Hampshire,1323459
New Jersey,8899339
New Mexico,2085287
New York,19651127
North Carolina,9848060
North Dakota,723393
Ohio,11570808
Oklahoma,3850568
Oregon,3930065
Pennsylvania,12773801
Rhode Island,1051511
South Carolina,4774839
South Dakota,844877
Tennessee,6495978
Texas,26448193
Utah,2900872
Vermont,626630
Virginia,8260405
Washington,6971406
West Virginia,1854304
Wisconsin,5742713
Wyoming,582658`}]},{name:`Creative Coding with the Console Project`,lesson:`Lesson 8: Project Development`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with the Console Project`,lesson:`Lesson 10a: Creative Coding with the Console Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
  
}`}],validationFiles:[],dataFiles:[]},{name:`Creative Coding with the Console Project`,lesson:`Lesson 10b: Creative Coding with the Console Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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