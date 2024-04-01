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

            // create a new chunk
            let chunk = VirtMemoryChunk::new(
                &mut self.memory.get_mut_data()[start..start + size], 
                start, 
                start + size
            );
            self.chunks.push(chunk);
            return Ok(Pointer::new(start));
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

    pub fn realloc<T: std::fmt::Debug>(&mut self, ptr: &mut Pointer<T>, size: usize) -> Result<Pointer<T>, String> {
        // find the chunk that contains the pointer
        let p = {
            let mut t = None;
            for p in self.chunks.iter() {
                let addr = ptr.as_address()?;
                if addr <= p.upper_bound() && addr >= p.lower_bound() {
                    // we need to copy the data from the &VirtMemoryChunk to a copied VirtMemoryChunk in pp
                    t = Some(VirtMemoryChunk::new(
                        &mut self.memory.get_mut_data()[p.lower_bound()..p.upper_bound()], 
                        p.lower_bound(), 
                        p.upper_bound()
                    ));
                    break;
                }
            }
            t
        };
        if let Some(chunk) = p {
            // check if the new size is greater than the current size
            if size > chunk.upper_bound() - chunk.lower_bound() {
                // allocate a new chunk
                let new_ptr = self.alloc(size)?;
                // copy the data from the old chunk to the new chunk
                for i in 0..size {
                    let value = unsafe{chunk.read_unchecked(ptr.as_address()? + i)};
                    self.write(&(new_ptr.add(i)?), value)?;
                }
                // free the old chunk
                self.free(ptr)?;
                return Ok(new_ptr);
            }
        }
        Err(format!("Failed to reallocate memory for size => {}", size))
    }

    /// Generally attempts to deallocate a MemoryChunk instance by removing it from the chunks vector.
    /// BUT this method aims to behave like the free() function in C, so it will NOT accept a Pointer to the chunk to be deallocated
    /// if it doesn't point to the first element of the chunk. (only works if the pointer is the same as what was returned by alloc())
    /// It also wont zero out the memory, so the data will still be there, but the chunk will be removed from the chunks vector.
    pub fn free<T>(&mut self, ptr: &mut Pointer<T>) -> Result<(), String> {
        for (i, chunk) in self.chunks.iter().enumerate() {
            if ptr.as_address()? == chunk.lower_bound() {
                self.chunks.remove(i);
                // NULL's the pointer
                ptr.set_address(0)?;
                return Ok(());
            }
        }
        Err(format!("Invalid free at address => {}", ptr.as_address()?))
    }

    /// Takes a given Pointer and attempts to find the corresponding MemoryChunk which contains the address (within its range [upper..lower])
    pub fn read<T>(&self, ptr: &Pointer<T>) -> Result<T, String> {
        for chunk in self.chunks.iter() {
            let addr = ptr.as_address()?;
            if addr <= chunk.upper_bound() && addr >= chunk.lower_bound() {
                return chunk.read(addr);
            }
        }
        Err(format!("Invalid read at address => {}", ptr.as_address()?))
    }

    /// Takes a given Pointer and attempts to find the corresponding MemoryChunk which contains the address (within its range [upper..lower])
    /// and writes the value to the address if found
    pub fn write<T>(&mut self, ptr: &Pointer<T>, value: T) -> Result<(), String> {
        for chunk in self.chunks.iter_mut() {
            let addr = ptr.as_address()?;
            if addr <= chunk.upper_bound() && addr >= chunk.lower_bound() {
                return chunk.write(addr, value);
            }
        }
        Err(format!("Invalid write at address => {}", ptr.as_address()?))
    }

    /// QOL function to write a buffer into mem
    /// of course this is checked for bounds
    /// but may be unsafe still
    pub fn write_buffer<T: Clone>(&mut self, ptr: &Pointer<T>, buffer: Vec<T>) -> Result<(), String> {
        for i in 0..buffer.len() {
            let p = ptr.add(i)?;
            let data = buffer[i].clone();
            self.write(&p, data)?;
        }
        Ok(())
    }

    /// QOL function to read a buffer from mem
    /// of course this is checked for bounds
    /// but may be unsafe still
    pub fn read_buffer<T: Clone>(&self, ptr: &Pointer<T>, size: usize) -> Result<Vec<T>, String> {
        let mut buffer = Vec::new();
        for i in 0..size {
            buffer.push(self.read(&(ptr.add(i)?))?);
        }
        Ok(buffer)
    }
}