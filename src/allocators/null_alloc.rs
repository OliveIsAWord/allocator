use crate::{AllocError, AllocResult, Allocator, StaticAllocator};

use std::ptr::NonNull;

// TODO: Dedup this code

//#[derive(Clone, Copy, Debug)]
pub struct NullAllocator;

unsafe impl Allocator for NullAllocator {
    fn allocate<T>(&mut self, _count: usize) -> AllocResult<NonNull<[T]>> {
        Err(AllocError)
    }
    fn owns<T>(&self, _block: NonNull<[T]>) -> bool {
        false
    }
    unsafe fn deallocate<T>(&mut self, _block: NonNull<[T]>) {
        panic!("NullAllocator never returns freeable memory")
    }
}

unsafe impl StaticAllocator for NullAllocator {
    fn allocate<T>(&self, _count: usize) -> AllocResult<NonNull<[T]>> {
        Err(AllocError)
    }
    fn owns<T>(&self, _block: NonNull<[T]>) -> bool {
        false
    }
    unsafe fn deallocate<T>(&self, _block: NonNull<[T]>) {
        panic!("NullAllocator never returns freeable memory")
    }
}
