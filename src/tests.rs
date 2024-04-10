use crate::{allocator::{free, realloc, Valloc}, virtual_memory::Pointer};

#[test]
fn alloc_u8() {
    let mut a = Valloc::new(1024);

    // allocate space (in bytes) for a character
    let mut ptr = a.alloc::<u8>(1).unwrap();

    // write the character 'H' to our allocated memory
    *ptr = 'H' as u8;

    // read the character from our allocated memory
    assert_eq!(*ptr, 'H' as u8);

    free(&mut a, &mut ptr).unwrap();
}

#[test]
fn alloc_string() {
    let mut a = Valloc::new(1024);

    // allocate space (in bytes) for a character
    let mut ptr = a.alloc_type::<u8>(6).unwrap();
    
    let text = b"Hello".iter().map(|&x| x).collect::<Vec<u8>>();
    let len = text.len();

    a.write_buffer(&mut ptr, text).unwrap();

    let text = a.read_buffer(&ptr, len).unwrap();
    let text = text.iter().map(|c| *c as char).collect::<Vec<char>>();
    text.iter().for_each(|t| print!("{t}")); println!();

    // test if reading from a pointer that is out of bounds will return an error
    let result = a.read(&ptr.add(10));
    if let Err(e) = result {
        println!("error that we expected: {}", e);
    } else {
        eprintln!("Expected an out of bounds error!");
        assert!(false);
    }

    free(&mut a, &mut ptr).unwrap();
}

#[test]
fn alloc_struct() {
    // custom struct to test with
    struct Test { a: u32, b: u32, z: u32 }
    let mut a = Valloc::new(1024);
    
    let mut ptr = a.alloc_type(1).unwrap();

    // write the struct to our allocated memory
    *ptr = Test { a: 10, b: 20, z: 30 };

    assert_eq!(ptr.a, 10);
    assert_eq!(ptr.b, 20);
    assert_eq!(ptr.z, 30);
    
    free(&mut a, &mut ptr).unwrap();
}

#[test]
fn ptr_free() {
    let mut a = Valloc::new(10);
    let mut ptr = a.alloc::<u8>(1).unwrap();

    free(&mut a, &mut ptr).unwrap();

    // this should fail (we will handle the error without a panic though)
    let result = a.read(&ptr);
    if let Err(e) = result {
        println!("{}", e);
    } else {
        eprintln!("Expected an freed pointer error!");
        assert!(false);
    }
}

#[test]
fn ptr_double_free() {
    let mut a = Valloc::new(10);
    let mut ptr = a.alloc::<u8>(1).unwrap();

    free(&mut a, &mut ptr).unwrap();

    // this should fail (we will handle the error without a panic though)
    let result = free(&mut a, &mut ptr);
    if let Err(e) = result {
        println!("{}", e);
    } else {
        eprintln!("Expected an double free error!");
        assert!(false);
    }
}

#[test]
fn ptr_null() {
    let mut a = Valloc::new(10);
    let _ = a.alloc::<u8>(1).unwrap();
    // replace the pointer with a null pointer
    let ptr = Pointer::NULL;
    
    // now attempt to write to the null pointer
    let result = a.write(&ptr, 10);
    if let Err(e) = result {
        println!("{}", e);
    } else {
        eprintln!("Expected an double free error!");
        assert!(false);
    }
    
    // now attempt to read to the null pointer    
    let result = a.read(&ptr);
    if let Err(e) = result {
        println!("{}", e);
    } else {
        eprintln!("Expected an double free error!");
        assert!(false);
    }
}

#[test]
fn ptr_cast() {
    let mut a = Valloc::new(10);
    let mut ptr = a.alloc::<u8>(1).unwrap();

    // write value
    *ptr = 10;

    // read initial value
    let before = *ptr;

    // cast the pointer to a different type
    let mut ptr = ptr.cast::<u32>().unwrap();

    // read from the pointer
    let after = *ptr;

    assert_eq!(before as u32, after);
    assert_eq!(before,  after as u8);

    free(&mut a, &mut ptr).unwrap();
}

#[test]
fn ptr_deref() {
    let mut a = Valloc::new(10);
    let mut ptr = a.alloc::<u8>(1).unwrap();

    // write to the pointer using the deref trait
    *ptr = 10;

    // read from the pointer using the deref trait
    assert_eq!(*ptr, 10);

    free(&mut a, &mut ptr).unwrap();
}

#[test]
fn realloc_test() {
    let mut a = Valloc::new(1024);
    let mut ptr_a = a.alloc::<u8>(1).unwrap();

    // write to the pointer
    *ptr_a = 10;

    // our pointer is freed and reallocated here
    let mut ptr_b = realloc(&mut a, ptr_a, 30).unwrap();
    
    println!("Chunks AFTER: {:?}", a.chunks());

    // read value
    let v1 = *ptr_b;
    print!("v1: {v1}, ");
    let v2 = a.read(&ptr_b).unwrap();
    println!("v2: {v2}");

    // read the value from the pointer
    assert_eq!(v1, 10);
    assert_eq!(v2, 10);

    free(&mut a, &mut ptr_b).unwrap();
}

#[test]
fn realloc_fail() {
    let mut a = Valloc::new(1024);
    let ptr = a.alloc::<u8>(1).unwrap();
    // this should fail to allocate space (not enough space)
    let result = realloc(&mut a, ptr, 1000000);
    if let Err(e) = result {
        println!("{}", e);
    } else {
        eprintln!("Expected an allocation error!");
        assert!(false);
    }
}

#[test]
fn realloc_struct() {
    #[derive(Debug)]
    struct Test { a: u32, b: u32, z: u32 }
    let mut a = Valloc::new(1024);
    let mut ptr = a.alloc_type::<Test>(1).unwrap();
    // write to the pointer
    *ptr = Test { a: 10, b: 20, z: 30 };
    println!("Size: {}", std::mem::size_of::<Test>());
    let mut ptr = realloc(&mut a, ptr, 24).unwrap();
    println!("{:?}", *ptr);
    // read the value from the pointer
    assert_eq!(ptr.a, 10);
    assert_eq!(ptr.b, 20);
    assert_eq!(ptr.z, 30);
    free(&mut a, &mut ptr).unwrap();
}

#[test]
fn realloc_string() {
    let mut a = Valloc::new(1024);
    let mut iptr = a.alloc_type::<u8>(6).unwrap();
    
    let text = b"Hello".iter().map(|&x| x).collect::<Vec<u8>>();
    let len = text.len();
    
    a.write_buffer(&mut iptr, text).unwrap();
    
    let mut ptr = realloc(&mut a, iptr, len-3).unwrap();

    let text = a.read_buffer(&ptr, len-3).unwrap();
    let text = text.iter().map(|c| *c as char).collect::<String>();
    let text = text.as_str();
    println!("{text}");

    assert_eq!(text, "He");

    free(&mut a, &mut ptr).unwrap();
}
