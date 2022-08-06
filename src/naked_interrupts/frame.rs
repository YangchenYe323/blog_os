//! This module contains the code to deal with
//! x86_86's stack structure for interrupt handling.

/// [ExceptionStackFrame] corresponds to the stack frame set by x86_64
/// cpus in cases of an CPU exception, which is of the following structure:
/// [Variable-length Alignment]
/// [Stack Segment]
/// [Stack Pointer]
/// [RFLAGS]
/// [Instruction Segment]
/// [Instruction Pointer]
/// [(Optional) Error Code]
///
/// Since the stack grows from higher address to lower address,
/// the field of the struct is declared in reverse order, so that we could map
/// the address on the stack directly to a pointer of the structure.
#[derive(Debug)]
#[repr(C)]
pub struct ExceptionStackFrame {
  pub instruction_pointer: u64,
  pub code_segment: u64,
  pub cpu_flags: u64,
  pub stack_pointer: u64,
  pub stack_segment: u64,
}
