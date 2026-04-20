#version 450

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragNormal;
layout(location = 3) in vec3 fragWorldPos;
layout(location = 4) in vec4 fragLightSpacePos; 

layout(location = 0) out vec4 outColor;

struct GlobalLight {
    vec4 direction; 
    vec4 color;     
};

struct SpotLight {
    vec4 position;  
    vec4 direction; 
    vec4 color;
    vec4 params;    
};

struct PointLight {
    vec4 position; // xyz: Position, w: Range
    vec4 color;    // xyz: Color, w: Intensity
};

layout(set = 0, binding = 0) uniform LightUBO {
    vec4 ambient_color;
    vec4 camera_pos;    
    
    uint global_count;
    uint spot_count;
    uint point_count; // Tracks active point lights
    uint pad1;
    
    GlobalLight global_lights[4];
    SpotLight spot_lights[10];
    PointLight point_lights[10];
} ubo;

layout(set = 0, binding = 1) uniform sampler2D shadowMap; 

float CalculateShadow(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords.xy = projCoords.xy * 0.5 + 0.5;
    
    if (projCoords.z > 1.0 || projCoords.z < 0.0 || 
        projCoords.x > 1.0 || projCoords.x < 0.0 || 
        projCoords.y > 1.0 || projCoords.y < 0.0) {
        return 0.0;
    }

    float currentDepth = projCoords.z;
    float bias = max(0.01 * (1.0 - dot(normal, lightDir)), 0.001);
    
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    
    for(int x = -1; x <= 1; ++x) {
        for(int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r; 
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;        
        }    
    }
    shadow /= 9.0;
    
    return shadow;
}

void main() {
    vec3 normal = normalize(fragNormal);
    vec3 ambient = ubo.ambient_color.xyz * ubo.ambient_color.w;
    
    vec3 global_lighting = vec3(0.0);
    
    // 1. Evaluate all Global Directional Lights (Suns/Moons)
    for(uint i = 0; i < ubo.global_count; i++) {
        GlobalLight light = ubo.global_lights[i];
        vec3 sunDir = normalize(-light.direction.xyz); 
        float diff = max(dot(normal, sunDir), 0.0);
        vec3 added_light = light.color.xyz * diff * light.direction.w;
        
        if (light.color.w > 0.5) {
            float shadow = CalculateShadow(fragLightSpacePos, normal, sunDir);
            added_light *= (1.0 - shadow);
        }
        
        global_lighting += added_light;
    }
    
    // 2. Evaluate Local Spotlights
    vec3 spot_lighting = vec3(0.0);
    for(uint i = 0; i < ubo.spot_count; i++) {
        SpotLight light = ubo.spot_lights[i];
        vec3 lightDir = light.position.xyz - fragWorldPos;
        float distance = length(lightDir);
        
        if (distance < light.position.w) { 
            lightDir = normalize(lightDir);
            float diff = max(dot(normal, lightDir), 0.0);
            float attenuation = clamp(1.0 - (distance / light.position.w), 0.0, 1.0);
            attenuation *= attenuation;

            float theta = dot(lightDir, normalize(-light.direction.xyz));
            float epsilon = light.color.w - light.params.x; 
            float intensity = clamp((theta - light.params.x) / epsilon, 0.0, 1.0);
            
            spot_lighting += light.color.xyz * diff * attenuation * intensity * light.direction.w;
        }
    }

    // 3. Evaluate Local Point Lights (NEW)
    vec3 point_lighting = vec3(0.0);
    for(uint i = 0; i < ubo.point_count; i++) {
        PointLight light = ubo.point_lights[i];
        vec3 lightDir = light.position.xyz - fragWorldPos;
        float distance = length(lightDir);
        
        // Is the pixel within the radius of the lightbulb?
        if (distance < light.position.w) { 
            lightDir = normalize(lightDir);
            float diff = max(dot(normal, lightDir), 0.0);
            
            // Quadratic falloff creates realistic smooth fading edges
            float attenuation = clamp(1.0 - (distance / light.position.w), 0.0, 1.0);
            attenuation *= attenuation; 
            
            point_lighting += light.color.xyz * diff * attenuation * light.color.w;
        }
    }

    // Combine Ambient + Global + Spots + Points
    vec3 result = (ambient + global_lighting + spot_lighting + point_lighting) * fragColor.xyz;
    outColor = vec4(result, fragColor.a);
}