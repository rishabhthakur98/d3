// src/vulkan_logic/shader_files/triangle.frag

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
    vec4 position;
    vec4 color;
};

layout(set = 0, binding = 0) uniform LightUBO {
    vec4 ambient_color;
    vec4 camera_pos;
    uint global_count;
    uint spot_count;
    uint point_count;
    uint _pad;
    vec4 fog_color;
    vec4 fog_params;
    
    // INCREASED: Allows iteration of 100 spawnable entities 
    GlobalLight global_lights[4];
    SpotLight spot_lights[100];
    PointLight point_lights[100];
} ubo;

layout(set = 0, binding = 1) uniform sampler2D shadowMap;

layout(push_constant) uniform PushConstants {
    mat4 view_proj;
    mat4 model;
    mat4 light_space_matrix;
} pc;

float CalculateShadow(vec4 fragPosLightSpace, vec3 normal, vec3 lightDir) {
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords.xy = projCoords.xy * 0.5 + 0.5;

    if (projCoords.z > 1.0 || projCoords.x < 0.0 || projCoords.x > 1.0 || projCoords.y < 0.0 || projCoords.y > 1.0) {
        return 0.0;
    }

    float currentDepth = projCoords.z;
    float bias = max(0.0005 * (1.0 - dot(normal, lightDir)), 0.0001);

    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);

    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMap, projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    return shadow / 9.0;
}

void main() {
    vec3 normal = normalize(fragNormal);
    vec3 ambient = ubo.ambient_color.xyz * ubo.ambient_color.w;
    
    vec3 global_lighting = vec3(0.0);
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

    vec3 point_lighting = vec3(0.0);
    for(uint i = 0; i < ubo.point_count; i++) {
        PointLight light = ubo.point_lights[i];
        vec3 lightDir = light.position.xyz - fragWorldPos;
        float distance = length(lightDir);
        if (distance < light.position.w) { 
            lightDir = normalize(lightDir);
            float diff = max(dot(normal, lightDir), 0.0);
            float attenuation = clamp(1.0 - (distance / light.position.w), 0.0, 1.0);
            attenuation *= attenuation;
            point_lighting += light.color.xyz * diff * attenuation * light.color.w;
        }
    }

    vec3 result = (ambient + global_lighting + spot_lighting + point_lighting) * fragColor.xyz;
    vec3 viewDirFog = ubo.camera_pos.xyz - fragWorldPos;
    float viewDist = length(viewDirFog);
    vec3 viewDirNorm = viewDirFog / viewDist;

    float distanceFog = exp(-viewDist * ubo.fog_color.w);
    float heightFog = exp(-(fragWorldPos.y - ubo.fog_params.y) * ubo.fog_params.x);
    float finalFogFactor = clamp(distanceFog * heightFog, 0.0, 1.0);
    
    float inScattering = 0.0;
    if (ubo.fog_params.z > 0.0 && ubo.global_count > 0 && ubo.global_lights[0].color.w > 0.5) {
        int steps = 8;
        float marchDist = min(viewDist, 100.0);
        float stepSize = marchDist / float(steps);
        
        vec3 currentPos = ubo.camera_pos.xyz;
        vec3 marchDir = -viewDirNorm; 
        vec3 sunDir = normalize(-ubo.global_lights[0].direction.xyz);
        
        for(int i = 0; i < steps; i++) {
            vec4 shadowSpace = pc.light_space_matrix * vec4(currentPos, 1.0);
            float shadow = CalculateShadow(shadowSpace, vec3(0.0, 1.0, 0.0), sunDir);
            
            inScattering += (1.0 - shadow) * ubo.fog_params.z * stepSize;
            currentPos += marchDir * stepSize;
        }
        
        inScattering = min(inScattering, 1.5);
    }
    
    float sunDot = 1.0;
    if (ubo.global_count > 0) {
        sunDot = max(dot(-viewDirNorm, -normalize(ubo.global_lights[0].direction.xyz)), 0.0);
    }
    float miePhase = pow(sunDot, 4.0) * 2.0 + 0.5; 
    
    vec3 finalFogColor = ubo.fog_color.xyz;
    if (ubo.global_count > 0) {
        finalFogColor += (ubo.global_lights[0].color.xyz * inScattering * miePhase);
    }

    result = mix(finalFogColor, result, finalFogFactor);
    outColor = vec4(result, fragColor.a);
}