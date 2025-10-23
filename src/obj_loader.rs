use glium::implement_vertex;
use nalgebra_glm as glm;


#[derive(Clone, Copy, Debug)]
pub struct Vertex{
    pub position: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct Face{
    pub vertex: [Vertex; 3]
}

impl Vertex {
    fn new() -> Vertex{
        Vertex{
            position: [0.0, 0.0, 0.0]
        }
    }
}

impl Face {
    fn new() -> Face{
        Face{
            vertex: [Vertex::new(), Vertex::new(), Vertex::new()]
        }
    }
}

pub struct Data{
    pub verticies: Vec<[f32; 3]>,
    pub normals: Vec<[f32;3]>,
    pub indices: Vec<u32>
}

#[derive(Clone, Copy)]
pub struct VertexData{
    pub position: [f32; 3],
    pub normal: [f32; 3],
}
implement_vertex!(VertexData, position, normal);

pub fn load(file_path: &str) -> Result<Vec<VertexData>, String>{
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(r) => return Err(r.to_string())
    };

    let mut ret = Vec::new();

    let mut vertexes = Vec::new();
    let mut faces = Vec::new();

    for line in content.lines(){
        if line.starts_with("v") {
            let mut v = Vertex::new();
            for (i, pos) in line[2..].split(" ").into_iter().enumerate(){
                if i > 2 {
                    return Err(String::from("Invalid obj file: too much vertex data"));
                }
                let parsed_pos: f32 = pos.parse().unwrap_or(0.0);
                
                v.position[i] = parsed_pos;
            }

            vertexes.push(v);
        }
        else if line.starts_with("f") {
            let mut face = Face::new();

            for (i, pos) in line[2..].split(" ").into_iter().enumerate(){
                if i > 2 {
                    return Err(String::from("Invalid obj file: too much face data"));
                }
                let index: u32 = pos.parse().unwrap_or(1) - 1;
                
                face.vertex[i] = vertexes[index as usize];
            }

            faces.push(face);
        }
    }

    for face in faces{
        // calculate normal
        let a = face.vertex[0];
        let b = face.vertex[1];
        let c = face.vertex[2];

        let a = glm::vec3(a.position[0], a.position[1], a.position[2]);
        let b = glm::vec3(b.position[0], b.position[1], b.position[2]);
        let c = glm::vec3(c.position[0], c.position[1], c.position[2]);

        let u = b - a;
        let v = c - a;

        let mut normal = glm::vec3(0.0, 1.0, 0.0);

        normal.x = (u.y * v.z) - (u.z * v.y);
        normal.y = (u.z * v.x) - (u.x * v.z);
        normal.z = (u.x * v.y) - (u.y * v.x);

        normal = glm::normalize(&normal);

        ret.push(VertexData{
            position: face.vertex[0].position,
            normal: [normal.x, normal.y, normal.z],
        });
        ret.push(VertexData{
            position: face.vertex[1].position,
            normal: [normal.x, normal.y, normal.z],
        });
        ret.push(VertexData{
            position: face.vertex[2].position,
            normal: [normal.x, normal.y, normal.z],
        });
    }

    Ok(ret)
}

pub fn load_from_file2(file_path: &str) -> Result<Data, String>{
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(r) => return Err(r.to_string())
    };

    let mut ret = Data { verticies: Vec::new(), indices: Vec::new(), normals: Vec::new() };

    

    for line in content.lines(){
        if line.starts_with("v") {
            let mut v: [f32; 3] = [0.0; 3];
            for (i, pos) in line[2..].split(" ").into_iter().enumerate(){
                if i > 2 {
                    return Err(String::from("Invalid obj file: too much vertex data"));
                }
                let parsed_pos: f32 = pos.parse().unwrap_or(0.0);
                
                v[i] = parsed_pos;
            }

            ret.verticies.push(v);
        }
        else if line.starts_with("f") {

            for (i, pos) in line[2..].split(" ").into_iter().enumerate(){
                if i > 2 {
                    return Err(String::from("Invalid obj file: too much face data"));
                }
                let index: u32 = pos.parse().unwrap_or(1) - 1;
                
                ret.indices.push(index);
            }

            
        }
    }

    if ret.indices.len() % 3 != 0{
        return Err(String::from("Invalid obj file: invalid indices len"));
    }
    println!("indices len {} ", ret.indices.len());

    for (_index, trangle) in ret.indices.chunks(3).enumerate(){
        
        let a = ret.verticies[trangle[0] as usize];
        let b = ret.verticies[trangle[1] as usize];
        let c = ret.verticies[trangle[2] as usize];

        let a = glm::vec3(a[0], a[1], a[2]);
        let b = glm::vec3(b[0], b[1], b[2]);
        let c = glm::vec3(c[0], c[1], c[2]);

        let u = b - a;
        let v = c - a;

        let mut normal = glm::vec3(0.0, 1.0, 0.0);

        normal.x = (u.y * v.z) - (u.z * v.y);
        normal.y = (u.z * v.x) - (u.x * v.z);
        normal.z = (u.x * v.y) - (u.y * v.x);

        normal = glm::normalize(&normal);

        ret.normals.push([normal.x, normal.y, normal.z]);
    }
    
    println!("verticies {} normals {}", ret.verticies.len(), ret.normals.len());

    Ok(ret)
}

pub fn load_from_file(file_path: &str) -> Result<Vec<Face>, String>{

    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(r) => return Err(r.to_string())
    };

    let mut vertexes = Vec::new();
    let mut ret = Vec::new();

    for line in content.lines(){
        if line.starts_with("v") {
            let mut v = Vertex::new();
            for (i, pos) in line[2..].split(" ").into_iter().enumerate(){
                if i > 2 {
                    return Err(String::from("Invalid obj file: too much vertex data"));
                }
                let parsed_pos: f32 = pos.parse().unwrap_or(0.0);
                
                v.position[i] = parsed_pos;
            }

            vertexes.push(v);
        }
        else if line.starts_with("f") {
            let mut face = Face::new();

            for (i, pos) in line[2..].split(" ").into_iter().enumerate(){
                if i > 2 {
                    return Err(String::from("Invalid obj file: too much face data"));
                }
                let index: u32 = pos.parse().unwrap_or(1) - 1;
                
                face.vertex[i] = vertexes[index as usize];
            }

            ret.push(face);
        }
    }

    Ok(ret)
}
