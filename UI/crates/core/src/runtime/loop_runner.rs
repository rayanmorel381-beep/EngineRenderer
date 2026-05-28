use std::time::Instant;

use enginerenderer::api::display::{BackendEvent, NativeWindow};

use crate::editor::shortcuts::{ShortcutAction, ShortcutTracker, default_bindings};
use crate::editor::{Editor, persistence};
use crate::platform::Platform;
use crate::state::AppState;
use crate::ui::immediate::context::UiContext;
use crate::ui::input::dispatch::dispatch;
use crate::ui::renderer::backend::RendererBackend;
use crate::ui::renderer::gl_overlay::GlOverlay;

pub fn run<P: Platform>(mut platform: P) -> i32 {
    let config = platform.config();
    let Some(mut window) = NativeWindow::open(config.width, config.height, &config.title) else {
        eprintln!("EngineRenderer app: failed to open native window");
        return 1;
    };

    let mut state = AppState::new(config.width, config.height);
    let mut ui = UiContext::new(config.width as u32, config.height as u32);
    let mut editor = Editor::new();
    if let Some(snapshot) = persistence::load() {
        persistence::restore(&mut editor, snapshot);
    }
    if let Some(scene) = persistence::load_scene() {
        editor.load_scene(scene);
    }
    for arg in std::env::args().skip(1) {
        editor.import_asset(std::path::Path::new(&arg));
    }
    let mut shortcuts = ShortcutTracker::new();
    let bindings = default_bindings();
    let mut renderer = GlOverlay::new();
    renderer.attach(&window);
    renderer.init(config.width as u32, config.height as u32);

    platform.on_start(&window, &mut state);

    let session_start = Instant::now();
    while !window.should_close() && !state.should_quit {
        let frame_start = Instant::now();

        for event in window.pump() {
            if let BackendEvent::Resized { width, height } = event {
                renderer.resize(width, height);
            }
            dispatch(&event, &mut ui, &mut state);
        }

        let drops = window.take_dropped_files();
        if !drops.is_empty() {
            editor.handle_dropped_files(&drops);
        }

        let time_seconds = session_start.elapsed().as_secs_f64();
        for action in shortcuts.poll(&ui, &bindings) {
            handle_shortcut(action, &mut state, &mut editor);
        }
        ui.begin_frame(state.frame_index, time_seconds);
        editor.show(&mut ui, &state);
        platform.on_frame(&window, &mut state);
        ui.end_frame();

        window.make_current();
        let clear = ui.theme.palette.background;
        renderer.begin_frame(clear);
        renderer.submit(&ui.draw_list);
        renderer.end_frame();
        window.swap_buffers();

        let elapsed = frame_start.elapsed().as_micros() as u64;
        state.tick(elapsed);
        editor.record_frame(elapsed);
    }

    platform.on_shutdown(&mut state);
    let _ = persistence::save(persistence::capture(&editor));
    let _ = persistence::save_scene(&editor.scene);
    0
}

fn handle_shortcut(action: ShortcutAction, state: &mut AppState, editor: &mut Editor) {
    use crate::views::console::ConsoleEntry;
    match action {
        ShortcutAction::Quit => state.should_quit = true,
        ShortcutAction::Save => {
            let _ = persistence::save_scene(&editor.scene);
            let _ = persistence::save(persistence::capture(editor));
            editor.console.push(ConsoleEntry::info("scene saved"));
        }
        ShortcutAction::Open => editor
            .console
            .push(ConsoleEntry::info("shortcut: open scene")),
        ShortcutAction::NewScene => editor
            .console
            .push(ConsoleEntry::info("shortcut: new scene")),
        ShortcutAction::Undo => editor.do_undo(),
        ShortcutAction::Redo => editor.do_redo(),
        ShortcutAction::DeleteSelected => editor.delete_selected(),
        ShortcutAction::DuplicateSelected => editor.duplicate_selected(),
        ShortcutAction::FrameSelected => editor.frame_selected(),
        ShortcutAction::ToggleEditMode => editor.toggle_edit_mode(),
        ShortcutAction::ExtrudeSelected => {
            if let Some(ref mut mesh) = editor.edit_mesh {
                crate::editor::mesh_ops::extrude_selected(mesh, 0.5);
            }
        }
        ShortcutAction::LoopCut => {
            if let Some(ref mut mesh) = editor.edit_mesh {
                let edges = crate::editor::mesh_ops::select_ring(mesh, crate::scene::mesh::EdgeId(0));
                crate::editor::mesh_ops::loop_cut(mesh, &edges, 0.5);
            }
        }
        ShortcutAction::Subdivide => {
            if let Some(ref mut mesh) = editor.edit_mesh {
                crate::editor::mesh_ops::subdivide(mesh);
            }
        }
    }
}

