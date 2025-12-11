static mut BUFFER: [u8; 24] = [0; 24];

#[unsafe(no_mangle)]
pub extern "C" fn itoa(n: isize) -> *const u8 {
    unsafe {
        let buffer = &raw mut BUFFER;

        if n == 0 {
            (*buffer)[0] = b'0';
            (*buffer)[1] = 0;
            return buffer as *const u8;
        }

        let mut idx = 0;
        let mut num = n;
        let is_negative = num < 0;

        if is_negative {
            (*buffer)[0] = b'-';
            idx = 1;
            num = -num;
        }

        let mut start = idx;
        let mut temp = num as usize;

        while temp > 0 {
            (*buffer)[idx] = (temp % 10) as u8 + b'0';
            temp /= 10;
            idx += 1;
        }

        let mut end = idx - 1;
        while start < end {
            let tmp = (*buffer)[start];
            (*buffer)[start] = (*buffer)[end];
            (*buffer)[end] = tmp;
            start += 1;
            end -= 1;
        }

        (*buffer)[idx] = 0;
        buffer as *const u8
    }
}
