/// Utilities to handle FoundationDB's allocation
/// Foundationdb uses arenas for allocation which are not aligned.
/// https://github.com/apple/foundationdb/blob/7aa578f616c24b60436429645427485b97520286/flow/Arena.cpp#L28-L31
///
/// Rust does not allow dereferencing unaligned pointers, so we copy the memory first to an aligned
/// pointer before constructing our slice.
use std::{
    alloc::Layout,
    ptr::{copy_nonoverlapping, read},
};

pub(crate) unsafe fn read_unaligned_slice<T>(src: *const T, len: i32) -> Vec<T> {
    let len = len as usize;
    let mut v = Vec::with_capacity(len);
    copy_nonoverlapping(src, v.as_mut_ptr(), len);
    v.set_len(len);
    v
}

#[allow(unused)]
pub(crate) unsafe fn read_unaligned_struct<T>(src: *const T) -> T {
    let layout = Layout::new::<T>();
    let aligned = std::alloc::alloc(layout);
    copy_nonoverlapping(src as *const u8, aligned, layout.size());
    read(aligned as *const T)
}
