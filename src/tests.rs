use crate::allocator::{SmartPointer, Valloc};
use std::mem::size_of;

#[test]
fn alloc_u8() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
    let mut ptr = allocator.alloc::<u8>(1).unwrap();

    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn alloc_string() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<&str>(13).unwrap();

    *ptr = "Hello, World!";

    assert_eq!(*ptr, "Hello, World!");

    allocator.free(ptr).unwrap();
}

#[test]
fn alloc_struct() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
    #[derive(Debug, Clone)]
    struct TestStruct {
        a: u8,
        b: u16,
        c: u32,
    }

    let mut ptr = allocator.alloc::<TestStruct>(size_of::<TestStruct>()).unwrap();

    *ptr = TestStruct {
        a: 1,
        b: 2,
        c: 3,
    };

    assert_eq!(ptr.a, 1);
    assert_eq!(ptr.b, 2);
    assert_eq!(ptr.c, 3);

    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_free() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<u8>(13).unwrap();

    // Free the pointer
    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_cast() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let ptr = allocator.alloc::<u16>(size_of::<u16>()).unwrap();

    let mut ptr = ptr.cast::<u8>();
    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<[u8]>(1).unwrap();
    (*ptr)[0] = 1;
    assert_eq!((*ptr)[0], 1);

    let mut ptr = allocator.realloc(ptr, 2).unwrap();

    ptr[1] = 2;
    assert_eq!(ptr[0], 1);
    assert_eq!(ptr[1], 2);

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_fail() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
}

#[test]
fn realloc_struct() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
}

#[test]
fn realloc_string() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
}

#[test]
fn single_ptr_stress_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
}

#[test]
fn many_ptr_stress_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
}
