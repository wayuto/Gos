use crate::{string::strlen, syscall};

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn write(fd: usize, buffer: *const u8, n: usize) -> isize {
    syscall(1, fd, buffer as usize, n)
}

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn print(fmt: *const u8) -> isize {
    let len = strlen(fmt);
    write(1, fmt, len)
}

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn println(fmt: *const u8) -> isize {
    let len = strlen(fmt);

    let str_result = write(1, fmt, len);
    if str_result < 0 {
        return str_result;
    }

    let newline = b"\n".as_ptr();
    let nl_result = write(1, newline, 1);

    if nl_result < 0 {
        nl_result
    } else {
        str_result + 1
    }
}
