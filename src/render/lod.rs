use bevy::{
    prelude::*,
    render::{camera::ActiveCameras, render_graph::base},
    transform::prelude::Translation,
};

use crate::{render::entity::Block, world::Chunk};

pub fn lod_update(
    camera: Res<ActiveCameras>,
    mut query: Query<&mut Chunk<Block>>,
    translation: Query<&Translation>,
) {
    let (camera_x, camera_y, camera_z) = if let Some(camera) = camera.get(base::camera::CAMERA3D) {
        let position = translation.get::<Translation>(camera).unwrap();
        (
            position.0.x() as i32,
            position.0.y() as i32,
            position.0.z() as i32,
        )
    } else {
        (0, 0, 0)
    };
    for mut chunk in &mut query.iter() {
        let (x, y, z) = chunk.position();
        let lod = ((camera_x - x * chunk.width() as i32).abs() / 128)
            .max((camera_y - y * chunk.width() as i32).abs() / 128)
            .max((camera_z - z * chunk.width() as i32).abs() / 128) as usize;
        chunk.set_lod(lod);
    }
}
