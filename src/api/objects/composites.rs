use crate::api::objects::SceneObject;
use crate::core::engine::rendering::raytracing::Vec3;

/// Composite objects built from multiple primitives.
///
/// Each function returns a [`SceneObject::Group`] that the scene builder
/// flattens automatically.
impl SceneObject {
    /// Solar system: central star with orbiting planets.
    pub fn solar_system(center: [f64; 3], star_radius: f64, planet_count: usize) -> Self {
        let mut objects = vec![Self::star(center, star_radius)];
        for i in 0..planet_count {
            let angle = std::f64::consts::TAU * i as f64 / planet_count.max(1) as f64;
            let dist = star_radius * 3.0 + i as f64 * star_radius * 2.0;
            let pos = [
                center[0] + dist * angle.cos(),
                center[1] + 0.15,
                center[2] + dist * angle.sin(),
            ];
            let r = star_radius * 0.25 + (i % 3) as f64 * star_radius * 0.08;
            let obj = match i % 4 {
                0 => Self::planet(pos, r),
                1 => Self::ocean_planet(pos, r),
                2 => Self::ice_planet(pos, r),
                _ => Self::lush_planet(pos, r),
            };
            objects.push(obj);
        }
        Self::Group(objects)
    }

    /// Black hole with accretion ring.
    pub fn black_hole_system(center: [f64; 3], hole_radius: f64) -> Self {
        let mut objects = vec![Self::black_hole(center, hole_radius)];
        for i in 0..16 {
            let angle = std::f64::consts::TAU * i as f64 / 16.0;
            let ring_r = hole_radius * 2.8;
            let pos = [
                center[0] + ring_r * angle.cos(),
                center[1] + 0.08 * ((i % 3) as f64 - 1.0),
                center[2] + ring_r * angle.sin(),
            ];
            objects.push(Self::sphere(pos, hole_radius * 0.12, "accretion_disk"));
        }
        Self::Group(objects)
    }

    /// Smooth tree built only from spheres.
    pub fn tree(base: [f64; 3], height: f64) -> Self {
        let scale = height.max(0.2);
        let objects = vec![
            Self::sphere([base[0], base[1] + 0.18 * scale, base[2]], 0.12 * scale, "tree_bark"),
            Self::sphere([base[0], base[1] + 0.42 * scale, base[2]], 0.11 * scale, "tree_bark"),
            Self::sphere([base[0], base[1] + 0.68 * scale, base[2]], 0.10 * scale, "tree_bark"),
            Self::sphere([base[0], base[1] + 0.96 * scale, base[2]], 0.09 * scale, "tree_bark"),
            Self::sphere([base[0], base[1] + 1.24 * scale, base[2]], 0.08 * scale, "tree_bark"),
            Self::sphere([base[0], base[1] + 1.72 * scale, base[2]], 0.58 * scale, "tree_foliage"),
            Self::sphere([base[0] - 0.28 * scale, base[1] + 1.46 * scale, base[2] + 0.18 * scale], 0.42 * scale, "tree_foliage"),
            Self::sphere([base[0] + 0.30 * scale, base[1] + 1.42 * scale, base[2] - 0.16 * scale], 0.40 * scale, "tree_foliage"),
            Self::sphere([base[0] - 0.08 * scale, base[1] + 1.92 * scale, base[2] - 0.12 * scale], 0.34 * scale, "tree_foliage"),
            Self::sphere([base[0] + 0.22 * scale, base[1] + 1.86 * scale, base[2] + 0.16 * scale], 0.30 * scale, "tree_foliage"),
            Self::sphere([base[0] - 0.34 * scale, base[1] + 1.70 * scale, base[2] - 0.20 * scale], 0.28 * scale, "tree_foliage"),
        ];
        Self::Group(objects)
    }

    /// Smooth domed house — no low-poly facets.
    pub fn house(center: [f64; 3], size: f64) -> Self {
        let scale = size.max(0.2);
        let mut objects = vec![
            Self::sphere([center[0], center[1] + 0.55 * scale, center[2]], 0.78 * scale, "architectural_plaster"),
            Self::sphere([center[0] - 0.58 * scale, center[1] + 0.48 * scale, center[2] + 0.10 * scale], 0.48 * scale, "architectural_plaster"),
            Self::sphere([center[0] + 0.58 * scale, center[1] + 0.48 * scale, center[2] - 0.10 * scale], 0.48 * scale, "architectural_plaster"),
            Self::sphere([center[0], center[1] + 1.12 * scale, center[2]], 0.62 * scale, "roof_tiles"),
            Self::sphere([center[0] - 0.40 * scale, center[1] + 1.02 * scale, center[2] + 0.14 * scale], 0.32 * scale, "roof_tiles"),
            Self::sphere([center[0] + 0.40 * scale, center[1] + 1.02 * scale, center[2] - 0.14 * scale], 0.32 * scale, "roof_tiles"),
            Self::sphere([center[0] - 0.18 * scale, center[1] + 1.54 * scale, center[2] - 0.24 * scale], 0.12 * scale, "architectural_plaster"),
            Self::sphere([center[0], center[1] + 0.22 * scale, center[2] + 0.68 * scale], 0.18 * scale, "tree_bark"),
        ];

        for &(x, y) in &[(-0.34, 0.68), (0.34, 0.68), (-0.34, 0.36), (0.34, 0.36)] {
            objects.push(Self::sphere(
                [center[0] + x * scale, center[1] + y * scale, center[2] + 0.58 * scale],
                0.12 * scale,
                "window_glass",
            ));
        }

        Self::Group(objects)
    }

    /// Smooth retro-futuristic car/hovercar made from blended spheres.
    pub fn car(center: [f64; 3], length: f64) -> Self {
        let scale = length.max(0.3) * 0.45;
        let mut objects = vec![
            Self::sphere([center[0] - 0.55 * scale, center[1] + 0.34 * scale, center[2]], 0.34 * scale, "automotive_paint"),
            Self::sphere([center[0], center[1] + 0.38 * scale, center[2]], 0.42 * scale, "automotive_paint"),
            Self::sphere([center[0] + 0.58 * scale, center[1] + 0.32 * scale, center[2]], 0.30 * scale, "automotive_paint"),
            Self::sphere([center[0] - 0.10 * scale, center[1] + 0.62 * scale, center[2]], 0.24 * scale, "window_glass"),
            Self::sphere([center[0] + 0.26 * scale, center[1] + 0.56 * scale, center[2]], 0.18 * scale, "window_glass"),
        ];

        for &(x, z) in &[(-0.62, -0.36), (-0.62, 0.36), (0.62, -0.36), (0.62, 0.36)] {
            objects.push(Self::sphere(
                [center[0] + x * scale, center[1] + 0.08 * scale, center[2] + z * scale],
                0.18 * scale,
                "rubber_tire",
            ));
            objects.push(Self::sphere(
                [center[0] + x * scale, center[1] + 0.08 * scale, center[2] + z * scale],
                0.08 * scale,
                "metallic_moon",
            ));
        }

        for z in [-0.18, 0.18] {
            objects.push(Self::sphere(
                [center[0] - 0.88 * scale, center[1] + 0.28 * scale, center[2] + z * scale],
                0.05 * scale,
                "solar_corona",
            ));
        }

        Self::Group(objects)
    }

    /// Row of objects spaced evenly along the X axis.
    pub fn row(objects: Vec<SceneObject>, spacing: f64, start_x: f64) -> Self {
        let mut positioned = Vec::with_capacity(objects.len());
        for (i, obj) in objects.into_iter().enumerate() {
            let offset = Vec3::new(start_x + i as f64 * spacing, 0.0, 0.0);
            positioned.push(Self::translated(obj, offset));
        }
        Self::Group(positioned)
    }

    fn translated(obj: SceneObject, offset: Vec3) -> SceneObject {
        match obj {
            SceneObject::Sphere { center, radius, material } => SceneObject::Sphere {
                center: center + offset,
                radius,
                material,
            },
            SceneObject::Triangle { a, b, c, material } => SceneObject::Triangle {
                a: a + offset,
                b: b + offset,
                c: c + offset,
                material,
            },
            SceneObject::Group(children) => SceneObject::Group(
                children.into_iter().map(|c| Self::translated(c, offset)).collect(),
            ),
        }
    }
}

