import java.io.FileOutputStream;
import org.code.media.SoundLoader;

/** Decode each named sound with the REAL org.code.media SoundLoader, and write its
 *  samples in caturra's VFS packing (raw little-endian signed 16-bit PCM), so both
 *  engines clip and compare the SAME samples and an audio decode is never what is
 *  being compared. The image half of this is PackAssets. */
public class PackSounds {
  public static void main(String[] args) throws Exception {
    for (String name : args) {
      double[] samples = SoundLoader.read(name);
      byte[] out = new byte[samples.length * 2];
      for (int i = 0; i < samples.length; i++) {
        double clamped = Math.max(-1.0, Math.min(1.0, samples[i]));
        short quantized = (short) Math.round(clamped * 32767);
        out[i * 2] = (byte) (quantized & 0xff);
        out[i * 2 + 1] = (byte) ((quantized >> 8) & 0xff);
      }
      FileOutputStream file = new FileOutputStream("__packed_sound_" + name);
      file.write(out);
      file.close();
    }
  }
}
