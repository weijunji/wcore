//! replace std to implement `panic` and `abort`

use core::panic::PanicInfo;

use crate::sbi::shutdown;

/// print information of panic and shutdown
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // TODO: unwind the stack
    unsafe {
        // use no lock to avoid deadlock in format
        println_no_lock!(
            "\x1b[1;31mpanic: '{}' in {}\x1b[0m",
            info.message().unwrap(),
            info.location().unwrap()
        );
    }
    shutdown()
}

/// stop the os
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
