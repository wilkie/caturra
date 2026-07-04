import java.util.Random;

public class FloatCorpus {
    public static void main(String[] args) {
        // Structured cases: powers of two, subnormals, boundaries.
        int[] specials = {
            0x00000001, 0x00000002, 0x00000010, 0x007FFFFF, // subnormals
            0x00800000, // MIN_NORMAL
            0x7F7FFFFF, // MAX_VALUE
            0x3F800000, 0x40000000, 0x54800000, // 1, 2, 2^42
        };
        for (int bits : specials) {
            emit(bits);
        }
        for (int e = -149; e <= 127; e++) {
            emit(Float.floatToRawIntBits((float) Math.pow(2, e)));
        }
        for (int i = 1; i <= 1000; i++) {
            emit(Float.floatToRawIntBits((float) i));
            emit(Float.floatToRawIntBits(i * 0.1f));
            emit(Float.floatToRawIntBits(i * 1e6f));
            emit(Float.floatToRawIntBits(1.0f / i));
        }
        // Trailing-zero mantissas (the non-shortest cases).
        for (int e = 0; e <= 120; e += 3) {
            for (int m = 0; m <= 23; m++) {
                int bits = ((e + 127) << 23) | (1 << m);
                emit(bits);
            }
        }
        Random random = new Random(42);
        for (int i = 0; i < 20000; i++) {
            emit(random.nextInt());
        }
    }

    static void emit(int bits) {
        float value = Float.intBitsToFloat(bits);
        if (Float.isNaN(value) || Float.isInfinite(value)) return;
        System.out.println(Integer.toHexString(bits) + " " + Float.toString(value));
    }
}
