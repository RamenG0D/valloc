use std::{cell::RefCell, fmt::Write, ops::{Deref, DerefMut, Index, IndexMut}, usize};

/// The Pointer type represents a pointer to a memory address
/// It also contricts the type of the data that is being pointed to to its Generic allowing for type safety.
/// 
/// the Type Generic is used to restrict / tell the Pointer what the data its pointing to is!
/// But it does NOT store the actual value of the data, it just tells the Pointer what the data is and its also used
/// elsewhere to ensure that the data being read/written correctly and the methods wont except invalid combonations pointers
/// and data types into methods that could cause undefined behavior.
#[derive(Clone, Copy)]
pub enum Pointer<T> {
    Pointer {
        address: *mut [T],
        index: usize
    },
    NULL
}

use std::ops::{Add, Sub};

impl<T> std::fmt::Debug for Pointer<T> 
    where T: std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pointer::Pointer {
                address,
                index
            } => {
                f.debug_struct("Pointer")
                    .field("address", address)
                    .field("index", index)
                    .field("Address_Range", &{
                        let len = unsafe{(**address).len()};
                        let index = *index;
                        index..(index + len)
                    })
                    .finish()
            },
            _ => {
                f.write_str("Pointer { NULL }")
            }
        }
    }
}

impl<T> Index<usize> for Pointer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.address().unwrap().get(index).expect("Failed to get the value of the pointer!")
    }
}

impl<T> IndexMut<usize> for Pointer<T> {

    fn index_mut(&mut self, index: usize) -> &mut T {
        self.address_mut().unwrap().get_mut(index).expect("Failed to get the value of the pointer!")
    }
}

impl<T, U> Add<U> for Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn add(self, rhs: U) -> Self::Output {
        let index = match self {
            Pointer::Pointer { index, .. } => index + rhs.into(),
            Pointer::NULL => 0
        };
        
        let address = self.address().unwrap();
        let address = address as *const [T];
        let address = address.cast_mut();

        Pointer::Pointer { address, index }
    }
}

impl<T, U> Sub<U> for Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn sub(self, rhs: U) -> Self::Output {
        let index = match self {
            Pointer::Pointer { index, .. } => index - rhs.into(),
            Pointer::NULL => 0
        };
        
        let address = self.address().unwrap();
        let address = address as *const [T];
        let address = address.cast_mut();

        Pointer::Pointer { address, index }
    }
}

impl<T, U> Add<U> for &Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn add(self, rhs: U) -> Self::Output {
        let index = match self {
            Pointer::Pointer { index, .. } => (*index) + rhs.into(),
            Pointer::NULL => 0
        };
        
        let address = self.address().unwrap();
        let address = address as *const [T];
        let address = address.cast_mut();

        Pointer::Pointer { address, index }
    }
}

impl<T, U> Sub<U> for &Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn sub(self, rhs: U) -> Self::Output {
        let index = match self {
            Pointer::Pointer { index, .. } => (*index) - rhs.into(),
            Pointer::NULL => 0
        };
        
        let address = self.address().unwrap();
        let address = address as *const [T];
        let address = address.cast_mut();

        Pointer::Pointer { address, index }
    }
}

/*impl<T, U> Add<U> for &mut Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn add(self, rhs: U) -> Self::Output {
        let index = match self {
            Pointer::Pointer { index, .. } => (*index) + rhs.into(),
            Pointer::NULL => 0
        };
        
        let address = self.address().unwrap();
        let address = address as *const [T];
        let address = address.cast_mut();

        Pointer::Pointer { address, index }
    }
}

impl<T, U> Sub<U> for &mut Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn sub(self, rhs: U) -> Self::Output {
        let index = match self {
            Pointer::Pointer { index, .. } => (*index) - rhs.into(),
            Pointer::NULL => 0
        };
        
        let address = self.address().unwrap();
        let address = address as *const [T];
        let address = address.cast_mut();

        Pointer::Pointer { address, index }
    }
}*/

impl<T> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Pointer::Pointer { address, index } => {
                unsafe{(*address).as_ref()}.get(index).expect("Failed to get the value of the pointer!")
            },
            Pointer::NULL => panic!("Attempted to dereference a NULL pointer")
        }
    }
}

impl<T> DerefMut for Pointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self {
            Pointer::Pointer { address, index } => {
                let ptr = unsafe{(*address).as_mut()};
                &mut ptr[index]
            },
            Pointer::NULL => panic!("Attempted to dereference a NULL pointer")
        }
    }
}

impl<T> Pointer<T> {
    pub fn new(address: &mut [T], index: usize) -> Self {
        Pointer::Pointer { address, index }
    }

    pub fn address(&self) -> Result<&[T], String> {
        match self {
            Pointer::Pointer { address, .. } => Ok(unsafe{(*address).as_ref()}.unwrap()),
            Pointer::NULL => Err("Attempted to get the address of a NULL pointer".to_string())
        }
    }

    pub fn address_mut(&mut self) -> Result<&mut [T], String> {
        match self {
            Pointer::Pointer { address, .. } => Ok(unsafe{(*address).as_mut()}.unwrap()),
            Pointer::NULL => Err("Attempted to get the address of a NULL pointer".to_string())
        }
    }
    
    pub fn get_index(&self) -> Result<usize, String> {
        match self {
            Pointer::Pointer { index, .. } => Ok(*index),
            Pointer::NULL => Err("Attempted to get the index of a NULL pointer".to_string())
        }
    }

    #[inline(always)]
    pub fn cast<N>(self) -> Result<Pointer<N>, String> {
        match self {
            Pointer::Pointer { address, index } => {
                let addr = unsafe{ (*address).as_mut() }.as_mut_ptr() as *mut N;
                let addr = unsafe { std::slice::from_raw_parts_mut(addr, (*address).len()) };
                Ok(Pointer::new(addr, index))
            },
            Pointer::NULL => Err("Attempted to cast a NULL pointer".to_string())
        }
    }
}

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
        let address = address * std::mem::size_of::<T>();
        let data = self.data.add(address) as *const T;

        data.read()
    }

    /// Write a byte to the memory chunk at the given address
    /// may panic b/c bounds checking is not done
    pub unsafe fn write_unchecked<T>(&mut self, address: usize, value: T) {
        let address = address * std::mem::size_of::<T>();
        let data = self.data.add(
            address
        ) as *mut T;
        
        data.write(value);
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
