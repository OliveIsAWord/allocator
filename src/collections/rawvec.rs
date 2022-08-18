use crate::utility::make_slice_ptr;
use crate::{AllocError, AllocResult, StaticAllocator};

use std::mem::{self, MaybeUninit};
use std::ptr::{self, NonNull};

const MIN_ALLOC_LEN: usize = 1;

pub struct RawVec<T, A>
where
    A: StaticAllocator,
{
    slice: NonNull<[T]>,
    alloc: A,
    //_marker: PhantomData<T>,
}

impl<T, A> RawVec<T, A>
where
    A: StaticAllocator,
{
    pub fn new_in(alloc: A) -> Self {
        debug_assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");
        let slice = make_slice_ptr(NonNull::dangling(), 0);
        Self { slice, alloc }
    }
    pub const fn len(&self) -> usize {
        self.slice.len()
    }
    pub const fn is_empty(&self) -> bool {
        self.len() > 0
    }
    pub const fn get_slice(&self) -> &[MaybeUninit<T>] {
        unsafe { &*(self.slice.as_ptr() as *const _) }
    }
    pub const fn get_slice_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { &mut *(self.slice.as_ptr() as *mut _) }
    }
    pub fn grow(&mut self) -> AllocResult {
        let len = self.len();
        let new_len = if len == 0 { MIN_ALLOC_LEN } else { 2 * len };
        if isize::try_from(new_len).is_err() {
            do yeet AllocError;
        }
        let new_slice = self.alloc.allocate::<T>(new_len)?;
        // TODO: this almost certainly causes UB without `MaybeUninit<T>`
        unsafe {
            ptr::copy(
                self.slice.as_ptr().cast::<T>(),
                new_slice.as_ptr().cast::<T>(),
                len,
            );
        }
        Ok(())
    }
}

impl<T, A> Drop for RawVec<T, A>
where
    A: StaticAllocator,
{
    fn drop(&mut self) {
        if !self.is_empty() {
            unsafe { self.alloc.deallocate(self.slice) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocators::SystemAllocator;

    #[test]
    fn drop_empty() {
        let _drop: RawVec<f64, _> = RawVec::new_in(SystemAllocator);
    }

    #[test]
    fn grow() {
        let mut uwu: RawVec<f64, _> = RawVec::new_in(SystemAllocator);
        uwu.grow().unwrap();
    }
}
