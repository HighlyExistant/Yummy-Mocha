#![allow(unused)]
/// This is probably one of the ugliest coded files on this entire project
/// so I need to revise it a bunch.
use std::{vec, io::Write, collections::HashMap, rc::Rc};

use drowsed_math::{FVec3, TransformQuaternion3D, complex::quaternion::Quaternion};
use fbxcel_dom::{fbxcel::tree::v7400::NodeHandle, v7400::Document, any::AnyDocument};

pub fn print_format(node: NodeHandle, depth: i32) {
    let new_depth = depth + 1;
    for child in node.children() {
        for i in 0..depth {
            print!("\t");
        }
        println!("{}: {:?}", child.name(), child.attributes());
        print_format(child, new_depth);
    }
}
pub fn print_format_file(node: NodeHandle, depth: i32, file: &mut std::fs::File) {
    let new_depth = depth + 1;
    for child in node.children() {
        for i in 0..depth {
            file.write("\t".as_bytes()).unwrap();
        }

        file.write_fmt(format_args!("{}: {:?}\n", child.name(), child.attributes())).unwrap();
        print_format_file(child, new_depth, file);
    }
}
#[derive(Default, Debug)]
pub enum GeometryNormal {
    #[default]
    None,
    ByPolygonVertex(Vec<FVec3>),
}
#[derive(Default, Debug)]
pub struct Material {
    tag: String,
    diffuse: FVec3 // Diffuse indices
}
#[derive(Default)]
pub struct Geometry {
    tag: String,
    vertices: Vec<FVec3>,
    polygon_indices: Vec<i32>,
    edges: Vec<i32>,
    normal: GeometryNormal,
}
pub struct ModelData {
    tag: String,
    transform: TransformQuaternion3D,
}
#[derive(Default)]
pub struct StandardModelData {
    pub tag: String,
    pub vertices: Vec<FVec3>,
    pub normals: Vec<FVec3>,
    pub indices: Vec<u32>,
    pub materials: Vec<Rc<Material>>,
    pub transform: TransformQuaternion3D,
}

impl StandardModelData {
    pub fn new(filepath: &str) -> Vec<Self> {
        let file = std::fs::File::open(filepath).expect("Failed to open file");
        let reader = std::io::BufReader::new(file);
        match AnyDocument::from_seekable_reader(reader).expect("Failed to load document") {
        AnyDocument::V7400(_fbx_ver, doc) => {
            return Self::parse(doc);
        }
        _ => panic!("Got FBX document of unsupported version"),
    }
    }
    pub fn parse(document: Box<Document>) -> Vec<Self> {
        let tree = document.tree();
        let root = tree.root();

        // Parse Data into seperate Geometries, Models and Materials.

        let mut _objects: (HashMap::<i64, Geometry>, HashMap::<i64,ModelData>, HashMap::<i64,Rc<Material>>) = (HashMap::new(), HashMap::new(), HashMap::new());
        let mut _connections: Vec<(i64, i64)> = vec![];
        for child in root.children() {
            match child.name() {
                "Objects" => {
                    _objects = Self::parse_objects(&child);
                }
                "Connections" => {
                    _connections = Self::parse_connections(&child);
                }
                _ => {}
            }
        }
        let mut model_sheis:Vec<StandardModelData> = Vec::with_capacity(_objects.1.len());
        let mut model_sheis_idx:Vec<i64> = Vec::with_capacity(_objects.1.len());

        for (idx, model) in _objects.1.iter() {
            model_sheis.push(StandardModelData {
                tag: model.tag.clone(),
                transform: model.transform,
                ..Default::default()
            });
            model_sheis_idx.push(*idx);
        }
        
        for connection in _connections {
            // if rhs == 0 it means its just initializing a model and we can just skip.
            // this is most likely a bad approach but ive been writing this for like 9 hours
            // now and im kinda sick of all this.
            if connection.1 == 0 {
                continue;
            }
            for i in 0..model_sheis.len() {
                if model_sheis_idx[i] == connection.1 {
                    // parse geometry data
                    if _objects.0.contains_key(&connection.0) {
                        let geometry = _objects.0.get(&connection.0).unwrap();
                        model_sheis[i].indices = Self::get_indices(geometry);

                        model_sheis[i].vertices = geometry.vertices.clone();
                        
                        match &geometry.normal {
                            GeometryNormal::ByPolygonVertex(v) => {
                                model_sheis[i].normals = vec![FVec3::from(0.0); model_sheis[i].vertices.len()];
                                let mut j = 0;
                                for idx in 0..(model_sheis[i].indices.len()-1) {
                                    let index = model_sheis[i].indices[idx] as usize;
                                    if model_sheis[i].normals[index] == 0.0 {
                                        let val = v[j];
                                        model_sheis[i].normals[index] = val;
                                    }
                                    j += 1;
                                }
                            } 
                            GeometryNormal::None => {}
                        }
                    } 
                    // parse material data. Keep in mind there can be multiple material data for 1 model.
                    else if _objects.2.contains_key(&connection.0) {
                        let material = _objects.2.get(&connection.0).unwrap();
                        model_sheis[i].materials.push(material.clone());
                    }
                }
            }
        }
        model_sheis
    }
    fn get_indices(geometry: &Geometry) -> Vec<u32>{
        let indices: Vec<u32> = geometry.polygon_indices.iter().map(|index| {
            if index.is_negative() {
                !*index as u32
            } else {
                *index as u32
            }
        }).collect();
        indices
    }
    fn parse_objects(node: &NodeHandle) -> (HashMap::<i64, Geometry>, HashMap::<i64,ModelData>, HashMap::<i64,Rc<Material>>) {
        let mut geometries = HashMap::<i64, Geometry>::new();
        let mut models = HashMap::<i64,ModelData>::new();
        let mut materials = HashMap::<i64,Rc<Material>>::new();

        for child in node.children() {
            match child.name() {
                "Geometry" => {
                    let (id, geo) = Self::parse_geometry(&child);
                    geometries.insert(id, geo);
                }
                "Model" => {
                    let (id, model) = Self::parse_model(&child);
                    models.insert(id, model);
                }
                "Material" => {
                    let (id, material) = Self::parse_material(&child);
                    materials.insert(id, material);
                }
                _ => {}
            }
        }
        (geometries, models, materials)
    }
    fn parse_geometry(node: &NodeHandle) -> (i64, Geometry) {
        let mut geo = Geometry {
            tag: String::new(),
            vertices: vec![],
            polygon_indices: vec![],
            edges: vec![],
            normal: GeometryNormal::None,
        };
        let collection_id = node.attributes()[0].get_i64().unwrap();
        geo.tag = node.attributes()[1].get_string().unwrap().into();
        for child in node.children() {
            match child.name() {
                "Vertices" => {
                    let attribute = child.attributes();
                    let verticesf64 = attribute[0].get_arr_f64().unwrap();
                    let mut verticesf32 = Vec::<FVec3>::with_capacity(verticesf64.len() / 3);
                    let mut i = 0;
                    while (i < verticesf64.len()) {
                        verticesf32.push(FVec3::new(
                            verticesf64[i] as f32, 
                            verticesf64[i + 1] as f32, 
                            verticesf64[i+ 2] as f32)
                        );
                        i += 3;
                    }
                    geo.vertices = verticesf32;
                }
                "PolygonVertexIndex" => {
                    let attribute = child.attributes();
                    let indicesi32 = attribute[0].get_arr_i32().unwrap().to_vec();
                    geo.polygon_indices = indicesi32;
                }
                "Edges" => {
                    let attribute = child.attributes();
                    let edges = attribute[0].get_arr_i32().unwrap().to_vec();
                    geo.edges = edges;
                }
                "LayerElementNormal" => {
                    let mut normals = Vec::<f64>::new();
                    let mut mapping = String::new();
                    for element in child.children() {
                        match element.name() {
                            "MappingInformationType" => {
                                mapping = element.attributes()[0].get_string().unwrap().into();
                            }
                            "Normals" => {
                                normals = element.attributes()[0].get_arr_f64().unwrap().to_vec();
                            }
                            _ => {}
                        }
                    }
                    if mapping == "ByPolygonVertex" {
                        let mut normalsf32 = Vec::<FVec3>::with_capacity(normals.len() / 3);
                        let mut i = 0;
                        while (i < normals.len()) {
                            normalsf32.push(FVec3::new(
                                normals[i] as f32, 
                                normals[i + 1] as f32, 
                                normals[i+ 2] as f32)
                            );
                            i += 3;
                        }
                        geo.normal = GeometryNormal::ByPolygonVertex(normalsf32);
                    }
                }
                _ => {}
            }
        }
        (collection_id, geo)
    }
    fn parse_model(node: &NodeHandle) -> (i64, ModelData) {
        let mut model = ModelData {
            tag: String::new(),
            transform: TransformQuaternion3D::default(),
        };
        let collection_id = node.attributes()[0].get_i64().unwrap();
        model.tag = node.attributes()[1].get_string().unwrap().into();
        for child in node.children() {
            match child.name() {
                "Properties70" => {
                    for property in child.children() {
                        let attribute_name = property.attributes()[0].get_string().unwrap();
                        match attribute_name {
                            "Translation" => {
                                model.transform.translation.x = property.attributes()[4].get_f64().unwrap() as f32;
                                model.transform.translation.y = property.attributes()[5].get_f64().unwrap() as f32;
                                model.transform.translation.z = property.attributes()[6].get_f64().unwrap() as f32;
                            }
                            "Rotation" => {
                                let euler = FVec3::new(
                                    property.attributes()[4].get_f64().unwrap() as f32, 
                                    property.attributes()[5].get_f64().unwrap() as f32, 
                                    property.attributes()[6].get_f64().unwrap() as f32,
                                );
                                model.transform.rotation = Quaternion::from_euler(euler);
                                // model.transform.rotation.x = property.attributes()[4].get_f64().unwrap() as f32;
                                // model.transform.rotation.y = property.attributes()[5].get_f64().unwrap() as f32;
                                // model.transform.rotation.z = property.attributes()[6].get_f64().unwrap() as f32;
                            }
                            "Scaling" => {
                                model.transform.scale.x = property.attributes()[4].get_f64().unwrap() as f32;
                                model.transform.scale.y = property.attributes()[5].get_f64().unwrap() as f32;
                                model.transform.scale.z = property.attributes()[6].get_f64().unwrap() as f32;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        (collection_id, model)
    }
    fn parse_material(node: &NodeHandle) -> (i64, Rc<Material>) {
        let mut material = Material {
            tag: String::new(),
            diffuse: FVec3::from(0.0),
        };

        let collection_id = node.attributes()[0].get_i64().unwrap();
        material.tag = node.attributes()[1].get_string().unwrap().into();

        for child in node.children() {
            match child.name() {
                "Properties70" => {
                    for property in child.children() {
                        let attribute_name = property.attributes()[0].get_string().unwrap();
                        match attribute_name {
                            "DiffuseColor" => {
                                material.diffuse.x = property.attributes()[4].get_f64().unwrap() as f32;
                                material.diffuse.y = property.attributes()[5].get_f64().unwrap() as f32;
                                material.diffuse.z = property.attributes()[6].get_f64().unwrap() as f32;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        (collection_id, Rc::new(material))
    }
    /// # parse_connections
    ///
    /// Everything in fbx gets assigned to a model.
    /// At the very start of connections the models get assigned to
    /// id: 0 to initialize them. After this different nodes such as
    /// geometry and material nodes get added to the model via connections.
    /// the way they get assigned is lhs -> rhs. 
    /// 
    /// ### Note: This could very well be wrong since I just gathered this information
    /// ### from parsing various fbx files and looking at the similarities and making assumptions.
    fn parse_connections(node: &NodeHandle) -> Vec<(i64, i64)> {
        let mut connections: Vec<(i64, i64)> = vec![];
        for child in node.children() {
            let attributelist = child.attributes();
            let lhs = attributelist[1].get_i64().unwrap();
            let rhs = attributelist[2].get_i64().unwrap();
            connections.push((lhs, rhs));
        }
        connections
    }
}