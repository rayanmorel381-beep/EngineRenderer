//! GLSL compute shader source for the GPU path tracer with BVH traversal.

/// SSBO binding indices used by the shader.
pub mod bindings {
    pub const OUTPUT: u32 = 0;
    pub const FRAME: u32 = 1;
    pub const SPHERES: u32 = 2;
    pub const TRIANGLES: u32 = 3;
    pub const AREA_LIGHTS: u32 = 4;
    pub const BVH_NODES: u32 = 5;
    pub const BVH_PRIMS: u32 = 6;
}

const SHADER_BODY: &str = r#"
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

struct Sphere {
    vec4 center_radius;
    vec4 albedo_rough;
    vec4 emission_metal;
    vec4 transmission_ior_pad;
    vec4 tex_base_scale;
    vec4 tex_accent_detail;
    vec4 tex_kind_weight_uvs;
};

struct Triangle {
    vec4 a_pad;
    vec4 b_pad;
    vec4 c_pad;
    vec4 albedo_rough;
    vec4 emission_metal;
    vec4 transmission_ior_pad;
    vec4 tex_base_scale;
    vec4 tex_accent_detail;
    vec4 tex_kind_weight_uvs;
};

struct AreaLight {
    vec4 position_pad;
    vec4 u_pad;
    vec4 v_pad;
    vec4 color_intensity;
};

struct BvhNode {
    vec4 min_leafcount;
    vec4 max_rightorfirst;
};

struct Frame {
    vec4 cam_origin_w;
    vec4 cam_lower_left_w;
    vec4 cam_horizontal_w;
    vec4 cam_vertical_w;
    vec4 sun_dir_intensity;
    vec4 sun_color_w;
    vec4 sky_top_w;
    vec4 sky_bottom_w;
    vec4 atmo_clouds;
    vec4 exposure_pad;
    uvec4 image_size_counts;
    uvec4 sample_seed_bounces;
};

layout(std430, binding = OUTPUT_BINDING) buffer OutBuf {
    vec4 pixels[];
} out_buf;

layout(std430, binding = FRAME_BINDING) readonly buffer FrameBuf {
    Frame frame;
} frame_buf;

layout(std430, binding = SPHERE_BINDING) readonly buffer SphereBuf {
    Sphere data[];
} spheres;

layout(std430, binding = TRIANGLE_BINDING) readonly buffer TriBuf {
    Triangle data[];
} triangles;

layout(std430, binding = AREA_LIGHT_BINDING) readonly buffer AreaBuf {
    AreaLight data[];
} area_lights;

layout(std430, binding = BVH_NODES_BINDING) readonly buffer BvhNodesBuf {
    BvhNode data[];
} bvh_nodes;

layout(std430, binding = BVH_PRIMS_BINDING) readonly buffer BvhPrimsBuf {
    uvec4 data[];
} bvh_prims;

uint pcg_hash(uint s) {
    s = s * 747796405u + 2891336453u;
    uint w = ((s >> ((s >> 28) + 4u)) ^ s) * 277803737u;
    return (w >> 22) ^ w;
}

float rand_float(inout uint state) {
    state = pcg_hash(state);
    return float(state) / 4294967296.0;
}

vec3 sample_hemisphere(vec3 normal, inout uint state) {
    float r1 = rand_float(state);
    float r2 = rand_float(state);
    float phi = 6.2831853 * r1;
    float cos_theta = sqrt(1.0 - r2);
    float sin_theta = sqrt(r2);
    vec3 w = normalize(normal);
    vec3 a = abs(w.x) > 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
    vec3 v_axis = normalize(cross(w, a));
    vec3 u_axis = cross(w, v_axis);
    return normalize(u_axis * (cos(phi) * sin_theta) + v_axis * (sin(phi) * sin_theta) + w * cos_theta);
}

struct Hit {
    float t;
    vec3 point;
    vec3 normal;
    vec3 albedo;
    vec3 emission;
    float roughness;
    float metallic;
    float transmission;
    float ior;
    int hit_kind;
};

vec3 proc_tex_apply(vec3 base_albedo, vec3 point, vec4 tex_base_scale, vec4 tex_accent_detail, vec4 tex_kind_weight_uvs) {
    float weight = tex_kind_weight_uvs.y;
    if (weight < 0.001) return base_albedo;
    uint kind = floatBitsToUint(tex_kind_weight_uvs.x);
    float scale = tex_base_scale.w;
    float uv_scale = tex_kind_weight_uvs.z;
    vec3 tex_base = tex_base_scale.xyz;
    vec3 accent = tex_accent_detail.xyz;
    float u = fract(abs(point.x));
    float v = fract(abs(point.z));
    float TAU = 6.2831853;
    float PI = 3.14159265;
    float uv_wave = (sin(u * uv_scale * scale * TAU) * 0.5 + 0.5) * 0.58
                  + (cos(v * uv_scale * scale * PI) * 0.5 + 0.5) * 0.42;
    float primary = sin((point.x + point.z * 0.35) * scale) * 0.5 + 0.5;
    float secondary = cos((point.y * 0.55 - point.z * 0.25) * scale * 1.37) * 0.5 + 0.5;
    float blend = clamp(primary * 0.42 + secondary * 0.22 + uv_wave * 0.36, 0.0, 1.0);
    float exponent = 1.2;
    if (kind == 0u) exponent = 1.6;
    else if (kind == 1u) exponent = 0.85;
    else if (kind == 2u) exponent = 2.2;
    vec3 textured = mix(tex_base, accent, pow(blend, exponent));
    return mix(base_albedo, textured, weight);
}

bool hit_sphere(int idx, vec3 ro, vec3 rd, float t_min, float t_max, inout Hit best) {
    Sphere s = spheres.data[idx];
    vec3 oc = ro - s.center_radius.xyz;
    float r = s.center_radius.w;
    float b = dot(oc, rd);
    float c = dot(oc, oc) - r * r;
    float disc = b * b - c;
    if (disc < 0.0) return false;
    float sq = sqrt(disc);
    float t = -b - sq;
    if (t < t_min || t > t_max) {
        t = -b + sq;
        if (t < t_min || t > t_max) return false;
    }
    if (t >= best.t) return false;
    best.t = t;
    best.point = ro + rd * t;
    best.normal = (best.point - s.center_radius.xyz) / max(r, 1e-6);
    best.albedo = proc_tex_apply(s.albedo_rough.xyz, best.point, s.tex_base_scale, s.tex_accent_detail, s.tex_kind_weight_uvs);
    best.roughness = s.albedo_rough.w;
    best.emission = s.emission_metal.xyz;
    best.metallic = s.emission_metal.w;
    best.transmission = s.transmission_ior_pad.x;
    best.ior = s.transmission_ior_pad.y;
    best.hit_kind = 1;
    return true;
}

bool hit_triangle(int idx, vec3 ro, vec3 rd, float t_min, float t_max, inout Hit best) {
    Triangle tri = triangles.data[idx];
    vec3 a = tri.a_pad.xyz;
    vec3 b = tri.b_pad.xyz;
    vec3 c = tri.c_pad.xyz;
    vec3 ab = b - a;
    vec3 ac = c - a;
    vec3 p = cross(rd, ac);
    float det = dot(ab, p);
    if (abs(det) < 1e-8) return false;
    float inv_det = 1.0 / det;
    vec3 tvec = ro - a;
    float u = dot(tvec, p) * inv_det;
    if (u < 0.0 || u > 1.0) return false;
    vec3 q = cross(tvec, ab);
    float v = dot(rd, q) * inv_det;
    if (v < 0.0 || u + v > 1.0) return false;
    float t = dot(ac, q) * inv_det;
    if (t < t_min || t > t_max) return false;
    if (t >= best.t) return false;
    best.t = t;
    best.point = ro + rd * t;
    vec3 nrm = normalize(cross(ab, ac));
    if (dot(nrm, rd) > 0.0) nrm = -nrm;
    best.normal = nrm;
    best.albedo = proc_tex_apply(tri.albedo_rough.xyz, best.point, tri.tex_base_scale, tri.tex_accent_detail, tri.tex_kind_weight_uvs);
    best.roughness = tri.albedo_rough.w;
    best.emission = tri.emission_metal.xyz;
    best.metallic = tri.emission_metal.w;
    best.transmission = tri.transmission_ior_pad.x;
    best.ior = tri.transmission_ior_pad.y;
    best.hit_kind = 2;
    return true;
}

bool aabb_hit(vec3 bmin, vec3 bmax, vec3 ro, vec3 inv_dir, float t_max) {
    vec3 t0 = (bmin - ro) * inv_dir;
    vec3 t1 = (bmax - ro) * inv_dir;
    vec3 tmin = min(t0, t1);
    vec3 tmax = max(t0, t1);
    float tn = max(max(tmin.x, tmin.y), tmin.z);
    float tf = min(min(tmax.x, tmax.y), tmax.z);
    return tf >= max(tn, 0.0) && tn < t_max;
}

bool intersect_scene(vec3 ro, vec3 rd, float t_min, float t_max, out Hit hit) {
    hit.t = t_max;
    hit.hit_kind = 0;
    bool found = false;
    vec3 inv_dir = 1.0 / rd;

    uint stack[32];
    int sp = 0;
    stack[sp] = 0u;
    sp = 1;

    while (sp > 0) {
        sp = sp - 1;
        uint node_idx = stack[sp];
        BvhNode n = bvh_nodes.data[node_idx];
        vec3 bmin = n.min_leafcount.xyz;
        vec3 bmax = n.max_rightorfirst.xyz;
        if (!aabb_hit(bmin, bmax, ro, inv_dir, hit.t)) continue;

        uint leaf_count = floatBitsToUint(n.min_leafcount.w);
        uint payload = floatBitsToUint(n.max_rightorfirst.w);
        if (leaf_count > 0u) {
            for (uint k = 0u; k < leaf_count; ++k) {
                uvec4 p = bvh_prims.data[payload + k];
                if (p.x == 0u) {
                    if (hit_sphere(int(p.y), ro, rd, t_min, t_max, hit)) found = true;
                } else {
                    if (hit_triangle(int(p.y), ro, rd, t_min, t_max, hit)) found = true;
                }
            }
        } else {
            uint left = node_idx + 1u;
            uint right = payload;
            if (sp < 30) {
                stack[sp] = right;
                sp = sp + 1;
                stack[sp] = left;
                sp = sp + 1;
            }
        }
    }
    return found;
}

bool occluded(vec3 ro, vec3 rd, float t_max) {
    vec3 inv_dir = 1.0 / rd;
    uint stack[32];
    int sp = 0;
    stack[sp] = 0u;
    sp = 1;

    while (sp > 0) {
        sp = sp - 1;
        uint node_idx = stack[sp];
        BvhNode n = bvh_nodes.data[node_idx];
        if (!aabb_hit(n.min_leafcount.xyz, n.max_rightorfirst.xyz, ro, inv_dir, t_max)) continue;

        uint leaf_count = floatBitsToUint(n.min_leafcount.w);
        uint payload = floatBitsToUint(n.max_rightorfirst.w);
        if (leaf_count > 0u) {
            Hit dummy;
            dummy.t = t_max;
            dummy.hit_kind = 0;
            for (uint k = 0u; k < leaf_count; ++k) {
                uvec4 p = bvh_prims.data[payload + k];
                if (p.x == 0u) {
                    if (hit_sphere(int(p.y), ro, rd, 1e-3, t_max, dummy)) return true;
                } else {
                    if (hit_triangle(int(p.y), ro, rd, 1e-3, t_max, dummy)) return true;
                }
            }
        } else {
            uint left = node_idx + 1u;
            uint right = payload;
            if (sp < 30) {
                stack[sp] = right;
                sp = sp + 1;
                stack[sp] = left;
                sp = sp + 1;
            }
        }
    }
    return false;
}

float cloud_hash13(vec3 p) {
    p = fract(p * 0.1031);
    p += dot(p, p.yzx + 19.19);
    return fract((p.x + p.y) * p.z);
}

float cloud_value_noise(vec3 p) {
    vec3 i = floor(p);
    vec3 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float n000 = cloud_hash13(i + vec3(0.0, 0.0, 0.0));
    float n100 = cloud_hash13(i + vec3(1.0, 0.0, 0.0));
    float n010 = cloud_hash13(i + vec3(0.0, 1.0, 0.0));
    float n110 = cloud_hash13(i + vec3(1.0, 1.0, 0.0));
    float n001 = cloud_hash13(i + vec3(0.0, 0.0, 1.0));
    float n101 = cloud_hash13(i + vec3(1.0, 0.0, 1.0));
    float n011 = cloud_hash13(i + vec3(0.0, 1.0, 1.0));
    float n111 = cloud_hash13(i + vec3(1.0, 1.0, 1.0));
    float nx00 = mix(n000, n100, f.x);
    float nx10 = mix(n010, n110, f.x);
    float nx01 = mix(n001, n101, f.x);
    float nx11 = mix(n011, n111, f.x);
    float nxy0 = mix(nx00, nx10, f.y);
    float nxy1 = mix(nx01, nx11, f.y);
    return mix(nxy0, nxy1, f.z);
}

float cloud_fbm(vec3 p) {
    float a = 0.0;
    float w = 0.5;
    for (int k = 0; k < 4; ++k) {
        a += w * cloud_value_noise(p);
        p *= 2.0;
        w *= 0.5;
    }
    return a;
}

vec3 sky_color(vec3 dir) {
    vec3 d = normalize(dir);
    float t = 0.5 * (d.y + 1.0);
    vec3 base = mix(frame_buf.frame.sky_bottom_w.xyz, frame_buf.frame.sky_top_w.xyz, clamp(t, 0.0, 1.0));

    float mie_g = frame_buf.frame.atmo_clouds.x;
    if (mie_g > 0.001) {
        vec3 sun_dir = normalize(-frame_buf.frame.sun_dir_intensity.xyz);
        float cos_th = clamp(dot(d, sun_dir), -1.0, 1.0);
        float ray_phase = (3.0 / 16.0) * (1.0 + cos_th * cos_th) / 3.14159265;
        float g2 = mie_g * mie_g;
        float denom = pow(max(1.0 + g2 - 2.0 * mie_g * cos_th, 0.0001), 1.5);
        float mie_phase = (3.0 / 8.0) * ((1.0 - g2) * (1.0 + cos_th * cos_th)) / ((2.0 + g2) * denom) / 3.14159265;
        vec3 sun_col = frame_buf.frame.sun_color_w.xyz * frame_buf.frame.sun_dir_intensity.w;
        base += sun_col * (ray_phase * 0.06 + mie_phase * 0.04) * max(d.y, 0.0);
    }

    float coverage = frame_buf.frame.atmo_clouds.y;
    if (coverage > 0.001 && d.y > 0.05) {
        float density_param = frame_buf.frame.atmo_clouds.z;
        float alt_norm = frame_buf.frame.atmo_clouds.w;
        float t_to_layer = alt_norm / max(d.y, 0.05);
        vec3 layer_p = d * t_to_layer * 12.0;
        float n = cloud_fbm(layer_p * 0.3);
        float shape = max(n - (1.0 - coverage), 0.0) / max(coverage, 0.01);
        float density = shape * density_param;
        vec3 sun_dir = normalize(-frame_buf.frame.sun_dir_intensity.xyz);
        vec3 sun_col = frame_buf.frame.sun_color_w.xyz * frame_buf.frame.sun_dir_intensity.w;
        float beer = exp(-density * 2.5);
        float powder = 1.0 - exp(-density * 5.0);
        vec3 cloud = (sun_col * beer * max(sun_dir.y * -1.0, 0.0) + base * 0.35) * powder;
        float alpha = clamp(density * 0.85, 0.0, 0.92);
        base = mix(base, cloud, alpha);
    }
    return base;
}

vec3 fresnel_schlick(float cos_theta, vec3 f0) {
    float m = clamp(1.0 - cos_theta, 0.0, 1.0);
    float m2 = m * m;
    return f0 + (vec3(1.0) - f0) * (m2 * m2 * m);
}

vec3 sample_area_lights(vec3 point, vec3 normal, vec3 albedo, float metallic, vec3 view_dir, inout uint state) {
    uint count = frame_buf.frame.sample_seed_bounces.w;
    if (count == 0u) return vec3(0.0);
    vec3 acc = vec3(0.0);
    for (uint i = 0u; i < count; ++i) {
        AreaLight L = area_lights.data[i];
        float u = rand_float(state);
        float v = rand_float(state);
        vec3 sample_pt = L.position_pad.xyz + L.u_pad.xyz * (u - 0.5) + L.v_pad.xyz * (v - 0.5);
        vec3 to_light = sample_pt - point;
        float dist2 = dot(to_light, to_light);
        float dist = sqrt(max(dist2, 1e-6));
        vec3 dir = to_light / dist;
        float ndotl = max(dot(normal, dir), 0.0);
        if (ndotl <= 0.0) continue;
        if (occluded(point + normal * 1e-3, dir, dist - 1e-3)) continue;
        vec3 f0 = mix(vec3(0.04), albedo, metallic);
        vec3 h = normalize(dir + view_dir);
        vec3 f = fresnel_schlick(max(dot(h, dir), 0.0), f0);
        vec3 brdf = (vec3(1.0) - f) * (1.0 - metallic) * albedo / 3.14159265;
        vec3 light_color = L.color_intensity.xyz * L.color_intensity.w;
        float falloff = 1.0 / max(dist2, 1e-2);
        acc += light_color * brdf * ndotl * falloff;
    }
    return acc;
}

vec3 trace_path(vec3 ro, vec3 rd, inout uint state) {
    vec3 throughput = vec3(1.0);
    vec3 radiance = vec3(0.0);
    uint max_bounces = frame_buf.frame.sample_seed_bounces.z;
    for (uint depth = 0u; depth <= max_bounces; ++depth) {
        Hit hit;
        if (!intersect_scene(ro, rd, 1e-3, 1e6, hit)) {
            radiance += throughput * sky_color(rd);
            break;
        }
        radiance += throughput * hit.emission;

        vec3 view_dir = -rd;
        vec3 sun_dir = normalize(-frame_buf.frame.sun_dir_intensity.xyz);
        float sun_intensity = frame_buf.frame.sun_dir_intensity.w;
        vec3 sun_color = frame_buf.frame.sun_color_w.xyz * sun_intensity;
        float ndotl = max(dot(hit.normal, sun_dir), 0.0);
        if (ndotl > 0.0 && !occluded(hit.point + hit.normal * 1e-3, sun_dir, 1e6)) {
            vec3 f0 = mix(vec3(0.04), hit.albedo, hit.metallic);
            vec3 h = normalize(sun_dir + view_dir);
            vec3 f = fresnel_schlick(max(dot(h, sun_dir), 0.0), f0);
            vec3 diffuse = (vec3(1.0) - f) * (1.0 - hit.metallic) * hit.albedo / 3.14159265;
            float spec_power = mix(2.0, 256.0, 1.0 - hit.roughness);
            vec3 spec = f * pow(max(dot(hit.normal, h), 0.0), spec_power) * (spec_power + 8.0) / (8.0 * 3.14159265);
            radiance += throughput * sun_color * ndotl * (diffuse + spec);
        }

        radiance += throughput * sample_area_lights(hit.point, hit.normal, hit.albedo, hit.metallic, view_dir, state);

        if (hit.transmission > 0.5) {
            float eta = hit.ior > 1.0 ? 1.0 / hit.ior : hit.ior;
            vec3 n = hit.normal;
            float cos_i = -dot(rd, n);
            if (cos_i < 0.0) {
                n = -n;
                cos_i = -cos_i;
                eta = hit.ior;
            }
            float k = 1.0 - eta * eta * (1.0 - cos_i * cos_i);
            if (k >= 0.0) {
                vec3 refr = normalize(eta * rd + (eta * cos_i - sqrt(k)) * n);
                ro = hit.point - n * 1e-3;
                rd = refr;
                throughput *= hit.albedo;
                continue;
            }
        }

        vec3 next = sample_hemisphere(hit.normal, state);
        vec3 f0 = mix(vec3(0.04), hit.albedo, hit.metallic);
        if (hit.metallic > 0.5 || hit.roughness < 0.2) {
            vec3 reflected = reflect(rd, hit.normal);
            float jitter = hit.roughness * 0.5;
            next = normalize(mix(reflected, next, jitter));
            throughput *= f0;
        } else {
            throughput *= hit.albedo;
        }
        ro = hit.point + hit.normal * 1e-3;
        rd = next;
        float p = max(throughput.x, max(throughput.y, throughput.z));
        if (depth >= 1u) {
            if (rand_float(state) > p) break;
            if (p > 0.0) throughput /= max(p, 1e-3);
        }
    }
    return radiance;
}

vec3 aces_film(vec3 x) {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0);
}

void main() {
    uvec2 pix = gl_GlobalInvocationID.xy;
    uint width = frame_buf.frame.image_size_counts.x;
    uint height = frame_buf.frame.image_size_counts.y;
    if (pix.x >= width || pix.y >= height) return;

    uint samples = frame_buf.frame.sample_seed_bounces.x;
    uint base_seed = frame_buf.frame.sample_seed_bounces.y;
    uint state = pcg_hash(base_seed ^ (pix.x * 1973u + pix.y * 9277u + 26699u));

    vec3 accum = vec3(0.0);
    for (uint s = 0u; s < samples; ++s) {
        float jitter_x = rand_float(state);
        float jitter_y = rand_float(state);
        float u = (float(pix.x) + jitter_x) / float(width);
        float v = (float(pix.y) + jitter_y) / float(height);
        vec3 ro = frame_buf.frame.cam_origin_w.xyz;
        vec3 dir = frame_buf.frame.cam_lower_left_w.xyz
            + frame_buf.frame.cam_horizontal_w.xyz * u
            + frame_buf.frame.cam_vertical_w.xyz * v
            - ro;
        vec3 rd = normalize(dir);
        accum += trace_path(ro, rd, state);
    }
    accum /= max(float(samples), 1.0);

    float exposure = frame_buf.frame.exposure_pad.x;
    vec3 mapped = aces_film(accum * exposure);

    uint flat_idx = pix.y * width + pix.x;
    out_buf.pixels[flat_idx] = vec4(mapped, 1.0);
}
"#;

/// Returns a complete shader source string for the requested GLSL flavour.
pub fn assemble(is_es: bool) -> String {
    let header = if is_es {
        "#version 310 es\nprecision highp float;\nprecision highp int;\n"
    } else {
        "#version 430 core\n"
    };
    let mut src = String::with_capacity(header.len() + SHADER_BODY.len() + 320);
    src.push_str(header);
    src.push_str(&format!("#define OUTPUT_BINDING {}\n", bindings::OUTPUT));
    src.push_str(&format!("#define FRAME_BINDING {}\n", bindings::FRAME));
    src.push_str(&format!("#define SPHERE_BINDING {}\n", bindings::SPHERES));
    src.push_str(&format!(
        "#define TRIANGLE_BINDING {}\n",
        bindings::TRIANGLES
    ));
    src.push_str(&format!(
        "#define AREA_LIGHT_BINDING {}\n",
        bindings::AREA_LIGHTS
    ));
    src.push_str(&format!(
        "#define BVH_NODES_BINDING {}\n",
        bindings::BVH_NODES
    ));
    src.push_str(&format!(
        "#define BVH_PRIMS_BINDING {}\n",
        bindings::BVH_PRIMS
    ));
    src.push_str(SHADER_BODY);
    src
}
