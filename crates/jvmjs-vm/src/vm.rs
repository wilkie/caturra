//! The virtual machine: class loading and bytecode interpretation.
//!
//! This is the skeleton of the interpreter. The execution model is a
//! classic frame-stack interpreter (JVMS §2.5–2.6): each method call
//! pushes a frame holding locals and an operand stack, and the dispatch
//! loop decodes one opcode at a time from the current method's `Code`
//! attribute.

use std::collections::HashMap;

use jvmjs_classfile::{ClassFile, MethodAccessFlags};

use crate::interpreter::Interpreter;
use crate::io::ConsoleIo;
use crate::value::{HeapObject, JValue};
use crate::vfs::VirtualFileSystem;

/// Errors surfaced to the host while loading or running a program.
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("class not found: {0}")]
    ClassNotFound(String),
    #[error("class {0} has no public static void main(String[]) method")]
    NoMainMethod(String),
    #[error("malformed class {name}: {reason}")]
    MalformedClass { name: String, reason: String },
    #[error("uncaught exception: {0}")]
    UncaughtException(String),
    #[error("the program ran too long and was stopped (possible infinite loop)")]
    InstructionBudgetExceeded,
    #[error("unsupported bytecode instruction {0:#04x} (not yet implemented by the jvmjs VM)")]
    UnsupportedOpcode(u8),
    #[error("unknown native member: {0} (not yet implemented by the jvmjs class library)")]
    UnknownIntrinsic(String),
    #[error("operand stack underflow (malformed bytecode)")]
    StackUnderflow,
}

/// Options controlling a VM instance.
#[derive(Debug, Clone)]
pub struct VmOptions {
    /// Upper bound on interpreted instructions per `run` call, so a
    /// student's `while (true) {}` can't hang the browser tab.
    pub max_instructions: u64,
    /// Maximum method-call depth; exceeding it raises Java's
    /// `StackOverflowError`. The interpreter currently recurses on the
    /// host stack, so this must stay well inside the WASM stack budget;
    /// deep-recursion support means moving to an explicit frame stack
    /// (see `specs/RUNTIME.md`).
    pub max_call_depth: u32,
}

impl Default for VmOptions {
    fn default() -> Self {
        Self {
            max_instructions: 500_000_000,
            max_call_depth: 256,
        }
    }
}

/// Exit status of a completed program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// `main` returned normally.
    Completed,
    /// The program called `System.exit(code)`.
    Exited(i32),
}

/// A JVM instance: loaded classes, a heap (to come), a virtual
/// filesystem, and console IO supplied by the host.
pub struct Vm<'host> {
    options: VmOptions,
    classes: HashMap<String, ClassFile>,
    vfs: &'host mut VirtualFileSystem,
    console: &'host mut dyn ConsoleIo,
}

impl std::fmt::Debug for Vm<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("options", &self.options)
            .field("classes", &self.classes.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl<'host> Vm<'host> {
    pub fn new(
        options: VmOptions,
        vfs: &'host mut VirtualFileSystem,
        console: &'host mut dyn ConsoleIo,
    ) -> Self {
        Self {
            options,
            classes: HashMap::new(),
            vfs,
            console,
        }
    }

    /// Register a compiled class with the VM, keyed by its binary name.
    pub fn load_class(&mut self, class: ClassFile) -> Result<(), VmError> {
        let name = class
            .class_name()
            .ok_or_else(|| VmError::MalformedClass {
                name: String::from("<unknown>"),
                reason: String::from("this_class does not resolve to a name"),
            })?
            .to_owned();
        self.classes.insert(name, class);
        Ok(())
    }

    /// Names of all loaded classes.
    #[must_use]
    pub fn loaded_classes(&self) -> Vec<&str> {
        self.classes.keys().map(String::as_str).collect()
    }

    /// Run `public static void main(String[] args)` of the named class.
    ///
    /// TODO(interpreter): `<clinit>` does not run yet (no statics in the
    /// current language stage), and `java.io.File` intrinsics are not
    /// wired to `self.vfs` yet (`File` lands with the classlib).
    pub fn run_main(&mut self, class_name: &str, args: &[String]) -> Result<ExitStatus, VmError> {
        let class = self
            .classes
            .get(class_name)
            .ok_or_else(|| VmError::ClassNotFound(class_name.to_owned()))?;

        let main = class
            .methods
            .iter()
            .find(|m| {
                class.constant_pool.get_utf8(m.name_index) == Some("main")
                    && class.constant_pool.get_utf8(m.descriptor_index)
                        == Some("([Ljava/lang/String;)V")
                    && m.access_flags.contains(MethodAccessFlags::STATIC)
            })
            .ok_or_else(|| VmError::NoMainMethod(class_name.to_owned()))?;

        let _ = self.vfs.is_empty(); // wired to File intrinsics with the classlib

        let mut interpreter = Interpreter::new(
            &self.classes,
            self.console,
            self.options.max_instructions,
            self.options.max_call_depth,
        );
        let arg_refs: Vec<JValue> = args
            .iter()
            .map(|a| JValue::Ref(Some(interpreter.intern_string(a))))
            .collect();
        let args_array = interpreter.heap.alloc(HeapObject::RefArray(arg_refs));
        let locals = vec![JValue::Ref(Some(args_array))];

        match interpreter.execute(class, main, locals) {
            Ok(_) => Ok(ExitStatus::Completed),
            Err(VmError::UncaughtException(message)) => {
                // Match the shape of the real `java` launcher's output.
                self.console
                    .stderr(format!("Exception in thread \"main\" {message}\n").as_bytes());
                Err(VmError::UncaughtException(message))
            }
            Err(other) => Err(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::BufferedConsole;
    use jvmjs_classfile::{ClassFile, Constant};

    fn class_named(name: &str) -> ClassFile {
        let mut class = ClassFile::new_java11();
        let name_index = class.constant_pool.intern_utf8(name);
        class.this_class = class.constant_pool.push(Constant::Class { name_index });
        class
    }

    #[test]
    fn load_class_registers_by_binary_name() {
        let mut vfs = VirtualFileSystem::new();
        let mut console = BufferedConsole::new();
        let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
        vm.load_class(class_named("Main")).unwrap();
        assert_eq!(vm.loaded_classes(), vec!["Main"]);
    }

    #[test]
    fn run_main_reports_missing_class() {
        let mut vfs = VirtualFileSystem::new();
        let mut console = BufferedConsole::new();
        let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
        let err = vm.run_main("Missing", &[]).unwrap_err();
        assert!(matches!(err, VmError::ClassNotFound(name) if name == "Missing"));
    }
}
