use crate::scene::job_scheduler::{JobPriority, JobScheduler, JobStatus};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct JobSchedulerView {}

impl Default for JobSchedulerView {
    fn default() -> Self { Self {} }
}

impl JobSchedulerView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, sched: &mut JobScheduler) {
        let panel = Panel::new("Planificateur de tâches").with_icon(Icon::Jobs);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("job_sched");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let en_lbl = if sched.enabled { "Activé" } else { "Suspendu" };
        if Button::new(en_lbl).with_style(if sched.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(lx, y, 100.0, row_h)).clicked {
            sched.enabled = !sched.enabled;
        }
        y += row_h + sp;

        let stats = format!("Workers: {}  |  En attente: {}  |  En cours: {}  |  Terminés: {}", sched.worker_count, sched.pending_count(), sched.running_count(), sched.total_completed);
        ui.draw_list.text(Vec2::new(lx, y), &stats, p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        let mut max_f = sched.max_jobs_per_frame as f64;
        Slider::new("Max tâches/frame", 1.0, 32.0).show(ui, id.child("mjpf"), Rect::new(lx, y, w, row_h), &mut max_f);
        sched.max_jobs_per_frame = max_f as usize;
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), "Tâches actives", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut cancel_job: Option<u64> = None;
        for job in &sched.jobs {
            let job_row = Rect::new(lx, y, w, row_h);
            let bg = match &job.status {
                JobStatus::Running => [0.1, 0.3, 0.1, 1.0],
                JobStatus::Pending => p.panel,
                JobStatus::Failed => [0.3, 0.1, 0.1, 1.0],
                _ => p.panel_hover,
            };
            ui.draw_list.rect(job_row, bg, 2.0);

            let prio_col = match &job.priority {
                JobPriority::Critical => p.error,
                JobPriority::High => p.warning,
                JobPriority::Normal => p.text,
                JobPriority::Low => p.text_muted,
            };
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &job.name, p.text, m.font_size_small);
            let st_lbl = format!("[{}] {:.0}ms", job.status.label(), job.elapsed_ms);
            ui.draw_list.text(Vec2::new(lx + 180.0, y + (row_h - m.font_size_small) * 0.5), &st_lbl, prio_col, m.font_size_small);

            if job.status == JobStatus::Running {
                let bar_x = lx + w * 0.6;
                let bar_w = w * 0.3;
                let bar_rect = Rect::new(bar_x, y + 4.0, bar_w, row_h - 8.0);
                ui.draw_list.rect(bar_rect, p.border, 2.0);
                let fill = Rect::new(bar_x, y + 4.0, bar_w * job.progress, row_h - 8.0);
                ui.draw_list.rect(fill, p.accent, 2.0);
            }

            let can_r = Rect::new(lx + w - 22.0, y + 2.0, 20.0, row_h - 4.0);
            if job.status == JobStatus::Pending {
                if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("cjob{}", job.id)), can_r).clicked {
                    cancel_job = Some(job.id);
                }
            }
            y += row_h + 2.0;
        }
        if let Some(jid) = cancel_job { sched.cancel(jid); }

        y += sp;
        let priority_labels = ["Critique", "Haute", "Normale", "Faible"];
        let btn_w = (w - sp * 3.0) / 4.0;
        for (k, lbl) in priority_labels.iter().enumerate() {
            let br = Rect::new(lx + k as f64 * (btn_w + sp), y, btn_w, row_h);
            let prio = match k {
                0 => JobPriority::Critical,
                1 => JobPriority::High,
                2 => JobPriority::Normal,
                _ => JobPriority::Low,
            };
            if Button::new(lbl).with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("sched{k}")), br).clicked {
                let name = format!("Tâche {:?} #{}", prio, sched.jobs.len() + 1);
                sched.schedule(name, prio);
            }
        }
    }
}
