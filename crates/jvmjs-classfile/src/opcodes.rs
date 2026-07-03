//! JVM bytecode opcode constants (JVMS §6.5).
//!
//! This is the shared contract between the compiler (which emits these)
//! and the VM (which interprets them). Only opcodes the current pipeline
//! uses are listed; the module grows with the language surface
//! (see `specs/LANGUAGE.md`).

pub const NOP: u8 = 0x00;
pub const ACONST_NULL: u8 = 0x01;
pub const ICONST_M1: u8 = 0x02;
pub const ICONST_0: u8 = 0x03;
pub const ICONST_1: u8 = 0x04;
pub const ICONST_2: u8 = 0x05;
pub const ICONST_3: u8 = 0x06;
pub const ICONST_4: u8 = 0x07;
pub const ICONST_5: u8 = 0x08;
pub const BIPUSH: u8 = 0x10;
pub const SIPUSH: u8 = 0x11;
pub const LDC: u8 = 0x12;
pub const LDC_W: u8 = 0x13;
pub const LDC2_W: u8 = 0x14;

// Local variable loads (with 0-3 short forms).
pub const ILOAD: u8 = 0x15;
pub const DLOAD: u8 = 0x18;
pub const ALOAD: u8 = 0x19;
pub const ILOAD_0: u8 = 0x1A;
pub const ILOAD_3: u8 = 0x1D;
pub const DLOAD_0: u8 = 0x26;
pub const DLOAD_3: u8 = 0x29;
pub const ALOAD_0: u8 = 0x2A;
pub const ALOAD_3: u8 = 0x2D;

// Array element loads.
pub const IALOAD: u8 = 0x2E;
pub const DALOAD: u8 = 0x31;
pub const AALOAD: u8 = 0x32;
pub const BALOAD: u8 = 0x33;
pub const CALOAD: u8 = 0x34;

// Local variable stores (with 0-3 short forms).
pub const ISTORE: u8 = 0x36;
pub const DSTORE: u8 = 0x39;
pub const ASTORE: u8 = 0x3A;
pub const ISTORE_0: u8 = 0x3B;
pub const ISTORE_3: u8 = 0x3E;
pub const DSTORE_0: u8 = 0x47;
pub const DSTORE_3: u8 = 0x4A;
pub const ASTORE_0: u8 = 0x4B;
pub const ASTORE_3: u8 = 0x4E;

// Array element stores.
pub const IASTORE: u8 = 0x4F;
pub const DASTORE: u8 = 0x52;
pub const AASTORE: u8 = 0x53;
pub const BASTORE: u8 = 0x54;
pub const CASTORE: u8 = 0x55;

// Stack manipulation.
pub const POP: u8 = 0x57;
/// Pops one category-2 value (double/long) or two category-1 values.
pub const POP2: u8 = 0x58;
pub const DUP: u8 = 0x59;
/// Duplicates one category-2 value or the top two category-1 values.
pub const DUP2: u8 = 0x5C;
pub const SWAP: u8 = 0x5F;

// Arithmetic.
pub const IADD: u8 = 0x60;
pub const DADD: u8 = 0x63;
pub const ISUB: u8 = 0x64;
pub const DSUB: u8 = 0x67;
pub const IMUL: u8 = 0x68;
pub const DMUL: u8 = 0x6B;
pub const IDIV: u8 = 0x6C;
pub const DDIV: u8 = 0x6F;
pub const IREM: u8 = 0x70;
pub const DREM: u8 = 0x73;
pub const INEG: u8 = 0x74;
pub const DNEG: u8 = 0x77;

// Numeric conversions.
pub const I2D: u8 = 0x87;
pub const D2I: u8 = 0x8E;
pub const I2C: u8 = 0x92;

// Comparisons.
pub const DCMPL: u8 = 0x97;
pub const DCMPG: u8 = 0x98;

// Branches. IFEQ..GOTO is a contiguous range used for scanning.
pub const IFEQ: u8 = 0x99;
pub const IFNE: u8 = 0x9A;
pub const IFLT: u8 = 0x9B;
pub const IFGE: u8 = 0x9C;
pub const IFGT: u8 = 0x9D;
pub const IFLE: u8 = 0x9E;
pub const IF_ICMPEQ: u8 = 0x9F;
pub const IF_ICMPNE: u8 = 0xA0;
pub const IF_ICMPLT: u8 = 0xA1;
pub const IF_ICMPGE: u8 = 0xA2;
pub const IF_ICMPGT: u8 = 0xA3;
pub const IF_ICMPLE: u8 = 0xA4;
pub const IF_ACMPEQ: u8 = 0xA5;
pub const IF_ACMPNE: u8 = 0xA6;
pub const GOTO: u8 = 0xA7;

// Returns.
pub const IRETURN: u8 = 0xAC;
pub const DRETURN: u8 = 0xAF;
pub const ARETURN: u8 = 0xB0;
pub const RETURN: u8 = 0xB1;

pub const GETSTATIC: u8 = 0xB2;
pub const PUTSTATIC: u8 = 0xB3;
pub const GETFIELD: u8 = 0xB4;
pub const PUTFIELD: u8 = 0xB5;
pub const INVOKEVIRTUAL: u8 = 0xB6;
pub const INVOKESPECIAL: u8 = 0xB7;
pub const INVOKESTATIC: u8 = 0xB8;
pub const NEW: u8 = 0xBB;

// Null-reference branches.
pub const IFNULL: u8 = 0xC6;
pub const IFNONNULL: u8 = 0xC7;

// Arrays.
pub const NEWARRAY: u8 = 0xBC;
pub const ANEWARRAY: u8 = 0xBD;
pub const ARRAYLENGTH: u8 = 0xBE;
pub const MULTIANEWARRAY: u8 = 0xC5;

// `newarray` element type codes (JVMS Table 6.5.newarray-A).
pub const T_BOOLEAN: u8 = 4;
pub const T_CHAR: u8 = 5;
pub const T_DOUBLE: u8 = 7;
pub const T_INT: u8 = 10;
