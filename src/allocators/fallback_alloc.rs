use crate::{AllocResult, Allocator, StaticAllocator};

use std::ptr::NonNull;

pub struct FallbackAllocator<A, B> {
    primary: A,
    fallback: B,
}

unsafe impl<A, B> Allocator for FallbackAllocator<A, B>
where
    A: Allocator,
    B: Allocator,
{
    fn allocate<T>(&mut self, count: usize) -> AllocResult<NonNull<[T]>> {
        self.primary
            .allocate(count)
            .or_else(|_| self.fallback.allocate(count))
    }
    fn owns<T>(&self, block: NonNull<[T]>) -> bool {
        self.primary.owns(block) || self.fallback.owns(block)
    }
    unsafe fn deallocate<T>(&mut self, block: NonNull<[T]>) {
        if self.primary.owns(block) {
            unsafe { self.primary.deallocate(block) }
        } else {
            unsafe { self.fallback.deallocate(block) }
        }
    }
}

unsafe impl<A, B> StaticAllocator for FallbackAllocator<A, B>
where
    A: StaticAllocator,
    B: StaticAllocator,
{
    fn allocate<T>(&self, count: usize) -> AllocResult<NonNull<[T]>> {
        self.primary
            .allocate(count)
            .or_else(|_| self.fallback.allocate(count))
    }
    fn owns<T>(&self, block: NonNull<[T]>) -> bool {
        self.primary.owns(block) || self.fallback.owns(block)
    }
    unsafe fn deallocate<T>(&self, block: NonNull<[T]>) {
        if self.primary.owns(block) {
            unsafe { self.primary.deallocate(block) }
        } else {
            unsafe { self.fallback.deallocate(block) }
        }
    }
}
