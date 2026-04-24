// src/vulkan_logic/shader_files/triangle.frag
#version 450
#extension GL_EXT_nonuniform_qualifier : require

layout(location = 0) in vec3 fragNormal;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragPosWorld;
layout(location = 3) in vec4 fragPosLightSpace;

layout(location = 0) out vec4 outColor;

layout(binding = 1) uniform sampler2D shadowMap;

// Must exactly match the vertex shader
struct GlobalLightData { vec4 direction; vec4 color; };
struct SpotLightData { vec4 position; vec4 direction; vec4 color; vec4 params; };
struct PointLightData { vec4 position; vec4 color; };
struct FogVolumeData { vec4 min_bounds; vec4 max_bounds; vec4 color_density; };

layout(std430, binding = 0) buffer LightSSBO {
    vec4 ambient_color; 
    vec4 camera_pos;    
    
    uint global_count;
    uint spot_count;
    uint point_count;          
    uint fog_count;      
    
    GlobalLightData global_lights[4];   
    SpotLightData spot_lights[16];     
    PointLightData point_lights[32];   
    FogVolumeData fog_volumes[];        
} light_data;

float ShadowCalculation(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords.xy = projCoords.xy * 0.5 + 0.5;
    
    if(projCoords.z > 1.0) return 0.0;
    
    float closestDepth = texture(shadowMap, projCoords.xy).r; 
    float currentDepth = projCoords.z;
    
    float bias = max(0.005 * (1.0 - dot(normal, lightDir)), 0.0005);  
    float shadow = currentDepth - bias > closestDepth  ? 1.0 : 0.0;
    
    return shadow;
}

void main() {
    vec3 norm = normalize(fragNormal);
    vec3 viewDir = normalize(light_data.camera_pos.xyz - fragPosWorld);
    
    vec3 result = light_data.ambient_color.rgb * light_data.ambient_color.a;

    // Base color hack for now
    vec3 albedo = vec3(0.8, 0.8, 0.8); 
    
    // Global Lights
    for (uint i = 0; i < light_data.global_count; i++) {
        vec3 lightDir = normalize(-light_data.global_lights[i].direction.xyz);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = diff * light_data.global_lights[i].color.rgb * light_data.global_lights[i].direction.w;
        
        float shadow = 0.0;
        if (light_data.global_lights[i].color.w > 0.5) { // Casts shadows
            shadow = ShadowCalculation(fragPosLightSpace, norm, lightDir);
        }
        
        result += (1.0 - shadow) * diffuse * albedo;
    }
    
    // Point Lights
    for (uint i = 0; i < light_data.point_count; i++) {
        vec3 lightDir = normalize(light_data.point_lights[i].position.xyz - fragPosWorld);
        float diff = max(dot(norm, lightDir), 0.0);
        
        float distance = length(light_data.point_lights[i].position.xyz - fragPosWorld);
        float attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * (distance * distance));    
        // Range check logic could go here based on position.w
        
        vec3 diffuse = diff * light_data.point_lights[i].color.rgb * light_data.point_lights[i].color.w * attenuation;
        result += diffuse * albedo;
    }

    outColor = vec4(result, 1.0);
}