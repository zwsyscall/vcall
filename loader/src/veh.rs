#![allow(dead_code, unused, unsafe_op_in_unsafe_fn)]
use std::ptr;
use windows::Win32::{
    Foundation::{EXCEPTION_BREAKPOINT, EXCEPTION_SINGLE_STEP, STATUS_ACCESS_VIOLATION},
    System::Diagnostics::Debug::{
        AddVectoredExceptionHandler, CONTEXT, EXCEPTION_CONTINUE_EXECUTION,
        EXCEPTION_CONTINUE_SEARCH, EXCEPTION_POINTERS,
    },
};

use crate::{dprintln, main};

pub static mut VEH_HANDLE: *mut core::ffi::c_void = ptr::null_mut();

unsafe extern "C" {
    static __ImageBase: u8;
}

#[inline(always)]
pub fn get_image_base() -> usize {
    unsafe { &__ImageBase as *const _ as usize }
}

#[inline(always)]
#[unsafe(link_section = ".loader")]
fn rng(seed: i32) -> i32 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

#[inline(always)]
#[unsafe(link_section = ".loader")]
pub fn decrypt_ptr(encrypted_val: i32, absolute_addr_of_val: usize) -> i32 {
    let base = get_image_base();
    let rva = absolute_addr_of_val - base;
    let mask = rng(rva as i32);

    encrypted_val ^ mask
}

#[inline(always)]
#[unsafe(link_section = ".loader")]
unsafe fn push(ctx: &mut CONTEXT, val: u64) {
    ctx.Rsp -= 8;
    *(ctx.Rsp as *mut u64) = val;
}

#[inline(always)]
#[unsafe(link_section = ".loader")]
unsafe fn pop(ctx: &mut CONTEXT) -> u64 {
    let val = *(ctx.Rsp as *const u64);
    ctx.Rsp += 8;
    val
}

#[unsafe(link_section = ".loader")]
pub unsafe extern "system" fn call_virtualizer(exception_ptr: *mut EXCEPTION_POINTERS) -> i32 {
    let rec = &*(*exception_ptr).ExceptionRecord;
    let ctx = &mut *(*exception_ptr).ContextRecord;

    match rec.ExceptionCode {
        // CALL REL32
        EXCEPTION_BREAKPOINT => {
            let rip = ctx.Rip;
            let offset_ptr = (rip + 1) as *const i32;
            let rel_offset = decrypt_ptr(*offset_ptr, offset_ptr as usize);
            let return_addr = rip + 5;

            // Push return
            push(ctx, return_addr);

            // Calculate the target address
            ctx.Rip = return_addr.wrapping_add(rel_offset as i64 as u64);
            return EXCEPTION_CONTINUE_EXECUTION;
        }
        // RET
        EXCEPTION_SINGLE_STEP => {
            if *(ctx.Rip as *mut u8) != 0xf1 {
                // We can panic or do other wild shit here
            }

            let ret_addr = pop(ctx);

            ctx.Rip = ret_addr;
            return EXCEPTION_CONTINUE_EXECUTION;
        }
        STATUS_ACCESS_VIOLATION => {
            if ctx.Rip != 0x0 {
                return EXCEPTION_CONTINUE_SEARCH;
            }

            // Some value stays in rax, I don't really care what it is for now
            pop(ctx);

            // push RIP to the "real main" function
            ctx.Rip = crate::demo as *const u64 as u64;
            return EXCEPTION_CONTINUE_EXECUTION;
        }
        _ => {
            return EXCEPTION_CONTINUE_SEARCH;
        }
    }
}

#[unsafe(link_section = ".loader")]
unsafe extern "system" fn tls_callback(
    _h_module: *mut std::ffi::c_void,
    dw_reason: u32,
    _lp_reserved: *mut std::ffi::c_void,
) {
    unsafe {
        if dw_reason == 1 {
            VEH_HANDLE = AddVectoredExceptionHandler(1, Some(call_virtualizer));
        }
    }
}

#[unsafe(link_section = ".CRT$XLB")]
#[used]
pub static CALLBACK: unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut std::ffi::c_void) =
    tls_callback;
