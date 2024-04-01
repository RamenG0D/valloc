use crate::allocator::Valloc;

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
    while let Ok(v) = a.read(&ptr.add(offset)) {
        print!("{}", v as char);
        offset += 1;
    }
    println!();

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
