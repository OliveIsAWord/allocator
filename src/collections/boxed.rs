use crate::utility::make_slice_ptr;
use crate::{AllocResult, StaticAllocator};

use std::mem;
use std::ptr::{self, NonNull};

pub struct Box<T, A>
where
    A: StaticAllocator,
{
    ptr: NonNull<T>,
    alloc: A,
}

impl<T, A> Box<T, A>
where
    A: StaticAllocator,
{
    pub fn new_in(value: T, alloc: A) -> AllocResult<Self> {
        alloc.allocate::<T>(1).map(|ptr| {
            let ptr = ptr.cast::<T>(); // cast pointer of [T] of length 1 to pointer of T
            debug_assert_eq!(ptr.as_ptr() as usize % mem::align_of::<T>(), 0); // sanity check: is pointer correctly aligned?
            unsafe {
                *ptr.as_ptr() = value;
            }
            Self { ptr, alloc }
        })
    }
    pub fn get(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, A> Drop for Box<T, A>
where
    A: StaticAllocator,
{
    fn drop(&mut self) {
        let block = make_slice_ptr(self.ptr, 1);
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            self.alloc.deallocate(block);
        }
    }
}
