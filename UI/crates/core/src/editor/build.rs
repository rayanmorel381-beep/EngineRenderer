#[derive(Clone, Debug, PartialEq)]
pub enum BuildTarget {
    DesktopLinux,
    DesktopWindows,
    DesktopMacos,
    Android,
    Ios,
    Wasm,
}

impl BuildTarget {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DesktopLinux => "Linux x86_64",
            Self::DesktopWindows => "Windows x86_64",
            Self::DesktopMacos => "macOS aarch64",
            Self::Android => "Android (AArch64)",
            Self::Ios => "iOS (AArch64)",
            Self::Wasm => "WebAssembly",
        }
    }
    pub const ALL: [BuildTarget; 6] = [
        BuildTarget::DesktopLinux, BuildTarget::DesktopWindows, BuildTarget::DesktopMacos,
        BuildTarget::Android, BuildTarget::Ios, BuildTarget::Wasm,
    ];
}

#[derive(Clone, Debug)]
pub struct BuildConfig {
    pub target: BuildTarget,
    pub output_path: String,
    pub release_mode: bool,
    pub bundle_assets: bool,
    pub strip_debug: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: BuildTarget::DesktopLinux,
            output_path: String::from("./dist"),
            release_mode: true,
            bundle_assets: true,
            strip_debug: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BuildStatus {
    Idle,
    Building(f64),
    Success,
    Failed(String),
}

impl BuildStatus {
    pub fn label(&self) -> String {
        match self {
            Self::Idle => "Prêt".to_owned(),
            Self::Building(p) => format!("Construction… {:.0}%", p * 100.0),
            Self::Success => "Succès".to_owned(),
            Self::Failed(e) => format!("Erreur: {e}"),
        }
    }
}
