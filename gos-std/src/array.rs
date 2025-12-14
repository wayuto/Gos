use core::ptr;

const MAX_RANGE_LEN: usize = 1024;
static mut RANGE_BUFFER_WITH_LEN: [isize; MAX_RANGE_LEN + 1] = [0; MAX_RANGE_LEN + 1];

#[unsafe(no_mangle)]
pub extern "C" fn range(start: isize, end: isize) -> *mut isize {
    if end <= start {
        return ptr::null_mut();
    }
    unsafe {
        let mut len = (end - start) as usize;

        let base_ptr = &mut RANGE_BUFFER_WITH_LEN[0] as *mut isize;

        if len > MAX_RANGE_LEN {
            len = MAX_RANGE_LEN;
        }

        *(base_ptr as *mut usize) = len;

        let data_ptr = base_ptr.add(1);

        for i in 0..len {
            *data_ptr.add(i) = start + i as isize;
        }

        base_ptr
    }
}
