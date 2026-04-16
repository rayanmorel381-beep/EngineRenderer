use crate::core::engine::math::{Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct PbrMaterial {
    pub base_color: Vec4,
    pub metallic: f32,
    pub roughness: f32,
    pub normal_map: Option<String>,
    pub metallic_map: Option<String>,
    pub roughness_map: Option<String>,
    pub ambient_occlusion: f32,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        PbrMaterial {
            base_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            normal_map: None,
            metallic_map: None,
            roughness_map: None,
            ambient_occlusion: 1.0,
        }
    }
}

pub trait Material {
    fn base_color(&self) -> Vec4;
    fn metallic(&self) -> f32;
    fn roughness(&self) -> f32;
    fn normal_map(&self) -> Option<&str>;
}

impl Material for PbrMaterial {
    fn base_color(&self) -> Vec4 {
        self.base_color
    }

    fn metallic(&self) -> f32 {
        self.metallic
    }

    fn roughness(&self) -> f32 {
        self.roughness
    }

    fn normal_map(&self) -> Option<&str> {
        self.normal_map.as_deref()
    }
}

pub mod pbr_shader {
    pub const VERTEX_SHADER: &str = r#"
        #version 300 es
        precision highp float;

        layout(location = 0) in vec3 position;
        layout(location = 1) in vec3 normal;
        layout(location = 2) in vec2 texcoord;

        uniform mat4 model;
        uniform mat4 view;
        uniform mat4 projection;

        out vec3 v_position;
        out vec3 v_normal;
        out vec2 v_texcoord;

        void main() {
            v_position = vec3(model * vec4(position, 1.0));
            v_normal = normalize(mat3(model) * normal);
            v_texcoord = texcoord;
            gl_Position = projection * view * vec4(v_position, 1.0);
        }
    "#;

    pub const FRAGMENT_SHADER: &str = r#"
        #version 300 es
        precision highp float;

        in vec3 v_position;
        in vec3 v_normal;
        in vec2 v_texcoord;

        uniform vec4 base_color;
        uniform float metallic;
        uniform float roughness;
        uniform sampler2D normal_map;
        uniform vec3 light_pos;
        uniform vec3 view_pos;

        out vec4 out_color;

        const float PI = 3.14159265359;

        vec3 fresnelSchlick(float cosTheta, vec3 F0) {
            return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
        }

        float DistributionGGX(vec3 N, vec3 H, float roughness) {
            float a = roughness * roughness;
            float a2 = a * a;
            float NdotH = max(dot(N, H), 0.0);
            float NdotH2 = NdotH * NdotH;

            float nom = a2;
            float denom = (NdotH2 * (a2 - 1.0) + 1.0);
            denom = PI * denom * denom;

            return nom / denom;
        }

        float GeometrySchlickGGX(float NdotV, float roughness) {
            float r = (roughness + 1.0);
            float k = (r * r) / 8.0;

            float nom = NdotV;
            float denom = NdotV * (1.0 - k) + k;

            return nom / denom;
        }

        float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
            float NdotV = max(dot(N, V), 0.0);
            float NdotL = max(dot(N, L), 0.0);
            float ggx2 = GeometrySchlickGGX(NdotV, roughness);
            float ggx1 = GeometrySchlickGGX(NdotL, roughness);

            return ggx1 * ggx2;
        }

        void main() {
            vec3 N = normalize(v_normal);
            vec3 V = normalize(view_pos - v_position);
            vec3 L = normalize(light_pos - v_position);
            vec3 H = normalize(V + L);

            float distance = length(light_pos - v_position);
            float attenuation = 1.0 / (distance * distance);
            vec3 radiance = vec3(1.0) * attenuation;

            vec3 F0 = vec3(0.04);
            F0 = mix(F0, base_color.rgb, metallic);
            vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);

            vec3 kS = F;
            vec3 kD = vec3(1.0) - kS;
            kD *= 1.0 - metallic;

            float NDF = DistributionGGX(N, H, roughness);
            float G = GeometrySmith(N, V, L, roughness);
            vec3 numerator = NDF * G * F;
            float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
            vec3 specular = numerator / denominator;

            float NdotL = max(dot(N, L), 0.0);
            vec3 Lo = (kD * base_color.rgb / PI + specular) * radiance * NdotL;

            vec3 ambient = vec3(0.03) * base_color.rgb;
            vec3 color = ambient + Lo;

            color = color / (color + vec3(1.0));
            color = pow(color, vec3(1.0 / 2.2));

            out_color = vec4(color, base_color.a);
        }
    "#;
}
