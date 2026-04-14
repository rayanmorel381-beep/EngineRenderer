#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorSchedule {
    pub(crate) chunks: usize,
    pub(crate) chunk_size: usize,
    pub(crate) frame_budget_us: u64,
}

pub(crate) fn recommended_chunk_size(work_items: usize) -> usize {
    if work_items == 0 {
        return 8;
    }
    let total = std::thread::available_parallelism()
        .map(|v| v.get()).unwrap_or(1).max(1);
    let physical = (total / 2).max(1);
    let raw = work_items / physical;
    let aligned = ((raw + 7) / 8) * 8;
    aligned.max(8)
}

pub(crate) fn build_schedule(work_items: usize) -> VendorSchedule {
    let chunk_size = recommended_chunk_size(work_items);
    let chunks = if work_items == 0 { 1 } else { work_items.div_ceil(chunk_size) };
    VendorSchedule { chunks, chunk_size, frame_budget_us: 8_333 }
}
