use image::{ImageBuffer, Luma};
use imageproc::edges::canny;
use rand::Rng;

use crate::util::{SwitchRev, Vec2I};

use super::ext_window::ExtWindowInfo;

const CREATURE_HIDE_SAFE_ZONE_DIMS: Vec2I = Vec2I { x: 20, y: 40 };

pub fn find_hiding_spot_in_window(window: &ExtWindowInfo) -> (Vec2I, Facing) {
    const FIRST_PASS_SCALEDOWN: u32 = 4;

    fn find_aaline(img: ImageBuffer<Luma<u8>, Vec<u8>>, tries: usize) -> Option<(Vec2I, Facing)> {
        const XMAR: u32 = CREATURE_HIDE_SAFE_ZONE_DIMS.x as u32 / FIRST_PASS_SCALEDOWN;
        const YMAR: u32 = CREATURE_HIDE_SAFE_ZONE_DIMS.y as u32 / FIRST_PASS_SCALEDOWN;
        const SAFETY_MAR: u32 = 2;
        let mut rng = rand::thread_rng();
        let horiz_to_start = rng.gen_bool(0.5);
        for i in 0..tries {
            let p0 = (
                rng.gen_range(XMAR + SAFETY_MAR..(img.width() - XMAR - SAFETY_MAR)),
                rng.gen_range(YMAR + SAFETY_MAR..(img.height() - YMAR - SAFETY_MAR)),
            );
            let horiz = (i % 2 == 0) == horiz_to_start;
            let rev = rng.gen_bool(0.5);

            let (iter, safe_zone_size) = if horiz {
                (
                    (p0.0..(img.width() - XMAR)).chain(XMAR..p0.0),
                    CREATURE_HIDE_SAFE_ZONE_DIMS.y / FIRST_PASS_SCALEDOWN as i32,
                )
            } else {
                (
                    (p0.1..(img.height() - YMAR)).chain(YMAR..p0.1),
                    CREATURE_HIDE_SAFE_ZONE_DIMS.x / FIRST_PASS_SCALEDOWN as i32,
                )
            };
            let mut iter = SwitchRev::conditional_reverse(iter, rev);
            let _ = iter.next(); // consume the first element, because it's easier to do that here

            #[inline]
            fn is_on(pix: Luma<u8>) -> bool {
                pix.0[0] > 0
            }
            let mut n_edge = if is_on(img[p0]) { 1 } else { 0 };
            let mut edge_start = if horiz { p0.0 } else { p0.1 };

            for i in iter {
                let p = if horiz { (i, p0.1) } else { (p0.0, i) };

                if is_on(img[p]) {
                    n_edge += 1;
                    continue;
                } else if n_edge > 0 && n_edge <= 2 {
                    fn pn(horiz: bool, p0: (u32, u32), i: u32, j: i32) -> (u32, u32) {
                        if horiz {
                            (i, p0.1.checked_add_signed(j).unwrap())
                        } else {
                            (p0.0.checked_add_signed(j).unwrap(), i)
                        }
                    }
                    let i0 = edge_start;
                    let irange = (i.min(i0) + 1)..=(i.max(i0) - 1);
                    let mut b = (safe_zone_size - 1, safe_zone_size - 1);
                    for j in 1..safe_zone_size {
                        if is_on(img[pn(horiz, p0, i0, j)])
                            || is_on(img[pn(horiz, p0, i, j)])
                            || irange.clone().any(|ii| !is_on(img[pn(horiz, p0, ii, j)]))
                        {
                            b.0 = j - 1;
                            break;
                        }
                    }
                    for j in 1..safe_zone_size {
                        if is_on(img[pn(horiz, p0, i0, j)])
                            || is_on(img[pn(horiz, p0, i, j)])
                            || irange.clone().any(|ii| !is_on(img[pn(horiz, p0, ii, j)]))
                        {
                            b.1 = j - 1;
                            break;
                        }
                    }
                    if b.0 + b.1 >= safe_zone_size {
                        let i = (i + i0) / 2;
                        let j = (b.1 - b.0) / 2;
                        let p = pn(horiz, p0, i, j);
                        let facing = if horiz {
                            if rev {
                                Facing::Left
                            } else {
                                Facing::Right
                            }
                        } else {
                            if rev {
                                Facing::Up
                            } else {
                                Facing::Down
                            }
                        };
                        return Some((
                            Vec2I {
                                x: p.0 as i32,
                                y: p.1 as i32,
                            },
                            facing,
                        ));
                    }
                }
                // else
                edge_start = i;
                n_edge = 0
            }
        }
        None
    }

    let img_resized = window
        .img()
        .resize(
            window.img().width() / FIRST_PASS_SCALEDOWN,
            window.img().height() / FIRST_PASS_SCALEDOWN,
            image::imageops::FilterType::Nearest,
        )
        .to_luma8();

    let detect_mini = canny(&img_resized, 0.5, 10.0);

    let (loc, facing) = find_aaline(detect_mini, 100).unwrap();

    (
        Vec2I {
            x: loc.x * FIRST_PASS_SCALEDOWN as i32 * window.img().width() as i32
                / window.rect().dim.x,
            y: loc.y * FIRST_PASS_SCALEDOWN as i32 * window.img().height() as i32
                / window.rect().dim.y,
        },
        facing,
    )
}

#[derive(Debug, Clone, Copy)]
pub enum Facing {
    Left,
    Right,
    Up,
    Down,
}
impl Facing {
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
    pub fn is_up(self) -> bool {
        matches!(self, Self::Up)
    }
    pub fn is_down(self) -> bool {
        matches!(self, Self::Down)
    }
    pub fn is_left(self) -> bool {
        matches!(self, Self::Left)
    }
    pub fn is_right(self) -> bool {
        matches!(self, Self::Right)
    }
}
