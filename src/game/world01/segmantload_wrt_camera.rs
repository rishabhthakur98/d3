// src/game/world01/segmantload_wrt_camera.rs
use glam::Vec3;
use crate::geometrical_shapes::game_object::GameObject;
use crate::light::spot::SpotLight;
use super::{mapsegment01, mapsegment02};

#[derive(Clone, Copy, PartialEq)]
pub enum SegmentId { Segment01, Segment02 }

pub struct SegmentTriggerZone {
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub active_segments: Vec<SegmentId>,
}

impl SegmentTriggerZone {
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min_bounds.x && point.x <= self.max_bounds.x &&
        point.y >= self.min_bounds.y && point.y <= self.max_bounds.y &&
        point.z >= self.min_bounds.z && point.z <= self.max_bounds.z
    }
}

pub struct WorldStreamer {
    pub load_zones: Vec<SegmentTriggerZone>,
    pub currently_loaded_segments: Vec<SegmentId>,
}

impl Default for WorldStreamer {
    fn default() -> Self { Self::new() }
}

impl WorldStreamer {
    pub fn new() -> Self {
        Self {
            load_zones: vec![
                SegmentTriggerZone {
                    min_bounds: Vec3::new(-100.0, -100.0, -50.0), max_bounds: Vec3::new(100.0, 100.0, 50.0), active_segments: vec![SegmentId::Segment01],
                },
                SegmentTriggerZone {
                    min_bounds: Vec3::new(-100.0, -100.0, 50.0), max_bounds: Vec3::new(100.0, 100.0, 150.0), active_segments: vec![SegmentId::Segment01, SegmentId::Segment02],
                },
                SegmentTriggerZone {
                    min_bounds: Vec3::new(-100.0, -100.0, 150.0), max_bounds: Vec3::new(100.0, 100.0, 300.0), active_segments: vec![SegmentId::Segment02],
                },
            ],
            currently_loaded_segments: Vec::new(),
        }
    }

    /// Now returns a tuple of Objects AND Lights so the game engine has full decoupled lists
    pub fn get_visible_objects(&mut self, camera_pos: Vec3) -> (Vec<GameObject>, Vec<SpotLight>) {
        let mut segments_needed = Vec::new();

        for zone in &self.load_zones {
            if zone.contains(camera_pos) {
                for &segment_id in &zone.active_segments {
                    if !segments_needed.contains(&segment_id) { segments_needed.push(segment_id); }
                }
            }
        }

        self.currently_loaded_segments = segments_needed.clone();

        let mut renderable_objects = Vec::new();
        let mut active_spots = Vec::new();
        
        for needed in segments_needed {
            let (objs, spots) = match needed {
                SegmentId::Segment01 => mapsegment01::load_segment(),
                SegmentId::Segment02 => mapsegment02::load_segment(),
            };
            renderable_objects.extend(objs);
            active_spots.extend(spots);
        }

        (renderable_objects, active_spots)
    }
}