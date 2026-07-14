package org.code.theater;

import java.io.FileOutputStream;
import java.io.IOException;
import org.code.media.Color;
import org.code.media.Font;
import org.code.media.FontStyle;
import org.code.media.Image;
import org.code.media.Pixel;

/**
 * NOT the real Scene: a stand-in with the real Scene's exact public signatures,
 * so a corpus level compiles and runs unchanged, and so `drawImage` can dump
 * the pixels the program actually produced. The real Scene draws into a
 * BufferedImage and encodes a GIF; caturra deliberately renders in the browser
 * instead, so there is no common pixel surface to compare there. What IS being
 * compared through this stub is the student's filter code running against the
 * REAL org.code.media Image/Pixel/Color — the pixel maths, which is the whole
 * of what caturra models.
 */
public class Scene {
  private static int drawn = 0;

  public Scene() {}

  public final int getWidth() { return 400; }

  public final int getHeight() { return 400; }

  public final void clear(String color) { new Color(color); }

  public final void clear(Color color) {}

  public final void playSound(double[] sound) {}

  public final void playSound(String filename) {}

  public final void playNote(int note, double seconds) {}

  public final void playNoteAndPause(int note, double seconds) {}

  public final void playNote(Instrument instrument, int note, double seconds) {}

  public final void playNoteAndPause(Instrument instrument, int note, double seconds) {}

  public final void pause(double seconds) {}

  /** Every drawn image is dumped: width, height, then one RGB triple per pixel. */
  private void dump(Image image) {
    try {
      int w = image.getWidth();
      int h = image.getHeight();
      byte[] out = new byte[8 + w * h * 3];
      out[0] = (byte) (w & 0xff); out[1] = (byte) ((w >> 8) & 0xff);
      out[2] = (byte) ((w >> 16) & 0xff); out[3] = (byte) ((w >> 24) & 0xff);
      out[4] = (byte) (h & 0xff); out[5] = (byte) ((h >> 8) & 0xff);
      out[6] = (byte) ((h >> 16) & 0xff); out[7] = (byte) ((h >> 24) & 0xff);
      int at = 8;
      for (int y = 0; y < h; y++) {
        for (int x = 0; x < w; x++) {
          Pixel p = image.getPixel(x, y);
          out[at++] = (byte) p.getRed();
          out[at++] = (byte) p.getGreen();
          out[at++] = (byte) p.getBlue();
        }
      }
      FileOutputStream f = new FileOutputStream("__ref_img_" + drawn);
      f.write(out);
      f.close();
      drawn++;
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
  }

  public final void drawImage(Image image, int x, int y, int size) { dump(image); }

  public final void drawImage(Image image, int x, int y, int size, double rotation) { dump(image); }

  public final void drawImage(Image image, int x, int y, int width, int height, double rotation) { dump(image); }

  public final void drawImage(String filename, int x, int y, int size) { new Image(filename); }

  public final void drawImage(String filename, int x, int y, int size, double rotation) { new Image(filename); }

  public final void drawImage(String filename, int x, int y, int width, int height, double rotation) { new Image(filename); }

  public final void setTextStyle(Font font, FontStyle style) {}

  public final void setTextHeight(int height) {}

  public final void setTextColor(String color) { new Color(color); }

  public final void setTextColor(Color color) {}

  public final void drawText(String text, int x, int y) {}

  public final void drawText(String text, int x, int y, double rotation) {}

  public final void drawLine(int startX, int startY, int endX, int endY) {}

  public final void drawRegularPolygon(int x, int y, int sides, int radius) {}

  public final void drawShape(int[] points, boolean close) {}

  public final void drawEllipse(int x, int y, int width, int height) {}

  public final void drawRectangle(int x, int y, int width, int height) {}

  public final void setStrokeWidth(double width) {}

  public final void setFillColor(Color color) {}

  public final void setStrokeColor(Color color) {}

  public final void setFillColor(String color) { new Color(color); }

  public final void setStrokeColor(String color) { new Color(color); }

  public final void removeStrokeColor() {}

  public final void removeFillColor() {}
}
