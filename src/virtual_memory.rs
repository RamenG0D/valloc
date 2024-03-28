use std::marker::PhantomData;
    
/// The Pointer type represents a pointer to a memory address
/// It also contricts the type of the data that is being pointed to to its Generic allowing for type safety.
/// 
/// the Type Generic is used to restrict / tell the Pointer what the data its pointing to is!
/// But it does NOT store the actual value of the data, it just tells the Pointer what the data is and its also used
/// elsewhere to ensure that the data being read/written correctly and the methods wont except invalid combonations pointers
/// and data types into methods that could cause undefined behavior.
#[derive(Debug, Clone, Copy)]
pub struct Pointer<Type> {
    address: usize,
    _phantom: PhantomData<Type>
}
impl<Type> Pointer<Type> {
    pub fn new(address: usize) -> Self {
        Self { address, _phantom: PhantomData }
    }

    #[inline(always)]
    pub fn as_address(&self) -> usize {
        self.address
    }

    #[inline(always)]
    pub fn set_address(&mut self, address: usize) {
        self.address = address;
    }

    #[inline(always)]
    pub fn cast<T>(&self) -> Pointer<T> {
        Pointer::new(self.address)
    }

    #[inline(always)]
    pub fn add(&self, offset: usize) -> Pointer<Type> {
        let offset = offset * std::mem::size_of::<Type>();
        Pointer::new(self.address + offset)
    }

    #[inline(always)]
    pub fn sub(&self, offset: usize) -> Pointer<Type> {
        let offset = offset * std::mem::size_of::<Type>();
        Pointer::new(self.address - offset)
    }
}

/// The Memory struct represents a block of memory.
/// It contains a fixed-size array of bytes.
/// It will be used to simulate the RAM of a computer.
/// 
/// # Fields
/// - `data`: a fixed-size array of bytes.
#[derive(Debug)]
pub struct VirtMemory { data: *mut u8, size: usize }

impl VirtMemory {
    /// Create a new Memory instance with all bytes set to 0
    pub fn new(size: usize) -> Self {
        let data = vec![0; size].into_boxed_slice();
        let data = Box::into_raw(data) as *mut u8;
        
        Self::from_mem(data, size)
    }

    pub fn from_mem(mem: *mut u8, size: usize) -> Self {
        Self { data: mem, size }
    }

    /// Get a reference to the data
    #[inline(always)]
    pub fn get_data(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data, self.size) }
    }

    /// Get a mutable reference to the data
    #[inline(always)]
    pub fn get_mut_data(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.data, self.size) }
    }
}

/// The MemoryChunk struct represents a chunk of memory.
/// It will be used to simulate the ability to "Own" a part of the memory (Ex: like malloc in C)
/// and is just a way for the kernel to keep track of the memory that is being used.
/// 
/// # Fields
/// - `data`: a slice of bytes.
/// - `ptr`: a Pointer to the start of the chunk.
#[derive(Debug)]
pub struct VirtMemoryChunk {
    data: *mut u8,
    lower_bound: usize,
    upper_bound: usize
}

impl VirtMemoryChunk {
    pub fn new(data: &mut [u8], start: usize, end: usize) -> Self {
        Self {
            data: data.as_mut_ptr() as *mut u8,
            lower_bound: start,
            upper_bound:   end,
        }
    }

    /// Get the upper bound of the chunk
    #[inline(always)]
    pub fn upper_bound(&self) -> usize {
        self.upper_bound
    }

    /// Get the lower bound of the chunk
    #[inline(always)]
    pub fn lower_bound(&self) -> usize {
        self.lower_bound
    }

    /// Read a byte from the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub unsafe fn read_unchecked<T>(&self, address: usize) -> T {
        let data = self.data.add(address);
        std::mem::transmute::<*mut u8, *mut T>(data).read()
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub unsafe fn write_unchecked<T>(&mut self, address: usize, mut value: T) {
        let value = std::mem::transmute::<*mut T, *mut u8>(&mut value);
        self.data.add(address).copy_from(value, std::mem::size_of::<T>());
    }

    /// Read a byte from the memory chunk at the given address
    pub fn read<T>(&self, address: usize) -> T {
        unsafe { self.read_unchecked(address) }
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub fn write<T>(&mut self, address: usize, value: T) {
        unsafe { self.write_unchecked(address, value); }
    }
}