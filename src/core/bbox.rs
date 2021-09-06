use cgmath::EuclideanSpace;

#[derive(Copy, Clone, Debug)]
pub struct Bbox {
    pub p_min: cgmath::Point3<f32>,
    pub p_max: cgmath::Point3<f32>,
}

impl Bbox {
    pub fn from_points(points: &[cgmath::Point3<f32>]) -> Self {
        let mut p_min = points[0];
        let mut p_max = points[0];
        points.iter().skip(1).for_each(|p| {
            p_min = min_point3(p_min, *p);
            p_max = max_point3(p_max, *p);
        });
        Self { p_min, p_max }
    }

    pub fn empty() -> Self {
        Self {
            p_min: cgmath::Point3::new(f32::MAX, f32::MAX, f32::MAX),
            p_max: cgmath::Point3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.p_min.x > self.p_max.x || self.p_min.y > self.p_max.y || self.p_min.z > self.p_max.z
    }

    pub fn merge(mut self, another: Bbox) -> Self {
        self.p_min = min_point3(self.p_min, another.p_min);
        self.p_max = max_point3(self.p_max, another.p_max);
        self
    }

    pub fn surface_area(&self) -> f32 {
        if self.is_empty() {
            0.0
        } else {
            let diff = self.p_max - self.p_min;
            diff.x * diff.y * diff.z
        }
    }

    pub fn centroid(&self) -> cgmath::Point3<f32> {
        self.p_min.midpoint(self.p_max)
    }
}

fn min_point3(a: cgmath::Point3<f32>, b: cgmath::Point3<f32>) -> cgmath::Point3<f32> {
    cgmath::Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
}

fn max_point3(a: cgmath::Point3<f32>, b: cgmath::Point3<f32>) -> cgmath::Point3<f32> {
    cgmath::Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
}
