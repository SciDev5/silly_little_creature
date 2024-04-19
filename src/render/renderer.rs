use std::{any::Any, collections::HashMap, marker::PhantomData, mem, sync::mpsc};

use crate::util::Vec2I;
use glfw::Context;

pub trait Renderable {
    fn render(&self, glu: GLUtil);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct RenderWindow {
    window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,

    pos: Vec2I,
    dim: Vec2I,
}

impl RenderWindow {
    fn new(renderer: &mut Renderer, pos: Vec2I, dim: Vec2I) -> Self {
        let (mut window, events) = renderer
            .glfw
            .create_window(
                dim.x as u32,
                dim.y as u32,
                "silly little creature :3",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window");

        // Initialize OpenGL
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        // Configure the window
        window.make_current();
        let real_pos = renderer.screen_origin + pos;
        window.set_pos(real_pos.x, real_pos.y);
        renderer.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        // Initialize polling
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_cursor_pos_polling(true);

        window.set_size_polling(true);
        window.set_pos_polling(true);

        // Throw it all in a struct
        Self {
            window,
            events,
            pos,
            dim,
        }
    }

    fn handle_events(
        &mut self,
        self_id: RenderWindowId,
        extra_events: &mpsc::Sender<(RenderWindowId, glfw::WindowEvent)>,
    ) {
        while let Some((_, event)) = self.events.receive() {
            match event {
                glfw::WindowEvent::Size(x, y) => {
                    self.dim = Vec2I { x, y };
                }
                glfw::WindowEvent::Pos(x, y) => {
                    self.pos = Vec2I { x, y };
                }
                event => match extra_events.send((self_id, event)) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("failed to send window event because {}", e);
                    }
                },
            }
        }
    }

    fn render<'a>(
        &mut self,
        renderer: &Renderer,
        renderables: impl Iterator<Item = &'a Box<dyn Renderable>>,
    ) {
        // do OpenGL rendering stuff
        unsafe {
            // Clear the screen
            gl::Viewport(0, 0, self.dim.x, self.dim.y);
            gl::ClearColor(
                0.0,
                0.0,
                0.0,
                if renderer.debug_nontransparent_clear {
                    0.1
                } else {
                    0.0
                },
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Disable clipping based on the depth buffer.
            gl::Disable(gl::DEPTH_TEST);
            // Set up transparency
            gl::Enable(gl::BLEND);
            TransparencyMode::Normal.apply();
        }

        for renderable in renderables {
            renderable.render(GLUtil::new(renderer, &self));
        }

        self.window.swap_buffers();
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }
    pub fn raw_window(&self) -> &glfw::PWindow {
        &self.window
    }
    pub fn raw_window_mut(&mut self) -> &mut glfw::PWindow {
        &mut self.window
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderableId<T: Renderable>(u64, PhantomData<T>);
impl<T: Renderable + 'static> RenderableId<T> {
    pub fn get<'a>(&self, renderer: &'a Renderer) -> Option<&'a T> {
        renderer
            .renderables
            .get(&self.0)
            .and_then(|r| r.as_any().downcast_ref::<T>())
    }
    pub fn get_mut<'a>(&self, renderer: &'a mut Renderer) -> Option<&'a mut T> {
        renderer
            .renderables
            .get_mut(&self.0)
            .and_then(|r| r.as_any_mut().downcast_mut::<T>())
    }
}
#[derive(Debug, Clone, Copy)]
pub struct RenderWindowId(u64);
impl RenderWindowId {
    pub fn get(self, renderer: &Renderer) -> Option<&RenderWindow> {
        renderer.windows.get(&self.0)
    }
    pub fn get_mut(self, renderer: &mut Renderer) -> Option<&mut RenderWindow> {
        renderer.windows.get_mut(&self.0)
    }
    pub fn still_exists(self, renderer: &Renderer) -> bool {
        renderer.windows.contains_key(&self.0)
    }

    pub fn set_location(self, renderer: &mut Renderer, pos: Vec2I, dim: Vec2I) {
        let screen_origin = renderer.screen_origin;
        let Some(window) = renderer.windows.get_mut(&self.0) else { return; };
        let real_pos = screen_origin + pos;
        window.window.set_pos(real_pos.x, real_pos.y);
        window.window.set_size(dim.x, dim.y);
        window.pos = real_pos;
        window.dim = dim;
    }
}

pub struct Renderer {
    glfw: glfw::Glfw,

    /// Top left corner of the primary monitor (in pixels)
    screen_origin: Vec2I,
    /// Dimensions of the primary monitor (in pixels)
    screen_dim: Vec2I,

    next_renderable_id: u64,
    renderables: HashMap<u64, Box<dyn Renderable>>,
    next_window_id: u64,
    windows: HashMap<u64, RenderWindow>,

    pub debug_nontransparent_clear: bool,
}
impl Renderer {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        glfw.window_hint(glfw::WindowHint::Floating(false));
        glfw.window_hint(glfw::WindowHint::Decorated(false));
        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));

        let (screen_origin, screen_dim) = {
            let (mx, my, mw, mh) = glfw.with_primary_monitor(|_, m| {
                m.map_or_else(
                    || {
                        eprintln!("Could not get monitor details, assuming 1920x1080");
                        (0, 0, 1920, 1080)
                    },
                    |m| m.get_workarea(),
                )
            });
            (Vec2I::new(mx, my), Vec2I::new(mw, mh))
        };

        Self {
            glfw,

            screen_origin,
            screen_dim,

            next_renderable_id: 0,
            renderables: HashMap::new(),

            next_window_id: 0,
            windows: HashMap::new(),

            debug_nontransparent_clear: false,
        }
    }

    pub fn add_renderable<T: Renderable + 'static>(&mut self, r: T) -> RenderableId<T> {
        self.next_renderable_id += 1;
        if self
            .renderables
            .insert(self.next_renderable_id, Box::new(r))
            .is_some()
        {
            panic!("registered two renderables with the same id somehow");
        }
        RenderableId(self.next_renderable_id, PhantomData)
    }
    pub fn add_window(&mut self, pos: Vec2I, dim: Vec2I) -> RenderWindowId {
        let window = RenderWindow::new(self, pos + self.screen_origin, dim);
        self.next_window_id += 1;
        self.windows.insert(self.next_window_id, window);
        RenderWindowId(self.next_window_id)
    }

    /// Updates the engine's state and poll events.
    pub fn handle_events(
        &mut self,
        extra_events: &mpsc::Sender<(RenderWindowId, glfw::WindowEvent)>,
    ) {
        self.glfw.poll_events();
        let mut to_close = None;
        for (id, window) in &mut self.windows {
            window.handle_events(RenderWindowId(*id), extra_events);
            if window.should_close() {
                to_close.get_or_insert_with(|| Vec::new()).push(*id);
            }
        }
        for id in to_close.into_iter().flatten() {
            self.windows.remove(&id);
        }
    }

    /// Redraws all owned windows with all tracked renderables.
    pub fn render(&mut self) {
        // This extra temporary `windows` HashMap allows simultaneous mutable borrowing of
        // self and the contents of `self.windows`
        let mut windows = HashMap::new();
        mem::swap(&mut windows, &mut self.windows);

        for window in windows.values_mut() {
            window.render(&self, self.renderables.values());
        }

        mem::swap(&mut windows, &mut self.windows);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GLUtil {
    window_pos: Vec2I,
    window_dim: Vec2I,
}
impl GLUtil {
    fn new(
        Renderer { screen_origin, .. }: &Renderer,
        RenderWindow { pos, dim, .. }: &RenderWindow,
    ) -> Self {
        Self {
            window_pos: *pos - *screen_origin,
            window_dim: *dim,
        }
    }

    pub fn viewport(&self, mut pos: Vec2I, dim: Vec2I, relative_to: RelativeTo) {
        pos = match relative_to {
            RelativeTo::Window => pos,
            RelativeTo::Screen => pos - self.window_pos,
        };
        unsafe {
            gl::Viewport(pos.x, self.window_dim.y - pos.y - dim.y, dim.x, dim.y);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelativeTo {
    Window,
    Screen,
}
#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    TopLeft,
    BottomCenter,
    Center,
}
impl Anchor {
    pub fn apply(self, pos: Vec2I, dim: Vec2I) -> Vec2I {
        match self {
            Self::TopLeft => pos,
            Self::BottomCenter => Vec2I::new(pos.x - dim.x / 2, pos.y - dim.y),
            Self::Center => Vec2I::new(pos.x - dim.x / 2, pos.y - dim.y / 2),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TransparencyMode {
    Normal,
    Replace,
}
impl TransparencyMode {
    pub fn apply(self) {
        unsafe {
            match self {
                Self::Normal => {
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    gl::BlendEquation(gl::FUNC_ADD);
                }
                Self::Replace => {
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ZERO);
                    gl::BlendEquation(gl::FUNC_ADD);
                }
            }
        }
    }
}
