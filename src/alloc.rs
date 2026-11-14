use buddy_alloc::{BuddyAllocParam, FastAllocParam, NonThreadsafeAlloc};

// These values can be tuned
const FAST_HEAP_SIZE: usize = 4 * 1024; // 4 KB
const HEAP_SIZE: usize = 16 * 1024; // 16 KB
const LEAF_SIZE: usize = 16;

#[repr(C, align(16))]
struct AlignedHeap<const N: usize>([u8; N]);

static mut FAST_HEAP: AlignedHeap<FAST_HEAP_SIZE> = AlignedHeap([0u8; FAST_HEAP_SIZE]);
static mut HEAP: AlignedHeap<HEAP_SIZE> = AlignedHeap([0u8; HEAP_SIZE]);

#[global_allocator]
static ALLOC: NonThreadsafeAlloc = unsafe {
    let fast_param = FastAllocParam::new(core::ptr::addr_of_mut!(FAST_HEAP.0).cast::<u8>(), FAST_HEAP_SIZE);
    let buddy_param = BuddyAllocParam::new(core::ptr::addr_of_mut!(HEAP.0).cast::<u8>(), HEAP_SIZE, LEAF_SIZE);
    NonThreadsafeAlloc::new(fast_param, buddy_param)
};
