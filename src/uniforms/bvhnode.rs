use crate::core::Bbox;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BvhNode {
    lc_ind: i32,
    rc_ind: i32,
    prim_start: i32,
    prim_end: i32,
    bbox: super::Bbox,
}

impl BvhNode {
    pub fn new(lc_ind: i32, rc_ind: i32, prim_start: i32, prim_end: i32, bbox: Bbox) -> Self {
        Self {
            lc_ind,
            rc_ind,
            prim_start,
            prim_end,
            bbox: super::Bbox::new(bbox.p_min, bbox.p_max),
        }
    }
}
