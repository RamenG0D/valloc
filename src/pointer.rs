use std::{ops::Index, usize};

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

use std::ops::{Add, Deref, DerefMut, IndexMut, Sub};

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
                        index..(index + (len - 1))
                    })
                    .finish()
            },
            _ => {
                f.write_str("Pointer { NULL }")
            }
        }
    }
}

impl<T> Index<usize> for Pointer<T> 
    where T: Clone
{
    type Output = T;

    fn index(&self, pindex: usize) -> &T {
        match *self {
            Pointer::Pointer { address,  .. } => {
                let address = unsafe{(*address).as_ref()};
                &address[pindex]
            },
            Pointer::NULL => panic!("Attempted to dereference a NULL pointer")
        }
    }
}

impl<T> IndexMut<usize> for Pointer<T> 
    where T: Clone
{
    fn index_mut(&mut self, pindex: usize) -> &mut T {
        match *self {
            Pointer::Pointer { address, .. } => {
                let address = unsafe{(*address).as_mut()};
                &mut address[pindex]
            },
            Pointer::NULL => panic!("Attempted to dereference a NULL pointer")
        }
    }
}

impl<T> Deref for Pointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Pointer::Pointer { address, index } => {
                let address = unsafe{(*address).as_ref()};
                &address[index]
            },
            Pointer::NULL => panic!("Attempted to dereference a NULL pointer")
        }
    }
}

impl<T> DerefMut for Pointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self {
            Pointer::Pointer { address, index } => {
                let address = unsafe{(*address).as_mut()};
                &mut address[index]
            },
            Pointer::NULL => panic!("Attempted to dereference a NULL pointer")
        }
    }
}

impl<T, U> Add<U> for Pointer<T> 
    where U: Into<usize>
{
    type Output = Pointer<T>;

    fn add(self, rhs: U) -> Self::Output {
        let rhs = rhs.into();

        let index = match self {
            Pointer::Pointer { index, .. } => index + rhs,
            Pointer::NULL => 0
        };
        
        let maddr = self.address().unwrap();
        let address = unsafe{maddr.as_ptr().add(rhs)};
        let address = address as *const T;
        let address = address.cast_mut();
        let address = std::ptr::slice_from_raw_parts_mut(address, maddr.len());

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
                let addr = (address as *const N).cast_mut();
                let address = std::ptr::slice_from_raw_parts_mut(addr, unsafe{(*address).len()});
                Ok(Pointer::Pointer { address, index })
            },
            Pointer::NULL => Err("Attempted to cast a NULL pointer".to_string())
        }
    }
}
