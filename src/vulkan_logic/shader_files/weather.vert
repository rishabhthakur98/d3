#version 450

struct WeatherParticleData {
    vec4 position; // w = random seed
};

layout(set = 0, binding = 0) uniform WeatherUBO {
    mat4 view_proj;
    vec4 camera_pos;     
    vec4 camera_right;   
    vec4 camera_up;      
    vec4 color;          
    vec4 params;         // x: Scale, y: Fall Speed, z: Is Rain, w: Time
    vec4 wind;           
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
    
    if (ubo.params.z < 0.5) {
        float flutter = sin(ubo.params.w * 2.0 + p.position.w) * 0.5;
        localPos.x += flutter;
    }

    vec3 worldPos = ubo.camera_pos.xyz + localPos;

    if (ubo.params.z > 0.5) {
        worldPos += ubo.camera_right.xyz * v.x * ubo.params.x;
        worldPos += vec3(0.0, 1.0, 0.0) * v.y * ubo.params.x * (ubo.params.y * 0.25);
    } else {
        worldPos += ubo.camera_right.xyz * v.x * ubo.params.x;
        worldPos += ubo.camera_up.xyz * v.y * ubo.params.x;
    }

    gl_Position = ubo.view_proj * vec4(worldPos, 1.0);
    
    fragUV = v + 0.5;
    fragColor = ubo.color;
    fragIsRain = ubo.params.z;
}