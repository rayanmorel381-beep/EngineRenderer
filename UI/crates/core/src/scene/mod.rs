use crate::ui::style::icons::Icon;

pub mod animation;
pub mod audio;
pub mod behavior_tree;
pub mod cloth;
pub mod collision;
pub mod constraints;
pub mod dsp;
pub mod events;
pub mod fluid;
pub mod gi;
pub mod lod;
pub mod material;
pub mod mesh;
pub mod navmesh;
pub mod network;
pub mod node_graph;
pub mod particles;
pub mod physics;
pub mod post_process;
pub mod prefab;
pub mod script;
pub mod script_runtime;
pub mod skeleton;
pub mod softbody;
pub mod streaming;
pub mod terrain;
pub mod xr;
pub mod ai_perception;
pub mod data_table;
pub mod decal;
pub mod destruction;
pub mod foliage;
pub mod ik;
pub mod input_map;
pub mod job_scheduler;
pub mod localization;
pub mod pcg;
pub mod quest;
pub mod render_graph;
pub mod spline;
pub mod vfx_graph;
pub mod water;
pub use animation::{AnimTrack, AnimationClip, Animator, Easing, Keyframe};
pub use audio::{AudioMixer, AudioRolloff, AudioSource};
pub use behavior_tree::{BehaviorTree, Blackboard, BlackboardValue, BtNodeKind, BtStatus, ConditionOp};
pub use cloth::ClothMesh;
pub use collision::{Aabb, Bvh, CollisionPair, collect_collision_pairs};
pub use constraints::{ConstraintWorld, Joint, JointKind, Ragdoll, RagdollBone, Spring};
pub use dsp::DspChain;
pub use events::{EventBus, GameEvent, GameEventKind};
pub use fluid::FluidVolume;
pub use gi::GiSettings;
pub use lod::{LodGroup, LodLevel};
pub use material::{AlphaMode, MaterialLibrary, PbrMaterial, TextureRef};
pub use mesh::{EditMesh, SelectMode};
pub use navmesh::{NavAgent, NavMesh};
pub use network::{NetworkObject, NetworkRole, NetworkState, ReplicationConfig};
pub use node_graph::{NodeGraph, ScriptNode};
pub use particles::{EmitterShape, Particle, ParticleEmitter, SimSpace};
pub use physics::{Collider, ColliderShape, PhysicsBody};
pub use post_process::{BloomSettings, ColorGrading, DepthOfFieldSettings, PostProcessVolume, SsaoSettings, ToneMappingMode, VignetteSettings};
pub use prefab::{Prefab, PrefabInstance, PrefabLibrary, PrefabObject};
pub use script::ScriptComponent;
pub use script_runtime::ScriptRuntime;
pub use skeleton::{AnimStateMachine, Bone, BoneTrack, Skeleton, SkeletalClip, TransitionCondition};
pub use softbody::SoftBody;
pub use streaming::AssetStreamingConfig;
pub use terrain::{TerrainData, TerrainLayer};
pub use xr::XrSettings;
pub use ai_perception::{AiPerceptionComponent, AiPerceptionConfig, HearingConfig, PerceivedActor, SenseKind, SightConfig};
pub use data_table::{DataColumn, DataFieldType, DataRow, DataTable, DataTableLibrary, DataValue};
pub use decal::{Decal, DecalBlendMode, DecalLayer};
pub use destruction::{DestructionBody, DestructionChunk, FractureMode};
pub use foliage::{FoliageInstance, FoliagePaintMode, FoliagePainter, FoliageType};
pub use ik::{IkChain, IkKind, IkRig};
pub use input_map::{InputAction, InputAxis, InputBinding, InputDevice, InputMap, InputTrigger};
pub use job_scheduler::{JobHandle, JobPriority, JobScheduler, JobStatus};
pub use localization::{LocaleCode, LocaleEntry, LocaleKey, LocalizationTable};
pub use pcg::{PcgEdge, PcgGraph, PcgNode, PcgNodeKind};
pub use quest::{DialogGraph, DialogKind, DialogNode, Quest, QuestJournal, QuestObjective, QuestStatus};
pub use render_graph::{PassKind, RenderGraph, RenderPass};
pub use spline::{DeformerScaleMode, Spline, SplineKind, SplineMeshDeformer, SplinePoint};
pub use vfx_graph::{VfxGraph, VfxNode, VfxNodeCategory};
pub use water::{FloatingBody, GerstnerWave, WaterBody};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ObjectId(pub u64);

impl ObjectId {
    pub fn allocate(&mut self) -> Self {
        let id = self.0;
        self.0 = self.0.wrapping_add(1);
        Self(id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    Empty,
    Cube,
    Sphere,
    Plane,
    Cylinder,
    Cone,
    Torus,
    Icosphere,
    Capsule,
    Hypercube4D,
    Simplex4D,
    Camera,
    DirectionalLight,
    PointLight,
    SpotLight,
}

impl PrimitiveKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::Cube => "Cube",
            Self::Sphere => "Sphere",
            Self::Plane => "Plane",
            Self::Cylinder => "Cylinder",
            Self::Cone => "Cone",
            Self::Torus => "Torus",
            Self::Icosphere => "Icosphere",
            Self::Capsule => "Capsule",
            Self::Hypercube4D => "Hypercube 4D",
            Self::Simplex4D => "Simplex 4D",
            Self::Camera => "Camera",
            Self::DirectionalLight => "Directional Light",
            Self::PointLight => "Point Light",
            Self::SpotLight => "Spot Light",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::Empty => Icon::Scene,
            Self::Cube
            | Self::Sphere
            | Self::Plane
            | Self::Cylinder
            | Self::Cone
            | Self::Torus
            | Self::Icosphere
            | Self::Capsule
            | Self::Hypercube4D
            | Self::Simplex4D => Icon::Mesh,
            Self::Camera => Icon::Camera,
            Self::DirectionalLight | Self::PointLight | Self::SpotLight => Icon::Light,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ObjectKind {
    Primitive(PrimitiveKind),
    Mesh { asset_index: usize },
}

impl ObjectKind {
    pub fn icon(&self) -> Icon {
        match self {
            Self::Primitive(p) => p.icon(),
            Self::Mesh { .. } => Icon::Mesh,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SceneObject {
    pub id: ObjectId,
    pub name: String,
    pub kind: ObjectKind,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
    pub intensity: f64,
    pub visible: bool,
    pub animator: Option<Animator>,
    pub physics: Option<PhysicsBody>,
    pub scripts: Vec<ScriptComponent>,
    pub material_index: Option<usize>,
    pub particles: Vec<ParticleEmitter>,
    pub skeleton: Option<Skeleton>,
    pub lod_group: Option<LodGroup>,
    pub audio_source: Option<AudioSource>,
    pub cloth: Option<ClothMesh>,
    pub soft_body: Option<SoftBody>,
    pub ragdoll: Option<Ragdoll>,
    pub network: Option<NetworkObject>,
    pub node_graph: Option<NodeGraph>,
    pub behavior_tree: Option<BehaviorTree>,
    pub ik_rig: Option<ik::IkRig>,
    pub vfx_graph: Option<vfx_graph::VfxGraph>,
    pub spline_deformer: Option<spline::SplineMeshDeformer>,
    pub foliage: Option<foliage::FoliagePainter>,
    pub water: Option<water::WaterBody>,
    pub decals: Option<decal::DecalLayer>,
    pub destruction: Option<destruction::DestructionBody>,
    pub ai_perception: Option<ai_perception::AiPerceptionComponent>,
}

impl SceneObject {
    pub fn primitive(id: ObjectId, kind: PrimitiveKind) -> Self {
        Self {
            id,
            name: kind.label().to_string(),
            kind: ObjectKind::Primitive(kind),
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            intensity: 1.0,
            visible: true,
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
        }
    }

    pub fn mesh(id: ObjectId, name: String, asset_index: usize) -> Self {
        Self {
            id,
            name,
            kind: ObjectKind::Mesh { asset_index },
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            intensity: 1.0,
            visible: true,
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
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Scene {
    pub objects: Vec<SceneObject>,
    next_id: ObjectId,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_primitive(&mut self, kind: PrimitiveKind) -> ObjectId {
        let mut object = SceneObject::primitive(self.next_id.allocate(), kind);
        object.name = self.unique_name(kind.label());
        let [x, y, z] = self.default_position(kind);
        object.position = [x, y, z];
        let id = object.id;
        self.objects.push(object);
        id
    }

    pub fn add_mesh(&mut self, name: String, asset_index: usize) -> ObjectId {
        let id = self.next_id.allocate();
        let unique = self.unique_name(&name);
        let mut object = SceneObject::mesh(id, unique, asset_index);
        object.position = self.default_position(PrimitiveKind::Cube);
        self.objects.push(object);
        id
    }

    pub fn remove(&mut self, id: ObjectId) -> bool {
        let before = self.objects.len();
        self.objects.retain(|o| o.id != id);
        before != self.objects.len()
    }

    pub fn get(&self, id: ObjectId) -> Option<&SceneObject> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut SceneObject> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn restore(&mut self, object: SceneObject) {
        self.objects.retain(|o| o.id != object.id);
        if object.id.0 >= self.next_id.0 {
            self.next_id.0 = object.id.0.wrapping_add(1);
        }
        self.objects.push(object);
    }

    fn default_position(&self, kind: PrimitiveKind) -> [f64; 3] {
        match kind {
            PrimitiveKind::Camera => [8.0, 6.0, 8.0],
            PrimitiveKind::DirectionalLight => [4.0, 10.0, 4.0],
            PrimitiveKind::PointLight => [2.0, 4.0, 2.0],
            PrimitiveKind::SpotLight => [3.0, 5.0, 3.0],
            _ => {
                let index = self
                    .objects
                    .iter()
                    .filter(|object| {
                        matches!(
                            object.kind,
                            ObjectKind::Primitive(
                                PrimitiveKind::Empty
                                    | PrimitiveKind::Cube
                                    | PrimitiveKind::Sphere
                                    | PrimitiveKind::Plane
                                    | PrimitiveKind::Cylinder
                                    | PrimitiveKind::Cone
                            ) | ObjectKind::Mesh { .. }
                        )
                    })
                    .count();
                let columns = 4usize;
                let col = index % columns;
                let row = index / columns;
                [
                    (col as f64 - (columns as f64 - 1.0) * 0.5) * 2.75,
                    0.0,
                    row as f64 * -2.75,
                ]
            }
        }
    }

    fn unique_name(&self, base: &str) -> String {
        if !self.objects.iter().any(|o| o.name == base) {
            return base.to_string();
        }
        let mut suffix = 1usize;
        loop {
            let candidate = format!("{base}.{suffix:03}");
            if !self.objects.iter().any(|o| o.name == candidate) {
                return candidate;
            }
            suffix += 1;
        }
    }
}
