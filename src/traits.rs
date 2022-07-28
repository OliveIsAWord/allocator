use std::ptr::NonNull;

/// # Safety
///
/// TODO
pub unsafe trait Allocator {
    fn allocate<T>(&mut self, count: usize) -> Option<NonNull<[T]>>;

    fn owns<T>(&self, block: NonNull<[T]>) -> bool;

    /// # Safety
    ///
    /// `block` must be a [`NonNull`] returned by this allocator.
    unsafe fn deallocate<T>(&mut self, block: NonNull<[T]>);
}

/// # Safety
///
/// TODO
pub unsafe trait StaticAllocator {
    fn allocate<T>(&self, count: usize) -> Option<NonNull<[T]>>;

    fn owns<T>(&self, block: NonNull<[T]>) -> bool;

    /// # Safety
    ///
    /// `block` must be a [`NonNull`] returned by this allocator.
    unsafe fn deallocate<T>(&self, block: NonNull<[T]>);
}
