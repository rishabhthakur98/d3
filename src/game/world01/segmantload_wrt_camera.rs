// src/game/world01/segmantload_wrt_camera.rs

use glam::Vec3;
use crate::geometrical_shapes::game_object::GameObject;
use super::{mapsegment01, mapsegment02};

/// Defines specific segments of the world that can be dynamically loaded
#[derive(Clone, Copy, PartialEq)]
pub enum SegmentId {
    Segment01,
    Segment02,
}

/// A bounding box that triggers the loading of a segment when the camera enters it
pub struct SegmentTriggerZone {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub active_segments: Vec<SegmentId>,
}

impl SegmentTriggerZone {
    /// Checks if a 3D point (like our camera) is inside this virtual cube
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min_bounds.x && point.x <= self.max_bounds.x &&
        point.y >= self.min_bounds.y && point.y <= self.max_bounds.y &&
        point.z >= self.min_bounds.z && point.z <= self.max_bounds.z
    }
}

/// Manages what is currently loaded in memory based on camera position
pub struct WorldStreamer {
    pub load_zones: Vec<SegmentTriggerZone>,
    pub currently_loaded_segments: Vec<SegmentId>,
}

impl Default for WorldStreamer {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldStreamer {
    pub fn new() -> Self {
        Self {
            // MAP VIRTUAL CUBES TO SEGMENTS HERE
            load_zones: vec![
                // If camera is between Z: -50 to 50, load Segment 01
                SegmentTriggerZone {
                    min_bounds: Vec3::new(-100.0, -100.0, -50.0),
                    max_bounds: Vec3::new(100.0, 100.0, 50.0),
                    active_segments: vec![SegmentId::Segment01],
                },
                // If camera is between Z: 50 to 150, load BOTH Segment 01 (so you can look back) and 02
                SegmentTriggerZone {
                    min_bounds: Vec3::new(-100.0, -100.0, 50.0),
                    max_bounds: Vec3::new(100.0, 100.0, 150.0),
                    active_segments: vec![SegmentId::Segment01, SegmentId::Segment02],
                },
                // If camera goes past Z: 150, drop Segment 01 from memory to save RAM
                SegmentTriggerZone {
                    min_bounds: Vec3::new(-100.0, -100.0, 150.0),
                    max_bounds: Vec3::new(100.0, 100.0, 300.0),
                    active_segments: vec![SegmentId::Segment02],
                },
            ],
            currently_loaded_segments: Vec::new(),
        }
    }

    /// Evaluates the camera location and returns a list of GameObjects that need rendering
    pub fn get_visible_objects(&mut self, camera_pos: Vec3) -> Vec<GameObject> {
        let mut segments_needed = Vec::new();

        // 1. Find which virtual cubes the camera is inside
        for zone in &self.load_zones {
            if zone.contains(camera_pos) {
                for &segment_id in &zone.active_segments {
                    if !segments_needed.contains(&segment_id) {
                        segments_needed.push(segment_id);
                    }
                }
            }
        }

        // Update tracking state
        self.currently_loaded_segments = segments_needed.clone();

        // 2. Actually load the physical geometry for the needed segments
        let mut renderable_objects = Vec::new();
        
        for needed in segments_needed {
            match needed {
                SegmentId::Segment01 => renderable_objects.extend(mapsegment01::load_segment()),
                SegmentId::Segment02 => renderable_objects.extend(mapsegment02::load_segment()),
            }
        }

        renderable_objects
    }
}