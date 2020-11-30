use num::{abs, signum};
use num_integer::lcm;
use regex::Regex;
use std::collections::HashSet;
use std::error;
use std::io::BufRead;
use crate::day::*;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

#[derive(PartialEq, Eq, Hash, Clone)]
struct Space {
    moons: Vec<Moon>,
}

impl Space {
    fn new(moons: Vec<Moon>) -> Self { Space { moons } }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Moon {
    x: i64,
    y: i64,
    z: i64,
    vx: i64,
    vy: i64,
    vz: i64,
}

impl Moon {
    fn new(x: i64, y: i64, z: i64) -> Self {
        Moon { x, y, z, vx: 0, vy: 0, vz: 0, }
    }

    fn pot(&self) -> i64 { abs(self.x) + abs(self.y) + abs(self.z) }

    fn kin(&self) -> i64 { abs(self.vx) + abs(self.vy) + abs(self.vz) }

    fn e(&self) -> i64 { self.pot() * self.kin() }
}

pub struct Day12 {}

impl Day for Day12 {
    fn tag(&self) -> &str { "12" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), 1000));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day12 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read, n: usize) -> BoxResult<i64> {
        let reader = io::BufReader::new(input);
        lazy_static! {
            static ref RE: Regex = Regex::new("<x=(.+), y=(.+), z=(.+)>").unwrap();
        }
        let mut space = Space::new(reader.lines().map(|l| {
            let l = l.unwrap();
            let cap = RE.captures(&l).unwrap();
            Moon::new(
                cap[1].parse().unwrap(),
                cap[2].parse().unwrap(),
                cap[3].parse().unwrap(), )
        }).collect());
        let apply_gravity = |space: &mut Space| {
            let moons = &mut space.moons;
            for i in 0..moons.len() {
                for j in 0..moons.len() {
                    if i != j {
                        moons[i].vx += signum(moons[j].x - moons[i].x);
                        moons[i].vy += signum(moons[j].y - moons[i].y);
                        moons[i].vz += signum(moons[j].z - moons[i].z);
                    }
                }
            }
        };
        let apply_velocity = |space: &mut Space| {
            for moon in space.moons.iter_mut() {
                moon.x += moon.vx;
                moon.y += moon.vy;
                moon.z += moon.vz;
            }
        };
        for _ in 0..n {
            apply_gravity(&mut space);
            apply_velocity(&mut space);
        }
        Ok(space.moons.iter().map(|m| m.e()).sum())
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<i64> {
        let reader = io::BufReader::new(input);
        lazy_static! {
            static ref RE: Regex = Regex::new("<x=(.+), y=(.+), z=(.+)>").unwrap();
        }
        let mut space = Space::new(reader.lines().map(|l| {
            let l = l.unwrap();
            let cap = RE.captures(&l).unwrap();
            Moon::new(
                cap[1].parse().unwrap(),
                cap[2].parse().unwrap(),
                cap[3].parse().unwrap(), )
        }).collect());
        let apply_x_gravity = |space: &mut Space| {
            let moons = &mut space.moons;
            for i in 0..moons.len() {
                for j in 0..moons.len() {
                    if i != j {
                        moons[i].vx += signum(moons[j].x - moons[i].x);
                    }
                }
            }
        };
        let apply_x_velocity = |space: &mut Space| {
            for moon in space.moons.iter_mut() {
                moon.x += moon.vx;
            }
        };
        let apply_y_gravity = |space: &mut Space| {
            let moons = &mut space.moons;
            for i in 0..moons.len() {
                for j in 0..moons.len() {
                    if i != j {
                        moons[i].vy += signum(moons[j].y - moons[i].y);
                    }
                }
            }
        };
        let apply_y_velocity = |space: &mut Space| {
            for moon in space.moons.iter_mut() {
                moon.y += moon.vy;
            }
        };
        let apply_z_gravity = |space: &mut Space| {
            let moons = &mut space.moons;
            for i in 0..moons.len() {
                for j in 0..moons.len() {
                    if i != j {
                        moons[i].vz += signum(moons[j].z - moons[i].z);
                    }
                }
            }
        };
        let apply_z_velocity = |space: &mut Space| {
            for moon in space.moons.iter_mut() {
                moon.z += moon.vz;
            }
        };
        let mut xp = 0i64;
        let mut seen: HashSet<Space> = HashSet::new();
        for _ in 0i64.. {
            apply_x_gravity(&mut space);
            apply_x_velocity(&mut space);
            if !seen.insert(space.clone()) { break; }
            xp += 1;
        }
        let mut yp = 0i64;
        let mut seen: HashSet<Space> = HashSet::new();
        for _ in 0i64.. {
            apply_y_gravity(&mut space);
            apply_y_velocity(&mut space);
            if !seen.insert(space.clone()) { break; }
            yp += 1;
        }
        let mut zp = 0i64;
        let mut seen: HashSet<Space> = HashSet::new();
        for _ in 0i64.. {
            apply_z_gravity(&mut space);
            apply_z_velocity(&mut space);
            if !seen.insert(space.clone()) { break; }
            zp += 1;
        }
        Ok(lcm(lcm(xp, yp),zp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, n: usize, v: i64) {
        assert_eq!(
            Day12 {}.part1_impl(&mut s.as_bytes(), n).unwrap(), v);
    }


    #[test]
    fn part1() {
        test1("<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>", 10, 179);
        test1("<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>", 100, 1940);
    }

    fn test2(s: &str, v: i64) {
        assert_eq!(
            Day12 {}.part2_impl(&mut s.as_bytes()).unwrap(), v);
    }


    #[test]
    fn part2() {
        test2("<x=-1, y=0, z=2>
<x=2, y=-10, z=-7>
<x=4, y=-8, z=8>
<x=3, y=5, z=-1>", 2772);
        test2("<x=-8, y=-10, z=0>
<x=5, y=5, z=10>
<x=2, y=-7, z=3>
<x=9, y=-8, z=-3>", 4686774924);
    }
}