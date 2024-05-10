use crate::allocator::{get_allocator, valloc_init, SmartPointer, Valloc};

/// Allocates a block of memory of the specified size using the vCPU allocator.
/// Returns a raw pointer to the allocated memory.
#[no_mangle]
pub extern "C" fn valloc(size: usize) -> *mut () {
    get_allocator().alloc::<()>(size).unwrap().as_ptr().cast()
}

/// Frees the memory block pointed to by `ptr` using the vCPU allocator.
#[no_mangle]
pub extern "C" fn vfree(ptr: *mut ()) {
    get_allocator().free::<()>(unsafe{SmartPointer::new_unchecked(ptr.cast())}).unwrap();
}

/// Resizes the memory block pointed to by `ptr` to the specified size using the vCPU allocator.
/// Returns a raw pointer to the resized memory block.
#[no_mangle]
pub extern "C" fn vrealloc(ptr: *mut (), size: usize) -> *mut std::ffi::c_void {
    get_allocator().realloc::<()>(unsafe{SmartPointer::new_unchecked(ptr.cast())}, size).unwrap().as_ptr().cast()
}

/// Allocates a block of memory of the specified size using the vCPU allocator.
/// Returns a raw pointer to the allocated memory.
#[no_mangle]
pub extern "C" fn virtual_alloc(allocator: &'static mut Valloc, size: usize) -> *mut () {
    match allocator.alloc::<()>(size) {
        Ok(val) => val.as_ptr(),
        Err(e) => panic!("{e}"),
    }
}

/// Frees the memory block pointed to by `ptr` using the vCPU allocator.
#[no_mangle]
pub extern "C" fn virtual_free(allocator: &'static mut Valloc, ptr: *mut ()) {
    match allocator.free::<()>(unsafe{SmartPointer::new_unchecked(ptr.cast())}) {
        Ok(_) => (),
        Err(e) => panic!("{e}"),
    }
}

/// Resizes the memory block pointed to by `ptr` to the specified size using the vCPU allocator.
/// Returns a raw pointer to the resized memory block.
#[no_mangle]
#[allow(unused_assignments)]
pub extern "C" fn virtual_realloc(allocator: &'static mut Valloc, mut ptr: *mut (), size: usize) -> *mut () {
    match allocator.realloc::<()>(unsafe{SmartPointer::new_unchecked(ptr.cast())}, size) {
        Ok(val) => {
            ptr = std::ptr::null_mut();
            val.as_ptr()
        },
        Err(e) => panic!("{e}"),
    }
}

/// Initializes the vCPU allocator with the specified size.
#[no_mangle]
pub extern "C" fn global_init(size: usize) {
    valloc_init(size);
}

/// Creates a new instance of a virtual allocator
#[no_mangle]
pub extern "C" fn new_valloc(mem: *mut (), len: usize) -> Box<Valloc<'static>> {
    let mem: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(mem.cast(), len) };
    Box::new(crate::allocator::Valloc::new(mem))
}

/// Frees the Virtual Allocator
#[no_mangle]
pub extern "C" fn free_valloc(allocator: Box<Valloc>) {
    drop(allocator);
}
