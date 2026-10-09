var e=[{name:`Part A — 1 test(s)`,lesson:`Box Of Candy (2023)`,view:`console`,grid:``,files:[{path:`BoxOfCandy.java`,text:`public class BoxOfCandy 
{ 
  /** box contains at least one row and is initialized in the constructor. */ 
  private Candy[][] box; 
  
  /** 
    * Moves one piece of candy in column col, if necessary and possible, so that the box 
    * element in row 0 of column col contains a piece of candy, as described in part (a). 
    * Returns false if there is no piece of candy in column col and returns true otherwise. 
    * Precondition: col is a valid column index in box. 
    */ 
  public boolean moveCandyToFirstRow(int col) 
  { /* to be implemented in part (a) */ } 
  
  /** 
    * Removes from box and returns a piece of candy with flavor specified by the parameter, or 
    * returns null if no such piece is found, as described in part (b) 
    */ 
  public Candy removeNextByFlavor(String flavor) 
  { /* to be implemented in part (b) */ } 
  
  // There may be instance variables, constructors, and methods that are not shown. 
}
`},{path:`Candy.java`,text:`public class Candy
{
    private String flavor;

    /** Constructor initializes Candy object */
    public Candy(String flavor)
    {
        this.flavor = flavor;
    }

    /** Returns a String representing the flavor of this piece of candy */ 
    public String getFlavor()
    {
        return flavor;
    }
}`}],validationFiles:[{path:`BoxOfCandyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;


@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BoxOfCandy Test")
public class BoxOfCandyTest
{

  
    private static Candy[][] getShallowCopy(Candy[][] box)
    {
        Candy[][] shallowCopy = new Candy[box.length][box[0].length];
        
        for(int r = 0; r < shallowCopy.length; r++)
            for(int c = 0; c < shallowCopy[0].length; c++)
                shallowCopy[r][c] = box[r][c];
        
        return shallowCopy;
    }
    
    // references must be to exact same objects
    public static boolean equals(Candy[][] one, Candy[][] two)
    {
        if(one.length != two.length || one[0].length != two[0].length)
            return false;
        
        for(int r = 0; r < one.length; r++)
            for(int c = 0; c < one[0].length; c++)
                if(one[r][c] != two[r][c])
                    return false;
        
        return true;
    }
   
    @Test
    @Order(1)
    @DisplayName("Test 1: MoveCandyToFirstRow")
    public void testMoveCandyToFirstRow()
    {
        BoxOfCandy boc = new BoxOfCandy();
        boc.box = new Candy[4][3];
        boc.box[0][1] = new Candy("lime");
        boc.box[1][1] = new Candy("orange");
        boc.box[2][2] = new Candy("cherry");
        boc.box[3][1] = new Candy("lemon");
        boc.box[3][2] = new Candy("grape");
        
        Candy[][] expectedResult = getShallowCopy(boc.box);
        
        assertTrue( ! boc.moveCandyToFirstRow(0) );
        assertTrue(equals(expectedResult, boc.box));
        
        assertTrue(boc.moveCandyToFirstRow(1));
        assertTrue(equals(expectedResult, boc.box));
        
        Candy[][] expectedResultAlt = getShallowCopy(expectedResult);
        
        expectedResult[0][2] = expectedResult[2][2];
        expectedResult[2][2] = null;
        
        expectedResultAlt[0][2] = expectedResultAlt[3][2];
        expectedResultAlt[3][2] = null;
        
        assertTrue(boc.moveCandyToFirstRow(2));
        assertTrue(
                equals(expectedResult, boc.box) ||
                equals(expectedResultAlt, boc.box));
    }

}`}],dataFiles:[]},{name:`Part B — 1 test(s)`,lesson:`Box Of Candy (2023)`,view:`console`,grid:``,files:[{path:`BoxOfCandy.java`,text:`public class BoxOfCandy 
{ 
  /** box contains at least one row and is initialized in the constructor. */ 
  private Candy[][] box; 
  
  /** 
    * Moves one piece of candy in column col, if necessary and possible, so that the box 
    * element in row 0 of column col contains a piece of candy, as described in part (a). 
    * Returns false if there is no piece of candy in column col and returns true otherwise. 
    * Precondition: col is a valid column index in box. 
    */ 
  public boolean moveCandyToFirstRow(int col) 
  { /* to be implemented in part (a) */ } 
  
  /** 
    * Removes from box and returns a piece of candy with flavor specified by the parameter, or 
    * returns null if no such piece is found, as described in part (b) 
    */ 
  public Candy removeNextByFlavor(String flavor) 
  { /* to be implemented in part (b) */ } 
  
  // There may be instance variables, constructors, and methods that are not shown. 
}
`},{path:`Candy.java`,text:`public class Candy
{
    private String flavor;

    /** Constructor initializes Candy object */
    public Candy(String flavor)
    {
        this.flavor = flavor;
    }

    /** Returns a String representing the flavor of this piece of candy */ 
    public String getFlavor()
    {
        return flavor;
    }
}`}],validationFiles:[{path:`BoxOfCandyTest.java`,text:`import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestMethodOrder;
import org.junit.jupiter.api.MethodOrderer.OrderAnnotation;
import org.junit.jupiter.api.Order;


@TestMethodOrder(OrderAnnotation.class)
@DisplayName("BoxOfCandy Test")
public class BoxOfCandyTest
{

  
    private static Candy[][] getShallowCopy(Candy[][] box)
    {
        Candy[][] shallowCopy = new Candy[box.length][box[0].length];
        
        for(int r = 0; r < shallowCopy.length; r++)
            for(int c = 0; c < shallowCopy[0].length; c++)
                shallowCopy[r][c] = box[r][c];
        
        return shallowCopy;
    }
    
    // references must be to exact same objects
    public static boolean equals(Candy[][] one, Candy[][] two)
    {
        if(one.length != two.length || one[0].length != two[0].length)
            return false;
        
        for(int r = 0; r < one.length; r++)
            for(int c = 0; c < one[0].length; c++)
                if(one[r][c] != two[r][c])
                    return false;
        
        return true;
    }
   
    @Test
    @Order(1)
    @DisplayName("Test 1: MoveCandyToFirstRow")
    public void testMoveCandyToFirstRow()
    {
        BoxOfCandy boc = new BoxOfCandy();
        boc.box = new Candy[4][3];
        boc.box[0][1] = new Candy("lime");
        boc.box[1][1] = new Candy("orange");
        boc.box[2][2] = new Candy("cherry");
        boc.box[3][1] = new Candy("lemon");
        boc.box[3][2] = new Candy("grape");
        
        Candy[][] expectedResult = getShallowCopy(boc.box);
        
        assertTrue( ! boc.moveCandyToFirstRow(0) );
        assertTrue(equals(expectedResult, boc.box));
        
        assertTrue(boc.moveCandyToFirstRow(1));
        assertTrue(equals(expectedResult, boc.box));
        
        Candy[][] expectedResultAlt = getShallowCopy(expectedResult);
        
        expectedResult[0][2] = expectedResult[2][2];
        expectedResult[2][2] = null;
        
        expectedResultAlt[0][2] = expectedResultAlt[3][2];
        expectedResultAlt[3][2] = null;
        
        assertTrue(boc.moveCandyToFirstRow(2));
        assertTrue(
                equals(expectedResult, boc.box) ||
                equals(expectedResultAlt, boc.box));
    }

}`}],dataFiles:[]}];export{e as LEVELS};