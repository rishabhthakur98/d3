#version 450

// --- INPUTS FROM VERTEX SHADER ---
layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragUV;

// --- OUTPUT TO SCREEN ---
layout(location = 0) out vec4 outColor;

void main() {
    outColor = fragColor;
}