use crate::core::engine::rendering::raytracing::Vec3;

pub const VSM_PAGE_SIZE: u32 = 128;
pub const VSM_ATLAS_PAGES_X: u32 = 16;
pub const VSM_ATLAS_PAGES_Y: u32 = 16;
pub const VSM_ATLAS_SIZE: u32 = VSM_PAGE_SIZE * VSM_ATLAS_PAGES_X;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageKey {
    pub clip_level: u32,
    pub page_x: u32,
    pub page_y: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum PageState {
    Free,
    Resident { atlas_x: u32, atlas_y: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct VsmPage {
    pub key: PageKey,
    pub state: PageState,
    pub last_used_frame: u64,
    pub shadow_depth: f32,
}

#[derive(Debug, Clone)]
pub struct VirtualShadowMap {
    pub clip_levels: u32,
    pub pages: Vec<VsmPage>,
    pub atlas_occupancy: Vec<bool>,
    pub current_frame: u64,
    pub light_direction: Vec3,
    pub world_texel_size: f64,
}

impl VirtualShadowMap {
    pub fn new(clip_levels: u32, light_direction: Vec3) -> Self {
        let total_atlas_pages = (VSM_ATLAS_PAGES_X * VSM_ATLAS_PAGES_Y) as usize;
        Self {
            clip_levels,
            pages: Vec::new(),
            atlas_occupancy: vec![false; total_atlas_pages],
            current_frame: 0,
            light_direction: light_direction.normalize(),
            world_texel_size: 0.1,
        }
    }

    pub fn mark_page_needed(&mut self, clip_level: u32, page_x: u32, page_y: u32) {
        let key = PageKey { clip_level, page_x, page_y };
        if !self.pages.iter().any(|p| p.key == key) {
            let slot = self.allocate_atlas_slot();
            let state = match slot {
                Some((ax, ay)) => PageState::Resident { atlas_x: ax, atlas_y: ay },
                None => {
                    let evicted = self.evict_lru_page();
                    match evicted {
                        Some((ax, ay)) => PageState::Resident { atlas_x: ax, atlas_y: ay },
                        None => PageState::Free,
                    }
                }
            };
            self.pages.push(VsmPage {
                key,
                state,
                last_used_frame: self.current_frame,
                shadow_depth: 1.0,
            });
        } else {
            for p in &mut self.pages {
                if p.key == key { p.last_used_frame = self.current_frame; }
            }
        }
    }

    pub fn query_shadow(
        &self,
        world_pos: Vec3,
        shading_normal: Vec3,
        clip_level: u32,
    ) -> f32 {
        let projected = self.project_to_light_space(world_pos);
        let page_x = (projected.x * VSM_ATLAS_PAGES_X as f64) as u32 % VSM_ATLAS_PAGES_X;
        let page_y = (projected.y * VSM_ATLAS_PAGES_Y as f64) as u32 % VSM_ATLAS_PAGES_Y;
        let key = PageKey { clip_level, page_x, page_y };

        let n_dot_l = shading_normal.dot(-self.light_direction).max(0.0) as f32;

        for page in &self.pages {
            if page.key == key {
                return match page.state {
                    PageState::Resident { .. } => {
                        let depth_bias = 0.005 + (1.0 - n_dot_l) * 0.01;
                        if (projected.z as f32) < page.shadow_depth - depth_bias {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    PageState::Free => 1.0,
                };
            }
        }

        1.0
    }

    pub fn update_page_depths(&mut self, scene_objects: &[crate::core::engine::rendering::raytracing::primitives::Sphere]) {
        let light_dir = self.light_direction;
        let world_texel_size = self.world_texel_size;
        for page in &mut self.pages {
            if let PageState::Resident { .. } = page.state {
                let page_center = Vec3::new(
                    (page.key.page_x as f64 + 0.5) / VSM_ATLAS_PAGES_X as f64,
                    (page.key.page_y as f64 + 0.5) / VSM_ATLAS_PAGES_Y as f64,
                    0.0,
                );
                let mut min_depth = 1.0_f32;
                for obj in scene_objects {
                    let lp = project_light_space(obj.center, light_dir);
                    let texel_size = world_texel_size * (1 << page.key.clip_level) as f64;
                    let dx = (lp.x - page_center.x).abs();
                    let dy = (lp.y - page_center.y).abs();
                    if dx < texel_size * 2.0 && dy < texel_size * 2.0 {
                        min_depth = min_depth.min(lp.z as f32);
                    }
                }
                page.shadow_depth = min_depth;
            }
        }
    }

    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
        self.pages.retain(|p| self.current_frame - p.last_used_frame < 4);
    }

    pub fn stats(&self) -> VsmStats {
        let resident = self.pages.iter().filter(|p| matches!(p.state, PageState::Resident { .. })).count();
        VsmStats {
            total_pages: self.pages.len(),
            resident_pages: resident,
            atlas_utilization: resident as f64 / (VSM_ATLAS_PAGES_X * VSM_ATLAS_PAGES_Y) as f64,
        }
    }

    fn project_to_light_space(&self, world_pos: Vec3) -> Vec3 {
        project_light_space(world_pos, self.light_direction)
    }

    fn allocate_atlas_slot(&mut self) -> Option<(u32, u32)> {
        for (i, occupied) in self.atlas_occupancy.iter_mut().enumerate() {
            if !*occupied {
                *occupied = true;
                let x = (i as u32) % VSM_ATLAS_PAGES_X;
                let y = (i as u32) / VSM_ATLAS_PAGES_X;
                return Some((x, y));
            }
        }
        None
    }

    fn evict_lru_page(&mut self) -> Option<(u32, u32)> {
        let lru_idx = self.pages
            .iter()
            .enumerate()
            .filter(|(_, p)| matches!(p.state, PageState::Resident { .. }))
            .min_by_key(|(_, p)| p.last_used_frame)
            .map(|(i, _)| i);

        if let Some(idx) = lru_idx {
            let slot = match self.pages[idx].state {
                PageState::Resident { atlas_x, atlas_y } => {
                    let flat = (atlas_y * VSM_ATLAS_PAGES_X + atlas_x) as usize;
                    if flat < self.atlas_occupancy.len() {
                        self.atlas_occupancy[flat] = false;
                    }
                    Some((atlas_x, atlas_y))
                }
                PageState::Free => None,
            };
            self.pages.remove(idx);
            slot
        } else {
            None
        }
    }
}

fn project_light_space(world_pos: Vec3, light_direction: Vec3) -> Vec3 {
    let fwd = -light_direction;
    let up_hint = if fwd.y.abs() < 0.99 { Vec3::new(0.0, 1.0, 0.0) } else { Vec3::new(1.0, 0.0, 0.0) };
    let right = up_hint.cross(fwd).normalize();
    let up = fwd.cross(right).normalize();
    Vec3::new(
        world_pos.dot(right) * 0.5 + 0.5,
        world_pos.dot(up) * 0.5 + 0.5,
        world_pos.dot(fwd) * 0.5 + 0.5,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct VsmStats {
    pub total_pages: usize,
    pub resident_pages: usize,
    pub atlas_utilization: f64,
}
