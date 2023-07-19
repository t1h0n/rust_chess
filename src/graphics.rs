use gl;
use gl::types::{GLenum, GLuint};
use nalgebra_glm as glm;
use stb_image::image::Image;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::{c_void, CStr, CString};
use std::fs;
use std::rc::Rc;

pub struct Shader {
    id: GLuint,
}
impl Shader {
    pub fn from_file(filename: &str, kind: gl::types::GLenum) -> Result<Shader, Box<dyn Error>> {
        let buf = fs::read(filename)?;
        let shader_source = unsafe { CString::from_vec_unchecked(buf) };
        Shader::from_source(shader_source.as_c_str(), kind)
    }
    pub fn from_source(source: &CStr, kind: gl::types::GLenum) -> Result<Shader, Box<dyn Error>> {
        let id = shader_from_source(source, kind)?;
        Ok(Shader { id })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn shader_from_source(source: &CStr, kind: gl::types::GLenum) -> Result<GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);
        let mut size: gl::types::GLsizei = 0;
        unsafe {
            gl::GetShaderInfoLog(id, len, &mut size, error.as_ptr() as *mut gl::types::GLchar);
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

pub struct ShaderProgram {
    id: GLuint,
    uniform_locations: HashMap<CString, i32>,
}

impl ShaderProgram {
    pub fn from_shaders(shaders: &[Shader]) -> Result<ShaderProgram, Box<dyn Error>> {
        let program_id = unsafe { gl::CreateProgram() };

        for shader in shaders {
            unsafe {
                gl::AttachShader(program_id, shader.id);
            }
        }

        unsafe {
            gl::LinkProgram(program_id);
        }

        // continue with error handling here

        for shader in shaders {
            unsafe {
                gl::DetachShader(program_id, shader.id);
            }
        }

        Ok(ShaderProgram {
            id: program_id,
            uniform_locations: HashMap::new(),
        })
    }
 
    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn hash_uniform_locations(&mut self, uniforms: &[&str]) {
        for uniform in uniforms {
            let name = CString::new(*uniform).unwrap();
            let location = unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) };
            self.uniform_locations.insert(name, location);
        }
    }
    fn retrieve_uniform_location(&self, name: &str) -> i32 {
        let name = CString::new(name).unwrap();
        if let Some(&location) = self.uniform_locations.get(&name) {
            return location;
        }
        unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) }
    }
    pub fn set_uniform_bool(&self, name: &str, value: bool) {
        let location = self.retrieve_uniform_location(name);
        unsafe {
            gl::Uniform1i(location, value.into());
        }
    }

    pub fn set_uniform_int(&self, name: &str, value: i32) {
        let location = self.retrieve_uniform_location(name);
        unsafe {
            gl::Uniform1i(location, value);
        }
    }
    pub fn set_uniform_float(&self, name: &str, value: f32) {
        let location = self.retrieve_uniform_location(name);
        unsafe {
            gl::Uniform1f(location, value);
        }
    }
    pub fn set_uniform_vec3f(&self, name: &str, value: glm::Vec3) {
        let location = self.retrieve_uniform_location(name);
        unsafe {
            gl::Uniform3fv(location, 1, value.as_ptr());
        }
    }
    pub fn set_uniform_mat4f(&self, name: &str, value: &glm::Mat4) {
        let location = self.retrieve_uniform_location(name);
        unsafe {
            gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr());
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}
pub struct Texture2D {
    id: GLuint,
    img: Rc<Image<u8>>,
}
impl Texture2D {
    pub fn new(img: Rc<Image<u8>>, img_format: GLenum) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                img_format.try_into().unwrap(),
                img.width as gl::types::GLsizei,
                img.height as gl::types::GLsizei,
                0,
                img_format,
                gl::UNSIGNED_BYTE,
                img.data.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR.try_into().unwrap(),
            );
        }
        Self { id, img }
    }
    pub fn get_id(&self) -> GLuint {
        self.id
    }
    pub fn get_image(&self) -> Rc<Image<u8>> {
        self.img.clone()
    }
}
impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
pub trait Drawable {
    fn draw(&self, projection: &glm::Mat4);
}
impl Drawable for Sprite {
    fn draw(&self, projection: &glm::Mat4) {
        let mvp = *projection * self.get_transform();
        self.shader.bind();
        if let Some(uniform_fn) = &self.uniform_setter {
            uniform_fn(self.shader.clone());
        }
        self.shader.set_uniform_mat4f("mvp", &mvp);
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindTexture(gl::TEXTURE_2D, self.texture.get_id());
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}
pub struct Sprite {
    pub shader: Rc<ShaderProgram>,
    pub texture: Rc<Texture2D>,
    pub rect: glm::Vec4,
    pub angle: f32,
    pub uniform_setter: Option<Box<dyn Fn(Rc<ShaderProgram>) -> ()>>,
    // opengl stuff
    vbo: GLuint,
    vao: GLuint,
}
impl Sprite {
    pub fn new(
        shader: Rc<ShaderProgram>,
        texture: Rc<Texture2D>,
        tex_rect: glm::Vec4,
        rect: glm::Vec4,
    ) -> Sprite {
        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;
        let img = texture.get_image();
        let w = img.width as f32;
        let h = img.height as f32;

        let x_1 = (tex_rect.x + tex_rect.z) / w;
        let x_0 = tex_rect.x / w;
        let y_0 = tex_rect.y / h;
        let y_1 = (tex_rect.y + tex_rect.w) / h;
        unsafe {
            let rect_vertices: [f32; 24] = [
                0.0, 1.0, x_0, y_0, // first triangle
                1.0, 1.0, x_1, y_0, //
                1.0, 0.0, x_1, y_1, //
                0.0, 1.0, x_0, y_0, // second triangle
                0.0, 0.0, x_0, y_1, //
                1.0, 0.0, x_1, y_1, //
            ];
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (rect_vertices.len() * std::mem::size_of::<f32>())
                    .try_into()
                    .unwrap(),
                rect_vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 4 * 4, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 4 * 4, (2 * 4) as *const c_void);
            gl::EnableVertexAttribArray(1);
            gl::BindVertexArray(0);
        }
        Self {
            shader,
            texture,
            rect,
            angle: 0.0,
            uniform_setter: None,
            vbo,
            vao,
        }
    }
    fn get_transform(&self) -> glm::Mat4 {
        let mut model = glm::translation(&glm::vec3(self.rect.x, self.rect.y, 0.0));
        if self.angle.is_normal() {
            model = glm::translate(
                &model,
                &glm::vec3::<f32>(0.5 * self.rect.z, 0.5 * self.rect.w, 0.0),
            );
            model = glm::rotate(
                &model,
                f32::to_radians(self.angle),
                &glm::vec3(0.0, 0.0, 1.0),
            );
            model = glm::translate(
                &model,
                &glm::vec3::<f32>(-0.5 * self.rect.z, -0.5 * self.rect.w, 0.0),
            );
        }
        model = glm::scale(&model, &glm::vec3::<f32>(self.rect.z, self.rect.w, 0.0));
        model
    }
}
impl Drop for Sprite {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

pub struct Rect {
    pub rect: glm::Vec4,
    pub angle: f32,
    pub shader: Rc<ShaderProgram>,
    pub uniform_setter: Option<Box<dyn Fn(Rc<ShaderProgram>) -> ()>>,
    // opengl stuff
    vbo: GLuint,
    vao: GLuint,
}

impl Rect {
    pub fn new(rect: glm::Vec4, shader: Rc<ShaderProgram>) -> Rect {
        let mut vao: gl::types::GLuint = 0;
        let mut vbo: gl::types::GLuint = 0;

        unsafe {
            let rect_vertices: [f32; 12] = [
                0.0, 1.0, // first triangle
                1.0, 1.0, //
                1.0, 0.0, //
                0.0, 1.0, // second triangle
                0.0, 0.0, //
                1.0, 0.0, //
            ];
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (rect_vertices.len() * std::mem::size_of::<f32>())
                    .try_into()
                    .unwrap(),
                rect_vertices.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * 4, std::ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::BindVertexArray(0);
        }
        Self {
            rect,
            shader,
            angle: 0.0,
            uniform_setter: None,
            vbo,
            vao,
        }
    }
    fn get_transform(&self) -> glm::Mat4 {
        let mut model = glm::translation(&glm::vec3(self.rect.x, self.rect.y, 0.0));
        if self.angle.is_normal() {
            model = glm::translate(
                &model,
                &glm::vec3::<f32>(0.5 * self.rect.z, 0.5 * self.rect.w, 0.0),
            );
            model = glm::rotate(
                &model,
                f32::to_radians(self.angle),
                &glm::vec3(0.0, 0.0, 1.0),
            );
            model = glm::translate(
                &model,
                &glm::vec3::<f32>(-0.5 * self.rect.z, -0.5 * self.rect.w, 0.0),
            );
        }
        model = glm::scale(&model, &glm::vec3::<f32>(self.rect.z, self.rect.w, 0.0));
        model
    }
}

impl Drawable for Rect {
    fn draw(&self, projection: &glm::Mat4) {
        let mvp = *projection * self.get_transform();
        self.shader.bind();
        if let Some(uniform_fn) = &self.uniform_setter {
            uniform_fn(self.shader.clone());
        }
        self.shader.set_uniform_mat4f("mvp", &mvp);
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}

impl Drop for Rect {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}
