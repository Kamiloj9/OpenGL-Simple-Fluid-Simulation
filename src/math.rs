use nalgebra_glm as glm;

pub fn create_projection_matrix(aspect_ratio: f32, fov: f32, zfar: f32, znear: f32) -> glm::Mat4{
    let f = 1.0 / (fov / 2.0).tan();

    let v0 = glm::vec4(f * aspect_ratio, 0.0, 0.0, 0.0);
    let v1 = glm::vec4(0.0, f, 0.0, 0.0);
    let v2 = glm::vec4(0.0, 0.0, (zfar+znear)/(zfar-znear), 1.0);
    let v3 = glm::vec4(0.0, 0.0, -(2.0*zfar*znear)/(zfar-znear), 0.0);
    glm::Mat4::new(
        v0[0], v0[1], v0[2], v0[3],
        v1[0], v1[1], v1[2], v1[3],
        v2[0], v2[1], v2[2], v2[3],
        v3[0], v3[1], v3[2], v3[3],
    )

    // [
    //     [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
    //     [         0.0         ,     f ,              0.0              ,   0.0],
    //     [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
    //     [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
    // ]
}

pub fn mat4_to_arr(arr: &glm::Mat4) -> [[f32;4];4]{
    [
        [arr[(0,0)], arr[(0,1)], arr[(0,2)], arr[(0,3)]],
        [arr[(1,0)], arr[(1,1)], arr[(1,2)], arr[(1,3)]],
        [arr[(2,0)], arr[(2,1)], arr[(2,2)], arr[(2,3)]],
        [arr[(3,0)], arr[(3,1)], arr[(3,2)], arr[(3,3)]],
    ]
}
