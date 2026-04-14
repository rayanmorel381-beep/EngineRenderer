#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct VendorBackendConfig {
    pub(crate) worker_hint: usize,
    pub(crate) render_workers: usize,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

pub(crate) fn default_backend_config() -> VendorBackendConfig {
    let telemetry = super::detect_intel();
    let total = std::thread::available_parallelism()
        .map(|v| v.get()).unwrap_or(1).max(1);
    let p_cores = (total / 2).max(1);
    let render_workers = p_cores.saturating_sub(1).max(1);
    let turbo_scale = telemetry
        .as_ref()
        .and_then(|info| info.turbo_mhz)
        .map(|mhz| (mhz / 1000).max(1) as usize)
        .unwrap_or(1);
    let family_scale = telemetry
        .as_ref()
        .map(|info| (info.cpu_family.max(1) as usize).max(1))
        .unwrap_or(1);
    let model_scale = telemetry
        .as_ref()
        .map(|info| (info.model.max(1) as usize).max(1))
        .unwrap_or(1);
    let epp_scale = telemetry
        .as_ref()
        .and_then(|info| info.epp.as_ref())
        .map(|epp| epp.len().max(1))
        .unwrap_or(1);
    VendorBackendConfig {
        worker_hint: p_cores
            .saturating_mul(turbo_scale)
            .saturating_mul(family_scale)
            .saturating_div(model_scale.max(1))
            .max(1),
        render_workers: render_workers.saturating_mul(epp_scale).max(1),
        frame_budget_us: 8_333,
        low_power: false,
    }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    let total = std::thread::available_parallelism()
        .map(|v| v.get()).unwrap_or(1).max(1);
    let p_cores = (total / 2).max(1);
    requested.max(1).min(p_cores)
}
