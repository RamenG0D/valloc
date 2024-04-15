
#[repr(C)]
pub struct DataBuffer {
    data: *mut u8, 
    len: usize,
}

impl Drop for DataBuffer {
    fn drop(&mut self) {
        let _ = unsafe{ Vec::from_raw_parts(self.data, self.len, self.len) };
    }
}

#[no_mangle]
#[inline(always)]
pub extern fn new_buffer(data: *mut u8, len: usize) -> DataBuffer {
    DataBuffer { data, len }
}
