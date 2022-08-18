use crate::{AllocError, AllocResult, Allocator, StaticAllocator};

use std::mem::MaybeUninit;
use std::ptr::NonNull;

pub struct SystemAllocator;

unsafe impl Allocator for SystemAllocator {
    fn allocate<T>(&mut self, count: usize) -> AllocResult<NonNull<[T]>> {
        StaticAllocator::allocate(self, count)
    }
    fn owns<T>(&self, block: NonNull<[T]>) -> bool {
        StaticAllocator::owns(self, block)
    }
    unsafe fn deallocate<T>(&mut self, block: NonNull<[T]>) {
        // SAFETY: the caller asserts block was returned by SystemAllocator::allocate which always
        // returns a valid Box pointer.
        unsafe { StaticAllocator::deallocate(self, block) }
    }
}

unsafe impl StaticAllocator for SystemAllocator {
    fn allocate<T>(&self, count: usize) -> AllocResult<NonNull<[T]>> {
        let mut vec: Vec<MaybeUninit<T>> = Vec::new();
        vec.try_reserve(count).map_err(|_| AllocError)?;
        let ptr = Box::into_raw(vec.into_boxed_slice()) as *mut [T];
        // SAFETY: Box always returns a non-null pointer.
        Ok(unsafe { NonNull::new_unchecked(ptr) })
    }
    fn owns<T>(&self, _block: NonNull<[T]>) -> bool {
        panic!("can't detect ownership of system allocation")
    }
    unsafe fn deallocate<T>(&self, block: NonNull<[T]>) {
        // SAFETY: the caller asserts block was returned by SystemAllocator::allocate which always
        // returns a valid Box pointer.
        let _drop = unsafe { Box::from_raw(block.as_ptr()) };
    }
}
