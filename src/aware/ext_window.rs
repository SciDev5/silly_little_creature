use image::{DynamicImage, GenericImageView, Rgba};

use win_screenshot::utils::HwndName;
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{GetWindowRect, IsWindow, IsWindowVisible},
};

use crate::{
    render::renderer::SELF_WINDOW_TITLE,
    util::{RectI, Vec2I},
};

fn window_visible(hwnd: isize) -> bool {
    unsafe { IsWindowVisible(HWND(hwnd)).as_bool() }
}

fn window_rect(hwnd: isize) -> Option<RectI> {
    let mut rect: RECT = Default::default();

    unsafe {
        if GetWindowRect(HWND(hwnd), &mut rect as *mut _).is_err() {
            return None;
        }
    }

    Some(RectI {
        pos: Vec2I {
            x: rect.left,
            y: rect.top,
        },
        dim: Vec2I {
            x: rect.right - rect.left,
            y: rect.bottom - rect.top,
        },
    })
}

fn screenshot(hwnd: isize) -> Option<DynamicImage> {
    let buf = win_screenshot::capture::capture_window(hwnd).ok()?;
    let mut img = DynamicImage::new_rgba8(buf.width, buf.height);
    let dest_buf = img.as_mut_rgba8().unwrap();
    let w = dest_buf.width();
    let h = dest_buf.height();
    for x in 0..w {
        for y in 0..h {
            let i = x + y * w;
            dest_buf[(x, y)] = Rgba([
                buf.pixels[(i * 4 + 0) as usize],
                buf.pixels[(i * 4 + 1) as usize],
                buf.pixels[(i * 4 + 2) as usize],
                buf.pixels[(i * 4 + 3) as usize],
            ])
            // k[(x, y)].0[3] = 255;
        }
    }

    Some(img)
}

#[derive(Debug, Clone)]
pub struct ExtWindowInfo {
    hwnd: isize,
    window_name: String,
    img: DynamicImage,
    rect: RectI,
}
impl ExtWindowInfo {
    pub fn still_exists(&self) -> bool {
        unsafe { IsWindow(HWND(self.hwnd)).as_bool() }
    }
    pub fn img(&self) -> &DynamicImage {
        &self.img
    }
    pub fn refresh_img(&mut self) -> &DynamicImage {
        if let Some(img) = screenshot(self.hwnd) {
            self.img = img;
        }
        &self.img
    }
    pub fn rect(&self) -> RectI {
        self.rect
    }
    pub fn refresh_rect(&mut self) -> RectI {
        if let Some(rect) = window_rect(self.hwnd) {
            self.rect = rect;
        }
        self.rect
    }
    pub fn name(&self) -> &str {
        self.window_name.as_str()
    }
}

pub fn iter_window_candidates() -> impl Iterator<Item = ExtWindowInfo> {
    win_screenshot::utils::window_list()
        .unwrap()
        .into_iter()
        .filter(|it| {
            window_visible(it.hwnd) // window must be visible
            && it.window_name != "Settings" // settings was giving me trouble so im just going to explicitly exclude it
             && it.window_name.as_str() != SELF_WINDOW_TITLE // dont match ourself
        })
        .filter_map(|HwndName { hwnd, window_name }| {
            let img = screenshot(hwnd).unwrap();

            if img
                .pixels()
                .step_by(97)
                .all(|(_, _, Rgba([r, g, b, _]))| r == 0 && g == 0 && b == 0)
            {
                return None;
            }
            let dim = window_rect(hwnd).unwrap();

            Some(ExtWindowInfo {
                hwnd,
                window_name,
                img,
                rect: dim,
            })
        })
}

// #[test]
// fn k() {
//     let a = iter_window_candidates().next().unwrap();

//     let (hs, h) = find_hiding_spot(&a);

//     let mut img = a.img.into_rgba8();
//     if h.is_horizontal() {
//         for d in 0..CREATURE_HIDE_SAFE_ZONE_DIMS.y as u32 {
//             img[(hs.x as u32, hs.y as u32 + d)] = Rgba([255, 0, 0, 255]);
//             img[(hs.x as u32, hs.y as u32 - d)] = Rgba([255, 0, 0, 255]);
//         }
//     } else {
//         for d in 0..CREATURE_HIDE_SAFE_ZONE_DIMS.x as u32 {
//             img[(hs.x as u32 + d, hs.y as u32)] = Rgba([255, 0, 0, 255]);
//             img[(hs.x as u32 - d, hs.y as u32)] = Rgba([255, 0, 0, 255]);
//         }
//     }

//     img.save("./t.png");

//     // for (
//     //     i,
//     //     WindowCandidateInfo {
//     //         window_name,
//     //         img,
//     //         dim,
//     //         ..
//     //     },
//     // ) in iter_window_candidates().enumerate()
//     // {
//     //     dbg!((i, window_name, img.dimensions(), dim));

//     //     let buf = img
//     //         .resize_exact(
//     //             img.width() / 4,
//     //             img.height() / 4,
//     //             image::imageops::FilterType::Nearest,
//     //         )
//     //         .to_luma8();

//     //     let detect = imageproc::edges::canny(&buf, 0.5, 10.0);
//     //     detect.save(format!("./test/test_{}.png", i)).unwrap();

//     //     // img.save(format!("./test/test_{}.png", i)).unwrap();
//     // }
// }
