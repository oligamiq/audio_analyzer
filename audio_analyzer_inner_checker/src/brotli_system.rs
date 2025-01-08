use std::ops;

use brotli::enc::BrotliEncoderMaxCompressedSizeMulti;

#[derive(Clone, Copy, Default)]
pub struct HeapAllocator {}

impl<T: core::clone::Clone + Default> alloc_no_stdlib::Allocator<T> for HeapAllocator {
    type AllocatedMemory = Rebox<T>;
    fn alloc_cell(self: &mut HeapAllocator, len: usize) -> Rebox<T> {
        let v: Vec<T> = vec![T::default(); len];
        let b = v.into_boxed_slice();
        Rebox::<T> { b }
    }
    fn free_cell(self: &mut HeapAllocator, _data: Rebox<T>) {}
}

impl brotli::enc::BrotliAlloc for HeapAllocator {}

pub struct Rebox<T> {
    b: Box<[T]>,
}

impl<T> From<Vec<T>> for Rebox<T> {
    fn from(data: Vec<T>) -> Self {
        Rebox::<T> {
            b: data.into_boxed_slice(),
        }
    }
}

impl<T> core::default::Default for Rebox<T> {
    fn default() -> Self {
        let v: Vec<T> = Vec::new();
        let b = v.into_boxed_slice();
        Rebox::<T> { b }
    }
}

impl<T> ops::Index<usize> for Rebox<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &(*self.b)[index]
    }
}

impl<T> ops::IndexMut<usize> for Rebox<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut (*self.b)[index]
    }
}

impl<T> alloc_no_stdlib::SliceWrapper<T> for Rebox<T> {
    fn slice(&self) -> &[T] {
        &self.b
    }
}

impl<T> alloc_no_stdlib::SliceWrapperMut<T> for Rebox<T> {
    fn slice_mut(&mut self) -> &mut [T] {
        &mut self.b
    }
}

pub fn compress_multi_thread(
    params: &brotli::enc::BrotliEncoderParams,
    input: Vec<u8>,
) -> Result<Vec<u8>, brotli::enc::BrotliEncoderThreadError> {
    let max_threads = num_cpus::get();

    let max_threads = max_threads - 1;

    let mut alloc_array = vec![0; max_threads]
        .iter()
        .map(|_| get_alloc())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let mut out_data = vec![0u8; BrotliEncoderMaxCompressedSizeMulti(input.len(), max_threads)];

    brotli::enc::compress_worker_pool(
        &params,
        &mut brotli::enc::Owned::new(Rebox::from(input)),
        out_data.as_mut_slice(),
        &mut alloc_array,
        &mut brotli::enc::worker_pool::new_work_pool(max_threads),
    )
    .map(|size| out_data[..size].to_vec())
}

fn get_alloc<ReturnValue, Join>(
) -> brotli::enc::SendAlloc<ReturnValue, brotli::enc::UnionHasher<HeapAllocator>, HeapAllocator, Join>
where
    ReturnValue: Send + 'static,
    Join: brotli::enc::threading::Joinable<ReturnValue, brotli::enc::BrotliEncoderThreadError>,
{
    brotli::enc::SendAlloc::new(HeapAllocator::default(), brotli::enc::UnionHasher::Uninit)
}
