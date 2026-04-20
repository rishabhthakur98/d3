#version 450

// We ONLY need the position!
layout(location = 0) in vec3 inPosition;

layout(push_constant) uniform PushConstants {
    mat4 light_view_proj; // Where the spotlight is looking
    mat4 model;           // Where the object is
} pc;

void main() {
    gl_Position = pc.light_view_proj * pc.model * vec4(inPosition, 1.0);
}