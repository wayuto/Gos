#![no_std]
#![no_main]
#![no_builtins]

use core::arch::asm;
use core::panic::PanicInfo;

pub mod convert;
pub mod gosio;
pub mod string;

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

unsafe extern "C" {
    fn main() -> i32;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        asm!("ud2", options(noreturn));
    }
}

#[inline(always)]
pub extern "C" fn syscall(nr: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    unsafe {
        asm!("
        mov rax, {nr} 
        mov rdi, {a1}
        mov rsi, {a2}
        mov rdx, {a3}
        syscall
        ", 
            nr = in(reg) nr,
            a1 = in(reg) a1,
            a2 = in(reg) a2,
            a3 = in(reg) a3,
        lateout("rax") ret,
        clobber_abi("C"),
        );
    }
    ret
}

#[unsafe(no_mangle)]
extern "C" fn _start() -> () {
    let ret = unsafe { main() };
    unsafe {
        asm!(
            "mov rdi, {ret}",
            "mov rax, 60",
            "syscall",
            ret = in(reg) ret as usize,
            options(noreturn)
        );
    }
}
