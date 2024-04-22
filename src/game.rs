use std::time::Duration;

use crate::{
    aware::creature::Creature,
    render::renderer::{RenderWindowId, Renderer},
    util::Vec2I,
};

pub struct Game {
    main_window: RenderWindowId,

    // hiding: Option<(ExtWindowInfo, Vec2I, Facing)>,
    creature: Creature,
}

impl Game {
    pub fn init(renderer: &mut Renderer) -> Self {
        Self {
            main_window: renderer.add_window(Vec2I::new(100, 100), Vec2I::new(500, 500)),
            creature: Creature::new(renderer),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.creature.update();
    }
    pub fn update_for_render(&mut self, renderer: &mut Renderer) {
        let new_window_rect = self.creature.update_for_render(renderer);
        self.main_window.set_location(renderer, new_window_rect);
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
                    glfw::Key::H if modifiers == glfw::Modifiers::Control => {
                        self.creature.hide();
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
    }
    pub fn on_mouse(
        &mut self,
        renderer: &mut Renderer,
        window: RenderWindowId,
        mouse_button: glfw::MouseButton,
        action: glfw::Action,
        modifiers: glfw::Modifiers,
    ) {
        self.creature.click();
    }

    pub fn running(&self, renderer: &Renderer) -> bool {
        self.main_window.still_exists(renderer)
    }
}
