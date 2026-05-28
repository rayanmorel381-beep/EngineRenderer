use std::path::Path;

use crate::assets::{AssetRegistry, ImportOutcome, ImportStatus};
use crate::editor::history::{HistoryEntry, HistoryStack, Transform};
use crate::editor::layout::EditorLayout;
use crate::editor::mesh_ops;
use crate::scene::mesh::EditMesh;
use crate::scene::{ObjectId, ObjectKind, PrimitiveKind, Scene};
use crate::state::AppState;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::Rect;
use crate::ui::panels::{MenuBar, MenuItem, StatusBar, ToolBar, ToolBarItem};
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, PropertyRow, Tab, TabBar, TreeNode};
use crate::views::console::ConsoleEntry;
use crate::views::content_browser::ContentBrowserItem;
use crate::views::timeline::TimelineTrack;
use crate::scene::{PhysicsBody, ScriptComponent};
use crate::scene::physics::step_physics;
use crate::views::{
    ConsoleView, ContentBrowserView, HierarchyView, InspectorView, MeshEditorView, ScriptEditorView,
    StatsView, TimelineView, ViewportView,
};

const ADD_PALETTE: [PrimitiveKind; 13] = [
    PrimitiveKind::Empty,
    PrimitiveKind::Cube,
    PrimitiveKind::Sphere,
    PrimitiveKind::Plane,
    PrimitiveKind::Cylinder,
    PrimitiveKind::Cone,
    PrimitiveKind::Torus,
    PrimitiveKind::Icosphere,
    PrimitiveKind::Capsule,
    PrimitiveKind::Hypercube4D,
    PrimitiveKind::Simplex4D,
    PrimitiveKind::Camera,
    PrimitiveKind::DirectionalLight,
];

pub struct Editor {
    pub scene: Scene,
    pub assets: AssetRegistry,
    pub selected: Option<ObjectId>,
    history: HistoryStack,
    pub in_edit_mode: bool,
    pub edit_mesh: Option<EditMesh>,
    pub mesh_editor: MeshEditorView,
    pub viewport: ViewportView,
    pub hierarchy: HierarchyView,
    pub inspector: InspectorView,
    pub content_browser: ContentBrowserView,
    pub console: ConsoleView,
    pub timeline: TimelineView,
    pub script_editor: ScriptEditorView,
    pub stats: StatsView,
    pub bottom_active_tab: usize,
    pub right_active_tab: usize,
    id_map: Vec<(WidgetId, ObjectId)>,
}

impl Default for Editor {
    fn default() -> Self {
        let mut editor = Self {
            scene: Scene::new(),
            assets: AssetRegistry::new(),
            selected: None,
            history: HistoryStack::default(),
            in_edit_mode: false,
            edit_mesh: None,
            mesh_editor: MeshEditorView::new(),
            viewport: ViewportView::new(),
            hierarchy: HierarchyView::new(),
            inspector: InspectorView::new(),
            content_browser: ContentBrowserView::new(),
            console: ConsoleView::new(),
            timeline: TimelineView::new(),
            script_editor: ScriptEditorView::new(),
            stats: StatsView::new(),
            bottom_active_tab: 0,
            right_active_tab: 0,
            id_map: Vec::new(),
        };
        editor.bootstrap();
        editor
    }
}

impl Editor {
    pub fn new() -> Self {
        Self::default()
    }

    fn bootstrap(&mut self) {
        let camera = self.scene.add_primitive(PrimitiveKind::Camera);
        self.scene.add_primitive(PrimitiveKind::DirectionalLight);
        self.scene.add_primitive(PrimitiveKind::Cube);
        self.selected = Some(camera);

        self.console.push(ConsoleEntry::info("RUXEL editor started"));
        self.console.push(ConsoleEntry::info(
            "Drop .obj/.glb/.gltf/.fbx/.blend files onto the window to import",
        ));

        self.sync_views();
    }

    pub fn add_primitive(&mut self, kind: PrimitiveKind) {
        let id = self.scene.add_primitive(kind);
        self.selected = Some(id);
        if let Some(obj) = self.scene.get(id) {
            self.history.push(HistoryEntry::AddObject(obj.clone()));
        }
        self.console
            .push(ConsoleEntry::info(format!("added {} to scene", kind.label())));
        self.sync_views();
    }

    pub fn import_asset(&mut self, path: &Path) {
        let ImportOutcome { asset_index, asset } = self.assets.import_path(path);
        match asset.status {
            ImportStatus::Loaded => self
                .console
                .push(ConsoleEntry::info(format!("import: {}", asset.message))),
            ImportStatus::Stub => self
                .console
                .push(ConsoleEntry::warning(format!("import: {}", asset.message))),
            ImportStatus::Failed => {
                self.console
                    .push(ConsoleEntry::error(format!("import: {}", asset.message)));
                self.sync_views();
                return;
            }
        }
        if matches!(
            asset.format,
            crate::assets::AssetFormat::Obj
                | crate::assets::AssetFormat::Glb
                | crate::assets::AssetFormat::Gltf
                | crate::assets::AssetFormat::Fbx
                | crate::assets::AssetFormat::Blend
        ) {
            let id = self.scene.add_mesh(asset.display_name.clone(), asset_index);
            self.selected = Some(id);
        }
        self.sync_views();
    }

    pub fn handle_dropped_files(&mut self, paths: &[String]) {
        for path in paths {
            self.import_asset(Path::new(path));
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(id) = self.selected.take() {
            if let Some(obj) = self.scene.get(id) {
                self.history.push(HistoryEntry::RemoveObject(obj.clone()));
            }
            if self.scene.remove(id) {
                self.console.push(ConsoleEntry::info("deleted object"));
            }
        }
        self.sync_views();
    }

    pub fn do_undo(&mut self) {
        if let Some(new_sel) = self.history.undo(&mut self.scene) {
            self.selected = new_sel;
            self.sync_views();
        } else {
            self.console.push(ConsoleEntry::info("nothing to undo"));
        }
    }

    pub fn do_redo(&mut self) {
        if let Some(new_sel) = self.history.redo(&mut self.scene) {
            self.selected = new_sel;
            self.sync_views();
        } else {
            self.console.push(ConsoleEntry::info("nothing to redo"));
        }
    }

    pub fn duplicate_selected(&mut self) {
        let Some(id) = self.selected else { return; };
        let Some(obj) = self.scene.get(id) else { return; };
        let kind_clone = obj.kind.clone();
        let pos = [obj.position[0] + 0.5, obj.position[1], obj.position[2] + 0.5];
        let rot = obj.rotation;
        let scale = obj.scale;
        let intensity = obj.intensity;
        let mesh_name = match &obj.kind {
            ObjectKind::Mesh { .. } => obj.name.clone(),
            _ => String::new(),
        };
        let new_id = match kind_clone {
            ObjectKind::Primitive(kind) => self.scene.add_primitive(kind),
            ObjectKind::Mesh { asset_index } => self.scene.add_mesh(mesh_name, asset_index),
        };
        if let Some(new_obj) = self.scene.get_mut(new_id) {
            new_obj.position = pos;
            new_obj.rotation = rot;
            new_obj.scale = scale;
            new_obj.intensity = intensity;
        }
        if let Some(new_obj) = self.scene.get(new_id) {
            self.history.push(HistoryEntry::AddObject(new_obj.clone()));
        }
        self.selected = Some(new_id);
        self.console.push(ConsoleEntry::info("duplicated object"));
        self.sync_views();
    }

    pub fn frame_selected(&mut self) {
        let Some(id) = self.selected else { return; };
        let Some(obj) = self.scene.get(id) else { return; };
        let center = obj.position;
        let radius = obj.scale.iter().copied().fold(0.0f64, f64::max);
        self.viewport.frame_camera_on(center, radius.max(1.0));
    }

    pub fn load_scene(&mut self, scene: Scene) {
        self.scene = scene;
        self.selected = None;
        self.history = HistoryStack::default();
        self.in_edit_mode = false;
        self.edit_mesh = None;
        self.console.push(ConsoleEntry::info("scene loaded"));
        self.sync_views();
    }

    pub fn toggle_edit_mode(&mut self) {
        if self.in_edit_mode {
            self.in_edit_mode = false;
            self.edit_mesh = None;
        } else if let Some(id) = self.selected {
            if let Some(obj) = self.scene.objects.iter().find(|o| o.id == id) {
                let mesh = match &obj.kind {
                    ObjectKind::Primitive(PrimitiveKind::Plane) => EditMesh::from_plane(id),
                    ObjectKind::Primitive(_) => EditMesh::from_cube(id),
                    _ => return,
                };
                self.edit_mesh = Some(mesh);
                self.in_edit_mode = true;
            }
        }
    }

    pub fn show(&mut self, ui: &mut UiContext, state: &AppState) {
        let viewport_rect = ui.screen_rect();
        let layout = EditorLayout::compute(
            viewport_rect,
            ui.theme.metrics.menu_bar_height,
            ui.theme.metrics.tool_bar_height,
            ui.theme.metrics.status_bar_height,
        );

        ui.draw_list
            .rect(viewport_rect, ui.theme.palette.background, 0.0);

        let menu_items = [
            MenuItem::new("File", &["New Scene", "Open", "Save", "Quit"]),
            MenuItem::new("Edit", &["Undo", "Redo", "Delete Selected"]),
            MenuItem::new(
                "Add",
                &[
                    "Empty",
                    "Cube",
                    "Sphere",
                    "Plane",
                    "Cylinder",
                    "Cone",
                    "Camera",
                    "Directional Light",
                    "Point Light",
                    "Spot Light",
                ],
            ),
            MenuItem::new("View", &["Hierarchy", "Inspector", "Console"]),
            MenuItem::new("Help", &["About"]),
        ];
        MenuBar::new(&menu_items).show(ui, layout.menu_bar);

        let tools = [
            ToolBarItem::new(Icon::New, "New Scene"),
            ToolBarItem::new(Icon::Open, "Open"),
            ToolBarItem::new(Icon::Save, "Save"),
            ToolBarItem::new(Icon::Add, "Add Cube"),
            ToolBarItem::new(Icon::Light, "Add Light"),
            ToolBarItem::new(Icon::Camera, "Add Camera"),
            ToolBarItem::new(Icon::Delete, "Delete Selected"),
            ToolBarItem::new(Icon::Play, "Play"),
            ToolBarItem::new(Icon::Pause, "Pause"),
            ToolBarItem::new(Icon::Stop, "Stop"),
        ];
        if let Some(idx) = ToolBar::new(&tools).show(ui, layout.tool_bar) {
            self.handle_toolbar(idx);
        }

        if layout.left_panel.width > 0.0 {
            let add_strip_h = ui.theme.metrics.tool_bar_height;
            let (add_strip, hierarchy_rect) = layout.left_panel.split_top(add_strip_h);
            self.show_add_strip(ui, add_strip);
            self.hierarchy.show(ui, hierarchy_rect);
        }
        self.sync_inspector_with_selection();

        let tabbar_h = ui.theme.metrics.tab_height;
        if layout.right_panel.width > 0.0 {
            let right_tabs = [Tab::new("Inspector"), Tab::new("Stats")];
            let (tabs_rect, body_rect) = layout.right_panel.split_top(tabbar_h);
            TabBar::new(&right_tabs).show(
                ui,
                WidgetId::hash_str("right_tabs"),
                tabs_rect,
                &mut self.right_active_tab,
            );
            match self.right_active_tab {
                0 => self.inspector.show(ui, body_rect),
                1 => self.stats.show(ui, body_rect),
                _ => {}
            }
        }

        if self.in_edit_mode {
            if let Some(ref mut mesh) = self.edit_mesh {
                self.mesh_editor.show(ui, layout.center_panel, mesh);
            }
            let pending = self.mesh_editor.pending.clone();
            if let Some(ref mut mesh) = self.edit_mesh {
                if pending.extrude { mesh_ops::extrude_selected(mesh, pending.extrude_amount); }
                if pending.loop_cut {
                    let edges = mesh_ops::select_ring(mesh, crate::scene::mesh::EdgeId(0));
                    mesh_ops::loop_cut(mesh, &edges, pending.loop_cut_t);
                }
                if pending.bevel { mesh_ops::bevel_selected_edges(mesh, pending.bevel_amount); }
                if let Some(fid) = pending.inset { mesh_ops::inset_face(mesh, fid, pending.inset_amount); }
                if pending.subdivide { mesh_ops::subdivide(mesh); }
                if pending.merge { mesh_ops::merge_vertices_by_distance(mesh, pending.merge_dist); }
                if pending.flip_normals { mesh_ops::flip_face_normals(mesh); }
            }
            self.mesh_editor.pending = crate::views::mesh_editor::view::PendingOp::default();
        } else {
            self.viewport
                .show(ui, layout.center_panel, &self.scene, self.selected);
        }

        if let Some(sel) = self.viewport.pending_select.take() {
            self.selected = sel;
            self.sync_views();
        }
        if let Some((id, pos)) = self.viewport.pending_move.take() {
            if let Some(obj) = self.scene.get_mut(id) {
                obj.position = pos;
            }
            self.write_inspector_from_scene();
        }
        if let Some((id, before_pos, after_pos)) = self.viewport.transform_commit.take() {
            if let Some(obj) = self.scene.get(id) {
                let rot = obj.rotation;
                let sc = obj.scale;
                self.history.push(HistoryEntry::SetTransform {
                    id,
                    before: Transform { position: before_pos, rotation: rot, scale: sc },
                    after: Transform { position: after_pos, rotation: rot, scale: sc },
                });
            }
        }

        let bottom_tabs = [
            Tab::new("Console"),
            Tab::new("Content Browser"),
            Tab::new("Timeline"),
            Tab::new("Scripts"),
        ];
        let (bottom_tab_rect, bottom_body_rect) = layout.bottom_panel.split_top(tabbar_h);
        TabBar::new(&bottom_tabs).show(
            ui,
            WidgetId::hash_str("bottom_tabs"),
            bottom_tab_rect,
            &mut self.bottom_active_tab,
        );
        match self.bottom_active_tab {
            0 => self.console.show(ui, bottom_body_rect),
            1 => self.content_browser.show(ui, bottom_body_rect),
            2 => self.timeline.show(ui, bottom_body_rect),
            3 => {
                let scripts = self
                    .selected
                    .and_then(|id| self.scene.get(id))
                    .map(|obj| obj.scripts.clone())
                    .unwrap_or_default();
                self.script_editor.show(ui, bottom_body_rect, &scripts);
                self.flush_script_editor_pending();
            }
            _ => {}
        }

        self.tick_simulation(state.last_frame_micros);

        self.write_scene_from_inspector();

        let fps = self.stats.fps();
        let asset_count = self.assets.len();
        let object_count = self.scene.len();
        let left_text = format!(
            "Frame {} | {:.1} FPS | {} obj | {} assets",
            state.frame_index, fps, object_count, asset_count
        );
        let center_text = format!("{}x{}", state.width, state.height);
        let right_text = if state.should_quit {
            "shutting down"
        } else {
            "drop files to import"
        };
        StatusBar::new(&left_text, &center_text, right_text).show(ui, layout.status_bar);
    }

    pub fn record_frame(&mut self, micros: u64) {
        self.stats.push_frame(micros);
    }

    fn show_add_strip(&mut self, ui: &mut UiContext, rect: Rect) {
        ui.draw_list.rect(
            rect,
            ui.theme.palette.panel,
            ui.theme.metrics.corner_radius_small,
        );
        let pad = ui.theme.metrics.padding * 0.5;
        let inner = Rect::new(
            rect.x + pad,
            rect.y + pad,
            (rect.width - pad * 2.0).max(0.0),
            (rect.height - pad * 2.0).max(0.0),
        );
        let count = ADD_PALETTE.len();
        if count == 0 || inner.width <= 0.0 {
            return;
        }
        let spacing = ui.theme.metrics.spacing * 0.5;
        let total_spacing = spacing * (count as f64 - 1.0);
        let cell_w = ((inner.width - total_spacing) / count as f64).max(20.0);
        let mut clicked: Option<PrimitiveKind> = None;
        for (i, kind) in ADD_PALETTE.iter().copied().enumerate() {
            let x = inner.x + i as f64 * (cell_w + spacing);
            let cell = Rect::new(x, inner.y, cell_w, inner.height);
            let id = WidgetId::hash_str("add_strip").child(kind.label());
            let interaction = Button::icon(kind.icon()).show(ui, id, cell);
            if interaction.clicked {
                clicked = Some(kind);
            }
        }
        if let Some(kind) = clicked {
            self.add_primitive(kind);
        }
    }

    fn handle_toolbar(&mut self, idx: usize) {
        match idx {
            0 => self.new_scene(),
            1 => self
                .console
                .push(ConsoleEntry::info("Open: drop a file or pass via CLI")),
            2 => self
                .console
                .push(ConsoleEntry::info("Save: layout persisted on quit")),
            3 => self.add_primitive(PrimitiveKind::Cube),
            4 => self.add_primitive(PrimitiveKind::DirectionalLight),
            5 => self.add_primitive(PrimitiveKind::Camera),
            6 => self.delete_selected(),
            7 => self.console.push(ConsoleEntry::info("Play")),
            8 => self.console.push(ConsoleEntry::info("Pause")),
            9 => self.console.push(ConsoleEntry::info("Stop")),
            _ => {}
        }
    }

    fn new_scene(&mut self) {
        self.scene = Scene::new();
        self.selected = None;
        self.console.push(ConsoleEntry::info("new scene"));
        self.sync_views();
    }

    fn sync_views(&mut self) {
        let mut nodes = Vec::with_capacity(self.scene.len() + 1);
        let root_id = WidgetId::hash_str("scene/root");
        nodes.push(TreeNode {
            id: root_id,
            label: "Scene".to_string(),
            icon: Icon::Scene,
            depth: 0,
            expanded: true,
            selected: false,
            has_children: !self.scene.is_empty(),
        });
        self.id_map.clear();
        for object in &self.scene.objects {
            let widget = WidgetId::hash_str(&format!("scene/obj/{}", object.id.0));
            self.id_map.push((widget, object.id));
            nodes.push(TreeNode {
                id: widget,
                label: object.name.clone(),
                icon: object.kind.icon(),
                depth: 1,
                expanded: true,
                selected: self.selected == Some(object.id),
                has_children: false,
            });
        }
        self.hierarchy.replace_nodes(nodes);
        if let Some(selected) = self.selected {
            let widget = WidgetId::hash_str(&format!("scene/obj/{}", selected.0));
            self.hierarchy.selected = Some(widget);
        } else {
            self.hierarchy.selected = None;
        }

        self.content_browser.items = self
            .assets
            .assets()
            .iter()
            .map(|asset| ContentBrowserItem::new(asset.display_name.clone(), asset.icon()))
            .collect();

        self.timeline.tracks = self
            .scene
            .objects
            .iter()
            .filter_map(|object| match &object.kind {
                ObjectKind::Primitive(
                    PrimitiveKind::Camera
                        | PrimitiveKind::DirectionalLight
                        | PrimitiveKind::PointLight
                        | PrimitiveKind::SpotLight,
                ) => Some(TimelineTrack {
                    name: object.name.clone(),
                    keyframes: vec![0.0],
                }),
                _ => None,
            })
            .collect();

        self.write_inspector_from_scene();
    }

    fn lookup_object(&self, widget: WidgetId) -> Option<ObjectId> {
        self.id_map
            .iter()
            .copied()
            .find(|(w, _)| *w == widget)
            .map(|(_, id)| id)
    }

    fn write_inspector_from_scene(&mut self) {
        let label = self
            .selected
            .and_then(|id| self.scene.get(id))
            .map(|object| match &object.kind {
                ObjectKind::Primitive(p) => format!("{} ({})", object.name, p.label()),
                ObjectKind::Mesh { asset_index } => {
                    let suffix = self
                        .assets
                        .get(*asset_index)
                        .map(|a| format!("Mesh {}", a.format.label()))
                        .unwrap_or_else(|| "Mesh".to_string());
                    format!("{} ({})", object.name, suffix)
                }
            });
        let inspector = &mut self.inspector;
        match self.selected.and_then(|id| self.scene.get(id)) {
            Some(object) => {
                inspector.object_name = label.unwrap_or_else(|| object.name.clone());
                inspector.object_type = match &object.kind {
                    ObjectKind::Primitive(kind) => kind.label().to_string(),
                    ObjectKind::Mesh { asset_index } => self
                        .assets
                        .get(*asset_index)
                        .map(|asset| format!("Mesh ({})", asset.format.label()))
                        .unwrap_or_else(|| "Mesh".to_string()),
                };
                inspector.position = object.position;
                inspector.rotation = object.rotation;
                inspector.scale = object.scale;
                inspector.material_intensity = object.intensity;
                inspector.extra_rows = match &object.kind {
                    ObjectKind::Primitive(kind) => vec![
                        PropertyRow::new("Category", "Primitive"),
                        PropertyRow::new("Kind", kind.label()),
                        PropertyRow::new("Visible", if object.visible { "Yes" } else { "No" }),
                    ],
                    ObjectKind::Mesh { asset_index } => {
                        let mut rows = vec![
                            PropertyRow::new("Category", "Imported Mesh"),
                            PropertyRow::new("Visible", if object.visible { "Yes" } else { "No" }),
                        ];
                        if let Some(asset) = self.assets.get(*asset_index) {
                            rows.push(PropertyRow::new("Format", asset.format.label()));
                            rows.push(PropertyRow::new("Vertices", asset.vertex_count.to_string()));
                            rows.push(PropertyRow::new("Indices", asset.index_count.to_string()));
                            rows.push(PropertyRow::new("Source", asset.display_name.clone()));
                        }
                        rows
                    }
                };
            }
            None => {
                inspector.object_name = "<no selection>".to_string();
                inspector.object_type = "None".to_string();
                inspector.position = [0.0, 0.0, 0.0];
                inspector.rotation = [0.0, 0.0, 0.0];
                inspector.scale = [1.0, 1.0, 1.0];
                inspector.material_intensity = 1.0;
                inspector.extra_rows.clear();
            }
        }
    }

    fn write_scene_from_inspector(&mut self) {
        let Some(id) = self.selected else {
            return;
        };
        let Some(object) = self.scene.get_mut(id) else {
            return;
        };
        object.position = self.inspector.position;
        object.rotation = self.inspector.rotation;
        object.scale = self.inspector.scale;
        object.intensity = self.inspector.material_intensity;
    }

    fn sync_inspector_with_selection(&mut self) {
        let Some(selected_widget) = self.hierarchy.selected else {
            return;
        };
        let Some(object_id) = self.lookup_object(selected_widget) else {
            return;
        };
        if self.selected != Some(object_id) {
            self.selected = Some(object_id);
            self.write_inspector_from_scene();
        }
    }

    fn tick_simulation(&mut self, last_frame_micros: u64) {
        let dt = last_frame_micros as f64 / 1_000_000.0;
        if dt <= 0.0 { return; }
        if self.timeline.playing {
            self.timeline.advance(dt);
        }
        for obj in self.scene.objects.iter_mut() {
            if let Some(ref mut anim) = obj.animator {
                if anim.playing {
                    anim.advance(dt);
                    if let Some(v) = anim.sample_property("pos.x") { obj.position[0] = v; }
                    if let Some(v) = anim.sample_property("pos.y") { obj.position[1] = v; }
                    if let Some(v) = anim.sample_property("pos.z") { obj.position[2] = v; }
                    if let Some(v) = anim.sample_property("rot.x") { obj.rotation[0] = v; }
                    if let Some(v) = anim.sample_property("rot.y") { obj.rotation[1] = v; }
                    if let Some(v) = anim.sample_property("rot.z") { obj.rotation[2] = v; }
                    if let Some(v) = anim.sample_property("scl.x") { obj.scale[0] = v; }
                    if let Some(v) = anim.sample_property("scl.y") { obj.scale[1] = v; }
                    if let Some(v) = anim.sample_property("scl.z") { obj.scale[2] = v; }
                }
            }
            if let Some(ref mut body) = obj.physics {
                step_physics(body, &mut obj.position, dt);
            }
        }
    }

    fn flush_script_editor_pending(&mut self) {
        if self.script_editor.pending_add {
            self.script_editor.pending_add = false;
            if let Some(id) = self.selected {
                if let Some(obj) = self.scene.get_mut(id) {
                    let n = obj.scripts.len() + 1;
                    obj.scripts.push(ScriptComponent::new(format!("Script{}", n)));
                }
            }
        }
        if let Some(idx) = self.script_editor.pending_remove.take() {
            if let Some(id) = self.selected {
                if let Some(obj) = self.scene.get_mut(id) {
                    if idx < obj.scripts.len() {
                        obj.scripts.remove(idx);
                        if self.script_editor.selected_script > 0 {
                            self.script_editor.selected_script -= 1;
                        }
                    }
                }
            }
        }
        if let Some(idx) = self.script_editor.pending_toggle.take() {
            if let Some(id) = self.selected {
                if let Some(obj) = self.scene.get_mut(id) {
                    if let Some(script) = obj.scripts.get_mut(idx) {
                        script.enabled = !script.enabled;
                    }
                }
            }
        }
        if let Some(id) = self.selected {
            if let Some(obj) = self.scene.get(id) {
                if self.script_editor.selected_script >= obj.scripts.len() && !obj.scripts.is_empty() {
                    self.script_editor.selected_script = obj.scripts.len() - 1;
                }
            }
        }
    }

    pub fn add_physics_to_selected(&mut self) {
        if let Some(id) = self.selected {
            if let Some(obj) = self.scene.get_mut(id) {
                if obj.physics.is_none() {
                    obj.physics = Some(PhysicsBody::default());
                }
            }
        }
    }

    pub fn remove_physics_from_selected(&mut self) {
        if let Some(id) = self.selected {
            if let Some(obj) = self.scene.get_mut(id) {
                obj.physics = None;
            }
        }
    }
}
