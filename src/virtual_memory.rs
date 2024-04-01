use std::marker::PhantomData;

pub enum Pointer<T> {
    Pointer(MemPointer<T>),
    NULL
}
type Result<T> = std::result::Result<T, String>;
impl<T> Pointer<T> {
    pub fn new(address: usize) -> Self {
        Self::Pointer(MemPointer::new(address))
    }

    fn non_null(&self) -> Result<&MemPointer<T>> {
        match self {
            Self::Pointer(ptr) => Ok(ptr),
            Self::NULL => {
                Err("Attempted to dereference a NULL pointer".to_string())
            }
        }
    }

    fn mut_non_null(&mut self) -> Result<&mut MemPointer<T>> {
        match self {
            Self::Pointer(ptr) => Ok(ptr),
            Self::NULL => {
                Err("Attempted to dereference a NULL pointer".to_string())
            }
        }
    }

    pub fn get_value(self) -> Result<MemPointer<T>> {
        match self {
            Self::Pointer(ptr) => Ok(ptr),
            Self::NULL => {
                Err("Attempted to dereference a NULL pointer".to_string())
            }
        }
    }

    pub fn as_address(&self) -> Result<usize> {
        let ptr = self.non_null()?;
        Ok( ptr.as_address() )
    }

    pub fn set_address(&mut self, address: usize) -> Result<()> {
        let ptr = self.mut_non_null()?;
        ptr.set_address(address);
        Ok(())
    }

    pub fn cast<U>(self) -> Result<Pointer<U>> {
        let ptr = self.get_value()?;
        Ok( Pointer::Pointer(ptr.cast::<U>()) )
    }

    pub fn add(&self, offset: usize) -> Result<Pointer<T>> {
        let ptr = self.non_null()?;
        Ok( Pointer::Pointer(ptr.add(offset)) )
    }

    pub fn sub(&self, offset: usize) -> Result<Pointer<T>> {
        let ptr = self.non_null()?;
        Ok( Pointer::Pointer(ptr.sub(offset)) )
    }

}

/// The Pointer type represents a pointer to a memory address
/// It also contricts the type of the data that is being pointed to to its Generic allowing for type safety.
/// 
/// the Type Generic is used to restrict / tell the Pointer what the data its pointing to is!
/// But it does NOT store the actual value of the data, it just tells the Pointer what the data is and its also used
/// elsewhere to ensure that the data being read/written correctly and the methods wont except invalid combonations pointers
/// and data types into methods that could cause undefined behavior.
#[derive(Debug, Clone, Copy)]
pub struct MemPointer<Type> {
    address: usize,
    _phantom: PhantomData<Type>
}
impl<Type> MemPointer<Type> {
    pub(self) fn new(address: usize) -> Self {
        Self { address, _phantom: PhantomData }
    }

    #[inline(always)]
    pub(self) fn as_address(&self) -> usize {
        self.address
    }

    #[inline(always)]
    pub(self) fn set_address(&mut self, address: usize) {
        self.address = address;
    }

    #[inline(always)]
    pub(self) fn cast<T>(self) -> MemPointer<T> {
        MemPointer::new(self.address)
    }

    #[inline(always)]
    pub(self) fn add(&self, offset: usize) -> MemPointer<Type> {
        let offset = offset * std::mem::size_of::<Type>();
        MemPointer::new(self.address + offset)
    }

    #[inline(always)]
    pub(self) fn sub(&self, offset: usize) -> MemPointer<Type> {
        let offset = offset * std::mem::size_of::<Type>();
        MemPointer::new(self.address - offset)
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

    // Read a byte from the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub unsafe fn read_unchecked<T>(&self, address: usize) -> T {
        let address = address + std::mem::size_of::<T>();
        let data = self.data.add(address) as *const T;
        data.read()
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub unsafe fn write_unchecked<T>(&mut self, address: usize, value: T) {
        let address = address + std::mem::size_of::<T>();
        let data = self.data.add(
            address
        ) as *mut T;
        data.write(value);
    }

    /// Read a byte from the memory chunk at the given address
    pub fn read<T>(&self, address: usize) -> Result<T> {
        if address >= self.lower_bound && address <= self.upper_bound {
            Ok(unsafe { self.read_unchecked(address) })
        } else {
            Err(format!("Out of bounds memory access at address => [ {address} ] for chunk with bounds [ {} - {} ]", self.lower_bound, self.upper_bound))
        }
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic if the address is out of bounds
    pub fn write<T>(&mut self, address: usize, value: T) -> Result<()> {
        if address >= self.lower_bound && address <= self.upper_bound {
            unsafe { self.write_unchecked(address, value) }
            Ok(())
        } else {
            Err(format!("Out of bounds memory access at address => [ {address} ] for chunk with bounds [ {} - {} ]", self.lower_bound, self.upper_bound))
        }
    }
}