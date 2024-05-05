use crate::allocator::{SmartPointer, Valloc};
use std::mem::size_of;

#[test]
fn alloc_u8() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
    let mut ptr = allocator.alloc::<u8>(1).unwrap();
    assert!(!ptr.is_null());

    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(&mut ptr).unwrap();
}

#[test]
fn alloc_string() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<str>(13).unwrap();
    assert!(!ptr.is_null());
    ptr.write(String::from("Hello, World!").as_mut_str());
    assert_eq!(&*ptr, "Hello, World!");

    allocator.free(&mut ptr).unwrap();
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

    impl From<*mut u8> for TestStruct {
        fn from(ptr: *mut u8) -> Self {
            let ptr = ptr as *mut TestStruct;
            unsafe { (*ptr).clone() }
        }
    }

    let mut ptr = allocator.alloc::<TestStruct>(size_of::<TestStruct>()).unwrap();
    assert!(!ptr.is_null());

    *ptr = TestStruct {
        a: 1,
        b: 2,
        c: 3,
    };

    assert_eq!(ptr.a, 1);
    assert_eq!(ptr.b, 2);
    assert_eq!(ptr.c, 3);

    allocator.free(&mut ptr).unwrap();
}

#[test]
fn ptr_free() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<u8>(13).unwrap();
    assert!(!ptr.is_null());

    // Free the pointer
    allocator.free(&mut ptr).unwrap();
}

#[test]
fn ptr_double_free() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<u8>(1).unwrap();
    assert!(!ptr.is_null());

    // Free the pointer
    allocator.free(&mut ptr).unwrap();

    // Free the pointer again
    assert!(allocator.free(&mut ptr).is_err());
}

#[test]
fn ptr_null() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);
    let mut ptr = allocator.alloc::<u8>(1).unwrap();
    assert!(!ptr.is_null());

    // Free the pointer
    allocator.free(&mut ptr).unwrap();

    // Check if the pointer is null
    assert!(ptr.is_null());
}

#[test]
fn ptr_cast() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<u16>(size_of::<u16>()).unwrap();
    assert!(!ptr.is_null());
    
    unsafe {
        let ptr = ptr.ptr().cast::<u8>();
        *ptr = 1;
        assert_eq!(*ptr, 1);
    }

    allocator.free(&mut ptr).unwrap();
}

#[test]
fn realloc_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak(), 1024);

    let mut ptr = allocator.alloc::<[u8]>(1).unwrap();
    assert!(!ptr.is_null());

    {
        let ptr = &mut (*ptr);
        ptr[0] = 1;
        assert_eq!(ptr[0], 1);
    }

    let mut ptr = allocator.realloc(&mut ptr, 2).unwrap();
    assert!(!ptr.is_null());

    {
        let ptr = &mut (*ptr);
        ptr[1] = 2;
        assert_eq!(ptr[0], 1);
        assert_eq!(ptr[1], 2);
    }

    allocator.free(&mut ptr).unwrap();
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
