#version 450

layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in float fragFoamMask;
layout(location = 3) in vec3 fragNormal;

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
    vec4 advanced_params1; 
    vec4 sky_reflection_color; 
} ubo;

void main() {
    // Load config parameters safely sent from Rust
    float spec_exp = ubo.advanced_params1.x;
    float fresnel_pow = ubo.advanced_params1.y;
    vec3 sky_color = ubo.sky_reflection_color.xyz;
    float foam_blend = ubo.sky_reflection_color.w;

    vec3 normal = normalize(fragNormal);

    float bank_fade = smoothstep(0.0, 0.15, fragUV.x) * smoothstep(1.0, 0.85, fragUV.x);
    vec3 albedo = mix(ubo.shallow_color.xyz, ubo.deep_color.xyz, bank_fade);
    
    // Utilize the configurable foam opacity
    albedo = mix(albedo, ubo.foam_color.xyz, fragFoamMask * foam_blend);

    vec3 viewDir = normalize(ubo.camera_pos.xyz - fragWorldPos);
    vec3 lightDir = normalize(-ubo.light_dir.xyz);
    
    // Utilize configurable fresnel power and sky reflection colors
    float NdotV = max(dot(viewDir, normal), 0.0);
    float fresnel = pow(1.0 - NdotV, fresnel_pow); 
    vec3 reflection = mix(albedo, sky_color, fresnel * 0.85); 

    // Utilize configurable specular exponent
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float NdotH = max(dot(normal, halfwayDir), 0.0);
    float spec = pow(NdotH, spec_exp); 
    vec3 specular = ubo.light_color.xyz * spec * ubo.light_dir.w;

    float final_alpha = mix(0.0, ubo.deep_color.w, bank_fade);
    outColor = vec4(reflection + specular, final_alpha);
}