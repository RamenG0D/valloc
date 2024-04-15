use crate::{allocator::Valloc, pointer};
// its not valid to use generics so we use this CPointer struct instead its just a Pointer<u8>
use super::{buffer::{new_buffer, BufferData}, pointer::CPointer as Pointer};

// global allocator
static mut ALLOCATOR: Option<Valloc> = None;

#[no_mangle]
fn get_allocator() -> &'static mut Valloc {
    unsafe {
        match ALLOCATOR {
            Some(ref mut allocator) => allocator,
            None => panic!("Allocator not initialized!")
        }
    }
}

#[no_mangle]
pub extern fn valloc_init(total_mem_size: usize) {
    unsafe { ALLOCATOR = Some(Valloc::new(total_mem_size)); }
}

#[no_mangle]
pub extern fn valloc(size: usize) -> Pointer {
    let mut ptr = get_allocator().alloc::<u8>(size).unwrap();
    let addr = ptr.address_mut().unwrap();
    let (address, len) = (addr.as_mut_ptr(), addr.len());
    Pointer::new(address, len, ptr.get_index().unwrap())
}

#[no_mangle]
pub extern fn vfree(ptr: &mut Pointer) {
    get_allocator().free::<u8>(&mut Into::<pointer::Pointer<u8>>::into(ptr)).unwrap();
}

#[no_mangle]
pub extern fn vwrite(ptr: &mut Pointer, value: u8) {
    get_allocator().write(&mut Into::<pointer::Pointer<u8>>::into(ptr), value).unwrap();
}

#[no_mangle]
pub extern fn vread(ptr: &Pointer) -> u8 {
    get_allocator().read(&Into::<pointer::Pointer<u8>>::into(ptr)).unwrap()
}

#[no_mangle]
pub extern fn read_buffer(ptr: &Pointer, size: usize) -> BufferData {
    let buffer = get_allocator().read_buffer(&mut Into::<pointer::Pointer<u8>>::into(ptr), size).unwrap().leak();
    let (ptr, len) = (buffer.as_mut_ptr(), buffer.len());
    new_buffer(ptr, len)
}

#[no_mangle]
pub extern "C" fn write_buffer(ptr: &mut Pointer, data: *mut u8, length: usize) {
    get_allocator().write_buffer(&mut Into::<pointer::Pointer<u8>>::into(ptr), data, length).unwrap();
}
