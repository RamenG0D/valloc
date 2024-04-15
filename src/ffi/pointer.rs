use std::{ops::Index, usize};
use crate::pointer;

#[repr(C)]
pub struct CPointer {
    address: *mut u8,
    len: usize,
    index: usize
}

impl CPointer {
    pub fn new(address: *mut u8, len: usize, index: usize) -> Self {
        Self { address, len, index }
    }
}

impl From<&mut CPointer> for pointer::Pointer<u8> {
    fn from(ptr: &mut CPointer) -> Self {
        pointer::Pointer::Pointer {
            address: std::ptr::slice_from_raw_parts_mut(ptr.address, ptr.len),
            index: ptr.index
        }
    }
}

impl From<&CPointer> for pointer::Pointer<u8> {
    fn from(ptr: &CPointer) -> Self {
        pointer::Pointer::Pointer {
            address: std::ptr::slice_from_raw_parts_mut(ptr.address, ptr.len),
            index: ptr.index
        }
    }
}

impl From<CPointer> for pointer::Pointer<u8> {
    fn from(ptr: CPointer) -> Self {
        pointer::Pointer::Pointer {
            address: std::ptr::slice_from_raw_parts_mut(ptr.address, ptr.len),
            index: ptr.index
        }
    }
}
