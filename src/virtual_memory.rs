use std::ops::{Deref, DerefMut};

/// The Pointer type represents a pointer to a memory address
/// It also contricts the type of the data that is being pointed to to its Generic allowing for type safety.
/// 
/// the Type Generic is used to restrict / tell the Pointer what the data its pointing to is!
/// But it does NOT store the actual value of the data, it just tells the Pointer what the data is and its also used
/// elsewhere to ensure that the data being read/written correctly and the methods wont except invalid combonations pointers
/// and data types into methods that could cause undefined behavior.
#[derive(Debug, Clone, Copy)]
pub enum Pointer<T> {
    Pointer {
        address: Option<*mut T>,
        index: usize,
        // phantom: PhantomData<T>,
        // chunk: Option<*const VirtMemoryChunk>
    },
    NULL
}

impl<T> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.address().unwrap().as_ref().unwrap() }
    }
}

impl<T> DerefMut for Pointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.address().unwrap().as_mut().unwrap() }
    }
}

impl<T> Pointer<T> {
    pub fn from_index(index: usize) -> Self {
        Pointer::Pointer { address: None, index }
    }

    pub fn new(address: *mut T, index: usize) -> Self {
        Pointer::Pointer { address: Some(address), index }
    }

    #[inline]
    pub fn address(&self) -> Result<*mut T, String> {
        match self {
            Pointer::Pointer { address, .. } => {
                Ok(address.unwrap())
            },
            Pointer::NULL => Err("Attempted to get the address of a NULL pointer".to_string())
        }
    }

    #[inline(always)]
    pub fn cast<N>(self) -> Result<Pointer<N>, String> {
        match self {
            Pointer::Pointer { address, index } => {
                Ok(Pointer::Pointer { address: address.map(|addr| addr as *mut N), index })
            },
            Pointer::NULL => Err("Attempted to cast a NULL pointer".to_string())
        }
    }

    pub fn index(&self) -> Result<usize, String> {
        match self {
            Pointer::Pointer { index, .. } => Ok(*index),
            Pointer::NULL => Err("Attempted to get the index of a NULL pointer".to_string())
        }
    }

    #[inline]
    pub fn add(&self, offset: usize) -> Pointer<T> {
        let offset = offset * std::mem::size_of::<T>();
        let addr = self.address().unwrap();

        Pointer::new(
            addr.wrapping_add(offset), 
            self.index().unwrap() + offset
        )
    }

    #[inline]
    pub fn sub(&self, offset: usize) -> Pointer<T> {
        let offset = offset * std::mem::size_of::<T>();
        let addr = self.address().unwrap();

        Pointer::new(
            addr.wrapping_sub(offset), 
            self.index().unwrap() - offset
        )
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
    /// may panic if the address is out of bounds
    pub unsafe fn read_unchecked<T>(&self, address: usize) -> T {
        let address = address * std::mem::size_of::<T>();
        let data = self.data.add(address) as *const T;
        data.read()
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub unsafe fn write_unchecked<T>(&mut self, address: usize, value: T) {
        let address = address * std::mem::size_of::<T>();
        let data = self.data.add(
            address
        ) as *mut T;
        data.write(value);
    }

    /// Read a byte from the memory chunk at the given address
    pub fn read<T>(&self, address: usize) -> Result<T, String> {
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