// src/game/world01/postprocess_config.rs

use crate::postprocessing::config::PostProcessConfig;

pub fn get_postprocess_config() -> PostProcessConfig {
    PostProcessConfig {
        enabled: true, 
        
        // 1. EXPOSURE: Bring this down. 
        // 1.2 was adding 20% artificial brightness to everything. 1.0 is neutral. 
        // Try 0.85 to 1.0 for a more grounded, realistic lighting setup.
        exposure: 0.9, 
        
        // 2. GAMMA: (CRITICAL FIX) 
        // If your image looks "milky" or extremely bright grey, you are likely 
        // double-applying Gamma correction. Set this to 1.0. 
        // (Vulkan's SRGB swapchain automatically applies 2.2, so applying 2.2 in the shader makes it 4.84!)
        gamma: 1.0,    
        
        // 3. CONTRAST: Increase this slightly.
        // Higher contrast pushes darks deeper and lights brighter, killing the "washed out" grey look.
        contrast: 1.25, 
        
        // 4. SATURATION: Boost colors slightly to compensate for the darker exposure.
        saturation: 1.15, 
        
        vignette_strength: 0.7, // Slightly stronger vignette to darken the screen edges
        chromatic_aberration: 0.005, 
        film_grain: 0.015, 
    }
}