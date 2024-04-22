#![windows_subsystem = "windows"]

use std::sync::mpsc;

use game::Game;
use util::DeltaTimer;

mod aware;
mod game;
mod render;
mod util;

fn main() {
    // Initialize render engine
    let mut renderer = render::renderer::Renderer::new();

    let mut game = Game::init(&mut renderer);

    // renderer.debug_nontransparent_clear = true;

    // Initialize delta time
    let mut delta_timer = DeltaTimer::new();

    let (ev_send, ev_recv) = mpsc::channel();

    // Render loop
    while game.running(&renderer) {
        renderer.handle_events(&ev_send);

        while let Ok((window_id, ev)) = ev_recv.try_recv() {
            match ev {
                glfw::WindowEvent::Key(key, _scancode, action, modifiers) => {
                    game.on_key(&mut renderer, window_id, key, action, modifiers)
                }
                glfw::WindowEvent::MouseButton(mouse_button, action, modifiers) => {
                    game.on_mouse(&mut renderer, window_id, mouse_button, action, modifiers)
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    game.on_cursor_pos(&mut renderer, window_id, (x, y))
                }
                _ => {}
            }
        }
        game.update(delta_timer.tick());
        game.update_for_render(&mut renderer);
        renderer.render();
    }
}
