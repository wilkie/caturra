import java.io.FileOutputStream;
import org.code.media.Image;
import org.code.media.Pixel;

/** Decode each named asset with the REAL org.code.media Image(String), and write
 *  its pixels in caturra's VFS packing, so both engines start from the same
 *  bytes and the decode (and the library's downscale-to-400) is not what is
 *  being compared. */
public class PackAssets {
  public static void main(String[] args) throws Exception {
    for (String name : args) {
      Image img = new Image(name);
      int w = img.getWidth(), h = img.getHeight();
      byte[] out = new byte[8 + w * h * 3];
      out[0] = (byte) (w & 0xff); out[1] = (byte) ((w >> 8) & 0xff);
      out[2] = (byte) ((w >> 16) & 0xff); out[3] = (byte) ((w >> 24) & 0xff);
      out[4] = (byte) (h & 0xff); out[5] = (byte) ((h >> 8) & 0xff);
      out[6] = (byte) ((h >> 16) & 0xff); out[7] = (byte) ((h >> 24) & 0xff);
      int at = 8;
      for (int y = 0; y < h; y++) {
        for (int x = 0; x < w; x++) {
          Pixel p = img.getPixel(x, y);
          out[at++] = (byte) p.getRed();
          out[at++] = (byte) p.getGreen();
          out[at++] = (byte) p.getBlue();
        }
      }
      FileOutputStream f = new FileOutputStream("__packed_" + name);
      f.write(out);
      f.close();
      System.out.println(name + " " + w + "x" + h);
    }
  }
}
