#![warn(unsafe_op_in_unsafe_fn)]

use std::mem::{self, MaybeUninit};
use std::ptr::{self, NonNull};

#[derive(Debug)]
pub struct Block<T: ?Sized> {
    pub ptr: NonNull<T>,
    pub count: usize,
}

impl<T> Block<T> {
    #[must_use]
    pub const fn size(self) -> usize {
        self.count * mem::size_of::<T>()
    }
}

impl<T: ?Sized> Clone for Block<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            count: self.count,
        }
    }
}

impl<T: ?Sized> Copy for Block<T> {}

/// # Safety
///
/// TODO
pub unsafe trait Allocator {
    fn allocate<T>(&mut self, count: usize) -> Option<Block<T>>;

    fn owns<T>(&self, block: Block<T>) -> bool;

    /// # Safety
    ///
    /// `block` must be a [Block] returned by this allocator.
    unsafe fn deallocate<T>(&mut self, block: Block<T>);
}

pub struct NullAllocator;

unsafe impl Allocator for NullAllocator {
    fn allocate<T>(&mut self, _count: usize) -> Option<Block<T>> {
        None
    }
    fn owns<T>(&self, _block: Block<T>) -> bool {
        false
    }
    unsafe fn deallocate<T>(&mut self, _block: Block<T>) {
        panic!("NullAllocator never returns freeable memory")
    }
}

pub struct FallbackAllocator<Primary: Allocator, Fallback: Allocator> {
    primary: Primary,
    fallback: Fallback,
}

unsafe impl<Primary, Fallback> Allocator for FallbackAllocator<Primary, Fallback>
where
    Primary: Allocator,
    Fallback: Allocator,
{
    fn allocate<T>(&mut self, count: usize) -> Option<Block<T>> {
        self.primary
            .allocate(count)
            .or_else(|| self.fallback.allocate(count))
    }
    fn owns<T>(&self, block: Block<T>) -> bool {
        self.primary.owns(block) || self.fallback.owns(block)
    }
    unsafe fn deallocate<T>(&mut self, block: Block<T>) {
        if self.primary.owns(block) {
            unsafe { self.primary.deallocate(block) }
        } else {
            unsafe { self.fallback.deallocate(block) }
        }
    }
}

pub struct StackAllocator<const SIZE: usize> {
    stack: [MaybeUninit<u8>; SIZE],
    index: usize,
}

impl<const SIZE: usize> StackAllocator<SIZE> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stack: [MaybeUninit::uninit(); SIZE],
            index: 0,
        }
    }
}

impl<const SIZE: usize> Default for StackAllocator<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

const fn ceiling_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

const fn align_to<T>(x: usize) -> usize {
    ceiling_div(x, mem::align_of::<T>())
}

unsafe impl<const SIZE: usize> Allocator for StackAllocator<SIZE> {
    fn allocate<T>(&mut self, count: usize) -> Option<Block<T>> {
        let aligned = align_to::<T>(self.index);
        let size = count * mem::size_of::<T>();
        let new_index = aligned + size;
        if new_index <= SIZE {
            self.index = new_index;
            let ptr = unsafe { NonNull::new_unchecked(self.stack.as_ptr().add(aligned) as *mut _) };
            Some(Block { ptr, count })
        } else {
            None
        }
    }
    fn owns<T>(&self, block: Block<T>) -> bool {
        let ptr = block.ptr.as_ptr().cast::<MaybeUninit<u8>>();
        let lower = self.stack.as_ptr();
        assert!(isize::try_from(SIZE).is_ok());
        let upper = unsafe { lower.add(SIZE) } as *mut _;
        lower <= ptr && ptr < upper
    }
    unsafe fn deallocate<T>(&mut self, block: Block<T>) {
        let ptr = unsafe { block.ptr.as_ptr().add(block.count) as *const MaybeUninit<u8> };
        let self_ptr = unsafe { self.stack.as_ptr().add(self.index) };
        if ptr == self_ptr {
            self.index -= block.size();
        }
    }
}

// impl Allocator for TODO {
//     fn allocate<T>(&mut self, count: usize) -> Option<Block<T>> { todo!() }
//     fn owns<T>(&self, block: Block<T>) -> bool { todo!() }
//     unsafe fn deallocate<T>(&mut self, block: Block<T>) { todo!() }
// }

/// # Safety
///
/// TODO
pub unsafe trait StaticAllocator {
    fn allocate<T>(&self, count: usize) -> Option<Block<T>>;

    fn owns<T>(&self, block: Block<T>) -> bool;

    /// # Safety
    ///
    /// `block` must be a [Block] returned by this allocator.
    unsafe fn deallocate<T>(&self, block: Block<T>);
}

macro_rules! make_static {
    ($new_type:ident => $alloc:ty, $alloc_instance:expr) => {
        pub struct $new_type;
        impl $new_type {
            fn get_alloc() -> ::std::sync::MutexGuard<'static, $alloc> {
                paste::paste! {
                    [< __INTERNAL_ $new_type:upper >]
                }.lock().unwrap()
            }
        }
        unsafe impl StaticAllocator for $new_type {
            fn allocate<T>(&self, count: usize) -> Option<Block<T>> {
                Self::get_alloc().allocate(count)
            }
            fn owns<T>(&self, block: Block<T>) -> bool {
                Self::get_alloc().owns(block)
            }
            unsafe fn deallocate<T>(&self, block: Block<T>) {
                unsafe { Self::get_alloc().deallocate(block) }
            }
        }
        paste::paste! {
            static [< __INTERNAL_ $new_type:upper >]: ::std::sync::Mutex<$alloc> = ::std::sync::Mutex::new($alloc_instance);
        }
    };
}

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
    pub fn new_in(value: T, alloc: A) -> Option<Self> {
        alloc.allocate::<T>(1).map(|Block { ptr, .. }| {
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
        let block = Block {
            ptr: self.ptr,
            count: 1,
        };
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            self.alloc.deallocate(block);
        }
    }
}

make_static! {Foo2 => StackAllocator<5>, StackAllocator::new()}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_allocation() {
        make_static! {Foo => StackAllocator<5>, StackAllocator::new()}
        let my_box = Box::new_in(8008135_i32, Foo).unwrap();
        assert_eq!(my_box.get(), &8008135);
    }
}
