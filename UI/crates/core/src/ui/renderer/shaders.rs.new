pub const VERTEX_SHADER: &str = "#version 330 core
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_color;
layout(location = 3) in float a_mode;
layout(location = 4) in vec2 a_local;
layout(location = 5) in vec4 a_params;
out vec2 v_uv;
out vec4 v_color;
flat out float v_mode;
out vec2 v_local;
flat out vec4 v_params;
uniform vec2 u_viewport;
void main() {
    vec2 ndc = vec2((a_pos.x / u_viewport.x) * 2.0 - 1.0,
                    1.0 - (a_pos.y / u_viewport.y) * 2.0);
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_uv = a_uv;
    v_color = a_color;
    v_mode = a_mode;
    v_local = a_local;
    v_params = a_params;
}
";

pub const FRAGMENT_SHADER: &str = "#version 330 core
in vec2 v_uv;
in vec4 v_color;
flat in float v_mode;
in vec2 v_local;
flat in vec4 v_params;
out vec4 frag_color;
uniform sampler2D u_tex;

float sd_round_box(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

void main() {
    int mode = int(v_mode + 0.5);
    if (mode == 0) {
        frag_color = v_color;
    } else if (mode == 1) {
        float a = texture(u_tex, v_uv).r;
        if (a < 0.04) discard;
        frag_color = vec4(v_color.rgb, v_color.a * a);
    } else if (mode == 2) {
        vec4 sampled = texture(u_tex, v_uv);
        frag_color = vec4(sampled.rgb * v_color.rgb, sampled.a * v_color.a);
    } else if (mode == 3) {
        vec2 half_extent = v_params.xy;
        float radius = v_params.z;
        float softness = max(v_params.w, 0.5);
        float d = sd_round_box(v_local, half_extent, radius);
        float alpha = 1.0 - smoothstep(-softness, softness, d);
        if (alpha <= 0.001) discard;
        frag_color = vec4(v_color.rgb, v_color.a * alpha);
    } else if (mode == 4) {
        vec2 half_extent = v_params.xy;
        float radius = v_params.z;
        float thickness = max(v_params.w, 1.0);
        float d = sd_round_box(v_local, half_extent, radius);
        float aa = 1.0;
        float outline = 1.0 - smoothstep(thickness - aa, thickness + aa, abs(d));
        if (outline <= 0.001) discard;
        frag_color = vec4(v_color.rgb, v_color.a * outline);
    } else if (mode == 5) {
        vec2 half_extent = v_params.xy;
        float radius = v_params.z;
        float spread = max(v_params.w, 1.0);
        float d = sd_round_box(v_local, half_extent, radius);
        float t = clamp(d / spread, 0.0, 1.0);
        float falloff = (1.0 - t) * (1.0 - t);
        if (falloff <= 0.002) discard;
        frag_color = vec4(v_color.rgb, v_color.a * falloff);
    } else if (mode == 6) {
        vec2 half_extent = v_params.xy;
        float radius = v_params.z;
        float softness = max(v_params.w, 0.5);
        float d = sd_round_box(v_local, half_extent, radius);
        float alpha = 1.0 - smoothstep(-softness, softness, d);
        if (alpha <= 0.001) discard;
        float t = clamp((v_local.y + half_extent.y) / (2.0 * half_extent.y), 0.0, 1.0);
        vec3 rgb = mix(v_color.rgb, vec3(v_uv.x, v_uv.y, v_color.a), 0.0);
        vec3 top = v_color.rgb;
        vec3 bot = v_color.rgb * (1.0 - 0.18) + vec3(0.0);
        vec3 grad = mix(top, bot, t);
        frag_color = vec4(grad, v_color.a * alpha);
    } else {
        frag_color = v_color;
    }
}
";
