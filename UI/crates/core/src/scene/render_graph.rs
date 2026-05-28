#[derive(Clone, Debug, PartialEq)]
pub enum PassKind {
    GBuffer,
    Lighting,
    Shadow,
    AmbientOcclusion,
    Bloom,
    ToneMapping,
    PostProcess,
    Custom,
    UI,
    Present,
}

impl PassKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::GBuffer => "G-Buffer",
            Self::Lighting => "Éclairage",
            Self::Shadow => "Ombres",
            Self::AmbientOcclusion => "AO",
            Self::Bloom => "Bloom",
            Self::ToneMapping => "Tone Mapping",
            Self::PostProcess => "Post-Process",
            Self::Custom => "Personnalisé",
            Self::UI => "UI",
            Self::Present => "Présentation",
        }
    }
    pub const ALL: [PassKind; 10] = [
        PassKind::GBuffer, PassKind::Lighting, PassKind::Shadow, PassKind::AmbientOcclusion,
        PassKind::Bloom, PassKind::ToneMapping, PassKind::PostProcess, PassKind::Custom,
        PassKind::UI, PassKind::Present,
    ];
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextureFormat {
    Rgba8,
    Rgba16F,
    Rgba32F,
    R32F,
    Depth24Stencil8,
    Depth32F,
}

impl TextureFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Rgba8 => "RGBA8", Self::Rgba16F => "RGBA16F", Self::Rgba32F => "RGBA32F",
            Self::R32F => "R32F", Self::Depth24Stencil8 => "Depth24S8", Self::Depth32F => "Depth32F",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderTexture {
    pub name: String,
    pub format: TextureFormat,
    pub width_scale: f64,
    pub height_scale: f64,
    pub mips: bool,
}

impl RenderTexture {
    pub fn new(name: impl Into<String>, format: TextureFormat) -> Self {
        Self { name: name.into(), format, width_scale: 1.0, height_scale: 1.0, mips: false }
    }
}

#[derive(Clone, Debug)]
pub struct RenderPass {
    pub id: u32,
    pub name: String,
    pub kind: PassKind,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub enabled: bool,
    pub msaa_samples: u8,
    pub clear_color: bool,
    pub clear_depth: bool,
    pub position: [f64; 2],
}

impl RenderPass {
    pub fn new(id: u32, name: impl Into<String>, kind: PassKind) -> Self {
        Self {
            id, name: name.into(), kind, inputs: Vec::new(), outputs: Vec::new(),
            enabled: true, msaa_samples: 1, clear_color: true, clear_depth: true,
            position: [0.0, 0.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderPassEdge {
    pub from: u32,
    pub to: u32,
    pub texture: String,
}

#[derive(Clone, Debug)]
pub struct RenderGraph {
    pub name: String,
    pub passes: Vec<RenderPass>,
    pub textures: Vec<RenderTexture>,
    pub edges: Vec<RenderPassEdge>,
    next_id: u32,
}

impl Default for RenderGraph {
    fn default() -> Self {
        let mut g = Self { name: "Frame Graph".to_string(), passes: Vec::new(), textures: Vec::new(), edges: Vec::new(), next_id: 0 };
        g.textures.push(RenderTexture::new("GBuffer_Albedo", TextureFormat::Rgba8));
        g.textures.push(RenderTexture::new("GBuffer_Normal", TextureFormat::Rgba16F));
        g.textures.push(RenderTexture::new("GBuffer_Depth", TextureFormat::Depth32F));
        g.textures.push(RenderTexture::new("HDR_Color", TextureFormat::Rgba16F));
        g.textures.push(RenderTexture::new("Final_Color", TextureFormat::Rgba8));
        g.textures.push(RenderTexture::new("Shadow_Map", TextureFormat::Depth32F));

        let mut p0 = RenderPass::new(g.alloc_id(), "G-Buffer Pass", PassKind::GBuffer);
        p0.outputs = vec!["GBuffer_Albedo".into(), "GBuffer_Normal".into(), "GBuffer_Depth".into()];
        p0.position = [0.0, 0.0];

        let mut p1 = RenderPass::new(g.alloc_id(), "Shadow Pass", PassKind::Shadow);
        p1.outputs = vec!["Shadow_Map".into()];
        p1.position = [0.0, 130.0];

        let mut p2 = RenderPass::new(g.alloc_id(), "Lighting Pass", PassKind::Lighting);
        p2.inputs = vec!["GBuffer_Albedo".into(), "GBuffer_Normal".into(), "GBuffer_Depth".into(), "Shadow_Map".into()];
        p2.outputs = vec!["HDR_Color".into()];
        p2.position = [240.0, 60.0];

        let mut p3 = RenderPass::new(g.alloc_id(), "AO Pass", PassKind::AmbientOcclusion);
        p3.inputs = vec!["GBuffer_Depth".into(), "GBuffer_Normal".into()];
        p3.outputs = vec!["AO_Result".into()];
        p3.position = [240.0, 190.0];

        let mut p4 = RenderPass::new(g.alloc_id(), "Tone Mapping", PassKind::ToneMapping);
        p4.inputs = vec!["HDR_Color".into()];
        p4.outputs = vec!["Final_Color".into()];
        p4.position = [480.0, 60.0];

        let mut p5 = RenderPass::new(g.alloc_id(), "UI Pass", PassKind::UI);
        p5.inputs = vec!["Final_Color".into()];
        p5.outputs = vec!["Final_Color".into()];
        p5.position = [720.0, 60.0];

        let mut p6 = RenderPass::new(g.alloc_id(), "Present", PassKind::Present);
        p6.inputs = vec!["Final_Color".into()];
        p6.position = [960.0, 60.0];

        g.edges.push(RenderPassEdge { from: p0.id, to: p2.id, texture: "GBuffer_Albedo".into() });
        g.edges.push(RenderPassEdge { from: p1.id, to: p2.id, texture: "Shadow_Map".into() });
        g.edges.push(RenderPassEdge { from: p2.id, to: p4.id, texture: "HDR_Color".into() });
        g.edges.push(RenderPassEdge { from: p4.id, to: p5.id, texture: "Final_Color".into() });
        g.edges.push(RenderPassEdge { from: p5.id, to: p6.id, texture: "Final_Color".into() });

        g.passes.push(p0);
        g.passes.push(p1);
        g.passes.push(p2);
        g.passes.push(p3);
        g.passes.push(p4);
        g.passes.push(p5);
        g.passes.push(p6);
        g
    }
}

impl RenderGraph {
    pub fn new() -> Self { Self::default() }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_pass(&mut self, kind: PassKind) -> u32 {
        let id = self.alloc_id();
        let name = kind.label().to_string();
        self.passes.push(RenderPass::new(id, name, kind));
        id
    }

    pub fn remove_pass(&mut self, id: u32) {
        self.passes.retain(|p| p.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    pub fn add_edge(&mut self, from: u32, to: u32, texture: impl Into<String>) {
        self.edges.push(RenderPassEdge { from, to, texture: texture.into() });
    }

    pub fn topological_order(&self) -> Vec<u32> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let ids: Vec<u32> = self.passes.iter().map(|p| p.id).collect();
        for id in &ids {
            self.visit_topo(*id, &mut visited, &mut result);
        }
        result
    }

    fn visit_topo(&self, id: u32, visited: &mut std::collections::HashSet<u32>, result: &mut Vec<u32>) {
        if visited.contains(&id) { return; }
        visited.insert(id);
        for edge in &self.edges {
            if edge.from == id { self.visit_topo(edge.to, visited, result); }
        }
        result.push(id);
    }
}
