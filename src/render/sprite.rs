use crate::util::Vec2I;

use super::{
    glrs,
    image_asset::ImageAsset,
    renderer::{Anchor, GLUtil, RelativeTo, Renderable, TransparencyMode},
};

const FULL_QUAD_TRIS: [[[f32; 2]; 3]; 2] = [
    [[1.0, 1.0], [-1.0, 1.0], [1.0, -1.0]],
    [[-1.0, -1.0], [1.0, -1.0], [-1.0, 1.0]],
];
pub struct Sprite<const N: usize> {
    vo: glrs::TriPosVO<2>,
    shaders: glrs::GLShaderProgramLinked,
    textures: [glrs::GLTexture2d; N],
    pub pos: (Vec2I, RelativeTo, Anchor),
    current_tex: usize,
}
impl<const N: usize> Sprite<N> {
    pub fn new(image_assets: [ImageAsset; N]) -> Self {
        Self {
            shaders: {
                let builder = glrs::GLShaderProgramBuilder::new();
                let vert = glrs::GLShader::load(
                    glrs::GLShaderType::Vertex,
                    include_str!("shader/sprite.vsh"),
                )
                .unwrap();
                let frag = glrs::GLShader::load(
                    glrs::GLShaderType::Fragment,
                    include_str!("shader/sprite.fsh"),
                )
                .unwrap();

                builder.attatch_shader(&vert);
                builder.attatch_shader(&frag);

                builder.link().unwrap()
            },
            vo: glrs::TriPosVO::new(FULL_QUAD_TRIS),
            textures: std::array::from_fn(|i| glrs::GLTexture2d::new(&image_assets[i])),
            current_tex: 0,
            pos: (Vec2I::new(0, 0), RelativeTo::Window, Anchor::TopLeft),
        }
    }

    pub fn set_tex(&mut self, i: usize, image_asset: &ImageAsset) {
        if i < N {
            self.textures[i].data_from_imageasset(image_asset);
        } else {
            eprintln!(
                "attempted to set texture with index greater than the number of image assets"
            );
        }
    }
    pub fn set_current_tex_index(&mut self, i: usize) {
        if i < N {
            self.current_tex = i;
        } else {
            eprintln!(
                "attempted to set texture with index greater than the number of image assets"
            );
        }
    }
}

impl<const N: usize> Renderable for Sprite<N> {
    fn render(&self, glu: GLUtil) {
        let (pos, relative_to, anchor) = self.pos;
        let dim = self.textures[self.current_tex].get_dimensions();
        glu.viewport(anchor.apply(pos, dim), dim, relative_to);
        TransparencyMode::Normal.apply();

        self.shaders.use_for_draw();
        self.vo.bind();
        self.textures[self.current_tex].bind(glrs::GLTextureSlot::Tex0, 1);
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 3 * FULL_QUAD_TRIS.len() as i32);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
