use crate::allocator::get_allocator;
// its not valid to use generics so we use this CCPointer struct instead its just a CPointer<u8>
use super::{
    buffer::{new_buffer, DataBuffer},
    pointer::CPointer
};

#[no_mangle]
/// Allocates a memory block of the specified size and returns a pointer to it
/// 
/// # Arguments
/// 
/// * `size` - The size of the memory block to allocate
/// 
/// # Returns
/// 
/// * `CPointer` - A pointer to the allocated memory block
/// 
/// # Safety
/// 
/// The caller must ensure that the pointer is freed using `vfree` when it is no longer needed
/// But the memory is owned by the allocator so it will be freed when the allocator is dropped
/// 
/// # Example
/// 
/// ```c
/// // Allocate memory, this returns a pointer to the "allocated" memory and is valid, but it is NOT a real pointer and cant be casted to a real pointer or dereferenced
/// CPointer ptr = valloc(10);
/// ```
pub extern fn valloc(size: usize) -> CPointer {
    let mut ptr = get_allocator().alloc::<u8>(size).unwrap();
    let addr = ptr.address_mut().unwrap();
    let (address, len) = (addr.as_mut_ptr(), addr.len());
    CPointer::new(address, len, ptr.get_index().unwrap())
}

#[no_mangle]
/// Frees the memory block pointed to by the specified pointer
/// 
/// # Arguments
/// 
/// * `ptr` - A pointer to the memory block to free
/// 
/// # Safety
/// 
/// The caller must ensure that the pointer is not used after it is freed or freed more than once
/// 
/// # Example
/// 
/// ```c
/// // Allocate memory
/// CPointer ptr = valloc(10);
/// 
/// // Free memory
/// vfree(&ptr);
/// 
/// // Accessing the memory after freeing it will result in a segmentation fault/undefined behavior
/// printf("Value at ptr: %d\n", vread(&ptr)); // doing this will result in undefined behavior
/// ```
pub extern fn vfree(ptr: &mut CPointer) {
    get_allocator().free::<u8>(&mut Into::<crate::pointer::Pointer<u8>>::into(ptr)).unwrap();
}

#[no_mangle]
/// Writes a value to the memory block pointed to by the specified pointer
/// 
/// # Arguments
/// 
/// * `ptr` - A pointer to the memory block to write to
/// 
/// * `value` - The value to write to the memory block
/// 
/// # Safety
/// 
/// The caller must ensure that the pointer is valid and points to a valid memory block
/// 
/// # Example
/// 
/// ```c
/// 
pub extern fn vwrite(ptr: &mut CPointer, value: u8) {
    get_allocator().write(&mut Into::<crate::pointer::Pointer<u8>>::into(ptr), value).unwrap();
}

#[no_mangle]
pub extern fn vread(ptr: &CPointer) -> u8 {
    get_allocator().read(&Into::<crate::pointer::Pointer<u8>>::into(ptr)).unwrap()
}

#[no_mangle]
pub extern fn read_buffer(ptr: &CPointer, size: usize) -> DataBuffer {
    let buffer = get_allocator().read_buffer(&mut Into::<crate::pointer::Pointer<u8>>::into(ptr), size).unwrap().leak();
    let (ptr, len) = (buffer.as_mut_ptr(), buffer.len());
    new_buffer(ptr, len)
}

#[no_mangle]
pub extern "C" fn write_buffer(ptr: &mut CPointer, data: *mut u8, length: usize) {
    get_allocator().write_buffer(&mut Into::<crate::pointer::Pointer<u8>>::into(ptr), data, length).unwrap();
}
