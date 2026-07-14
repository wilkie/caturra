var e=[{name:`Predict and Run: Attributes`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Content canvas = new Content();
    Movie findingNemo = new Movie();
    TVShow strangerThings = new TVShow();

    System.out.println("Content Year: " + canvas.year);
    System.out.println("Movie Title: " + findingNemo.title);
    System.out.println("TV Show Episodes: " + strangerThings.numEpisodes);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean", "int", "double", "String"};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result;
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie {

  private String title;     // The title of the movie
  private int year;         // The year the movie was released
  private int runningTime;  // The length of the movie in minutes
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow {

  private String title;     // The title of the TV show
  private int year;         // The year the TV show was released
  private int numEpisodes;  // The number of episodes
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Classes and Attributes #1`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Content canvas = new Content();
    Movie findingNemo = new Movie();
    TVShow strangerThings = new TVShow();

    System.out.println("Content Year: " + canvas.year);
    System.out.println("Movie Title: " + findingNemo.title);
    System.out.println("TV Show Episodes: " + strangerThings.numEpisodes);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean", "int", "double", "String"};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result;
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie {

  private String title;     // The title of the movie
  private int year;         // The year the movie was released
  private int runningTime;  // The length of the movie in minutes
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow {

  private String title;     // The title of the TV show
  private int year;         // The year the TV show was released
  private int numEpisodes;  // The number of episodes
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Classes and Attributes #2`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Content canvas = new Content();
    Movie findingNemo = new Movie();
    TVShow strangerThings = new TVShow();

    System.out.println("Content Year: " + canvas.year);
    System.out.println("Movie Title: " + findingNemo.title);
    System.out.println("TV Show Episodes: " + strangerThings.numEpisodes);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean", "int", "double", "String"};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result;
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie {

  private String title;     // The title of the movie
  private int year;         // The year the movie was released
  private int runningTime;  // The length of the movie in minutes
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow {

  private String title;     // The title of the TV show
  private int year;         // The year the TV show was released
  private int numEpisodes;  // The number of episodes
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Classes and Attributes #3`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Content canvas = new Content();
    Movie findingNemo = new Movie();
    TVShow strangerThings = new TVShow();

    System.out.println("Content Year: " + canvas.year);
    System.out.println("Movie Title: " + findingNemo.title);
    System.out.println("TV Show Episodes: " + strangerThings.numEpisodes);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean", "int", "double", "String"};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result;
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie {

  private String title;     // The title of the movie
  private int year;         // The year the movie was released
  private int runningTime;  // The length of the movie in minutes
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow {

  private String title;     // The title of the TV show
  private int year;         // The year the TV show was released
  private int numEpisodes;  // The number of episodes
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: Declaring Instance Variables (a)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an Instrument object, then print the instance variables for
     * the Instrument object to the console using AttributesHelper.printAttributes().
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Instrument.java`,text:`/*
 * Represents an instrument at a music store
 */
public class Instrument {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare instance variables for the type of instrument, the price of an
   * instrument, and whether or not an instrument has strings.
   * -----------------------------------------------------------------------------
   */


  
  
}`}],validationFiles:[{path:`InstrumentTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Instrument.java Test")
public class InstrumentTest {
    
  String message;
  String messageGap = "\\n       ";
  Instrument testInstrument;
  Class instrumentClass;
  ArrayList<String> instrumentFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testInstrument = new Instrument();
    instrumentClass = testInstrument.getClass();
  
    instrumentFields = AttributesHelper.getClassFieldsList(instrumentClass, "Instrument");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the type of instrument => ")
  public void testTypeInstanceVariable() {
    String typeMessage = "The type of instrument should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(instrumentFields, "String", "Instrument");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the price => ")
  public void testPriceInstanceVariable() {
    String typeMessage = "The price should be of type double." + messageGap;
  
    String actualField = AttributesHelper.findField(instrumentFields, "double", "Instrument");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("double", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Declare an instance variable for whether or not an instrument has strings => ")
  public void testHasStringsInstanceVariable() {
    String typeMessage = "Whether or not an instrument has strings should be of type boolean." + messageGap;
  
    String actualField = AttributesHelper.findField(instrumentFields, "boolean", "Instrument");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("boolean", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Declaring Instance Variables (b)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Recipe object, then print the instance variables for
     * the Recipe object to the console using AttributesHelper.printAttributes().
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Recipe.java`,text:`/*
 * Represents a recipe
 */
public class Recipe {

  /* --------------------------- TO DO ---------------------------
   * ✅ Declare instance variables for the type of meal, the
   * number of servings, and whether or not it is healthy.
   * -------------------------------------------------------------
   */


  
  
}`}],validationFiles:[{path:`RecipeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Recipe.java Test")
public class RecipeTest {
    
  String message;
  String messageGap = "\\n       ";
  Recipe testRecipe;
  Class recipeClass;
  ArrayList<String> recipeFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testRecipe = new Recipe();
    recipeClass = testRecipe.getClass();
  
    recipeFields = AttributesHelper.getClassFieldsList(recipeClass, "Recipe");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the type of meal => ")
  public void testTypeInstanceVariable() {
    String typeMessage = "The type of meal should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(recipeFields, "String", "Recipe");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for number of servings => ")
  public void testNumServingsInstanceVariable() {
    String typeMessage = "The number of servings should be of type int." + messageGap;
  
    String actualField = AttributesHelper.findField(recipeFields, "int", "Recipe");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("int", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Declare an instance variable for whether or not a recipe is healthy => ")
  public void testIsHealthyInstanceVariable() {
    String typeMessage = "Whether or not a recipe is healthy should be of type boolean." + messageGap;
  
    String actualField = AttributesHelper.findField(recipeFields, "boolean", "Recipe");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("boolean", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Declaring Instance Variables (c)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate an Event object, then print the instance variables for
     * the Event object to the console using AttributesHelper.printAttributes().
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() - 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Event.java`,text:`/*
 * Represents an event
 */
public class Event {

  /* --------------------------- TO DO ---------------------------
   * ✅ Declare instance variables for the name of an event, whether
   * or not an event is online, and the ticket price of an event.
   * -------------------------------------------------------------
   */


  
  
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
    
  String message;
  String messageGap = "\\n       ";
  Event testEvent;
  Class eventClass;
  ArrayList<String> eventFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testEvent = new Event();
    eventClass = testEvent.getClass();
  
    eventFields = AttributesHelper.getClassFieldsList(eventClass, "Event");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the name of an event => ")
  public void testNameInstanceVariable() {
    String typeMessage = "The name of an event should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(eventFields, "String", "Event");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for whether or not it is an online event => ")
  public void testIsOnlineInstanceVariable() {
    String typeMessage = "Whether or not it is an online event should be of type boolean." + messageGap;
  
    String actualField = AttributesHelper.findField(eventFields, "boolean", "Event");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("boolean", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Declare an instance variable for the ticket price => ")
  public void testPriceInstanceVariable() {
    String typeMessage = "The price of a ticket should be of type double." + messageGap;
  
    String actualField = AttributesHelper.findField(eventFields, "double", "Event");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("double", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Declaring Instance Variables (d)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Book object, then print the instance variables for
     * the Book object to the console using AttributesHelper.printAttributes().
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Book.java`,text:`/*
 * Represents a book
 */
public class Book {

  /* ----------------------------- TO DO -----------------------------
   * ✅ Declare instance variables for the title of a book, the number
   * of pages in a book, and whether or not a book is available.
   * -----------------------------------------------------------------
   */


  
  
}`}],validationFiles:[{path:`BookTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Book.java Test")
public class BookTest {
    
    String message;
    String messageGap = "\\n       ";
    Book testBook;
    Class bookClass;
    ArrayList<String> bookFields;
    String accessMessage;

    @BeforeEach
    public void setup() {
        accessMessage = "Instance variables should be private." + messageGap;

        testBook = new Book();
        bookClass = testBook.getClass();

        bookFields = AttributesHelper.getClassFieldsList(bookClass, "Book");
    }

    @Test
    @Order(1)
    @DisplayName("Declare an instance variable for the title => ")
    public void testTitleInstanceVariable() {
        String typeMessage = "The title of a book should be of type String." + messageGap;

        String actualField = AttributesHelper.findField(bookFields, "String", "Book");
        String actualType = AttributesHelper.getFieldType(actualField);
        String actualAccess = AttributesHelper.getAccess(actualField);

        assertEquals("String", actualType, typeMessage);
        assertEquals("private", actualAccess, accessMessage);
    }

    @Test
    @Order(2)
    @DisplayName("Declare an instance variable for number of pages => ")
    public void testNumPagesInstanceVariable() {
        String typeMessage = "The number of pages should be of type int." + messageGap;

        String actualField = AttributesHelper.findField(bookFields, "int", "Book");
        String actualType = AttributesHelper.getFieldType(actualField);
        String actualAccess = AttributesHelper.getAccess(actualField);

        assertEquals("int", actualType, typeMessage);
        assertEquals("private", actualAccess, accessMessage);
    }

    @Test
    @Order(3)
    @DisplayName("Declare an instance variable for whether or not a book is available => ")
    public void testIsAvailableInstanceVariable() {
        String typeMessage = "Whether or not a book is available should be of type boolean." + messageGap;

        String actualField = AttributesHelper.findField(bookFields, "boolean", "Book");
        String actualType = AttributesHelper.getFieldType(actualField);
        String actualAccess = AttributesHelper.getAccess(actualField);

        assertEquals("boolean", actualType, typeMessage);
        assertEquals("private", actualAccess, accessMessage);
    }
}`}],dataFiles:[]},{name:`Investigate and Modify: Refactoring Code #1`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Content canvas = new Content();
    Movie findingNemo = new Movie();
    TVShow strangerThings = new TVShow();

    System.out.println("Content Year: " + canvas.year);
    System.out.println("Movie Title: " + findingNemo.title);
    System.out.println("TV Show Episodes: " + strangerThings.numEpisodes);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean", "int", "double", "String"};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result;
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie {

  private String title;     // The title of the movie
  private int year;         // The year the movie was released
  private int runningTime;  // The length of the movie in minutes
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow {

  private String title;     // The title of the TV show
  private int year;         // The year the TV show was released
  private int numEpisodes;  // The number of episodes
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Refactoring Code #2`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Content canvas = new Content();
    Movie findingNemo = new Movie();
    TVShow strangerThings = new TVShow();

    System.out.println("Content Year: " + canvas.year);
    System.out.println("Movie Title: " + findingNemo.title);
    System.out.println("TV Show Episodes: " + strangerThings.numEpisodes);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */


    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean", "int", "double", "String"};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result;
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie {

  private String title;     // The title of the movie
  private int year;         // The year the movie was released
  private int runningTime;  // The length of the movie in minutes
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow {

  private String title;     // The title of the TV show
  private int year;         // The year the TV show was released
  private int numEpisodes;  // The number of episodes
  
}`}],validationFiles:[{path:`ContentTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Content, Movie, and TVShow Test")
public class ContentTest {

  Content testContent;
  Movie testMovie;
  TVShow testTVShow;

  String inheritedHeading;
  String message;
  String messageGap = "\\n       ";

  @BeforeEach
  public void setup() {
    testContent = new Content();
    testMovie = new Movie();
    testTVShow = new TVShow();

    inheritedHeading = ">> inherited from Content class <<\\n";
  }
   
  @Test
  @Order(1)
  @SuppressWarnings("unchecked")
  @DisplayName("Movie is a subclass of the Content class => ")
  public void testMovieIsSubclassOfContent() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the Movie class is a subclass of the Content class.";
    message += messageGap;

    Class contentClass = testContent.getClass();
    Class movieClass = testMovie.getClass();
    
    assertTrue(contentClass.isAssignableFrom(movieClass), message);
  }

  @Test
  @Order(2)
  @SuppressWarnings("unchecked")
  @DisplayName("TVShow is a subclass of the Content class => ")
  public void testTVShowIsSubclassOfContent() {
    message = "Use the keyword \\"extends\\" in the class header to indicate that the TVShow class is a subclass of the Content class.";
    message += messageGap;

    Class contentClass = testContent.getClass();
    Class tvShowClass = testTVShow.getClass();
    
    assertTrue(contentClass.isAssignableFrom(tvShowClass), message);
  }

  @Test
  @Order(3)
  @DisplayName("Movie class declares an instance variable for the running time and inherits title and year from Content => ")
  public void testMovieInstanceVariable() {
    AttributesHelper.setInfo(testMovie);
    ArrayList<String> superclassFields = AttributesHelper.getSuperClassFields();
    ArrayList<String> movieFields = AttributesHelper.getCurrentClassFields();

    assertTrue(superclassFields.contains("private String title"), "Movie class should inherit the title and year from the Content class." + messageGap);
    assertTrue(superclassFields.contains("private int year"), "Movie class should inherit the title and year from the Content class." + messageGap);
    assertTrue(!movieFields.contains("private String title"), "Movie class should only declare an instance variable for the running time." + messageGap);
    assertTrue(!movieFields.contains("private int year"), "Movie class should only declare an instance variable for the running time." + messageGap);
    assertTrue(movieFields.contains("private int runningTime"), "Movie class should only declare an instance variable for the running time." + messageGap);
  }

  @Test
  @Order(4)
  @DisplayName("TVShow class declares an instance variable for the number of episodes and inherits title and year from Content => ")
  public void testTVShowInstanceVariable() {
    AttributesHelper.setInfo(testTVShow);
    ArrayList<String> superclassFields = AttributesHelper.getSuperClassFields();
    ArrayList<String> tvFields = AttributesHelper.getCurrentClassFields();

    assertTrue(superclassFields.contains("private String title"), "TVShow class should inherit the title and year from the Content class." + messageGap);
    assertTrue(superclassFields.contains("private int year"), "TVShow class should inherit the title and year from the Content class." + messageGap);
    assertTrue(!tvFields.contains("private String title"), "TVShow class should only declare an instance variable for the number of episodes." + messageGap);
    assertTrue(!tvFields.contains("private int year"), "TVShow class should only declare an instance variable for the number of episodes." + messageGap);
    assertTrue(tvFields.contains("private int numEpisodes"), "TVShow class should only declare an instance variable for the number of episodes." + messageGap);
  }
  
}`}],dataFiles:[]},{name:`Practice: The Food Truck (a)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ------------------------------------- TO DO -------------------------------------
     * ✅ Instantiate Dessert and Cookie objects, then print the instance variables for
     * the Dessert and Cookie objects to the console using AttributesHelper.printAttributes().
     * ---------------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Cookie.java`,text:`/*
 * Represents a cookie that can be sold at a food truck
 * Cookie is a type of Dessert
 */
public class Cookie {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Refactor the Cookie class to be a subclass of Dessert.
   * -------------------------------------------------------------------
   */

  private String flavor;     // The flavor of a cookie
  private double price;      // The price of a cookie
  private boolean isChewy;   // Whether or not a cookie is chewy
  
}`},{path:`Dessert.java`,text:`/*
 * Represents a dessert that can be sold at a food truck
 */
public class Dessert {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Declare instance variables for the flavor and price of a dessert.
   * -------------------------------------------------------------------
   */



  
  
}`}],validationFiles:[{path:`CookieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert and Cookie Test")
public class CookieTest {
    
  String message;
  String messageGap = "\\n       ";
  Dessert testDessert;
  Cookie testCookie;
  Class dessertClass;
  ArrayList<String> dessertFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testDessert = new Dessert();
    dessertClass = testDessert.getClass();
    testCookie = new Cookie();
  
    dessertFields = AttributesHelper.getClassFieldsList(dessertClass, "Dessert");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the flavor of a dessert => ")
  public void testFlavorInstanceVariable() {
    String typeMessage = "The flavor of a dessert should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "String", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the price of a dessert => ")
  public void testPriceInstanceVariable() {
    String typeMessage = "The price of a dessert should be of type double." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "double", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("double", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @SuppressWarnings("unchecked")
  @DisplayName("Refactor the Cookie class to be a subclass of Dessert => ")
  public void testCookieIsSubclass() {
    String subclassMessage = "The Cookie class should extend the Dessert class." + messageGap;
  
    Class cookieClass = testCookie.getClass();
  
    assertTrue(dessertClass.isAssignableFrom(cookieClass), subclassMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Cookie class inherits attributes from the Dessert class => ")
  public void testCookieInheritsDessertFields() {
    String inheritMessage = "The Cookie class should inherit the flavor and price instance variables from Dessert." + messageGap;
  
    AttributesHelper.setInfo(testCookie);
    ArrayList<String> superclassFields = AttributesHelper.getSuperClassFields();
    ArrayList<String> cookieFields = AttributesHelper.getCurrentClassFields();
  
    assertTrue(superclassFields.contains("private String flavor"), inheritMessage);
    assertTrue(superclassFields.contains("private double price"), inheritMessage);
    assertTrue(!cookieFields.contains("private String flavor"), inheritMessage);
    assertTrue(!cookieFields.contains("private double price"), inheritMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Cookie class declares an instance variable for whether or not it is chewy => ")
  public void testCookieInstanceVariable() {
    String fieldMessage = "The Cookie class should declare only the instance variable isChewy." + messageGap;
  
    AttributesHelper.setInfo(testCookie);
    ArrayList<String> cookieFields = AttributesHelper.getCurrentClassFields();

    assertTrue(cookieFields.size() == 1, fieldMessage);
    assertEquals("private boolean isChewy", cookieFields.get(0), fieldMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (b)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ------------------------------------- TO DO -------------------------------------
     * ✅ Instantiate Dessert and Pie objects, then print the instance variables for
     * the Dessert and Pie objects to the console using AttributesHelper.printAttributes().
     * ---------------------------------------------------------------------------------
     */


    

    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Dessert.java`,text:`/*
 * Represents a dessert that can be sold at a food truck
 */
public class Dessert {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Declare instance variables for the flavor and price of a dessert.
   * -------------------------------------------------------------------
   */



  

}`},{path:`Pie.java`,text:`/*
 * Represents a pie that can be sold at a food truck
 * Pie is a type of Dessert
 */
public class Pie {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Refactor the Pie class to be a subclass of Dessert.
   * -------------------------------------------------------------------
   */

  private String flavor;     // The flavor of a pie
  private double price;      // The price of a pie
  private int diameter;      // The diameter of a pie
  
}`}],validationFiles:[{path:`PieTest.java`,text:`  import static org.junit.jupiter.api.Assertions.*;
  
  import java.util.ArrayList;
  
  import org.junit.jupiter.api.BeforeEach;
  import org.junit.jupiter.api.DisplayName;
  import org.junit.jupiter.api.Test;
  import org.junit.jupiter.api.TestMethodOrder;
  import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
  import org.junit.jupiter.api.Order;
  
  @TestMethodOrder(OrderAnnotation.class)
  @DisplayName("Dessert and Pie Test")
  public class PieTest {
  
  String message;
  String messageGap = "\\n       ";
  Dessert testDessert;
  Pie testPie;
  Class dessertClass;
  ArrayList<String> dessertFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testDessert = new Dessert();
    dessertClass = testDessert.getClass();
    testPie = new Pie();
  
    dessertFields = AttributesHelper.getClassFieldsList(dessertClass, "Dessert");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the flavor of a dessert => ")
  public void testFlavorInstanceVariable() {
    String typeMessage = "The flavor of a dessert should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "String", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the price of a dessert => ")
  public void testPriceInstanceVariable() {
    String typeMessage = "The price of a dessert should be of type double." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "double", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("double", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @SuppressWarnings("unchecked")
  @DisplayName("Refactor the Pie class to be a subclass of Dessert => ")
  public void testPieIsSubclass() {
    String subclassMessage = "The Pie class should extend the Dessert class." + messageGap;
  
    Class pieClass = testPie.getClass();
  
    assertTrue(dessertClass.isAssignableFrom(pieClass), subclassMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Pie class inherits attributes from the Dessert class => ")
  public void testPieInheritsDessertFields() {
    String inheritMessage = "The Pie class should inherit the flavor and price instance variables from Dessert." + messageGap;
  
    AttributesHelper.setInfo(testPie);
    ArrayList<String> superclassFields = AttributesHelper.getSuperClassFields();
    ArrayList<String> pieFields = AttributesHelper.getCurrentClassFields();
  
    assertTrue(superclassFields.contains("private String flavor"), inheritMessage);
    assertTrue(superclassFields.contains("private double price"), inheritMessage);
    assertTrue(!pieFields.contains("private String flavor"), inheritMessage);
    assertTrue(!pieFields.contains("private double price"), inheritMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Pie class declares an instance variable for the diameter of a pie => ")
  public void testPieInstanceVariable() {
    String fieldMessage = "The Pie class should declare only the instance variable diameter." + messageGap;
  
    AttributesHelper.setInfo(testPie);
    ArrayList<String> pieFields = AttributesHelper.getCurrentClassFields();

    assertTrue(pieFields.size() == 1, fieldMessage);
    assertEquals("private int diameter", pieFields.get(0), fieldMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (c)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ------------------------------------- TO DO -------------------------------------
     * ✅ Instantiate Dessert and Donut objects, then print the instance variables for
     * the Dessert and Donut objects to the console using AttributesHelper.printAttributes().
     * ---------------------------------------------------------------------------------
     */

    


    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Dessert.java`,text:`/*
 * Represents a dessert that can be sold at a food truck
 */
public class Dessert {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Declare instance variables for the flavor and price of a dessert.
   * -------------------------------------------------------------------
   */



  
  
}`},{path:`Donut.java`,text:`/*
 * Represents a donut that can be sold at a food truck
 * Donut is a type of Dessert
 */
public class Donut {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Refactor the Donut class to be a subclass of Dessert.
   * -------------------------------------------------------------------
   */

  private String flavor;          // The flavor of a donut
  private double price;           // The price of a donut
  private boolean hasSprinkles;   // Whether or not a donut has sprinkles
  
}`}],validationFiles:[{path:`DonutTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert and Donut Test")
public class DonutTest {
    
  String message;
  String messageGap = "\\n       ";
  Dessert testDessert;
  Donut testDonut;
  Class dessertClass;
  ArrayList<String> dessertFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testDessert = new Dessert();
    dessertClass = testDessert.getClass();
    testDonut = new Donut();
  
    dessertFields = AttributesHelper.getClassFieldsList(dessertClass, "Dessert");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the flavor of a dessert => ")
  public void testFlavorInstanceVariable() {
    String typeMessage = "The flavor of a dessert should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "String", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the price of a dessert => ")
  public void testPriceInstanceVariable() {
    String typeMessage = "The price of a dessert should be of type double." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "double", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("double", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @SuppressWarnings("unchecked")
  @DisplayName("Refactor the Donut class to be a subclass of Dessert => ")
  public void testDonutIsSubclass() {
    String subclassMessage = "The Donut class should extend the Dessert class." + messageGap;
  
    Class donutClass = testDonut.getClass();
  
    assertTrue(dessertClass.isAssignableFrom(donutClass), subclassMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Donut class inherits attributes from the Dessert class => ")
  public void testDonutInheritsDessertFields() {
    String inheritMessage = "The Donut class should inherit the flavor and price instance variables from Dessert." + messageGap;
  
    AttributesHelper.setInfo(testDonut);
    ArrayList<String> superclassFields = AttributesHelper.getSuperClassFields();
    ArrayList<String> donutFields = AttributesHelper.getCurrentClassFields();

    assertTrue(superclassFields.contains("private String flavor"), inheritMessage);
    assertTrue(superclassFields.contains("private double price"), inheritMessage);
    assertTrue(!donutFields.contains("private String flavor"), inheritMessage);
    assertTrue(!donutFields.contains("private double price"), inheritMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Donut class declares an instance variable for whether or not it has sprinkles => ")
  public void testDonutInstanceVariable() {
    String fieldMessage = "The Donut class should declare only the instance variable hasSprinkles." + messageGap;
  
    AttributesHelper.setInfo(testDonut);
    ArrayList<String> donutFields = AttributesHelper.getCurrentClassFields();

    assertTrue(donutFields.size() == 1, fieldMessage);
    assertEquals("private boolean hasSprinkles", donutFields.get(0), fieldMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (d)`,lesson:`Lesson 2: Attributes`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* -------------------------------------- TO DO --------------------------------------
     * ✅ Instantiate Dessert and Cupcake objects, then print the instance variables for
     * the Dessert and Cupcake objects to the console using AttributesHelper.printAttributes().
     * -----------------------------------------------------------------------------------
     */

    


    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Cupcake.java`,text:`/*
 * Represents a cupcake that can be sold at a food truck
 * Cupcake is a type of Dessert
 */
public class Cupcake {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Refactor the Cupcake class to be a subclass of Dessert.
   * -------------------------------------------------------------------
   */

  private String flavor;    // The flavor of a cupcake
  private double price;     // The price of a cupcake
  private boolean isMini;   // Whether or not a cupcake is a miniature cupcake
  
}`},{path:`Dessert.java`,text:`/*
 * Represents a dessert that can be sold at a food truck
 */
public class Dessert {

  /* ------------------------------ TO DO ------------------------------
   * ✅ Declare instance variables for the flavor and price of a dessert.
   * -------------------------------------------------------------------
   */



  
  
}`}],validationFiles:[{path:`CupcakeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert and Cupcake Test")
public class CupcakeTest {
    
  String message;
  String messageGap = "\\n       ";
  Dessert testDessert;
  Cupcake testCupcake;
  Class dessertClass;
  ArrayList<String> dessertFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testDessert = new Dessert();
    dessertClass = testDessert.getClass();
    testCupcake = new Cupcake();
  
    dessertFields = AttributesHelper.getClassFieldsList(dessertClass, "Dessert");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the flavor of a dessert => ")
  public void testFlavorInstanceVariable() {
    String typeMessage = "The flavor of a dessert should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "String", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the price of a dessert => ")
  public void testPriceInstanceVariable() {
    String typeMessage = "The price of a dessert should be of type double." + messageGap;
  
    String actualField = AttributesHelper.findField(dessertFields, "double", "Dessert");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("double", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @SuppressWarnings("unchecked")
  @DisplayName("Refactor the Cupcake class to be a subclass of Dessert => ")
  public void testCupcakeIsSubclass() {
    String subclassMessage = "The Cupcake class should extend the Dessert class." + messageGap;
  
    Class cupcakeClass = testCupcake.getClass();
  
    assertTrue(dessertClass.isAssignableFrom(cupcakeClass), subclassMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Cupcake class inherits attributes from the Dessert class => ")
  public void testCupcakeInheritsDessertFields() {
    String inheritMessage = "The Cupcake class should inherit the flavor and price instance variables from Dessert." + messageGap;
  
    AttributesHelper.setInfo(testCupcake);
    ArrayList<String> superclassFields = AttributesHelper.getSuperClassFields();
    ArrayList<String> cupcakeFields = AttributesHelper.getCurrentClassFields();

    assertTrue(superclassFields.contains("private String flavor"), inheritMessage);
    assertTrue(superclassFields.contains("private double price"), inheritMessage);
    assertTrue(!cupcakeFields.contains("private String flavor"), inheritMessage);
    assertTrue(!cupcakeFields.contains("private double price"), inheritMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Cupcake class declares an instance variable for whether or not it is mini => ")
  public void testCupcakeInstanceVariable() {
    String fieldMessage = "The Cupcake class should declare only the instance variable isMini." + messageGap;
  
    AttributesHelper.setInfo(testCupcake);
    ArrayList<String> cupcakeFields = AttributesHelper.getCurrentClassFields();

    assertTrue(cupcakeFields.size() == 1, fieldMessage);
    assertEquals("private boolean isMini", cupcakeFields.get(0), fieldMessage);
  }
}`}],dataFiles:[]},{name:`Predict and Run: No-Argument Constructors`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Vlogger valerie = new Vlogger();

    ConstructorsHelper.printConstructors(valerie);

    System.out.println();
    
    System.out.println("Name: " + valerie.getName());
    System.out.println("Year Joined: " + valerie.getYearJoined());
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Vlogger.java`,text:`/*
 * Represents a vlogger on a social media app
 */
public class Vlogger {

  private String name;        // The name of a vlogger
  private int yearJoined;     // The year a vlogger joined the app

  /*
   * Sets name to "new vlogger" and yearJoined to 2022
   */
  public Vlogger() {
    name = "new vlogger";
    yearJoined = 2022;
  }
  
  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to yearJoined
   */
  public int getYearJoined() {
    return yearJoined;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



  
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Constructors #1`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Vlogger valerie = new Vlogger();

    ConstructorsHelper.printConstructors(valerie);
    
    valerie.printVloggerInfo();
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();

    int start = currentConstructor.indexOf("(");
    int end = currentConstructor.indexOf(")");

    currentConstructor.substring(start, end);
    int comma = currentConstructor.indexOf(",");
  
      while (comma != -1) {
        String currentParameter = currentConstructor.substring(0, comma);
        parametersList.add(currentParameter);
        currentConstructor = currentConstructor.substring(comma + 1);
        comma = currentConstructor.indexOf(",");
      }
  
      return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
  
  
  
}`},{path:`Vlogger.java`,text:`/*
 * Represents a vlogger on a social media app
 */
public class Vlogger {

  private String name;        // The name of a vlogger
  private int yearJoined;     // The year a vlogger joined the app

  /*
   * Sets name to "new vlogger" and yearJoined to 2022
   */
  public Vlogger() {
    name = "new vlogger";
    yearJoined = 2022;
  }
  
  /*
   * Prints the name of the vlogger and
   * the year the vlogger joined the app
   */
  public void printVloggerInfo() {
    System.out.println("Name: " + name);
    System.out.println("Year Joined: " + yearJoined);
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



  
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Constructors #2`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Vlogger valerie = new Vlogger();

    ConstructorsHelper.printConstructors(valerie);
    
    valerie.printVloggerInfo();
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();

    int start = currentConstructor.indexOf("(");
    int end = currentConstructor.indexOf(")");

    currentConstructor.substring(start, end);
    int comma = currentConstructor.indexOf(",");
  
      while (comma != -1) {
        String currentParameter = currentConstructor.substring(0, comma);
        parametersList.add(currentParameter);
        currentConstructor = currentConstructor.substring(comma + 1);
        comma = currentConstructor.indexOf(",");
      }
  
      return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
  
  
  
}`},{path:`Vlogger.java`,text:`/*
 * Represents a vlogger on a social media app
 */
public class Vlogger {

  private String name;        // The name of a vlogger
  private int yearJoined;     // The year a vlogger joined the app

  /*
   * Sets name to "new vlogger" and yearJoined to 2022
   */
  public Vlogger() {
    name = "new vlogger";
    yearJoined = 2022;
  }
  
  /*
   * Prints the name of the vlogger and
   * the year the vlogger joined the app
   */
  public void printVloggerInfo() {
    System.out.println("Name: " + name);
    System.out.println("Year Joined: " + yearJoined);
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



  
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Constructors #3`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Vlogger valerie = new Vlogger();

    ConstructorsHelper.printConstructors(valerie);
    
    valerie.printVloggerInfo();
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();

    int start = currentConstructor.indexOf("(");
    int end = currentConstructor.indexOf(")");

    currentConstructor.substring(start, end);
    int comma = currentConstructor.indexOf(",");
  
      while (comma != -1) {
        String currentParameter = currentConstructor.substring(0, comma);
        parametersList.add(currentParameter);
        currentConstructor = currentConstructor.substring(comma + 1);
        comma = currentConstructor.indexOf(",");
      }
  
      return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
  
  
  
}`},{path:`Vlogger.java`,text:`/*
 * Represents a vlogger on a social media app
 */
public class Vlogger {

  private String name;        // The name of a vlogger
  private int yearJoined;     // The year a vlogger joined the app

  /*
   * Sets name to "new vlogger" and yearJoined to 2022
   */
  public Vlogger() {
    name = "new vlogger";
    yearJoined = 2022;
  }
  
  /*
   * Prints the name of the vlogger and
   * the year the vlogger joined the app
   */
  public void printVloggerInfo() {
    System.out.println("Name: " + name);
    System.out.println("Year Joined: " + yearJoined);
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */



  
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: Writing a Constructor (a)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Player object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject), and print the values
     * assigned to the instance variables using the methods in the Player class.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Player.java`,text:`/*
 * Represents a player in a game
 */
public class Player {

  private String name;    // The name of a player
  private int level;      // The level of a player

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the
   * name and level instance variables.
   * -----------------------------------------------------------------------------
   */






  

  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to level
   */
  public int getLevel() {
    return level;
  }
  
}`}],validationFiles:[{path:`PlayerTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

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
  Class playerClass;
  ArrayList<String> playerConstructors;
  
  @BeforeEach
  public void setup() {
    testPlayer = new Player();
    playerClass = testPlayer.getClass();
    playerConstructors = ConstructorsHelper.getClassConstructorsList(playerClass, "Player");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Player class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Player()", playerConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a default value to the name instance variable => ")
  public void testNameDefaultValue() {
    String defaultMessage = "name should have a default value assigned, such as \\"new player\\"." + messageGap;
    assertNotNull(testPlayer.getName(), defaultMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a default value to the level instance variable => ")
  public void testLevelDefaultValue() {
    String defaultMessage = "level should have a default value assigned, such as 1." + messageGap;
    assertTrue(testPlayer.getLevel() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Writing a Constructor (b)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Profile object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject), and print the values
     * assigned to the instance variables using the methods in the Profile class.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Profile.java`,text:`/*
 * Represents a user's profile on a social media app
 */
public class Profile {

  private String name;      // The name of a user
  private int yearJoined;   // The year a user joined the app

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the
   * name and yearJoined instance variables.
   * -----------------------------------------------------------------------------
   */





  

  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to yearJoined
   */
  public int getYearJoined() {
    return yearJoined;
  }
  
}`}],validationFiles:[{path:`ProfileTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Profile.java Test")
public class ProfileTest {
    
  String messageGap = "\\n       ";
  Profile testProfile;
  Class profileClass;
  ArrayList<String> profileConstructors;
  
  @BeforeEach
  public void setup() {
    testProfile = new Profile();
    profileClass = testProfile.getClass();
    profileConstructors = ConstructorsHelper.getClassConstructorsList(profileClass, "Profile");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Profile class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Profile()", profileConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a default value to the name instance variable => ")
  public void testNameDefaultValue() {
    String defaultMessage = "name should have a default value assigned, such as \\"new user\\"." + messageGap;
    assertNotNull(testProfile.getName(), defaultMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a default value to the yearJoined instance variable => ")
  public void testYearJoinedDefaultValue() {
    String defaultMessage = "yearJoined should have a default value assigned, such as 2022." + messageGap;
    assertTrue(testProfile.getYearJoined() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Writing a Constructor (c)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Painting object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject), and print the values
     * assigned to the instance variables using the methods in the Painting class.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Painting.java`,text:`/*
 * Represents a painting
 */
public class Painting {

  private String title;    // The title of a painting
  private int year;        // The year a painting was created

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the
   * title and year instance variables.
   * -----------------------------------------------------------------------------
   */





  

  /*
   * Returns the current value assigned to title
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the current value assigned to year
   */
  public int getYear() {
    return year;
  }
  
}`}],validationFiles:[{path:`PaintingTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Painting.java Test")
public class PaintingTest {
    
  String messageGap = "\\n       ";
  Painting testPainting;
  Class paintingClass;
  ArrayList<String> paintingConstructors;
  
  @BeforeEach
  public void setup() {
    testPainting = new Painting();
    paintingClass = testPainting.getClass();
    paintingConstructors = ConstructorsHelper.getClassConstructorsList(paintingClass, "Painting");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Painting class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Painting()", paintingConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a default value to the title instance variable => ")
  public void testTitleDefaultValue() {
    String defaultMessage = "title should have a default value assigned, such as \\"new painting\\"." + messageGap;
    assertNotNull(testPainting.getTitle(), defaultMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a default value to the year instance variable => ")
  public void testYearDefaultValue() {
    String defaultMessage = "year should have a default value assigned, such as 2022." + messageGap;
    assertTrue(testPainting.getYear() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Writing a Constructor (d)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Shirt object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject), and print the values
     * assigned to the instance variables using the methods in the Shirt class.
     * -----------------------------------------------------------------------------
     */


    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Shirt.java`,text:`/*
 * Represents a custom shirt
 */
public class Shirt {

  private String size;     // The size of a shirt
  private double price;    // The price of a shirt

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the
   * size and price instance variables.
   * -----------------------------------------------------------------------------
   */






  

  /*
   * Returns the current value assigned to size
   */
  public String getSize() {
    return size;
  }

  /*
   * Returns the current value assigned to price
   */
  public double getPrice() {
    return price;
  }
  
}`}],validationFiles:[{path:`ShirtTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Shirt.java Test")
public class ShirtTest {
    
  String messageGap = "\\n       ";
  Shirt testShirt;
  Class shirtClass;
  ArrayList<String> shirtConstructors;
  
  @BeforeEach
  public void setup() {
    testShirt = new Shirt();
    shirtClass = testShirt.getClass();
    shirtConstructors = ConstructorsHelper.getClassConstructorsList(shirtClass, "Shirt");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Shirt class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Shirt()", shirtConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a default value to the size instance variable => ")
  public void testSizeDefaultValue() {
    String defaultMessage = "size should have a default value assigned, such as \\"small\\"." + messageGap;
    assertNotNull(testShirt.getSize(), defaultMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a default value to the price instance variable => ")
  public void testPriceDefaultValue() {
    String defaultMessage = "price should have a default value assigned, such as 5.99." + messageGap;
    assertTrue(testShirt.getPrice() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Writing a Class (a)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Song object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject) and the values assigned
     * to the instance variables using the methods in the Song class.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Song.java`,text:`/*
 * Represents a song on a playlist
 */
public class Song {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare instance variables for the title of a song and its rating.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables for the title of a song and its rating.
   * -----------------------------------------------------------------------------
   */

  


  /*
   * Returns the current value assigned to title
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the current value assigned to rating
   */
  public int getRating() {
    return rating;
  }
  
}`}],validationFiles:[{path:`SongTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Song.java Test")
public class SongTest {
    
  String messageGap = "\\n       ";
  Song testSong;
  Class songClass;
  ArrayList<String> songConstructors;
  ArrayList<String> songFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testSong = new Song();
    songClass = testSong.getClass();
  
    songFields = AttributesHelper.getClassFieldsList(songClass, "Song");
    songConstructors = ConstructorsHelper.getClassConstructorsList(songClass, "Song");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the title => ")
  public void testTitleInstanceVariable() {
    String typeMessage = "The title of a song should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(songFields, "String", "Song");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the rating of a song => ")
  public void testRatingInstanceVariable() {
    String typeMessage = "The rating of a song should be of type int." + messageGap;
  
    String actualField = AttributesHelper.findField(songFields, "int", "Song");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("int", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Write a no-argument constructor in the Song class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Song()", songConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Assign a default value to the title instance variable => ")
  public void testTitleDefaultValue() {
    String defaultMessage = "title should have a default value assigned, such as \\"new song\\"." + messageGap;
    assertNotNull(testSong.getTitle(), defaultMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Assign a default value to the rating instance variable => ")
  public void testRatingDefaultValue() {
    String defaultMessage = "rating should have a default value assigned, such as 1." + messageGap;
    assertTrue(testSong.getRating() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Writing a Class (b)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Team object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject) and the values assigned
     * to the instance variables using the methods in the Team class.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Team.java`,text:`/*
 * Represents a school sports team
 */
public class Team {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare instance variables for the team name and number of athletes.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables for the team name and number of athletes.
   * -----------------------------------------------------------------------------
   */



  
  /*
   * Returns the current value assigned to teamName
   */
  public String getTeamName() {
    return teamName;
  }

  /*
   * Returns the current value assigned to numAthletes
   */
  public int getNumAthletes() {
    return numAthletes;
  }
  
}`}],validationFiles:[{path:`TeamTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Team.java Test")
public class TeamTest {
    
  String messageGap = "\\n       ";
  Team testTeam;
  Class teamClass;
  ArrayList<String> teamConstructors;
  ArrayList<String> teamFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testTeam = new Team();
    teamClass = testTeam.getClass();
  
    teamFields = AttributesHelper.getClassFieldsList(teamClass, "Team");
    teamConstructors = ConstructorsHelper.getClassConstructorsList(teamClass, "Team");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the name of the team => ")
  public void testTeamNameInstanceVariable() {
    String typeMessage = "The team name should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(teamFields, "String", "Team");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the number of athletes on the team => ")
  public void testNumAthletesInstanceVariable() {
    String typeMessage = "The number of athletes on the team should be of type int." + messageGap;
  
    String actualField = AttributesHelper.findField(teamFields, "int", "Team");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("int", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Write a no-argument constructor in the Team class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Team()", teamConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Assign a default value to the title instance variable => ")
  public void testTeamNameDefaultValue() {
    String defaultMessage = "teamName should have a default value assigned, such as \\"new team\\"." + messageGap;
    assertNotNull(testTeam.getTeamName(), defaultMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Assign a default value to the numAthletes instance variable => ")
  public void testNumAthletesDefaultValue() {
    String defaultMessage = "numAthletes should have a default value assigned, such as 5." + messageGap;
    assertTrue(testTeam.getNumAthletes() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Writing a Class (c)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Poem object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject) and the values assigned
     * to the instance variables using the methods in the Poem class.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Poem.java`,text:`/*
 * Represents a poem
 */
public class Poem {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare instance variables for the title and number of lines in a poem.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables for the title and number of lines in a poem.
   * -----------------------------------------------------------------------------
   */




  /*
   * Returns the current value assigned to title
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the current value assigned to numLines
   */
  public int getNumLines() {
    return numLines;
  }
  
}`}],validationFiles:[{path:`PoemTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

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
  Class songClass;
  ArrayList<String> poemConstructors;
  ArrayList<String> poemFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testPoem = new Poem();
    songClass = testPoem.getClass();
  
    poemFields = AttributesHelper.getClassFieldsList(songClass, "Poem");
    poemConstructors = ConstructorsHelper.getClassConstructorsList(songClass, "Poem");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the title => ")
  public void testTitleInstanceVariable() {
    String typeMessage = "The title of a poem should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(poemFields, "String", "Poem");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the number of lines in the poem => ")
  public void testNumLinesInstanceVariable() {
    String typeMessage = "The number of lines in a poem should be of type int." + messageGap;
  
    String actualField = AttributesHelper.findField(poemFields, "int", "Poem");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("int", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Write a no-argument constructor in the Poem class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Poem()", poemConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Assign a default value to the title instance variable => ")
  public void testTitleDefaultValue() {
    String defaultMessage = "title should have a default value assigned, such as \\"new poem\\"." + messageGap;
    assertNotNull(testPoem.getTitle(), defaultMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Assign a default value to the numLines instance variable => ")
  public void testNumLinesDefaultValue() {
    String defaultMessage = "numLines should have a default value assigned, such as 1." + messageGap;
    assertTrue(testPoem.getNumLines() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Writing a Class (d)`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Club object. Print the constructor to the console using
     * ConstructorsHelper.printConstructors(nameOfObject) and the values assigned
     * to the instance variables using the methods in the Club class.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`AttributesHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class AttributesHelper {

  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassFields;
  private static ArrayList<String> superClassFields;

  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassFields = getClassFieldsList(currentClass, currentClassName);
    superClassFields = getClassFieldsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getCurrentClassFields() {
    return currentClassFields;
  }
  
  public static ArrayList<String> getSuperClassFields() {
    return superClassFields;
  }
  
  public static ArrayList<String> getClassFieldsList(Class currentClass, String className) {
    Field[] classFields = currentClass.getDeclaredFields();
    ArrayList<String> fieldsList = fieldsToList(Arrays.toString(classFields));
    cleanFields(fieldsList, className);
    return fieldsList;
  }
  
  private static ArrayList<String> fieldsToList(String fieldsAsText) {
    fieldsAsText = fieldsAsText.substring(1, fieldsAsText.length() - 1);
    ArrayList<String> fieldsList = new ArrayList<String>();

    String currentField = "";
    int comma = fieldsAsText.indexOf(",");

    while (comma != -1) {
      currentField = fieldsAsText.substring(0, comma);
      fieldsList.add(currentField);
      fieldsAsText = fieldsAsText.substring(comma + 2);
      comma = fieldsAsText.indexOf(",");
    }

    fieldsList.add(fieldsAsText);
    return fieldsList;
  }
  
  public static void cleanFields(ArrayList<String> classFieldsList, String className) {
    for (int index = 0; index < classFieldsList.size(); index++) {
      String currentField = classFieldsList.get(index);

      currentField = removeClassName(currentField, className);
      currentField = removeJavaLang(currentField);
      currentField.trim();

      classFieldsList.set(index, currentField);
    }
  }
  
  public static String removeClassName(String currentField, String className) {
    int location = currentField.indexOf(className);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + className.length() + 1);
      location = currentField.indexOf(className);
    }

    return currentField;
  }
  
  public static String removeJavaLang(String currentField) {
    String textToFind = "java.lang.";
    int location = currentField.indexOf(textToFind);

    while (location != -1) {
      currentField = currentField.substring(0, location) + currentField.substring(location + textToFind.length());
      location = currentField.indexOf(textToFind);
    }

    return currentField;
  }
  
  public static String findField(ArrayList<String> classFieldsList, String typeToFind, String className) {
    String result = "MISSING";
    typeToFind += " ";

    for (int index = 0; index < classFieldsList.size(); index++) {
      String current = classFieldsList.get(index);

      if (current.indexOf(typeToFind) > 0) {
        result = removeClassName(current, className);
        result = removeJavaLang(result);
      }
    }

    return result;
  }
  
  public static String getAccess(String currentField) {
    String result = "";

    if (!currentField.equals("MISSING")) {
      result = currentField.substring(0, currentField.indexOf(" "));
    
    }

    return result;
  }
  
  public static String getFieldType(String currentField) {
    String[] possibleTypes = {"boolean ", "int ", "double ", "String "};
    String result = "MISSING";

    for (int index = 0; index < possibleTypes.length; index++) {
      if (currentField.indexOf(possibleTypes[index]) > 0) {
        result = possibleTypes[index];
      }
    }

    return result.trim();
  }
  
  public static void printAttributes(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getAttributesHeading());
    System.out.println(getCurrentClassAttributes());

    if (!superClassName.equals("Object")) {
      System.out.println(getSuperAttributes(superClassName));
    }
  }
  
  public static String getAttributesHeading() {       
    String result = currentClassName + " Class Attributes";
    result += "\\n------------------------------";
    return result;
  }
  
  public static String getSuperAttributesHeading(String superClassName) {
    return "\\n>> inherited from " + superClassName + " class <<\\n";
  }
  
  public static String getCurrentClassAttributes() {
    ArrayList<String> currentClassFields = getCurrentClassFields();
    return getListAsText(currentClassFields);
  }
  
  public static String getSuperAttributes(String superClassName) {
    ArrayList<String> superClassFields = getSuperClassFields();
    return getSuperAttributesHeading(superClassName) + getListAsText(superClassFields);
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }

}`},{path:`Club.java`,text:`/*
 * Represents a school club
 */
public class Club {

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Declare instance variables for the sponsor's name and number of members.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables for the sponsor's name and number of members.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns the current value assigned to sponsor
   */
  public String getSponsor() {
    return sponsor;
  }

  /*
   * Returns the current value assigned to numMembers
   */
  public int getNumMembers() {
    return numMembers;
  }
  
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`ClubTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Club.java Test")
public class ClubTest {
    
  String messageGap = "\\n       ";
  Club testClub;
  Class clubClass;
  ArrayList<String> clubConstructors;
  ArrayList<String> clubFields;
  String accessMessage;
  
  @BeforeEach
  public void setup() {
    accessMessage = "Instance variables should be private." + messageGap;
  
    testClub = new Club();
    clubClass = testClub.getClass();
  
    clubFields = AttributesHelper.getClassFieldsList(clubClass, "Club");
    clubConstructors = ConstructorsHelper.getClassConstructorsList(clubClass, "Club");
  }
  
  @Test
  @Order(1)
  @DisplayName("Declare an instance variable for the sponsor's name => ")
  public void testSponsorInstanceVariable() {
    String typeMessage = "The sponsor's name should be of type String." + messageGap;
  
    String actualField = AttributesHelper.findField(clubFields, "String", "Club");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("String", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Declare an instance variable for the number of club members => ")
  public void testNumMembersInstanceVariable() {
    String typeMessage = "The number of club members should be of type int." + messageGap;
  
    String actualField = AttributesHelper.findField(clubFields, "int", "Club");
    String actualType = AttributesHelper.getFieldType(actualField);
    String actualAccess = AttributesHelper.getAccess(actualField);
  
    assertEquals("int", actualType, typeMessage);
    assertEquals("private", actualAccess, accessMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Write a no-argument constructor in the Club class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Club()", clubConstructors.get(0), constructorMessage);
  }
  
  @Test
  @Order(4)
  @DisplayName("Assign a default value to the sponsor instance variable => ")
  public void testSponsorDefaultValue() {
    String defaultMessage = "sponsor should have a default value assigned, such as \\"new cong\\"." + messageGap;
    assertNotNull(testClub.getSponsor(), defaultMessage);
  }
  
  @Test
  @Order(5)
  @DisplayName("Assign a default value to the numMembers instance variable => ")
  public void testNumMembersDefaultValue() {
    String defaultMessage = "numMembers should have a default value assigned, such as 1." + messageGap;
    assertTrue(testClub.getNumMembers() > 0, defaultMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck`,lesson:`Lesson 3: No-Argument Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Dessert object. Use ConstructorsHelper.printConstructors(nameOfObject)
     * to print the constructor for the Dessert object to the console.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`DessertTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.*;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert.java Test")
public class DessertTest {
    
  String messageGap = "\\n       ";
  Dessert testDessert;
  Class dessertClass;
  ArrayList<String> dessertConstructors;
  
  @BeforeEach
  public void setup() {
    testDessert = new Dessert();
    dessertClass = testDessert.getClass();
    dessertConstructors = ConstructorsHelper.getClassConstructorsList(dessertClass, "Dessert");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Dessert class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    assertEquals("public Dessert()", dessertConstructors.get(0), constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Print the constructor for the Dessert class to the console => ")
  public void testPrintDessertConstructor() {
    String printMessage = "Call the ConstructorsHelper.printConstructors() method to print the constructor for the Dessert object." + messageGap;

    List<String> output = SystemOutTestRunner.run();
    ConstructorsHelper.setInfo(testDessert);
    String expected = ConstructorsHelper.getCurrentClassConstructors();

    assertTrue(output.contains(expected), printMessage);
  }
}`}],dataFiles:[]},{name:`Predict and Run: Parameterized Constructors`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Mountain everest = new Mountain("Mount Everest", 29029, true);
    ConstructorsHelper.printConstructors(everest);

    System.out.println();
    System.out.println("Values Assigned to everest Instance Variables");
    everest.printMountainInfo();

    System.out.println();

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(start, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
      start = comma + 1;
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`},{path:`Mountain.java`,text:`/*
 * Represents a mountain
 */
public class Mountain {

  private String name;              // The name of a mountain
  private int height;               // The height of a mountain
  private boolean hasBeenClimbed;   // Whether or not a mountain has been climbed

  /*
   * Sets name to "unknown", height to the minimum height of a
   * mountain (1000 ft), and hasBeenClimbed to false
   */
  public Mountain() {
    name = "unknown";
    height = 1000;
    hasBeenClimbed = false;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to climbedStatus
   */
  public Mountain(String aName, int aHeight, boolean climbedStatus) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = climbedStatus;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to false
   */
  public Mountain(String aName, int aHeight) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = false;
  }

  /*
   * Prints the name and height of the mountain
   * and whether or not it has been climbed
   */
  public void printMountainInfo() {
    System.out.println("Name: " + name);
    System.out.println("Height: " + height);
    System.out.println("Has been climbed? " + hasBeenClimbed);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Parameterized Constructors #1`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Mountain.java`,text:`/*
 * Represents a mountain
 */
public class Mountain {

  private String name;              // The name of a mountain
  private int height;               // The height of a mountain
  private boolean hasBeenClimbed;   // Whether or not a mountain has been climbed

  /*
   * Sets name to "unknown", height to the minimum height of a
   * mountain (1000 ft), and hasBeenClimbed to false
   */
  public Mountain() {
    name = "unknown";
    height = 1000;
    hasBeenClimbed = false;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to climbedStatus
   */
  public Mountain(String aName, int aHeight, boolean climbedStatus) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = climbedStatus;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to false
   */
  public Mountain(String aName, int aHeight) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = false;
  }

  /*
   * Prints the name and height of the mountain
   * and whether or not it has been climbed
   */
  public void printMountainInfo() {
    System.out.println("Name: " + name);
    System.out.println("Height: " + height);
    System.out.println("Has been climbed? " + hasBeenClimbed);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Parameterized Constructors #2`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Mountain.java`,text:`/*
 * Represents a mountain
 */
public class Mountain {

  private String name;              // The name of a mountain
  private int height;               // The height of a mountain
  private boolean hasBeenClimbed;   // Whether or not a mountain has been climbed

  /*
   * Sets name to "unknown", height to the minimum height of a
   * mountain (1000 ft), and hasBeenClimbed to false
   */
  public Mountain() {
    name = "unknown";
    height = 1000;
    hasBeenClimbed = false;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to climbedStatus
   */
  public Mountain(String aName, int aHeight, boolean climbedStatus) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = climbedStatus;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to false
   */
  public Mountain(String aName, int aHeight) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = false;
  }

  /*
   * Prints the name and height of the mountain
   * and whether or not it has been climbed
   */
  public void printMountainInfo() {
    System.out.println("Name: " + name);
    System.out.println("Height: " + height);
    System.out.println("Has been climbed? " + hasBeenClimbed);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Parameterized Constructors #3`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Mountain.java`,text:`/*
 * Represents a mountain
 */
public class Mountain {

  private String name;              // The name of a mountain
  private int height;               // The height of a mountain
  private boolean hasBeenClimbed;   // Whether or not a mountain has been climbed

  /*
   * Sets name to "unknown", height to the minimum height of a
   * mountain (1000 ft), and hasBeenClimbed to false
   */
  public Mountain() {
    name = "unknown";
    height = 1000;
    hasBeenClimbed = false;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to climbedStatus
   */
  public Mountain(String aName, int aHeight, boolean climbedStatus) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = climbedStatus;
  }

  /*
   * Sets name to aName, height to aHeight, and hasBeenClimbed to false
   */
  public Mountain(String aName, int aHeight) {
    name = aName;
    height = aHeight;
    hasBeenClimbed = false;
  }

  /*
   * Prints the name and height of the mountain
   * and whether or not it has been climbed
   */
  public void printMountainInfo() {
    System.out.println("Name: " + name);
    System.out.println("Height: " + height);
    System.out.println("Has been climbed? " + hasBeenClimbed);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: Parameterized Constructors (a)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Rollercoaster object using the parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    

    
    
    
  }
}`},{path:`Rollercoaster.java`,text:`/*
 * Represents a rollercoaster at a theme park
 */
public class Rollercoaster {

  private String name;     // The name of a rollercoaster
  private int speed;       // The speed of a rollercoaster

  /*
   * Sets the name to "unknown" and speed to 20
   */
  public Rollercoaster() {
    name = "unknown";
    speed = 20;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name and speed instance variables.
   * -----------------------------------------------------------------------------
   */

  

  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to speed
   */
  public int getSpeed() {
    return speed;
  }
  
}`}],validationFiles:[{path:`RollercoasterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Rollercoaster.java Test")
public class RollercoasterTest {
    
  String messageGap = "\\n       ";
  Rollercoaster testRollercoaster;
  Class rollercoasterClass;
  ArrayList<String> rollercoasterConstructors;
  
  @BeforeEach
  public void setup() {
    testRollercoaster = new Rollercoaster();
    rollercoasterClass = testRollercoaster.getClass();
    rollercoasterConstructors = ConstructorsHelper.getClassConstructorsList(rollercoasterClass, "Rollercoaster");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor in the Rollercoaster class => ")
  public void testParameterizedConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String intParameterMessage = "The constructor should specify an int parameter to assign a value to speed." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(rollercoasterConstructors, 2, "String", "int");
    assertNotNull(actualConstructor, "Constructor with two parameters not found.");
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a specific value to the name instance variable => ")
  public void testNameDefaultValue() {
    String valueMessage = "name should be assigned the value passed to the String parameter." + messageGap;

    try {
        Rollercoaster testRollercoasterTwoParameters = new Rollercoaster("Rollercoaster", 50);
        assertNotNull(testRollercoasterTwoParameters, "The Rollercoaster object should not be null.");
        assertEquals("Rollercoaster", testRollercoasterTwoParameters.getName(), valueMessage);
    } catch (Exception e) {
        fail("An exception occurred during object creation: " + e.getMessage());
    }
    
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a specific value to the speed instance variable => ")
  public void testSpeedDefaultValue() {
    String valueMessage = "speed should be assigned the value passed to the int parameter." + messageGap;

    try {
        Rollercoaster testRollercoasterTwoParameters = new Rollercoaster("Rollercoaster", 50);
        assertNotNull(testRollercoasterTwoParameters, "The Rollercoaster object should not be null.");
        assertTrue(testRollercoasterTwoParameters.getSpeed() > 0, valueMessage);
    } catch (Exception e) {
        fail("An exception occurred during object creation: " + e.getMessage());
    }

  }
}`}],dataFiles:[]},{name:`Practice: Parameterized Constructors (b)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a NationalPark object using the parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
  }
}`},{path:`NationalPark.java`,text:`/*
 * Represents a national park
 */
public class NationalPark {

  private String name;     // The name of a national park
  private int area;        // The area in acres of a national park

  /*
   * Sets the name to "unknown" and area to 100
   */
  public NationalPark() {
    name = "unknown";
    area = 100;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name and area instance variables.
   * -----------------------------------------------------------------------------
   */



  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to area
   */
  public int getArea() {
    return area;
  }
  
}`}],validationFiles:[{path:`NationalParkTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("NationalPark.java Test")
public class NationalParkTest {
    
  String messageGap = "\\n       ";
  NationalPark testNationalPark;
  Class nationalParkClass;
  ArrayList<String> nationalParkConstructors;
  
  @BeforeEach
  public void setup() {
    testNationalPark = new NationalPark("Hot Springs", 5554);
    nationalParkClass = testNationalPark.getClass();
    nationalParkConstructors = ConstructorsHelper.getClassConstructorsList(nationalParkClass, "NationalPark");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor in the NationalPark class => ")
  public void testParameterizedConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String intParameterMessage = "The constructor should specify an int parameter to assign a value to area." + messageGap;
  
    String actualConstructor = nationalParkConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
  
    assertEquals("public NationalPark", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("int"), intParameterMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a specific value to the name instance variable => ")
  public void testNameDefaultValue() {
    String valueMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    assertNotNull(testNationalPark.getName(), valueMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a specific value to the area instance variable => ")
  public void testAreaDefaultValue() {
    String valueMessage = "area should be assigned the value passed to the int parameter." + messageGap;
    assertTrue(testNationalPark.getArea() > 0, valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Parameterized Constructors (c)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Character object using the parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
  }
}`},{path:`Character.java`,text:`/*
 * Represents a character in a play
 */
public class Character {

  private String name;          // The name of a character
  private boolean isLeadRole;   // Whether or not a character is a lead role

  /*
   * Sets the name to "unknown" and isLeadRole to false
   */
  public Character() {
    name = "unknown";
    isLeadRole = false;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name and isLeadRole instance variables.
   * -----------------------------------------------------------------------------
   */



  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to isLeadRole
   */
  public boolean hasLeadRole() {
    return isLeadRole;
  }
  
}`}],validationFiles:[{path:`CharacterTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Character.java Test")
public class CharacterTest {
    
  String messageGap = "\\n       ";
  Character testCharacter;
  Class characterClass;
  ArrayList<String> characterConstructors;
  
  @BeforeEach
  public void setup() {
    testCharacter = new Character("Hamlet", true);
    characterClass = testCharacter.getClass();
    characterConstructors = ConstructorsHelper.getClassConstructorsList(characterClass, "Character");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor in the Character class => ")
  public void testParameterizedConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to isLeadRole." + messageGap;
  
    String actualConstructor = characterConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
  
    assertEquals("public Character", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("boolean"), booleanParameterMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a specific value to the name instance variable => ")
  public void testNameDefaultValue() {
    String valueMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    assertNotNull(testCharacter.getName(), valueMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a specific value to the isLeadRole instance variable => ")
  public void testIsLeadRoleDefaultValue() {
    String valueMessage = "isLeadRole should be assigned the value passed to the boolean parameter." + messageGap;
    assertTrue(testCharacter.hasLeadRole(), valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Parameterized Constructors (d)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Museum object using the parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
  }
}`},{path:`Museum.java`,text:`/*
 * Represents a museum
 */
public class Museum {

  private String name;           // The name of a museum
  private boolean isNonProfit;   // Whether or not a museum is non-profit

  /*
   * Sets the name to "unknown" and isNonProfit to false
   */
  public Museum() {
    name = "unknown";
    isNonProfit = false;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name and isNonProfit instance variables.
   * -----------------------------------------------------------------------------
   */



  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to isNonProfit
   */
  public boolean getIsNonProfit() {
    return isNonProfit;
  }
  
}`}],validationFiles:[{path:`MuseumTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

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
  Museum testMuseum;
  Class museumClass;
  ArrayList<String> museumConstructors;
  
  @BeforeEach
  public void setup() {
    testMuseum = new Museum("The Louvre", true);
    museumClass = testMuseum.getClass();
    museumConstructors = ConstructorsHelper.getClassConstructorsList(museumClass, "Museum");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor in the Museum class => ")
  public void testParameterizedConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to isNonProfit." + messageGap;
  
    String actualConstructor = museumConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
  
    assertEquals("public Museum", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("boolean"), booleanParameterMessage);
  }
  
  @Test
  @Order(2)
  @DisplayName("Assign a specific value to the name instance variable => ")
  public void testNameDefaultValue() {
    String valueMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    assertNotNull(testMuseum.getName(), valueMessage);
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign a specific value to the isNonProfit instance variable => ")
  public void testIsNonProfitDefaultValue() {
    String valueMessage = "isNonProfit should be assigned the value passed to the boolean parameter." + messageGap;
    assertTrue(testMuseum.getIsNonProfit(), valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Multiple Parameterized Constructors (a)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Team object using each parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Team.java`,text:`/*
 * Represents a team in a sports league
 */
public class Team {

  private String name;     // The name of a team
  private int numWins;     // The number of championship wins
  
  /*
   * Sets the name to "unknown" and numWins to 0
   */
  public Team() {
    name = "unknown";
    numWins = 0;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name and numWins instance variables.
   * -----------------------------------------------------------------------------
   */



  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign a specific value to the
   * name instance variable and a default value to the numWins instance variable.
   * -----------------------------------------------------------------------------
   */


  
  
  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to numWins
   */
  public int getNumWins() {
    return numWins;
  }
  
}`}],validationFiles:[{path:`TeamTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Team.java Test")
public class TeamTest {
    
  String messageGap = "\\n       ";
  Team testTeam;
  Class teamClass;
  ArrayList<String> teamConstructors;
  
  @BeforeEach
  public void setup() {
    testTeam = new Team();
    teamClass = testTeam.getClass();
    teamConstructors = ConstructorsHelper.getClassConstructorsList(teamClass, "Team");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor to assign values for name and numWins => ")
  public void testTwoParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String intParameterMessage = "The constructor should specify a int parameter to assign a value to numWins." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(teamConstructors, 2, "String", "int");
    assertNotNull(actualConstructor, "Constructor with two parameters not found.");
  }
  
  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor to assign a value to name => ")
  public void testOneParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    
    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(teamConstructors, 1, "String");
    assertNotNull(actualConstructor, "Constructor with one parameter not found.");
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign specific values to the name and numWins instance variables => ")
  public void testNameAndNumWinsAssigned() {
    String nameMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    String winsMessage = "numWins should be assigned the value passed to the int parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Team.class, String.class, int.class)) {
      Team testTeamTwoParameters = (Team) ConstructorsHelper.createInstance(Team.class, "Seahawks", 8);
    
      assertNotNull(testTeamTwoParameters, "The Team object should not be null.");
      String actualName = testTeamTwoParameters.getName();
      int actualWins = testTeamTwoParameters.getNumWins();
  
      assertEquals("Seahawks", actualName, nameMessage);
      assertEquals(8, actualWins, winsMessage);
    } else {
      fail("The constructor with parameters (String, int) does not exist.");
    }
  }
  
  @Test
  @Order(4)
  @DisplayName("Assign a specific value to name and a default value to numWins => ")
  public void testNameAssigned() {
    String nameMessage = "name should be assigned the value passed to the String parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Team.class, String.class)) {
      Team testTeamOneParameter = (Team) ConstructorsHelper.createInstance(Team.class, "Cowboys");
    
      assertNotNull(testTeamOneParameter, "The Team object should not be null.");
      String actualName = testTeamOneParameter.getName();
  
      assertEquals("Cowboys", actualName, nameMessage);
    } else {
      fail("The constructor with parameter (String) does not exist.");
    }
  }
}`}],dataFiles:[]},{name:`Practice: Multiple Parameterized Constructors (b)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Project object using each parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
  }
}`},{path:`Project.java`,text:`/*
 * Represents a photographer's project of photos
 */
public class Project {

  private String title;     // The title of a photography project
  private int numPhotos;    // The number of photos in a project
  
  /*
   * Sets the name to "unknown" and numPhotos to 0
   */
  public Project() {
    title = "unknown";
    numPhotos = 0;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the title and numPhotos instance variables.
   * -----------------------------------------------------------------------------
   */



  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign a specific value to the title
   * instance variable and a default value to the numPhotos instance variable.
   * -----------------------------------------------------------------------------
   */


  
  
  /*
   * Returns the current value assigned to title
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the current value assigned to numPhotos
   */
  public int getNumPhotos() {
    return numPhotos;
  }
  
}`}],validationFiles:[{path:`ProjectTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Project.java Test")
public class ProjectTest {
    
  String messageGap = "\\n       ";
  Project testProject;
  Class projectClass;
  ArrayList<String> projectConstructors;
  
  @BeforeEach
  public void setup() {
    testProject = new Project();
    projectClass = testProject.getClass();
    projectConstructors = ConstructorsHelper.getClassConstructorsList(projectClass, "Project");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor to assign values for title and numPhotos => ")
  public void testTwoParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to title." + messageGap;
    String intParameterMessage = "The constructor should specify a int parameter to assign a value to numPhotos." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(projectConstructors, 2, "String", "int");
    assertNotNull(actualConstructor, "Constructor with two parameters not found.");
  }
  
  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor to assign a value to title => ")
  public void testOneParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to title." + messageGap;
    
    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(projectConstructors, 1, "String");
    assertNotNull(actualConstructor, "Constructor with one parameter not found.");
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign specific values to the title and numPhotos instance variables => ")
  public void testNameAndNumPhotosAssigned() {
    String titleMessage = "title should be assigned the value passed to the String parameter." + messageGap;
    String photosMessage = "numPhotos should be assigned the value passed to the int parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Project.class, String.class, int.class)) {
      Project testProjectTwoParameters = (Project) ConstructorsHelper.createInstance(Project.class, "Abstract", 23);
    
      assertNotNull(testProjectTwoParameters, "The Project object should not be null.");
      String actualTitle = testProjectTwoParameters.getTitle();
      int actualNumPhotos = testProjectTwoParameters.getNumPhotos();
  
      assertEquals("Abstract", actualTitle, titleMessage);
      assertEquals(23, actualNumPhotos, photosMessage);
    } else {
      fail("The constructor with parameters (String, int) does not exist.");
    }
  }
  @Test
  @Order(4)
  @DisplayName("Assign a specific value to title and a default value to numPhotos => ")
  public void testTitleAssigned() {
    String titleMessage = "title should be assigned the value passed to the String parameter." + messageGap;
  
    if (ConstructorsHelper.hasConstructor(Project.class, String.class)) {
      Project testProjectOneParameter = (Project) ConstructorsHelper.createInstance(Project.class, "Random Objects");
  
      assertNotNull(testProjectOneParameter, "The Project object should not be null.");
      String actualTitle = testProjectOneParameter.getTitle();
      int actualNumPhotos = testProjectOneParameter.getNumPhotos();
  
      assertEquals("Random Objects", actualTitle, titleMessage); // ✅ FIXED
      assertEquals(0, actualNumPhotos, "numPhotos should be set to default value of 0.");
    } else {
      fail("The constructor with parameter (String) does not exist.");
    }
  }

}`}],dataFiles:[]},{name:`Practice: Multiple Parameterized Constructors (c)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Planets object using each parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
  }
}`},{path:`Planet.java`,text:`/*
 * Represents a planet
 */
public class Planet {

  private String name;             // The name of a planet
  private int numMoons;            // The number of moons a planet has
  private boolean hasRingSystem;   // Whether or not a planet has a ring system
  
  /*
   * Sets the name to "unknown", numMoons to 1,
   * and hasRingSystem to false
   */
  public Planet() {
    name = "unknown";
    numMoons = 1;
    hasRingSystem = false;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name, numMoons, and hasRingSystem instance variables.
   * -----------------------------------------------------------------------------
   */



  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign a specific value to the name
   * and numMoons instance variables and a default value to hasRingSystem.
   * -----------------------------------------------------------------------------
   */


  
  
  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to numMoons
   */
  public int getNumMoons() {
    return numMoons;
  }

  /*
   * Returns the current value assigned to hasRingSystem
   */
  public boolean getHasRingSystem() {
    return hasRingSystem;
  }
  
}`}],validationFiles:[{path:`PlanetTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Planet.java Test")
public class PlanetTest {
    
  String messageGap = "\\n       ";
  Planet testPlanet;
  Class planetClass;
  ArrayList<String> planetConstructors;
  
  @BeforeEach
  public void setup() {
    testPlanet = new Planet();
    planetClass = testPlanet.getClass();
    planetConstructors = ConstructorsHelper.getClassConstructorsList(planetClass, "Planet");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor to assign values for name, numMoons, and hasRingSystem => ")
  public void testThreeParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String intParameterMessage = "The constructor should specify a int parameter to assign a value to numMoons." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to hasRingSystem." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(planetConstructors, 3, "String", "int", "boolean");
    assertNotNull(actualConstructor, "Constructor with three parameters not found.");
  }

  @Test
  @Order(2)
  @DisplayName("Assign specific values to the name, numMoons, and hasRingSystem instance variables => ")
  public void testNameNumMoonsHasRingSystemAssigned() {
    String nameMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    String numMoonsMessage = "numMoons should be assigned the value passed to the int parameter." + messageGap;
    String hasRingSystemMessage = "hasRingSystem should be assigned the value passed to the boolean parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Planet.class, String.class, int.class, boolean.class)) {
      Planet testPlanetThreeParameters = (Planet) ConstructorsHelper.createInstance(Planet.class, "Jupiter", 79, true);
    
      assertNotNull(testPlanetThreeParameters, "The testPlanetThreeParameters should not be null.");
      String actualName = testPlanetThreeParameters.getName();
      int actualNumMoons = testPlanetThreeParameters.getNumMoons();
      boolean actualRingSystem = testPlanetThreeParameters.getHasRingSystem();
  
      assertEquals("Jupiter", actualName, nameMessage);
      assertEquals(79, actualNumMoons, numMoonsMessage);
      assertTrue(actualRingSystem, hasRingSystemMessage);
    } else {
      fail("The constructor with parameters (String, int, boolean) does not exist.");
    }
  }
  
  @Test
  @Order(3)
  @DisplayName("Write a parameterized constructor to assign values to name and numMoons => ")
  public void testTwoParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String intParameterMessage = "The constructor should specify a int parameter to assign a value to numMoons." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(planetConstructors, 2, "String", "int");
    assertNotNull(actualConstructor, "Constructor with two parameters not found.");
  }
  
  @Test
  @Order(4)
  @DisplayName("Assigns specific values to name and numMoons and a default value to hasRingSystem => ")
  public void testNameNumMoonsAssigned() {
    String nameMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    String numMoonsMessage = "numMoons should be assigned the value passed to the int parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Planet.class, String.class, int.class)) {
      Planet testPlanetTwoParameters = (Planet) ConstructorsHelper.createInstance(Planet.class, "Mars", 2);
  
      assertNotNull(testPlanetTwoParameters, "The testPlanetTwoParameters should not be null.");
      String actualName = testPlanetTwoParameters.getName();
      int actualNumMoons = testPlanetTwoParameters.getNumMoons();
  
      assertEquals("Mars", actualName, nameMessage);
      assertEquals(2, actualNumMoons, numMoonsMessage);
    } else {
      fail("The constructor with parameters (String, int) does not exist.");
    }
  }

}`}],dataFiles:[]},{name:`Practice: Multiple Parameterized Constructors (d)`,lesson:`Lesson 4: Parameterized Constructors`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate a Electronic object using each parameterized constructor,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */

    
    
    
    
  }
}`},{path:`Electronic.java`,text:`/*
 * Represents an electronic at a store
 */
public class Electronic {

  private String name;             // The name of an electronic
  private double price;            // The price of an electronic
  private boolean isRefurbished;   // Whether or not an electronic is refurbished
  
  /*
   * Sets the name to "unknown", price to 50.99
   * and isRefurbished to false
   */
  public Electronic() {
    name = "unknown";
    price = 50.99;
    isRefurbished = false;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specific values to
   * the name, price, and isRefurbished instance variables.
   * -----------------------------------------------------------------------------
   */



  
  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign a specific value to the name
   * and price instance variables and a default value to isRefurbished.
   * -----------------------------------------------------------------------------
   */


  
  
  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to price
   */
  public double getPrice() {
    return price;
  }

  /*
   * Returns the current value assigned to isRefurbished
   */
  public boolean getIsRefurbished() {
    return isRefurbished;
  }
  
}`}],validationFiles:[{path:`ElectronicTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Electronic.java Test")
public class ElectronicTest {
    
  String messageGap = "\\n       ";
  Electronic testElectronic;
  Class electronicClass;
  ArrayList<String> electronicConstructors;
  
  @BeforeEach
  public void setup() {
    testElectronic = new Electronic();
    electronicClass = testElectronic.getClass();
    electronicConstructors = ConstructorsHelper.getClassConstructorsList(electronicClass, "Electronic");
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor to assign values for name, price, and isRefurbished => ")
  public void testThreeParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to isRefurbished." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(electronicConstructors, 3, "String", "double", "boolean");
    assertNotNull(actualConstructor, "Constructor with three parameters not found.");
  }
  
  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor to assign values to name and price => ")
  public void testTwoParameterConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;

    String actualConstructor = ConstructorsHelper.findConstructorWithParameters(electronicConstructors, 2, "String", "double");
    assertNotNull(actualConstructor, "Constructor with two parameters not found.");
  }
  
  @Test
  @Order(3)
  @DisplayName("Assign specific values to the name, price, and isRefurbished instance variables => ")
  public void testNamePriceIsRefurbishedAssigned() {
    String nameMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    String priceMessage = "price should be assigned the value passed to the double parameter." + messageGap;
    String isRefurbishedMessage = "isRefurbished should be assigned the value passed to the boolean parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Electronic.class, String.class, double.class, boolean.class)) {
      Electronic testElectronicThreeParameters = (Electronic) ConstructorsHelper.createInstance(Electronic.class, "Cell Phone", 125.75, true);
    
      assertNotNull(testElectronicThreeParameters, "The Electronic object should not be null.");
      String actualName = testElectronicThreeParameters.getName();
      double actualPrice = testElectronicThreeParameters.getPrice();
      boolean actualRefurbished = testElectronicThreeParameters.getIsRefurbished();
  
      assertEquals("Cell Phone", actualName, nameMessage);
      assertEquals(125.75, actualPrice, priceMessage);
      assertTrue(actualRefurbished, isRefurbishedMessage);
    } else {
      fail("The constructor with parameters (String, double, boolean) does not exist.");
    }
  }
  
  @Test
  @Order(4)
  @DisplayName("Assigns specific values to name and price and a default value to isRefurbished => ")
  public void testNamePriceAssigned() {
    String nameMessage = "name should be assigned the value passed to the String parameter." + messageGap;
    String priceMessage = "price should be assigned the value passed to the double parameter." + messageGap;

    if (ConstructorsHelper.hasConstructor(Electronic.class, String.class, double.class)) {
      Electronic testElectronicTwoParameters = (Electronic) ConstructorsHelper.createInstance(Electronic.class, "Laptop", 580.99);
    
      assertNotNull(testElectronicTwoParameters, "The Electronic object should not be null.");
      String actualName = testElectronicTwoParameters.getName();
      double actualPrice = testElectronicTwoParameters.getPrice();
  
      assertEquals("Laptop", actualName, nameMessage);
      assertEquals(580.99, actualPrice, priceMessage);
    } else {
      fail("The constructor with parameters (String, double) does not exist.");
    }
  }
}`}],dataFiles:[]},{name:`Predict: Getting User Input`,lesson:`Lesson 5: User Input`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    Scanner input = new Scanner(System.in);

    System.out.print("Enter the first number: ");
    int first = input.nextInt();

    System.out.print("Enter the second number: ");
    int second = input.nextInt();

    int sum = first + second;

    System.out.println("The sum of " + first + " and " + second + " is " + sum);

    input.close();
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Getting User Input (a)`,lesson:`Lesson 5: User Input`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    Greeter myGreeter = new Greeter();

    // TO DO #1: Create a Scanner object called input.

    
    // Call the printGreeting method with the Scanner
    myGreeter.printGreeting(input);

    // TO DO #2: Close the Scanner object.

    
  }
}
`},{path:`Greeter.java`,text:`import java.util.Scanner;

/*
 * Gets input from a user and prints a greeting
 */
public class Greeter {

  public void printGreeting(Scanner input) {
    // TO DO #3: Get the user's name with the Scanner input.
    

    // TO DO #4: Print a greeting using their name.
    
    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Getting User Input (b)`,lesson:`Lesson 5: User Input`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    Calculator myCalc = new Calculator();

    // TO DO #1: Create a Scanner object called input.

    
    myCalc.printSum(input);
    
  }
}`},{path:`Calculator.java`,text:`import java.util.Scanner;

/*
 * Gets input from a user and prints the sum
 */
public class Calculator {

  public void printSum(Scanner input) {
    // TO DO #2: Get the first number from the user with the Scanner input.
    

    // TO DO #3: Get the second number from the user.
    

    // TO DO #4: Print the sum of the two numbers.
    

    // TO DO #5: Close the Scanner.
    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Getting User Input (c)`,lesson:`Lesson 5: User Input`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    StudentCreator myCreator = new StudentCreator();

    // TO DO #1: Create a Scanner object called input.

    
    myCreator.createStudent(input);
    
  }
}`},{path:`Student.java`,text:`/*
 * Represents a student
 */
public class Student {

  private String name;  // the name of the student
  private double gpa;   // the student's gpa

  /*
   * Constructor to create a Student object
   * with the specified name and GPA
   */
  public Student(String newName, double newGpa) {
    name = newName;
    gpa = newGpa;
  }
  
}`},{path:`StudentCreator.java`,text:`import java.util.Scanner;

/*
 * Gets user input to instantiate a Student object
 */
public class StudentCreator {

  public void createStudent(Scanner input) {
    // TO DO #2: Get the student's name with the Scanner input.
    

    // TO DO #3: Get the student's GPA.
    

    // TO DO #4: Create a Student object with the user's values.
    

    // TO DO #5: Print a confirmation that the Student was created.
    

    // TO DO #6: Close the Scanner.
    
    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Getting User Input (d)`,lesson:`Lesson 5: User Input`,view:`neighborhood`,grid:`1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.Painter;
import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    PainterCreator myCreator = new PainterCreator();

    // TO DO #1: Create a Scanner object called input.

    
    myCreator.createPainter(input);
    
  }
}`},{path:`PainterCreator.java`,text:`import org.code.neighborhood.Painter;
import java.util.Scanner;

/*
 * Gets user input to instantiate a Painter object
 */
public class PainterCreator {

  public void createPainter(Scanner input) {
    // TO DO #2: Get the starting x location with the Scanner input.
    

    // TO DO #3: Get the starting y location with the Scanner input.
    

    // TO DO #4: Get the direction to face with the Scanner input.
    

    // TO DO #5: Get the amount of paint with the Scanner input.
    

    // TO DO #6: Create a Painter object using the user input.
    

    // TO DO #7: Close the Scanner.
    
    
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Constructors and Scope`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Shoes sneakers = new Shoes();
    System.out.println("sneakers");
    System.out.println("---------------");
    sneakers.printShoesInfo();

    System.out.println();

    Shoes converse = new Shoes("purple", 8, true);
    System.out.println("converse");
    System.out.println("---------------");
    converse.printShoesInfo();
    
  }
}`},{path:`Shoes.java`,text:`/*
 * Represents a pair of shoes
 */
public class Shoes {

  private String color;            // The color of a pair of shoes
  private double size;             // The size of a pair of shoes
  private boolean hasShoelaces;    // Whether or not a pair of shoes has shoelaces

  /*
   * Sets color to "white", size to 7.5, and hasShoelaces to true
   */
  public Shoes() {
    color = "white";
    size = 7.5;
    hasShoelaces = true;
  }

  /*
   * Sets color to color, size to size, and hasShoelaces to hasShoelaces
   */
  public Shoes(String color, double size, boolean hasShoelaces) {
    color = color;
    size = size;
    hasShoelaces = hasShoelaces;
  }

  /*
   * Prints the color and size of the pair of shoes
   * and whether or not it has shoelaces
   */
  public void printShoesInfo() {
    System.out.println("Color: " + color);
    System.out.println("Size: " + size);
    System.out.println("Has Shoelaces? " + hasShoelaces);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The this Keyword #1`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Shoes.java`,text:`/*
 * Represents a pair of shoes
 */
public class Shoes {

  private String color;            // The color of a pair of shoes
  private double size;             // The size of a pair of shoes
  private boolean hasShoelaces;    // Whether or not a pair of shoes has shoelaces

  /*
   * Sets color to "white", size to 7.5, and hasShoelaces to true
   */
  public Shoes() {
    color = "white";
    size = 7.5;
    hasShoelaces = true;
  }

  /*
   * Sets color to color, size to size, and hasShoelaces to hasShoelaces
   */
  public Shoes(String color, double size, boolean hasShoelaces) {
    color = color;
    size = size;
    hasShoelaces = hasShoelaces;
  }

  /*
   * Prints the color and size of the pair of shoes
   * and whether or not it has shoelaces
   */
  public void printShoesInfo() {
    System.out.println("Color: " + color);
    System.out.println("Size: " + size);
    System.out.println("Has Shoelaces? " + hasShoelaces);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The this Keyword #2`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Shoes.java`,text:`/*
 * Represents a pair of shoes
 */
public class Shoes {

  private String color;            // The color of a pair of shoes
  private double size;             // The size of a pair of shoes
  private boolean hasShoelaces;    // Whether or not a pair of shoes has shoelaces

  /*
   * Sets color to "white", size to 7.5, and hasShoelaces to true
   */
  public Shoes() {
    color = "white";
    size = 7.5;
    hasShoelaces = true;
  }

  /*
   * Sets color to color, size to size, and hasShoelaces to hasShoelaces
   */
  public Shoes(String color, double size, boolean hasShoelaces) {
    color = color;
    size = size;
    hasShoelaces = hasShoelaces;
  }

  /*
   * Prints the color and size of the pair of shoes
   * and whether or not it has shoelaces
   */
  public void printShoesInfo() {
    System.out.println("Color: " + color);
    System.out.println("Size: " + size);
    System.out.println("Has Shoelaces? " + hasShoelaces);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The this Keyword #3`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Shoes.java`,text:`/*
 * Represents a pair of shoes
 */
public class Shoes {

  private String color;            // The color of a pair of shoes
  private double size;             // The size of a pair of shoes
  private boolean hasShoelaces;    // Whether or not a pair of shoes has shoelaces

  /*
   * Sets color to "white", size to 7.5, and hasShoelaces to true
   */
  public Shoes() {
    color = "white";
    size = 7.5;
    hasShoelaces = true;
  }

  /*
   * Sets color to color, size to size, and hasShoelaces to hasShoelaces
   */
  public Shoes(String color, double size, boolean hasShoelaces) {
    color = color;
    size = size;
    hasShoelaces = hasShoelaces;
  }

  /*
   * Prints the color and size of the pair of shoes
   * and whether or not it has shoelaces
   */
  public void printShoesInfo() {
    System.out.println("Color: " + color);
    System.out.println("Size: " + size);
    System.out.println("Has Shoelaces? " + hasShoelaces);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: The this Keyword (a)`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate Crop objects using the no-argument and parameterized constructors,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Crop.java`,text:`/*
 * Represents a crop on a farm
 */
public class Crop {

  private String type;      // The type of crop
  private int numDays;      // The number of days it takes a crop to mature

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables using the this keyword.
   * -----------------------------------------------------------------------------
   */


  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specified values to the
   * instance variables using the this keyword.
   * -----------------------------------------------------------------------------
   */


  

  /*
   * Returns the current value assigned to type
   */
  public String getType() {
    return type;
  }

  /*
   * Returns the current value assigned to numDays
   */
  public int getNumDays() {
    return numDays;
  }
  
}`}],validationFiles:[{path:`CropTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Crop.java Test")
public class CropTest {
    
  String messageGap = "\\n       ";
  Crop testCrop;
  Class cropClass;
  ArrayList<String> cropConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testCrop = new Crop();
    cropClass = testCrop.getClass();
    cropConstructors = ConstructorsHelper.getClassConstructorsList(cropClass, "Crop");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Crop class => ")
  public void testNoArgConstructor() {
    String actualConstructor = cropConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    assertEquals("public Crop", actualName, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Assign a default value to the type instance variable => ")
  public void testTypeDefaultValue() {
    String typeMessage = "The constructor should assign a default value to type." + messageGap;
    assertNotNull(testCrop.getType(), typeMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Assign a default value to the numDays instance variable => ")
  public void testNumDaysDefaultValue() {
    String numDaysMessage = "The constructor should assign a default value to numDays." + messageGap;
    assertTrue(testCrop.getNumDays() > 0, numDaysMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Write a parameterized constructor in the Crop class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to type." + messageGap;
    String intParameterMessage = "The constructor should specify a int parameter to assign a value to numDays." + messageGap;
    
    String actualConstructor = cropConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Crop", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("int"), intParameterMessage);
  }

  @Test
  @Order(5)
  @DisplayName("Assign a specific value to the type instance variable => ")
  public void testTypeValueAssigned() {
    String valueMessage = "type should be assigned the value passed to the String parameter." + messageGap;

    Crop testCropParameters = new Crop("pumpkin", 110);

    assertEquals("pumpkin", testCropParameters.getType(), valueMessage);
  }

  @Test
  @Order(6)
  @DisplayName("Assign a specific value to the numDays instance variable => ")
  public void testNumDaysValueAssigned() {
    String valueMessage = "numDays should be assigned the value passed to the int parameter." + messageGap;

    Crop testCropParameters = new Crop("pumpkin", 110);

    assertEquals(110, testCropParameters.getNumDays(), valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The this Keyword (b)`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate HairProduct objects using the no-argument and parameterized constructors,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`HairProduct.java`,text:`/*
 * Represents a hair product used by a hair stylist
 */
public class HairProduct {

  private String type;      // The type of hair product
  private double price;     // The price of a hair product
  private boolean isCurly;  // Whether or not a hair product is for curly hair

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables using the this keyword.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specified values to the
   * instance variables using the this keyword.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns the current value assigned to type
   */
  public String getType() {
    return type;
  }

  /*
   * Returns the current value assigned to price
   */
  public double getPrice() {
    return price;
  }

  /*
   * Returns the current value assigned to isCurly
   */
  public boolean getIsCurly() {
    return isCurly;
  }
  
}`}],validationFiles:[{path:`HairProductTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("HairProduct.java Test")
public class HairProductTest {
    
  String messageGap = "\\n       ";
  HairProduct testHairProduct;
  Class hairProductClass;
  ArrayList<String> hairProductConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testHairProduct = new HairProduct();
    hairProductClass = testHairProduct.getClass();
    hairProductConstructors = ConstructorsHelper.getClassConstructorsList(hairProductClass, "HairProduct");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the HairProduct class => ")
  public void testNoArgConstructor() {
    String actualConstructor = hairProductConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    assertEquals("public HairProduct", actualName, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Assign a default value to the type instance variable => ")
  public void testTypeDefaultValue() {
    String typeMessage = "The constructor should assign a default value to type." + messageGap;
    assertNotNull(testHairProduct.getType(), typeMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Assign a default value to the price instance variable => ")
  public void testPriceDefaultValue() {
    String priceMessage = "The constructor should assign a default value to price." + messageGap;
    assertTrue(testHairProduct.getPrice() > 0, priceMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Write a parameterized constructor in the HairProduct class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to type." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    
    String actualConstructor = hairProductConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public HairProduct", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
  }

  @Test
  @Order(5)
  @DisplayName("Assign a specific value to the type instance variable => ")
  public void testTypeValueAssigned() {
    String valueMessage = "type should be assigned the value passed to the String parameter." + messageGap;

    HairProduct testHairProductParameters = new HairProduct("conditioner", 12.75, true);

    assertEquals("conditioner", testHairProductParameters.getType(), valueMessage);
  }

  @Test
  @Order(6)
  @DisplayName("Assign a specific value to the price instance variable => ")
  public void testPriceValueAssigned() {
    String valueMessage = "price should be assigned the value passed to the double parameter." + messageGap;

    HairProduct testHairProductParameters = new HairProduct("conditioner", 12.75, true);

    assertEquals(12.75, testHairProductParameters.getPrice(), valueMessage);
  }

  @Test
  @Order(7)
  @DisplayName("Assign a specific value to the isCurly instance variable => ")
  public void testIsCurlyValueAssigned() {
    String valueMessage = "isCurly should be assigned the value passed to the boolean parameter." + messageGap;

    HairProduct testHairProductParameters = new HairProduct("conditioner", 12.75, true);

    assertTrue(testHairProductParameters.getIsCurly(), valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The this Keyword (c)`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate Candle objects using the no-argument and parameterized constructors,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Candle.java`,text:`/*
 * Represents a candle made by a candle maker
 */
public class Candle {

  private String scent;        // The scent of a candle
  private double price;        // The price of a candle
  private boolean isSeasonal;  // Whether or not a candle is a seasonal scent

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables using the this keyword.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specified values to the
   * instance variables using the this keyword.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns the current value assigned to scent
   */
  public String getScent() {
    return scent;
  }

  /*
   * Returns the current value assigned to price
   */
  public double getPrice() {
    return price;
  }

  /*
   * Returns the current value assigned to isSeasonal
   */
  public boolean getIsSeasonal() {
    return isSeasonal;
  }
  
}`}],validationFiles:[{path:`CandleTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Candle.java Test")
public class CandleTest {
    
  String messageGap = "\\n       ";
  Candle testCandle;
  Class candleClass;
  ArrayList<String> candleConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testCandle = new Candle();
    candleClass = testCandle.getClass();
    candleConstructors = ConstructorsHelper.getClassConstructorsList(candleClass, "Candle");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Candle class => ")
  public void testNoArgConstructor() {
    String actualConstructor = candleConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    assertEquals("public Candle", actualName, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Assign a default value to the scent instance variable => ")
  public void testScentDefaultValue() {
    String scentMessage = "The constructor should assign a default value to scent." + messageGap;
    assertNotNull(testCandle.getScent(), scentMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Assign a default value to the price instance variable => ")
  public void testPriceDefaultValue() {
    String priceMessage = "The constructor should assign a default value to price." + messageGap;
    assertTrue(testCandle.getPrice() > 0, priceMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Write a parameterized constructor in the Candle class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to scent." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to isSeasonal." + messageGap;
    
    String actualConstructor = candleConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Candle", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
    assertTrue(actualParameters.contains("boolean"), booleanParameterMessage);
  }

  @Test
  @Order(5)
  @DisplayName("Assign a specific value to the scent instance variable => ")
  public void testScentValueAssigned() {
    String valueMessage = "scent should be assigned the value passed to the String parameter." + messageGap;

    Candle testCandleParameters = new Candle("lavender", 12.75, false);

    assertEquals("lavender", testCandleParameters.getScent(), valueMessage);
  }

  @Test
  @Order(6)
  @DisplayName("Assign a specific value to the price instance variable => ")
  public void testPriceValueAssigned() {
    String valueMessage = "price should be assigned the value passed to the double parameter." + messageGap;

    Candle testCandleParameters = new Candle("citrus", 8.99, false);

    assertEquals(8.99, testCandleParameters.getPrice(), valueMessage);
  }

  @Test
  @Order(7)
  @DisplayName("Assign a specific value to the isSeasonal instance variable => ")
  public void testIsSeasonalValueAssigned() {
    String valueMessage = "isSeasonal should be assigned the value passed to the boolean parameter." + messageGap;

    Candle testCandleParameters = new Candle("gingerbread", 10.50, true);

    assertTrue(testCandleParameters.getIsSeasonal(), valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The this Keyword (d)`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate Course objects using the no-argument and parameterized constructors,
     * then print the constructors and values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`},{path:`Course.java`,text:`/*
 * Represents a course taught by a teacher
 */
public class Course {

  private String name;          // The name of a course
  private int numStudents;      // The number of students in a course
  private boolean isSemester;   // Whether or not it is a semester course

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a no-argument constructor to assign default values to the instance
   * variables using the this keyword.
   * -----------------------------------------------------------------------------
   */



  

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a parameterized constructor to assign specified values to the
   * instance variables using the this keyword.
   * -----------------------------------------------------------------------------
   */



  

  /*
   * Returns the current value assigned to name
   */
  public String getName() {
    return name;
  }

  /*
   * Returns the current value assigned to numStudents
   */
  public int getNumStudents() {
    return numStudents;
  }

  /*
   * Returns the current value assigned to isSemester
   */
  public boolean getIsSemester() {
    return isSemester;
  }
  
}`}],validationFiles:[{path:`CourseTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Course.java Test")
public class CourseTest {
    
  String messageGap = "\\n       ";
  Course testCourse;
  Class courseClass;
  ArrayList<String> courseConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testCourse = new Course();
    courseClass = testCourse.getClass();
    courseConstructors = ConstructorsHelper.getClassConstructorsList(courseClass, "Course");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Course class => ")
  public void testNoArgConstructor() {
    String actualConstructor = courseConstructors.get(0);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    assertEquals("public Course", actualName, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Assign a default value to the name instance variable => ")
  public void testNameDefaultValue() {
    String typeMessage = "The constructor should assign a default value to name." + messageGap;
    assertNotNull(testCourse.getName(), typeMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Assign a default value to the numStudents instance variable => ")
  public void testNumStudentsDefaultValue() {
    String numStudentsMessage = "The constructor should assign a default value to numStudents." + messageGap;
    assertTrue(testCourse.getNumStudents() > 0, numStudentsMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Write a parameterized constructor in the Course class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to name." + messageGap;
    String intParameterMessage = "The constructor should specify an int parameter to assign a value to numStudents." + messageGap;
    
    String actualConstructor = courseConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Course", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("int"), intParameterMessage);
  }

  @Test
  @Order(5)
  @DisplayName("Assign a specific value to the name instance variable => ")
  public void testNameValueAssigned() {
    String valueMessage = "name should be assigned the value passed to the String parameter." + messageGap;

    Course testCourseParameters = new Course("cybersecurity", 15, true);

    assertEquals("cybersecurity", testCourseParameters.getName(), valueMessage);
  }

  @Test
  @Order(6)
  @DisplayName("Assign a specific value to the numStudents instance variable => ")
  public void testNumStudentsValueAssigned() {
    String valueMessage = "numStudents should be assigned the value passed to the int parameter." + messageGap;

    Course testCourseParameters = new Course("cybersecurity", 15, true);

    assertEquals(15, testCourseParameters.getNumStudents(), valueMessage);
  }

  @Test
  @Order(7)
  @DisplayName("Assign a specific value to the isSemester instance variable => ")
  public void testIsSemesterValueAssigned() {
    String valueMessage = "isSemester should be assigned the value passed to the boolean parameter." + messageGap;

    Course testCourseParameters = new Course("cybersecurity", 15, true);

    assertTrue(testCourseParameters.getIsSemester(), valueMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The this Keyword`,lesson:`Lesson 6: The this Keyword`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate Dessert objects using the no-argument and parameterized
     * constructors. Then use ConstructorsHelper.printConstructors(nameOfObject)
     * to print the constructors for the Dessert objects to the console.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`DessertTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert.java Test")
public class DessertTest {
    
  String messageGap = "\\n       ";
  Dessert testDessert;
  Class dessertClass;
  ArrayList<String> dessertConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testDessert = new Dessert();
    dessertClass = testDessert.getClass();
    dessertConstructors = ConstructorsHelper.getClassConstructorsList(dessertClass, "Dessert");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write a parameterized constructor in the Dessert class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to flavor." + messageGap;
    String doubleParameterMessage = "The constructor should specify an double parameter to assign a value to price." + messageGap;
    
    String actualConstructor = dessertConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Dessert", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
  }

}`}],dataFiles:[]},{name:`Predict and Run: Constructors and Inheritance`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Prints the title and year of the content
   */
  public void printContentInfo() {
    System.out.println("Title: " + title);
    System.out.println("Year: " + year);
  }

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Prints the running time of the movie
   */
  public void printMovieInfo() {
    printContentInfo();
    System.out.println("Running Time: " + runningTime);
  }
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */






  

  /*
   * Prints the title, year, and number of episodes for the TV show
   */
  public void printTVShowInfo() {
    printContentInfo();
    System.out.println("Number of Episodes: " + numEpisodes);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The super Keyword #1`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Prints the title and year of the content
   */
  public void printContentInfo() {
    System.out.println("Title: " + title);
    System.out.println("Year: " + year);
  }

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Prints the running time of the movie
   */
  public void printMovieInfo() {
    printContentInfo();
    System.out.println("Running Time: " + runningTime);
  }
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */






  

  /*
   * Prints the title, year, and number of episodes for the TV show
   */
  public void printTVShowInfo() {
    printContentInfo();
    System.out.println("Number of Episodes: " + numEpisodes);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The super Keyword #2`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Prints the title and year of the content
   */
  public void printContentInfo() {
    System.out.println("Title: " + title);
    System.out.println("Year: " + year);
  }

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Prints the running time of the movie
   */
  public void printMovieInfo() {
    printContentInfo();
    System.out.println("Running Time: " + runningTime);
  }
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */






  

  /*
   * Prints the title, year, and number of episodes for the TV show
   */
  public void printTVShowInfo() {
    printContentInfo();
    System.out.println("Number of Episodes: " + numEpisodes);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: The super Keyword #3`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Prints the title and year of the content
   */
  public void printContentInfo() {
    System.out.println("Title: " + title);
    System.out.println("Year: " + year);
  }

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Prints the running time of the movie
   */
  public void printMovieInfo() {
    printContentInfo();
    System.out.println("Running Time: " + runningTime);
  }
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */






  

  /*
   * Prints the title, year, and number of episodes for the TV show
   */
  public void printTVShowInfo() {
    printContentInfo();
    System.out.println("Number of Episodes: " + numEpisodes);
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Practice: PainterPlus Constructors (a)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* -------------------------------------- TO DO --------------------------------------
     * ✅ Instantiate a PainterPlus object at (2, 2) facing south with 16 units of paint.
     * -----------------------------------------------------------------------------------
     */


    
    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeAll;
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
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    String message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
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
  @DisplayName("Write a parameterized constructor in the PainterPlus class => ")
  public void testParameterizedConstructor() {
    String xMessage = "The constructor should specify an int parameter to assign a value to x." + messageGap;
    String yMessage = "The constructor should specify an int parameter to assign a value to y." + messageGap;
    String directionMessage = "The constructor should specify a String parameter to assign a value to direction." + messageGap;
    String paintMessage = "The constructor should specify an int parameter to assign a value to paint." + messageGap;
    
    int randomX = (int)(Math.random() * 8);
    int randomY = (int)(Math.random() * 8);
    String[] directions = {"north", "south", "east", "west"};
    int randomIndex = (int)(Math.random() * directions.length);
    String randomDirection = directions[randomIndex];
    int randomPaint = (int)(Math.random() * (20 - 10)) + 10;
    
    PainterPlus testPainterPlus = new PainterPlus(randomX, randomY, randomDirection, randomPaint);

    assertEquals(randomX, testPainterPlus.getX(), xMessage);
    assertEquals(randomY, testPainterPlus.getY(), yMessage);
    assertEquals(randomDirection, testPainterPlus.getDirection(), directionMessage);
    assertEquals(randomPaint, testPainterPlus.getMyPaint(), paintMessage);
  }
   
  @Test
  @Order(2)
  @DisplayName("Instantiate a PainterPlus object at (2, 2) => ")
  public void testPainterPlusStartLocation() {
    String xMessage = "Call the parameterized constructor and pass 2 as the argument for x." + messageGap;
    String yMessage = "Call the parameterized constructor and pass 2 as the argument for y." + messageGap;
      
    int startingX = primaryPainterLog.getStartingPosition().getX();
    int startingY = primaryPainterLog.getStartingPosition().getY();

    assertEquals(2, startingX, xMessage);
    assertEquals(2, startingY, yMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Instantiate a PainterPlus object facing \\"south\\" => ")
  public void testPainterPlusFacingSouth() {
    String directionMessage = "Call the parameterized constructor and pass \\"south\\" as the argument for direction." + messageGap;

    String startingDirection = primaryPainterLog.getStartingPosition().getDirection();

    assertEquals("south", startingDirection, directionMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Instantiate a PainterPlus object with 16 units of paint => ")
  public void testPainterPlusStartPaint() {
    String paintMessage = "Call the parameterized constructor and pass 16 as the argument for paint." + messageGap;

    int startingPaint = primaryPainterLog.getStartingPaintCount();

    assertEquals(16, startingPaint, paintMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Constructors (b)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
1,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* -------------------------------------- TO DO --------------------------------------
     * ✅ Instantiate a PainterPlus object at (0, 9) facing north with 10 units of paint.
     * -----------------------------------------------------------------------------------
     */

    
    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeAll;
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
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    String message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
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
  @DisplayName("Write a parameterized constructor in the PainterPlus class => ")
  public void testParameterizedConstructor() {
    String xMessage = "The constructor should specify an int parameter to assign a value to x." + messageGap;
    String yMessage = "The constructor should specify an int parameter to assign a value to y." + messageGap;
    String directionMessage = "The constructor should specify a String parameter to assign a value to direction." + messageGap;
    String paintMessage = "The constructor should specify an int parameter to assign a value to paint." + messageGap;
    
    int randomX = (int)(Math.random() * 8);
    int randomY = (int)(Math.random() * 8);
    String[] directions = {"north", "south", "east", "west"};
    int randomIndex = (int)(Math.random() * directions.length);
    String randomDirection = directions[randomIndex];
    int randomPaint = (int)(Math.random() * (20 - 10)) + 10;
    
    PainterPlus testPainterPlus = new PainterPlus(randomX, randomY, randomDirection, randomPaint);

    assertEquals(randomX, testPainterPlus.getX(), xMessage);
    assertEquals(randomY, testPainterPlus.getY(), yMessage);
    assertEquals(randomDirection, testPainterPlus.getDirection(), directionMessage);
    assertEquals(randomPaint, testPainterPlus.getMyPaint(), paintMessage);
  }
   
  @Test
  @Order(2)
  @DisplayName("Instantiate a PainterPlus object at (0, 9) => ")
  public void testPainterPlusStartLocation() {
    String xMessage = "Call the parameterized constructor and pass 0 as the argument for x." + messageGap;
    String yMessage = "Call the parameterized constructor and pass 9 as the argument for y." + messageGap;
      
    int startingX = primaryPainterLog.getStartingPosition().getX();
    int startingY = primaryPainterLog.getStartingPosition().getY();

    assertEquals(0, startingX, xMessage);
    assertEquals(9, startingY, yMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Instantiate a PainterPlus object facing \\"north\\" => ")
  public void testPainterPlusFacingNorth() {
    String directionMessage = "Call the parameterized constructor and pass \\"north\\" as the argument for direction." + messageGap;

    String startingDirection = primaryPainterLog.getStartingPosition().getDirection();

    assertEquals("north", startingDirection, directionMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Instantiate a PainterPlus object with 10 units of paint => ")
  public void testPainterPlusStartPaint() {
    String paintMessage = "Call the parameterized constructor and pass 10 as the argument for paint." + messageGap;

    int startingPaint = primaryPainterLog.getStartingPaintCount();

    assertEquals(10, startingPaint, paintMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Constructors (c)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ------------------------------ TO DO ------------------------------
     * ✅ Instantiate a PainterPlus object next to the traffic cone
     * facing west with 5 units of paint.
     * -------------------------------------------------------------------
     */

    
    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeAll;
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
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    String message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
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
  @DisplayName("Write a parameterized constructor in the PainterPlus class => ")
  public void testParameterizedConstructor() {
    String xMessage = "The constructor should specify an int parameter to assign a value to x." + messageGap;
    String yMessage = "The constructor should specify an int parameter to assign a value to y." + messageGap;
    String directionMessage = "The constructor should specify a String parameter to assign a value to direction." + messageGap;
    String paintMessage = "The constructor should specify an int parameter to assign a value to paint." + messageGap;
    
    int randomX = (int)(Math.random() * 8);
    int randomY = (int)(Math.random() * 8);
    String[] directions = {"north", "south", "east", "west"};
    int randomIndex = (int)(Math.random() * directions.length);
    String randomDirection = directions[randomIndex];
    int randomPaint = (int)(Math.random() * (20 - 10)) + 10;
    
    PainterPlus testPainterPlus = new PainterPlus(randomX, randomY, randomDirection, randomPaint);

    assertEquals(randomX, testPainterPlus.getX(), xMessage);
    assertEquals(randomY, testPainterPlus.getY(), yMessage);
    assertEquals(randomDirection, testPainterPlus.getDirection(), directionMessage);
    assertEquals(randomPaint, testPainterPlus.getMyPaint(), paintMessage);
  }
   
  @Test
  @Order(2)
  @DisplayName("Instantiate a PainterPlus object at (5, 3) => ")
  public void testPainterPlusStartLocation() {
    String xMessage = "Call the parameterized constructor and pass 5 as the argument for x." + messageGap;
    String yMessage = "Call the parameterized constructor and pass 3 as the argument for y." + messageGap;
      
    int startingX = primaryPainterLog.getStartingPosition().getX();
    int startingY = primaryPainterLog.getStartingPosition().getY();

    assertEquals(5, startingX, xMessage);
    assertEquals(3, startingY, yMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Instantiate a PainterPlus object facing \\"west\\" => ")
  public void testPainterPlusFacingNorth() {
    String directionMessage = "Call the parameterized constructor and pass \\"west\\" as the argument for direction." + messageGap;

    String startingDirection = primaryPainterLog.getStartingPosition().getDirection();

    assertEquals("west", startingDirection, directionMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Instantiate a PainterPlus object with 5 units of paint => ")
  public void testPainterPlusStartPaint() {
    String paintMessage = "Call the parameterized constructor and pass 5 as the argument for paint." + messageGap;

    int startingPaint = primaryPainterLog.getStartingPaintCount();

    assertEquals(5, startingPaint, paintMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: PainterPlus Constructors (d)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`neighborhood`,grid:`0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 1,0 1,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 1,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
0,0 0,0 0,0 0,0 0,0 0,0 1,0 1,0 1,0 1,0 1,0 0,0 0,0 0,0 0,0 0,0
`,files:[{path:`Main.java`,text:`import org.code.neighborhood.*;

public class Main {
  public static void main(String[] args) {

    /* ------------------------------ TO DO ------------------------------
     * ✅ Instantiate a PainterPlus object in front of the red house
     * facing east with 20 units of paint.
     * -------------------------------------------------------------------
     */

    
    
  }
}`}],validationFiles:[{path:`PainterPlusTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.code.neighborhood.*;
import org.code.validation.*;

import org.junit.jupiter.api.BeforeAll;
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
  static String messageGap = "\\n       ";
   
  @BeforeAll
  public static void setup() {   
    String message = "The Painter might not be instantiated, doesn't have enough paint, or tried to move off the grid." + messageGap;
    
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
  @DisplayName("Write a parameterized constructor in the PainterPlus class => ")
  public void testParameterizedConstructor() {
    String xMessage = "The constructor should specify an int parameter to assign a value to x." + messageGap;
    String yMessage = "The constructor should specify an int parameter to assign a value to y." + messageGap;
    String directionMessage = "The constructor should specify a String parameter to assign a value to direction." + messageGap;
    String paintMessage = "The constructor should specify an int parameter to assign a value to paint." + messageGap;
    
    int randomX = (int)(Math.random() * 8);
    int randomY = (int)(Math.random() * 8);
    String[] directions = {"north", "south", "east", "west"};
    int randomIndex = (int)(Math.random() * directions.length);
    String randomDirection = directions[randomIndex];
    int randomPaint = (int)(Math.random() * (20 - 10)) + 10;
    
    PainterPlus testPainterPlus = new PainterPlus(randomX, randomY, randomDirection, randomPaint);

    assertEquals(randomX, testPainterPlus.getX(), xMessage);
    assertEquals(randomY, testPainterPlus.getY(), yMessage);
    assertEquals(randomDirection, testPainterPlus.getDirection(), directionMessage);
    assertEquals(randomPaint, testPainterPlus.getMyPaint(), paintMessage);
  }
   
  @Test
  @Order(2)
  @DisplayName("Instantiate a PainterPlus object at (3, 5) => ")
  public void testPainterPlusStartLocation() {
    String xMessage = "Call the parameterized constructor and pass 3 as the argument for x." + messageGap;
    String yMessage = "Call the parameterized constructor and pass 5 as the argument for y." + messageGap;
      
    int startingX = primaryPainterLog.getStartingPosition().getX();
    int startingY = primaryPainterLog.getStartingPosition().getY();

    assertEquals(3, startingX, xMessage);
    assertEquals(5, startingY, yMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Instantiate a PainterPlus object facing \\"east\\" => ")
  public void testPainterPlusFacingNorth() {
    String directionMessage = "Call the parameterized constructor and pass \\"east\\" as the argument for direction." + messageGap;

    String startingDirection = primaryPainterLog.getStartingPosition().getDirection();

    assertEquals("east", startingDirection, directionMessage);
  }

  @Test
  @Order(4)
  @DisplayName("Instantiate a PainterPlus object with 20 units of paint => ")
  public void testPainterPlusStartPaint() {
    String paintMessage = "Call the parameterized constructor and pass 20 as the argument for paint." + messageGap;

    int startingPaint = primaryPainterLog.getStartingPaintCount();

    assertEquals(20, startingPaint, paintMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: The Food Truck (a)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate objects using the no-argument and parameterized
     * constructors. Then use ConstructorsHelper.printConstructors(nameOfObject)
     * to print the constructors for the objects to the console.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`CookieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cookie.java Test")
public class CookieTest {
    
  String messageGap = "\\n       ";
  Cookie testCookie;
  Class cookieClass;
  ArrayList<String> cookieConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testCookie = new Cookie();
    cookieClass = testCookie.getClass();
    cookieConstructors = ConstructorsHelper.getClassConstructorsList(cookieClass, "Cookie");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Cookie class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;

    String expected = "public Cookie()";
    String actual = cookieConstructors.get(0);

    assertEquals(expected, actual, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor in the Cookie class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to flavor." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to isChewy." + messageGap;
    
    String actualConstructor = cookieConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Cookie", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
    assertTrue(actualParameters.contains("boolean"), booleanParameterMessage);
  }

}`}],dataFiles:[]},{name:`Practice: The Food Truck (b)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate objects using the no-argument and parameterized
     * constructors. Then use ConstructorsHelper.printConstructors(nameOfObject)
     * to print the constructors for the objects to the console.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`PieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Pie.java Test")
public class PieTest {
    
  String messageGap = "\\n       ";
  Pie testPie;
  Class pieClass;
  ArrayList<String> pieConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testPie = new Pie();
    pieClass = testPie.getClass();
    pieConstructors = ConstructorsHelper.getClassConstructorsList(pieClass, "Pie");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Pie class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;

    String expected = "public Pie()";
    String actual = pieConstructors.get(0);

    assertEquals(expected, actual, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor in the Pie class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to flavor." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    String intParameterMessage = "The constructor should specify an int parameter to assign a value to diameter." + messageGap;
    
    String actualConstructor = pieConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Pie", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
    assertTrue(actualParameters.contains("int"), intParameterMessage);
  }

}`}],dataFiles:[]},{name:`Practice: The Food Truck (c)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate objects using the no-argument and parameterized
     * constructors. Then use ConstructorsHelper.printConstructors(nameOfObject)
     * to print the constructors for the objects to the console.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`DonutTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

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
  Donut testDonut;
  Class donutClass;
  ArrayList<String> donutConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testDonut = new Donut();
    donutClass = testDonut.getClass();
    donutConstructors = ConstructorsHelper.getClassConstructorsList(donutClass, "Donut");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Donut class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;

    String expected = "public Donut()";
    String actual = donutConstructors.get(0);

    assertEquals(expected, actual, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor in the Donut class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to flavor." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to hasSprinkles." + messageGap;
    
    String actualConstructor = donutConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Donut", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
    assertTrue(actualParameters.contains("boolean"), booleanParameterMessage);
  }

}`}],dataFiles:[]},{name:`Practice: The Food Truck (d)`,lesson:`Lesson 7: Constructors and Inheritance`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Instantiate objects using the no-argument and parameterized
     * constructors. Then use ConstructorsHelper.printConstructors(nameOfObject)
     * to print the constructors for the objects to the console.
     * -----------------------------------------------------------------------------
     */


    
    
    
  }
}`},{path:`ConstructorsHelper.java`,text:`import java.lang.reflect.*;
import java.util.*;

public final class ConstructorsHelper {
  
  private static Class currentClass;
  private static Class superClass;
  private static String currentClassName;
  private static String superClassName;
  private static ArrayList<String> currentClassConstructors;
  private static ArrayList<String> superClassConstructors;

  public static void printConstructors(Object currentObject) {
    setInfo(currentObject);

    System.out.println(getConstructorsHeading());
    System.out.println(getCurrentClassConstructors());
  }
  
  public static String getConstructorsHeading() {
    String result = getCurrentClassName() + " Class Constructors";
    result += "\\n------------------------------";
    return result;
  }
  
  public static void setInfo(Object testObject) {
    currentClass = testObject.getClass();
    superClass = currentClass.getSuperclass();

    currentClassName = currentClass.getSimpleName();
    superClassName = superClass.getSimpleName();

    currentClassConstructors = getClassConstructorsList(currentClass, currentClassName);
    superClassConstructors = getClassConstructorsList(superClass, superClassName);
  }
  
  public static String getCurrentClassName() {
    return currentClassName;
  }
  
  public static String getSuperClassName() {
    return superClassName;
  }
  
  public static ArrayList<String> getClassConstructors() {
    return currentClassConstructors;
  }
  
  public static ArrayList<String> getSuperClassConstructors() {
    return superClassConstructors;
  }
  
  public static ArrayList<String> getClassConstructorsList(Class currentClass, String className) {
    Constructor[] classConstructors = currentClass.getDeclaredConstructors();
    ArrayList<String> constructorsList = constructorsToList(Arrays.toString(classConstructors));
    cleanConstructors(constructorsList);
    return constructorsList;
  }
  
  private static ArrayList<String> constructorsToList(String constructorsAsText) {
    constructorsAsText = constructorsAsText.substring(1, constructorsAsText.length() - 1);
    ArrayList<String> constructorsList = new ArrayList<String>();

    String currentConstructor = "";
    int start = constructorsAsText.indexOf("public");
    int end = constructorsAsText.indexOf(")");

    while (start != -1 && end != -1) {
      currentConstructor = constructorsAsText.substring(start, end + 1);
      constructorsList.add(currentConstructor);
      constructorsAsText = constructorsAsText.substring(end + 1);
      start = constructorsAsText.indexOf("public");
      end = constructorsAsText.indexOf(")");
    }

    constructorsList.add(constructorsAsText);
    return constructorsList;
  }
  
  public static void cleanConstructors(ArrayList<String> classConstructorsList) {
    for (int index = 0; index < classConstructorsList.size(); index++) {
      String currentConstructor = classConstructorsList.get(index);

      currentConstructor = removeJavaLang(currentConstructor);
      currentConstructor.trim();

      classConstructorsList.set(index, currentConstructor);
    }
  }

  public static String getCurrentClassConstructors() {
    ArrayList<String> currentClassConstructors = getClassConstructors();
    return getListAsText(currentClassConstructors);
  }
  
  public static String removeJavaLang(String currentConstructor) {
    String textToFind = "java.lang.";
    int location = currentConstructor.indexOf(textToFind);

    while (location != -1) {
      currentConstructor = currentConstructor.substring(0, location) + currentConstructor.substring(location + textToFind.length());
      location = currentConstructor.indexOf(textToFind);
    }

    return currentConstructor;
  }
  
  public static ArrayList<String> getParameters(String currentConstructor) {
    ArrayList<String> parametersList = new ArrayList<String>();
    
    int start = currentConstructor.indexOf("(") + 1;
    
    currentConstructor = currentConstructor.substring(start);
    int comma = currentConstructor.indexOf(",");
    
    while (comma != -1) {
      String currentParameter = currentConstructor.substring(0, comma);
      parametersList.add(currentParameter);
      currentConstructor = currentConstructor.substring(comma + 1);
      comma = currentConstructor.indexOf(",");
    }
    
    parametersList.add(currentConstructor.substring(0, currentConstructor.length() - 1));
    
    return parametersList;
  }
  
  public static String getListAsText(ArrayList<String> classInfoList) {
    String result = "";

    for (String currentField : classInfoList) {
      result += currentField + "\\n";
    }

    return result.trim();
  }
   
}`}],validationFiles:[{path:`CupcakeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import java.util.ArrayList;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cupcake.java Test")
public class CupcakeTest {
    
  String messageGap = "\\n       ";
  Cupcake testCupcake;
  Class cupcakeClass;
  ArrayList<String> cupcakeConstructors;
  String constructorMessage;

  @BeforeEach
  public void setup() {
    testCupcake = new Cupcake();
    cupcakeClass = testCupcake.getClass();
    cupcakeConstructors = ConstructorsHelper.getClassConstructorsList(cupcakeClass, "Cupcake");
    constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;
  }
  
  @Test
  @Order(1)
  @DisplayName("Write a no-argument constructor in the Cupcake class => ")
  public void testNoArgConstructor() {
    String constructorMessage = "The constructor should be public and have the same name as the class." + messageGap;

    String expected = "public Cupcake()";
    String actual = cupcakeConstructors.get(0);

    assertEquals(expected, actual, constructorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Write a parameterized constructor in the Cupcake class => ")
  public void testParameterizedConstructor() {
    String stringParameterMessage = "The constructor should specify a String parameter to assign a value to flavor." + messageGap;
    String doubleParameterMessage = "The constructor should specify a double parameter to assign a value to price." + messageGap;
    String booleanParameterMessage = "The constructor should specify a boolean parameter to assign a value to isMini." + messageGap;
    
    String actualConstructor = cupcakeConstructors.get(1);
    String actualName = actualConstructor.substring(0, actualConstructor.indexOf("("));
    ArrayList<String> actualParameters = ConstructorsHelper.getParameters(actualConstructor);
    
    assertEquals("public Cupcake", actualName, constructorMessage);
    assertTrue(actualParameters.contains("String"), stringParameterMessage);
    assertTrue(actualParameters.contains("double"), doubleParameterMessage);
    assertTrue(actualParameters.contains("boolean"), booleanParameterMessage);
  }

}`}],dataFiles:[]},{name:`Predict and Run: Variables`,lesson:`Lesson 8: Variables`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int score = 125;
    double temperature = 89.3;
    boolean status = true;
    String helloGreeting = "Hello World";
    String welcomeGreeting = new String("Welcome to Java");

    System.out.println("score: " + score);
    System.out.println("temperature: " + temperature);
    System.out.println("status: " + status);
    System.out.println("helloGreeting: " + helloGreeting);
    System.out.println("welcomeGreeting: " + welcomeGreeting);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Variables`,lesson:`Lesson 8: Variables`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    int score = 125;
    double temperature = 89.3;
    boolean status = true;
    String helloGreeting = "Hello World";
    String welcomeGreeting = new String("Welcome to Java");

    System.out.println("score: " + score);
    System.out.println("temperature: " + temperature);
    System.out.println("status: " + status);
    System.out.println("helloGreeting: " + helloGreeting);
    System.out.println("welcomeGreeting: " + welcomeGreeting);

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */





    
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Accessor Methods`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Course graphicDesign = new Course(6);
    Course algebra = new Course(24);

    int numStudents = graphicDesign.getNumStudents();
    System.out.println("Graphic Design Students: " + numStudents);

    System.out.println("Algebra Students: " + algebra.getNumStudents());

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */

    
    
  }
}`},{path:`Course.java`,text:`/*
 * Represents a course
 */
public class Course {

  private int numStudents;   // The number of students in a course

  /*
   * Sets numStudents to the specified
   * number of students in a course
   */
  public Course(int numStudents) {
    this.numStudents = numStudents;
  }

  /*
   * Returns the number of students in the course
   */
  public int getNumStudents() {
    return numStudents;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Accessor Methods #1`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Course graphicDesign = new Course(6);
    Course algebra = new Course(24);

    int numStudents = graphicDesign.getNumStudents();
    System.out.println("Graphic Design Students: " + numStudents);

    System.out.println("Algebra Students: " + algebra.getNumStudents());

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */

    
    
  }
}`},{path:`Course.java`,text:`/*
 * Represents a course
 */
public class Course {

  private int numStudents;   // The number of students in a course

  /*
   * Sets numStudents to the specified
   * number of students in a course
   */
  public Course(int numStudents) {
    this.numStudents = numStudents;
  }

  /*
   * Returns the number of students in the course
   */
  public int getNumStudents() {
    return numStudents;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Accessor Methods #2`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    Course graphicDesign = new Course(6);
    Course algebra = new Course(24);

    int numStudents = graphicDesign.getNumStudents();
    System.out.println("Graphic Design Students: " + numStudents);

    System.out.println("Algebra Students: " + algebra.getNumStudents());

    /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */

    
    
  }
}`},{path:`Course.java`,text:`/*
 * Represents a course
 */
public class Course {

  private int numStudents;   // The number of students in a course

  /*
   * Sets numStudents to the specified
   * number of students in a course
   */
  public Course(int numStudents) {
    this.numStudents = numStudents;
  }

  /*
   * Returns the number of students in the course
   */
  public int getNumStudents() {
    return numStudents;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: Accessor Methods (a)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the accessor method on the dallasWeather object and print the result.
     * -----------------------------------------------------------------------------
     */

    // Creates a Weather object
    Weather dallasWeather = new Weather(92.5);

    
    
  }
}`},{path:`Weather.java`,text:`/*
 * Represents the weather conditions in a city
 */
public class Weather {

  private double temperature;   // The temperature in a city

  /*
   * Sets temperature to the specified temperature
   */
  public Weather(double temperature) {
    this.temperature = temperature;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write an accessor method to return the value assigned to temperature.
   * -----------------------------------------------------------------------------
   */


  


  
}`}],validationFiles:[{path:`WeatherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Weather.java Test")
public class WeatherTest {
    
  String messageGap = "\\n       ";
  Weather testWeather;
  String accessorMessage;
  
  @BeforeEach
  public void setup() {
    accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
  }
  
  @Test
  @Order(1)
  @DisplayName("Write an accessor method for the temperature instance variable => ")
  public void testGetTemperature() {
    double randomTemp = (Math.random() * (100 - 20)) + 20;
    testWeather = new Weather(randomTemp);
    assertEquals(randomTemp, testWeather.getTemperature(), accessorMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Accessor Methods (b)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the accessor method on the theGiver object and print the result.
     * -----------------------------------------------------------------------------
     */

    // Creates a Book object
    Book theGiver = new Book("The Giver");

    
    
  }
}`},{path:`Book.java`,text:`/*
 * Represents a book a student has read
 */
public class Book {

  private String title;    // The title of a book

  /*
   * Sets title to the specified title
   */
  public Book(String title) {
    this.title = title;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write an accessor method to return the value assigned to title.
   * -----------------------------------------------------------------------------
   */


  


  
}`}],validationFiles:[{path:`BookTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Book.java Test")
public class BookTest {
    
  String messageGap = "\\n       ";
  Book testBook;
  String accessorMessage;
  String[] testBooks;

  @BeforeEach
  public void setup() {
    accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
    testBooks = new String[]{"Great Expectations", "Odyssey", "Emma", "Frankenstein", "Charlotte's Web"};
  }

  @Test
  @Order(1)
  @DisplayName("Write an accessor method for the title instance variable => ")
  public void testGetTitle() {
    int randomIndex = (int)(Math.random() * testBooks.length);
    String randomBook = testBooks[randomIndex];

    testBook = new Book(randomBook);
    assertEquals(randomBook, testBook.getTitle(), accessorMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Accessor Methods (c)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the accessor method on the birthdayPlaylist object and print the result.
     * -----------------------------------------------------------------------------
     */

    // Creates a Playlist object
    Playlist birthdayPlaylist = new Playlist(28);
    
    
    
  }
}`},{path:`Playlist.java`,text:`/*
 * Represents a playlist used at an event
 */
public class Playlist {

  private int numSongs;    // The number of songs in a playlist

  /*
   * Sets numSongs to the specified number of songs
   */
  public Playlist(int numSongs) {
    this.numSongs = numSongs;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write an accessor method to return the value assigned to numSongs.
   * -----------------------------------------------------------------------------
   */


  


  
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
  Playlist testPlaylist;
  String accessorMessage;

  @BeforeEach
  public void setup() {
      accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write an accessor method for the numSongs instance variable => ")
  public void testGetNumSongs() {
    int randomNumSongs = (int)(Math.random() * (100 - 50)) + 50;

    testPlaylist = new Playlist(randomNumSongs);
    assertEquals(randomNumSongs, testPlaylist.getNumSongs(), accessorMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Accessor Methods (d)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the accessor method on the tetrisChamp object and print the result.
     * -----------------------------------------------------------------------------
     */

    // Creates a Player object
    Player tetrisChamp = new Player(682);

    
    
  }
}`},{path:`Player.java`,text:`/*
 * Represents a player in a game
 */
public class Player {

  private int highScore;   // A player's high score

  /*
   * Sets highScore to the specified high score
   */
  public Player(int highScore) {
    this.highScore = highScore;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write an accessor method to return the value assigned to highScore.
   * -----------------------------------------------------------------------------
   */


  


  
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
  String accessorMessage;

  @BeforeEach
  public void setup() {
    accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
  }

  @Test
  @Order(1)
  @DisplayName("Write an accessor method for the highScore instance variable => ")
  public void testGetHighScore() {
    int randomHighScore = (int)(Math.random() * (1000 - 500)) + 500;
  
    testPlayer = new Player(randomHighScore);
    assertEquals(randomHighScore, testPlayer.getHighScore(), accessorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Accessor Methods`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its accessor methods
     * and print the results.
     * -----------------------------------------------------------------------------
     */


    


    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * accessor methods and print the results.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`}],validationFiles:[{path:`DessertTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert.java Test")
public class DessertTest {
    
  String messageGap = "\\n       ";
  Dessert testDessert;
  String accessorMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testDessert = new Dessert(randomFlavor, randomPrice);
  }

  @Test
  @Order(1)
  @DisplayName("Write an accessor method for the flavor instance variable => ")
  public void testGetFlavor() {
    assertEquals(randomFlavor, testDessert.getFlavor(), accessorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Write an accessor method for the price instance variable => ")
  public void testGetPrice() {
    assertEquals(randomPrice, testDessert.getPrice(), accessorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (a)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its accessor methods
     * and print the results.
     * -----------------------------------------------------------------------------
     */


    


    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * accessor methods and print the results.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`}],validationFiles:[{path:`CookieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cookie.java Test")
public class CookieTest {
    
  String messageGap = "\\n       ";
  Cookie testCookie;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Cookie class should extend the Dessert class so it inherits the accessor methods in the Dessert class." + messageGap;
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "shortbread"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testCookie = new Cookie(randomFlavor, randomPrice, true);
  }

  @Test
  @Order(1)
  @DisplayName("getFlavor() returns the flavor of the Cookie => ")
  public void testGetFlavor() {
    assertEquals(randomFlavor, testCookie.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("getPrice() returns the price of the Cookie => ")
  public void testGetPrice() {
    assertEquals(randomPrice, testCookie.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write an accessor method for whether or not a cookie is chewy => ")
  public void testGetIsChewy() {
    String accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
    assertTrue(testCookie.getIsChewy(), accessorMessage);
  }

}`}],dataFiles:[]},{name:`Practice: The Food Truck (b)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its accessor methods
     * and print the results.
     * -----------------------------------------------------------------------------
     */


    


    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * accessor methods and print the results.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`}],validationFiles:[{path:`PieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Pie.java Test")
public class PieTest {
    
  String messageGap = "\\n       ";
  Pie testPie;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;
  int randomDiameter;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Pie class should extend the Dessert class so it inherits the accessor methods in the Dessert class." + messageGap;
    testFlavors = new String[]{"chocolate", "buttermilk", "apple", "pecan", "pumpkin"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;
    randomDiameter = (int)(Math.random() * (24 - 6)) + 6;

    testPie = new Pie(randomFlavor, randomPrice, randomDiameter);
  }

  @Test
  @Order(1)
  @DisplayName("getFlavor() returns the flavor of the Pie => ")
  public void testGetFlavor() {
    assertEquals(randomFlavor, testPie.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("getPrice() returns the price of the Pie => ")
  public void testGetPrice() {
    assertEquals(randomPrice, testPie.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write an accessor method for the diameter => ")
  public void testGetDiameter() {
    String accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
    assertEquals(randomDiameter, testPie.getDiameter(), accessorMessage);
  }

}`}],dataFiles:[]},{name:`Practice: The Food Truck (c)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its accessor methods
     * and print the results.
     * -----------------------------------------------------------------------------
     */


    


    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * accessor methods and print the results.
     * -----------------------------------------------------------------------------
     */



    
    
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
  Donut testDonut;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Donut class should extend the Dessert class so it inherits the accessor methods in the Dessert class." + messageGap;
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "blueberry", "cinnamon"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testDonut = new Donut(randomFlavor, randomPrice, true);
  }

  @Test
  @Order(1)
  @DisplayName("getFlavor() returns the flavor of the Donut => ")
  public void testGetFlavor() {
    assertEquals(randomFlavor, testDonut.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("getPrice() returns the price of the Donut => ")
  public void testGetPrice() {
    assertEquals(randomPrice, testDonut.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write an accessor method for whether or not it has sprinkles => ")
  public void testGetHasSprinkles() {
    String accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
    assertTrue(testDonut.getHasSprinkles(), accessorMessage);
  }

}`}],dataFiles:[]},{name:`Practice: The Food Truck (d)`,lesson:`Lesson 9: Accessor Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its accessor methods
     * and print the results.
     * -----------------------------------------------------------------------------
     */


    


    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * accessor methods and print the results.
     * -----------------------------------------------------------------------------
     */



    
    
  }
}`}],validationFiles:[{path:`CupcakeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cupcake.java Test")
public class CupcakeTest {
    
  String messageGap = "\\n       ";
  Cupcake testCupcake;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Cupcake class should extend the Dessert class so it inherits the accessor methods in the Dessert class." + messageGap;
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "red velvet", "oreo"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testCupcake = new Cupcake(randomFlavor, randomPrice, true);
  }

  @Test
  @Order(1)
  @DisplayName("getFlavor() returns the flavor of the Cupcake => ")
  public void testGetFlavor() {
    assertEquals(randomFlavor, testCupcake.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("getPrice() returns the price of the Cupcake => ")
  public void testGetPrice() {
    assertEquals(randomPrice, testCupcake.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write an accessor method for whether or not it is a mini cupcake => ")
  public void testGetIsMini() {
    String accessorMessage = "The body of an accessor method should contain the return keyword followed by the name of the instance variable." + messageGap;
    assertTrue(testCupcake.getIsMini(), accessorMessage);
  }

}`}],dataFiles:[]},{name:`Predict and Run: Operators and Expressions`,lesson:`Lesson 10: Operators and Expressions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Expressions #1`,lesson:`Lesson 10: Operators and Expressions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Expressions #2`,lesson:`Lesson 10: Operators and Expressions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Expressions #3`,lesson:`Lesson 10: Operators and Expressions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Operators #1`,lesson:`Lesson 10: Operators and Expressions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Operators #2`,lesson:`Lesson 10: Operators and Expressions`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`}],validationFiles:[],dataFiles:[]},{name:`Predict and Run: Mutator Methods`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Clothes.java`,text:`/*
 * Represents a clothing item
 */
public class Clothes {

  private String size;    // The size of a clothing item
  private double price;   // The price of a clothing item

  /*
   * Sets the size to small and price to 10.99
   */
  public Clothes() {
    size = "small";
    price = 10.99;
  }

  /*
   * Sets the size to the specified size and
   * the price to the specified price
   */
  public Clothes(String size, double price) {
    this.size = size;
    this.price = price;
  }

  /*
   * Returns the value of size of the clothing item
   */
  public String getSize() {
    return size;
  }

  /*
   * Returns the value of price of the clothing item
   */
  public double getPrice() {
    return price;
  }

  /*
   * Sets the size of the clothing item to the specified size
   */
  public void setSize(String newSize) {
    size = newSize;
  }

  /*
   * Sets the price of the clothing item to the specified price
   */
  public void setPrice(double newPrice) {
    price = newPrice;
  }
  
}`},{path:`Shirt.java`,text:`/*
 * Represents a shirt
 */
public class Shirt extends Clothes {

  private boolean hasButtons;    // Whether or not a shirt has buttons

  /*
   * Sets the size to small, price to 10.99, and hasButtons to true
   */
  public Shirt() {
    hasButtons = true;
  }

  /*
   * Sets the size to the specified size, the price to the specified
   * price, and hasButtons to the specified status
   */
  public Shirt(String size, double price, boolean hasButtons) {
    super(size, price);
    this.hasButtons = hasButtons;
  }

  /*
   * Returns whether or not the shirt has buttons
   */
  public boolean getHasButtons() {
    return hasButtons;
  }

  /*
   * Sets hasButtons to the specified status
   */
  public void setHasButtons(boolean newHasButtons) {
    hasButtons = newHasButtons;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Mutator Methods #1`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Clothes.java`,text:`/*
 * Represents a clothing item
 */
public class Clothes {

  private String size;    // The size of a clothing item
  private double price;   // The price of a clothing item

  /*
   * Sets the size to small and price to 10.99
   */
  public Clothes() {
    size = "small";
    price = 10.99;
  }

  /*
   * Sets the size to the specified size and
   * the price to the specified price
   */
  public Clothes(String size, double price) {
    this.size = size;
    this.price = price;
  }

  /*
   * Returns the value of size of the clothing item
   */
  public String getSize() {
    return size;
  }

  /*
   * Returns the value of price of the clothing item
   */
  public double getPrice() {
    return price;
  }

  /*
   * Sets the size of the clothing item to the specified size
   */
  public void setSize(String newSize) {
    size = newSize;
  }

  /*
   * Sets the price of the clothing item to the specified price
   */
  public void setPrice(double newPrice) {
    price = newPrice;
  }
  
}`},{path:`Shirt.java`,text:`/*
 * Represents a shirt
 */
public class Shirt extends Clothes {

  private boolean hasButtons;    // Whether or not a shirt has buttons

  /*
   * Sets the size to small, price to 10.99, and hasButtons to true
   */
  public Shirt() {
    hasButtons = true;
  }

  /*
   * Sets the size to the specified size, the price to the specified
   * price, and hasButtons to the specified status
   */
  public Shirt(String size, double price, boolean hasButtons) {
    super(size, price);
    this.hasButtons = hasButtons;
  }

  /*
   * Returns whether or not the shirt has buttons
   */
  public boolean getHasButtons() {
    return hasButtons;
  }

  /*
   * Sets hasButtons to the specified status
   */
  public void setHasButtons(boolean newHasButtons) {
    hasButtons = newHasButtons;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Mutator Methods #2`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Clothes.java`,text:`/*
 * Represents a clothing item
 */
public class Clothes {

  private String size;    // The size of a clothing item
  private double price;   // The price of a clothing item

  /*
   * Sets the size to small and price to 10.99
   */
  public Clothes() {
    size = "small";
    price = 10.99;
  }

  /*
   * Sets the size to the specified size and
   * the price to the specified price
   */
  public Clothes(String size, double price) {
    this.size = size;
    this.price = price;
  }

  /*
   * Returns the value of size of the clothing item
   */
  public String getSize() {
    return size;
  }

  /*
   * Returns the value of price of the clothing item
   */
  public double getPrice() {
    return price;
  }

  /*
   * Sets the size of the clothing item to the specified size
   */
  public void setSize(String newSize) {
    size = newSize;
  }

  /*
   * Sets the price of the clothing item to the specified price
   */
  public void setPrice(double newPrice) {
    price = newPrice;
  }
  
}`},{path:`Shirt.java`,text:`/*
 * Represents a shirt
 */
public class Shirt extends Clothes {

  private boolean hasButtons;    // Whether or not a shirt has buttons

  /*
   * Sets the size to small, price to 10.99, and hasButtons to true
   */
  public Shirt() {
    hasButtons = true;
  }

  /*
   * Sets the size to the specified size, the price to the specified
   * price, and hasButtons to the specified status
   */
  public Shirt(String size, double price, boolean hasButtons) {
    super(size, price);
    this.hasButtons = hasButtons;
  }

  /*
   * Returns whether or not the shirt has buttons
   */
  public boolean getHasButtons() {
    return hasButtons;
  }

  /*
   * Sets hasButtons to the specified status
   */
  public void setHasButtons(boolean newHasButtons) {
    hasButtons = newHasButtons;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: Mutator Methods (a)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the mutator method on the dallasWeather object to assign a new value
     * to its temperature instance variable, then print the updated value.
     * -----------------------------------------------------------------------------
     */

    // Creates a Weather object
    Weather dallasWeather = new Weather(92.5);

    // Prints the current value assigned to temperature
    System.out.println("Dallas Current Temperature: " + dallasWeather.getTemperature());

    
    
    
  }
}`},{path:`Weather.java`,text:`/*
 * Represents the weather conditions in a city
 */
public class Weather {

  private double temperature;   // The temperature in a city

  /*
   * Sets temperature to the specified temperature
   */
  public Weather(double temperature) {
    this.temperature = temperature;
  }

  /*
   * Returns the value assigned to temperature
   */
  public double getTemperature() {
    return temperature;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a mutator method to change the value assigned to temperature.
   * -----------------------------------------------------------------------------
   */



  
}`}],validationFiles:[{path:`WeatherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Weather.java Test")
public class WeatherTest {
  
  String messageGap = "\\n       ";
  Weather testWeather;
  String mutatorMessage;

  @BeforeEach
  public void setup() {
    mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    double randomTemp = (Math.random() * (100 - 20)) + 20;
    testWeather = new Weather(randomTemp);
  }

  @Test
  @Order(1)
  @DisplayName("Write a mutator method for the temperature instance variable => ")
  public void testSetTemperature() {
    double randomTemp = (Math.random() * (100 - 20)) + 20;
    testWeather.setTemperature(randomTemp);
    assertEquals(randomTemp, testWeather.getTemperature(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Mutator Methods (b)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the mutator method on the favBook object to assign a new value
     * to its title instance variable, then print the updated value.
     * -----------------------------------------------------------------------------
     */

    // Creates a Book object
    Book favBook = new Book("The Giver");

    // Prints the current value assigned to title
    System.out.println("Favorite Book Title: " + favBook.getTitle());


    
    
  }
}`},{path:`Book.java`,text:`/*
 * Represents a book a student has read
 */
public class Book {

  private String title;    // The title of a book

  /*
   * Sets title to the specified title
   */
  public Book(String title) {
    this.title = title;
  }

  /*
   * Returns the value assigned to title
   */
  public String getTitle() {
    return title;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a mutator method to change the value assigned to title.
   * -----------------------------------------------------------------------------
   */



  
}`}],validationFiles:[{path:`BookTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Book.java Test")
public class BookTest {
  
  String messageGap = "\\n       ";
  Book testBook;
  String mutatorMessage;
  String[] testBooks;

  @BeforeEach
  public void setup() {
    mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    testBooks = new String[]{"Great Expectations", "Odyssey", "Emma", "Frankenstein", "Charlotte's Web"};

    int randomIndex = (int)(Math.random() * testBooks.length);
    String randomBook = testBooks[randomIndex];
    testBook = new Book(randomBook);
  }

  @Test
  @Order(1)
  @DisplayName("Write a mutator method for the title instance variable => ")
  public void testSetTitle() {
    int randomIndex = (int)(Math.random() * testBooks.length);
    String randomBook = testBooks[randomIndex];
    testBook.setTitle(randomBook);
    assertEquals(randomBook, testBook.getTitle(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Mutator Methods (c)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the mutator method on the birthdayPlaylist object to assign a new value
     * to its numSongs instance variable, then print the updated value.
     * -----------------------------------------------------------------------------
     */

    // Creates a Playlist object
    Playlist birthdayPlaylist = new Playlist(28);
    
    // Prints the current value assigned to numSongs
    System.out.println("Current Number of Songs: " + birthdayPlaylist.getNumSongs());


    
    
  }
}`},{path:`Playlist.java`,text:`/*
 * Represents a playlist used at an event
 */
public class Playlist {

  private int numSongs;    // The number of songs in a playlist

  /*
   * Sets numSongs to the specified number of songs
   */
  public Playlist(int numSongs) {
    this.numSongs = numSongs;
  }

  /*
   * Returns the value assigned to numSongs
   */
  public int getNumSongs() {
    return numSongs;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a mutator method to change the value assigned to numSongs.
   * -----------------------------------------------------------------------------
   */



  
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
  Playlist testPlaylist;
  String mutatorMessage;

  @BeforeEach
  public void setup() {
    mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    int randomNumSongs = (int)(Math.random() * (100 - 50)) + 50;
    testPlaylist = new Playlist(randomNumSongs);
  }

  @Test
  @Order(1)
  @DisplayName("Write a mutator method for the numSongs instance variable => ")
  public void testSetNumSongs() {
    int randomNumSongs = (int)(Math.random() * (100 - 50)) + 50;
    testPlaylist.setNumSongs(randomNumSongs);
    assertEquals(randomNumSongs, testPlaylist.getNumSongs(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Skill Building: Mutator Methods (d)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Call the mutator method on the tetrisChamp object to assign a new value
     * to its highScore instance variable, then print the updated value.
     * -----------------------------------------------------------------------------
     */

    // Creates a Player object
    Player tetrisChamp = new Player(682);

    // Prints the current value assigned to highScore
    System.out.println("Current High Score: " + tetrisChamp.getHighScore());


    
    
  }
}`},{path:`Player.java`,text:`/*
 * Represents a player in a game
 */
public class Player {

  private int highScore;   // A player's high score

  /*
   * Sets highScore to the specified high score
   */
  public Player(int highScore) {
    this.highScore = highScore;
  }

  /*
   * Returns the value assigned to highScore
   */
  public int getHighScore() {
    return highScore;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a mutator method to change the value assigned to highScore.
   * -----------------------------------------------------------------------------
   */



  
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
  String mutatorMessage;

  @BeforeEach
  public void setup() {
    mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    int randomHighScore = (int)(Math.random() * (1000 - 500)) + 500;
    testPlayer = new Player(randomHighScore);
  }

  @Test
  @Order(1)
  @DisplayName("Write a mutator method for the highScore instance variable => ")
  public void testSetHighScore() {
    int randomHighScore = (int)(Math.random() * (1000 - 500)) + 500;
    testPlayer.setHighScore(randomHighScore);
    assertEquals(randomHighScore, testPlayer.getHighScore(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: Mutator Methods`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object to get input from a user
    Scanner input = new Scanner(System.in);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its mutator methods
     * to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * mutator methods to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */




    // Closes the Scanner input
    input.close();
    
  }
}`}],validationFiles:[{path:`DessertTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert.java Test")
public class DessertTest {
  
  String messageGap = "\\n       ";
  Dessert testDessert;
  String mutatorMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testDessert = new Dessert(randomFlavor, randomPrice);
  }

  @Test
  @Order(1)
  @DisplayName("Write a mutator method for the flavor instance variable => ")
  public void testSetFlavor() {
    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    testDessert.setFlavor(randomFlavor);
    assertEquals(randomFlavor, testDessert.getFlavor(), mutatorMessage);
  }

  @Test
  @Order(2)
  @DisplayName("Write a mutator method for the price instance variable => ")
  public void testSetPrice() {
    randomPrice = (Math.random() * (20 - 2)) + 2;
    testDessert.setPrice(randomPrice);
    assertEquals(randomPrice, testDessert.getPrice(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (a)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object to get input from a user
    Scanner input = new Scanner(System.in);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its mutator methods
     * to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * mutator methods to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */




    // Closes the Scanner input
    input.close();
    
  }
}`}],validationFiles:[{path:`CookieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cookie.java Test")
public class CookieTest {
  
  String messageGap = "\\n       ";
  Cookie testCookie;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Cookie class should extend the Dessert class so it inherits the mutator methods in the Dessert class." + messageGap;

    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testCookie = new Cookie(randomFlavor, randomPrice, false);
  }

  @Test
  @Order(1)
  @DisplayName("setFlavor() modifies the flavor of the Cookie => => ")
  public void testSetFlavor() {
    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    testCookie.setFlavor(randomFlavor);
    assertEquals(randomFlavor, testCookie.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("setPrice() modifies the price of the Cookie => ")
  public void testSetPrice() {
    randomPrice = (Math.random() * (20 - 2)) + 2;
    testCookie.setPrice(randomPrice);
    assertEquals(randomPrice, testCookie.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write a mutator method for the isChewy instance variable => ")
  public void testSetIsChewy() {
    String mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    testCookie.setIsChewy(true);
    assertTrue(testCookie.getIsChewy(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (b)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object to get input from a user
    Scanner input = new Scanner(System.in);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its mutator methods
     * to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * mutator methods to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */




    // Closes the Scanner input
    input.close();
    
  }
}`}],validationFiles:[{path:`PieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Pie.java Test")
public class PieTest {
  
  String messageGap = "\\n       ";
  Pie testPie;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;
  int randomDiameter;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Pie class should extend the Dessert class so it inherits the mutator methods in the Dessert class." + messageGap;

    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;
    randomDiameter = (int)(Math.random() * (24 - 6)) + 6;

    testPie = new Pie(randomFlavor, randomPrice, randomDiameter);
  }

  @Test
  @Order(1)
  @DisplayName("setFlavor() modifies the flavor of the Pie => => ")
  public void testSetFlavor() {
    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    testPie.setFlavor(randomFlavor);
    assertEquals(randomFlavor, testPie.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("setPrice() modifies the price of the Pie => ")
  public void testSetPrice() {
    randomPrice = (Math.random() * (20 - 2)) + 2;
    testPie.setPrice(randomPrice);
    assertEquals(randomPrice, testPie.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write a mutator method for the diameter instance variable => ")
  public void testSetDiameter() {
    String mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    randomDiameter = (int)(Math.random() * (24 - 6)) + 6;
    testPie.setDiameter(randomDiameter);

    assertEquals(randomDiameter, testPie.getDiameter(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (c)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object to get input from a user
    Scanner input = new Scanner(System.in);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its mutator methods
     * to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * mutator methods to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */




    // Closes the Scanner input
    input.close();
    
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
  Donut testDonut;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Donut class should extend the Dessert class so it inherits the mutator methods in the Dessert class." + messageGap;

    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testDonut = new Donut(randomFlavor, randomPrice, false);
  }

  @Test
  @Order(1)
  @DisplayName("setFlavor() modifies the flavor of the Donut => => ")
  public void testSetFlavor() {
    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    testDonut.setFlavor(randomFlavor);
    assertEquals(randomFlavor, testDonut.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("setPrice() modifies the price of the Donut => ")
  public void testSetPrice() {
    randomPrice = (Math.random() * (20 - 2)) + 2;
    testDonut.setPrice(randomPrice);
    assertEquals(randomPrice, testDonut.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write a mutator method for the hasSprinkles instance variable => ")
  public void testSetHasSprinkles() {
    String mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    testDonut.setHasSprinkles(true);
    assertTrue(testDonut.getHasSprinkles(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Practice: The Food Truck (d)`,lesson:`Lesson 11: Mutator Methods`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object to get input from a user
    Scanner input = new Scanner(System.in);

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate a Dessert object, then call its mutator methods
     * to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass, then call its
     * mutator methods to change the values assigned to the instance variables.
     * -----------------------------------------------------------------------------
     */




    // Closes the Scanner input
    input.close();
    
  }
}`}],validationFiles:[{path:`CupcakeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cupcake.java Test")
public class CupcakeTest {
  
  String messageGap = "\\n       ";
  Cupcake testCupcake;
  String superclassMessage;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    superclassMessage = "The Cupcake class should extend the Dessert class so it inherits the mutator methods in the Dessert class." + messageGap;

    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testCupcake = new Cupcake(randomFlavor, randomPrice, false);
  }

  @Test
  @Order(1)
  @DisplayName("setFlavor() modifies the flavor of the Cupcake => => ")
  public void testSetFlavor() {
    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    testCupcake.setFlavor(randomFlavor);
    assertEquals(randomFlavor, testCupcake.getFlavor(), superclassMessage);
  }

  @Test
  @Order(2)
  @DisplayName("setPrice() modifies the price of the Cupcake => ")
  public void testSetPrice() {
    randomPrice = (Math.random() * (20 - 2)) + 2;
    testCupcake.setPrice(randomPrice);
    assertEquals(randomPrice, testCupcake.getPrice(), superclassMessage);
  }

  @Test
  @Order(3)
  @DisplayName("Write a mutator method for the isMini instance variable => ")
  public void testSetIsMini() {
    String mutatorMessage = "The mutator method should specify a parameter that has the same data type as the instance variable.";
    mutatorMessage += "\\n        The body of the mutator method should assign the value passed to the parameter to the instance variable.";
    mutatorMessage += messageGap;

    testCupcake.setIsMini(true);
    assertTrue(testCupcake.getIsMini(), mutatorMessage);
  }
}`}],dataFiles:[]},{name:`Predict and Run: Printing Objects`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Returns the title of the content
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the year the content was released
   */
  public int getYear() {
    return year;
  }

  /*
   * Sets the title of the content to newTitle
   */
  public void setTitle(String newTitle) {
    title = newTitle;
  }

  /*
   * Sets the year the content was released to newYear
   */
  public void setYear(int newYear) {
    year = newYear;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Returns the length of the movie in minutes
   */
  public int getRunningTime() {
    return runningTime;
  }

  /*
   * Sets runningTime to newRunningTime
   */
  public void setRunningTime(int newRunningTime) {
    runningTime = newRunningTime;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /*
   * Sets numEpisodes to 1
   */
  public TVShow() {
    numEpisodes = 1;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and numEpisodes to the specified numEpisodes
   */
  public TVShow(String title, int year, int numEpisodes) {
    super(title, year);
    this.numEpisodes = numEpisodes;
  }

  /*
   * Returns the number of episodes
   */
  public int getNumEpisodes() {
    return numEpisodes;
  }

  /*
   * Sets numEpisodes to newNumEpisodes
   */
  public void setNumEpisodes(int newNumEpisodes) {
    numEpisodes = newNumEpisodes;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Printing Objects #1`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Returns the title of the content
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the year the content was released
   */
  public int getYear() {
    return year;
  }

  /*
   * Sets the title of the content to newTitle
   */
  public void setTitle(String newTitle) {
    title = newTitle;
  }

  /*
   * Sets the year the content was released to newYear
   */
  public void setYear(int newYear) {
    year = newYear;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Returns the length of the movie in minutes
   */
  public int getRunningTime() {
    return runningTime;
  }

  /*
   * Sets runningTime to newRunningTime
   */
  public void setRunningTime(int newRunningTime) {
    runningTime = newRunningTime;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /*
   * Sets numEpisodes to 1
   */
  public TVShow() {
    numEpisodes = 1;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and numEpisodes to the specified numEpisodes
   */
  public TVShow(String title, int year, int numEpisodes) {
    super(title, year);
    this.numEpisodes = numEpisodes;
  }

  /*
   * Returns the number of episodes
   */
  public int getNumEpisodes() {
    return numEpisodes;
  }

  /*
   * Sets numEpisodes to newNumEpisodes
   */
  public void setNumEpisodes(int newNumEpisodes) {
    numEpisodes = newNumEpisodes;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Investigate and Modify: Printing Objects #2`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
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
}`},{path:`Content.java`,text:`/*
 * Represents content on a streaming app
 */
public class Content {

  private String title;     // The title of the content
  private int year;         // The year the content was released

  /*
   * Sets the title to "unknown" and year to 1990
   */
  public Content() {
    this("unknown", 1990);
  }

  /*
   * Sets the title to the specified title and
   * the year to the specified year
   */
  public Content(String title, int year) {
    this.title = title;
    this.year = year;
  }

  /*
   * Returns the title of the content
   */
  public String getTitle() {
    return title;
  }

  /*
   * Returns the year the content was released
   */
  public int getYear() {
    return year;
  }

  /*
   * Sets the title of the content to newTitle
   */
  public void setTitle(String newTitle) {
    title = newTitle;
  }

  /*
   * Sets the year the content was released to newYear
   */
  public void setYear(int newYear) {
    year = newYear;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  

}`},{path:`Movie.java`,text:`/*
 * Represents a movie on a streaming app
 */
public class Movie extends Content {

  private int runningTime;  // The length of the movie in minutes

  /*
   * Sets the runningTime to 60
   */
  public Movie() {
    runningTime = 60;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and the runningTime to the specified running time
   */
  public Movie(String title, int year, int runningTime) {
    super(title, year);
    this.runningTime = runningTime;
  }

  /*
   * Returns the length of the movie in minutes
   */
  public int getRunningTime() {
    return runningTime;
  }

  /*
   * Sets runningTime to newRunningTime
   */
  public void setRunningTime(int newRunningTime) {
    runningTime = newRunningTime;
  }

  /* ---- 🔎 ADD YOUR CODE BELOW THIS LINE ---- */




  
  
}`},{path:`TVShow.java`,text:`/*
 * Represents a TV show on a streaming app
 */
public class TVShow extends Content {

  private int numEpisodes;  // The number of episodes

  /*
   * Sets numEpisodes to 1
   */
  public TVShow() {
    numEpisodes = 1;
  }

  /*
   * Sets the title to the specified title, the year to the specified
   * year, and numEpisodes to the specified numEpisodes
   */
  public TVShow(String title, int year, int numEpisodes) {
    super(title, year);
    this.numEpisodes = numEpisodes;
  }

  /*
   * Returns the number of episodes
   */
  public int getNumEpisodes() {
    return numEpisodes;
  }

  /*
   * Sets numEpisodes to newNumEpisodes
   */
  public void setNumEpisodes(int newNumEpisodes) {
    numEpisodes = newNumEpisodes;
  }
  
}`}],validationFiles:[],dataFiles:[]},{name:`Skill Building: The toString() Method (a)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the chicagoWeather object.
     * -----------------------------------------------------------------------------
     */

    // Creates a Weather object
    Weather chicagoWeather = new Weather(57.1);

    
    
  }
}`},{path:`Weather.java`,text:`/*
 * Represents the weather conditions in a city
 */
public class Weather {

  private double temperature;   // The temperature in a city

  /*
   * Sets temperature to the specified temperature
   */
  public Weather(double temperature) {
    this.temperature = temperature;
  }

  /*
   * Returns the value assigned to temperature
   */
  public double getTemperature() {
    return temperature;
  }

  /*
   * Sets temperature to newTemperature
   */
  public void setTemperature(double newTemperature) {
    temperature = newTemperature;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a toString() method that returns a String containing the value
   * assigned to the temperature instance variable. For example, Temperature: 92.5
   * -----------------------------------------------------------------------------
   */




  
  
}`}],validationFiles:[{path:`WeatherTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Weather.java Test")
public class WeatherTest {
  
  String messageGap = "\\n       ";
  Weather testWeather;
  double randomTemp;

  @BeforeEach
  public void setup() {
    randomTemp = (Math.random() * (100 - 20)) + 20;
    testWeather = new Weather(randomTemp);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Weather object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the text \\"Temperature: \\" followed by the value assigned to the temperature instance variable." + messageGap;

    String expected = "Temperature: " + randomTemp;
    String actual = testWeather.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: The toString() Method (b)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the favBook object.
     * -----------------------------------------------------------------------------
     */

    // Creates a Book object
    Book favBook = new Book("The Giver");
    

    
  }
}`},{path:`Book.java`,text:`/*
 * Represents a book a student has read
 */
public class Book {

  private String title;    // The title of a book

  /*
   * Sets title to the specified title
   */
  public Book(String title) {
    this.title = title;
  }

  /*
   * Returns the value assigned to title
   */
  public String getTitle() {
    return title;
  }

  /*
   * Sets title to newTitle
   */
  public void setTitle(String newTitle) {
    title = newTitle;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a toString() method that returns a String containing the value assigned
   * to the title instance variable. For example, Title: "Adventures in Java Lab"
   * -----------------------------------------------------------------------------
   */




  

}`}],validationFiles:[{path:`BookTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Book.java Test")
public class BookTest {
  
  String messageGap = "\\n       ";
  Book testBook;
  String[] testBooks;
  String randomBook;

  @BeforeEach
  public void setup() {
    testBooks = new String[]{"Great Expectations", "Odyssey", "Emma", "Frankenstein", "Charlotte's Web"};
    int randomIndex = (int)(Math.random() * testBooks.length);
    randomBook = testBooks[randomIndex];
    testBook = new Book(randomBook);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Book object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the text \\"Title: \\" followed by the value assigned to the title instance variable." + messageGap;

    String expected = "Title: " + randomBook;
    String actual = testBook.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: The toString() Method (c)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the birthdayPlaylist object.
     * -----------------------------------------------------------------------------
     */

    // Creates a Playlist object
    Playlist birthdayPlaylist = new Playlist(28);
    
    
    
  }
}`},{path:`Playlist.java`,text:`/*
 * Represents a playlist used at an event
 */
public class Playlist {

  private int numSongs;    // The number of songs in a playlist

  /*
   * Sets numSongs to the specified number of songs
   */
  public Playlist(int numSongs) {
    this.numSongs = numSongs;
  }

  /*
   * Returns the value assigned to numSongs
   */
  public int getNumSongs() {
    return numSongs;
  }

  /*
   * Sets numSongs to newNumSongs
   */
  public void setNumSongs(int newNumSongs) {
    numSongs = newNumSongs;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a toString() method that returns a String containing the value assigned
   * to the numSongs instance variable. For example, Number of Songs: 15
   * -----------------------------------------------------------------------------
   */





  
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
  Playlist testPlaylist;
  int randomNumSongs;

  @BeforeEach
  public void setup() {
    randomNumSongs = (int)(Math.random() * (100 - 50)) + 50;
    testPlaylist = new Playlist(randomNumSongs);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Playlist object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the text \\"Number of Songs: \\" followed by the value assigned to the numSongs instance variable." + messageGap;

    String expected = "Number of Songs: " + randomNumSongs;
    String actual = testPlaylist.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Skill Building: The toString() Method (d)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Print the birthdayPlaylist object.
     * -----------------------------------------------------------------------------
     */

    // Creates a Player object
    Player tetrisChamp = new Player(682);
    
    
    
  }
}`},{path:`Player.java`,text:`/*
 * Represents a player in a game
 */
public class Player {

  private int highScore;   // A player's high score

  /*
   * Sets highScore to the specified high score
   */
  public Player(int highScore) {
    this.highScore = highScore;
  }

  /*
   * Returns the value assigned to highScore
   */
  public int getHighScore() {
    return highScore;
  }

  /*
   * Sets highScore to newHighScore
   */
  public void setHighScore(int newHighScore) {
    highScore = newHighScore;
  }

  /* ----------------------------------- TO DO -----------------------------------
   * ✅ Write a toString() method that returns a String containing the value assigned
   * to the highScore instance variable. For example, High Score: 572
   * -----------------------------------------------------------------------------
   */




  
  
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
  int randomHighScore;

  @BeforeEach
  public void setup() {
    randomHighScore = (int)(Math.random() * (1000 - 500)) + 500;
    testPlayer = new Player(randomHighScore);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Player object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the text \\"High Score: \\" followed by the value assigned to the highScore instance variable." + messageGap;

    String expected = "High Score: " + randomHighScore;
    String actual = testPlayer.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: The toString() Method`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate and print a Dessert object.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass then print the object.
     * -----------------------------------------------------------------------------
     */



    
  }
}`}],validationFiles:[{path:`DessertTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Dessert.java Test")
public class DessertTest {
  
  String messageGap = "\\n       ";
  Dessert testDessert;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testDessert = new Dessert(randomFlavor, randomPrice);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Dessert object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the text \\"Flavor: \\" followed by the value assigned to the flavor instance variable,";
    toStringMessage += "\\n        then the text \\"Price: \\" followed by the value assigned to the price instance variable on a new line.";
    toStringMessage += messageGap;

    String expected = "Flavor: " + randomFlavor + "\\nPrice: " + randomPrice;
    String actual = testDessert.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: The Food Truck (a)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate and print a Dessert object.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass then print the object.
     * -----------------------------------------------------------------------------
     */



    
  }
}`}],validationFiles:[{path:`CookieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cookie.java Test")
public class CookieTest {
  
  String messageGap = "\\n       ";
  Cookie testCookie;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testCookie = new Cookie(randomFlavor, randomPrice, true);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Cookie object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the String returned from the Dessert class toString() method,";
    toStringMessage += "\\n        then the text \\"Is Chewy? \\" followed by the value assigned to the isChewy instance variable on a new line.";
    toStringMessage += messageGap;

    String expected = "Flavor: " + randomFlavor + "\\nPrice: " + randomPrice + "\\nIs Chewy? " + testCookie.getIsChewy();
    String actual = testCookie.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: The Food Truck (b)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate and print a Dessert object.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass then print the object.
     * -----------------------------------------------------------------------------
     */



    
  }
}`}],validationFiles:[{path:`PieTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Pie.java Test")
public class PieTest {
  
  String messageGap = "\\n       ";
  Pie testPie;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;
  int randomDiameter;

  @BeforeEach
  public void setup() {
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;
    randomDiameter = (int)(Math.random() * (24 - 6)) + 6;

    testPie = new Pie(randomFlavor, randomPrice, randomDiameter);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Pie object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the String returned from the Dessert class toString() method,";
    toStringMessage += "\\n        then the text \\"Diameter: \\" followed by the value assigned to the diameter instance variable on a new line.";
    toStringMessage += messageGap;

    String expected = "Flavor: " + randomFlavor + "\\nPrice: " + randomPrice + "\\nDiameter: " + randomDiameter;
    String actual = testPie.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: The Food Truck (c)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate and print a Dessert object.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass then print the object.
     * -----------------------------------------------------------------------------
     */



    
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
  Donut testDonut;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testDonut = new Donut(randomFlavor, randomPrice, true);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Donut object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the String returned from the Dessert class toString() method,";
    toStringMessage += "\\n        then the text \\"Has Sprinkles? \\" followed by the value assigned to the hasSprinkles instance variable on a new line.";
    toStringMessage += messageGap;

    String expected = "Flavor: " + randomFlavor + "\\nPrice: " + randomPrice + "\\nHas Sprinkles? " + testDonut.getHasSprinkles();
    String actual = testDonut.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Practice: The Food Truck (d)`,lesson:`Lesson 12: Printing Objects`,view:`console`,grid:``,files:[{path:`Main.java`,text:`public class Main {
  public static void main(String[] args) {

    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 6 - Instantiate and print a Dessert object.
     * -----------------------------------------------------------------------------
     */



    /* ----------------------------------- TO DO -----------------------------------
     * ✅ Level 7 - Create an instance of the Dessert subclass then print the object.
     * -----------------------------------------------------------------------------
     */



    
  }
}`}],validationFiles:[{path:`CupcakeTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;

@TestMethodOrder(OrderAnnotation.class)
@DisplayName("Cupcake.java Test")
public class CupcakeTest {
  
  String messageGap = "\\n       ";
  Cupcake testCupcake;
  String[] testFlavors;
  String randomFlavor;
  double randomPrice;

  @BeforeEach
  public void setup() {
    testFlavors = new String[]{"chocolate", "vanilla", "strawberry", "oatmeal", "caramel"};

    int randomIndex = (int)(Math.random() * testFlavors.length);
    randomFlavor = testFlavors[randomIndex];
    randomPrice = (Math.random() * (20 - 2)) + 2;

    testCupcake = new Cupcake(randomFlavor, randomPrice, true);
  }

  @Test
  @Order(1)
  @DisplayName("Write a toString() method to return information about a Cupcake object => ")
  public void testToString() {
    String toStringMessage = "The toString() method should return a String containing the String returned from the Dessert class toString() method,";
    toStringMessage += "\\n        then the text \\"Is Mini? \\" followed by the value assigned to the isMini instance variable on a new line.";
    toStringMessage += messageGap;

    String expected = "Flavor: " + randomFlavor + "\\nPrice: " + randomPrice + "\\nIs Mini? " + testCupcake.getIsMini();
    String actual = testCupcake.toString();

    assertEquals(expected, actual, toStringMessage);
  }
  
}`}],dataFiles:[]},{name:`Store Management Project`,lesson:`Lesson 13a: Store Management Project`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object - feel free to delete if not using!
    Scanner input = new Scanner(System.in);






    // Closes the Scanner object
    input.close();
    
  }
}`}],validationFiles:[],dataFiles:[]},{name:`Store Management Project`,lesson:`Lesson 13b: Store Management Project [1-Day Version]`,view:`console`,grid:``,files:[{path:`Main.java`,text:`import java.util.Scanner;

public class Main {
  public static void main(String[] args) {

    // Creates a Scanner object - feel free to delete if not using!
    Scanner input = new Scanner(System.in);





    
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
}`}],validationFiles:[],dataFiles:[]}];export{e as LEVELS};