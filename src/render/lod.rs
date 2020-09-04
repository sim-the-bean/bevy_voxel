use bevy::{
    prelude::*,
    render::{camera::ActiveCameras, render_graph::base},
    transform::prelude::Translation,
};

use crate::{collections::lod_tree::Voxel, world::Map};

pub fn lod_update<T: Voxel>(
    camera: Res<ActiveCameras>,
    mut query: Query<&mut Map<T>>,
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
    for mut world in &mut query.iter() {
        for chunk in &mut world.iter_mut() {
            let (x, y, z) = chunk.position();
            let lod = ((camera_x - x).abs() / 128)
                .max((camera_y - y).abs() / 128)
                .max((camera_z - z).abs() / 128)
                as usize;
            chunk.set_lod(lod);
        }
    }
}
