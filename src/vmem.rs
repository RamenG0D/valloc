use std::{cell::RefCell, usize};

/// The Memory struct represents a block of memory.
/// It contains a fixed-size array of bytes.
/// It will be used to simulate the RAM of a computer.
/// 
/// # Fields
/// - `data`: a fixed-size array of bytes.
#[derive(Debug)]
pub struct VirtMemory { data: RefCell<Box<[u8]>> }

impl VirtMemory {
    /// Create a new Memory instance (size in bytes) with all bytes set to 0
    pub fn new(size: usize) -> Self {
        let ptr: Box<[u8]> = vec![0u8; size].into_boxed_slice();
        Self { data: RefCell::new(ptr) }
    }

    pub fn from_mem(mem: Box<[u8]>) -> Self {
        Self { data: RefCell::new(mem) }
    }

    /// Get a reference to the data
    #[inline(always)]
    pub fn get_data(&mut self) -> &mut [u8] {
        self.data.get_mut()
    }
}

/// The MemoryChunk struct represents a chunk of memory.
/// It will be used to simulate the ability to "Own" a part of the memory (Ex: like malloc in C)
/// and is just a way for the kernel to keep track of the memory that is being used.
/// 
/// # Fields
/// - `data`: a slice of bytes.
/// - `ptr`: a Pointer to the start of the chunk.
#[derive(Debug, Clone, Copy)]
pub struct VirtMemoryChunk {
    data: *mut u8,
    lower_bound: usize,
    upper_bound: usize
}

impl VirtMemoryChunk {
    pub fn new(data: &mut [u8], start: usize, end: usize) -> Self {
        Self::from_data(data.as_mut_ptr(), start, end)
    }

    pub fn from_data(data: *mut u8, start: usize, end: usize) -> Self {
        Self {
            data,
            lower_bound: start,
            upper_bound:   end,
        }
    }

    /// Get the upper bound of the chunk
    #[inline]
    pub fn upper_bound(&self) -> usize {
        self.upper_bound
    }

    /// Get the lower bound of the chunk
    #[inline]
    pub fn lower_bound(&self) -> usize {
        self.lower_bound
    }

    #[inline]
    pub fn data(&self) -> *mut u8 {
        self.data
    }
    
    pub unsafe fn read_ref<T>(&self, address: usize) -> Result<&T, String> {
        if address >= self.lower_bound && address <= self.upper_bound {
            let data = self.data.add(address) as *const T;
            Ok(data.as_ref().unwrap())
        } else {
            Err(format!("Out of bounds memory access at address => [ {address} ] for chunk with bounds [ {} - {} ]", self.lower_bound, self.upper_bound))
        }
    }

    pub unsafe fn read_mut<T>(&self, address: usize) -> Result<&mut T, String> {
        if address >= self.lower_bound && address <= self.upper_bound {
            let data = self.data.add(address) as *mut T;
            Ok(data.as_mut().unwrap())
        } else {
            Err(format!("Out of bounds memory access at address => [ {address} ] for chunk with bounds [ {} - {} ]", self.lower_bound, self.upper_bound))
        }
    }

    // Read a byte from the memory chunk at the given address
    /// may panic b/c bounds checking is not done
    pub unsafe fn read_unchecked<T>(&self, address: usize) -> T {
        let data = self.data.add(address) as *const T;
        data.read()
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic b/c bounds checking is not done
    pub unsafe fn write_unchecked<T>(&mut self, address: usize, value: T) {
        let data = self.data.add(address) as *mut T;
        *data = value;
    }

    /// Read a byte from the memory chunk at the given address
    pub fn read<T>(&self, address: usize) -> Result<T, String> 
        where T: std::fmt::Debug
    {
        if address >= self.lower_bound && address <= self.upper_bound {
            Ok(unsafe { self.read_unchecked(address) })
        } else {
            Err(format!("Out of bounds memory access at address => [ {address} ] for chunk with bounds [ {} - {} ]", self.lower_bound, self.upper_bound))
        }
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub fn write<T>(&mut self, address: usize, value: T) -> Result<(), String> {
        if address >= self.lower_bound && address <= self.upper_bound {
            unsafe { self.write_unchecked(address, value) }
            Ok(())
        } else {
            Err(format!("Out of bounds memory access at address => [ {address} ] for chunk with bounds [ {} - {} ]", self.lower_bound, self.upper_bound))
        }
    }
}
