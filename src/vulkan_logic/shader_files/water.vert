#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inUV;

layout(set = 0, binding = 0) uniform WaterUBO {
    mat4 view_proj;
    vec4 camera_pos;
    vec4 light_dir;
    vec4 light_color;
    vec4 base_color;
    vec4 shallow_color;
    vec4 params; // x: Time, y: Speed, z: Strength, w: Transparency
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pc;

layout(location = 0) out vec3 fragWorldPos;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out float fragHeightRatio; 

void main() {
    float time = ubo.params.x * ubo.params.y;
    float strength = ubo.params.z;
    
    vec3 pos = inPosition;
    
    // Sum of sine waves to create procedural displacement
    float wave1 = sin(pos.x * 0.5 + time) * cos(pos.z * 0.5 + time) * 0.5;
    float wave2 = sin(pos.x * 1.2 - time * 1.3) * sin(pos.z * 1.8 + time) * 0.25;
    float wave3 = cos(pos.x * 2.5 + time * 0.8) * 0.15;
    
    float total_wave = (wave1 + wave2 + wave3) * strength;
    pos.y += total_wave;

    vec4 worldPos = pc.model * vec4(pos, 1.0);
    gl_Position = ubo.view_proj * worldPos;
    
    fragWorldPos = worldPos.xyz;
    fragUV = inUV;
    
    // Used by the fragment shader to color the crests lighter than the troughs
    fragHeightRatio = clamp((total_wave / strength) * 0.5 + 0.5, 0.0, 1.0); 
}