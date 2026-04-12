//! Integration tests for the main Engine entry point.

use enginerenderer::api::engine::Engine;

#[test]
fn engine_default_run_produces_pixels() {
    let engine = Engine::test_minimal();
    let report = engine.run().expect("Engine::run failed");
    assert!(report.rendered_pixels > 0, "The renderer produced no pixels.");
    assert!(report.output_path.exists(), "The rendered frame was not written to disk.");
    assert_eq!(report.rendered_pixels, report.width * report.height);
    assert!(report.duration_ms < u128::MAX);
    assert!(report.object_count > 0);
    assert!(report.triangle_count > 0 || report.object_count > 0);
    assert!(report.bvh.node_count >= report.bvh.leaf_count);
}

#[test]
fn engine_gallery_produces_multiple_reports() {
    let engine = Engine::test_minimal();
    let reports = engine
        .render_gallery()
        .expect("render_gallery failed");
    assert!(reports.len() >= 6);
    assert!(reports.iter().all(|r| r.output_path.exists()));
    assert!(reports.iter().all(|r| r.rendered_pixels == r.width * r.height));
}

#[test]
fn engine_production_reference_runs() {
    let engine = Engine::test_minimal();
    let report = engine.run().expect("production_reference run failed");
    assert!(report.rendered_pixels > 0);
}
