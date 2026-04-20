#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec2 inUV;

layout(push_constant) uniform PushConstants {
    mat4 view_proj; 
    mat4 model;     
    mat4 light_space_matrix; // NEW: The projection matrix generated strictly by the Spotlight logic
} pc;

layout(location = 0) out vec4 fragColor;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out vec3 fragNormal;
layout(location = 3) out vec3 fragWorldPos;
layout(location = 4) out vec4 fragLightSpacePos; // NEW: Send pixel location in light space to fragment shader

void main() {
    vec4 worldPos = pc.model * vec4(inPosition, 1.0);
    gl_Position = pc.view_proj * worldPos;
    
    fragColor = inColor;
    fragUV = inUV;
    
    fragNormal = normalize(mat3(pc.model) * inNormal);
    fragWorldPos = worldPos.xyz;
    
    // Convert this exact vertex location into the light's perspective for Shadow Filtering
    fragLightSpacePos = pc.light_space_matrix * worldPos;
}