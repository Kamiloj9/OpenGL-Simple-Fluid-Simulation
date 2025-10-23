use nalgebra_glm as glm;

pub enum CameraMovment {
    None = 0,
    Up = 1 << 0,
    Down = 1 << 1,
    Left = 1 << 2,
    Right = 1 << 3,
    Forward = 1 << 4,
    Backwards = 1 << 5,
}
pub struct Camera{
    pub position: glm::Vec3,
    pub direction: glm::Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub right: glm::Vec3,
    pub camera_control_state: u32,
}

impl Camera{
    pub fn new(position: glm::Vec3, pitch: f32, yaw: f32) -> Camera{
        let up: glm::Vec3 = glm::vec3(0.0, 1.0, 0.0);
        
        let yaw = yaw.to_radians();
        let pitch = pitch.to_radians();
        let dir_tmp: glm::Vec3 = glm::vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
        let direction: glm::Vec3 = dir_tmp;
        let direction = glm::normalize(&direction);
        let new_right =  glm::normalize(&direction.cross(&up)) * -1.0;
        Camera{
            position,
            direction,
            pitch,
            yaw,
            right: new_right,
            camera_control_state: 0
        }
    }

    pub fn update_rotation(&mut self, pitch: f32, yaw: f32){

        self.pitch = pitch;
        self.yaw = yaw;

        let up: glm::Vec3 = glm::vec3(0.0, 1.0, 0.0);

        let yaw = yaw.to_radians();
        let pitch = pitch.to_radians();
        let dir_tmp: glm::Vec3 = glm::vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
        self.direction = glm::normalize(&dir_tmp);
        self.right = glm::normalize(&self.direction.cross(&up)) * -1.0;
    }

    pub fn calculate_view_matrix(&self) -> [[f32; 4]; 4]{
        let up = glm::vec3(0.0, 1.0, 0.0);
        
        let f = {
            let f = self.direction;
            let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
            let len = len.sqrt();
            [f[0] / len, f[1] / len, f[2] / len]
        };
    
        let s = [up[1] * f[2] - up[2] * f[1],
                 up[2] * f[0] - up[0] * f[2],
                 up[0] * f[1] - up[1] * f[0]];
    
        let s_norm = {
            let len: f32 = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
            let len = len.sqrt();
            [s[0] / len, s[1] / len, s[2] / len]
        };
    
        let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
                 f[2] * s_norm[0] - f[0] * s_norm[2],
                 f[0] * s_norm[1] - f[1] * s_norm[0]];
    
        let p = [-self.position[0] * s_norm[0] - self.position[1] * s_norm[1] - self.position[2] * s_norm[2],
                 -self.position[0] * u[0] - self.position[1] * u[1] - self.position[2] * u[2],
                 -self.position[0] * f[0] - self.position[1] * f[1] - self.position[2] * f[2]];
    
        [
            [s_norm[0], u[0], f[0], 0.0],
            [s_norm[1], u[1], f[1], 0.0],
            [s_norm[2], u[2], f[2], 0.0],
            [p[0], p[1], p[2], 1.0],
        ]
    }
}
