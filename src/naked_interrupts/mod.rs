//! This modules provides alternative implementation of the
//! interrupt handling functionalities without relying on the 
//! [x86_84] crate. That is, it builds its own IDT type and handles
//! the calling conventions manually.

pub mod idt;
mod handlers;

lazy_static! {
	/// Global IDT
	pub static ref IDT: idt::Idt = {
		let mut idt = idt::Idt::new();
		idt.set_handler(0, handlers::divide_by_zero_handler);
		idt.set_handler(3, handlers::breakpoint_handler);
		idt
	};
}

/// Initialize IDT
pub fn init_idt() {
	IDT.load();
}