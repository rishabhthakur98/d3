#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 1) in vec4 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    // Procedurally turn the square quad into a soft, blurry sphere
    vec2 center = fragUV - 0.5;
    float dist = length(center);
    
    // Smoothstep creates a perfect gradient from the center to the edge
    float shape = smoothstep(0.5, 0.1, dist);
    
    outColor = vec4(fragColor.rgb, fragColor.a * shape);
}