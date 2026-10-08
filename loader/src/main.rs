use std::arch::asm;
mod debug;
mod veh;

#[macro_export]
macro_rules! dprintln {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

#[inline(never)]
fn some_function() {
    let dbg = unsafe { windows::Win32::System::Diagnostics::Debug::IsDebuggerPresent() };
    println!("This is an example function");
    println!("BeingDebugged: {}", dbg.0);
    println!("Done with some_function->ret");
}

#[inline(never)]
fn sleep() {
    let mut i = 0;
    let now = std::time::Instant::now();
    while i < 10_000 {
        unsafe {
            asm!("rdseed eax; xor eax, eax", options(nostack, nomem));
        }
        i += 1;
    }
    println!("Took {:#3?}", now.elapsed());
}

fn main() {
    let _time = std::time::Instant::now();
    demo();

    println!("Took: {:#?}", _time.elapsed());
    /*
    unsafe {
        asm!(
            "
        xor eax, eax
        jmp rax
        "
        )
    }
    */
}

pub fn demo() {
    crate::debug::no_strings_on_me();
    some_function();
    sleep();
    println!("Done, bye!");
}
