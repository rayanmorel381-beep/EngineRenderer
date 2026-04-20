#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arch {
    X86,
    Arm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Os {
    Linux,
    Windows,
    Macos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vendor {
    Amd,
    Intel,
    Apple,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CpuConfig {
    pub(crate) vendor: Vendor,
    pub(crate) worker_hint: usize,
    pub(crate) render_workers: usize,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuConfig {
    pub(crate) vendor: Vendor,
    pub(crate) workgroup_size: usize,
    pub(crate) compute_queues: usize,
    pub(crate) render_threads: usize,
    pub(crate) double_buffered: bool,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DisplayConfig {
    pub(crate) vendor: Vendor,
    pub(crate) page_size: usize,
    pub(crate) target_render_fps: u32,
    pub(crate) latency_budget_us: u64,
    pub(crate) scan_out_latency_us: u64,
    pub(crate) vsync_slots: usize,
    pub(crate) double_buffered: bool,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Schedule {
    pub chunks: usize,
    pub chunk_size: usize,
    pub frame_budget_us: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RamConfig {
    pub(crate) page_size: usize,
    pub(crate) total_bytes: u64,
    pub(crate) available_bytes: Option<u64>,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ComputeDispatchConfig {
    pub(crate) arch: Arch,
    pub(crate) os: Os,
    pub(crate) cpu: CpuConfig,
    pub(crate) gpu: GpuConfig,
    pub(crate) display: DisplayConfig,
    pub(crate) ram: RamConfig,
}

pub(crate) fn detect_arch() -> Arch {
    let known = [Arch::X86, Arch::Arm];
    if known.is_empty() {
        return Arch::X86;
    }
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        return Arch::X86;
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return Arch::Arm;
    }
    #[allow(unreachable_code)]
    Arch::X86
}

pub(crate) fn detect_os() -> Os {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        return map_os_x86(super::x86::os::detect_os());
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return map_os_arm(super::arm::os::detect_os());
    }
    #[allow(unreachable_code)]
    Os::Linux
}

pub(crate) fn default_config() -> ComputeDispatchConfig {
    ComputeDispatchConfig {
        arch: detect_arch(),
        os: detect_os(),
        cpu: default_cpu_config(),
        gpu: default_gpu_config(),
        display: default_display_config(),
        ram: default_ram_config(),
    }
}

pub(crate) fn default_ram_config() -> RamConfig {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let r = super::x86::os::default_ram_config();
        return RamConfig {
            page_size: r.page_size,
            total_bytes: r.total_bytes,
            available_bytes: r.available_bytes,
            frame_budget_us: r.frame_budget_us,
            low_power: r.low_power,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let r = super::arm::os::default_ram_config();
        return RamConfig {
            page_size: r.page_size,
            total_bytes: r.total_bytes,
            available_bytes: r.available_bytes,
            frame_budget_us: r.frame_budget_us,
            low_power: r.low_power,
        };
    }
    #[allow(unreachable_code)]
    RamConfig {
        page_size: 4096,
        total_bytes: 0,
        available_bytes: None,
        frame_budget_us: 8_333,
        low_power: false,
    }
}

pub(crate) fn default_cpu_config() -> CpuConfig {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let c = super::x86::os::default_cpu_config();
        return CpuConfig {
            vendor: map_vendor_x86(c.vendor),
            worker_hint: c.worker_hint,
            render_workers: c.render_workers,
            frame_budget_us: c.frame_budget_us,
            low_power: c.low_power,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let c = super::arm::os::default_cpu_config();
        return CpuConfig {
            vendor: map_vendor_arm(c.vendor),
            worker_hint: c.worker_hint,
            render_workers: c.render_workers,
            frame_budget_us: c.frame_budget_us,
            low_power: c.low_power,
        };
    }
    #[allow(unreachable_code)]
    CpuConfig {
        vendor: Vendor::Unknown,
        worker_hint: 1,
        render_workers: 1,
        frame_budget_us: 8_333,
        low_power: false,
    }
}

pub(crate) fn default_gpu_config() -> GpuConfig {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let c = super::x86::os::default_gpu_config();
        return GpuConfig {
            vendor: map_vendor_x86(c.vendor),
            workgroup_size: c.workgroup_size,
            compute_queues: c.compute_queues,
            render_threads: c.render_threads,
            double_buffered: c.double_buffered,
            frame_budget_us: c.frame_budget_us,
            low_power: c.low_power,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let c = super::arm::os::default_gpu_config();
        return GpuConfig {
            vendor: map_vendor_arm(c.vendor),
            workgroup_size: c.workgroup_size,
            compute_queues: c.compute_queues,
            render_threads: c.render_threads,
            double_buffered: c.double_buffered,
            frame_budget_us: c.frame_budget_us,
            low_power: c.low_power,
        };
    }
    #[allow(unreachable_code)]
    GpuConfig {
        vendor: Vendor::Unknown,
        workgroup_size: 1,
        compute_queues: 1,
        render_threads: 1,
        double_buffered: true,
        frame_budget_us: 8_333,
        low_power: false,
    }
}

pub(crate) fn default_display_config() -> DisplayConfig {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let c = super::x86::os::default_display_config();
        return DisplayConfig {
            vendor: map_vendor_x86(c.vendor),
            page_size: c.page_size,
            target_render_fps: c.target_render_fps,
            latency_budget_us: c.latency_budget_us,
            scan_out_latency_us: c.scan_out_latency_us,
            vsync_slots: c.vsync_slots,
            double_buffered: c.double_buffered,
            low_power: c.low_power,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let c = super::arm::os::default_display_config();
        return DisplayConfig {
            vendor: map_vendor_arm(c.vendor),
            page_size: c.page_size,
            target_render_fps: c.target_render_fps,
            latency_budget_us: c.latency_budget_us,
            scan_out_latency_us: c.scan_out_latency_us,
            vsync_slots: c.vsync_slots,
            double_buffered: c.double_buffered,
            low_power: c.low_power,
        };
    }
    #[allow(unreachable_code)]
    DisplayConfig {
        vendor: Vendor::Unknown,
        page_size: 4096,
        target_render_fps: 120,
        latency_budget_us: 8_333,
        scan_out_latency_us: 16_666,
        vsync_slots: 4,
        double_buffered: true,
        low_power: false,
    }
}

pub(crate) fn build_cpu_schedule(work_items: usize) -> Schedule {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let s = super::x86::os::build_cpu_schedule(work_items);
        return Schedule {
            chunks: s.chunks,
            chunk_size: s.chunk_size,
            frame_budget_us: s.frame_budget_us,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let s = super::arm::os::build_cpu_schedule(work_items);
        return Schedule {
            chunks: s.chunks,
            chunk_size: s.chunk_size,
            frame_budget_us: s.frame_budget_us,
        };
    }
    #[allow(unreachable_code)]
    Schedule {
        chunks: 1,
        chunk_size: 1,
        frame_budget_us: 8_333,
    }
}

pub(crate) fn build_gpu_schedule(work_items: usize) -> Schedule {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let s = super::x86::os::build_gpu_schedule(work_items);
        return Schedule {
            chunks: s.chunks,
            chunk_size: s.chunk_size,
            frame_budget_us: s.frame_budget_us,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let s = super::arm::os::build_gpu_schedule(work_items);
        return Schedule {
            chunks: s.chunks,
            chunk_size: s.chunk_size,
            frame_budget_us: s.frame_budget_us,
        };
    }
    #[allow(unreachable_code)]
    Schedule {
        chunks: 1,
        chunk_size: 1,
        frame_budget_us: 8_333,
    }
}

pub(crate) fn build_display_schedule(work_items: usize) -> Schedule {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let s = super::x86::os::build_display_schedule(work_items);
        return Schedule {
            chunks: s.chunks,
            chunk_size: s.chunk_size,
            frame_budget_us: s.frame_budget_us,
        };
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        let s = super::arm::os::build_display_schedule(work_items);
        return Schedule {
            chunks: s.chunks,
            chunk_size: s.chunk_size,
            frame_budget_us: s.frame_budget_us,
        };
    }
    #[allow(unreachable_code)]
    Schedule {
        chunks: 1,
        chunk_size: 1,
        frame_budget_us: 8_333,
    }
}

pub(crate) fn clamp_cpu_workers(requested: usize) -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        return super::x86::os::clamp_cpu_workers(requested);
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return super::arm::os::clamp_cpu_workers(requested);
    }
    #[allow(unreachable_code)]
    requested.max(1)
}

pub(crate) fn clamp_gpu_workers(requested: usize) -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        return super::x86::os::clamp_gpu_workers(requested);
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return super::arm::os::clamp_gpu_workers(requested);
    }
    #[allow(unreachable_code)]
    requested.max(1)
}

pub(crate) fn clamp_display_workers(requested: usize) -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        return super::x86::os::clamp_display_workers(requested);
    }
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    {
        return super::arm::os::clamp_display_workers(requested);
    }
    #[allow(unreachable_code)]
    requested.max(1)
}

    pub(crate) fn build_ram_schedule(work_items: usize) -> Schedule {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            let s = super::x86::os::build_ram_schedule(work_items);
            return Schedule {
                chunks: s.chunks,
                chunk_size: s.chunk_size,
                frame_budget_us: s.frame_budget_us,
            };
        }
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        {
            let s = super::arm::os::build_ram_schedule(work_items);
            return Schedule {
                chunks: s.chunks,
                chunk_size: s.chunk_size,
                frame_budget_us: s.frame_budget_us,
            };
        }
        #[allow(unreachable_code)]
        Schedule {
            chunks: 1,
            chunk_size: 1,
            frame_budget_us: 8_333,
        }
    }

    pub(crate) fn clamp_ram_workers(requested: usize) -> usize {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            return super::x86::os::clamp_ram_workers(requested);
        }
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        {
            return super::arm::os::clamp_ram_workers(requested);
        }
        #[allow(unreachable_code)]
        requested.max(1)
    }

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn map_os_x86(v: super::x86::os::Os) -> Os {
    match v {
        super::x86::os::Os::Linux => Os::Linux,
        super::x86::os::Os::Windows => Os::Windows,
        super::x86::os::Os::Macos => Os::Macos,
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
fn map_os_arm(v: super::arm::os::Os) -> Os {
    match v {
        super::arm::os::Os::Linux => Os::Linux,
        super::arm::os::Os::Windows => Os::Windows,
        super::arm::os::Os::Macos => Os::Macos,
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn map_vendor_x86(v: super::x86::os::Vendor) -> Vendor {
    match v {
        super::x86::os::Vendor::Amd => Vendor::Amd,
        super::x86::os::Vendor::Intel => Vendor::Intel,
        super::x86::os::Vendor::Apple => Vendor::Apple,
        super::x86::os::Vendor::Unknown => Vendor::Unknown,
    }
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
fn map_vendor_arm(v: super::arm::os::Vendor) -> Vendor {
    match v {
        super::arm::os::Vendor::Amd => Vendor::Amd,
        super::arm::os::Vendor::Intel => Vendor::Intel,
        super::arm::os::Vendor::Apple => Vendor::Apple,
        super::arm::os::Vendor::Unknown => Vendor::Unknown,
    }
}

// ─── Generic compute job types ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeDeviceKind {
    Gpu,
    CpuSimd,
    CpuScalar,
}

#[derive(Debug, Clone, Copy)]
pub struct ComputeCapabilities {
    pub kind: ComputeDeviceKind,
    pub max_workgroups: u32,
    pub max_workgroup_size: u32,
    pub parallel_lanes: u32,
    pub shared_memory_bytes: u32,
}

impl ComputeCapabilities {
    pub fn cpu_simd() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let lanes = {
            let has_avx2 = std::is_x86_feature_detected!("avx2");
            let has_avx = std::is_x86_feature_detected!("avx");
            if has_avx2 || has_avx { 8 } else { 4 }
        };
        #[cfg(target_arch = "aarch64")]
        let lanes = 4;
        #[cfg(target_arch = "arm")]
        let lanes = 2;
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64", target_arch = "arm")))]
        let lanes = 1;

        Self {
            kind: ComputeDeviceKind::CpuSimd,
            max_workgroups: 4096,
            max_workgroup_size: 256,
            parallel_lanes: lanes,
            shared_memory_bytes: 0,
        }
    }

    pub fn cpu_scalar() -> Self {
        Self {
            kind: ComputeDeviceKind::CpuScalar,
            max_workgroups: 1,
            max_workgroup_size: 1,
            parallel_lanes: 1,
            shared_memory_bytes: 0,
        }
    }

    pub fn gpu(
        max_workgroups: u32,
        max_workgroup_size: u32,
        parallel_lanes: u32,
        shared_memory_bytes: u32,
    ) -> Self {
        Self {
            kind: ComputeDeviceKind::Gpu,
            max_workgroups,
            max_workgroup_size,
            parallel_lanes,
            shared_memory_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KernelConfig {
    pub workgroup_size_x: u16,
    pub workgroup_size_y: u16,
    pub workgroup_size_z: u16,
    pub shared_memory_bytes: u32,
}

impl KernelConfig {
    pub fn new(x: u16, y: u16, z: u16) -> Self {
        Self {
            workgroup_size_x: x,
            workgroup_size_y: y,
            workgroup_size_z: z,
            shared_memory_bytes: 0,
        }
    }

    pub fn thread_count(&self) -> u32 {
        (self.workgroup_size_x as u32)
            * (self.workgroup_size_y as u32)
            * (self.workgroup_size_z as u32)
    }

    pub fn with_shared_memory(mut self, bytes: u32) -> Self {
        self.shared_memory_bytes = bytes;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComputeJob {
    pub job_id: u32,
    pub grid_x: u32,
    pub grid_y: u32,
    pub grid_z: u32,
    pub config: KernelConfig,
}

pub struct ComputeJobBatch {
    pub jobs: Vec<ComputeJob>,
    pub capacity: usize,
    pub kernel_tag: u32,
    pub kernel_size_bytes: u32,
    pub scene_signature: u64,
    pub object_count: u32,
    pub triangle_count: u32,
}

impl ComputeJobBatch {
    pub fn new(capacity: usize) -> Self {
        Self {
            jobs: Vec::new(),
            capacity,
            kernel_tag: 0,
            kernel_size_bytes: 0,
            scene_signature: 0,
            object_count: 0,
            triangle_count: 0,
        }
    }

    pub fn set_metadata(
        &mut self,
        kernel_tag: u32,
        kernel_size_bytes: u32,
        scene_signature: u64,
        object_count: u32,
        triangle_count: u32,
    ) {
        self.kernel_tag = kernel_tag;
        self.kernel_size_bytes = kernel_size_bytes;
        self.scene_signature = scene_signature;
        self.object_count = object_count;
        self.triangle_count = triangle_count;
    }

    pub fn push_job(
        &mut self,
        job_id: u32,
        grid_x: u32,
        grid_y: u32,
        grid_z: u32,
        config: KernelConfig,
    ) -> bool {
        if self.jobs.len() >= self.capacity {
            return false;
        }
        self.jobs.push(ComputeJob { job_id, grid_x, grid_y, grid_z, config });
        true
    }

    pub fn total_threads(&self) -> u64 {
        self.jobs
            .iter()
            .map(|job| {
                (job.grid_x as u64)
                    * (job.grid_y as u64)
                    * (job.grid_z as u64)
                    * (job.config.thread_count() as u64)
            })
            .sum()
    }

    pub fn clear(&mut self) {
        self.jobs.clear();
        self.kernel_tag = 0;
        self.kernel_size_bytes = 0;
        self.scene_signature = 0;
        self.object_count = 0;
        self.triangle_count = 0;
    }
}

use std::sync::{Condvar, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct ComputeQueue {
    submitted_count: AtomicU32,
    completed_count: AtomicU32,
    idle_signal: (Mutex<()>, Condvar),
}

impl Default for ComputeQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeQueue {
    pub fn new() -> Self {
        Self {
            submitted_count: AtomicU32::new(0),
            completed_count: AtomicU32::new(0),
            idle_signal: (Mutex::new(()), Condvar::new()),
        }
    }

    pub fn submit_batch(&self, batch_size: u32) {
        self.submitted_count.fetch_add(batch_size, Ordering::Release);
    }

    pub fn mark_batch_complete(&self, batch_size: u32) {
        self.completed_count.fetch_add(batch_size, Ordering::Release);
        if self.is_idle() {
            self.idle_signal.1.notify_all();
        }
    }

    pub fn pending_jobs(&self) -> u32 {
        self.submitted_count.load(Ordering::Acquire)
            .saturating_sub(self.completed_count.load(Ordering::Acquire))
    }

    pub fn is_idle(&self) -> bool {
        self.pending_jobs() == 0
    }

    pub fn wait_idle(&self) {
        if self.is_idle() {
            return;
        }
        let mut guard = match self.idle_signal.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        while !self.is_idle() {
            guard = match self.idle_signal.1.wait(guard) {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
        }
    }
}

pub struct CommandBuffer {
    data: Vec<u8>,
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push_u32(&mut self, val: u32) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn push_u64(&mut self, val: u64) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        if self.is_empty() {
            return;
        }
        self.data.clear();
    }

    pub fn align_to(&mut self, alignment: usize) {
        while !self.data.len().is_multiple_of(alignment) {
            self.data.push(0);
        }
    }
}
