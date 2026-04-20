use crate::geometrical_shapes::{cuboid::Cuboid, game_object::GameObject, transform::Transform};
pub fn spawn() -> GameObject {
    GameObject::new(Cuboid::generate(15.0, 8.0, 15.0, [0.3, 0.4, 0.6, 1.0]), Transform::default())
}