#[derive(Debug)]
pub struct MemoryBuffer {
    data_ptr: *mut u8,
    size: usize,
}

impl MemoryBuffer {
    pub fn new(size: usize) -> Self {
        let mut data = vec![0u8; size];
        let data_ptr = data.as_mut_ptr();
        std::mem::forget(data);

        Self { data_ptr, size }
    }

    #[inline(always)]
    pub fn load<T>(&self, paddr: u64) -> T
    where
        T: Copy,
    {
        debug_assert!((paddr as usize) + std::mem::size_of::<T>() <= self.size);
        unsafe {
            let ptr = self.data_ptr.add(paddr as usize) as *const T;
            ptr.read_unaligned()
        }
    }

    #[inline(always)]
    pub fn store<T>(&mut self, paddr: u64, val: T) {
        debug_assert!((paddr as usize) + std::mem::size_of::<T>() <= self.size);
        unsafe {
            let ptr = self.data_ptr.add(paddr as usize) as *mut T;
            ptr.write_unaligned(val);
        }
    }

    pub const fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data_ptr
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        unsafe {
            Vec::from_raw_parts(self.data_ptr, self.size, self.size);
        }
    }
}
