use std::time::{Duration, SystemTime};

use rand::Rng;

use crate::{
    include_imageasset,
    render::{
        renderer::{Anchor, RelativeTo, RenderableId, Renderer},
        sprite::Sprite,
    },
    util::{RectI, Vec2I},
};

use super::{
    ext_window::{iter_window_candidates, ExtWindowInfo},
    hiding::{find_hiding_spot_in_window, Facing},
};

pub struct Creature {
    /// Sprite textures:
    /// [
    ///     0: idle
    ///     1: idle+arms_raised
    ///     2: idle+talk
    ///     3: idle+talk+arms_raised
    ///     4: jump
    ///     5: peek_left
    ///     6: peek_right
    ///     7: peek_up
    ///     8: hidden
    ///     9: shocked
    /// ]
    sprite: RenderableId<Sprite<10>>,
    /// Text textures:
    /// [
    ///     0: hidden
    ///     1: "click me"
    ///     2: 0 catch taunt
    ///     3: 1+ catch taunt
    /// ]
    text_sprite: RenderableId<Sprite<4>>,

    state: CreatureState,

    last_pos: Vec2I,
    last_end_pos: Vec2I,

    screen_center: Vec2I,

    catch_count: u32,
    message_state: u32,
}

impl Creature {
    pub fn new(renderer: &mut Renderer) -> Self {
        Self {
            sprite: renderer.add_renderable(Sprite::new([
                include_imageasset!("../assets/creature/creature_idle.png"),
                include_imageasset!("../assets/creature/creature_idle_armsraised.png"),
                include_imageasset!("../assets/creature/creature_idle_talk.png"),
                include_imageasset!("../assets/creature/creature_idle_talk_armsraised.png"),
                include_imageasset!("../assets/creature/creature_jump.png"),
                include_imageasset!("../assets/creature/creature_peek_left.png"),
                include_imageasset!("../assets/creature/creature_peek_right.png"),
                include_imageasset!("../assets/creature/creature_peek_up.png"),
                include_imageasset!("../assets/empty.png"),
                include_imageasset!("../assets/creature/creature_shocked.png"),
            ])),
            text_sprite: renderer.add_renderable(Sprite::new([
                include_imageasset!("../assets/empty.png"),
                include_imageasset!("../assets/text/click_me.png"),
                include_imageasset!("../assets/text/0.png"),
                include_imageasset!("../assets/text/1.png"),
            ])),
            screen_center: renderer.center_pos(),
            last_pos: renderer.center_pos(),
            last_end_pos: renderer.center_pos(),
            state: CreatureState::Idle {
                pos: Some(renderer.center_pos()),
                arms_raised: true,
                try_talk_at: SystemTime::now()
                    .checked_add(Duration::from_millis(1000))
                    .unwrap(),
            },
            catch_count: 0,
            message_state: 0,
        }
    }

    pub fn hide(&mut self) {
        let top_window = iter_window_candidates().next().unwrap();
        let (p, f) = find_hiding_spot_in_window(&top_window);

        self.state = CreatureState::Jumping {
            from: Some(self.last_pos),
            to: Vec2I {
                x: self.last_pos.x,
                y: -100,
            },
            t_begin: SystemTime::now(),
            duration: Duration::from_millis(800),
            following_state: Box::new(CreatureState::Hiding {
                target_window: top_window,
                pos: p,
                facing: f,
                peek: false,
                peek_end_t: SystemTime::now()
                    .checked_add(Duration::from_millis(
                        rand::thread_rng().gen_range(5000..=10000),
                    ))
                    .unwrap(),
            }),
        }
    }
    pub fn click(&mut self) {
        match self.state {
            CreatureState::Hiding {
                peek: true,
                facing,
                pos,
                ..
            } => self.found(pos, facing),
            CreatureState::Idle { .. } | CreatureState::Talking { .. } => self.hide(),
            _ => {}
        }
    }
    fn found(&mut self, pos: Vec2I, facing: Facing) {
        self.catch_count += 1;
        self.message_state = 0;
        self.state = CreatureState::Shocked {
            from: pos,
            to: facing,
            t_begin: SystemTime::now(),
            following_state: Box::new(CreatureState::Jumping {
                from: None,
                to: self.screen_center,
                t_begin: SystemTime::now().checked_add(SHOCKED_TIME).unwrap(),
                duration: Duration::from_millis(500),
                following_state: Box::new(CreatureState::Idle {
                    pos: None,
                    arms_raised: false,
                    try_talk_at: SystemTime::now()
                        .checked_add(SHOCKED_TIME)
                        .unwrap()
                        .checked_add(Duration::from_millis(300))
                        .unwrap(),
                }),
            }),
        }
    }

    fn start_talking(&mut self) {
        let id = match (self.catch_count, self.message_state) {
            (0, 0) => {
                self.message_state = 1;
                2
            }
            (0, 1) => 1,
            (_, 0) => {
                self.message_state = 1;
                3
            }
            (_, 1) => 1,
            (_, _) => unreachable!(),
        };
        self.state = CreatureState::Talking {
            pos: self.last_pos,
            arms_raised: rand::thread_rng().gen_bool(0.65),
            t_begin_talking: SystemTime::now(),
            duration: Duration::from_millis(3000),
            id,
        };
    }

    pub fn update(&mut self) {
        match &mut self.state {
            CreatureState::Hiding {
                peek, peek_end_t, ..
            } => {
                if SystemTime::now() > *peek_end_t {
                    *peek = !*peek;
                    *peek_end_t = SystemTime::now()
                        .checked_add(Duration::from_millis(if *peek {
                            rand::thread_rng().gen_range(250..=750)
                        } else {
                            rand::thread_rng().gen_range(2000..=10000)
                            // rand::thread_rng().gen_range(50..=150)
                        }))
                        .unwrap();
                }
            }
            CreatureState::Idle { try_talk_at, .. } => {
                if SystemTime::now() > *try_talk_at {
                    self.start_talking();
                }
            }
            CreatureState::Talking {
                t_begin_talking,
                duration,
                ..
            } => {
                if t_begin_talking.elapsed().unwrap_or(Duration::ZERO) > *duration {
                    self.state = CreatureState::Idle {
                        pos: None,
                        arms_raised: false,
                        try_talk_at: SystemTime::now()
                            .checked_add(Duration::from_millis(
                                rand::thread_rng().gen_range(3000..6000),
                            ))
                            .unwrap(),
                    };
                    self.last_end_pos = self.last_pos;
                }
            }
            CreatureState::Jumping {
                t_begin,
                duration,
                following_state,
                ..
            } => {
                if t_begin.elapsed().unwrap_or(Duration::ZERO) > *duration {
                    self.state = (&**following_state).clone();
                    self.last_end_pos = self.last_pos;
                }
            }
            CreatureState::Shocked {
                t_begin,
                following_state,
                ..
            } => {
                if t_begin.elapsed().unwrap_or(Duration::ZERO) > SHOCKED_TIME {
                    self.state = (&**following_state).clone();
                    self.last_end_pos = self.last_pos;
                }
            }
        }
    }
    pub fn update_for_render(&mut self, renderer: &mut Renderer) -> RectI {
        {
            let text_sprite = self.text_sprite.get_mut(renderer).unwrap();
            text_sprite.pos.1 = RelativeTo::Absolute;
            text_sprite.pos.2 = Anchor::BottomCenter;
            text_sprite.set_current_tex_index(0);
        }
        {
            let sprite = self.sprite.get_mut(renderer).unwrap();
            sprite.pos.1 = RelativeTo::Absolute;
            sprite.pos.2 = Anchor::Center;
        }

        let sprite = self.sprite.get_mut(renderer).unwrap();
        match &self.state {
            CreatureState::Hiding {
                target_window,
                pos,
                facing,
                peek,
                ..
            } => {
                sprite.pos.0 = target_window.rect().pos + *pos;
                sprite.set_current_tex_index(if *peek {
                    match facing {
                        Facing::Left => 5,  // peek left
                        Facing::Right => 6, // peek right
                        _ => 7,             // peek up
                    }
                } else {
                    8 // hidden
                })
            }
            CreatureState::Idle {
                pos, arms_raised, ..
            } => {
                if let Some(pos) = *pos {
                    sprite.pos.0 = pos;
                }
                sprite.set_current_tex_index(match arms_raised {
                    false => 0,
                    true => 1,
                });
            }
            CreatureState::Talking {
                pos,
                arms_raised,
                t_begin_talking,
                id,
                ..
            } => {
                sprite.pos.0 = *pos;
                let mouth_open = t_begin_talking.elapsed().unwrap().as_millis() % 600 > 300;
                sprite.set_current_tex_index(match (arms_raised, mouth_open) {
                    (false, false) => 0,
                    (true, false) => 1,
                    (false, true) => 2,
                    (true, true) => 3,
                });

                let text_sprite = self.text_sprite.get_mut(renderer).unwrap();
                text_sprite.set_current_tex_index(*id);
            }
            CreatureState::Jumping {
                from,
                to,
                t_begin,
                duration,
                ..
            } => {
                let from = from.unwrap_or(self.last_end_pos);
                let u = (t_begin.elapsed().unwrap_or(Duration::ZERO).as_secs_f64()
                    / duration.as_secs_f64())
                .min(1.0);
                let v = 1.0 - u;
                const JUMPPOWER: f64 = 200.0;

                let p0 = (from.x as f64, from.y as f64);
                let p3 = (to.x as f64, to.y as f64);
                let p1 = (p0.0, p0.1 - JUMPPOWER * 3.0);
                let p2 = (p3.0, p3.1 - JUMPPOWER * 3.0);

                let x = (p0.0 * (v * v * v)
                    + 3.0 * p1.0 * (u * v * v)
                    + 3.0 * p2.0 * (u * u * v)
                    + p3.0 * (u * u * u)) as i32;
                let y = (p0.1 * (v * v * v)
                    + 3.0 * p1.1 * (u * v * v)
                    + 3.0 * p2.1 * (u * u * v)
                    + p3.1 * (u * u * u)) as i32;

                sprite.pos.0 = Vec2I {
                    x: x as i32,
                    y: y as i32,
                };
                sprite.set_current_tex_index(4);
            }
            CreatureState::Shocked {
                from, to, t_begin, ..
            } => {
                let t = (t_begin.elapsed().unwrap_or(Duration::ZERO).as_secs_f64()
                    / SHOCKED_TIME.as_secs_f64())
                .min(1.0);
                let l = (if to.is_horizontal() {
                    sprite.current_dims().x
                } else {
                    sprite.current_dims().y
                }) as f64;
                let tx = 2.0 * t - t * t;
                let ty = 4.0 * t - 4.0 * t * t;
                let (dx, dy) = match to {
                    Facing::Left => (-1.0, 0.0),
                    Facing::Right => (1.0, 0.0),
                    _ => (0.0, 1.0),
                };
                sprite.pos.0 = Vec2I {
                    x: ((tx * l * dx) + (0.5 * l * dx)) as i32,
                    y: -((ty * l) + (0.5 * tx * l * dy) + 0.5 * l) as i32,
                } + *from;
                sprite.set_current_tex_index(9);
            }
        }

        {
            let sprite = self.sprite.get_mut(renderer).unwrap();
            let pos = sprite.pos.0;
            self.last_pos = pos;
            let dim = sprite.current_dims();
            drop(sprite);
            let text_sprite = self.text_sprite.get_mut(renderer).unwrap();
            text_sprite.pos.0 = self.last_pos
                + Vec2I {
                    x: 0,
                    y: -dim.y * 12 / 20,
                };
            let text_dim = text_sprite.current_dims();
            let text_visible = text_dim.x > 10;
            let mut rect = RectI {
                pos: (pos - dim / 2),
                dim,
            };
            if text_visible {
                rect.extend_lr((text_dim.x - dim.x) / 2 + 5);
                rect.extend_up(dim.y / 10 + text_dim.y + 5);
            }
            rect
        }
    }
}

const SHOCKED_TIME: Duration = Duration::from_millis(500);
#[derive(Debug, Clone)]
enum CreatureState {
    Hiding {
        target_window: ExtWindowInfo,
        pos: Vec2I,
        facing: Facing,
        peek: bool,
        peek_end_t: SystemTime,
    },
    Idle {
        pos: Option<Vec2I>,
        arms_raised: bool,
        try_talk_at: SystemTime,
    },
    Talking {
        pos: Vec2I,
        arms_raised: bool,
        t_begin_talking: SystemTime,
        duration: Duration,
        id: usize,
    },
    Jumping {
        from: Option<Vec2I>,
        to: Vec2I,
        t_begin: SystemTime,
        duration: Duration,
        following_state: Box<CreatureState>,
    },
    Shocked {
        from: Vec2I,
        to: Facing,
        t_begin: SystemTime,
        following_state: Box<CreatureState>,
    },
}
