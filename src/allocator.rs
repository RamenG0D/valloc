use std::{
    alloc::Allocator, cell::RefCell, collections::LinkedList, ptr::NonNull
};

// global allocator
static mut ALLOCATOR:  Option<GlobalValloc> = None;
static mut GLOBAL_MEM: Option<&mut [u8]>    = None;

#[derive(Debug)]
pub struct GlobalValloc<'a>(RefCell<Valloc<'a>>);
impl<'a> GlobalValloc<'a> {
    pub fn new(allocator: Valloc<'a>) -> Self {
        Self(RefCell::new(allocator))
    }
}

pub fn global_allocator() -> &'static mut GlobalValloc<'static> {
    unsafe{ ALLOCATOR.as_mut() }.expect("Failed to get global allocator")
}

impl<'a> From<Valloc<'a>> for GlobalValloc<'a> {
    fn from(value: Valloc<'a>) -> Self {
        Self::new(value)
    }
}

unsafe impl Allocator for &mut GlobalValloc<'_> {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe{&mut*self.0.as_ptr()}
            .alloc(layout.size())
            .map(|ptr: SmartPointer<[u8]>| ptr.non_null_ptr())
            .map_err(|_| std::alloc::AllocError)
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        unsafe{&mut*self.0.as_ptr()}
            .free(SmartPointer::new(ptr))
            .unwrap();
    }
}

// convenience type for a pointer
pub struct SmartPointer<T> 
    where T: ?Sized
{
    ptr: NonNull<T>,
}

impl<T> SmartPointer<T> 
    where T: ?Sized
{
    pub fn new(ptr: NonNull<T>) -> Self {
        Self {ptr}
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn non_null_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    pub fn cast<U: Sized>(&self) -> SmartPointer<U> {
        SmartPointer::new(self.ptr.cast())
    }
}

impl<T> std::ops::Deref for SmartPointer<T> 
    where T: ?Sized
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> std::ops::DerefMut for SmartPointer<T> 
    where T: ?Sized
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> std::ops::Index<usize> for SmartPointer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.ptr.offset(index as isize).as_ptr() }
    }
}

impl<T> std::ops::IndexMut<usize> for SmartPointer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.ptr.offset(index as isize).as_ptr() }
    }
}

/// Get a mutable reference to the global allocator
/// 
/// # Safety
/// 
/// The caller must ensure that the allocator is initialized before calling this function
/// 
/// # Example
/// 
/// 
/// use valloc::allocator::get_allocator;
/// let allocator = get_allocator();
/// 
/// 
/// # Returns
/// 
/// * `&'static mut Valloc` - A mutable reference to the global allocator
/// 
/// # Panics
/// 
/// This function will panic if the allocator is not initialized
pub fn get_allocator() -> &'static mut Valloc<'static> {
    pub unsafe fn get_allocator() -> Result<&'static mut Valloc<'static>, &'static str> {
        match ALLOCATOR {
            Some(ref mut allocator) => Ok(allocator.0.get_mut()),
            None => Err("Allocator not initialized!")
        }
    }
    unsafe{ get_allocator().unwrap() }
}

/// Initializes the allocator with the total memory size (in bytes)
/// 
/// # Arguments
/// 
/// * `msize` - The total memory size to allocate
pub fn valloc_init(msize: usize) {
    #[cfg(debug_assertions)]
    if unsafe{ALLOCATOR.is_some()} { panic!("Allocator already initialized!"); }
    #[cfg(debug_assertions)]
    if unsafe{GLOBAL_MEM.is_some()} { panic!("Memory already initialized!"); }
    

    unsafe { GLOBAL_MEM = Some(vec![0u8; msize].leak()); }
    unsafe { ALLOCATOR = Some(GlobalValloc::new(Valloc::new(GLOBAL_MEM.as_deref_mut().unwrap()))); }
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
pub struct Valloc<'a> {
    memory: &'a mut [u8],

    chunks: ChunkList, 
}

impl From<&[u8]> for Valloc<'_> {
    fn from(value: &[u8]) -> Self {
        let (len, mem) = (value.len(), value.as_ptr() as *mut u8);
        Valloc::new(unsafe{std::slice::from_raw_parts_mut(mem, len)})
    }
}

#[derive(Debug)]
pub struct ChunkList {
    list: LinkedList< Box<ChunkNode> >,
    available: usize,
}

impl ChunkList {
    pub fn new(start: Option< Box<ChunkNode> >, available: usize) -> Self {
        let mut list = LinkedList::new();
        if let Some(start) = start {
            list.push_back(start);
        }
        Self { list, available }
    }

    pub fn iter(&self) -> std::collections::linked_list::Iter< Box<ChunkNode> > {
        self.list.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::linked_list::IterMut< Box<ChunkNode> > {
        self.list.iter_mut()
    }

    pub fn into_iter(self) -> std::collections::linked_list::IntoIter< Box<ChunkNode> > {
        self.list.into_iter()
    }

    pub fn push_back(&mut self, chunk: Box<ChunkNode>) {
        self.list.push_back(chunk);
    }

    pub fn push_front(&mut self, chunk: Box<ChunkNode>) {
        self.list.push_front(chunk);
    }

    pub fn pop_back(&mut self) -> Option<Box<ChunkNode>> {
        self.list.pop_back()
    }

    pub fn pop_front(&mut self) -> Option<Box<ChunkNode>> {
        self.list.pop_front()
    }

    pub fn get_available(&self) -> usize {
        self.available
    }

    pub fn set_available(&mut self, available: usize) {
        self.available = available;
    }
}

#[derive(Debug, Clone)]
pub struct ChunkNode {
    ptr: *mut u8,
    size: usize,
    in_use: bool
}

impl ChunkNode {
    pub fn new(ptr: *mut u8, size: usize, in_use: bool) -> Self {
        // upon creation, the chunk is in use
        // and when free is called, it will be set to false
        Self { ptr, size, in_use }
    }

    pub fn get_ptr<T: Sized>(&self) -> *mut T
    {
        self.ptr as *mut T
    }

    pub fn ptr_unsized<T: ?Sized>(&self) -> &*mut T {
        unsafe{ std::mem::transmute(&self.ptr) }
    }
    
    pub fn get_size(&self) -> usize {
        self.size
    }
}

impl<'a> Valloc<'a> {
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
    /// let kernel = Valloc::new(vec![0u8; 1024].leak());
    /// ```
    pub fn new(memory: &'a mut [u8]) -> Self {
        // we need to cast the memory to a u8 pointer
        // we also must adjust the len to be in bytes (which is len * `the difference between size_of::<T>() and size_of::<u8>()`)
        let len = memory.len();
        let chunks = {
            let memory = memory.as_mut().as_mut_ptr() as *mut u8;
            ChunkList::new(
                Some(
                    Box::new(ChunkNode::new(
                        memory, 
                        len, 
                        false
                    ))
                ),
                len
            )
        };

        Self { memory, /*our heap chunk starts out spanning the entire memory*/ chunks }
    }
}

impl Valloc<'_> {
    pub fn from_mem(
        memory: NonNull<u8>, len: usize
    ) -> Self {
        debug_assert!(len > 0, "Memory length must be greater than 0!");

        let chunks = ChunkList::new(
            Some(
                Box::new(ChunkNode::new(
                    memory.as_ptr(), 
                    len, 
                    false
                ))
            ),
            len
        );
        Self { memory: unsafe{std::slice::from_raw_parts_mut(memory.as_ptr(), len)}, chunks }
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
    /// * `Ok(*mut T)` - A pointer to the start of the allocated chunk if successful.
    /// * `Err(String)` - An error message if allocation fails.
    /// 
    /// # Note
    /// 
    /// This method allocates in bytes.
    pub fn alloc<T: ?Sized>(&mut self, size: usize) -> Result<SmartPointer<T>, &'static str> {
        alloc(self, size)
    }

    /// Reallocate a MemoryChunk instance.
    /// 
    /// This method reallocates the memory for a given pointer to a new size.
    /// 
    /// # Arguments
    /// 
    /// * `SmartPointer` - The pointer to the memory chunk to be reallocated.
    /// * `new_size` - The new size of the memory chunk, in bytes.
    /// 
    /// # Returns
    /// 
    /// * `Ok(*mut T)` - A pointer to the reallocated memory chunk if successful.
    /// * `Err(String)` - An error message if reallocation fails.
    pub fn realloc<T: ?Sized>(&mut self, ptr: SmartPointer<T>, new_size: usize) -> Result<SmartPointer<T>, String> {
        realloc(self, ptr, new_size)
    }

    /// # Description
    /// 
    /// Allocate a new array of T.
    /// This just makes sure that the new size is a multiple of the size of T.
    /// and allows rust to enforce slice (array) safety.
    /// 
    /// # Arguments
    /// 
    /// * `new_size` - The new size of the array, in bytes.
    /// 
    /// # Returns
    /// 
    /// * `Ok(*mut [T])` - A pointer to the start of the allocated array if successful.
    /// * `Err(String)` - An error message if allocation fails.
    /// 
    /// # Note
    /// 
    /// This method DOES `NOT` allocate in bytes!
    /// It allocates in multiples of the size of T.
    pub fn alloc_array<T: Sized>(&mut self, new_size: usize) -> Result<SmartPointer<[T]>, String> {
        // because its sized we can check if the new size is a multiple of the size of T if it is then we can use alloc and safely cast the pointer to an array of T
        let ptr = self.alloc::<[T]>(new_size * std::mem::size_of::<T>())?;
        Ok(ptr)
    }

    /// Deallocate a MemoryChunk instance.
    /// 
    /// This method removes a MemoryChunk instance from the chunks vector.
    /// It behaves like the free() function in C and only accepts a pointer to the first element of the chunk.
    /// The memory is not zeroed out, so the data will still be there, but the chunk will be removed from the chunks vector.
    /// 
    /// # Arguments
    /// 
    /// * `SmartPointer` - A mutable reference to the pointer to the memory chunk to be deallocated.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If deallocation is successful.
    /// * `Err(String)` - An error message if deallocation fails.
    pub fn free<T: ?Sized>(&mut self, ptr: SmartPointer<T>) -> Result<(), String> {
        free(self, ptr)
    }
}

pub fn alloc<T: ?Sized>(vallocator: &mut Valloc, size: usize) -> Result<SmartPointer<T>, &'static str> {
    // only check if not release
    #[cfg(debug_assertions)]
    if size == 0 { return Err("Size must be greater than 0!"); }

    // first we need to check if there is enough space in the memory
    if size > vallocator.memory.len() {
        return Err("Not enough space in memory!");
    }

    // then we need to check if there is enough contiguous space in the memory
    let mut iter = vallocator.chunks.iter_mut();
    let chunk = iter.find(|x| !x.in_use && x.size >= size).ok_or("Not enough contiguous space in memory!")?;
    let mut new_chunk = None;

    // and check if we need to split the chunk
    if chunk.size > size {
        // we need to split the chunk
        new_chunk = Some(Box::new(ChunkNode::new(
            (chunk.get_ptr::<u8>() as usize + size) as *mut u8,
            chunk.size - size,
            false
        )));
    }
    // we also need to update the size of the chunk
    chunk.size = size;

    // now we need to set the chunk to in use
    chunk.in_use = true;
    // and get the pointer to the chunk
    let ptr: SmartPointer<T> = {
        let ptr = chunk.ptr_unsized::<T>();
        SmartPointer::new(
            NonNull::new(*ptr).expect("Failed to create SmartPointer!")
        )
    };

    // and update the available size    
    vallocator.chunks.available -= size;

    // check if we need to add a new chunk
    if let Some(new_chunk) = new_chunk {
        // insert the new chunk after the current chunk
        vallocator.chunks.list.push_back(new_chunk);
    }

    // return the unsized type pointer
    Ok(ptr)
}

pub fn free<T: ?Sized>(vallocator: &mut Valloc, ptr: SmartPointer<T>) -> Result<(), String> {
    // now we need to check if the pointer is in the chunks
    let mut iter = vallocator.chunks.iter_mut().peekable();
    // check for any adjacent chunks that are not in use
    // and merge them with the current chunk
    while let Some(chunk) = iter.next() {
        if chunk.get_ptr() == (ptr.as_ptr() as *mut u8) {
            // check if the chunk is in use
            if !chunk.in_use {
                return Err(format!("Pointer is not in use: SmartPointer:{{{:#X}}}, Maybe it was already freed?", (ptr.as_ptr() as *mut u8) as usize));
            }

            // check if the next chunk is not in use
            if let Some(next) = iter.peek() {
                if !next.in_use {
                    // merge the next chunk with the current chunk
                    chunk.size += next.size;
                    // remove the next chunk
                    iter.next();
                }
            }

            // check if the previous chunk is not in use
            if let Some(prev) = iter.peek_mut() {
                if !prev.in_use {
                    // merge the previous chunk with the current chunk
                    prev.size += chunk.size;
                    // remove the current chunk
                    iter.next();
                }
            }

            // and set the chunk to not in use
            chunk.in_use = false;
            
            // and update the available size
            vallocator.chunks.available += chunk.get_size();

            return Ok(());
        }
    }

    // then we need to check if the pointer is in the chunks
    Err(format!("Pointer is not in use: SmartPointer:{{{:#X}}}, Maybe it was already freed?", (ptr.as_ptr() as *mut u8) as usize))
}

pub fn realloc<T: ?Sized>(vallocator: &mut Valloc, ptr: SmartPointer<T>, nsize: usize) -> Result<SmartPointer<T>, String> {
    // first we need to check if the pointer is in the memory
    if (ptr.as_ptr() as *mut u8 as usize) < vallocator.memory.as_ptr() as usize || (ptr.as_ptr() as *mut u8) >= (vallocator.memory.as_ptr() as usize + vallocator.memory.len()) as *mut u8 {
        return Err(format!("Pointer is not in memory: SmartPointer:{{{:#X}}}", (ptr.as_ptr() as *const u8) as usize));
    }

    // now we let the other functions `alloc` and `free` do all the heavy lifting here :D
    // by using them to just allocate a new chunk of size (nsize)
    // then we place the old SmartPointer's data into the new chunk
    // and lastly we just free the old chunk

    let lsize = vallocator.chunks.iter()
        .find(|x| x.get_ptr() == ptr.as_ptr() as *mut u8)
        .ok_or(format!("Pointer not found in chunks: SmartPointer:{{{:#X}}}", (ptr.as_ptr() as *mut u8) as usize))?
        .get_size();

    // allocate a new chunk of size (nsize)
    let nptr: SmartPointer<T> = alloc(vallocator, nsize)?;
    {
        // copy the data from the old chunk to the new chunk
        // first we are going to reinterpret the pointers as u8 pointers
        let (optr, nptr) = (ptr.as_ptr() as *mut u8, nptr.as_ptr() as *mut u8);
        // then we are going to copy the data from the old chunk to the new chunk
        unsafe { std::ptr::copy(optr, nptr, lsize * std::mem::size_of::<u8>()); }
    }

    // free the old chunk
    free(vallocator, ptr)?;

    // return the new pointer
    Ok(nptr)
}
