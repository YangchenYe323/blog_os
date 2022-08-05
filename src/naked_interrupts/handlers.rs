
use crate::println;

/// Devide By Zero handler
pub extern "C" fn divide_by_zero_handler() -> ! {
    println!("EXCEPTION: DIVIDE BY ZERO");
    loop {}
}

/// Break point handler
pub extern "C" fn breakpoint_handler() -> ! {
	println!("EXCEPTION: BREAKPOINT");
	loop {}
}