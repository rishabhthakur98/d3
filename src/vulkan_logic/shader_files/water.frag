#version 450

layout(location = 0) in vec3 fragWorldPos;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in float fragHeightRatio;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform WaterUBO {
    mat4 view_proj;
    vec4 camera_pos;
    vec4 light_dir;
    vec4 light_color;
    vec4 base_color;
    vec4 shallow_color;
    vec4 params; 
} ubo;

void main() {
    // 1. Calculate highly accurate normals based on the actual distorted geometry
    vec3 dx = dFdx(fragWorldPos);
    vec3 dy = dFdy(fragWorldPos);
    vec3 normal = normalize(cross(dx, dy));

    vec3 viewDir = normalize(ubo.camera_pos.xyz - fragWorldPos);
    vec3 lightDir = normalize(-ubo.light_dir.xyz);

    // 2. Base Color Mixing (Deep vs Shallow)
    vec3 albedo = mix(ubo.base_color.xyz, ubo.shallow_color.xyz, fragHeightRatio);

    // 3. Fresnel Effect (More reflective at grazing angles)
    float fresnel = dot(viewDir, normal);
    fresnel = clamp(1.0 - fresnel, 0.0, 1.0);
    fresnel = pow(fresnel, 3.0);
    
    vec3 reflectionColor = mix(albedo, ubo.light_color.xyz * 0.8, fresnel);

    // 4. Specular Highlight (The shiny sun reflection on the waves)
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 128.0); // High shininess
    vec3 specular = ubo.light_color.xyz * spec * ubo.light_dir.w;

    vec3 finalColor = reflectionColor + specular;

    // Apply global transparency
    outColor = vec4(finalColor, ubo.params.w);
}