// src/vulkan_logic/shader_files/postprocess.frag
#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform PostProcessUBO {
    float enabled;
    float exposure;
    float gamma;
    float contrast;
    float saturation;
    float vignette_strength;
    float chromatic_aberration;
    float film_grain;
    float time;
} ubo;

layout(set = 0, binding = 1) uniform sampler2D screenTexture;

vec3 ACESFilm(vec3 x) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((x*(a*x+b))/(x*(c*x+d)+e), 0.0, 1.0);
}

float random(vec2 uv) {
    return fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    if (ubo.enabled < 0.5) {
        vec4 raw = texture(screenTexture, fragUV);
        outColor = vec4(pow(max(raw.rgb, vec3(0.0)), vec3(1.0/ubo.gamma)), raw.a);
        return;
    }

    vec2 offset = (fragUV - 0.5) * ubo.chromatic_aberration * 0.01;
    float r = texture(screenTexture, fragUV - offset).r;
    float g = texture(screenTexture, fragUV).g;
    float b = texture(screenTexture, fragUV + offset).b;
    vec3 color = vec3(r, g, b);

    color *= ubo.exposure;
    color = ACESFilm(color);
    color = (color - 0.5) * max(ubo.contrast, 0.0) + 0.5;

    vec3 luma = vec3(dot(color, vec3(0.299, 0.587, 0.114)));
    color = mix(luma, color, ubo.saturation);

    float dist = distance(fragUV, vec2(0.5));
    float vignette = smoothstep(0.8, ubo.vignette_strength * 0.2, dist);
    color *= vignette;

    float noise = (random(fragUV + ubo.time) - 0.5) * ubo.film_grain;
    color += noise;

    color = pow(max(color, vec3(0.0)), vec3(1.0 / ubo.gamma));

    outColor = vec4(color, 1.0);
}