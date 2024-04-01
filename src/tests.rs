use crate::{allocator::Valloc, virtual_memory::Pointer};

#[test]
fn alloc_string() {
    let mut a = Valloc::new(1024);

    // allocate space (in bytes) for a character
    // NOTE! 
    let mut ptr = a.alloc_type::<u8>(6).unwrap();
    // this is the other method (slightly more annoying) to write to the memory (individual bytes)
    /* write the character 'H' to our allocated memory
     * kernel.write(&ptr,          'H').unwrap();
     * kernel.write(&(ptr.add(1)), 'e').unwrap();
     * kernel.write(&(ptr.add(2)), 'l').unwrap();
     * kernel.write(&(ptr.add(3)), 'l').unwrap();
     * kernel.write(&(ptr.add(4)), 'o').unwrap();
     */
    let text = b"Hello".iter().map(|&x| x).collect::<Vec<u8>>();
    a.write_buffer(&ptr, text).unwrap();

    // read the character from our allocated memory
    let mut offset = 0;
    while let Ok(v) = a.read(&ptr.add(offset).unwrap()) {
        print!("{}", v as char);
        offset += 1;
    }
    println!();

    // test if reading from a pointer that is out of bounds will return an error
    let result = a.read(&ptr.add(10).unwrap());
    if let Err(e) = result {
        println!("EXPECTED ERROR: {}", e);
    } else {
        eprintln!("Expected an out of bounds error!");
        assert!(false);
    }

    a.free(&mut ptr).unwrap();
}

#[test]
fn alloc_struct() {
    // custom struct to test with
    struct Test { a: u32, b: u32 }
    
    let mut a = Valloc::new(1024);
    
    let mut ptr = a.alloc_type(1).unwrap();

    a.write(&ptr, Test { a: 10, b: 20 }).unwrap();
    let value = a.read(&ptr).unwrap();

    assert_eq!(value.a, 10);
    assert_eq!(value.b, 20);
    
    a.free(&mut ptr).unwrap();
}

#[test]
fn ptr_free() {
    let mut a = Valloc::new(10);
    let mut ptr = a.alloc::<u8>(1).unwrap();

    a.free(&mut ptr).unwrap();

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

    a.free(&mut ptr).unwrap();

    // this should fail (we will handle the error without a panic though)
    let result = a.free(&mut ptr);
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

    let _ = a.read(&ptr).unwrap();
}

#[test]
fn ptr_cast() {
    let mut a = Valloc::new(10);
    let ptr = a.alloc::<u8>(1).unwrap();

    // read initial value
    let before = a.read(&ptr).unwrap();

    // cast the pointer to a different type
    let mut ptr = ptr.cast::<u32>().unwrap();

    // read from the pointer
    let after = a.read(&ptr).unwrap();

    assert_eq!(before, after as u8);

    a.free(&mut ptr).unwrap();
}
