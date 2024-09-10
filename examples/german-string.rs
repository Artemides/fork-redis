use std::{
    alloc::Layout, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull, sync::atomic::AtomicUsize,
};
#[repr(C)]
struct GermanString {
    len: u32,
    prefix: [u8; 4],
    trailing: Trailing,
}

#[repr(C)]
union Trailing {
    buf: [u8; 8],
    ptr: ManuallyDrop<SharedDynBytes>,
}

#[repr(C)]
struct SharedDynBytes {
    ptr: NonNull<SharedDynBytesInner<[u8; 0]>>,
    phantom: PhantomData<SharedDynBytesInner<[u8; 0]>>,
}
#[repr(C)]
struct SharedDynBytesInner<T: ?Sized> {
    count: AtomicUsize,
    data: T,
}

impl SharedDynBytesInner<[u8]> {
    fn cast(thin_ptr: *mut u8, len: usize) -> *mut SharedDynBytesInner<[u8]> {
        let fake = std::ptr::slice_from_raw_parts_mut(thin_ptr, len);
        fake as *mut SharedDynBytesInner<[u8]>
    }
}
pub enum Error {
    TooLong,
}

impl SharedDynBytes {
    fn from(bytes: &[u8]) -> Self {
        let ptr = if bytes.is_empty() {
            NonNull::dangling()
        } else {
            let layout = shared_dyn_bytes_inner_layout(bytes.len());
            let nullable = unsafe { std::alloc::alloc(layout) };
            let nullable_fat_ptr = SharedDynBytesInner::<[u8]>::cast(nullable, bytes.len());

            let Some(fat_ptr) = NonNull::new(nullable_fat_ptr) else {
                std::alloc::handle_alloc_error(layout)
            };
            unsafe {
                let inner = &mut (*fat_ptr.as_ptr());
                std::ptr::write(&mut inner.count, AtomicUsize::new(1));
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), inner.data.as_mut_ptr(), bytes.len());
            }
            fat_ptr.cast()
        };

        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}
fn main() {}

fn shared_dyn_bytes_inner_layout(len: usize) -> Layout {
    Layout::new::<SharedDynBytesInner<()>>()
        .extend(Layout::array::<u8>(len).unwrap())
        .expect("A valid layout SharedDynBytesInner")
        .0
        .pad_to_align()
}
