use std::ptr::NonNull;

pub fn make_slice_ptr<T>(ptr: NonNull<T>, count: usize) -> NonNull<[T]> {
    // SAFETY: This is unsound, but `ptr::from_raw_parts_mut` is unstable and the API scares me.
    unsafe {
        NonNull::new_unchecked(std::slice::from_raw_parts_mut(ptr.as_ptr(), count) as *mut [T])
    }
}
