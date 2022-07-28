#![feature(slice_ptr_get)]
#![feature(const_mut_refs)]
#![warn(unsafe_op_in_unsafe_fn)]

pub mod allocators;
pub mod collections;
mod traits;
pub(crate) mod utility;

pub use traits::{Allocator, StaticAllocator};

#[macro_export]
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
        unsafe impl $crate::StaticAllocator for $new_type {
            fn allocate<T>(&self, count: usize) -> Option<::std::ptr::NonNull<[T]>> {
                Self::get_alloc().allocate(count)
            }
            fn owns<T>(&self, block: ::std::ptr::NonNull<[T]>) -> bool {
                Self::get_alloc().owns(block)
            }
            unsafe fn deallocate<T>(&self, block: ::std::ptr::NonNull<[T]>) {
                unsafe { Self::get_alloc().deallocate(block) }
            }
        }
        paste::paste! {
            static [< __INTERNAL_ $new_type:upper >]: ::std::sync::Mutex<$alloc> = ::std::sync::Mutex::new($alloc_instance);
        }
    };
}
