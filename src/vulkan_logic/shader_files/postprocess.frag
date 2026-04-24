// src/vulkan_logic/shader_files/postprocess.frag
#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 0) out vec4 outColor;

layout(std430, binding = 0) buffer PostProcessSSBO {
    float enabled;
    float exposure;
    float gamma;
    float contrast;
    float saturation;
    float vignette_strength;
    float chromatic_aberration;
    float film_grain;
    float time;
    float _pad[3]; 
} pp;

layout(binding = 1) uniform sampler2D sceneColor;

void main() {
    if (pp.enabled < 0.5) {
        outColor = texture(sceneColor, fragUV);
        return;
    }

    vec2 uv = fragUV;
    
    // Chromatic Aberration
    vec2 offset = vec2(pp.chromatic_aberration * 0.005, 0.0);
    float r = texture(sceneColor, uv - offset).r;
    float g = texture(sceneColor, uv).g;
    float b = texture(sceneColor, uv + offset).b;
    vec3 color = vec3(r, g, b);
    
    // Exposure & Gamma mapping
    color = color * pp.exposure;
    color = pow(color, vec3(1.0 / pp.gamma));
    
    // Contrast
    color = (color - 0.5) * pp.contrast + 0.5;
    
    // Saturation
    float luma = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(luma), color, pp.saturation);
    
    // Vignette
    float dist = distance(uv, vec2(0.5));
    color *= smoothstep(0.8, pp.vignette_strength * 0.79, dist * (pp.vignette_strength + 0.5));

    // Film Grain (Simple pseudo-random noise)
    float noise = fract(sin(dot(uv + pp.time, vec2(12.9898, 78.233))) * 43758.5453);
    color -= noise * pp.film_grain * 0.1;

    outColor = vec4(color, 1.0);
}