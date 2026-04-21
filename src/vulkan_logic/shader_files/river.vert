#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inUV;

layout(set = 0, binding = 0) uniform RiverUBO {
    mat4 view_proj;
    vec4 camera_pos;
    vec4 light_dir;
    vec4 light_color;
    vec4 deep_color;
    vec4 shallow_color;
    vec4 foam_color;
    vec4 params; 
    vec4 advanced_params1; 
    vec4 sky_reflection_color; 
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pc;

layout(location = 0) out vec3 fragWorldPos;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out float fragFoamMask;
layout(location = 3) out vec3 fragNormal; 

vec3 GerstnerWave(vec2 d, float w, float s, vec3 p, float time) {
    float k = 2.0 * 3.14159 / w;
    float c = sqrt(9.8 / k);
    vec2 dir = normalize(d);
    float f = k * (dot(dir, p.xz) - c * time);
    float a = s / k;
    return vec3(dir.x * (a * cos(f)), a * sin(f), dir.y * (a * cos(f)));
}

vec3 GetDisplacedPosition(vec3 p, float time, float strength) {
    vec3 w1 = GerstnerWave(vec2(0.1, 1.0), 15.0, 0.25 * strength, p, time);
    vec3 w2 = GerstnerWave(vec2(-0.2, 1.0), 8.0, 0.15 * strength, p, time * 1.2);
    vec3 w3 = GerstnerWave(vec2(0.5, 0.5), 4.0, 0.1 * strength, p, time * 0.8);
    return p + w1 + w2 + w3;
}

void main() {
    float time = ubo.params.x * ubo.params.y;
    float strength = ubo.params.z;
    
    // Load config parameters safely sent from Rust
    float sample_dist = ubo.advanced_params1.z; 
    float scroll_speed = ubo.advanced_params1.w;
    
    vec3 displacedPos = GetDisplacedPosition(inPosition, time, strength);
    
    // Computes normals dynamically using the configurable sample distance
    vec3 p_right = GetDisplacedPosition(inPosition + vec3(sample_dist, 0.0, 0.0), time, strength);
    vec3 p_forward = GetDisplacedPosition(inPosition + vec3(0.0, 0.0, sample_dist), time, strength);
    vec3 localNormal = normalize(cross(p_forward - displacedPos, p_right - displacedPos));

    vec4 worldPos = pc.model * vec4(displacedPos, 1.0);
    gl_Position = ubo.view_proj * worldPos;
    
    fragWorldPos = worldPos.xyz;
    fragNormal = normalize(mat3(pc.model) * localNormal);
    
    // Utilize the configurable scroll speed
    fragUV = vec2(inUV.x, inUV.y - (time * scroll_speed));
    fragFoamMask = clamp(displacedPos.y * 2.0, 0.0, 1.0);
}