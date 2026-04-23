// src/vulkan_logic/shader_files/weather.vert
#version 450

struct WeatherParticleData {
    vec4 position; // w = random seed
    vec4 color;
    vec4 params;   // x: scale, y: fall_speed, z: is_rain, w: time
};

layout(set = 0, binding = 0) uniform WeatherUBO {
    mat4 view_proj;
    vec4 camera_pos;     
    vec4 camera_right;   
    vec4 camera_up;
    uint particle_count;
    uint pad1, pad2, pad3;
    WeatherParticleData particles[3000]; 
} ubo;

layout(location = 0) out vec2 fragUV;
layout(location = 1) out vec4 fragColor;
layout(location = 2) out float fragIsRain;

const vec2 quad[6] = vec2[](
    vec2(-0.5, -0.5), vec2(0.5, -0.5), vec2(0.5, 0.5),
    vec2(-0.5, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5)
);

void main() {
    WeatherParticleData p = ubo.particles[gl_InstanceIndex];
    vec2 v = quad[gl_VertexIndex];

    vec3 localPos = p.position.xyz;
    if (p.params.z < 0.5) { // Is Snow, apply flutter
        float flutter = sin(p.params.w * 2.0 + p.position.w) * 0.5;
        localPos.x += flutter;
    }

    vec3 worldPos = ubo.camera_pos.xyz + localPos;
    if (p.params.z > 0.5) { // Is Rain, stretch drop by velocity
        worldPos += ubo.camera_right.xyz * v.x * p.params.x;
        worldPos += vec3(0.0, 1.0, 0.0) * v.y * p.params.x * (p.params.y * 0.25);
    } else { // Snow, standard billboard
        worldPos += ubo.camera_right.xyz * v.x * p.params.x;
        worldPos += ubo.camera_up.xyz * v.y * p.params.x;
    }

    gl_Position = ubo.view_proj * vec4(worldPos, 1.0);
    fragUV = v + 0.5;
    fragColor = p.color;
    fragIsRain = p.params.z;
}