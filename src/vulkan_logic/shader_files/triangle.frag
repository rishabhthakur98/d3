// ... [Keep the CalculateShadow function exactly as it is] ...

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

    // --- AAA VOLUMETRIC FOG & GOD RAYS ---
    vec3 viewDirFog = ubo.camera_pos.xyz - fragWorldPos;
    float viewDist = length(viewDirFog);
    vec3 viewDirNorm = viewDirFog / viewDist; // Points from pixel TO camera

    // 1. Distance & Height Falloff
    float distanceFog = exp(-viewDist * ubo.fog_color.w);
    float heightFog = exp(-(fragWorldPos.y - ubo.fog_params.y) * ubo.fog_params.x);
    float finalFogFactor = clamp(distanceFog * heightFog, 0.0, 1.0);

    // 2. Volumetric Raymarching (Light Shafts)
    float inScattering = 0.0;
    if (ubo.fog_params.z > 0.0 && ubo.global_count > 0 && ubo.global_lights[0].color.w > 0.5) {
        int steps = 8;
        
        // SAFEGUARD: Cap the march distance so looking at distant buildings doesn't multiply light to infinity
        float marchDist = min(viewDist, 100.0); 
        float stepSize = marchDist / float(steps);
        
        // March from the camera towards the object for better God Ray resolution
        vec3 currentPos = ubo.camera_pos.xyz;
        vec3 marchDir = -viewDirNorm; 
        vec3 sunDir = normalize(-ubo.global_lights[0].direction.xyz);
        
        for(int i = 0; i < steps; i++) {
            vec4 shadowSpace = pc.light_space_matrix * vec4(currentPos, 1.0);
            float shadow = CalculateShadow(shadowSpace, vec3(0.0, 1.0, 0.0), sunDir);
            
            // Add light only if this pocket of air is NOT in shadow
            inScattering += (1.0 - shadow) * ubo.fog_params.z * stepSize;
            currentPos += marchDir * stepSize;
        }
        
        // SAFEGUARD: Prevent the god rays from exceeding absolute white
        inScattering = min(inScattering, 1.5); 
    }
    
    // 3. Mie Scattering Phase (Makes God Rays glow brighter near the sun)
    float sunDot = 1.0;
    if (ubo.global_count > 0) {
        sunDot = max(dot(-viewDirNorm, -normalize(ubo.global_lights[0].direction.xyz)), 0.0);
    }
    float miePhase = pow(sunDot, 4.0) * 2.0 + 0.5; 
    
    vec3 finalFogColor = ubo.fog_color.xyz;
    if (ubo.global_count > 0) {
        finalFogColor += (ubo.global_lights[0].color.xyz * inScattering * miePhase);
    }

    // 4. Blend the raw pixel color into the atmospheric fog
    result = mix(finalFogColor, result, finalFogFactor);

    outColor = vec4(result, fragColor.a);
}