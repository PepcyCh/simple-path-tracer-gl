use std::collections::HashSet;

use super::Triangle;
use crate::{core::Bbox, uniforms};

pub struct BvhAccel {
    bvh_root: Option<Box<BvhNode>>,
}

struct BvhNode {
    lc: Option<Box<BvhNode>>,
    rc: Option<Box<BvhNode>>,
    bbox: Bbox,
    start: usize,
    end: usize,
    index: u32,
}

impl BvhAccel {
    pub fn new(
        mut triangles: &mut Vec<Triangle>,
        max_leaf_size: usize,
        bucket_number: usize,
    ) -> Self {
        if triangles.is_empty() {
            return Self { bvh_root: None };
        };

        let mut curr_node_index = 0;

        let mut bbox = triangles[0].bbox();
        triangles.iter().skip(1).for_each(|prim| {
            bbox = bbox.merge(prim.bbox());
        });
        let mut bvh_root = Box::new(BvhNode::new(0, triangles.len(), bbox, curr_node_index));
        curr_node_index += 1;

        let mut stack = vec![&mut bvh_root];
        while let Some(u) = stack.pop() {
            if u.size() <= max_leaf_size {
                continue;
            }

            let bbox = u.bbox;
            let len_per_bucket = (bbox.p_max - bbox.p_min) / bucket_number as f32;

            let mut boxes_x = vec![Bbox::empty(); bucket_number];
            let mut boxes_y = vec![Bbox::empty(); bucket_number];
            let mut boxes_z = vec![Bbox::empty(); bucket_number];

            let mut prim_indices_x = vec![vec![]; bucket_number];
            let mut prim_indices_y = vec![vec![]; bucket_number];
            let mut prim_indices_z = vec![vec![]; bucket_number];

            for i in u.start..u.end {
                let bbox = triangles[i].bbox();
                let centroid = bbox.centroid();

                let x = (centroid.x - bbox.p_min.x) / len_per_bucket.x;
                if x >= 0.0 && x < bucket_number as f32 {
                    let x = x as usize;
                    boxes_x[x] = boxes_x[x].merge(bbox);
                    prim_indices_x[x].push(i);
                }

                let y = (centroid.y - bbox.p_min.y) / len_per_bucket.y;
                if y >= 0.0 && y < bucket_number as f32 {
                    let y = y as usize;
                    boxes_y[y] = boxes_y[y].merge(bbox);
                    prim_indices_y[y].push(i);
                }

                let z = (centroid.z - bbox.p_min.z) / len_per_bucket.z;
                if z >= 0.0 && z < bucket_number as f32 {
                    let z = z as usize;
                    boxes_z[z] = boxes_z[z].merge(bbox);
                    prim_indices_z[z].push(i);
                }
            }

            let (best_cost_x, best_split_x) = if len_per_bucket.x > 0.0001 {
                Self::find_best_split(&boxes_x, &prim_indices_x, u.size(), bucket_number)
            } else {
                (f32::MAX, bucket_number / 2)
            };
            let (best_cost_y, best_split_y) = if len_per_bucket.y > 0.0001 {
                Self::find_best_split(&boxes_y, &prim_indices_y, u.size(), bucket_number)
            } else {
                (f32::MAX, bucket_number / 2)
            };
            let (best_cost_z, best_split_z) = if len_per_bucket.z > 0.0001 {
                Self::find_best_split(&boxes_z, &prim_indices_z, u.size(), bucket_number)
            } else {
                (f32::MAX, bucket_number / 2)
            };

            let (lc, rc) = if best_cost_x <= best_cost_y && best_cost_x <= best_cost_z {
                Self::split_at(
                    best_split_x,
                    bucket_number,
                    &boxes_x,
                    &mut prim_indices_x,
                    &mut triangles,
                    u.start,
                    u.end,
                    &mut curr_node_index,
                )
            } else if best_cost_y <= best_cost_x && best_cost_y <= best_cost_z {
                Self::split_at(
                    best_split_y,
                    bucket_number,
                    &boxes_y,
                    &mut prim_indices_y,
                    &mut triangles,
                    u.start,
                    u.end,
                    &mut curr_node_index,
                )
            } else {
                Self::split_at(
                    best_split_z,
                    bucket_number,
                    &boxes_z,
                    &mut prim_indices_z,
                    &mut triangles,
                    u.start,
                    u.end,
                    &mut curr_node_index,
                )
            };
            if lc.size() == 0 || rc.size() == 0 {
                continue;
            }
            u.lc = Some(lc);
            u.rc = Some(rc);

            stack.push(u.lc.as_mut().unwrap());
            stack.push(u.rc.as_mut().unwrap());
        }

        Self {
            bvh_root: Some(bvh_root),
        }
    }

    pub fn fill_in_uniform(&self, uniform: &mut uniforms::SceneUniform) {
        if self.bvh_root.is_none() {
            return;
        }

        let mut stack = vec![self.bvh_root.as_ref().unwrap()];
        while let Some(u) = stack.pop() {
            let node = uniforms::BvhNode::new(
                u.lc.as_ref().map_or(-1, |c| c.index as i32),
                u.rc.as_ref().map_or(-1, |c| c.index as i32),
                u.start as i32,
                u.end as i32,
                u.bbox,
            );
            assert!(
                (u.index as usize) < uniform.bvh_nodes.len(),
                "too many bvh nodes"
            );
            uniform.bvh_nodes[u.index as usize] = node;
            if !u.is_leaf() {
                stack.push(u.lc.as_ref().unwrap());
                stack.push(u.rc.as_ref().unwrap());
            }
        }
    }

    fn find_best_split(
        boxes: &Vec<Bbox>,
        prim_indices: &Vec<Vec<usize>>,
        prim_total_count: usize,
        bucket_number: usize,
    ) -> (f32, usize) {
        let mut best_cost = f32::MAX;
        let mut best_split = 0;
        let mut boxes_l = boxes.clone();
        let mut boxes_r = boxes.clone();
        let mut prim_count = vec![0; bucket_number];
        prim_count[0] = prim_indices[0].len();
        for i in 1..bucket_number {
            boxes_l[i] = boxes_l[i].merge(boxes_l[i - 1]);
            prim_count[i] = prim_count[i - 1] + prim_indices[i].len();
        }
        for i in (0..bucket_number - 1).rev() {
            boxes_r[i] = boxes_r[i].merge(boxes_r[i + 1]);
        }
        for i in 1..bucket_number {
            let left_surface_area = boxes_l[i - 1].surface_area();
            let left_count = prim_count[i - 1] as f32;
            let right_surface_area = boxes_r[i].surface_area();
            let right_count = prim_total_count as f32 - left_count;
            let cost = left_surface_area * left_count + right_surface_area * right_count;
            if cost < best_cost {
                best_cost = cost;
                best_split = i;
            }
        }
        (best_cost, best_split)
    }

    fn split_at(
        split: usize,
        bucket_number: usize,
        boxes: &Vec<Bbox>,
        prim_indices: &mut Vec<Vec<usize>>,
        primitives: &mut Vec<Triangle>,
        start: usize,
        end: usize,
        curr_node_index: &mut u32,
    ) -> (Box<BvhNode>, Box<BvhNode>) {
        let mut left_bbox = Bbox::empty();
        let mut left_indices = vec![];
        for i in 0..split {
            left_bbox = left_bbox.merge(boxes[i]);
            left_indices.append(&mut prim_indices[i]);
        }
        let left_indices: HashSet<usize> = left_indices.into_iter().collect();

        let mut right_bbox = Bbox::empty();
        let mut right_indices = vec![];
        for i in split..bucket_number {
            right_bbox = right_bbox.merge(boxes[i]);
            right_indices.append(&mut prim_indices[i]);
        }
        let right_indices: HashSet<usize> = right_indices.into_iter().collect();

        let mut rp = end - 1;
        let mid = start + left_indices.len();
        for lp in start..mid {
            if !left_indices.contains(&lp) {
                while right_indices.contains(&rp) {
                    rp -= 1;
                }
                primitives.swap(lp, rp);
                rp -= 1;
            }
        }
        let lc = Box::new(BvhNode::new(start, mid, left_bbox, *curr_node_index));
        let rc = Box::new(BvhNode::new(mid, end, right_bbox, *curr_node_index + 1));
        *curr_node_index += 2;
        (lc, rc)
    }
}

impl BvhNode {
    fn new(start: usize, end: usize, bbox: Bbox, index: u32) -> Self {
        Self {
            lc: None,
            rc: None,
            bbox,
            start,
            end,
            index,
        }
    }

    fn size(&self) -> usize {
        self.end - self.start
    }
    fn is_leaf(&self) -> bool {
        self.lc.is_none()
    }
}
