use std::{cell::RefCell, marker::PhantomData, ops::{Deref, DerefMut, Index, IndexMut}};
use crate::errors::AllocError;
use anyhow::{self, Context};
use unique::Unique;

#[derive(Debug)]
pub struct Ptr<'a, T: ?Sized> {
    ptr: Unique<T>, size: usize,
    _p: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> Deref for Ptr<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T: ?Sized> DerefMut for Ptr<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a, T> Index<usize> for Ptr<'a, [T]> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe{ &(*self.get_ptr())[index] }
    }
}

impl<'a, T> IndexMut<usize> for Ptr<'a, [T]> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe{ &mut (*self.get_ptr())[index] }
    }
}

impl<'a, T: ?Sized> Ptr<'a, T> {
    pub fn new(ptr: *mut T, len: usize) -> anyhow::Result<Self> {
        let res = Self {
            ptr: Unique::new(ptr).ok_or(AllocError::InvalidPointer)?,
            size: len,
            _p: PhantomData,
        };

        Ok(res)
    }

    pub fn cast<U>(&self) -> Ptr<'a, U> {
        let ptr = self.get_ptr() as *mut U;
        Ptr::new(ptr, self.len()).unwrap()
    }

    pub fn get_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn to_bytes(&self) -> Ptr<'a, [u8]> {
        let ptr = self.get_ptr() as *mut u8;
        let ptr = unsafe {
            std::slice::from_raw_parts_mut(ptr, self.len())
        };
        Ptr::new(ptr, self.len()).unwrap()
    }

    pub fn copy_bytes_raw(&mut self, data: &[u8]) {
        let ptr = self.get_ptr() as *mut u8;
        unsafe {
            std::ptr::copy(data.as_ptr(), ptr, self.len());
        }
    }

    pub fn copy_bytes<Object>(&mut self, other: Object) {
        // used to reinterpret any data as its byte representation
        fn to_bytes<'a, T>(data: T) -> &'a [u8] {
            let ptr = &data as *const T as *const u8;
            let len = std::mem::size_of::<T>();
            unsafe {
                std::slice::from_raw_parts(ptr, len)
            }
        }

        let data = to_bytes(other);

        self.copy_bytes_raw(data);
    }
}

#[derive(Debug)]
pub struct MemChunk {
    start: usize,
    end: usize,
    free: bool,
}

impl MemChunk {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end, free: true }
    }

    pub fn size(&self) -> usize {
        self.end - self.start
    }

    pub fn is_free(&self) -> bool {
        self.free
    }

    pub fn set_free(&mut self, free: bool) {
        self.free = free;
    }
}

/// # Allocator
///
/// The `Allocator` struct is a simple memory allocator that can be used to 'allocate' and 'free' memory (uses already allocated internal memory).
///
/// # Example
///
/// ```
/// use crate::valloc::allocator::Allocator;
/// use std::mem::size_of;
/// let mut allocator = Allocator::new(1024);
/// let ptr = allocator.alloc::<u8>(size_of::<u8>()).unwrap();
/// let mut ptr = ptr.cast::<u16>();
/// *ptr = 1;
/// assert_eq!(*ptr, 1);
/// allocator.free(ptr).unwrap();
/// ```
pub struct Allocator<'a> {
    memory: RefCell<Box<[u8]>>,
    mmap: Vec<MemChunk>,
    _p: PhantomData<&'a ()>
}

impl<'a> Allocator<'a> {
    pub fn new(size: usize) -> Self {
        let mut mmap = Vec::new();
        // Our First memory chunk spans the entire memory (we then subdivide it during allocation)
        mmap.push(MemChunk::new(0, size));
        Self {
            memory: RefCell::new(vec![0; size].into_boxed_slice()),
            _p: PhantomData,
            mmap,
        }
    }

    pub fn get_memory(&self) -> &RefCell<Box<[u8]>> {
        &self.memory
    }

    pub fn alloc<T: ?Sized>(&mut self, size: usize) -> anyhow::Result<Ptr<'a, T>> {
        alloc(self, size).context("Failed to allocate memory")
    }

    pub fn free<T: ?Sized>(&mut self, ptr: Ptr<'a, T>) -> anyhow::Result<()> {
        free(self, ptr).context("Failed to free memory")
    }

    pub fn realloc<T: ?Sized>(&mut self, ptr: Ptr<'a, T>, new_size: usize) -> anyhow::Result<Ptr<'a, T>> {
        realloc(self, ptr, new_size).context("Failed to reallocate memory")
    }
}

/// used to convert a pointer to an index in the memory array (if the pointer is not within the memory range then an error is returned)
fn ptr_to_index(ptr: usize, start: usize, len: usize) -> Result<usize, AllocError> {
    if (ptr < start) || (ptr > (start + len)) {
        return Err(AllocError::InvalidPointer);
    }

    Ok(ptr - start)
}

pub fn alloc<'a, T: ?Sized>(allocatorator: &mut Allocator<'a>, size: usize) -> Result<Ptr<'a, T>, AllocError> {
    debug_assert_ne!(size, 0, "Cannot allocate 0 bytes");

    let mut mem = allocatorator.memory.borrow_mut();

    // find the first chunk that is free and has enough space
    let (index, nchunk) = {
        let (index, chunk) = allocatorator.mmap.iter_mut().enumerate().find(|(_, chunk)| chunk.is_free() && chunk.size() >= size).ok_or(AllocError::OOM)?;

        // split the chunk into the portion we are using and the portion that is leftover
        let nchunk = MemChunk::new(chunk.start + size, chunk.end);
        // nchunk.free = true; // this is the default value

        // make the end of the current chunk the stop after the new size ([*********] into [*****][***])
        chunk.end = chunk.start + size;

        (index, nchunk)
    };

    // now we need to insert the new chunk into the mmap
    allocatorator.mmap.insert(index + 1, nchunk);

    // return the pointer to the start of the chunk
    let (ptr, len) = {
        let mem = {
            let ochunk = &allocatorator.mmap[index];
            &mut mem[ochunk.start..ochunk.end]
        };
        (mem.as_mut_ptr(), mem.len())
    };
    debug_assert!(!ptr.is_null());

    // now we must do a little type fuckery to get the pointer to the correct type
    let ptr = &ptr as *const *mut _ as *const *mut T;
    let ptr = unsafe{ *ptr };

    Ptr::new(ptr, len).or(Err(AllocError::InvalidPointer))
}

pub fn free<'a, T: ?Sized>(allocatorator: &mut Allocator<'a>, ptr: Ptr<'a, T>) -> Result<(), AllocError> {
    let mem = allocatorator.memory.try_borrow().or(Err(AllocError::InvalidPointer))?;

    // find the chunk that the pointer is in
    let ptr = ptr.get_ptr() as *const () as usize;

    // SAFTEY: we must ensure that the pointer is acctually within / to the array (from mem.as_ptr() to mem.as_ptr() + mem.len())
    // if the pointer is not within the memory range then we should return an error
    // this is A CRITICAL SAFETY CHECK do NOT change / remove this
    let index = ptr_to_index(ptr, mem.as_ptr() as usize, mem.len())?;

    if let Some(chunk) = allocatorator.mmap.iter_mut().find(|chunk| chunk.start == index) {
        chunk.set_free(true);
        Ok(())
    } else {
        Err(AllocError::ChunkNotFound)
    }
}

pub fn realloc<'a, T: ?Sized>(allocatorator: &mut Allocator<'a>, ptr: Ptr<'a, T>, nsize: usize) -> Result<Ptr<'a, T>, AllocError> {
    debug_assert_ne!(nsize, 0, "Cannot allocate 0 bytes");

    // check if the size is the same as the current size
    if ptr.len() == nsize {
        return Ok(ptr);
    }

    // get the index of the pointer
    let index = {
        let mem = allocatorator.memory.borrow();
        ptr_to_index(ptr.get_ptr() as *const () as usize, mem.as_ptr() as usize, mem.len())?
    };

    // check if the size is less than the current size
    // we can just shrink the current pointer
    if nsize < ptr.len() {
        // find the chunk that the pointer is in
        let newchunk;
        if let Some(chunk) = allocatorator.mmap.iter_mut().find(|chunk| chunk.start == index) {
            // we can just split the chunk into the portion we keep and the portion thats leftover fro the split
            let nchunk = MemChunk::new(chunk.start + nsize, chunk.end);
            // nchunk.free = true; // this is the default value

            // make the end of the current chunk the stop after the new size ([*********] into [*****][***])
            chunk.end = chunk.start + nsize;

            // now we need to insert the new chunk into the mmap
            newchunk = Some(nchunk);
        } else {
            return Err(AllocError::ChunkNotFound);
        }

        match newchunk {
            Some(chunk) => {
                // insert the new chunk into the mmap
                allocatorator.mmap.insert(index + 1, chunk);
            }
            _ => unreachable!()
        }

        Ok(ptr)
    } else {
        // we can allocate a new pointer and copy the data over, then free the old one
        let mut nptr = alloc::<T>(allocatorator, nsize)?;

        // copy the data from the old pointer to the new pointer
        let mut nbytes = nptr.to_bytes();
        let pbytes = ptr.to_bytes();
        // now we can copy the data
        for i in 0..pbytes.len() {
            // copy the bytes from the old pointer to the new pointer
            nbytes[i] = pbytes[i];
        }

        // now we just need to copy those bytes back to the new pointer
        nptr.copy_bytes_raw(&*nbytes);

        // free the old pointer
        free(allocatorator, ptr)?;

        Ok(nptr)
    }
}
