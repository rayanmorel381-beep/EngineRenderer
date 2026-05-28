use std::fs;
use std::path::PathBuf;

use crate::editor::Editor;
use crate::scene::{ObjectId, ObjectKind, PrimitiveKind, Scene, SceneObject};

#[derive(Copy, Clone, Debug, Default)]
pub struct LayoutSnapshot {
    pub right_active_tab: usize,
    pub bottom_active_tab: usize,
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("enginerenderer").join("editor-layout.cfg")
}

pub fn capture(editor: &Editor) -> LayoutSnapshot {
    LayoutSnapshot {
        right_active_tab: editor.right_active_tab,
        bottom_active_tab: editor.bottom_active_tab,
    }
}

pub fn restore(editor: &mut Editor, snapshot: LayoutSnapshot) {
    editor.right_active_tab = snapshot.right_active_tab;
    editor.bottom_active_tab = snapshot.bottom_active_tab;
}

pub fn save(snapshot: LayoutSnapshot) -> std::io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = format!(
        "right_active_tab={}\nbottom_active_tab={}\n",
        snapshot.right_active_tab, snapshot.bottom_active_tab
    );
    fs::write(path, body)
}

pub fn load() -> Option<LayoutSnapshot> {
    let path = config_path();
    let body = fs::read_to_string(path).ok()?;
    let mut snapshot = LayoutSnapshot::default();
    for line in body.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value: usize = value.trim().parse().ok()?;
        match key.trim() {
            "right_active_tab" => snapshot.right_active_tab = value,
            "bottom_active_tab" => snapshot.bottom_active_tab = value,
            _ => {}
        }
    }
    Some(snapshot)
}

pub fn scene_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("enginerenderer").join("scene.ruxel")
}

pub fn save_scene(scene: &Scene) -> std::io::Result<()> {
    let path = scene_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut body = String::new();
    for obj in &scene.objects {
        let ObjectKind::Primitive(kind) = &obj.kind else {
            continue;
        };
        let name = obj.name.replace('%', "%25").replace(' ', "%20").replace('=', "%3D");
        body.push_str(&format!(
            "obj id={} kind={} name={} px={} py={} pz={} rx={} ry={} rz={} sx={} sy={} sz={} intensity={} visible={}\n",
            obj.id.0,
            kind.label().replace(' ', "%20"),
            name,
            obj.position[0], obj.position[1], obj.position[2],
            obj.rotation[0], obj.rotation[1], obj.rotation[2],
            obj.scale[0], obj.scale[1], obj.scale[2],
            obj.intensity,
            if obj.visible { 1 } else { 0 },
        ));
    }
    fs::write(path, body)
}

fn url_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex = format!("{h1}{h2}");
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                out.push(byte as char);
            } else {
                out.push('%');
                out.push(h1);
                out.push(h2);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn label_to_kind(label: &str) -> Option<PrimitiveKind> {
    match label {
        "Empty" => Some(PrimitiveKind::Empty),
        "Cube" => Some(PrimitiveKind::Cube),
        "Sphere" => Some(PrimitiveKind::Sphere),
        "Plane" => Some(PrimitiveKind::Plane),
        "Cylinder" => Some(PrimitiveKind::Cylinder),
        "Cone" => Some(PrimitiveKind::Cone),
        "Torus" => Some(PrimitiveKind::Torus),
        "Icosphere" => Some(PrimitiveKind::Icosphere),
        "Capsule" => Some(PrimitiveKind::Capsule),
        "Hypercube%204D" => Some(PrimitiveKind::Hypercube4D),
        "Simplex%204D" => Some(PrimitiveKind::Simplex4D),
        "Camera" => Some(PrimitiveKind::Camera),
        "Directional%20Light" => Some(PrimitiveKind::DirectionalLight),
        "Point%20Light" => Some(PrimitiveKind::PointLight),
        "Spot%20Light" => Some(PrimitiveKind::SpotLight),
        _ => None,
    }
}

pub fn load_scene() -> Option<Scene> {
    let path = scene_path();
    let body = fs::read_to_string(path).ok()?;
    let mut scene = Scene::new();
    for line in body.lines() {
        if !line.starts_with("obj ") {
            continue;
        }
        let mut id = 0u64;
        let mut kind_str = String::new();
        let mut name = String::new();
        let mut position = [0.0f64; 3];
        let mut rotation = [0.0f64; 3];
        let mut scale = [1.0f64; 3];
        let mut intensity = 1.0f64;
        let mut visible = true;
        for token in line["obj ".len()..].split_ascii_whitespace() {
            let Some((k, v)) = token.split_once('=') else {
                continue;
            };
            match k {
                "id" => {
                    id = v.parse().unwrap_or(0);
                }
                "kind" => {
                    kind_str = v.to_string();
                }
                "name" => {
                    name = url_decode(v);
                }
                "px" => {
                    position[0] = v.parse().unwrap_or(0.0);
                }
                "py" => {
                    position[1] = v.parse().unwrap_or(0.0);
                }
                "pz" => {
                    position[2] = v.parse().unwrap_or(0.0);
                }
                "rx" => {
                    rotation[0] = v.parse().unwrap_or(0.0);
                }
                "ry" => {
                    rotation[1] = v.parse().unwrap_or(0.0);
                }
                "rz" => {
                    rotation[2] = v.parse().unwrap_or(0.0);
                }
                "sx" => {
                    scale[0] = v.parse().unwrap_or(1.0);
                }
                "sy" => {
                    scale[1] = v.parse().unwrap_or(1.0);
                }
                "sz" => {
                    scale[2] = v.parse().unwrap_or(1.0);
                }
                "intensity" => {
                    intensity = v.parse().unwrap_or(1.0);
                }
                "visible" => {
                    visible = v != "0";
                }
                _ => {}
            }
        }
        let Some(kind) = label_to_kind(&kind_str) else {
            continue;
        };
        let obj = SceneObject {
            id: ObjectId(id),
            name: if name.is_empty() {
                kind.label().to_string()
            } else {
                name
            },
            kind: ObjectKind::Primitive(kind),
            position,
            rotation,
            scale,
            intensity,
            visible,
            animator: None,
            physics: None,
            scripts: Vec::new(),
            material_index: None,
            particles: Vec::new(),
            skeleton: None,
            lod_group: None,
            audio_source: None,
            cloth: None,
            soft_body: None,
            ragdoll: None,
            network: None,
            node_graph: None,
            behavior_tree: None,
            ik_rig: None,
            vfx_graph: None,
            spline_deformer: None,
            foliage: None,
            water: None,
            decals: None,
            destruction: None,
            ai_perception: None,
        };
        scene.restore(obj);
    }
    if scene.is_empty() {
        None
    } else {
        Some(scene)
    }
}
