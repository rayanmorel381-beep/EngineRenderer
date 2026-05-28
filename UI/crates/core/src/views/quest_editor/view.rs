use crate::scene::quest::{QuestJournal, QuestObjective, QuestStatus};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct QuestEditorView {
    pub selected_quest: Option<usize>,
    pub selected_node: Option<u32>,
}

impl Default for QuestEditorView {
    fn default() -> Self { Self { selected_quest: None, selected_node: None } }
}

impl QuestEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, journal: &mut QuestJournal) {
        let panel = Panel::new("Journal de quêtes").with_icon(Icon::Quest);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("quest_ed");

        let (list_r, detail_r) = body.split_left(200.0);
        let llx = list_r.x + pad;
        let lw = list_r.width - pad * 2.0;
        let mut ly = list_r.y + pad;

        ui.draw_list.text(Vec2::new(llx, ly), &format!("Quêtes ({})", journal.quests.len()), p.text, m.font_size_normal);
        ly += m.font_size_normal + sp;

        let mut remove_quest: Option<usize> = None;
        for (i, quest) in journal.quests.iter().enumerate() {
            let sel = self.selected_quest == Some(i);
            let qr = Rect::new(llx, ly, lw, row_h);
            let bg = if sel { p.panel_active } else { p.panel };
            ui.draw_list.rect(qr, bg, 2.0);
            let status_col = match &quest.status {
                QuestStatus::Active => p.accent,
                QuestStatus::Completed => p.success,
                QuestStatus::Failed => p.error,
                _ => p.text_muted,
            };
            ui.draw_list.text(Vec2::new(llx + 4.0, ly + (row_h - m.font_size_small) * 0.5), &quest.name, status_col, m.font_size_small);
            let del_r = Rect::new(llx + lw - 20.0, ly + 2.0, 18.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("qdel{i}")), del_r).clicked {
                remove_quest = Some(i);
            }
            if ui.is_rect_hovered(qr) { self.selected_quest = Some(i); }
            ly += row_h + 2.0;
        }
        if let Some(idx) = remove_quest {
            journal.remove_quest(idx);
            if self.selected_quest == Some(idx) { self.selected_quest = None; self.selected_node = None; }
        }
        if Button::new("+ Quête").with_style(ButtonStyle::Secondary).show(ui, id.child("add_q"), Rect::new(llx, ly, lw, row_h)).clicked {
            journal.add_quest(format!("Quête {}", journal.quests.len() + 1));
        }

        if let Some(qidx) = self.selected_quest {
            if qidx < journal.quests.len() {
                let quest = &mut journal.quests[qidx];
                let dx = detail_r.x + pad;
                let dw = detail_r.width - pad * 2.0;
                let mut dy = detail_r.y + pad;

                ui.draw_list.text(Vec2::new(dx, dy), &quest.name, p.text, m.font_size_normal);
                dy += m.font_size_normal + sp;

                let st_col = match &quest.status {
                    QuestStatus::Active => p.accent, QuestStatus::Completed => p.success,
                    QuestStatus::Failed => p.error, _ => p.text_muted,
                };
                ui.draw_list.text(Vec2::new(dx, dy), &format!("Statut: {}", quest.status.label()), st_col, m.font_size_small);
                dy += m.font_size_small + sp;

                if quest.status == QuestStatus::NotStarted {
                    if Button::new("Démarrer").with_style(ButtonStyle::Primary).show(ui, id.child("qstart"), Rect::new(dx, dy, 90.0, row_h)).clicked {
                        quest.start();
                    }
                } else if quest.status == QuestStatus::Active {
                    let btn_w = dw / 2.0 - 3.0;
                    if Button::new("Terminer").with_style(ButtonStyle::Primary).show(ui, id.child("qcomp"), Rect::new(dx, dy, btn_w, row_h)).clicked {
                        quest.complete();
                    }
                    if Button::new("Échouer").with_style(ButtonStyle::Danger).show(ui, id.child("qfail"), Rect::new(dx + btn_w + 6.0, dy, btn_w, row_h)).clicked {
                        quest.fail();
                    }
                }
                dy += row_h + sp;

                ui.draw_list.text(Vec2::new(dx, dy), "Objectifs", p.text, m.font_size_normal);
                dy += m.font_size_normal + sp;

                for obj in quest.objectives.iter_mut() {
                    let done_col = if obj.completed { p.success } else { p.text };
                    let check = if obj.completed { "✓" } else { "○" };
                    let progress = format!("{} {}/{} — {}", check, obj.count_current, obj.count_required, obj.description);
                    ui.draw_list.text(Vec2::new(dx + pad, dy), &progress, done_col, m.font_size_small);
                    dy += m.font_size_small + 4.0;
                    let mut req_f = obj.count_required as f64;
                    Slider::new("Requis", 1.0, 100.0).show(ui, id.child(&format!("obj_req_{}", obj.id)), Rect::new(dx + pad, dy, dw - pad, row_h), &mut req_f);
                    obj.count_required = req_f as u32;
                    dy += row_h + sp;
                }

                if Button::new("+ Objectif").with_style(ButtonStyle::Secondary).show(ui, id.child("add_obj"), Rect::new(dx, dy, 110.0, row_h)).clicked {
                    let oid = quest.objectives.len() as u32;
                    quest.objectives.push(QuestObjective::new(oid, format!("Objectif {}", oid + 1)));
                }
                dy += row_h + sp;

                if let Some(dialog) = &quest.dialog {
                    ui.draw_list.text(Vec2::new(dx, dy), &format!("Dialogue: {} nœuds", dialog.nodes.len()), p.text_muted, m.font_size_small);
                } else {
                    if Button::new("Créer dialogue").with_style(ButtonStyle::Secondary).show(ui, id.child("create_dlg"), Rect::new(dx, dy, 140.0, row_h)).clicked {
                        quest.dialog = Some(crate::scene::quest::DialogGraph::new());
                    }
                }
            }
        }
    }
}
