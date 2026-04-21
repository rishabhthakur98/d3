#version 450

// We generate the UVs procedurally to create a massive triangle covering the whole screen
layout(location = 0) out vec3 fragRayDir;

layout(push_constant) uniform PushConstants {
    mat4 inv_view_proj;
} pc;

void main() {
    vec2 uvs[3] = vec2[](vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    vec2 uv = uvs[gl_VertexIndex];
    
    // Z is set to 1.0, so the sky renders far behind everything.
    gl_Position = vec4(uv, 1.0, 1.0);
    
    // Un-project the screen coordinate back into world direction space
    vec4 target = pc.inv_view_proj * gl_Position;
    fragRayDir = target.xyz / target.w;
}