// src/vulkan_logic/shader_files/clouds.frag
#version 450

layout(location = 0) in vec3 fragRayDir;
layout(location = 0) out vec4 outColor;

struct CloudVolumeData {
    vec4 min_bounds;      // xyz: min_bounds, w: coverage
    vec4 max_bounds;      // xyz: max_bounds, w: density
    vec4 base_color;      // xyz: base_color, w: wind_speed
    vec4 highlight_color; // xyz: highlight_color
    vec4 wind_dir;        // xyz: wind_direction
};

layout(set = 0, binding = 0) uniform CloudUBO {
    mat4 inv_view_proj;
    vec4 camera_pos;
    vec4 sun_dir;
    vec4 sun_color;
    float time;
    uint cloud_count;
    uint pad1, pad2;
    CloudVolumeData clouds[10];
} ubo;

// Hash and 3D Noise for procedural cloud shaping
float hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

float noise(vec3 x) {
    vec3 i = floor(x);
    vec3 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash(i + vec3(0,0,0)), hash(i + vec3(1,0,0)), f.x),
                   mix(hash(i + vec3(0,1,0)), hash(i + vec3(1,1,0)), f.x), f.y),
               mix(mix(hash(i + vec3(0,0,1)), hash(i + vec3(1,0,1)), f.x),
                   mix(hash(i + vec3(0,1,1)), hash(i + vec3(1,1,1)), f.x), f.y), f.z);
}

// Fractional Brownian Motion (FBM) to create fluffy, organic shapes
float fbm(vec3 p) {
    float f = 0.0;
    float amp = 0.5;
    for (int i = 0; i < 4; i++) {
        f += amp * noise(p);
        p *= 2.0;
        amp *= 0.5;
    }
    return f;
}

// Safely intersect the view ray with the AABB boundary of the dynamic Cloud Volume!
vec2 intersectAABB(vec3 ro, vec3 rd, vec3 boxMin, vec3 boxMax) {
    vec3 invRd = 1.0 / (rd + 1e-6); // Prevent zero division
    vec3 t0 = (boxMin - ro) * invRd;
    vec3 t1 = (boxMax - ro) * invRd;
    
    vec3 tmin = min(t0, t1);
    vec3 tmax = max(t0, t1);
    
    float tNear = max(max(tmin.x, tmin.y), tmin.z);
    float tFar = min(min(tmax.x, tmax.y), tmax.z);
    
    return vec2(tNear, tFar);
}

void main() {
    vec3 rayDir = normalize(fragRayDir);
    vec3 sunDir = normalize(-ubo.sun_dir.xyz);
    
    float finalDensity = 0.0;
    vec3 finalCloudColor = vec3(0.0);

    // AAA Iteration: Process every single volumetric bounding box inside the map
    for (uint c = 0; c < ubo.cloud_count; c++) {
        CloudVolumeData cloud = ubo.clouds[c];
        
        vec2 hit = intersectAABB(ubo.camera_pos.xyz, rayDir, cloud.min_bounds.xyz, cloud.max_bounds.xyz);
        float tNear = hit.x;
        float tFar = hit.y;

        // If the ray enters the localized cloud box
        if (tNear < tFar && tFar > 0.0) {
            tNear = max(tNear, 0.0);
            
            // Limit steps for AAA performance balance
            int steps = 32; 
            float stepSize = (tFar - tNear) / float(steps);
            vec3 currentPos = ubo.camera_pos.xyz + rayDir * tNear;
            
            float totalDensity = 0.0;
            float lightEnergy = 0.0;
            vec3 windOffset = cloud.wind_dir.xyz * (ubo.time * cloud.base_color.w);

            for (int i = 0; i < steps; i++) {
                // Sample the 3D Noise to find "Fluffiness"
                vec3 samplePos = (currentPos + windOffset) * 0.01;
                float n = fbm(samplePos);
                
                // Shape the cloud: dense in the middle, fading smoothly at the top and bottom
                float heightFraction = (currentPos.y - cloud.min_bounds.y) / (cloud.max_bounds.y - cloud.min_bounds.y);
                float verticalGradient = 1.0 - abs(heightFraction * 2.0 - 1.0);
                
                float density = max(0.0, n - (1.0 - cloud.min_bounds.w)) * verticalGradient;
                
                if (density > 0.0) {
                    // If we hit a cloud, calculate how much sunlight hits this exact piece of fluff
                    float shadowSample = fbm(samplePos + sunDir * 0.05);
                    float localLight = exp(-shadowSample * 2.0); 
                    
                    // Beer's Law: Accumulate physical cloud thickness
                    totalDensity += density * cloud.max_bounds.w;
                    lightEnergy += density * localLight * exp(-totalDensity);
                    
                    // Early exit optimization if cloud becomes fully opaque
                    if (totalDensity > 1.0) {
                        totalDensity = 1.0;
                        break;
                    }
                }
                currentPos += rayDir * stepSize;
            }

            if (totalDensity > 0.0) {
                // Blend the deep cloud shadows with the bright sunlit edges
                vec3 cloudColor = mix(cloud.base_color.xyz, cloud.highlight_color.xyz, lightEnergy);
                cloudColor *= ubo.sun_color.xyz * ubo.sun_dir.w; // Tint by the sunset/sunrise
                
                // Merge this cloud's impact into the global ray layer!
                finalCloudColor = mix(finalCloudColor, cloudColor, totalDensity);
                finalDensity = min(finalDensity + totalDensity, 1.0);
            }
        }
    }

    if (finalDensity <= 0.0) {
        discard;
    }

    outColor = vec4(finalCloudColor, finalDensity);
}