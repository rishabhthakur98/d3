#version 450

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragNormal;
layout(location = 3) in vec3 fragWorldPos;

layout(location = 0) out vec4 outColor;

// Defines the data structure matching our Rust SpotLightData
struct SpotLight {
    vec4 position;  
    vec4 direction; 
    vec4 color;     
    vec4 params;    
};

// Defines the UBO matching our Rust LightUBO
layout(set = 0, binding = 0) uniform LightUBO {
    vec4 ambient_color; 
    vec4 global_dir;    
    vec4 global_color;  
    vec4 camera_pos;    
    
    uint spot_count;
    uint pad1, pad2, pad3;
    
    SpotLight spot_lights[10];
} ubo;

void main() {
    vec3 normal = normalize(fragNormal);
    
    // 1. Ambient Lighting
    vec3 ambient = ubo.ambient_color.xyz * ubo.ambient_color.w;
    
    // 2. Spotlight Calculation
    vec3 spot_lighting = vec3(0.0);
    
    for(uint i = 0; i < ubo.spot_count; i++) {
        SpotLight light = ubo.spot_lights[i];
        vec3 lightDir = light.position.xyz - fragWorldPos;
        float distance = length(lightDir);

        if (distance < light.position.w) { // Is pixel within the light's range?
            lightDir = normalize(lightDir);
            
            // Diffuse bounce (how direct is the light hitting the surface?)
            float diff = max(dot(normal, lightDir), 0.0);

            // Attenuation (light drops off quadratically over distance)
            float attenuation = clamp(1.0 - (distance / light.position.w), 0.0, 1.0);
            attenuation *= attenuation;

            // Cone Angle (soft edges on the flashlight/streetlight)
            float theta = dot(lightDir, normalize(-light.direction.xyz));
            float epsilon = light.color.w - light.params.x; // Inner cone - Outer cone
            float intensity = clamp((theta - light.params.x) / epsilon, 0.0, 1.0);

            spot_lighting += light.color.xyz * diff * attenuation * intensity * light.direction.w;
        }
    }

    // Combine all lights and multiply by the actual color of the object
    vec3 result = (ambient + spot_lighting) * fragColor.xyz;
    
    outColor = vec4(result, fragColor.a);
}