#version 450

layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in float fragFoamMask;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform RiverUBO {
    mat4 view_proj;
    vec4 camera_pos;
    vec4 light_dir;
    vec4 light_color;
    vec4 deep_color;
    vec4 shallow_color;
    vec4 foam_color;
    vec4 params; 
} ubo;

void main() {
    // Procedural accurate normals from the displaced Gerstner waves
    vec3 dx = dFdx(fragWorldPos);
    vec3 dy = dFdy(fragWorldPos);
    vec3 normal = normalize(cross(dx, dy));
    if (length(normal) < 0.1 || isnan(normal.x)) { normal = vec3(0.0, 1.0, 0.0); }

    // 1. Shoreline Blending using UV.x (0.0 to 1.0 across the width)
    // Smoothly fades alpha from 0 at the banks to 1 in the middle
    float bank_fade = smoothstep(0.0, 0.15, fragUV.x) * smoothstep(1.0, 0.85, fragUV.x);
    
    // The edges (shallow) get the shallow color. The middle gets the deep color.
    vec3 albedo = mix(ubo.shallow_color.xyz, ubo.deep_color.xyz, bank_fade);
    
    // Add white foam on the crests of the waves
    albedo = mix(albedo, ubo.foam_color.xyz, fragFoamMask * 0.8);

    // 2. Physics Shading (Fresnel & Specular)
    vec3 viewDir = normalize(ubo.camera_pos.xyz - fragWorldPos);
    vec3 lightDir = normalize(-ubo.light_dir.xyz);
    
    float fresnel = pow(clamp(1.0 - dot(viewDir, normal), 0.0, 1.0), 4.0);
    vec3 reflection = mix(albedo, ubo.light_color.xyz * 0.9, fresnel);

    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 150.0);
    vec3 specular = ubo.light_color.xyz * spec * ubo.light_dir.w;

    // Apply the gradient alpha fade to the final output
    float final_alpha = mix(0.0, ubo.deep_color.w, bank_fade);

    outColor = vec4(reflection + specular, final_alpha);
}