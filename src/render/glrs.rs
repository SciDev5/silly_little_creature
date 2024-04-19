use std::{
    ffi::{c_void, CString},
    fmt::Display,
};

use gl::types::{GLchar, GLenum, GLint, GLuint, GLvoid};

use crate::util::Vec2I;

use super::image_asset::ImageAsset;

#[derive(Debug, Clone, Copy)]
pub enum GLShaderType {
    Fragment,
    Vertex,
}
impl GLShaderType {
    fn gl_value(&self) -> GLuint {
        match self {
            Self::Fragment => gl::FRAGMENT_SHADER,
            Self::Vertex => gl::VERTEX_SHADER,
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::Fragment => "Fragment",
            Self::Vertex => "Vertex",
        }
    }
}

#[derive(Debug)]
pub enum GLError {
    ShaderCompilation { typ: GLShaderType, message: String },
    ShaderLinking { message: String },
}
impl Display for GLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ !!!!!!! ] ")?;
        match self {
            GLError::ShaderCompilation { typ, message } => {
                write!(f, "Shader/{} compile failed:\n{}", typ.name(), message)
            }
            GLError::ShaderLinking { message } => write!(f, "Shader link failed:\n{}", message),
        }
    }
}
impl std::error::Error for GLError {}

pub struct GLShader {
    sources: Vec<CString>,
    typ: GLShaderType,
    ref_id: u32,
}
impl GLShader {
    pub fn load(typ: GLShaderType, source: &str) -> Result<Self, GLError> {
        let mut shader = Self::new(typ);
        shader.source(source);
        shader.compile()?;
        Ok(shader)
    }
    pub fn new(typ: GLShaderType) -> Self {
        Self {
            sources: vec![],
            ref_id: unsafe { gl::CreateShader(typ.gl_value()) },
            typ,
        }
    }
    pub fn source(&mut self, source: &str) {
        unsafe {
            self.sources.push(CString::new(source).unwrap());
            gl::ShaderSource(
                self.ref_id,
                1,
                &self.sources.last().unwrap().as_ptr(),
                std::ptr::null(),
            );
        }
    }
    pub fn compile(&mut self) -> Result<(), GLError> {
        unsafe {
            gl::CompileShader(self.ref_id);

            let mut success: GLint = 0;
            gl::GetShaderiv(self.ref_id, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len: GLint = 0;
                gl::GetShaderiv(self.ref_id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
                gl::GetShaderInfoLog(
                    self.ref_id,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                buffer.set_len((len as usize).saturating_sub(1));
                Err(GLError::ShaderCompilation {
                    typ: self.typ,
                    message: std::str::from_utf8(&buffer).unwrap().to_string(),
                })
            } else {
                Ok(())
            }
        }
    }
}
impl Drop for GLShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.ref_id) }
    }
}

struct GLShaderProgram {
    ref_id: u32,
}
impl GLShaderProgram {
    fn new() -> Self {
        Self {
            ref_id: unsafe { gl::CreateProgram() },
        }
    }
}
impl Drop for GLShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.ref_id) }
    }
}
pub struct GLShaderProgramBuilder {
    obj: GLShaderProgram,
}
impl GLShaderProgramBuilder {
    pub fn new() -> Self {
        Self {
            obj: GLShaderProgram::new(),
        }
    }
    pub fn attatch_shader(&self, shader: &GLShader) {
        unsafe { gl::AttachShader(self.obj.ref_id, shader.ref_id) }
    }
    pub fn link(self) -> Result<GLShaderProgramLinked, GLError> {
        unsafe {
            gl::LinkProgram(self.obj.ref_id);

            let mut success: GLint = 0;
            gl::GetProgramiv(self.obj.ref_id, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len: GLint = 0;
                gl::GetProgramiv(self.obj.ref_id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
                gl::GetProgramInfoLog(
                    self.obj.ref_id,
                    len,
                    std::ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                buffer.set_len((len as usize).saturating_sub(1));
                Err(GLError::ShaderLinking {
                    message: std::str::from_utf8(&buffer).unwrap().to_string(),
                })
            } else {
                Ok(GLShaderProgramLinked { obj: self.obj })
            }
        }
    }
}
pub struct GLShaderProgramLinked {
    obj: GLShaderProgram,
}
impl GLShaderProgramLinked {
    pub fn use_for_draw(&self) {
        unsafe { gl::UseProgram(self.obj.ref_id) }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GLTexPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Default for GLTexPixel {
    fn default() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}
pub struct GLTexture2d {
    ref_id: GLuint,
    dims: Vec2I,
}
impl GLTexture2d {
    pub fn new(image_asset: &ImageAsset) -> Self {
        let mut ref_id = 0;
        unsafe {
            gl::GenTextures(1, &mut ref_id);
            gl::BindTexture(gl::TEXTURE_2D, ref_id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        }
        let mut s = Self {
            ref_id,
            dims: Vec2I::new(0, 0),
        };
        s.data_from_imageasset(image_asset);
        s
    }
    pub fn bind(&self, slot: GLTextureSlot, location: GLint) {
        unsafe {
            gl::ActiveTexture(slot.gl_enum());
            gl::BindTexture(gl::TEXTURE_2D, self.ref_id);
            gl::Uniform1i(location, slot.gl_int());
        }
    }
    pub fn update_partial<const WS: usize, const HS: usize>(
        &self,
        x0: usize,
        y0: usize,
        data: [[GLTexPixel; WS]; HS],
    ) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.ref_id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x0 as GLint,
                y0 as GLint,
                WS as GLint,
                HS as GLint,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            )
        }
    }
    pub fn data_from_color(&self, color: GLTexPixel) {
        self.tex_image_2d(
            vec![color; self.dims.x as usize * self.dims.y as usize].as_ptr() as *const GLvoid,
        );
    }
    pub fn data_from_imageasset(&mut self, data: &ImageAsset) {
        self.dims = data.get_dimensions();
        self.tex_image_2d(data.pixels().as_ptr() as *const GLvoid);
    }
    fn tex_image_2d(&self, data: *const GLvoid) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.ref_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                self.dims.x,
                self.dims.y,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data,
            )
        }
    }
    pub fn get_dimensions(&self) -> Vec2I {
        self.dims
    }
}
impl Drop for GLTexture2d {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.ref_id) }
    }
}
pub enum GLTextureSlot {
    Tex0,
}
impl GLTextureSlot {
    fn gl_enum(&self) -> GLenum {
        match self {
            Self::Tex0 => gl::TEXTURE0,
        }
    }
    fn gl_int(&self) -> GLint {
        match self {
            Self::Tex0 => 0,
        }
    }
}

pub struct TriPosVO<const N: usize> {
    tris: [[[f32; 2]; 3]; N],
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
}
impl<const N: usize> TriPosVO<N> {
    pub fn new(tris: [[[f32; 2]; 3]; N]) -> Self {
        let mut self_ = Self {
            vbo: 0,
            vao: 0,
            tris,
        };
        // Create and bind the vertex buffer
        unsafe {
            gl::GenBuffers(1, &mut self_.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self_.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (N * (3 * 2) * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &self_.tris[0][0][0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        // Create and bind the vertex array object
        unsafe {
            gl::GenVertexArrays(1, &mut self_.vao);
            gl::BindVertexArray(self_.vao);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (2 * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
        }

        self_
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
        }
    }
}
impl<const N: usize> Drop for TriPosVO<N> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

pub struct F32VO<const N: usize, const S: usize> {
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
    pub data: [[f32; S]; N],
}
impl<const L: usize, const S: usize> F32VO<L, S> {
    pub fn new(data: [[f32; S]; L]) -> Self {
        let mut self_ = Self {
            vbo: 0,
            vao: 0,
            data,
        };
        // Create and bind the vertex buffer
        unsafe {
            gl::GenBuffers(1, &mut self_.vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self_.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (L * S * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &data[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        // Create and bind the vertex array object
        unsafe {
            gl::GenVertexArrays(1, &mut self_.vao);
            gl::BindVertexArray(self_.vao);
            gl::VertexAttribPointer(
                0,
                S as gl::types::GLint,
                gl::FLOAT,
                gl::FALSE,
                (S * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
        }

        self_
    }
    pub fn update(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (L * S * std::mem::size_of::<gl::types::GLfloat>()) as gl::types::GLsizeiptr,
                &self.data[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW,
            );
        }
    }
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::EnableVertexAttribArray(0);
        }
    }
}
impl<const L: usize, const S: usize> Drop for F32VO<L, S> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
