use crate::virtual_memory::{Pointer, VirtMemory, VirtMemoryChunk};

struct BoxValue<T: ?Sized>(Box<T>);

impl Into<Box<[u8]>> for BoxValue<[u8]> {
    fn into(self) -> Box<[u8]> {
        self.0
    }
}

impl<T> TryFrom<&mut [T]> for BoxValue<[T]> {
    type Error = String;
    fn try_from(value: &mut [T]) -> Result<Self, Self::Error> {
        let ptr = unsafe{ Box::from_raw(value) };
        Ok(BoxValue(ptr))
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

    pub fn from_memory<T>(memory: T) -> Self 
        where T: Into<Box<[u8]>>
    {
        let memory = memory.into();
        Self {
            memory: VirtMemory::from_mem(memory),
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
        let size = if size == 0 { return Err("Cannot allocate 0 bytes".to_string()); } else if size == 1 { size } else { size - 1 };

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

            let m = self.memory.get_data().as_mut();
            //self.memory.get_mut_data().borrow();
            let ptr = &mut m[start..start + size];

            // create a new chunk
            let chunk = VirtMemoryChunk::new(
                ptr, 
                start, 
                start + size
            );
            self.chunks.push(chunk);

            let ptr = unsafe{ std::mem::transmute::<&mut [u8], &mut [T]>(ptr) };

            // let plen = ptr.len();
            // let p = ptr.as_mut_ptr() as *mut T;
            // let ptr = unsafe{ std::slice::from_raw_parts_mut(p, plen) };
            // let ptr = unsafe{ Box::from_raw(ptr) };

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

    pub fn realloc<T>(&mut self, ptr: Pointer<T>, new_size: usize) -> Result<Pointer<T>, String> 
        // where T: std::fmt::Debug
    {
        realloc(self, ptr, new_size)
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
            return Ok(unsafe{chunk.read_unchecked(addr)});
        }
        Err(format!("Invalid read at address => {}", addr))
    }

    /// Takes a given Pointer and attempts to find the corresponding MemoryChunk which contains the address (within its range [upper..lower])
    /// and writes the value to the address if found
    pub fn write<T>(&mut self, ptr: &Pointer<T>, value: T) -> Result<(), String> {
        let addr = ptr.index()?;
        if let Some(chunk) = self.chunks.iter_mut().find(|chunk| {
            addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
        }) {
            return Ok(unsafe{chunk.write_unchecked(addr, value)});
        }
        Err(format!("Invalid write at address => {addr}"))
    }

    /// QOL function to write a buffer into mem
    /// of course this is checked for bounds
    /// but may be unsafe still
    pub fn write_buffer<T>(&mut self, ptr: &Pointer<T>, buffer: Vec<T>) -> Result<(), String> {
        let mut i = 0;
        for val in buffer {
            let ptr = ptr.add(i);
            self.write(&ptr, val)?;
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
    let addr = match ptr.index() {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Invalid Ptr Free: {e}"))
    };

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

pub fn realloc<T>(vallocator: &mut Valloc, mut ptr: Pointer<T>, nsize: usize) -> Result<Pointer<T>, String> {
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
    let addr = ptr.index()?;

    // find the chunk that contains the pointer
    if let Some(chunk) = vallocator.chunks.iter_mut().find(move |chunk| {
        addr <= chunk.upper_bound() && addr >= chunk.lower_bound()
    }).copied() {
        let chunk_size = chunk.upper_bound() - chunk.lower_bound();
        
        // check if our current chunk size is 0
        if chunk_size == 0 {
            // return an error message, since we can't reallocate memory that doesn't exist
            return Err(format!("Failed to reallocate memory for size => {nsize}: Current size is 0 (or somehow broke the size of the mem_chunk)"));
        } else if nsize < chunk_size {
            // Okay...
            // we just deallocate the difference in sizes
            // and return the same pointer
            let diff = chunk_size - nsize;
            let (nchunk, nptr) = if diff > 0 {
                // let mut ptr: Box<[u8]> = TryInto::<BoxValue<[u8]>>::try_into(&mut vallocator.memory.get_data()[chunk.lower_bound()..chunk.lower_bound() + nsize]).unwrap().into();
                // let mem_ptr = &mut vallocator.memory.get_data()[chunk.lower_bound()..chunk.lower_bound() + nsize];
                // let ptr = unsafe{ std::mem::transmute::<&mut [u8], &mut [T]>() };
                let mut nptr = ptr.address_mut().expect("Failed to get the address of the pointer!");
                
                (
                    Some(VirtMemoryChunk::new(
                        &mut vallocator.memory.get_data()[chunk.lower_bound()..chunk.lower_bound() + nsize], 
                        chunk.lower_bound(), 
                        chunk.lower_bound() + nsize
                    )),
                    Some(Pointer::new(
                        &mut nptr, 
                        chunk.lower_bound()
                    ))
                )
            } else {
                (None, None)
            };

            if let Some(nchunk) = nchunk {
                vallocator.chunks.push(nchunk);
            } else {
                return Err(format!("Failed to reallocate memory: New size is greater than the current size"));
            }
            
            // get data from the old buffer
            let data = vallocator.read_buffer(&ptr, diff)?;

            // remove old chunk
            vallocator.chunks.retain(|c| {
                chunk.lower_bound() != c.lower_bound() && 
                chunk.upper_bound() != c.upper_bound()
            });
            
            if let Some(ptr) = nptr {
                // write into new buffer
                vallocator.write_buffer(&ptr, data)?;
                return Ok(ptr);
            } else {
                return Err(format!("Failed to reallocate memory for size => {nsize}: Couldn't find chunk that contains address => {addr}"));
            }
        }
        
        // allocate a spcae the size of our current chunk size + the new chunk size
        let mut new_ptr: Pointer<T> = vallocator.alloc(chunk_size + nsize)?;
        dbg!(chunk_size);
        dbg!(chunk_size + nsize);
        println!("Ptr {{ address: {:?}, index: {}, len {} }}", ptr.address().unwrap().as_ptr(), ptr.index().unwrap(), ptr.address().unwrap().len());
        println!("New Ptr {{ address: {:?}, index: {}, len: {} }}", new_ptr.address().unwrap().as_ptr(), new_ptr.index().unwrap(), ptr.address().unwrap().len());
        // read the data from the old ptr (store it to put it in the new ptr)
        let data = vallocator.read_buffer(&ptr, chunk_size)?;
        // write the data to the new ptr
        vallocator.write_buffer(&mut new_ptr, data)?;

        // free the old ptr (Note: this does NOT zero out the memory, it just removes the chunk from the chunks vector, see free() for more on this)
        vallocator.free(&mut ptr)?;

        vallocator.chunks.retain(|c| {
            if chunk.lower_bound() == c.lower_bound() && chunk.upper_bound() == c.upper_bound() { false } else { true }
        });
        
        // were done return the new ptr
        return Ok(new_ptr);
    }
    Err(format!("Failed to reallocate memory for size => {nsize}: Couldn't find chunk that contains address => {addr}"))
}