use std::time::Duration;

use crate::{
    include_imageasset,
    render::{
        renderer::{Anchor, RelativeTo, RenderWindowId, RenderableId, Renderer},
        sprite::Sprite,
    },
    util::Vec2I,
};

pub struct Game {
    main_window: RenderWindowId,
    cool_sprite: RenderableId<Sprite<1>>,
    t: f64,
}

impl Game {
    pub fn init(renderer: &mut Renderer) -> Self {
        Self {
            t: 0.0,
            main_window: renderer.add_window(Vec2I::new(100, 100), Vec2I::new(500, 500)),
            cool_sprite: renderer.add_renderable({
                let mut sprite = Sprite::new([include_imageasset!("assets/test_100x100.png")]);
                sprite.pos = (Vec2I::new(500, 500), RelativeTo::Screen, Anchor::TopLeft);
                sprite
            }),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.t += dt.as_secs_f64();
    }
    pub fn update_for_render(&mut self, renderer: &mut Renderer) {
        let t = self.t;
        self.cool_sprite.get_mut(renderer).unwrap().pos.0.y = (t.sin() * 200.0 + 200.0) as i32;
        self.main_window.set_location(
            renderer,
            Vec2I::new((t.sin() * 200.0 + 200.0) as i32, 10),
            Vec2I::new(500, 500),
        );
    }

    pub fn on_key(
        &mut self,
        renderer: &mut Renderer,
        window: RenderWindowId,
        key: glfw::Key,
        action: glfw::Action,
        modifiers: glfw::Modifiers,
    ) {
        if let glfw::Action::Press = action {
            if let Some(w) = window.get_mut(renderer) {
                let w = w.raw_window_mut();
                match key {
                    glfw::Key::D if modifiers == glfw::Modifiers::Control => {
                        let is_decorated = w.is_decorated();
                        w.set_decorated(!is_decorated);
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn on_cursor_pos(
        &mut self,
        renderer: &mut Renderer,
        window: RenderWindowId,
        pos: (f64, f64),
    ) {
        dbg!(pos);
    }
    pub fn on_mouse(
        &mut self,
        renderer: &mut Renderer,
        window: RenderWindowId,
        mouse_button: glfw::MouseButton,
        action: glfw::Action,
        modifiers: glfw::Modifiers,
    ) {
    }

    pub fn running(&self, renderer: &Renderer) -> bool {
        self.main_window.still_exists(renderer)
    }
}
