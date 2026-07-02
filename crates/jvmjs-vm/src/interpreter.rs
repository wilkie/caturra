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

    /// Execute one method to completion (v0: no user-method calls yet,
    /// so there is exactly one frame).
    pub fn execute(
        &mut self,
        class: &ClassFile,
        method: &MethodInfo,
        locals: Vec<JValue>,
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

            let opcode = *bytes.get(pc).ok_or_else(|| {
                malformed(format!("execution ran off the end of bytecode at pc {pc}"))
            })?;
            pc += 1;

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
            &self.heap,
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
    #[allow(dead_code)] // locals are read once loads land (stage 1).
    locals: Vec<JValue>,
    stack: Vec<JValue>,
}

impl Frame {
    fn pop(&mut self) -> Result<JValue, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }
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
