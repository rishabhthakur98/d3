#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 1) in vec4 fragColor;
layout(location = 2) in float fragIsRain;

layout(location = 0) out vec4 outColor;

void main() {
    vec2 center = fragUV - 0.5;
    
    if (fragIsRain > 0.5) {
        float streak = 1.0 - abs(center.x * 4.0); 
        streak *= smoothstep(0.5, 0.0, abs(center.y)); 
        outColor = vec4(fragColor.rgb, fragColor.a * max(0.0, streak));
    } else {
        float dist = length(center);
        float shape = smoothstep(0.5, 0.1, dist);
        outColor = vec4(fragColor.rgb, fragColor.a * shape);
    }
}