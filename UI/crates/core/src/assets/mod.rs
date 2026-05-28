use std::path::{Path, PathBuf};

use crate::ui::style::icons::Icon;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssetKind {
    Mesh,
    Texture,
    Material,
    Scene,
    Unknown,
}

impl AssetKind {
    pub fn icon(self) -> Icon {
        match self {
            Self::Mesh => Icon::Mesh,
            Self::Texture => Icon::Texture,
            Self::Material => Icon::Material,
            Self::Scene => Icon::Scene,
            Self::Unknown => Icon::Folder,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Mesh => "Mesh",
            Self::Texture => "Texture",
            Self::Material => "Material",
            Self::Scene => "Scene",
            Self::Unknown => "Asset",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AssetFormat {
    Obj,
    Glb,
    Gltf,
    Fbx,
    Blend,
    Png,
    Jpeg,
    Ktx,
    Hdr,
    Mtl,
    Json,
    Toml,
    Other,
}

impl AssetFormat {
    pub fn from_extension(ext: &str) -> Self {
        let lower = ext.to_ascii_lowercase();
        match lower.as_str() {
            "obj" => Self::Obj,
            "glb" => Self::Glb,
            "gltf" => Self::Gltf,
            "fbx" => Self::Fbx,
            "blend" => Self::Blend,
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            "ktx" | "ktx2" => Self::Ktx,
            "hdr" | "exr" => Self::Hdr,
            "mtl" => Self::Mtl,
            "json" => Self::Json,
            "toml" => Self::Toml,
            _ => Self::Other,
        }
    }

    pub fn kind(self) -> AssetKind {
        match self {
            Self::Obj | Self::Glb | Self::Gltf | Self::Fbx | Self::Blend => AssetKind::Mesh,
            Self::Png | Self::Jpeg | Self::Ktx | Self::Hdr => AssetKind::Texture,
            Self::Mtl => AssetKind::Material,
            Self::Json | Self::Toml => AssetKind::Scene,
            Self::Other => AssetKind::Unknown,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Obj => "OBJ",
            Self::Glb => "GLB",
            Self::Gltf => "glTF",
            Self::Fbx => "FBX",
            Self::Blend => "Blender",
            Self::Png => "PNG",
            Self::Jpeg => "JPEG",
            Self::Ktx => "KTX",
            Self::Hdr => "HDR",
            Self::Mtl => "MTL",
            Self::Json => "JSON",
            Self::Toml => "TOML",
            Self::Other => "File",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImportStatus {
    Loaded,
    Stub,
    Failed,
}

#[derive(Clone, Debug)]
pub struct ImportedAsset {
    pub display_name: String,
    pub source_path: PathBuf,
    pub kind: AssetKind,
    pub format: AssetFormat,
    pub status: ImportStatus,
    pub message: String,
    pub byte_size: u64,
    pub vertex_count: usize,
    pub index_count: usize,
}

impl ImportedAsset {
    pub fn icon(&self) -> Icon {
        self.kind.icon()
    }
}

pub struct ImportOutcome {
    pub asset_index: usize,
    pub asset: ImportedAsset,
}

#[derive(Default)]
pub struct AssetRegistry {
    assets: Vec<ImportedAsset>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn assets(&self) -> &[ImportedAsset] {
        &self.assets
    }

    pub fn get(&self, index: usize) -> Option<&ImportedAsset> {
        self.assets.get(index)
    }

    pub fn import_path(&mut self, raw: impl AsRef<Path>) -> ImportOutcome {
        let path = raw.as_ref().to_path_buf();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let format = AssetFormat::from_extension(&extension);
        let display_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("imported")
            .to_string();
        let metadata_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let mut asset = ImportedAsset {
            display_name,
            source_path: path.clone(),
            kind: format.kind(),
            format,
            status: ImportStatus::Stub,
            message: format!("registered ({})", format.label()),
            byte_size: metadata_size,
            vertex_count: 0,
            index_count: 0,
        };
        match format {
            AssetFormat::Obj => apply_obj_import(&mut asset, &path),
            AssetFormat::Glb => apply_glb_import(&mut asset, &path),
            AssetFormat::Gltf | AssetFormat::Fbx | AssetFormat::Blend => {
                asset.status = ImportStatus::Stub;
                asset.message = format!(
                    "{} import not yet implemented; asset registered without geometry",
                    format.label()
                );
            }
            _ => {
                asset.status = ImportStatus::Stub;
                asset.message = format!("{} stored as raw asset", format.label());
            }
        }
        let asset_index = self.assets.len();
        self.assets.push(asset.clone());
        ImportOutcome {
            asset_index,
            asset,
        }
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

fn apply_obj_import(asset: &mut ImportedAsset, path: &Path) {
    use enginerenderer::api::loaders::obj::ObjLoader;
    let loader = ObjLoader;
    match loader.load_from_path(path) {
        Ok(mesh) => {
            asset.status = ImportStatus::Loaded;
            asset.vertex_count = mesh.vertices.len();
            asset.index_count = mesh.indices.len();
            asset.message = format!(
                "OBJ loaded: {} vertices, {} indices",
                mesh.vertices.len(),
                mesh.indices.len()
            );
        }
        Err(err) => {
            asset.status = ImportStatus::Failed;
            asset.message = format!("OBJ import failed: {err}");
        }
    }
}

fn apply_glb_import(asset: &mut ImportedAsset, path: &Path) {
    use enginerenderer::api::loaders::glb::{GLB_HEADER_SIZE, MAX_GLB_FILE_SIZE, validate_glb_header};
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(err) => {
            asset.status = ImportStatus::Failed;
            asset.message = format!("GLB read failed: {err}");
            return;
        }
    };
    if bytes.len() < GLB_HEADER_SIZE || bytes.len() as u64 > MAX_GLB_FILE_SIZE {
        asset.status = ImportStatus::Failed;
        asset.message = format!("GLB rejected: invalid size {}", bytes.len());
        return;
    }
    match validate_glb_header(&bytes) {
        Ok(header) => {
            asset.status = ImportStatus::Loaded;
            asset.message = format!("GLB validated: {} bytes", header.declared_length);
        }
        Err(err) => {
            asset.status = ImportStatus::Failed;
            asset.message = format!("GLB validation failed: {err:?}");
        }
    }
}
