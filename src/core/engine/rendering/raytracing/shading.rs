use super::acceleration::BvhNode;
use super::math::Vec3;
use super::primitives::{Material, Ray, EPSILON};
use super::scene::{DirectionalLight, Scene};
use crate::core::engine::rendering::{
    lod::manager::LodManager,
    lod::selection::LodSelection,
    texture::procedural_texture::ProceduralTexture,
};

#[derive(Debug, Clone, Copy)]
pub struct TraceContext<'a> {
    pub scene: &'a Scene,
    pub lod_manager: &'a LodManager,
    pub global_bounce_limit: u32,
    pub seed: u32,
    pub bvh: Option<&'a BvhNode>,
}

#[derive(Debug, Clone, Copy)]
pub struct AreaLightSample<'a> {
    pub scene: &'a Scene,
    pub point: Vec3,
    pub normal: Vec3,
    pub view_direction: Vec3,
    pub base_color: Vec3,
    pub material: Material,
    pub lod: LodSelection,
    pub seed: u32,
    pub bvh: Option<&'a BvhNode>,
}

#[derive(Debug, Clone, Copy)]
pub struct IndirectDiffuseInput<'a> {
    pub scene: &'a Scene,
    pub point: Vec3,
    pub normal: Vec3,
    pub lod_manager: &'a LodManager,
    pub depth: u32,
    pub global_bounce_limit: u32,
    pub samples: u32,
    pub seed: u32,
    pub bvh: Option<&'a BvhNode>,
}

pub fn shade_hit(ray: Ray, hit: &super::primitives::HitRecord, depth: u32, trace: TraceContext<'_>) -> Vec3 {
    let lod = trace.lod_manager.select(hit.distance, hit.radius);
    let local_limit = trace.global_bounce_limit.min(lod.max_bounces.max(1));
    let tex = hit.material.surface_texture();
    let normal = perturb_normal(hit.point, hit.uv, hit.normal, hit.material, lod, &tex);
    let base_color = hit.material.textured_albedo(hit.point, hit.uv, lod);
    let micro_roughness = tex.sample_roughness_uv(
        hit.point * (0.35 + lod.texture_frequency * 0.08),
        hit.uv,
        hit.material.uv_scale,
    );
    let effective_roughness = (hit.material.roughness * 0.68 + micro_roughness * 0.32).clamp(0.02, 0.98);

    let light_direction = (-trace.scene.sun.direction).normalize();
    let n_dot_l = normal.dot(light_direction).max(0.0);
    let shadow_samples = if depth == 0 { lod.shadow_samples } else { 1 };
    let visibility = if shadow_samples == 0 {
        1.0
    } else {
        soft_shadow(trace.scene, hit.point, normal, trace.scene.sun, shadow_samples, trace.seed, trace.bvh)
    };
    let view_direction = -ray.direction;

    let (fresnel, specular_strength, clearcoat_highlight) =
        compute_specular(normal, light_direction, view_direction, hit.material, base_color, effective_roughness);

    let diffuse = base_color * (1.0 - hit.material.metallic.clamp(0.0, 1.0)) * (1.0 - hit.material.transmission * 0.35);
    let specular_term = fresnel * specular_strength * lod.reflection_boost + Vec3::splat(clearcoat_highlight);
    let direct_light = ((diffuse * n_dot_l) + specular_term) * trace.scene.sun.color * trace.scene.sun.intensity * visibility;

    let subsurface_light = compute_subsurface(normal, light_direction, base_color, hit.material, trace.scene, visibility);

    let area_light = if depth == 0 && !trace.scene.area_lights.is_empty() && lod.shadow_samples > 0 {
        area_light_contribution(AreaLightSample {
            scene: trace.scene,
            point: hit.point,
            normal,
            view_direction,
            base_color,
            material: hit.material,
            lod,
            seed: trace.seed,
            bvh: trace.bvh,
        })
    } else {
        Vec3::ZERO
    };

    let ao = if depth == 0 && lod.ao_samples > 0 && hit.material.ambient_occlusion > 0.01 {
        ambient_occlusion(trace.scene, hit.point, normal, lod.ao_samples, trace.seed, trace.bvh) * hit.material.ambient_occlusion
    } else {
        1.0
    };
    let ambient = sky_color(trace.scene, normal, trace.lod_manager) * (0.16 + 0.52 * ao);
    let indirect = if depth == 0 && lod.ao_samples > 0 && trace.global_bounce_limit > 1 {
        indirect_diffuse(IndirectDiffuseInput {
            scene: trace.scene,
            point: hit.point,
            normal,
            lod_manager: trace.lod_manager,
            depth,
            global_bounce_limit: trace.global_bounce_limit,
            samples: lod.ao_samples,
            seed: trace.seed,
            bvh: trace.bvh,
        }) * diffuse
    } else {
        Vec3::ZERO
    };

    let caustics = if depth == 0 {
        caustic_estimate(trace.scene, hit.point, normal, view_direction) * (0.45 + ao * 0.55)
    } else {
        Vec3::ZERO
    };
    let rim_factor = (1.0 - normal.dot(view_direction).max(0.0)).powf(4.0);
    let rim = hit.material.sheen * rim_factor;
    let volume_light = trace.scene.volume.inscattering(ray, hit.distance.max(0.0), trace.scene.sun);
    let fast_density = trace.scene.volume.local_density_fast(ray.at(hit.distance * 0.5));
    let sigma = fast_density * (1.0 + trace.scene.volume.absorption);
    let transmittance = (-sigma * hit.distance.max(0.0) * 0.18).exp().clamp(0.0, 1.0);

    let mut shaded = (hit.material.emission + direct_light + subsurface_light + area_light + diffuse * ambient + indirect + rim + caustics)
        * transmittance
        + volume_light;

    if hit.material.transmission > 0.01 && depth + 1 < local_limit {
        let refracted_direction = ray.direction.refract(normal, 1.0 / hit.material.ior.max(1.01)).normalize();
        let refracted_ray = Ray::new(hit.point - normal * EPSILON * 3.0, refracted_direction);
        let transmitted = trace_ray(refracted_ray, depth + 1, TraceContext { seed: trace.seed ^ 0x51ED_270B, ..trace });
        shaded = shaded.lerp(transmitted + ambient * 0.25, hit.material.transmission.clamp(0.0, 0.85));
    }

    if depth + 1 < local_limit {
        let reflection_weight = (hit.material.reflectivity + hit.material.clearcoat * 0.18).clamp(0.0, 1.2);
        if reflection_weight > 0.01 {
            let reflected = ray.direction.reflect(normal).normalize();
            let glossy_direction = (reflected + random_in_unit_sphere(trace.seed ^ 0x9E37_79B9) * effective_roughness).normalize();
            let reflected_ray = Ray::new(hit.point + normal * EPSILON * 3.0, glossy_direction);
            let reflected_light = trace_ray(reflected_ray, depth + 1, TraceContext { seed: trace.seed ^ 0x85EB_CA6B, ..trace });
            shaded += reflected_light * reflection_weight * lod.reflection_boost;
        }
    }

    shaded
}

pub fn trace_ray(ray: Ray, depth: u32, trace: TraceContext<'_>) -> Vec3 {
    if let Some(hit) = BvhNode::hit_scene(trace.scene, &ray, EPSILON, f64::INFINITY, trace.bvh) {
        shade_hit(ray, &hit, depth, trace)
    } else {
        let horizon = trace.lod_manager.horizon_detail(100.0);
        let sky = sky_color(trace.scene, ray.direction, trace.lod_manager) * (0.7 + horizon * 0.3);
        let volume_light = trace.scene.volume.inscattering(ray, 120.0, trace.scene.sun);
        let fast_density = trace.scene.volume.local_density_fast(ray.at(42.0));
        let sigma = fast_density * (1.0 + trace.scene.volume.absorption);
        let transmittance = (-sigma * 120.0 * 0.18).exp().clamp(0.0, 1.0);
        (sky * transmittance + volume_light).clamp(0.0, 12.0)
    }
}

fn compute_specular(
    normal: Vec3,
    light_direction: Vec3,
    view_direction: Vec3,
    material: Material,
    base_color: Vec3,
    effective_roughness: f64,
) -> (Vec3, f64, f64) {
    let half_vector = (light_direction + view_direction).normalize();
    let tangent_hint = if normal.x.abs() > normal.z.abs() {
        Vec3::new(-normal.y, normal.x, 0.0)
    } else {
        Vec3::new(0.0, -normal.z, normal.y)
    };
    let tangent = tangent_hint.normalize();
    let bitangent = normal.cross(tangent).normalize();
    let anisotropy = material.anisotropy.clamp(0.0, 1.0);
    let anisotropy_shape = (half_vector.dot(tangent).abs() * (1.0 + anisotropy * 1.8)
        + half_vector.dot(bitangent).abs() * (1.0 - anisotropy * 0.55))
        .clamp(0.35, 2.0);
    let specular_power = 20.0 + (1.0 - effective_roughness) * 180.0;
    let specular_strength = normal.dot(half_vector).max(0.0).powf(specular_power) * anisotropy_shape;
    let clearcoat_highlight = normal.dot(half_vector).max(0.0).powf(220.0) * material.clearcoat;

    let fresnel = Vec3::splat(0.04).lerp(base_color, material.metallic.clamp(0.0, 1.0));
    let rim_factor = (1.0 - normal.dot(view_direction).max(0.0)).powf(4.0);
    let iridescent_tint = Vec3::new(
        (rim_factor * 7.0 + 0.0).sin() * 0.5 + 0.5,
        (rim_factor * 7.0 + 2.1).sin() * 0.5 + 0.5,
        (rim_factor * 7.0 + 4.2).sin() * 0.5 + 0.5,
    );
    let fresnel = fresnel.lerp(fresnel * (Vec3::splat(0.72) + iridescent_tint * 0.9), material.iridescence);

    (fresnel, specular_strength, clearcoat_highlight)
}

fn compute_subsurface(
    normal: Vec3,
    light_direction: Vec3,
    base_color: Vec3,
    material: Material,
    scene: &Scene,
    visibility: f64,
) -> Vec3 {
    let n_dot_l = normal.dot(light_direction).max(0.0);
    let wrap = 0.35 + material.subsurface * 0.45;
    let wrapped_diffuse = ((n_dot_l + wrap) / (1.0 + wrap)).clamp(0.0, 1.0);
    let back_scatter = (-normal).dot(light_direction).max(0.0).powf(1.5);
    base_color
        * scene.sun.color
        * scene.sun.intensity
        * (wrapped_diffuse * 0.18 + back_scatter * 0.65)
        * material.subsurface
        * (0.35 + visibility * 0.65)
}

pub fn soft_shadow(
    scene: &Scene,
    point: Vec3,
    normal: Vec3,
    sun: DirectionalLight,
    samples: u32,
    seed: u32,
    bvh: Option<&BvhNode>,
) -> f64 {
    if samples == 0 {
        return 1.0;
    }

    let total = samples.max(1);
    let mut vis = 0.0;

    for i in 0..total {
        let basis = random_in_unit_sphere(seed ^ i.wrapping_mul(0x27D4_EB2D));
        let jittered = (-(sun.direction.normalize()) + basis * sun.angular_radius).normalize();
        let shadow_ray = Ray::new(point + normal * EPSILON * 4.0, jittered);
        if !BvhNode::any_hit(scene, &shadow_ray, 500.0, bvh) {
            vis += 1.0;
        }
    }

    vis / total as f64
}

pub fn area_light_contribution(input: AreaLightSample<'_>) -> Vec3 {
    let mut contribution = Vec3::ZERO;

    for (li, light) in input.scene.area_lights.iter().enumerate() {
        let sample_count = input.lod.shadow_samples.clamp(1, 2) as usize;
        let mut light_total = Vec3::ZERO;

        for si in 0..sample_count {
            let ss = input.seed ^ ((li as u32 + 1).wrapping_mul(0x7FEB_352D)) ^ ((si as u32 + 1).wrapping_mul(0x846C_A68B));
            let su = random_scalar(ss ^ 0xA24B_AED4);
            let sv = random_scalar(ss ^ 0x9FB2_1C65);
            let lp = light.sample_point(su, sv);
            let to_light = lp - input.point;
            let dist_sq = to_light.length_squared().max(0.01);
            let dist = dist_sq.sqrt();
            let ld = to_light / dist;
            let ndl = input.normal.dot(ld).max(0.0);

            if ndl <= 0.0 { continue; }

            let sr = Ray::new(input.point + input.normal * EPSILON * 4.0, ld);
            if BvhNode::any_hit(input.scene, &sr, (dist - EPSILON * 6.0).max(EPSILON), input.bvh) { continue; }

            let hv = (ld + input.view_direction).normalize();
            let th = if input.normal.x.abs() > input.normal.z.abs() { Vec3::new(-input.normal.y, input.normal.x, 0.0) } else { Vec3::new(0.0, -input.normal.z, input.normal.y) };
            let t = th.normalize();
            let bt = input.normal.cross(t).normalize();
            let aniso = input.material.anisotropy.clamp(0.0, 1.0);
            let ab = (hv.dot(t).abs() * (1.0 + aniso * 1.5) + hv.dot(bt).abs() * (1.0 - aniso * 0.5)).clamp(0.4, 2.0);
            let sp = 24.0 + (1.0 - input.material.roughness.clamp(0.02, 0.98)) * 160.0;
            let spec = input.normal.dot(hv).max(0.0).powf(sp) * (0.22 + input.material.reflectivity * 0.78 + input.material.clearcoat * 0.35) * ab;
            let fresnel = Vec3::splat(0.04).lerp(input.base_color, input.material.metallic.clamp(0.0, 1.0));
            let rf = (1.0 - input.normal.dot(input.view_direction).max(0.0)).powf(3.0);
            let irid = Vec3::new((rf * 7.0).sin() * 0.5 + 0.5, (rf * 7.0 + 2.1).sin() * 0.5 + 0.5, (rf * 7.0 + 4.2).sin() * 0.5 + 0.5);
            let fresnel = fresnel.lerp(fresnel * (Vec3::splat(0.7) + irid * 0.95), input.material.iridescence);
            let diff = input.base_color * (1.0 - input.material.metallic.clamp(0.0, 1.0)) * ndl;
            let sss = input.base_color * input.material.subsurface * (0.12 + rf * 0.35);
            let atten = light.intensity / (1.0 + dist_sq * 0.08);
            light_total += (diff + sss + fresnel * spec * input.lod.reflection_boost) * light.color * atten;
        }

        contribution += light_total / sample_count as f64;
    }

    contribution
}

pub fn indirect_diffuse(input: IndirectDiffuseInput<'_>) -> Vec3 {
    if input.samples == 0 || input.depth + 1 >= input.global_bounce_limit.max(1) {
        return Vec3::ZERO;
    }

    let total = input.samples.clamp(1, 2);
    let mut indirect = Vec3::ZERO;
    let sun_dir = (-input.scene.sun.direction).normalize();

    for i in 0..total {
        let ds = input.seed ^ i.wrapping_mul(0x94D0_49BB);
        let rd = random_hemisphere(input.normal, ds);
        let id = (input.normal * 0.55 + sun_dir * 0.30 + random_in_unit_sphere(ds ^ 0xA24B_AED4) * 0.15).normalize();
        let dir = if i % 3 == 0 { id } else { rd };
        let br = Ray::new(input.point + input.normal * EPSILON * 3.0, dir);
        let cosine = input.normal.dot(dir).max(0.0);

        let bounced = if let Some(hit) = BvhNode::hit_scene(input.scene, &br, EPSILON, 16.0, input.bvh) {
            let ba = hit.material.albedo;
            let bndl = hit.normal.dot(sun_dir).max(0.0);
            let sr = Ray::new(hit.point + hit.normal * EPSILON * 4.0, sun_dir);
            let bv = if BvhNode::any_hit(input.scene, &sr, 200.0, input.bvh) { 0.3 } else { 1.0 };
            (ba * bndl * bv * 0.60 + hit.material.emission * 0.24 + sky_color(input.scene, dir, input.lod_manager) * 0.16) * cosine
        } else {
            sky_color(input.scene, dir, input.lod_manager) * cosine * 0.48
        };

        indirect += bounced;
    }

    indirect / total as f64
}

pub fn caustic_estimate(scene: &Scene, point: Vec3, normal: Vec3, view_direction: Vec3) -> Vec3 {
    let mut caustic = Vec3::ZERO;
    let ld = (-scene.sun.direction).normalize();
    let receive = normal.dot(ld).max(0.0);
    if receive <= 0.0 { return Vec3::ZERO; }

    for obj in &scene.objects {
        let focus = obj.material.transmission * 1.35 + obj.material.clearcoat * 0.45 + obj.material.reflectivity * 0.20;
        if focus <= 0.08 { continue; }

        let to_focus = obj.center - point;
        let dist = to_focus.length().max(0.2);
        let alignment = to_focus.normalize().dot(ld).max(0.0).powf(14.0);
        let conc = (obj.radius / dist).clamp(0.0, 1.4);
        let edge = (1.0 - normal.dot(view_direction).max(0.0)).powf(2.5) * 0.18 + 0.10;
        let tint = obj.material.albedo * 0.42 + scene.sun.color * 0.58;
        caustic += tint * alignment * conc * focus * receive * edge;
    }

    caustic.clamp(0.0, 2.8)
}

pub fn ambient_occlusion(
    scene: &Scene,
    point: Vec3,
    normal: Vec3,
    samples: u32,
    seed: u32,
    bvh: Option<&BvhNode>,
) -> f64 {
    if samples == 0 {
        return 1.0;
    }

    let total = samples.max(1);
    let mut clear = 0.0;

    for i in 0..total {
        let dir = random_hemisphere(normal, seed ^ i.wrapping_mul(0x94D0_49BB));
        let ray = Ray::new(point + normal * EPSILON * 2.0, dir);
        if !BvhNode::any_hit(scene, &ray, 2.5, bvh) {
            clear += 1.0;
        }
    }

    clear / total as f64
}

pub fn sky_color(scene: &Scene, direction: Vec3, lod_manager: &LodManager) -> Vec3 {
    let t = 0.5 * (direction.y + 1.0);
    let sun_direction = (-scene.sun.direction).normalize();
    let base = scene.sky_bottom.lerp(scene.sky_top, t);
    let horizon_factor = lod_manager.horizon_detail(800.0);
    let horizon_haze = scene.sky_bottom.lerp(scene.sky_top, 0.35) * (1.0 - direction.y.abs()).powf(3.0) * 0.35 * horizon_factor;
    let sun_alignment = direction.dot(sun_direction).max(0.0);
    let sun_disc = scene.sun.color * scene.sun.intensity * sun_alignment.powf(2200.0 * (1.0 - scene.sun.angular_radius).max(0.2));
    let sun_halo = scene.sun.color * sun_alignment.powf(18.0) * 0.12;
    let stars = Vec3::splat(star_field(direction) * (1.0 - t).powf(2.6) * 0.8);
    let hdri = if let Some(ref env) = scene.hdri {
        let sun_direction = (-scene.sun.direction).normalize();
        env.hdri_probe(direction, sun_direction) * 0.15
    } else {
        Vec3::ZERO
    };
    (base + horizon_haze + sun_halo + sun_disc + stars + hdri).clamp(0.0, 12.0)
}

fn perturb_normal(
    point: Vec3,
    uv: Option<(f64, f64)>,
    normal: Vec3,
    material: Material,
    lod: LodSelection,
    tex: &ProceduralTexture,
) -> Vec3 {
    let wave = Vec3::new(
        (point.x * lod.texture_frequency).sin() * 0.12 * lod.normal_intensity,
        ((point.x + point.z) * lod.texture_frequency * 0.55).cos() * 0.05 * lod.normal_intensity,
        (point.z * lod.texture_frequency * 1.2).sin() * 0.12 * lod.normal_intensity,
    );
    let texture_normal = tex.sample_normal_uv(point * (0.35 + lod.texture_frequency * 0.08), uv, material.uv_scale);
    (normal + wave + texture_normal * material.normal_map_strength * (0.8 + material.clearcoat * 0.3)).normalize()
}

fn star_field(direction: Vec3) -> f64 {
    let hash = direction.x.to_bits() ^ direction.y.to_bits().rotate_left(11) ^ direction.z.to_bits().rotate_left(19) ^ 0xA511_E9B3;
    let folded = (hash as u32) ^ ((hash >> 32) as u32);
    let sparkle = random_scalar(folded);
    if sparkle > 0.996 { (sparkle - 0.996) * 250.0 } else { 0.0 }
}

pub fn random_hemisphere(normal: Vec3, seed: u32) -> Vec3 {
    let v = random_in_unit_sphere(seed).normalize();
    if v.dot(normal) < 0.0 { -v } else { v }
}

pub fn random_in_unit_sphere(seed: u32) -> Vec3 {
    let x = random_scalar(seed ^ 0x68BC_21EB) * 2.0 - 1.0;
    let y = random_scalar(seed ^ 0x02E5_BE93) * 2.0 - 1.0;
    let z = random_scalar(seed ^ 0x967A_889B) * 2.0 - 1.0;
    let candidate = Vec3::new(x, y, z);
    if candidate.length_squared() <= 1.0 { candidate } else { candidate.normalize() * 0.999 }
}

pub fn random_scalar(seed: u32) -> f64 {
    let mut v = seed.wrapping_add(0x9E37_79B9);
    v ^= v >> 16;
    v = v.wrapping_mul(0x85EB_CA6B);
    v ^= v >> 13;
    v = v.wrapping_mul(0xC2B2_AE35);
    v ^= v >> 16;
    v as f64 / u32::MAX as f64
}

pub fn make_seed(x: u32, y: u32, sample: u32) -> u32 {
    x.wrapping_mul(1973) ^ y.wrapping_mul(9277) ^ sample.wrapping_mul(26699) ^ 0xA511_E9B3
}

pub fn luminance_estimate(color: Vec3) -> f64 {
    color.x * 0.2126 + color.y * 0.7152 + color.z * 0.0722
}

pub fn tone_map(color: Vec3, exposure: f64) -> Vec3 {
    let exposed = color * exposure.max(0.1);
    Vec3::new(aces_curve(exposed.x), aces_curve(exposed.y), aces_curve(exposed.z))
}

fn aces_curve(value: f64) -> f64 {
    ((value * (2.51 * value + 0.03)) / (value * (2.43 * value + 0.59) + 0.14)).clamp(0.0, 1.0)
}
