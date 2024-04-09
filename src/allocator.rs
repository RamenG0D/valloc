use crate::virtual_memory::{Pointer, VirtMemory, VirtMemoryChunk};

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

    pub fn from_memory<'a, T>(memory: T) -> Self 
        where T: Into<&'a [u8]>
    {
        let mem: &[u8] = memory.into();
        Self {
            memory: VirtMemory::from_mem(mem.as_ptr() as *mut u8, mem.len()),
            chunks: Vec::new()
        }
    }

    pub fn get_memory(&self) -> &VirtMemory {
        &self.memory
    }

    /// Allocates a new MemoryChunk instance, checking if there is enough CONTIGUOUS space in the memory to allocate the chunk.
    /// If there is enough space, it will create a new MemoryChunk instance and add it to the chunks vector.
    /// Returns a Pointer to the start of the chunk if successful, otherwise returns an error message.
    /// 
    /// # Note
    /// THIS ALLOCATES IN BYTES!
    pub fn alloc<T>(&mut self, size: usize) -> Result<Pointer<T>, String> {
        if size == 0 { return Err("Cannot allocate 0 bytes".to_string()); }
        let size = size - 1;

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

            // check if we have enough space to allocate
            if end >= self.memory.get_data().len() {
                return Err(format!("Failed to allocate memory for size => {size}"));
            }

            let ptr = &mut self.memory.get_mut_data()[start..start + size];

            // create a new chunk
            let chunk = VirtMemoryChunk::new(
                ptr, 
                start, 
                start + size
            );
            self.chunks.push(chunk);

            let ptr = ptr as *mut [u8] as *mut T;
            return Ok(Pointer::new(ptr, start));
        }
        
        Err(format!("Failed to allocate memory for size => {}", size))
    }

    pub fn alloc_type<T>(&mut self, size: usize) -> Result<Pointer<T>, String> {
        // this is to automatically convert the alloc size from
        // n bytes
        // to
        // n * sizeof(Type) bytes
        self.alloc(size * std::mem::size_of::<T>())
    }

    pub fn realloc<T>(&mut self, ptr: &mut Pointer<T>, nsize: usize) -> Result<Pointer<T>, String> {
        realloc(self, ptr, nsize)
    }

    /// Generally attempts to deallocate a MemoryChunk instance by removing it from the chunks vector.
    /// BUT this method aims to behave like the free() function in C, so it will NOT accept a Pointer to the chunk to be deallocated
    /// if it doesn't point to the first element of the chunk. (only works if the pointer is the same as what was returned by alloc())
    /// It also wont zero out the memory, so the data will still be there, but the chunk will be removed from the chunks vector.
    pub fn free<T>(&mut self, ptr: &mut Pointer<T>) -> Result<(), String> {
        free(self, ptr)
    }

    /// Takes a given Pointer and attempts to find the corresponding MemoryChunk which contains the address (within its range [upper..lower])
    pub fn read<T>(&self, ptr: &Pointer<T>) -> Result<T, String> {
        let addr = ptr.index()?;
        if let Some(chunk) = self.chunks.iter().find(move |chunk| {
            addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
        }) {
            return chunk.read(addr);
        }
        Err(format!("Invalid read at address => {}", addr))
    }

    /// Takes a given Pointer and attempts to find the corresponding MemoryChunk which contains the address (within its range [upper..lower])
    /// and writes the value to the address if found
    pub fn write<T>(&mut self, ptr: &Pointer<T>, value: T) -> Result<(), String> {
        let addr = ptr.index()?;
        if let Some(chunk) = self.chunks.iter_mut().find(move |chunk| {
            addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
        }) {
            return chunk.write(addr, value);
        }
        Err(format!("Invalid write at address => {}", addr))
    }

    /// QOL function to write a buffer into mem
    /// of course this is checked for bounds
    /// but may be unsafe still
    pub fn write_buffer<T>(&mut self, ptr: &Pointer<T>, buffer: Vec<T>) -> Result<(), String> {
        let mut i = 0;
        for val in buffer {
            self.write(&ptr.add(i), val)?;
            i += 1;
        }
        Ok(())
    }

    /// QOL function to read a buffer from mem
    /// of course this is checked for bounds
    /// but may be unsafe still
    pub fn read_buffer<T>(&self, ptr: &Pointer<T>, size: usize) -> Result<Vec<T>, String> {
        let mut buffer = Vec::new();
        for i in 0..size {
            buffer.push(
                self.read(&ptr.add(i))?
            );
        }
        Ok(buffer)
    }
}

pub fn free<T>(vallocator: &mut Valloc, ptr: &mut Pointer<T>) -> Result<(), String> {
    let addr = ptr.index()?;

    if vallocator.chunks.iter().find(|chunk| {
        addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
    }).is_none() {
        return Err(format!("Failed to free memory at address => {}", addr));
    }

    // find the chunk that contains the pointer
    vallocator.chunks.retain(|chunk| {
        if addr <= chunk.upper_bound() && addr >= chunk.lower_bound() { false } else { true }
    });

    Ok(())
}

pub fn realloc<T>(vallocator: &mut Valloc, ptr: &mut Pointer<T>, nsize: usize) -> Result<Pointer<T>, String> {
    let addr = ptr.index()?;
    // find the chunk that contains the pointer
    if let Some(chunk) = vallocator.chunks.iter_mut().find(move |chunk| {
        addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
    }) {
        let chunk_size = chunk.upper_bound() - chunk.lower_bound();
        if chunk_size >= nsize {
            let ptr = Pointer::new(
                ptr.address().unwrap(),
                ptr.index().unwrap()
            );
            return Ok(ptr);
        }
        
        let new_ptr = vallocator.alloc(nsize)?;
        let buffer = vallocator.read_buffer(ptr, chunk_size)?;
        
        vallocator.free(ptr)?;

        vallocator.write_buffer(&new_ptr, buffer)?;
        
        return Ok(new_ptr);
    }
    Err(format!("Failed to reallocate memory for size => {}", nsize))
}