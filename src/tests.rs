use crate::allocator::Allocator;
use std::mem::size_of;

#[test]
fn alloc_u8() {
    let mut allocator = Allocator::new(1024);

    let mut ptr = match allocator.alloc::<u8>(4) {
        Ok(ptr) => ptr,
        Err(_) => panic!("Failed to allocate memory"),
    };

    *ptr = 1;

    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn alloc_string() {
    let mut allocator = Allocator::new(1024);

    // Allocate a new String
    let mut ptr = allocator.alloc::<String>(13).unwrap();

    *ptr = "Hello, World!".to_string();

    assert_eq!(*ptr, "Hello, World!");

    println!("{}", *ptr);

    allocator.free(ptr).unwrap();
}

#[test]
fn alloc_struct() {
    let mut allocator = Allocator::new(1024);
    #[derive(Debug, Clone)]
    struct TestStruct {
        a: u8,
        b: u16,
        c: u32,
    }

    let mut ptr = allocator
        .alloc::<TestStruct>(size_of::<TestStruct>())
        .unwrap();

    *ptr = TestStruct { a: 1, b: 2, c: 3 };

    assert_eq!(ptr.a, 1);
    assert_eq!(ptr.b, 2);
    assert_eq!(ptr.c, 3);

    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_free() {
    let mut allocator = Allocator::new(1024);

    let ptr = allocator.alloc::<u8>(13).unwrap();

    // Free the pointer
    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_cast() {
    let mut allocator = Allocator::new(1024);

    let ptr = allocator.alloc::<u16>(size_of::<u16>()).unwrap();

    let mut ptr = ptr.cast::<u8>();
    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn ptr_cast_small_to_large() {
    let mut allocator = Allocator::new(1024);

    let ptr = allocator.alloc::<u8>(size_of::<u8>()).unwrap();

    let mut ptr = ptr.cast::<u16>();
    *ptr = 1;
    assert_eq!(*ptr, 1);

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_test() {
    let mut allocator = Allocator::new(1024);

    let mut ptr = allocator.alloc::<[u8]>(1).unwrap();
    ptr[0] = 1;
    assert_eq!(ptr[0], 1);

    let mut ptr = allocator.realloc::<[u8]>(ptr, 2).unwrap();

    ptr[1] = 2;
    assert_eq!(ptr[0], 1);
    assert_eq!(ptr[1], 2);

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_fail() {
    let mut allocator = Allocator::new(1024);

    let mut ptr = allocator.alloc::<[u8]>(1).unwrap();
    ptr[0] = 1;
    assert_eq!(ptr[0], 1);

    // there will be an error here so we never allocate the new memory
    let ptr = allocator.realloc(ptr, usize::MAX /* we cannot allocate usize::MAX bytes when our memory is only 1024 bytes long silly :P */);
    assert!(ptr.is_err());
}

#[test]
fn realloc_struct() {
    let mut allocator = Allocator::new(1024);
    #[derive(Debug, Clone)]
    struct TestStruct {
        a: u8,
        b: u16,
        c: u32,
        tmp: &'static str
    }

    let mut ptr = allocator
        .alloc::<TestStruct>(size_of::<TestStruct>())
        .unwrap();

    *ptr = TestStruct { a: 1, b: 2, c: 3, tmp: "Hello, World!" };

    assert_eq!(ptr.a, 1);
    assert_eq!(ptr.b, 2);
    assert_eq!(ptr.c, 3);

    // create a new pointer with double the size (now we have a pointer to 2 TestStructs)
    let ptr = allocator.realloc::<TestStruct>(ptr.cast(), size_of::<TestStruct>() * 2).unwrap();
    // Warning! You can cast the pointer to any fixed size array BUT you should attempt not to as it is not checked
    // but the result of accesing out of bound appears to be, well lets hear it from the dev:
    // "WTF HOW... WHY IS THIS WORKING, HOW DID I NOT GET A SEGFAULT, AND WHY IS THE VALUE JUST A THE STRUCT WITH ALL MEMBERS INITED WITH A ZERO VALUE"
    // - Update
    // "It got worse... I tested with a &'static str and it is uninitialized memory :("
    // - Update
    // "It's even worse than I thought... again... I now realize that if you just read the memory past the bounds of the array anything past the bounds is just straight up uninitialized memory (like `C``)"
    // TODO: See if we can fix this
    let mut ptr = ptr.cast::<[TestStruct; 2]>();

    ptr[1] = TestStruct { a: 4, b: 5, c: 6, tmp: "Hello, World! x2" };

    // debug print the pointers
    for p in ptr.iter() {
        println!("{:?}", p);
    }

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
    let mut allocator = Allocator::new(1024);

    const S: &str = "Hello, World!";

    // Allocate a new String (char array)
    let mut ptr = allocator.alloc::<[char]>(S.len()+1).unwrap();

    // Copy the string into the allocated memory
    for (i, c) in S.chars().enumerate() {
        ptr[i] = c;
    }

    // compare each character to the string
    for (i, c) in S.chars().enumerate() {
        assert_eq!(ptr[i], c);
    }

    let nsize = S.len() - 2;
    let ptr = allocator.realloc(ptr, nsize).unwrap();

    for (i, c) in S.chars().enumerate() {
        assert_eq!(ptr[i], c);
        println!("{}", ptr[i]);
    }

    allocator.free(ptr).unwrap();
}

#[test]
fn multi_alloc_multi_free() {
    let mut allocator = Allocator::new(1024);

    for _ in 0..100 {
        // allocate a pointer then wite a value to it, then free it
        // do this 2 per iteration
        let mut ptr = allocator.alloc::<u8>(1).unwrap();
        *ptr = 1;
        allocator.free(ptr).unwrap();

        let mut ptr = allocator.alloc::<u8>(1).unwrap();
        *ptr = 1;
        allocator.free(ptr).unwrap();
    }

    // now allocate a large pointer write to it and check the value
    let mut ptr = allocator.alloc::<[u8]>(400).unwrap();
    for i in 0..400 {
        ptr[i] = i as u8;
    }
    for i in 0..400 {
        assert_eq!(ptr[i], i as u8);
    }

    allocator.free(ptr).unwrap();
}

#[test]
fn realloc_string() {
    let mut allocator = Allocator::new(1024);

    let mut ptr = allocator.alloc::<String>(size_of::<String>()).unwrap();
    *ptr = "Hello, World!".to_string();
    assert_eq!(*ptr, "Hello, World!");

    let mut ptr = allocator.realloc::<[String; 2]>(ptr.cast(), size_of::<String>() * 2).unwrap();
    ptr[1] = "Hello, World! times 2 :)".to_string();

    assert_eq!(ptr[0], "Hello, World!");
    assert_eq!(ptr[1], "Hello, World! times 2 :)");
}

#[test]
fn single_ptr_stress_test() {
    let mut allocator = Allocator::new(1024);

    let mut ptr = allocator.alloc::<[u8]>(1024).unwrap();
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
    let mut allocator = Allocator::new(1024);

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
