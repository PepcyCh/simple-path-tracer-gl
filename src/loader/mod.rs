use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{bail, Context, Result};
use bytemuck::Zeroable;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};

use crate::{
    core::{BvhAccel, Material, MeshVertex, Triangle, TriangleMesh},
    renderer::{OutputConfig, Renderer},
    uniforms,
};

struct InputLoader {
    path: PathBuf,
    meshes: Vec<Vec<Rc<TriangleMesh>>>,
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Renderer> {
    let mut loader = InputLoader::new(path);
    loader.load()
}

impl InputLoader {
    fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        Self {
            path,
            meshes: vec![],
        }
    }

    fn load(&mut self) -> Result<Renderer> {
        let json_file = std::fs::File::open(&self.path)?;
        let json_reader = std::io::BufReader::new(json_file);
        let json_value: serde_json::Value = serde_json::from_reader(json_reader)?;

        let output_config_json = json_value.get("output").context("top: no 'output' field")?;
        let output_config = self.load_output(output_config_json)?;

        let max_depth = get_int_field(&json_value, "top", "max_depth")?;

        let camera_json = json_value.get("camera").context("top: no 'camera' field")?;
        let camera = self.load_camera(camera_json)?;

        let materials_json = json_value
            .get("materials")
            .context("top: no 'materials' field")?;
        let materials = self.load_materials(materials_json)?;

        let meshes_json = json_value.get("meshes").context("top: no 'meshes' field")?;
        self.meshes = self.load_meshes(meshes_json)?;

        let lights_json = json_value.get("lights").context("top: no 'lights' field")?;
        let lights = self.load_lights(lights_json)?;

        let objects_json = json_value
            .get("objects")
            .context("top: no 'objects' field")?;
        let (mut triangles, transforms) = self.load_objects(objects_json)?;

        let bvh_json = json_value.get("bvh").context("top: no 'bvh' field")?;
        let bvh = self.load_bvh(bvh_json, &mut triangles)?;

        let mut scene_uniform = unsafe {
            let layout = std::alloc::Layout::new::<uniforms::SceneUniform>();
            let prt = std::alloc::alloc(layout) as *mut uniforms::SceneUniform;
            Box::from_raw(prt)
        };

        scene_uniform.max_depth = max_depth;
        // bvh nodes
        bvh.fill_in_uniform(&mut scene_uniform);
        // mesh vertices
        let mut vertex_index = 0;
        let mut index_offsets = vec![0; self.meshes.len()];
        let mut index_offset = 0;
        for model in &self.meshes {
            for mesh in model {
                for vert in &mesh.vertices {
                    assert!(
                        vertex_index < scene_uniform.vertices.len(),
                        "too many vertices"
                    );
                    scene_uniform.vertices[vertex_index] =
                        uniforms::Vertex::new(vert.position, vert.normal);
                    vertex_index += 1;
                }
                index_offsets[mesh.mesh_index as usize] = index_offset;
                index_offset += mesh.vertices.len();
            }
        }
        // objects & object triangles
        for (index, tri) in triangles.iter().enumerate() {
            let offset = index_offsets[tri.mesh.mesh_index as usize];
            let indices = [
                tri.indices[0] + offset,
                tri.indices[1] + offset,
                tri.indices[2] + offset,
            ];
            assert!(index < scene_uniform.triangles.len(), "too many triangles");
            assert!(
                (tri.trans_index as usize) < scene_uniform.objects.len(),
                "too many objects"
            );
            scene_uniform.triangles[index] =
                uniforms::Triangle::new(indices, tri.material, tri.trans_index);
            scene_uniform.objects[tri.trans_index as usize] =
                uniforms::SceneObject::new(transforms[tri.trans_index as usize]);
        }
        //materials
        for (index, mat) in materials.iter().enumerate() {
            assert!(index < scene_uniform.materials.len(), "too many materials");
            scene_uniform.materials[index] = uniforms::Material::new(
                mat.albedo,
                mat.ior,
                mat.roughness,
                mat.metallic,
                mat.is_translucent,
            );
        }
        // lights
        scene_uniform.lights_count = lights.len() as u32;
        for (index, light) in lights.into_iter().enumerate() {
            assert!(index < scene_uniform.lights.len(), "too many lights");
            scene_uniform.lights[index] = light;
        }

        let mut variable_uniform = uniforms::VariableUniform::zeroed();
        variable_uniform.camera = camera;
        variable_uniform.curr_light_index = 0;

        Ok(Renderer::new(
            output_config,
            scene_uniform,
            variable_uniform,
        ))
    }

    fn load_output(&self, value: &serde_json::Value) -> Result<OutputConfig> {
        let file = get_str_field_or(value, "output", "file", "pt_{}.jpg")?;
        let width = get_int_field(value, "output", "width")?;
        let height = get_int_field(value, "output", "height")?;
        let scale = get_int_field_or(value, "output", "scale", 1)?;
        Ok(OutputConfig {
            file: file.to_string(),
            width,
            height,
            scale,
        })
    }

    fn load_camera(&self, value: &serde_json::Value) -> Result<uniforms::Camera> {
        let eye = get_float_array3_field(value, "camera-perspective", "eye")?;
        let forward = get_float_array3_field(value, "camera-perspective", "forward")?;
        let up = get_float_array3_field(value, "camera-perspective", "up")?;
        let fov = get_float_field(value, "camera-perspective", "fov")?;
        Ok(uniforms::Camera::new(
            eye.into(),
            forward.into(),
            up.into(),
            fov,
        ))
    }

    fn load_materials(&self, value: &serde_json::Value) -> Result<Vec<Material>> {
        let arr = value
            .as_array()
            .context("top: 'materials' should be an array")?;
        let mut materials = Vec::with_capacity(arr.len());
        for mat_json in arr {
            let ior = get_float_field(mat_json, "material", "ior")?;
            let albedo = get_float_array3_field(mat_json, "material", "albedo")?;
            let roughness = get_float_field(mat_json, "material", "roughness")?;
            let metallic = get_float_field(mat_json, "material", "metallic")?;
            let is_translucent = get_bool_field(mat_json, "material", "is_translucent")?;
            let mat = Material::new(albedo, ior, roughness, metallic, is_translucent);
            materials.push(mat);
        }
        Ok(materials)
    }

    fn load_meshes(&self, value: &serde_json::Value) -> Result<Vec<Vec<Rc<TriangleMesh>>>> {
        let arr = value
            .as_array()
            .context("top: 'meshes' should be an array")?;

        let mut meshes = Vec::with_capacity(arr.len());
        let mut mesh_index = 0;

        for mesh_json in arr {
            let file = mesh_json
                .as_str()
                .context("meshes: elements should be string")?;

            let mut obj_load_option = tobj::LoadOptions::default();
            obj_load_option.triangulate = true;
            obj_load_option.single_index = true;
            let (models, _) = tobj::load_obj(self.path.with_file_name(file), &obj_load_option)?;

            let mut meshes_temp = vec![];
            for model in models {
                let indices = model.mesh.indices;
                let vertex_count = model.mesh.positions.len() / 3;
                let mut vertices = vec![MeshVertex::default(); vertex_count];
                for i in 0..vertex_count {
                    let i0 = 3 * i;
                    let i1 = 3 * i + 1;
                    let i2 = 3 * i + 2;
                    if i2 < model.mesh.positions.len() {
                        vertices[i].position = Point3::new(
                            model.mesh.positions[i0],
                            model.mesh.positions[i1],
                            model.mesh.positions[i2],
                        );
                    }
                    if i2 < model.mesh.normals.len() {
                        vertices[i].normal = Vector3::new(
                            model.mesh.normals[i0],
                            model.mesh.normals[i1],
                            model.mesh.normals[i2],
                        );
                    }
                }

                let mesh = TriangleMesh::new(vertices, indices, mesh_index);
                mesh_index += 1;
                meshes_temp.push(Rc::new(mesh));
            }

            meshes.push(meshes_temp);
        }
        Ok(meshes)
    }

    fn load_objects(
        &self,
        value: &serde_json::Value,
    ) -> Result<(Vec<Triangle>, Vec<Matrix4<f32>>)> {
        let arr = value
            .as_array()
            .context("top: 'objects' should be an array")?;
        let mut triangles = vec![];
        let mut transforms = Vec::with_capacity(arr.len());
        for (obj_index, obj_json) in arr.iter().enumerate() {
            let trans = load_transform(obj_json, "object", "transform")?;
            let material = get_int_field(obj_json, "object", "material")?;
            let mesh_index_orig = get_int_array2_field(obj_json, "object", "mesh")?;
            let mesh = &self.meshes[mesh_index_orig[0] as usize][mesh_index_orig[1] as usize];

            let triangle_count = mesh.indices.len() / 3;
            for i in 0..triangle_count {
                let i0 = mesh.indices[3 * i] as usize;
                let i1 = mesh.indices[3 * i + 1] as usize;
                let i2 = mesh.indices[3 * i + 2] as usize;
                let index = triangles.len() as u32;
                triangles.push(Triangle::new(
                    mesh.clone(),
                    index,
                    [i0, i1, i2],
                    material,
                    &trans,
                    obj_index as u32,
                ));
            }

            transforms.push(trans);
        }
        Ok((triangles, transforms))
    }

    fn load_lights(&self, value: &serde_json::Value) -> Result<Vec<uniforms::Light>> {
        let arr = value
            .as_array()
            .context("top: 'lights' should be an array")?;
        let mut lights = Vec::with_capacity(arr.len());
        for light_json in arr {
            let ty = get_str_field(light_json, "light", "type")?;
            let light = match ty {
                "point" => {
                    let position = get_float_array3_field(light_json, "light-point", "position")?;
                    let strength = get_float_array3_field(light_json, "light-point", "strength")?;
                    uniforms::Light::point(position, strength)
                }
                "directional" => {
                    let direction =
                        get_float_array3_field(light_json, "light-directional", "direction")?;
                    let strength =
                        get_float_array3_field(light_json, "light-directional", "strength")?;
                    uniforms::Light::directional(direction, strength)
                }
                _ => bail!(format!("light: unknown type '{}'", ty)),
            };
            lights.push(light)
        }
        Ok(lights)
    }

    fn load_bvh(
        &self,
        value: &serde_json::Value,
        triangles: &mut Vec<Triangle>,
    ) -> Result<BvhAccel> {
        let max_leaf_size = get_int_field_or(value, "bvh", "max_leaf_size", 4)? as usize;
        let bucket_number = get_int_field_or(value, "bvh", "bucket_number", 16)? as usize;
        Ok(BvhAccel::new(triangles, max_leaf_size, bucket_number))
    }
}

fn get_bool_field(value: &serde_json::Value, env: &str, field: &str) -> Result<bool> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_bool()
        .context(format!("{}: '{}' should be a boolean", env, field))
}

fn get_str_field_or<'a>(
    value: &'a serde_json::Value,
    env: &str,
    field: &str,
    default: &'a str,
) -> Result<&'a str> {
    if let Some(_) = value.get(field) {
        get_str_field(value, env, field)
    } else {
        Ok(default)
    }
}

fn get_str_field<'a>(value: &'a serde_json::Value, env: &str, field: &str) -> Result<&'a str> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_str()
        .context(format!("{}: '{}' should be a string", env, field))
}

fn get_float_field(value: &serde_json::Value, env: &str, field: &str) -> Result<f32> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_f64()
        .map(|f| f as f32)
        .context(format!("{}: '{}' should be a float", env, field))
}

fn get_int_field_or(
    value: &serde_json::Value,
    env: &str,
    field: &str,
    default: u32,
) -> Result<u32> {
    if let Some(_) = value.get(field) {
        get_int_field(value, env, field)
    } else {
        Ok(default)
    }
}

fn get_int_field(value: &serde_json::Value, env: &str, field: &str) -> Result<u32> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    field_value
        .as_u64()
        .map(|f| f as u32)
        .context(format!("{}: '{}' should be an int", env, field))
}

fn get_int_array2_field(value: &serde_json::Value, env: &str, field: &str) -> Result<[u32; 2]> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    let error_info = format!("{}: '{}' should be an array with 2 ints", env, field);
    let arr = field_value.as_array().context(error_info.clone())?;
    if arr.len() == 2 {
        let arr0 = arr[0].as_u64().context(error_info.clone())? as u32;
        let arr1 = arr[1].as_u64().context(error_info.clone())? as u32;
        Ok([arr0, arr1])
    } else {
        bail!(error_info)
    }
}

fn get_float_array3_field(value: &serde_json::Value, env: &str, field: &str) -> Result<[f32; 3]> {
    let field_value = value
        .get(field)
        .context(format!("{}: no '{}' field", env, field))?;
    let error_info = format!("{}: '{}' should be an array with 3 floats", env, field);
    let arr = field_value.as_array().context(error_info.clone())?;
    if arr.len() == 3 {
        let arr0 = arr[0].as_f64().context(error_info.clone())? as f32;
        let arr1 = arr[1].as_f64().context(error_info.clone())? as f32;
        let arr2 = arr[2].as_f64().context(error_info.clone())? as f32;
        Ok([arr0, arr1, arr2])
    } else {
        bail!(error_info)
    }
}

fn load_transform(value: &serde_json::Value, env: &str, field: &str) -> Result<Matrix4<f32>> {
    let trans_json = value.get(field);
    if trans_json.is_none() {
        return Ok(Matrix4::identity());
    }
    let trans_json = trans_json.unwrap();

    let mut matrix = Matrix4::identity();
    if let Some(mat_json) = trans_json.get("matrix") {
        let error_info = format!("{}: 'matrix' should be an array with 16 floats", env);
        let mat_arr = mat_json.as_array().context(error_info.clone())?;
        if mat_arr.len() != 16 {
            bail!(error_info)
        }
        matrix.x.x = mat_arr[0].as_f64().context(error_info.clone())? as f32;
        matrix.x.y = mat_arr[1].as_f64().context(error_info.clone())? as f32;
        matrix.x.z = mat_arr[2].as_f64().context(error_info.clone())? as f32;
        matrix.x.w = mat_arr[3].as_f64().context(error_info.clone())? as f32;
        matrix.y.x = mat_arr[4].as_f64().context(error_info.clone())? as f32;
        matrix.y.y = mat_arr[5].as_f64().context(error_info.clone())? as f32;
        matrix.y.z = mat_arr[6].as_f64().context(error_info.clone())? as f32;
        matrix.y.w = mat_arr[7].as_f64().context(error_info.clone())? as f32;
        matrix.z.x = mat_arr[8].as_f64().context(error_info.clone())? as f32;
        matrix.z.y = mat_arr[9].as_f64().context(error_info.clone())? as f32;
        matrix.z.z = mat_arr[10].as_f64().context(error_info.clone())? as f32;
        matrix.z.w = mat_arr[11].as_f64().context(error_info.clone())? as f32;
        matrix.w.x = mat_arr[12].as_f64().context(error_info.clone())? as f32;
        matrix.w.y = mat_arr[13].as_f64().context(error_info.clone())? as f32;
        matrix.w.z = mat_arr[14].as_f64().context(error_info.clone())? as f32;
        matrix.w.w = mat_arr[15].as_f64().context(error_info.clone())? as f32;
    }
    if let Some(_) = trans_json.get("scale") {
        let scale = get_float_array3_field(trans_json, env, "scale")?;
        matrix = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]) * matrix;
    }
    if let Some(_) = trans_json.get("rotate") {
        let rotate = get_float_array3_field(trans_json, env, "rotate")?;
        matrix = Matrix4::from_angle_z(cgmath::Deg(rotate[2]))
            * Matrix4::from_angle_x(cgmath::Deg(rotate[0]))
            * Matrix4::from_angle_y(cgmath::Deg(rotate[1]))
            * matrix;
    }
    if let Some(_) = trans_json.get("translate") {
        let translate = get_float_array3_field(trans_json, env, "translate")?;
        matrix = Matrix4::from_translation(Vector3::new(translate[0], translate[1], translate[2]))
            * matrix;
    }
    if !matrix.is_invertible() {
        println!("WARNING: singular transform matrix found");
    }

    Ok(matrix)
}
