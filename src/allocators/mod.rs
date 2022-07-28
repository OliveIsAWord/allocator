mod fallback_alloc;
mod null_alloc;
mod stack_alloc;

pub use fallback_alloc::FallbackAllocator;
pub use null_alloc::NullAllocator;
pub use stack_alloc::StackAllocator;

// impl Allocator for TODO {
//     fn allocate<T>(&mut self, count: usize) -> Option<NonNull<[T]>> { todo!() }
//     fn owns<T>(&self, block: NonNull<[T]>) -> bool { todo!() }
//     unsafe fn deallocate<T>(&mut self, block: NonNull<[T]>) { todo!() }
// }

use crate::{Allocator, StaticAllocator};
use std::ptr::NonNull;
use std::sync::Mutex;

// unsafe impl<A> Allocator for A
// where
//     A: StaticAllocator,
// {
//     fn allocate<T>(&mut self, count: usize) -> Option<NonNull<[T]>> {
//         self.allocate(count)
//     }
//     fn owns<T>(&mut self, block: NonNull<[T]>) -> bool {
//         self.owns(block)
//     }
//     unsafe fn deallocate<T>(&mut self, block: NonNull<[T]>) {
//         unsafe { self.deallocate(block) }
//     }
// }

unsafe impl<A> StaticAllocator for Mutex<A>
where
    A: Allocator,
{
    fn allocate<T>(&self, count: usize) -> Option<NonNull<[T]>> {
        self.lock().unwrap().allocate(count)
    }
    fn owns<T>(&self, block: NonNull<[T]>) -> bool {
        self.lock().unwrap().owns(block)
    }
    unsafe fn deallocate<T>(&self, block: NonNull<[T]>) {
        unsafe { self.lock().unwrap().deallocate(block) }
    }
}
