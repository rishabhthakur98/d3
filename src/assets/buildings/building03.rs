use crate::geometrical_shapes::{cuboid::Cuboid, game_object::GameObject, transform::Transform};
pub fn spawn() -> GameObject {
    GameObject::new(Cuboid::generate(5.0, 20.0, 5.0, [0.6, 0.3, 0.3, 1.0]), Transform::default())
}