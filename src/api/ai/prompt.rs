use crate::api::materials::catalog::MaterialCatalog;
use crate::api::scenes::builder::SceneBuilder;
use crate::core::engine::rendering::raytracing::Vec3;

pub fn scene_from_prompt(prompt: &str) -> SceneBuilder {
    let lower = prompt.to_lowercase();
    let tokens = tokenize(&lower);
    let mut builder = SceneBuilder::new();
    let catalog = MaterialCatalog;

    let mut object_index: usize = 0;
    let mut has_star = false;

    // ---------- star / sun ----------
    if contains_any(&tokens, &["star", "sun", "soleil", "étoile", "etoile"]) {
        builder = builder.add_sphere(
            Vec3::ZERO,
            1.6,
            catalog.by_name("stellar_surface"),
        );
        has_star = true;
        object_index += 1;
    }

    // ---------- planets ----------
    let planet_count = count_tokens(&tokens, &["planet", "planète", "planete"]);
    for i in 0..planet_count {
        let angle = std::f64::consts::TAU * i as f64 / planet_count.max(1) as f64;
        let dist = 5.0 + i as f64 * 3.2;
        let pos = Vec3::new(angle.cos() * dist, 0.2, angle.sin() * dist);
        builder = builder.add_sphere(pos, 0.55, catalog.by_name("rocky_world"));
        object_index += 1;
    }

    // ---------- ocean ----------
    if contains_any(&tokens, &["ocean", "océan", "eau", "water"]) {
        let pos = orbit_slot(object_index, has_star);
        builder = builder.add_sphere(pos, 0.6, catalog.by_name("ocean_world"));
        object_index += 1;
    }

    // ---------- ice ----------
    if contains_any(&tokens, &["ice", "glace", "frozen", "gelé", "gele"]) {
        let pos = orbit_slot(object_index, has_star);
        builder = builder.add_sphere(pos, 0.48, catalog.by_name("icy_world"));
        object_index += 1;
    }

    // ---------- moon ----------
    let moon_count = count_tokens(&tokens, &["moon", "lune"]);
    for i in 0..moon_count {
        let parent = if object_index > 0 { object_index - 1 } else { 0 };
        let angle = std::f64::consts::TAU * i as f64 / moon_count.max(1) as f64;
        let offset = Vec3::new(angle.cos() * 1.2, 0.4, angle.sin() * 1.2);
        let base = orbit_slot(parent, has_star);
        builder = builder.add_sphere(base + offset, 0.22, catalog.by_name("metallic_moon"));
        object_index += 1;
    }

    // ---------- car ----------
    if contains_any(&tokens, &["car", "voiture", "auto"]) {
        let pos = orbit_slot(object_index, has_star);
        builder = builder.add_sphere(pos, 0.5, catalog.by_name("automotive_paint"));
        object_index += 1;
    }

    // ---------- house ----------
    if contains_any(&tokens, &["house", "maison", "building", "bâtiment"]) {
        let pos = orbit_slot(object_index, has_star);
        builder = builder.add_sphere(pos, 0.65, catalog.by_name("architectural_plaster"));
        object_index += 1;
    }

    // ---------- tree ----------
    let tree_count = count_tokens(&tokens, &["tree", "arbre"]);
    for i in 0..tree_count {
        let pos = orbit_slot(object_index + i, has_star) + Vec3::new(0.0, 0.3, 0.0);
        builder = builder.add_sphere(pos, 0.4, catalog.by_name("tree_foliage"));
    }
    object_index += tree_count;

    // ---------- black hole ----------
    if contains_any(&tokens, &["black hole", "trou noir", "blackhole"]) {
        let pos = orbit_slot(object_index, has_star) + Vec3::new(0.0, 0.0, -4.0);
        builder = builder.add_sphere(pos, 0.9, catalog.by_name("event_horizon"));
        // accretion ring approximation
        for ring_i in 0..8 {
            let a = std::f64::consts::TAU * ring_i as f64 / 8.0;
            let ring_pos = pos + Vec3::new(a.cos() * 2.0, 0.05, a.sin() * 2.0);
            builder = builder.add_sphere(ring_pos, 0.18, catalog.by_name("accretion_disk"));
        }
        object_index += 1;
    }

    // ---------- nebula ----------
    if contains_any(&tokens, &["nebula", "nébuleuse", "nebuleuse", "fog", "brouillard"]) {
        builder = builder.with_dense_volume();
    }

    // Fallback: if nothing was added, add a default star + planet
    if object_index == 0 {
        builder = builder
            .add_sphere(Vec3::ZERO, 1.6, catalog.by_name("stellar_surface"))
            .add_sphere(Vec3::new(5.0, 0.3, 0.0), 0.55, catalog.by_name("ocean_world"));
    }

    builder
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tokenize(text: &str) -> Vec<String> {
    // Keep bigrams like "black hole" and "trou noir"
    let mut tokens: Vec<String> = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    for window in words.windows(2) {
        tokens.push(format!("{} {}", window[0], window[1]));
    }
    for word in &words {
        tokens.push((*word).to_string());
    }
    tokens
}

fn contains_any(tokens: &[String], needles: &[&str]) -> bool {
    needles.iter().any(|n| tokens.iter().any(|t| t == n))
}

fn count_tokens(tokens: &[String], needles: &[&str]) -> usize {
    // Count how many individual word tokens match (not bigrams)
    tokens
        .iter()
        .filter(|t| !t.contains(' ') && needles.contains(&t.as_str()))
        .count()
        .max(if contains_any(tokens, needles) { 1 } else { 0 })
}

fn orbit_slot(index: usize, has_star: bool) -> Vec3 {
    let base_dist = if has_star { 5.0 } else { 2.0 };
    let angle = std::f64::consts::TAU * index as f64 / 8.0;
    let dist = base_dist + index as f64 * 2.5;
    Vec3::new(angle.cos() * dist, 0.15, angle.sin() * dist)
}
