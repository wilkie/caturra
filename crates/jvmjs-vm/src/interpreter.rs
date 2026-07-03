//! The bytecode dispatch loop (JVMS §2.5–2.6, §6.5).
//!
//! A classic frame interpreter: locals + operand stack per method,
//! decoding one opcode at a time. The opcode surface grows in lockstep
//! with the compiler (`specs/LANGUAGE.md`); an opcode the VM doesn't
//! know yet is a `VmError`, never undefined behavior.

use std::collections::HashMap;

use jvmjs_classfile::{
    CODE_ATTRIBUTE, ClassFile, Constant, MethodInfo, opcodes as op, read_code_attribute,
};

use crate::intrinsics::{self, IntrinsicStatics};
use crate::io::ConsoleIo;
use crate::value::{Heap, HeapRef, JValue};
use crate::vm::VmError;

/// State for one `run`: the heap, interned strings, intrinsic
/// singletons, and the instruction budget.
///
/// (User-method calls — `invokestatic` and friends — will add a
/// reference to the loaded-classes map here; the current opcode surface
/// only calls intrinsics.)
pub(crate) struct Interpreter<'run> {
    pub console: &'run mut dyn ConsoleIo,
    pub heap: Heap,
    pub statics: IntrinsicStatics,
    string_pool: HashMap<String, HeapRef>,
    remaining_instructions: u64,
}

impl<'run> Interpreter<'run> {
    pub fn new(console: &'run mut dyn ConsoleIo, max_instructions: u64) -> Self {
        Self {
            console,
            heap: Heap::new(),
            statics: IntrinsicStatics::default(),
            string_pool: HashMap::new(),
            remaining_instructions: max_instructions,
        }
    }

    /// Allocate (or reuse) the interned string for `text`, per JLS
    /// string-literal semantics.
    pub fn intern_string(&mut self, text: &str) -> HeapRef {
        if let Some(existing) = self.string_pool.get(text) {
            return *existing;
        }
        let reference = self.heap.alloc_string(text);
        self.string_pool.insert(text.to_owned(), reference);
        reference
    }

    /// Execute one method to completion (no user-method calls yet, so
    /// there is exactly one frame).
    ///
    /// The dispatch loop is one long match by design; splitting it per
    /// opcode family would only scatter the instruction set.
    #[allow(clippy::too_many_lines)]
    pub fn execute(
        &mut self,
        class: &ClassFile,
        method: &MethodInfo,
        mut locals: Vec<JValue>,
    ) -> Result<(), VmError> {
        let class_name = class.class_name().unwrap_or("<unknown>").to_owned();
        let malformed = |reason: String| VmError::MalformedClass {
            name: class_name.clone(),
            reason,
        };

        let code_info = method
            .attributes
            .iter()
            .find(|a| class.constant_pool.get_utf8(a.name_index) == Some(CODE_ATTRIBUTE))
            .ok_or_else(|| malformed(String::from("method has no Code attribute")))?;
        let code = read_code_attribute(&code_info.info)
            .map_err(|e| malformed(format!("bad Code attribute: {e}")))?;

        if locals.len() < usize::from(code.max_locals) {
            locals.resize(usize::from(code.max_locals), JValue::Int(0));
        }
        let mut frame = Frame {
            locals,
            stack: Vec::new(),
        };
        let bytes = &code.code;
        let mut pc = 0usize;

        loop {
            if self.remaining_instructions == 0 {
                return Err(VmError::InstructionBudgetExceeded);
            }
            self.remaining_instructions -= 1;

            let addr = pc;
            let opcode = *bytes.get(addr).ok_or_else(|| {
                malformed(format!(
                    "execution ran off the end of bytecode at pc {addr}"
                ))
            })?;
            pc = addr + 1;

            match opcode {
                op::NOP => {}
                op::ACONST_NULL => frame.stack.push(JValue::NULL),
                op::ICONST_M1..=op::ICONST_5 => {
                    let value = i32::from(opcode) - i32::from(op::ICONST_0);
                    frame.stack.push(JValue::Int(value));
                }
                op::BIPUSH => {
                    let byte = read_u8(bytes, &mut pc, &malformed)?;
                    frame.stack.push(JValue::Int(i32::from(byte.cast_signed())));
                }
                op::SIPUSH => {
                    let short = read_u16(bytes, &mut pc, &malformed)?;
                    frame
                        .stack
                        .push(JValue::Int(i32::from(short.cast_signed())));
                }
                op::LDC => {
                    let index = u16::from(read_u8(bytes, &mut pc, &malformed)?);
                    let value = self.load_constant(class, index, &malformed)?;
                    frame.stack.push(value);
                }
                op::LDC_W | op::LDC2_W => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    let value = self.load_constant(class, index, &malformed)?;
                    frame.stack.push(value);
                }

                // ----- locals -----
                op::ILOAD | op::DLOAD | op::ALOAD => {
                    let slot = usize::from(read_u8(bytes, &mut pc, &malformed)?);
                    frame.load(slot, &malformed)?;
                }
                op::ILOAD_0..=op::ILOAD_3 => {
                    frame.load(usize::from(opcode - op::ILOAD_0), &malformed)?;
                }
                op::DLOAD_0..=op::DLOAD_3 => {
                    frame.load(usize::from(opcode - op::DLOAD_0), &malformed)?;
                }
                op::ALOAD_0..=op::ALOAD_3 => {
                    frame.load(usize::from(opcode - op::ALOAD_0), &malformed)?;
                }
                op::ISTORE | op::DSTORE | op::ASTORE => {
                    let slot = usize::from(read_u8(bytes, &mut pc, &malformed)?);
                    frame.store(slot, &malformed)?;
                }
                op::ISTORE_0..=op::ISTORE_3 => {
                    frame.store(usize::from(opcode - op::ISTORE_0), &malformed)?;
                }
                op::DSTORE_0..=op::DSTORE_3 => {
                    frame.store(usize::from(opcode - op::DSTORE_0), &malformed)?;
                }
                op::ASTORE_0..=op::ASTORE_3 => {
                    frame.store(usize::from(opcode - op::ASTORE_0), &malformed)?;
                }

                // ----- stack manipulation -----
                op::POP => {
                    frame.pop()?;
                }
                op::DUP => {
                    let top = *frame.stack.last().ok_or(VmError::StackUnderflow)?;
                    frame.stack.push(top);
                }
                op::SWAP => {
                    let len = frame.stack.len();
                    if len < 2 {
                        return Err(VmError::StackUnderflow);
                    }
                    frame.stack.swap(len - 1, len - 2);
                }

                // ----- arithmetic (Java wrapping semantics) -----
                op::IADD => frame.int_binop(i32::wrapping_add)?,
                op::ISUB => frame.int_binop(i32::wrapping_sub)?,
                op::IMUL => frame.int_binop(i32::wrapping_mul)?,
                op::IDIV => frame.int_division(i32::wrapping_div)?,
                op::IREM => frame.int_division(i32::wrapping_rem)?,
                op::INEG => {
                    let value = frame.pop_int()?;
                    frame.stack.push(JValue::Int(value.wrapping_neg()));
                }
                op::DADD => frame.double_binop(|a, b| a + b)?,
                op::DSUB => frame.double_binop(|a, b| a - b)?,
                op::DMUL => frame.double_binop(|a, b| a * b)?,
                op::DDIV => frame.double_binop(|a, b| a / b)?,
                op::DREM => frame.double_binop(|a, b| a % b)?,
                op::DNEG => {
                    let value = frame.pop_double()?;
                    frame.stack.push(JValue::Double(-value));
                }

                // ----- conversions -----
                op::I2D => {
                    let value = frame.pop_int()?;
                    frame.stack.push(JValue::Double(f64::from(value)));
                }
                op::D2I => {
                    let value = frame.pop_double()?;
                    // `as` matches JVM d2i: NaN -> 0, saturating bounds.
                    frame.stack.push(JValue::Int(value as i32));
                }
                op::I2C => {
                    let value = frame.pop_int()?;
                    // i2c truncates to 16 bits and zero-extends.
                    let truncated = (value.cast_unsigned() & 0xFFFF).cast_signed();
                    frame.stack.push(JValue::Int(truncated));
                }

                // ----- comparisons and branches -----
                op::DCMPL | op::DCMPG => {
                    let b = frame.pop_double()?;
                    let a = frame.pop_double()?;
                    let result = if a.is_nan() || b.is_nan() {
                        if opcode == op::DCMPG { 1 } else { -1 }
                    } else if a < b {
                        -1
                    } else {
                        i32::from(a > b)
                    };
                    frame.stack.push(JValue::Int(result));
                }
                op::IFEQ..=op::IFLE => {
                    let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                    let value = frame.pop_int()?;
                    let jump = match opcode {
                        op::IFEQ => value == 0,
                        op::IFNE => value != 0,
                        op::IFLT => value < 0,
                        op::IFGE => value >= 0,
                        op::IFGT => value > 0,
                        _ => value <= 0,
                    };
                    if jump {
                        pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                    }
                }
                op::IF_ICMPEQ..=op::IF_ICMPLE => {
                    let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                    let b = frame.pop_int()?;
                    let a = frame.pop_int()?;
                    let jump = match opcode {
                        op::IF_ICMPEQ => a == b,
                        op::IF_ICMPNE => a != b,
                        op::IF_ICMPLT => a < b,
                        op::IF_ICMPGE => a >= b,
                        op::IF_ICMPGT => a > b,
                        _ => a <= b,
                    };
                    if jump {
                        pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                    }
                }
                op::IF_ACMPEQ | op::IF_ACMPNE => {
                    let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                    let b = frame.pop_ref()?;
                    let a = frame.pop_ref()?;
                    let jump = (a == b) == (opcode == op::IF_ACMPEQ);
                    if jump {
                        pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                    }
                }
                op::GOTO => {
                    let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                    pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                }

                // ----- objects -----
                op::NEW => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    let target = class
                        .constant_pool
                        .get_class_name(index)
                        .ok_or_else(|| malformed(format!("bad class ref at pool {index}")))?;
                    let object = intrinsics::instantiate(target).ok_or_else(|| {
                        VmError::UnknownIntrinsic(format!("cannot instantiate {target}"))
                    })?;
                    let reference = self.heap.alloc(object);
                    frame.stack.push(JValue::Ref(Some(reference)));
                }
                op::INVOKESPECIAL => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    let (target_class, method_name, descriptor) = class
                        .constant_pool
                        .get_member_ref(index)
                        .map(|(c, m, d)| (c.to_owned(), m.to_owned(), d.to_owned()))
                        .ok_or_else(|| malformed(format!("bad method ref at pool {index}")))?;
                    let arg_count = descriptor_arg_count(&descriptor)
                        .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
                    for _ in 0..arg_count {
                        frame.pop()?;
                    }
                    let Some(receiver) = frame.pop_ref()? else {
                        return Err(VmError::UncaughtException(String::from(
                            "java.lang.NullPointerException",
                        )));
                    };
                    intrinsics::invoke_special(
                        &self.heap,
                        receiver,
                        &target_class,
                        &method_name,
                        &descriptor,
                    )?;
                }
                op::GETSTATIC => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    let (field_class, field_name, _descriptor) = class
                        .constant_pool
                        .get_member_ref(index)
                        .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
                    let value = self
                        .statics
                        .static_field(&mut self.heap, field_class, field_name)
                        .ok_or_else(|| {
                            VmError::UnknownIntrinsic(format!("{field_class}.{field_name}"))
                        })?;
                    frame.stack.push(value);
                }
                op::INVOKEVIRTUAL => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.invoke_virtual_op(class, &mut frame, index, &malformed)?;
                }
                op::RETURN => return Ok(()),
                other => return Err(VmError::UnsupportedOpcode(other)),
            }
        }
    }

    /// `invokevirtual`: resolve the method ref, pop arguments and
    /// receiver, and dispatch (v0: intrinsics only).
    fn invoke_virtual_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (target_class, method_name, descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, m, d)| (c.to_owned(), m.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad method ref at pool {index}")))?;

        let arg_count = descriptor_arg_count(&descriptor)
            .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(frame.pop()?);
        }
        args.reverse();

        let JValue::Ref(receiver) = frame.pop()? else {
            return Err(malformed(String::from(
                "invokevirtual receiver is not a reference",
            )));
        };
        let Some(receiver) = receiver else {
            return Err(VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot invoke \
                 \"{target_class}.{method_name}\" because the receiver is null"
            )));
        };

        let result = intrinsics::invoke_virtual(
            &mut self.heap,
            self.console,
            receiver,
            &target_class,
            &method_name,
            &descriptor,
            &args,
        )?;
        if let Some(value) = result {
            frame.stack.push(value);
        }
        Ok(())
    }

    /// Materialize a loadable constant-pool entry as a value
    /// (JVMS §5.1: `ldc` family).
    fn load_constant(
        &mut self,
        class: &ClassFile,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<JValue, VmError> {
        let constant = class
            .constant_pool
            .get(index)
            .ok_or_else(|| malformed(format!("bad constant pool index {index}")))?;
        match constant {
            Constant::Integer(v) => Ok(JValue::Int(*v)),
            Constant::Float(v) => Ok(JValue::Float(*v)),
            Constant::Long(v) => Ok(JValue::Long(*v)),
            Constant::Double(v) => Ok(JValue::Double(*v)),
            Constant::String { string_index } => {
                let text = class
                    .constant_pool
                    .get_utf8(*string_index)
                    .ok_or_else(|| malformed(format!("bad string constant at {index}")))?
                    .to_owned();
                Ok(JValue::Ref(Some(self.intern_string(&text))))
            }
            other => Err(malformed(format!(
                "cannot load constant {other:?} with ldc"
            ))),
        }
    }
}

struct Frame {
    locals: Vec<JValue>,
    stack: Vec<JValue>,
}

impl Frame {
    fn pop(&mut self) -> Result<JValue, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn pop_int(&mut self) -> Result<i32, VmError> {
        match self.pop()? {
            JValue::Int(value) => Ok(value),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected an int on the stack, found {other:?}"
            ))),
        }
    }

    fn pop_double(&mut self) -> Result<f64, VmError> {
        match self.pop()? {
            JValue::Double(value) => Ok(value),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected a double on the stack, found {other:?}"
            ))),
        }
    }

    fn pop_ref(&mut self) -> Result<Option<HeapRef>, VmError> {
        match self.pop()? {
            JValue::Ref(reference) => Ok(reference),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected a reference on the stack, found {other:?}"
            ))),
        }
    }

    fn load(&mut self, slot: usize, malformed: &impl Fn(String) -> VmError) -> Result<(), VmError> {
        let value = *self
            .locals
            .get(slot)
            .ok_or_else(|| malformed(format!("load from out-of-range local slot {slot}")))?;
        self.stack.push(value);
        Ok(())
    }

    fn store(
        &mut self,
        slot: usize,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let value = self.pop()?;
        let target = self
            .locals
            .get_mut(slot)
            .ok_or_else(|| malformed(format!("store to out-of-range local slot {slot}")))?;
        *target = value;
        Ok(())
    }

    fn int_binop(&mut self, apply: impl Fn(i32, i32) -> i32) -> Result<(), VmError> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(JValue::Int(apply(a, b)));
        Ok(())
    }

    /// `idiv`/`irem`: like [`Self::int_binop`] plus Java's division-by-zero
    /// exception.
    fn int_division(&mut self, apply: impl Fn(i32, i32) -> i32) -> Result<(), VmError> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        if b == 0 {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.ArithmeticException: / by zero",
            )));
        }
        self.stack.push(JValue::Int(apply(a, b)));
        Ok(())
    }

    fn double_binop(&mut self, apply: impl Fn(f64, f64) -> f64) -> Result<(), VmError> {
        let b = self.pop_double()?;
        let a = self.pop_double()?;
        self.stack.push(JValue::Double(apply(a, b)));
        Ok(())
    }
}

/// Resolve a branch offset relative to the branch opcode's address.
fn branch_target(
    addr: usize,
    offset: i16,
    code_len: usize,
    malformed: &impl Fn(String) -> VmError,
) -> Result<usize, VmError> {
    let target = i64::try_from(addr).expect("code fits i64") + i64::from(offset);
    usize::try_from(target)
        .ok()
        .filter(|t| *t < code_len)
        .ok_or_else(|| malformed(format!("branch at pc {addr} to out-of-range {target}")))
}

fn read_u8(
    bytes: &[u8],
    pc: &mut usize,
    malformed: &impl Fn(String) -> VmError,
) -> Result<u8, VmError> {
    let byte = *bytes
        .get(*pc)
        .ok_or_else(|| malformed(format!("truncated instruction at pc {pc}")))?;
    *pc += 1;
    Ok(byte)
}

fn read_u16(
    bytes: &[u8],
    pc: &mut usize,
    malformed: &impl Fn(String) -> VmError,
) -> Result<u16, VmError> {
    let high = read_u8(bytes, pc, malformed)?;
    let low = read_u8(bytes, pc, malformed)?;
    Ok(u16::from_be_bytes([high, low]))
}

/// Number of arguments in a method descriptor (each argument is one
/// operand-stack entry in this VM regardless of JVMS slot width).
fn descriptor_arg_count(descriptor: &str) -> Option<usize> {
    let inner = descriptor.strip_prefix('(')?;
    let (args, _return_type) = inner.split_once(')')?;
    let mut count = 0usize;
    let mut chars = args.chars();
    while let Some(c) = chars.next() {
        match c {
            'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'S' | 'Z' => count += 1,
            'L' => {
                chars.by_ref().find(|&c| c == ';')?;
                count += 1;
            }
            '[' => {
                // Array dimensions don't add arguments; the element
                // type that follows will be counted... unless it's
                // another '[', which the loop handles naturally. But a
                // '[' before a primitive would double-count, so consume
                // the element type here.
                let mut element = chars.next()?;
                while element == '[' {
                    element = chars.next()?;
                }
                if element == 'L' {
                    chars.by_ref().find(|&c| c == ';')?;
                }
                count += 1;
            }
            _ => return None,
        }
    }
    Some(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_arg_counts() {
        assert_eq!(descriptor_arg_count("()V"), Some(0));
        assert_eq!(descriptor_arg_count("(I)V"), Some(1));
        assert_eq!(descriptor_arg_count("(IDZ)V"), Some(3));
        assert_eq!(descriptor_arg_count("(Ljava/lang/String;I)V"), Some(2));
        assert_eq!(descriptor_arg_count("([Ljava/lang/String;)V"), Some(1));
        assert_eq!(descriptor_arg_count("([[I[D)V"), Some(2));
        assert_eq!(descriptor_arg_count("bogus"), None);
    }
}
