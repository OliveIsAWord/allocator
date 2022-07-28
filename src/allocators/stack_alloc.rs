use crate::Allocator;

use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use std::ptr::NonNull;

pub struct StackAllocator<'a> {
    stack: NonNull<[u8]>,
    index: usize,
    phantom: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

impl<'a> StackAllocator<'a> {
    /// # Safety
    ///
    /// Don't drop it! :3
    #[must_use]
    pub const unsafe fn new(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        Self {
            stack: unsafe { NonNull::new_unchecked(slice as *mut _ as *mut _) },
            index: 0,
            phantom: PhantomData,
        }
    }

    #[must_use]
    pub const fn size(&self) -> usize {
        self.stack.len()
    }
}

unsafe impl Allocator for StackAllocator<'_> {
    fn allocate<T>(&mut self, count: usize) -> Option<NonNull<[T]>> {
        let offset = self.stack.as_ptr().cast::<u8>() as usize;
        let aligned = self.index + negative_rem_euclid(offset + self.index, mem::align_of::<T>());
        let size = count * mem::size_of::<T>();
        let new_index = aligned + size;
        (new_index <= self.size()).then(|| {
            self.index = new_index;
            unsafe {
                NonNull::new_unchecked(
                    self.stack.as_ptr().get_unchecked_mut(aligned..new_index) as *mut [T]
                )
            }
        })
    }
    fn owns<T>(&self, block: NonNull<[T]>) -> bool {
        let ptr = block.as_ptr().cast::<u8>();
        let lower = self.stack.as_ptr().cast::<u8>();
        assert!(isize::try_from(self.size()).is_ok());
        let upper = unsafe { lower.add(self.size()) }.cast();
        lower <= ptr && ptr < upper
    }
    unsafe fn deallocate<T>(&mut self, block: NonNull<[T]>) {
        let ptr = unsafe { block.as_ptr().cast::<T>().add(block.len()) as *const u8 };
        let self_ptr = unsafe { self.stack.as_ptr().cast::<u8>().add(self.index) };
        if ptr == self_ptr {
            self.index -= mem::size_of::<T>() * block.len();
        }
    }
}

unsafe impl Send for StackAllocator<'_> {}

/// Calculates `(-lhs) % rhs` using Euclidian modulo
fn negative_rem_euclid(lhs: usize, rhs: usize) -> usize {
    assert!(rhs > 0); // I'm pretty sure every type, even () and !, have non-zero alignment
    (rhs - lhs % rhs) % rhs
}

// const fn ceiling_div(a: usize, b: usize) -> usize {
//     (a + b - 1) / b
// }

// const fn align_to<T>(x: usize) -> usize {
//     let align = mem::align_of::<T>();
//     ceiling_div(x, align) * align
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::Box;
    use crate::make_static;

    #[test]
    fn stack_allocation() {
        static mut STACK: [MaybeUninit<u8>; 7] = [MaybeUninit::new(0); 7];
        make_static! {Foo => StackAllocator<'static>, unsafe { StackAllocator::new(&mut STACK) }}
        let my_box = Box::new_in(800813555_i32, Foo).unwrap();
        assert_eq!(my_box.get(), &800813555);
    }

    #[test]
    fn stack_allocation_multiple() {
        static mut STACK: [MaybeUninit<u8>; 11] = [MaybeUninit::new(0); 11];
        make_static! {Foo => StackAllocator<'static>, unsafe { StackAllocator::new(&mut STACK) }}
        let box1 = Box::new_in(123456789_i32, Foo).unwrap();
        let box2 = Box::new_in(314159265_i32, Foo).unwrap();
        assert_eq!(box1.get(), &123456789);
        assert_eq!(box2.get(), &314159265);
    }

    #[test]
    fn stack_deallocation() {
        static mut STACK: [MaybeUninit<u8>; 7] = [MaybeUninit::new(0); 7];
        make_static! {Foo => StackAllocator<'static>, unsafe { StackAllocator::new(&mut STACK) }}
        let box1 = Box::new_in(42_u32, Foo).unwrap();
        assert_eq!(box1.get(), &42);
        assert!(Box::new_in(69_u32, Foo).is_none()); // not enough room for allocation
        drop(box1);
        let box2 = Box::new_in(69_u32, Foo).unwrap();
        assert_eq!(box2.get(), &69);
    }

    // // Correctly fails to compile
    // #[test]
    // fn stack_allocation_litetime() {
    //     use std::cell::RefCell;
    //     let mut stack: RefCell<[MaybeUninit<u8>; 1]> = RefCell::new([MaybeUninit::new(0); 1]);
    //     let alloc = Mutex::new(unsafe { StackAllocator::new(&mut *stack.borrow_mut()) }); // `Foo` semantically holds a mutable reference on `STACK`
    //     let my_box = Box::new_in(42_u8, alloc).unwrap();
    // }
}
