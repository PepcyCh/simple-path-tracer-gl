use std::rc::Rc;

use cgmath::{Matrix4, Transform};

use super::{Bbox, TriangleMesh};

pub struct Triangle {
    pub mesh: Rc<TriangleMesh>,
    pub index: u32,
    pub indices: [usize; 3],
    pub material: u32,
    pub trans_index: u32,
    pub bbox: Bbox,
}

impl Triangle {
    pub fn new(
        mesh: Rc<TriangleMesh>,
        index: u32,
        indices: [usize; 3],
        material: u32,
        trans: &Matrix4<f32>,
        trans_index: u32,
    ) -> Self {
        let p0 = trans.transform_point(mesh.position(indices[0]));
        let p1 = trans.transform_point(mesh.position(indices[1]));
        let p2 = trans.transform_point(mesh.position(indices[2]));
        let bbox = Bbox::from_points(&[p0, p1, p2]);
        Self {
            mesh,
            index,
            indices,
            material,
            trans_index,
            bbox,
        }
    }

    pub fn bbox(&self) -> Bbox {
        self.bbox
    }
}
