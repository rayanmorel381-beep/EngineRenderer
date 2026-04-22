//! Smoke test de l'API de diagnostic.

use enginerenderer::api;

#[test]
fn diagnose_compute_environment_is_callable() {
    api::diagnose_compute_environment();
}
