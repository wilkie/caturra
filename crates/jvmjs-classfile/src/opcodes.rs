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
pub const RETURN: u8 = 0xB1;
pub const GETSTATIC: u8 = 0xB2;
pub const INVOKEVIRTUAL: u8 = 0xB6;
