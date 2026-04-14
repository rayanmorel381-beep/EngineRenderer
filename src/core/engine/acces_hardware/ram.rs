#[derive(Clone, Copy, Debug)]
pub struct RamConfig {
    pub page_size: usize,
    pub total_bytes: u64,
    pub available_bytes: Option<u64>,
}

pub fn default_ram_config() -> RamConfig {
    let r = super::arch::compute_dispatch::default_ram_config();
    RamConfig {
        page_size: r.page_size,
        total_bytes: r.total_bytes,
        available_bytes: r.available_bytes,
    }
}
