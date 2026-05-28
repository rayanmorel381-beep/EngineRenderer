use crate::scene::material::PbrMaterial;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct MaterialEditorView {
    pub selected: usize,
    alpha_mode_sel: usize,
}

impl Default for MaterialEditorView {
    fn default() -> Self {
        Self { selected: 0, alpha_mode_sel: 0 }
    }
}

impl MaterialEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, mat: &mut PbrMaterial) {
        let panel = Panel::new("Material Editor").with_icon(Icon::Material);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;

        let id = WidgetId::hash_str("mat_ed");
        let mut y = body.y + pad;
        let w = body.width - pad * 2.0;
        let lx = body.x + pad;

        ui.draw_list.text(Vec2::new(lx, y), "Albedo", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 2.0;

        for (ci, label) in ["R", "G", "B", "A"].iter().enumerate() {
            let r = Rect::new(lx, y, w, row_h);
            Slider::new(label, 0.0, 1.0).format(":.3").show(ui, id.child(label), r, &mut mat.albedo[ci]);
            y += row_h + sp;
        }

        y += sp;
        ui.draw_list.text(Vec2::new(lx, y), "PBR", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 2.0;

        Slider::new("Metallic", 0.0, 1.0).format(":.3").show(ui, id.child("met"), Rect::new(lx, y, w, row_h), &mut mat.metallic);
        y += row_h + sp;
        Slider::new("Roughness", 0.0, 1.0).format(":.3").show(ui, id.child("rough"), Rect::new(lx, y, w, row_h), &mut mat.roughness);
        y += row_h + sp;

        y += sp;
        ui.draw_list.text(Vec2::new(lx, y), "Emissive", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 2.0;
        for (ci, label) in ["Em.R", "Em.G", "Em.B"].iter().enumerate() {
            let r = Rect::new(lx, y, w, row_h);
            Slider::new(label, 0.0, 1.0).format(":.3").show(ui, id.child(label), r, &mut mat.emissive[ci]);
            y += row_h + sp;
        }
        Slider::new("Strength", 0.0, 20.0).show(ui, id.child("emstr"), Rect::new(lx, y, w, row_h), &mut mat.emissive_strength);
        y += row_h + sp;

        y += sp;
        ui.draw_list.text(Vec2::new(lx, y), "Surface", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 2.0;
        Slider::new("Normal Scale", 0.0, 2.0).show(ui, id.child("nscl"), Rect::new(lx, y, w, row_h), &mut mat.normal_scale);
        y += row_h + sp;
        Slider::new("Occlusion", 0.0, 1.0).show(ui, id.child("occ"), Rect::new(lx, y, w, row_h), &mut mat.occlusion_strength);
        y += row_h + sp;
        Slider::new("Alpha Cutoff", 0.0, 1.0).show(ui, id.child("acut"), Rect::new(lx, y, w, row_h), &mut mat.alpha_cutoff);
        y += row_h + sp;

        let mode_opts: Vec<&str> = crate::scene::material::AlphaMode::ALL.iter().map(|a| a.label()).collect();
        self.alpha_mode_sel = crate::scene::material::AlphaMode::ALL.iter().position(|a| *a == mat.alpha_mode).unwrap_or(0);
        if Dropdown::new("Alpha Mode", &mode_opts).show(ui, id.child("alpha"), Rect::new(lx, y, w, row_h), &mut self.alpha_mode_sel) {
            mat.alpha_mode = crate::scene::material::AlphaMode::ALL[self.alpha_mode_sel].clone();
        }
        y += row_h + sp;

        let dbl_label = if mat.double_sided { "Double-Sided: ON" } else { "Double-Sided: OFF" };
        if Button::new(dbl_label).with_style(if mat.double_sided { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("dbl"), Rect::new(lx, y, w * 0.48, row_h)).clicked {
            mat.double_sided = !mat.double_sided;
        }
        let wf_label = if mat.wireframe { "Wireframe: ON" } else { "Wireframe: OFF" };
        if Button::new(wf_label).with_style(if mat.wireframe { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("wf"), Rect::new(lx + w * 0.52, y, w * 0.48, row_h)).clicked {
            mat.wireframe = !mat.wireframe;
        }
        y += row_h + sp;

        let sh_label = if mat.cast_shadows { "Cast Shadow: ON" } else { "Cast Shadow: OFF" };
        if Button::new(sh_label).with_style(if mat.cast_shadows { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("csh"), Rect::new(lx, y, w * 0.48, row_h)).clicked {
            mat.cast_shadows = !mat.cast_shadows;
        }
        let rs_label = if mat.receive_shadows { "Recv Shadow: ON" } else { "Recv Shadow: OFF" };
        if Button::new(rs_label).with_style(if mat.receive_shadows { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("rsh"), Rect::new(lx + w * 0.52, y, w * 0.48, row_h)).clicked {
            mat.receive_shadows = !mat.receive_shadows;
        }
    }
}
