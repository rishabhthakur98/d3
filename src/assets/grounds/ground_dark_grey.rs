use crate::geometrical_shapes::{rectangle::Rectangle, game_object::GameObject, transform::Transform};
pub fn spawn() -> GameObject {
    GameObject::new(Rectangle::generate(20.0, 20.0, [0.15, 0.15, 0.15, 1.0]), Transform::default())
}