use crate::allocator::{global_allocator, valloc_init, Valloc};
use std::mem::size_of;

#[test]
fn custom_vec() {
    // init the global valloc
    valloc_init(1024);
    let mut test = Vec::new_in(global_allocator());
    

    test.push(4u8);
    test.push(5u8);
    test.push(test[0] + test[1]);

    assert_eq!(test[0], 4);
    assert_eq!(test[1], 5);
    assert_eq!(test[2], 9);
}

#[test]
fn alloc_clousre() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let mut ptr = allocator.alloc::<&dyn Fn(i8, i8) -> u8>(size_of::<&dyn Fn(i8, i8) -> u8>()).unwrap();
    *ptr = &|a, b| { (a + b) as u8 };

    assert_eq!(ptr(1, 2), 3);

    allocator.free(ptr).unwrap();
}

// #[test]
// fn alloc_u8() {
//     let mut allocator = Valloc::from(vec![0; 1024]);
//     let mut ptr = if let Ok(p) = allocator.alloc::<u8>(1) {
//         p
//     } else {
//         panic!("Failed to allocate memory");
//     };

//     *ptr = 1;
//     assert_eq!(*ptr, 1);

//     allocator.free(ptr).unwrap();
// }

#[test]
fn alloc_string() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    // Allocate a new String
    let mut ptr = allocator.alloc::<String>(13).unwrap();

    *ptr = "Hello, World!".to_string();

    assert_eq!(*ptr, "Hello, World!");

    allocator.free(ptr).unwrap();
}

#[test]
fn alloc_struct() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());
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
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let ptr = allocator.alloc::<u8>(13).unwrap();

    // Free the pointer
    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_cast() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let ptr = allocator.alloc::<u16>(size_of::<u16>()).unwrap();

    let mut ptr = ptr.cast::<u8>();
    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_cast_small_to_large() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let ptr = allocator.alloc::<u8>(size_of::<u8>()).unwrap();

    let mut ptr = ptr.cast::<u16>();
    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let mut ptr = allocator.alloc::<[u8]>(1).unwrap();
    ptr[0] = 1;
    assert_eq!(ptr[0], 1);

    let mut ptr = allocator.realloc(ptr, 2).unwrap();

    ptr[1] = 2;
    assert_eq!(ptr[0], 1);
    assert_eq!(ptr[1], 2);

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_fail() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let mut ptr = allocator.alloc::<[u8]>(1).unwrap();
    ptr[0] = 1;
    assert_eq!(ptr[0], 1);

    // there will be an error here so we never allocate the new memory
    let ptr = allocator.realloc(ptr, 0);
    assert!(ptr.is_err());
}

#[test]
fn realloc_struct() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());
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

    // create a new pointer with double the size (now we have a pointer to 2 TestStructs)
    let mut ptr = allocator.realloc(ptr, size_of::<TestStruct>() * 2).unwrap();

    ptr[1] = TestStruct {
        a: 4,
        b: 5,
        c: 6,
    };

    // assert that the old values are still there
    assert_eq!(ptr[0].a, 1);
    assert_eq!(ptr[0].b, 2);
    assert_eq!(ptr[0].c, 3);
    // assert the new struct has the correct values
    assert_eq!(ptr[1].a, 4);
    assert_eq!(ptr[1].b, 5);
    assert_eq!(ptr[1].c, 6);

    allocator.free(ptr).unwrap();
}

#[test]
fn alloc_array_chars() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    // Allocate a new String (char array)
    let ptr = allocator.alloc::<[char]>(13).unwrap();

    const S: &str = "Hello, World!";

    // Copy the string into the allocated memory
    unsafe {
        std::ptr::copy(S.as_bytes().iter().map(|x| *x as char).collect::<Vec<char>>().as_ptr(), ptr.as_ptr() as *mut char, S.len());
    }

    // compare each character to the string
    for (i, c) in S.chars().enumerate() {
        assert_eq!(ptr[i], c);
    }

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_string() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let mut ptr = allocator.alloc::<String>(size_of::<String>()).unwrap();
    *ptr = "Hello, World!".to_string();
    assert_eq!(*ptr, "Hello, World!");

    let mut ptr = allocator.realloc(ptr, size_of::<String>() * 2).unwrap();
    ptr[1] = "Hello, World! times 2 :)".to_string();

    assert_eq!(ptr[0], "Hello, World!");
    assert_eq!(ptr[1], "Hello, World! times 2 :)");
}

#[test]
fn single_ptr_stress_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let mut ptr = allocator.alloc::<u8>(1024).unwrap();
    for i in 0..1024 {
        ptr[i] = i as u8;
    }

    for i in 0..1024 {
        assert_eq!(ptr[i], i as u8);
    }

    allocator.free(ptr).unwrap();
}

#[test]
fn many_ptr_stress_test() {
    let mut allocator = Valloc::new(vec![0; 1024].leak());

    let mut ptrs = Vec::new();
    for _ in 0..100 {
        let mut ptr = allocator.alloc::<u8>(1).unwrap();
        *ptr = 1;
        ptrs.push(ptr);
    }

    for ptr in ptrs {
        allocator.free(ptr).unwrap();
    }
}
