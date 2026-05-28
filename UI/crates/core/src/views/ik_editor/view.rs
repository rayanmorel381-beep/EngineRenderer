use crate::scene::ik::{IkChain, IkKind, IkRig};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct IkEditorView {
    pub selected_chain: Option<usize>,
    pub kind_idx: usize,
}

impl Default for IkEditorView {
    fn default() -> Self { Self { selected_chain: None, kind_idx: 0 } }
}

impl IkEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, rig: &mut IkRig) {
        let panel = Panel::new("IK Rig").with_icon(Icon::Skeleton);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("ik_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Chaînes IK ({})", rig.chains.len()), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_chain: Option<usize> = None;
        for (i, chain) in rig.chains.iter_mut().enumerate() {
            let selected = self.selected_chain == Some(i);
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, if selected { p.panel_active } else { p.panel }, 2.0);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &format!("{} [{}]", chain.name, chain.kind.label()), p.text, m.font_size_small);

            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("csel{i}")), sel_r).clicked {
                self.selected_chain = if selected { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("cdel{i}")), del_r).clicked {
                remove_chain = Some(i);
            }
            y += row_h + 2.0;

            if selected {
                Slider::new("Cible X", -20.0, 20.0).show(ui, id.child(&format!("ctx{i}")), Rect::new(lx, y, w, row_h), &mut chain.target[0]);
                y += row_h + sp;
                Slider::new("Cible Y", -20.0, 20.0).show(ui, id.child(&format!("cty{i}")), Rect::new(lx, y, w, row_h), &mut chain.target[1]);
                y += row_h + sp;
                Slider::new("Cible Z", -20.0, 20.0).show(ui, id.child(&format!("ctz{i}")), Rect::new(lx, y, w, row_h), &mut chain.target[2]);
                y += row_h + sp;
                Slider::new("Poids", 0.0, 1.0).show(ui, id.child(&format!("cw{i}")), Rect::new(lx, y, w, row_h), &mut chain.weight);
                y += row_h + sp;
                let mut iters_f = chain.max_iterations as f64;
                Slider::new("Itérations max", 1.0, 64.0).show(ui, id.child(&format!("cit{i}")), Rect::new(lx, y, w, row_h), &mut iters_f);
                chain.max_iterations = iters_f as usize;
                y += row_h + sp;
                let en_lbl = if chain.enabled { "Actif" } else { "Inactif" };
                if Button::new(en_lbl).with_style(if chain.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child(&format!("cen{i}")), Rect::new(lx, y, 100.0, row_h)).clicked {
                    chain.enabled = !chain.enabled;
                }
                y += row_h + sp;
            }
        }
        if let Some(idx) = remove_chain {
            rig.remove_chain(idx);
            if self.selected_chain == Some(idx) { self.selected_chain = None; }
        }

        let kind_labels: Vec<&str> = IkKind::ALL.iter().map(|k| k.label()).collect();
        let kind_w = 120.0;
        let kind_r = Rect::new(lx, y, kind_w, row_h);
        if Button::new(kind_labels[self.kind_idx % kind_labels.len()]).with_style(ButtonStyle::Secondary)
            .show(ui, id.child("kind_cyc"), kind_r).clicked {
            self.kind_idx = (self.kind_idx + 1) % IkKind::ALL.len();
        }
        let add_r = Rect::new(lx + kind_w + sp, y, w - kind_w - sp, row_h);
        if Button::new("+ Chaîne IK").with_style(ButtonStyle::Secondary).show(ui, id.child("add_chain"), add_r).clicked {
            let kind = IkKind::ALL[self.kind_idx % IkKind::ALL.len()].clone();
            let mut chain = IkChain::new(format!("IK {}", rig.chains.len() + 1));
            chain.kind = kind;
            rig.add_chain(chain);
        }
    }
}
