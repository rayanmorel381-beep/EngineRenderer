
use std::time::Instant;

use crate::api::engine::engine_api::EngineApi;
use crate::core::engine::acces_hardware::arch::compute_dispatch;
use crate::core::engine::acces_hardware::NativeHardwareBackend;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Supported compute CPU architecture families.
pub enum ComputeArch {
    /// x86 and x86_64 architectures.
    X86,
    /// ARM and AArch64 architectures.
    Arm,
}

impl ComputeArch {
    /// Returns a stable lowercase identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::Arm => "arm",
        }
    }

    /// Parses an architecture identifier.
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "x86" | "x86_64" | "amd64" => Some(Self::X86),
            "arm" | "aarch64" | "arm64" => Some(Self::Arm),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Supported operating systems for diagnostics override.
pub enum ComputeOs {
    /// Linux-based systems.
    Linux,
    /// Windows systems.
    Windows,
    /// macOS systems.
    Macos,
}

impl ComputeOs {
    /// Returns a stable lowercase identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
        }
    }

    /// Parses an operating-system identifier.
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "linux" => Some(Self::Linux),
            "windows" | "win" => Some(Self::Windows),
            "macos" | "mac" | "darwin" => Some(Self::Macos),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Supported hardware vendor categories used by diagnostics.
pub enum ComputeVendor {
    /// AMD vendor bucket.
    Amd,
    /// Intel vendor bucket.
    Intel,
    /// Apple vendor bucket.
    Apple,
    /// Unknown or unsupported vendor bucket.
    Unknown,
}

impl ComputeVendor {
    /// Returns a stable lowercase identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Amd => "amd",
            Self::Intel => "intel",
            Self::Apple => "apple",
            Self::Unknown => "unknown",
        }
    }

    /// Parses a vendor identifier.
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "amd" => Some(Self::Amd),
            "intel" => Some(Self::Intel),
            "apple" => Some(Self::Apple),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Selectable diagnostics output component.
pub enum DiagnosticComponent {
    /// CPU section.
    Cpu,
    /// GPU section.
    Gpu,
    /// RAM section.
    Ram,
    /// Display section.
    Display,
}

impl DiagnosticComponent {
    /// Returns a stable lowercase identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Gpu => "gpu",
            Self::Ram => "ram",
            Self::Display => "display",
        }
    }

    /// Parses a diagnostics component identifier.
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "cpu" => Some(Self::Cpu),
            "gpu" => Some(Self::Gpu),
            "ram" => Some(Self::Ram),
            "display" | "screen" => Some(Self::Display),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// Optional platform overrides used for synthetic diagnostics runs.
pub struct DiagnosticOverrides {
    /// Optional architecture override.
    pub arch: Option<ComputeArch>,
    /// Optional operating-system override.
    pub os: Option<ComputeOs>,
    /// Optional vendor override.
    pub vendor: Option<ComputeVendor>,
}

#[derive(Clone, Copy, Debug, Default)]
/// Diagnostics execution options.
pub struct DiagnosticsOptions {
    /// Enables JSON output mode.
    pub json: bool,
    /// Enables verbose output mode.
    pub verbose: bool,
    /// Enables synthetic benchmark computation.
    pub bench: bool,
    /// Restricts output to a single component.
    pub component: Option<DiagnosticComponent>,
    /// Platform override values.
    pub overrides: DiagnosticOverrides,
}

#[derive(Clone, Debug)]
/// Generic schedule shape report.
pub struct ScheduleReport {
    /// Number of work chunks.
    pub chunks: usize,
    /// Chunk size in work items.
    pub chunk_size: usize,
    /// Target budget in microseconds.
    pub frame_budget_us: u64,
}

#[derive(Clone, Debug)]
/// CPU diagnostics report section.
pub struct CpuReport {
    /// Selected vendor category.
    pub vendor: ComputeVendor,
    /// Worker count hint from dispatch configuration.
    pub worker_hint: usize,
    /// Effective render worker count.
    pub render_workers: usize,
    /// CPU frame budget in microseconds.
    pub frame_budget_us: u64,
    /// Whether low-power mode is enabled.
    pub low_power: bool,
    /// CPU scheduling report.
    pub schedule: ScheduleReport,
}

#[derive(Clone, Debug)]
/// GPU diagnostics report section.
pub struct GpuReport {
    /// Selected vendor category.
    pub vendor: ComputeVendor,
    /// Compute workgroup size.
    pub workgroup_size: usize,
    /// Number of compute queues.
    pub compute_queues: usize,
    /// Render thread count.
    pub render_threads: usize,
    /// Whether double buffering is enabled.
    pub double_buffered: bool,
    /// GPU frame budget in microseconds.
    pub frame_budget_us: u64,
    /// Whether low-power mode is enabled.
    pub low_power: bool,
    /// GPU scheduling report.
    pub schedule: ScheduleReport,
}

#[derive(Clone, Debug)]
/// Display diagnostics report section.
pub struct DisplayReport {
    /// Selected vendor category.
    pub vendor: ComputeVendor,
    /// Display page size in bytes.
    pub page_size: usize,
    /// Target render FPS for display pacing.
    pub target_render_fps: u32,
    /// Display latency budget in microseconds.
    pub latency_budget_us: u64,
    /// Scan-out latency in microseconds.
    pub scan_out_latency_us: u64,
    /// Number of vsync slots.
    pub vsync_slots: usize,
    /// Whether double buffering is enabled.
    pub double_buffered: bool,
    /// Whether low-power mode is enabled.
    pub low_power: bool,
    /// Display scheduling report.
    pub schedule: ScheduleReport,
}

#[derive(Clone, Debug)]
/// RAM diagnostics report section.
pub struct RamReport {
    /// Memory page size in bytes.
    pub page_size: usize,
    /// Total RAM in bytes.
    pub total_bytes: u64,
    /// Available RAM in bytes when available.
    pub available_bytes: Option<u64>,
    /// RAM frame budget in microseconds.
    pub frame_budget_us: u64,
    /// Whether low-power mode is enabled.
    pub low_power: bool,
    /// RAM scheduling report.
    pub schedule: ScheduleReport,
}

#[derive(Clone, Debug)]
/// Detected hardware capability report.
pub struct HardwareReport {
    /// Number of logical CPU cores.
    pub logical_cores: u32,
    /// Reported VRAM capacity in bytes.
    pub vram_bytes: u64,
    /// Reported RAM capacity in bytes.
    pub total_ram_bytes: u64,
    /// Recommended render thread count.
    pub optimal_render_threads: usize,
    /// Maximum framebuffer allocation size in bytes.
    pub max_framebuffer_bytes: u64,
    /// Maximum single GPU allocation size in bytes.
    pub max_gpu_allocation_bytes: u64,
}

#[derive(Clone, Debug)]
/// Synthetic benchmark summary.
pub struct BenchmarkReport {
    /// Number of benchmark iterations.
    pub iterations: usize,
    /// Total benchmark duration in milliseconds.
    pub total_ms: u128,
    /// Average iteration duration in microseconds.
    pub avg_us: u128,
}

#[derive(Clone, Debug)]
/// Full compute-environment diagnostics report.
pub struct ComputeEnvironmentReport {
    /// Selected architecture.
    pub arch: ComputeArch,
    /// Selected operating system.
    pub os: ComputeOs,
    /// CPU section.
    pub cpu: CpuReport,
    /// GPU section.
    pub gpu: GpuReport,
    /// Display section.
    pub display: DisplayReport,
    /// RAM section.
    pub ram: RamReport,
    /// Hardware capabilities section.
    pub hardware: HardwareReport,
    /// Optional benchmark section.
    pub benchmark: Option<BenchmarkReport>,
    /// Effective override configuration.
    pub overrides: DiagnosticOverrides,
}

impl EngineApi {
    /// Builds a structured compute-environment report.
    pub fn compute_environment_report(&self, options: &DiagnosticsOptions) -> ComputeEnvironmentReport {
        let config = compute_dispatch::default_config();
        let backend = NativeHardwareBackend::detect();
        let hardware = backend.hw_caps().clone();

        let mut report = ComputeEnvironmentReport {
            arch: map_arch(config.arch),
            os: map_os(config.os),
            cpu: CpuReport {
                vendor: map_vendor(config.cpu.vendor),
                worker_hint: config.cpu.worker_hint,
                render_workers: config.cpu.render_workers,
                frame_budget_us: config.cpu.frame_budget_us,
                low_power: config.cpu.low_power,
                schedule: to_schedule(compute_dispatch::build_cpu_schedule(config.cpu.render_workers.max(1))),
            },
            gpu: GpuReport {
                vendor: map_vendor(config.gpu.vendor),
                workgroup_size: config.gpu.workgroup_size,
                compute_queues: config.gpu.compute_queues,
                render_threads: config.gpu.render_threads,
                double_buffered: config.gpu.double_buffered,
                frame_budget_us: config.gpu.frame_budget_us,
                low_power: config.gpu.low_power,
                schedule: to_schedule(compute_dispatch::build_gpu_schedule(config.gpu.render_threads.max(1))),
            },
            display: DisplayReport {
                vendor: map_vendor(config.display.vendor),
                page_size: config.display.page_size,
                target_render_fps: config.display.target_render_fps,
                latency_budget_us: config.display.latency_budget_us,
                scan_out_latency_us: config.display.scan_out_latency_us,
                vsync_slots: config.display.vsync_slots,
                double_buffered: config.display.double_buffered,
                low_power: config.display.low_power,
                schedule: to_schedule(compute_dispatch::build_display_schedule(config.display.vsync_slots.max(1))),
            },
            ram: RamReport {
                page_size: config.ram.page_size,
                total_bytes: config.ram.total_bytes,
                available_bytes: config.ram.available_bytes,
                frame_budget_us: config.ram.frame_budget_us,
                low_power: config.ram.low_power,
                schedule: to_schedule(compute_dispatch::build_ram_schedule(config.cpu.render_workers.max(1))),
            },
            hardware: HardwareReport {
                logical_cores: hardware.logical_cores,
                vram_bytes: hardware.vram_bytes,
                total_ram_bytes: config.ram.total_bytes,
                optimal_render_threads: hardware.optimal_render_threads(),
                max_framebuffer_bytes: hardware.max_framebuffer_bytes(),
                max_gpu_allocation_bytes: hardware.max_gpu_allocation_bytes(),
            },
            benchmark: None,
            overrides: options.overrides,
        };

        apply_overrides(&mut report, options.overrides);

        if options.bench {
            report.benchmark = Some(run_benchmark(report.hardware.logical_cores));
        }

        report
    }

    /// Prints diagnostics using the provided options.
    pub fn diagnose_compute_environment(&self, options: &DiagnosticsOptions) {
        let report = self.compute_environment_report(options);
        if options.json {
            eprintln!("{}", report.to_json(options.component, options.verbose));
            return;
        }
        report.print_text(options.component, options.verbose);
        if options.verbose {
            crate::core::engine::rendering::shader_dispatcher::diagnose_compute_environment();
        }
    }
}

impl ComputeEnvironmentReport {
    /// Prints a human-readable diagnostics summary.
    pub fn print_text(&self, component: Option<DiagnosticComponent>, verbose: bool) {
        eprintln!("compute-detect: arch={} os={}", self.arch.as_str(), self.os.as_str());
        if let Some(arch) = self.overrides.arch {
            eprintln!("override: arch={}", arch.as_str());
        }
        if let Some(os) = self.overrides.os {
            eprintln!("override: os={}", os.as_str());
        }
        if let Some(vendor) = self.overrides.vendor {
            eprintln!("override: vendor={}", vendor.as_str());
        }

        if component.is_none() || component == Some(DiagnosticComponent::Cpu) {
            eprintln!(
                "cpu: vendor={} worker_hint={} render_workers={} frame_budget_us={} low_power={} schedule={}/{}/{}",
                self.cpu.vendor.as_str(),
                self.cpu.worker_hint,
                self.cpu.render_workers,
                self.cpu.frame_budget_us,
                self.cpu.low_power,
                self.cpu.schedule.chunks,
                self.cpu.schedule.chunk_size,
                self.cpu.schedule.frame_budget_us
            );
        }

        if component.is_none() || component == Some(DiagnosticComponent::Gpu) {
            eprintln!(
                "gpu: vendor={} workgroup_size={} queues={} render_threads={} double_buffered={} frame_budget_us={} low_power={} schedule={}/{}/{}",
                self.gpu.vendor.as_str(),
                self.gpu.workgroup_size,
                self.gpu.compute_queues,
                self.gpu.render_threads,
                self.gpu.double_buffered,
                self.gpu.frame_budget_us,
                self.gpu.low_power,
                self.gpu.schedule.chunks,
                self.gpu.schedule.chunk_size,
                self.gpu.schedule.frame_budget_us
            );
        }

        if component.is_none() || component == Some(DiagnosticComponent::Display) {
            eprintln!(
                "display: vendor={} page_size={} target_render_fps={} latency_budget_us={} scan_out_latency_us={} vsync_slots={} double_buffered={} low_power={} schedule={}/{}/{}",
                self.display.vendor.as_str(),
                self.display.page_size,
                self.display.target_render_fps,
                self.display.latency_budget_us,
                self.display.scan_out_latency_us,
                self.display.vsync_slots,
                self.display.double_buffered,
                self.display.low_power,
                self.display.schedule.chunks,
                self.display.schedule.chunk_size,
                self.display.schedule.frame_budget_us
            );
        }

        if component.is_none() || component == Some(DiagnosticComponent::Ram) {
            eprintln!(
                "ram: page_size={} total_bytes={} available_bytes={} frame_budget_us={} low_power={} schedule={}/{}/{}",
                self.ram.page_size,
                self.ram.total_bytes,
                self.ram.available_bytes.map(|v| v.to_string()).unwrap_or_else(|| "null".to_string()),
                self.ram.frame_budget_us,
                self.ram.low_power,
                self.ram.schedule.chunks,
                self.ram.schedule.chunk_size,
                self.ram.schedule.frame_budget_us
            );
        }

        if verbose {
            eprintln!(
                "hardware: logical_cores={} vram_bytes={} total_ram_bytes={} optimal_render_threads={} max_framebuffer_bytes={} max_gpu_allocation_bytes={}",
                self.hardware.logical_cores,
                self.hardware.vram_bytes,
                self.hardware.total_ram_bytes,
                self.hardware.optimal_render_threads,
                self.hardware.max_framebuffer_bytes,
                self.hardware.max_gpu_allocation_bytes
            );
        }

        if let Some(bench) = &self.benchmark {
            eprintln!(
                "bench: iterations={} total_ms={} avg_us={}",
                bench.iterations,
                bench.total_ms,
                bench.avg_us
            );
        }
    }

    /// Serializes diagnostics to a JSON string.
    pub fn to_json(&self, component: Option<DiagnosticComponent>, verbose: bool) -> String {
        let mut fields: Vec<String> = Vec::new();
        fields.push(format!("\"arch\":\"{}\"", self.arch.as_str()));
        fields.push(format!("\"os\":\"{}\"", self.os.as_str()));

        if component.is_none() || component == Some(DiagnosticComponent::Cpu) {
            fields.push(format!(
                "\"cpu\":{{\"vendor\":\"{}\",\"worker_hint\":{},\"render_workers\":{},\"frame_budget_us\":{},\"low_power\":{},\"schedule\":{{\"chunks\":{},\"chunk_size\":{},\"frame_budget_us\":{}}}}}",
                self.cpu.vendor.as_str(),
                self.cpu.worker_hint,
                self.cpu.render_workers,
                self.cpu.frame_budget_us,
                self.cpu.low_power,
                self.cpu.schedule.chunks,
                self.cpu.schedule.chunk_size,
                self.cpu.schedule.frame_budget_us
            ));
        }

        if component.is_none() || component == Some(DiagnosticComponent::Gpu) {
            fields.push(format!(
                "\"gpu\":{{\"vendor\":\"{}\",\"workgroup_size\":{},\"compute_queues\":{},\"render_threads\":{},\"double_buffered\":{},\"frame_budget_us\":{},\"low_power\":{},\"schedule\":{{\"chunks\":{},\"chunk_size\":{},\"frame_budget_us\":{}}}}}",
                self.gpu.vendor.as_str(),
                self.gpu.workgroup_size,
                self.gpu.compute_queues,
                self.gpu.render_threads,
                self.gpu.double_buffered,
                self.gpu.frame_budget_us,
                self.gpu.low_power,
                self.gpu.schedule.chunks,
                self.gpu.schedule.chunk_size,
                self.gpu.schedule.frame_budget_us
            ));
        }

        if component.is_none() || component == Some(DiagnosticComponent::Display) {
            fields.push(format!(
                "\"display\":{{\"vendor\":\"{}\",\"page_size\":{},\"target_render_fps\":{},\"latency_budget_us\":{},\"scan_out_latency_us\":{},\"vsync_slots\":{},\"double_buffered\":{},\"low_power\":{},\"schedule\":{{\"chunks\":{},\"chunk_size\":{},\"frame_budget_us\":{}}}}}",
                self.display.vendor.as_str(),
                self.display.page_size,
                self.display.target_render_fps,
                self.display.latency_budget_us,
                self.display.scan_out_latency_us,
                self.display.vsync_slots,
                self.display.double_buffered,
                self.display.low_power,
                self.display.schedule.chunks,
                self.display.schedule.chunk_size,
                self.display.schedule.frame_budget_us
            ));
        }

        if component.is_none() || component == Some(DiagnosticComponent::Ram) {
            let available_bytes = self
                .ram
                .available_bytes
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".to_string());
            fields.push(format!(
                "\"ram\":{{\"page_size\":{},\"total_bytes\":{},\"available_bytes\":{},\"frame_budget_us\":{},\"low_power\":{},\"schedule\":{{\"chunks\":{},\"chunk_size\":{},\"frame_budget_us\":{}}}}}",
                self.ram.page_size,
                self.ram.total_bytes,
                available_bytes,
                self.ram.frame_budget_us,
                self.ram.low_power,
                self.ram.schedule.chunks,
                self.ram.schedule.chunk_size,
                self.ram.schedule.frame_budget_us
            ));
        }

        if verbose {
            fields.push(format!(
                "\"hardware\":{{\"logical_cores\":{},\"vram_bytes\":{},\"total_ram_bytes\":{},\"optimal_render_threads\":{},\"max_framebuffer_bytes\":{},\"max_gpu_allocation_bytes\":{}}}",
                self.hardware.logical_cores,
                self.hardware.vram_bytes,
                self.hardware.total_ram_bytes,
                self.hardware.optimal_render_threads,
                self.hardware.max_framebuffer_bytes,
                self.hardware.max_gpu_allocation_bytes
            ));
        }

        if self.overrides.arch.is_some() || self.overrides.os.is_some() || self.overrides.vendor.is_some() {
            fields.push(format!(
                "\"overrides\":{{\"arch\":{},\"os\":{},\"vendor\":{}}}",
                self.overrides.arch.map(|v| format!("\"{}\"", v.as_str())).unwrap_or_else(|| "null".to_string()),
                self.overrides.os.map(|v| format!("\"{}\"", v.as_str())).unwrap_or_else(|| "null".to_string()),
                self.overrides.vendor.map(|v| format!("\"{}\"", v.as_str())).unwrap_or_else(|| "null".to_string())
            ));
        }

        if let Some(bench) = &self.benchmark {
            fields.push(format!(
                "\"bench\":{{\"iterations\":{},\"total_ms\":{},\"avg_us\":{}}}",
                bench.iterations,
                bench.total_ms,
                bench.avg_us
            ));
        }

        format!("{{{}}}", fields.join(","))
    }
}

fn to_schedule(schedule: compute_dispatch::Schedule) -> ScheduleReport {
    ScheduleReport {
        chunks: schedule.chunks,
        chunk_size: schedule.chunk_size,
        frame_budget_us: schedule.frame_budget_us,
    }
}

fn map_arch(arch: compute_dispatch::Arch) -> ComputeArch {
    match arch {
        compute_dispatch::Arch::X86 => ComputeArch::X86,
        compute_dispatch::Arch::Arm => ComputeArch::Arm,
    }
}

fn map_os(os: compute_dispatch::Os) -> ComputeOs {
    match os {
        compute_dispatch::Os::Linux => ComputeOs::Linux,
        compute_dispatch::Os::Windows => ComputeOs::Windows,
        compute_dispatch::Os::Macos => ComputeOs::Macos,
    }
}

fn map_vendor(vendor: compute_dispatch::Vendor) -> ComputeVendor {
    match vendor {
        compute_dispatch::Vendor::Amd => ComputeVendor::Amd,
        compute_dispatch::Vendor::Intel => ComputeVendor::Intel,
        compute_dispatch::Vendor::Apple => ComputeVendor::Apple,
        compute_dispatch::Vendor::Unknown => ComputeVendor::Unknown,
    }
}

fn apply_overrides(report: &mut ComputeEnvironmentReport, overrides: DiagnosticOverrides) {
    if let Some(arch) = overrides.arch {
        report.arch = arch;
    }
    if let Some(os) = overrides.os {
        report.os = os;
    }
    if let Some(vendor) = overrides.vendor {
        report.cpu.vendor = vendor;
        report.gpu.vendor = vendor;
        report.display.vendor = vendor;
    }
}

fn run_benchmark(logical_cores: u32) -> BenchmarkReport {
    let iterations = (logical_cores as usize).saturating_mul(128).max(128);
    let start = Instant::now();
    let mut schedule_weight: u128 = 0;
    let mut worker_weight: u128 = 0;

    for i in 1..=iterations {
        let work_items = i.saturating_mul(64);
        let cpu = compute_dispatch::build_cpu_schedule(work_items);
        let gpu = compute_dispatch::build_gpu_schedule(work_items);
        let display = compute_dispatch::build_display_schedule(work_items);
        let ram = compute_dispatch::build_ram_schedule(work_items);
        let cpu_workers = compute_dispatch::clamp_cpu_workers(work_items);
        let gpu_workers = compute_dispatch::clamp_gpu_workers(work_items);
        let display_workers = compute_dispatch::clamp_display_workers(work_items);
        let ram_workers = compute_dispatch::clamp_ram_workers(work_items);
        schedule_weight = schedule_weight
            .saturating_add(cpu.chunks as u128)
            .saturating_add(gpu.chunks as u128)
            .saturating_add(display.chunks as u128)
            .saturating_add(ram.chunks as u128)
            .saturating_add(cpu.chunk_size as u128)
            .saturating_add(gpu.chunk_size as u128)
            .saturating_add(display.chunk_size as u128)
            .saturating_add(ram.chunk_size as u128);
        worker_weight = worker_weight
            .saturating_add(cpu_workers as u128)
            .saturating_add(gpu_workers as u128)
            .saturating_add(display_workers as u128)
            .saturating_add(ram_workers as u128);
    }

    let elapsed = start.elapsed();
    let synthetic_overhead = schedule_weight
        .saturating_add(worker_weight)
        .saturating_div(iterations as u128);
    let total_ms = elapsed.as_millis().saturating_add(synthetic_overhead / 1_000);
    let avg_us = elapsed.as_micros()
        .saturating_add(synthetic_overhead)
        / (iterations as u128);

    BenchmarkReport {
        iterations,
        total_ms,
        avg_us,
    }
}