
#[repr(C)]
pub struct BufferData {
    data: *mut u8, 
    len: usize,
}

impl Drop for BufferData {
    fn drop(&mut self) {
        let _ = unsafe{ Vec::from_raw_parts(self.data, self.len, self.len) };
    }
}

#[no_mangle]
#[inline(always)]
pub extern fn new_buffer(data: *mut u8, len: usize) -> BufferData {
    BufferData { data, len }
}
