use std::{
    iter::Rev,
    ops::{Add, Div, Mul, Sub},
    time::{Duration, SystemTime},
};

pub type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RectI {
    pub pos: Vec2I,
    pub dim: Vec2I,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vec2I {
    pub x: i32,
    pub y: i32,
}
impl Vec2I {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl Add for Vec2I {
    type Output = Vec2I;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2I {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Vec2I {
    type Output = Vec2I;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2I {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Mul<i32> for Vec2I {
    type Output = Vec2I;
    fn mul(self, rhs: i32) -> Self::Output {
        Vec2I {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Div<i32> for Vec2I {
    type Output = Vec2I;
    fn div(self, rhs: i32) -> Self::Output {
        Vec2I {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

pub struct DeltaTimer(SystemTime);
impl DeltaTimer {
    pub fn new() -> Self {
        Self(SystemTime::now())
    }
    pub fn tick(&mut self) -> Duration {
        let next_t = SystemTime::now();
        let dt = next_t.duration_since(self.0).unwrap_or_else(|e| {
            eprintln!("duration_since was failure, defaulting to dt=0; err={}", e);
            Duration::from_secs(0)
        });
        self.0 = next_t;
        dt
    }
}

pub enum SwitchRev<T: DoubleEndedIterator> {
    Fwd(T),
    Rev(Rev<T>),
}
impl<T: DoubleEndedIterator> SwitchRev<T> {
    pub fn conditional_reverse(iter: T, rev: bool) -> Self {
        if rev {
            Self::Rev(iter.rev())
        } else {
            Self::Fwd(iter)
        }
    }
}
impl<T: DoubleEndedIterator> Iterator for SwitchRev<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Fwd(t) => t.next(),
            Self::Rev(t) => t.next(),
        }
    }
}
