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

/// State for one `run`: the loaded classes, heap, interned strings,
/// intrinsic singletons, and the instruction/call-depth budgets.
pub(crate) struct Interpreter<'run> {
    pub classes: &'run HashMap<String, ClassFile>,
    pub console: &'run mut dyn ConsoleIo,
    pub heap: Heap,
    pub intrinsic_statics: IntrinsicStatics,
    string_pool: HashMap<String, HeapRef>,
    /// Static field values per user class, created on first use.
    statics: HashMap<String, HashMap<String, JValue>>,
    /// Classes whose initialization has started (JVMS §5.5: recursive
    /// initialization by the same thread proceeds).
    init_started: std::collections::HashSet<String>,
    remaining_instructions: u64,
    call_depth: u32,
    max_call_depth: u32,
}

impl<'run> Interpreter<'run> {
    pub fn new(
        classes: &'run HashMap<String, ClassFile>,
        console: &'run mut dyn ConsoleIo,
        max_instructions: u64,
        max_call_depth: u32,
    ) -> Self {
        Self {
            classes,
            console,
            heap: Heap::new(),
            intrinsic_statics: IntrinsicStatics::default(),
            string_pool: HashMap::new(),
            statics: HashMap::new(),
            init_started: std::collections::HashSet::new(),
            remaining_instructions: max_instructions,
            call_depth: 0,
            max_call_depth,
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

    /// Execute one method to completion, returning its result value
    /// (`None` for void). Recursion mirrors Java's call stack, guarded
    /// by `max_call_depth` → `StackOverflowError`.
    pub fn execute(
        &mut self,
        class: &ClassFile,
        method: &MethodInfo,
        locals: Vec<JValue>,
    ) -> Result<Option<JValue>, VmError> {
        self.call_depth += 1;
        if self.call_depth > self.max_call_depth {
            self.call_depth -= 1;
            return Err(VmError::UncaughtException(String::from(
                "java.lang.StackOverflowError",
            )));
        }
        let result = self.execute_frame(class, method, locals);
        self.call_depth -= 1;
        result
    }

    /// The dispatch loop is one long match by design; splitting it per
    /// opcode family would only scatter the instruction set.
    #[allow(clippy::too_many_lines)]
    fn execute_frame(
        &mut self,
        class: &ClassFile,
        method: &MethodInfo,
        mut locals: Vec<JValue>,
    ) -> Result<Option<JValue>, VmError> {
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
                op::POP2 => {
                    // One category-2 value or two category-1 values.
                    let top = frame.pop()?;
                    if !matches!(top, JValue::Double(_) | JValue::Long(_)) {
                        frame.pop()?;
                    }
                }
                op::DUP => {
                    let top = *frame.stack.last().ok_or(VmError::StackUnderflow)?;
                    frame.stack.push(top);
                }
                op::DUP2 => {
                    // One category-2 value or the top two category-1s.
                    let top = *frame.stack.last().ok_or(VmError::StackUnderflow)?;
                    if matches!(top, JValue::Double(_) | JValue::Long(_)) {
                        frame.stack.push(top);
                    } else {
                        let len = frame.stack.len();
                        if len < 2 {
                            return Err(VmError::StackUnderflow);
                        }
                        let under = frame.stack[len - 2];
                        frame.stack.push(under);
                        frame.stack.push(top);
                    }
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
                op::IFNULL | op::IFNONNULL => {
                    let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                    let reference = frame.pop_ref()?;
                    if reference.is_none() == (opcode == op::IFNULL) {
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
                        .ok_or_else(|| malformed(format!("bad class ref at pool {index}")))?
                        .to_owned();
                    let object = if self.classes.contains_key(&target) {
                        self.ensure_initialized(&target)?;
                        self.new_instance(&target)
                    } else {
                        intrinsics::instantiate(&target).ok_or_else(|| {
                            VmError::UnknownIntrinsic(format!("cannot instantiate {target}"))
                        })?
                    };
                    let reference = self.heap.alloc(object);
                    frame.stack.push(JValue::Ref(Some(reference)));
                }
                op::INVOKESPECIAL => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.invoke_special_op(class, &mut frame, index, &malformed)?;
                }
                op::GETSTATIC => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.getstatic_op(class, &mut frame, index, &malformed)?;
                }
                op::PUTSTATIC => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.putstatic_op(class, &mut frame, index, &malformed)?;
                }
                op::GETFIELD => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.getfield_op(class, &mut frame, index, &malformed)?;
                }
                op::PUTFIELD => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.putfield_op(class, &mut frame, index, &malformed)?;
                }
                op::INVOKEVIRTUAL => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    self.invoke_virtual_op(class, &mut frame, index, &malformed)?;
                }
                op::INVOKESTATIC => {
                    let index = read_u16(bytes, &mut pc, &malformed)?;
                    let (target_class, method_name, descriptor) = class
                        .constant_pool
                        .get_member_ref(index)
                        .map(|(c, m, d)| (c.to_owned(), m.to_owned(), d.to_owned()))
                        .ok_or_else(|| malformed(format!("bad method ref at pool {index}")))?;
                    let widths = descriptor_arg_widths(&descriptor)
                        .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
                    let mut args = Vec::with_capacity(widths.len());
                    for _ in &widths {
                        args.push(frame.pop()?);
                    }
                    args.reverse();
                    let result = self.invoke_static(
                        &target_class,
                        &method_name,
                        &descriptor,
                        &widths,
                        &args,
                    )?;
                    if let Some(value) = result {
                        frame.stack.push(value);
                    }
                }
                // ----- arrays -----
                op::NEWARRAY => {
                    let atype = read_u8(bytes, &mut pc, &malformed)?;
                    let length = check_array_size(frame.pop_int()?)?;
                    let object = match atype {
                        op::T_INT | op::T_BOOLEAN | op::T_CHAR => {
                            crate::value::HeapObject::IntArray(vec![0; length])
                        }
                        op::T_DOUBLE => crate::value::HeapObject::DoubleArray(vec![0.0; length]),
                        other => {
                            return Err(malformed(format!("unsupported newarray type {other}")));
                        }
                    };
                    let reference = self.heap.alloc(object);
                    frame.stack.push(JValue::Ref(Some(reference)));
                }
                op::ANEWARRAY => {
                    let _class_index = read_u16(bytes, &mut pc, &malformed)?;
                    let length = check_array_size(frame.pop_int()?)?;
                    let reference = self.heap.alloc(crate::value::HeapObject::RefArray(vec![
                            JValue::NULL;
                            length
                        ]));
                    frame.stack.push(JValue::Ref(Some(reference)));
                }
                op::MULTIANEWARRAY => {
                    let class_index = read_u16(bytes, &mut pc, &malformed)?;
                    let dims = usize::from(read_u8(bytes, &mut pc, &malformed)?);
                    let descriptor = class
                        .constant_pool
                        .get_class_name(class_index)
                        .ok_or_else(|| malformed(format!("bad class ref at pool {class_index}")))?
                        .to_owned();
                    let mut counts = Vec::with_capacity(dims);
                    for _ in 0..dims {
                        counts.push(frame.pop_int()?);
                    }
                    counts.reverse();
                    let reference = self.alloc_multi_array(&descriptor, &counts, &malformed)?;
                    frame.stack.push(JValue::Ref(Some(reference)));
                }
                op::ARRAYLENGTH => {
                    let reference = frame.pop_ref()?.ok_or_else(null_array)?;
                    let length = match self.heap.get(reference) {
                        Some(crate::value::HeapObject::IntArray(v)) => v.len(),
                        Some(crate::value::HeapObject::DoubleArray(v)) => v.len(),
                        Some(crate::value::HeapObject::RefArray(v)) => v.len(),
                        _ => return Err(malformed(String::from("arraylength on a non-array"))),
                    };
                    frame
                        .stack
                        .push(JValue::Int(i32::try_from(length).unwrap_or(i32::MAX)));
                }
                op::IALOAD | op::BALOAD | op::CALOAD => {
                    let (reference, index) = frame.pop_array_access()?;
                    let Some(crate::value::HeapObject::IntArray(values)) = self.heap.get(reference)
                    else {
                        return Err(malformed(String::from("int-array load on a non-int-array")));
                    };
                    let value = *array_get(values, index)?;
                    frame.stack.push(JValue::Int(value));
                }
                op::DALOAD => {
                    let (reference, index) = frame.pop_array_access()?;
                    let Some(crate::value::HeapObject::DoubleArray(values)) =
                        self.heap.get(reference)
                    else {
                        return Err(malformed(String::from("daload on a non-double-array")));
                    };
                    let value = *array_get(values, index)?;
                    frame.stack.push(JValue::Double(value));
                }
                op::AALOAD => {
                    let (reference, index) = frame.pop_array_access()?;
                    let Some(crate::value::HeapObject::RefArray(values)) = self.heap.get(reference)
                    else {
                        return Err(malformed(String::from("aaload on a non-reference-array")));
                    };
                    let value = *array_get(values, index)?;
                    frame.stack.push(value);
                }
                op::IASTORE | op::BASTORE | op::CASTORE => {
                    let value = frame.pop_int()?;
                    let (reference, index) = frame.pop_array_access()?;
                    let Some(crate::value::HeapObject::IntArray(values)) =
                        self.heap.get_mut(reference)
                    else {
                        return Err(malformed(String::from(
                            "int-array store on a non-int-array",
                        )));
                    };
                    let slot = array_get_mut(values, index)?;
                    // Stores mask to the element width (JVMS castore /
                    // bastore); boolean arrays hold 0/1.
                    *slot = match opcode {
                        op::CASTORE => i32::from(value.cast_unsigned() as u16),
                        op::BASTORE => value & 1,
                        _ => value,
                    };
                }
                op::DASTORE => {
                    let value = frame.pop_double()?;
                    let (reference, index) = frame.pop_array_access()?;
                    let Some(crate::value::HeapObject::DoubleArray(values)) =
                        self.heap.get_mut(reference)
                    else {
                        return Err(malformed(String::from("dastore on a non-double-array")));
                    };
                    *array_get_mut(values, index)? = value;
                }
                op::AASTORE => {
                    let value = frame.pop()?;
                    let (reference, index) = frame.pop_array_access()?;
                    let Some(crate::value::HeapObject::RefArray(values)) =
                        self.heap.get_mut(reference)
                    else {
                        return Err(malformed(String::from("aastore on a non-reference-array")));
                    };
                    *array_get_mut(values, index)? = value;
                }

                op::RETURN => return Ok(None),
                op::IRETURN | op::DRETURN | op::ARETURN => {
                    let value = frame.pop()?;
                    return Ok(Some(value));
                }
                other => return Err(VmError::UnsupportedOpcode(other)),
            }
        }
    }

    /// `getstatic` (kept out of the dispatch loop so its temporaries
    /// don't inflate every interpreter frame in debug builds).
    #[inline(never)]
    fn getstatic_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (field_class, field_name, _descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, f, d)| (c.to_owned(), f.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let value = if self.classes.contains_key(&field_class) {
            self.ensure_initialized(&field_class)?;
            *self
                .statics
                .get(&field_class)
                .and_then(|fields| fields.get(&field_name))
                .ok_or_else(|| {
                    malformed(format!("unknown static field {field_class}.{field_name}"))
                })?
        } else {
            self.intrinsic_statics
                .static_field(&mut self.heap, &field_class, &field_name)
                .ok_or_else(|| VmError::UnknownIntrinsic(format!("{field_class}.{field_name}")))?
        };
        frame.stack.push(value);
        Ok(())
    }

    #[inline(never)]
    fn putstatic_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (field_class, field_name, _descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, f, d)| (c.to_owned(), f.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        if !self.classes.contains_key(&field_class) {
            return Err(VmError::UnknownIntrinsic(format!(
                "{field_class}.{field_name}"
            )));
        }
        self.ensure_initialized(&field_class)?;
        let value = frame.pop()?;
        let slot = self
            .statics
            .get_mut(&field_class)
            .and_then(|fields| fields.get_mut(&field_name))
            .ok_or_else(|| malformed(format!("unknown static field {field_class}.{field_name}")))?;
        *slot = value;
        Ok(())
    }

    #[inline(never)]
    fn getfield_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (_, field_name, _) = class
            .constant_pool
            .get_member_ref(index)
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let field_name = field_name.to_owned();
        let reference = frame.pop_ref()?.ok_or_else(|| {
            VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot read field \"{field_name}\" \
                 because the object is null"
            ))
        })?;
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get(reference)
        else {
            return Err(malformed(String::from("getfield on a non-object")));
        };
        let value = *fields
            .get(&field_name)
            .ok_or_else(|| malformed(format!("unknown field {field_name}")))?;
        frame.stack.push(value);
        Ok(())
    }

    #[inline(never)]
    fn putfield_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (_, field_name, _) = class
            .constant_pool
            .get_member_ref(index)
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let field_name = field_name.to_owned();
        let value = frame.pop()?;
        let reference = frame.pop_ref()?.ok_or_else(|| {
            VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot assign field \"{field_name}\" \
                 because the object is null"
            ))
        })?;
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get_mut(reference)
        else {
            return Err(malformed(String::from("putfield on a non-object")));
        };
        fields.insert(field_name, value);
        Ok(())
    }

    /// `invokespecial`: user constructors and intrinsic `<init>`s.
    #[inline(never)]
    fn invoke_special_op(
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
        let widths = descriptor_arg_widths(&descriptor)
            .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
        let mut args = Vec::with_capacity(widths.len());
        for _ in &widths {
            args.push(frame.pop()?);
        }
        args.reverse();
        let Some(receiver) = frame.pop_ref()? else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        if let Some(target) = classes.get(&target_class) {
            // A user constructor.
            let ctor = target
                .methods
                .iter()
                .find(|m| {
                    target.constant_pool.get_utf8(m.name_index) == Some(method_name.as_str())
                        && target.constant_pool.get_utf8(m.descriptor_index)
                            == Some(descriptor.as_str())
                })
                .ok_or_else(|| {
                    malformed(format!(
                        "no constructor {method_name}{descriptor} in {target_class}"
                    ))
                })?;
            let mut locals = Vec::with_capacity(1 + args.len());
            locals.push(JValue::Ref(Some(receiver)));
            for (value, width) in args.iter().zip(&widths) {
                locals.push(*value);
                if *width == 2 {
                    locals.push(JValue::Int(0));
                }
            }
            self.execute(target, ctor, locals)?;
        } else {
            intrinsics::invoke_special(
                &self.heap,
                receiver,
                &target_class,
                &method_name,
                &descriptor,
            )?;
        }
        Ok(())
    }

    /// Run a user class's static initialization on first use
    /// (JVMS §5.5): default static field values, then `<clinit>`.
    fn ensure_initialized(&mut self, class_name: &str) -> Result<(), VmError> {
        if self.init_started.contains(class_name) {
            return Ok(());
        }
        self.init_started.insert(class_name.to_owned());

        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let Some(class) = classes.get(class_name) else {
            return Ok(());
        };
        let mut fields = HashMap::new();
        for field in &class.fields {
            if !field
                .access_flags
                .contains(jvmjs_classfile::FieldAccessFlags::STATIC)
            {
                continue;
            }
            let name = class
                .constant_pool
                .get_utf8(field.name_index)
                .unwrap_or_default()
                .to_owned();
            let descriptor = class
                .constant_pool
                .get_utf8(field.descriptor_index)
                .unwrap_or_default();
            fields.insert(name, default_for_descriptor(descriptor));
        }
        self.statics.insert(class_name.to_owned(), fields);

        if let Some(clinit) = find_static_method(class, "<clinit>", "()V") {
            self.execute(class, clinit, Vec::new())?;
        }
        Ok(())
    }

    /// Allocate an instance of a user class with defaulted fields.
    fn new_instance(&mut self, class_name: &str) -> crate::value::HeapObject {
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let class = classes.get(class_name).expect("checked by caller");
        let mut fields = HashMap::new();
        for field in &class.fields {
            if field
                .access_flags
                .contains(jvmjs_classfile::FieldAccessFlags::STATIC)
            {
                continue;
            }
            let name = class
                .constant_pool
                .get_utf8(field.name_index)
                .unwrap_or_default()
                .to_owned();
            let descriptor = class
                .constant_pool
                .get_utf8(field.descriptor_index)
                .unwrap_or_default();
            fields.insert(name, default_for_descriptor(descriptor));
        }
        crate::value::HeapObject::Instance {
            class_name: class_name.to_owned(),
            fields,
        }
    }

    /// Dispatch an instance method on a user-defined object, with the
    /// default `toString` when the class doesn't define one.
    fn invoke_user_virtual(
        &mut self,
        receiver: HeapRef,
        instance_class: &str,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Option<JValue>, VmError> {
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let class = classes
            .get(instance_class)
            .ok_or_else(|| VmError::MalformedClass {
                name: instance_class.to_owned(),
                reason: String::from("instance of an unloaded class"),
            })?;
        let method = class.methods.iter().find(|m| {
            !m.access_flags
                .contains(jvmjs_classfile::MethodAccessFlags::STATIC)
                && class.constant_pool.get_utf8(m.name_index) == Some(method_name)
                && class.constant_pool.get_utf8(m.descriptor_index) == Some(descriptor)
        });
        let Some(method) = method else {
            // Object.toString() default: "ClassName@<hex>".
            if method_name == "toString" && descriptor == "()Ljava/lang/String;" {
                let text = format!("{instance_class}@{receiver:x}");
                let reference = self.heap.alloc_string(&text);
                return Ok(Some(JValue::Ref(Some(reference))));
            }
            return Err(VmError::UnknownIntrinsic(format!(
                "{instance_class}.{method_name}{descriptor}"
            )));
        };

        self.ensure_initialized(instance_class)?;
        let widths = descriptor_arg_widths(descriptor).ok_or_else(|| VmError::MalformedClass {
            name: instance_class.to_owned(),
            reason: format!("bad descriptor {descriptor}"),
        })?;
        let mut locals = Vec::with_capacity(1 + args.len());
        locals.push(JValue::Ref(Some(receiver)));
        for (value, width) in args.iter().zip(&widths) {
            locals.push(*value);
            if *width == 2 {
                locals.push(JValue::Int(0));
            }
        }
        self.execute(class, method, locals)
    }

    /// Recursively allocate a (possibly partial) multi-dimensional
    /// array described by `descriptor` (e.g. `[[I`) with the given
    /// leading dimension counts.
    fn alloc_multi_array(
        &mut self,
        descriptor: &str,
        counts: &[i32],
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<HeapRef, VmError> {
        let (first, rest_counts) = counts
            .split_first()
            .ok_or_else(|| malformed(String::from("multianewarray with zero dimensions")))?;
        let length = check_array_size(*first)?;
        let element_descriptor = descriptor
            .strip_prefix('[')
            .ok_or_else(|| malformed(format!("bad array descriptor {descriptor}")))?;

        if rest_counts.is_empty() {
            let object = match element_descriptor {
                "I" | "Z" | "C" => crate::value::HeapObject::IntArray(vec![0; length]),
                "D" => crate::value::HeapObject::DoubleArray(vec![0.0; length]),
                _ => crate::value::HeapObject::RefArray(vec![JValue::NULL; length]),
            };
            return Ok(self.heap.alloc(object));
        }

        let mut rows = Vec::with_capacity(length);
        for _ in 0..length {
            let row = self.alloc_multi_array(element_descriptor, rest_counts, malformed)?;
            rows.push(JValue::Ref(Some(row)));
        }
        Ok(self.heap.alloc(crate::value::HeapObject::RefArray(rows)))
    }

    /// `invokestatic`: dispatch to a user-defined static method,
    /// building its frame's locals from the popped arguments (doubles
    /// occupy two slots, JVMS §2.6.1).
    fn invoke_static(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        widths: &[u16],
        args: &[JValue],
    ) -> Result<Option<JValue>, VmError> {
        // Decouple the class borrow from `self` so the recursive
        // `execute` can take `&mut self`.
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let Some(class) = classes.get(class_name) else {
            // TODO(classlib): fall back to intrinsic static methods
            // (Math.abs, Integer.parseInt, ...) when they land.
            return Err(VmError::UnknownIntrinsic(format!(
                "{class_name}.{method_name}{descriptor}"
            )));
        };
        self.ensure_initialized(class_name)?;
        let method = find_static_method(class, method_name, descriptor).ok_or_else(|| {
            VmError::MalformedClass {
                name: class_name.to_owned(),
                reason: format!("no static method {method_name}{descriptor}"),
            }
        })?;

        let mut locals = Vec::with_capacity(args.len());
        for (value, width) in args.iter().zip(widths) {
            locals.push(*value);
            if *width == 2 {
                locals.push(JValue::Int(0)); // padding for the wide slot
            }
        }
        self.execute(class, method, locals)
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

        // User-defined objects dispatch on the instance's actual class;
        // everything else is an intrinsic (PrintStream, StringBuilder).
        let instance_class = match self.heap.get(receiver) {
            Some(crate::value::HeapObject::Instance { class_name, .. }) => Some(class_name.clone()),
            _ => None,
        };
        let result = if let Some(instance_class) = instance_class {
            self.invoke_user_virtual(receiver, &instance_class, &method_name, &descriptor, &args)?
        } else {
            intrinsics::invoke_virtual(
                &mut self.heap,
                self.console,
                receiver,
                &target_class,
                &method_name,
                &descriptor,
                &args,
            )?
        };
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

    /// Pop `index` then `arrayref` for an array access, raising NPE for
    /// a null array.
    fn pop_array_access(&mut self) -> Result<(HeapRef, i32), VmError> {
        let index = self.pop_int()?;
        let reference = self.pop_ref()?.ok_or_else(null_array)?;
        Ok((reference, index))
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

/// The default value for a field of the given descriptor (JLS §4.12.5).
fn default_for_descriptor(descriptor: &str) -> JValue {
    match descriptor.chars().next() {
        Some('D') => JValue::Double(0.0),
        Some('I' | 'Z' | 'C' | 'B' | 'S') => JValue::Int(0),
        Some('J') => JValue::Long(0),
        Some('F') => JValue::Float(0.0),
        _ => JValue::NULL,
    }
}

fn null_array() -> VmError {
    VmError::UncaughtException(String::from(
        "java.lang.NullPointerException: the array is null",
    ))
}

/// Validate an array creation size (JVMS: negative →
/// `NegativeArraySizeException`).
fn check_array_size(size: i32) -> Result<usize, VmError> {
    usize::try_from(size).map_err(|_| {
        VmError::UncaughtException(format!("java.lang.NegativeArraySizeException: {size}"))
    })
}

/// Bounds-checked array element access with Java 11's exception
/// message format.
fn array_get<T>(values: &[T], index: i32) -> Result<&T, VmError> {
    usize::try_from(index)
        .ok()
        .and_then(|i| values.get(i))
        .ok_or_else(|| out_of_bounds(index, values.len()))
}

fn array_get_mut<T>(values: &mut [T], index: i32) -> Result<&mut T, VmError> {
    let length = values.len();
    usize::try_from(index)
        .ok()
        .and_then(|i| values.get_mut(i))
        .ok_or_else(|| out_of_bounds(index, length))
}

fn out_of_bounds(index: i32, length: usize) -> VmError {
    VmError::UncaughtException(format!(
        "java.lang.ArrayIndexOutOfBoundsException: Index {index} out of bounds for length {length}"
    ))
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

/// Find a class's static method by name and exact descriptor.
fn find_static_method<'c>(
    class: &'c ClassFile,
    name: &str,
    descriptor: &str,
) -> Option<&'c MethodInfo> {
    class.methods.iter().find(|m| {
        m.access_flags
            .contains(jvmjs_classfile::MethodAccessFlags::STATIC)
            && class.constant_pool.get_utf8(m.name_index) == Some(name)
            && class.constant_pool.get_utf8(m.descriptor_index) == Some(descriptor)
    })
}

/// Local-variable slot widths of each argument in a method descriptor
/// (JVMS §2.6.1: double/long take two slots).
fn descriptor_arg_widths(descriptor: &str) -> Option<Vec<u16>> {
    let inner = descriptor.strip_prefix('(')?;
    let (args, _return_type) = inner.split_once(')')?;
    let mut widths = Vec::new();
    let mut chars = args.chars();
    while let Some(c) = chars.next() {
        match c {
            'B' | 'C' | 'F' | 'I' | 'S' | 'Z' => widths.push(1),
            'D' | 'J' => widths.push(2),
            'L' => {
                chars.by_ref().find(|&c| c == ';')?;
                widths.push(1);
            }
            '[' => {
                // Arrays are references (width 1) regardless of the
                // element type; consume the element descriptor.
                let mut element = chars.next()?;
                while element == '[' {
                    element = chars.next()?;
                }
                if element == 'L' {
                    chars.by_ref().find(|&c| c == ';')?;
                }
                widths.push(1);
            }
            _ => return None,
        }
    }
    Some(widths)
}

/// Number of arguments in a method descriptor (each argument is one
/// operand-stack entry in this VM regardless of JVMS slot width).
fn descriptor_arg_count(descriptor: &str) -> Option<usize> {
    descriptor_arg_widths(descriptor).map(|w| w.len())
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
