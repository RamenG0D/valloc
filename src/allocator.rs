use crate::{pointer::Pointer, vmem::{VirtMemory, VirtMemoryChunk}};
use std::mem::size_of;

// global allocator
static mut ALLOCATOR: Option<Valloc> = None;

#[no_mangle]
/// Get a mutable reference to the global allocator
/// 
/// # Safety
/// 
/// The caller must ensure that the allocator is initialized before calling this function
/// 
/// # Example
/// 
/// ```
/// use valloc::allocator::get_allocator;
/// let allocator = get_allocator();
/// ```
/// 
/// # Returns
/// 
/// * `&'static mut Valloc` - A mutable reference to the global allocator
/// 
/// # Panics
/// 
/// This function will panic if the allocator is not initialized
pub fn get_allocator() -> &'static mut Valloc {
    unsafe{match ALLOCATOR {
        Some(ref mut allocator) => allocator,
        None => panic!("Allocator not initialized!")
    }}
}

#[no_mangle]
/// Initializes the allocator with the total memory size (in bytes)
/// 
/// # Arguments
/// 
/// * `total_mem_size` - The total memory size to allocate
/// 
/// # Safety
/// 
/// The caller must ensure that the allocator is initialized only once
/// 
/// 
pub extern fn valloc_init(total_mem_size: usize) {
    if unsafe{ALLOCATOR.is_some()} {
        panic!("Allocator already initialized!");
    } else {
        unsafe{ ALLOCATOR = Some(Valloc::new(total_mem_size)); }
    }
}

/// The Valloc struct represents a Virtual Memory Allocator.
/// It will be used to simulate the ability to allocate and deallocate memory.
/// and is used on a simple stack or heap allocated array of bytes.
/// 
/// # Fields
/// - `memory`: a Memory instance
/// - `chunks`: a vector of MemoryChunk instances
/// 
/// # Methods
/// - `alloc()`: allocate a new MemoryChunk instance
/// - `free()` : deallocate a MemoryChunk instance
/// - `read()` : read from the memory
/// - `write()`: write to the memory
#[derive(Debug)]
pub struct Valloc {
    memory: VirtMemory,
    chunks: Vec<VirtMemoryChunk>
}

impl Valloc {
    /// Create a new Kernel instance with a given amount of memory (in bytes)
    pub fn new(size: usize) -> Self {
        Self {
            memory: VirtMemory::new(size),
            chunks: Vec::new()
        }
    }

    /// Create a new Kernel instance from existing memory.
    /// 
    /// # Arguments
    /// 
    /// * `memory` - The existing memory to be used by the Kernel.
    /// 
    /// # Example
    /// 
    /// ```
    /// use valloc::allocator::Valloc;
    /// let memory = vec![0u8; 1024].into_boxed_slice();
    /// let kernel = Valloc::from_memory(memory);
    /// ```
    pub fn from_memory<T>(memory: T) -> Self 
        where T: Into<Box<[u8]>>
    {
        let memory = memory.into();
        Self {
            memory: VirtMemory::from_mem(memory),
            chunks: Vec::new()
        }
    }

    /// Get a reference to the virtual memory used by the Kernel.
    pub fn get_memory(&self) -> &VirtMemory {
        &self.memory
    }

    /// Get a reference to the chunks vector.
    pub fn chunks(&self) -> &Vec<VirtMemoryChunk> {
        &self.chunks
    }

    /// Allocate a new MemoryChunk instance.
    /// 
    /// This method checks if there is enough contiguous space in the memory to allocate the chunk.
    /// If there is enough space, it creates a new MemoryChunk instance and adds it to the chunks vector.
    /// 
    /// # Arguments
    /// 
    /// * `size` - The size of the chunk to be allocated, in bytes.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Pointer<T>)` - A pointer to the start of the allocated chunk if successful.
    /// * `Err(String)` - An error message if allocation fails.
    /// 
    /// # Note
    /// 
    /// This method allocates in bytes.
    pub fn alloc<T>(&mut self, size: usize) -> Result<Pointer<T>, String> {
        let size = match size {
            0 => return Err("Cannot allocate 0 bytes".to_string()),
            _ => size - 1
        };

        for (i, data) in self.memory.get_data().iter().enumerate() {
            if *data != 0 { continue; }
            let (start, end) = (i, i + size);

            // skip if chunk is already allocated
            if self.chunks.iter().any(|chunk| {
                let lower = chunk.lower_bound();
                let upper = chunk.upper_bound();
                start >= lower && start <= upper || 
                end >= lower && end <= upper
            }) {
                continue;
            }

            let mptr = self.memory.get_data().as_mut().as_mut_ptr();
            let mem_ptr = unsafe{mptr.add(start)};

            // create a new chunk
            let chunk = VirtMemoryChunk::from_data(
                mem_ptr, 
                start, 
                end
            );
            self.chunks.push(chunk);

            let ptr = unsafe{ std::mem::transmute::<*mut u8, *mut T>(mptr) };
            let ptr = std::ptr::slice_from_raw_parts_mut(ptr, size + 1);
            let ptr = unsafe{ptr.as_mut()}.expect("Failed to construct Pointer");
            let ptr = Pointer::new(ptr, start);

            return Ok(ptr);
        }
        
        Err(format!("Failed to allocate memory for size => {size}"))
    }

    /// Allocate a new MemoryChunk instance with a specific type.
    /// 
    /// This method automatically converts the allocation size from `n bytes` to `n * sizeof(Type)` bytes.
    /// 
    /// # Arguments
    /// 
    /// * `size` - The size of the chunk to be allocated, in number of elements of type T.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Pointer<T>)` - A pointer to the start of the allocated chunk if successful.
    /// * `Err(String)` - An error message if allocation fails.
    pub fn alloc_type<T>(&mut self, size: usize) -> Result<Pointer<T>, String> {
        self.alloc(size * std::mem::size_of::<T>())
    }

    /// Reallocate a MemoryChunk instance.
    /// 
    /// This method reallocates the memory for a given pointer to a new size.
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - The pointer to the memory chunk to be reallocated.
    /// * `new_size` - The new size of the memory chunk, in bytes.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Pointer<T>)` - A pointer to the reallocated memory chunk if successful.
    /// * `Err(String)` - An error message if reallocation fails.
    pub fn realloc<T>(&mut self, ptr: Pointer<T>, new_size: usize) -> Result<Pointer<T>, String> 
        where T: std::fmt::Debug + Copy
    {
        realloc(self, ptr, new_size)
    }

    /// Deallocate a MemoryChunk instance.
    /// 
    /// This method removes a MemoryChunk instance from the chunks vector.
    /// It behaves like the free() function in C and only accepts a pointer to the first element of the chunk.
    /// The memory is not zeroed out, so the data will still be there, but the chunk will be removed from the chunks vector.
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - A mutable reference to the pointer to the memory chunk to be deallocated.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If deallocation is successful.
    /// * `Err(String)` - An error message if deallocation fails.
    pub fn free<T>(&mut self, ptr: &mut Pointer<T>) -> Result<(), String> {
        free(self, ptr)
    }

    /// Read a value from memory.
    /// 
    /// This method takes a pointer and attempts to find the corresponding MemoryChunk which contains the address.
    /// It then reads the value from the address if found.
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - A reference to the pointer to the memory location to be read.
    /// 
    /// # Returns
    /// 
    /// * `Ok(T)` - The value read from memory if successful.
    /// * `Err(String)` - An error message if reading fails.
    pub fn read<T: Clone>(&self, ptr: &Pointer<T>) -> Result<T, String> {
        let addr = ptr.get_index()?;
        if let Some(chunk) = self.chunks.iter().find(move |chunk| {
            addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
        }) {
            return Ok(unsafe{chunk.read_unchecked(addr)});
        } else {
            Err(format!("Invalid read at address => {}", addr))
        }
    }

    /// Write a value to memory.
    /// 
    /// This method takes a pointer and attempts to find the corresponding MemoryChunk which contains the address.
    /// It then writes the value to the address if found.
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - A reference to the pointer to the memory location to be written.
    /// * `value` - The value to be written to memory.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If writing is successful.
    /// * `Err(String)` - An error message if writing fails.
    pub fn write<T>(&mut self, ptr: &Pointer<T>, value: T) -> Result<(), String> {
        let addr = match ptr.get_index() {
            Ok(addr) => addr,
            Err(e) => return Err(format!("Invalid Ptr Write: {e}"))
        };

        if let Some(chunk) = self.chunks.iter_mut().find(move |chunk| {
            addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
        }) {
            unsafe{chunk.write_unchecked(addr, value)};
            Ok(())
        } else {
            Err(format!("Invalid write at address => {}", addr))
        }
    }

    /// Write a buffer to memory.
    /// 
    /// This method writes a buffer of values to memory starting from the given pointer.
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - A reference to the pointer to the memory location to start writing the buffer.
    /// * `buffer` - The buffer of values to be written to memory.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If writing is successful.
    /// * `Err(String)` - An error message if writing fails.
    pub fn write_buffer<T: Copy>(&mut self, ptr: &Pointer<T>, buffer: *const T, len: usize) -> Result<(), String> {
        for i in 0..len {
            let ptr = *ptr + i;
            let val = unsafe{*buffer.wrapping_add(i)};
            self.write(&ptr, val)?;
        }
        Ok(())
    }

    /// Read a buffer from memory.
    /// 
    /// This method reads a buffer of values from memory starting from the given pointer.
    /// 
    /// # Arguments
    /// 
    /// * `ptr` - A reference to the pointer to the memory location to start reading the buffer.
    /// * `size` - The size of the buffer, in number of elements of type T.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Vec<T>)` - The buffer of values read from memory if successful.
    /// * `Err(String)` - An error message if reading fails.
    pub fn read_buffer<T: Copy>(&self, ptr: &Pointer<T>, size: usize) -> Result<Vec<T>, String> {
        let mut buffer = Vec::new();
        let ptr = *ptr;
        for i in 0..size {
            buffer.push(self.read(&(ptr + i))?);
        }
        buffer.shrink_to_fit();
        Ok(buffer)
    }
}

pub fn free<T>(vallocator: &mut Valloc, ptr: &mut Pointer<T>) -> Result<(), String> {
    let addr = match ptr.get_index() {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Invalid Ptr Free: {e}"))
    };

    if vallocator.chunks.iter().find(|chunk| {
        addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
    }).is_some() {
        // we need to properly compare the real address (internal array pointer)
        let ptr = ptr.address().unwrap().as_ptr() as usize;
        // find the chunk that contains the pointer
        vallocator.chunks.retain(|chunk| {
            let chunk_ptr = chunk.data() as usize;
            chunk_ptr != ptr
        });
    } else {
        return Err(format!("Failed to free memory at address => {addr}"));
    }

    Ok(())
}

pub fn realloc<T>(vallocator: &mut Valloc, mut ptr: Pointer<T>, nsize: usize) -> Result<Pointer<T>, String> 
    where T: std::fmt::Debug + Copy
{
    // check if the new size is 0
    if nsize == 0 { return Err("Cannot reallocate 0 bytes".to_string()); }
    // if the size is greater than the current amount of memory left
    if nsize > vallocator.memory.get_data().len() {
        return Err(format!("Failed to reallocate memory for size => {nsize}: Not enough memory left"));
    }
    // if new size is the same as the current size
    if nsize == ptr.address().unwrap().len() {
        return Ok(ptr);
    }

    // get the address (index in valloc memory) of the pointer
    let addr = ptr.get_index()?;
    // find the chunk that contains the pointer
    if let Some(chunk) = vallocator.chunks.iter_mut().find(move |chunk| {
        addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
    }).copied() {
        let chunk_size = chunk.upper_bound() - chunk.lower_bound();
        if nsize < chunk_size {
            // Okay...
            // we just deallocate the difference in sizes
            // and return the same pointer
            let diff = chunk_size - nsize;
            let (nchunk, nptr) = if diff > 0 {
                (
                    Some(VirtMemoryChunk::new(
                        &mut vallocator.memory.get_data()[chunk.lower_bound()..chunk.lower_bound() + nsize], 
                        chunk.lower_bound(), 
                        chunk.lower_bound() + nsize
                    )),
                    Some(Pointer::new(
                        unsafe{ std::mem::transmute::<&mut [u8], &mut [T]>(vallocator.memory.get_data()) }, 
                        chunk.lower_bound()
                    ))
                )
            } else {
                (None, None)
            };
            
            // get data from the old buffer
            let data = vallocator.read_buffer(&ptr, diff)?;

            // free old ptr
            vallocator.free(&mut ptr)?;

            if let Some(nchunk) = nchunk {
                vallocator.chunks.push(nchunk);
            } else {
                return Err(format!("Failed to reallocate memory: New size is greater than the current size"));
            }
            
            if let Some(ptr) = nptr {
                // write into new buffer
                vallocator.write_buffer(&ptr, data.as_ptr(), data.len())?;
                return Ok(ptr);
            } else {
                return Err(format!("Failed to reallocate memory for size => {nsize}: Couldn't find chunk that contains address => {addr}"));
            }
        }
        
        // allocate a spcae the size of our current chunk size + the new chunk size
        let mut new_ptr: Pointer<T> = vallocator.alloc(nsize)?;

        // read the data from the old ptr (store it to put it in the new ptr)
        let len = (chunk_size+1) / size_of::<T>();
        dbg!(chunk_size); dbg!(len);
        let old_data = vallocator.read_buffer(&ptr, len)?;

        // write the data to the new ptr
        for i in 0..old_data.len() {
            new_ptr[i] = old_data[i];
        }

        vallocator.write_buffer(&mut new_ptr, old_data.as_ptr(), old_data.len())?;
        
        // free the old ptr (Note: this does NOT zero out the memory, it just removes the chunk from the chunks vector, see free() for more on this)
        vallocator.free(&mut ptr)?;

        // were done return the new ptr
        return Ok(new_ptr);
    }
    Err(format!("Failed to reallocate memory for size => {nsize}: Couldn't find chunk that contains address => {addr}"))
}