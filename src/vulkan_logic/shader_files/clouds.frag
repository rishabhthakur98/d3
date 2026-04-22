#version 450

layout(location = 0) in vec3 fragRayDir;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform CloudUBO {
    mat4 inv_view_proj;
    vec4 camera_pos;
    vec4 sun_dir;
    vec4 sun_color;
    vec4 base_color;
    vec4 highlight_color;
    vec4 params; // x: Time, y: Coverage, z: Density, w: Wind Speed
    vec4 wind_dir;
    vec4 heights; // x: Min Height, y: Max Height
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

// Intersects a ray with a flat Y-plane to figure out where the clouds start and end
float intersectPlane(vec3 ro, vec3 rd, float height) {
    if (abs(rd.y) < 0.001) return -1.0;
    float t = (height - ro.y) / rd.y;
    return t > 0.0 ? t : -1.0;
}

void main() {
    vec3 rayDir = normalize(fragRayDir);
    
    // Only draw clouds if looking upwards into the sky bounding box
    if (rayDir.y < 0.0) {
        discard;
    }

    float tMin = intersectPlane(ubo.camera_pos.xyz, rayDir, ubo.heights.x);
    float tMax = intersectPlane(ubo.camera_pos.xyz, rayDir, ubo.heights.y);
    
    if (tMin < 0.0 || tMax < 0.0) {
        discard;
    }

    vec3 sunDir = normalize(-ubo.sun_dir.xyz);
    float time = ubo.params.x;
    vec3 windOffset = ubo.wind_dir.xyz * (time * ubo.params.w);

    // Raymarching Setup
    int steps = 32; // Limit steps for AAA performance balance
    float stepSize = (tMax - tMin) / float(steps);
    vec3 currentPos = ubo.camera_pos.xyz + rayDir * tMin;
    
    float totalDensity = 0.0;
    float lightEnergy = 0.0;

    for (int i = 0; i < steps; i++) {
        // Sample the 3D Noise to find "Fluffiness"
        vec3 samplePos = (currentPos + windOffset) * 0.01;
        float n = fbm(samplePos);
        
        // Shape the cloud: dense in the middle, fading at the top and bottom
        float heightFraction = (currentPos.y - ubo.heights.x) / (ubo.heights.y - ubo.heights.x);
        float verticalGradient = 1.0 - abs(heightFraction * 2.0 - 1.0);
        
        float density = max(0.0, n - (1.0 - ubo.params.y)) * verticalGradient;
        
        if (density > 0.0) {
            // If we hit a cloud, calculate how much sunlight hits this exact piece of fluff
            float shadowSample = fbm(samplePos + sunDir * 0.05);
            float localLight = exp(-shadowSample * 2.0); 
            
            // Beer's Law: Accumulate physical cloud thickness
            totalDensity += density * ubo.params.z;
            lightEnergy += density * localLight * exp(-totalDensity);
            
            // Early exit optimization if cloud becomes fully opaque
            if (totalDensity > 1.0) {
                totalDensity = 1.0;
                break;
            }
        }
        currentPos += rayDir * stepSize;
    }

    if (totalDensity <= 0.0) {
        discard;
    }

    // Blend the deep cloud shadows with the bright sunlit edges
    vec3 cloudColor = mix(ubo.base_color.xyz, ubo.highlight_color.xyz, lightEnergy);
    cloudColor *= ubo.sun_color.xyz * ubo.sun_dir.w; // Tint by the sunset/sunrise

    outColor = vec4(cloudColor, totalDensity);
}