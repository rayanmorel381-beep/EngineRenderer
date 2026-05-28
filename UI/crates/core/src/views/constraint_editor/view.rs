use crate::scene::constraints::{ConstraintWorld, JointKind};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct ConstraintEditorView {
    pub selected_joint: Option<usize>,
    pub selected_spring: Option<usize>,
    pub joint_kind_sel: usize,
}

impl Default for ConstraintEditorView {
    fn default() -> Self { Self { selected_joint: None, selected_spring: None, joint_kind_sel: 0 } }
}

impl ConstraintEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, world: &mut ConstraintWorld) {
        let panel = Panel::new("Contraintes physiques").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("constraint_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Joints ({})", world.joints.len()), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_joint: Option<usize> = None;
        for (i, joint) in world.joints.iter_mut().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            let is_sel = self.selected_joint == Some(i);
            ui.draw_list.rect(row, if is_sel { p.selection } else { p.panel }, 2.0);
            let label = format!("[{}] {} — {}", i, joint.name, joint.kind.label());
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_normal) * 0.5), &label, p.text, m.font_size_normal);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 32.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("✓").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("jsel{i}")), sel_r).clicked {
                self.selected_joint = if is_sel { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("jdel{i}")), del_r).clicked {
                remove_joint = Some(i);
            }
            y += row_h + 2.0;

            if is_sel {
                let detail_rect = Rect::new(lx + 8.0, y, w - 8.0, row_h);
                Slider::new("Limite basse", -180.0, 0.0).show(ui, id.child(&format!("jlo{i}")), detail_rect, &mut joint.lower_limit);
                y += row_h + 2.0;
                let detail_rect2 = Rect::new(lx + 8.0, y, w - 8.0, row_h);
                Slider::new("Limite haute", 0.0, 180.0).show(ui, id.child(&format!("jhi{i}")), detail_rect2, &mut joint.upper_limit);
                y += row_h + 2.0;
            }
        }
        if let Some(idx) = remove_joint { world.remove_joint(idx as u64); self.selected_joint = None; }

        if Button::new("+ Joint").with_style(ButtonStyle::Secondary).show(ui, id.child("add_joint"), Rect::new(lx, y, w, row_h)).clicked {
            let kind = JointKind::ALL[self.joint_kind_sel % JointKind::ALL.len()].clone();
            world.add_joint(kind, Default::default(), Default::default());
        }
        y += row_h + sp * 2.0;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Ressorts ({})", world.springs.len()), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_spring: Option<usize> = None;
        for (i, spring) in world.springs.iter_mut().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            let is_sel = self.selected_spring == Some(i);
            ui.draw_list.rect(row, if is_sel { p.selection } else { p.panel }, 2.0);
            let label = format!("[{}] {} — raideur: {:.1}", i, spring.name, spring.stiffness);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_normal) * 0.5), &label, p.text, m.font_size_normal);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 32.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("✓").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("ssel{i}")), sel_r).clicked {
                self.selected_spring = if is_sel { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("sdel{i}")), del_r).clicked {
                remove_spring = Some(i);
            }
            y += row_h + 2.0;

            if is_sel {
                Slider::new("Raideur", 0.0, 200.0).show(ui, id.child(&format!("sk{i}")), Rect::new(lx+8.0, y, w-8.0, row_h), &mut spring.stiffness);
                y += row_h + 2.0;
                Slider::new("Amortissement", 0.0, 50.0).show(ui, id.child(&format!("sd{i}")), Rect::new(lx+8.0, y, w-8.0, row_h), &mut spring.damping);
                y += row_h + 2.0;
                Slider::new("Longueur repos", 0.01, 20.0).show(ui, id.child(&format!("sr{i}")), Rect::new(lx+8.0, y, w-8.0, row_h), &mut spring.rest_length);
                y += row_h + 2.0;
            }
        }
        if let Some(idx) = remove_spring { world.remove_spring(idx as u64); self.selected_spring = None; }

        if Button::new("+ Ressort").with_style(ButtonStyle::Secondary).show(ui, id.child("add_spring"), Rect::new(lx, y, w, row_h)).clicked {
            world.add_spring(Default::default(), Default::default());
        }
    }
}
