use evmap;
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;
use std::collections::HashSet;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day18 {}

impl day::Day for Day18 {
    fn tag(&self) -> &str { "18" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day18 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let (grid_r, mut grid_w) = evmap::new();
        let mut origin = (0, 0);
        let (mut mx, mut my) = (0, 0);
        let mut keys = HashSet::new();
        reader.lines().enumerate().for_each(|(y, l)|
            l.unwrap().chars().enumerate().for_each(|(x, c)| {
                if x > mx { mx = x };
                if y > my { my = y };
//                eprintln!("{} {} {:?}", x, y, c);
                let mut c = c;
                match c {
                    '@' => { origin = (x, y); c = '.'; },
//                    '.' | '#' => (),
//                    _ => {
//                        if c.is_ascii_lowercase() {
//                            key
//                        }
//                        c = '.';
//                    }
                    _ => if c.is_ascii_lowercase() { keys.insert(c); },
                };
                grid_w.update((x, y), c);
                grid_w.refresh();
            }));
        let dirs = vec![(0, -1), (1, 0), (0, 1), (-1, 0)];

        let next = |(x, y): (usize, usize)| dirs.iter().flat_map(|&(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 { None }
            else {
                let (nx, ny) = (nx as usize, ny as usize);
                grid_r.get_and(
                    &(nx, ny),
                    |c| if c[0] != '#' { Some((nx, ny)) } else { None })
                    .unwrap_or(None)
            }
        }).collect::<Vec<_>>();

        fn to_string(s: &HashSet<char>) -> String {
            s.into_iter().sorted().join("")
        }

        let mut seen = HashSet::new();
//        eprintln!("{}", to_string(&keys));
        let r = (1..).fold_while(
            (1, vec![(origin, None as Option<(usize, usize)>, keys)]),
            |(_, states), i| {
//                eprintln!("i {}", i);
            let new_states: Vec<_> = states.into_iter()
                .flat_map(|state| {
                    let (pos, last, locked_keys) = state;
                    let seen_key = (pos, to_string(&locked_keys));
                    if seen.contains(&seen_key) { vec![] } else {
                        seen.insert(seen_key);
//                    eprintln!("i {} pos {:?} last {:?} locked_keys {:?}", i, pos, last, locked_keys);
                        let candidates = next(pos);
//                    eprintln!("candidates {:?}", candidates);
                        if candidates.len() == 1 && Some(candidates[0]) == last {
//                            eprintln!("plugging {:?}", pos);
                            grid_w.update(pos, '#');
                            grid_w.refresh();
                        }
                        let candidates2: Vec<_> = candidates.iter()
                            .filter(|&n|
                                Some(*n) != last
                                    && grid_r.get_and(
                                    n,
                                    |c|
                                        !c[0].is_ascii_uppercase()
                                            || !locked_keys.contains(
                                            &c[0].to_ascii_lowercase()))
                                    == Some(true))
                            .map(|&p| {
                                let key = grid_r.get_and(
                                    &p,
                                    |c|
                                        if c[0].is_ascii_lowercase() { Some(c[0]) } else { None });
                                let mut locked_keys = locked_keys.clone();
                                let mut last = Some(pos);
                                match key {
                                    Some(Some(key)) => {
                                        locked_keys.remove(&key);
                                        last = None;
                                    },
                                    _ => (),
                                };
                                let seen_key = to_string(&locked_keys);
                                (p, last, locked_keys, seen_key)
                            }).filter(|(p, _, _, seen_key)|
                            !seen.contains(&(*p, seen_key.clone())))
                            .map(|(p, last, locked_keys, _)| (p, last, locked_keys))
                            .collect();
//                    eprintln!("candidates2 {:?}", candidates2);
                        candidates2
                    }
                })
                .collect();
//                eprintln!("new_states {:?}", new_states);
                let r = (i, new_states.clone());
                if new_states.into_iter().find(|(_, _, locked_keys)| locked_keys.is_empty()) != None {
                    Done(r)
                } else {
                    Continue(r)
                }
            });
        let r = r.into_inner().0;
        eprintln!("{:?}", r);
        Ok(r)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let (grid_r, mut grid_w) = evmap::new();
        let mut origin = (0, 0);
        let (mut mx, mut my) = (0, 0);
        let mut keys = HashSet::new();
        reader.lines().enumerate().for_each(|(y, l)|
            l.unwrap().chars().enumerate().for_each(|(x, c)| {
                if x > mx { mx = x };
                if y > my { my = y };
//                eprintln!("{} {} {:?}", x, y, c);
                let mut c = c;
                match c {
                    '@' => {
                        origin = (x, y);
                        c = '.';
                    },
//                    '.' | '#' => (),
//                    _ => {
//                        if c.is_ascii_lowercase() {
//                            key
//                        }
//                        c = '.';
//                    }
                    _ => if c.is_ascii_lowercase() { keys.insert(c); },
                };
                grid_w.update((x, y), c);
                grid_w.refresh();
            }));
        let (ox, oy) = origin;
        let origins = vec![
            (ox - 1, oy - 1),
            (ox + 1, oy - 1),
            (ox - 1, oy + 1),
            (ox + 1, oy + 1)];
        grid_w.update((ox, oy - 1), '#');
        grid_w.update((ox - 1, oy), '#');
        grid_w.update((ox, oy), '#');
        grid_w.update((ox +  1, oy), '#');
        grid_w.update((ox, oy + 1), '#');

        let dirs = vec![(0, -1), (1, 0), (0, 1), (-1, 0)];

        let next = |(x, y): (usize, usize)| dirs.iter().flat_map(|&(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 { None } else {
                let (nx, ny) = (nx as usize, ny as usize);
                grid_r.get_and(
                    &(nx, ny),
                    |c| if c[0] != '#' { Some((nx, ny)) } else { None })
                    .unwrap_or(None)
            }
        }).collect::<Vec<_>>();

        fn to_string(s: &HashSet<char>) -> String {
            s.into_iter().sorted().join("")
        }

        let mut seen = HashSet::new();
//        eprintln!("{}", to_string(&keys));

        let robots: Vec<_> = origins.into_iter().map(|o| (o, None)).collect();
        let r = (1..).fold_while(
            (1, vec![(robots, keys)]),
            |(_, states), i| {
//                eprintln!("i {}", i);
                let new_states: Vec<_> = states.into_iter()
                    .flat_map(|state| {
                        let (robots, locked_keys) = state;
                        let locked_keys = locked_keys.clone();
                        robots.iter().flat_map(|(pos, last)| {
                            let (pos, last) = (*pos, *last);
                            let seen_key = (pos, to_string(&locked_keys));
                            if seen.contains(&seen_key) { vec![] } else {
                                seen.insert(seen_key);
//                                eprintln!("i {} pos {:?} last {:?} locked_keys {:?}", i, pos, last, locked_keys);
                                let candidates = next(pos);
//                                eprintln!("candidates {:?}", candidates);
                                if candidates.len() == 1 && Some(candidates[0]) == last {
//                                    eprintln!("plugging {:?}", pos);
                                    grid_w.update(pos, '#');
                                    grid_w.refresh();
                                }
                                let candidates2: Vec<_> = candidates.iter()
                                    .filter(|&n|
                                        Some(*n) != last
                                            && grid_r.get_and(
                                            n,
                                            |c|
                                                !c[0].is_ascii_uppercase()
                                                    || !locked_keys.contains(
                                                    &c[0].to_ascii_lowercase()))
                                            == Some(true))
                                    .map(|&p| {
                                        let key = grid_r.get_and(
                                            &p,
                                            |c|
                                                if c[0].is_ascii_lowercase() { Some(c[0]) } else { None });
                                        let mut locked_keys = locked_keys.clone();
                                        let mut last = Some(pos);
                                        match key {
                                            Some(Some(key)) => {
                                                locked_keys.remove(&key);
                                                last = None;
                                            },
                                            _ => (),
                                        };
                                        let seen_key = to_string(&locked_keys);
                                        (p, last, locked_keys, seen_key)
                                    }).filter(|(p, _, _, seen_key)|
                                    !seen.contains(&(*p, seen_key.clone())))
                                    .map(|(p, last, locked_keys, _)| {
                                        let mut robots = robots.clone();
                                        robots.iter_mut().for_each(|t| if t.0 == pos { *t = (p, last); });
                                        (robots, locked_keys.clone())
                                    })
                                    .collect();
//                                eprintln!("candidates2 {:?}", candidates2);
                                candidates2
                            }
                        }).collect::<Vec<_>>()
                    })
                    .collect();
//                eprintln!("new_states {:?}", new_states);
                let r = (i, new_states.clone());
                if new_states.is_empty() || new_states.into_iter().find(|(_, locked_keys)| locked_keys.is_empty()) != None {
                    Done(r)
                } else {
                    Continue(r)
                }
            });
        let r = r.into_inner().0;
        eprintln!("{:?}", r);
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, n: usize) {
        assert_eq!(Day18 {}.part1_impl(&mut s.as_bytes()).unwrap(), n);
    }

    #[test]
    fn part1() {
        test1("#########
#b.A.@.a#
#########", 8);
        test1("########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################", 86);
        test1("########################
#...............b.C.D.f#
#.######################
#.....@.a.B.c.d.A.e.F.g#
########################", 132);
        test1("#################
#i.G..c...e..H.p#
########.########
#j.A..b...f..D.o#
########@########
#k.E..a...g..B.n#
########.########
#l.F..d...h..C.m#
#################", 136);
        test1("########################
#@..............ac.GI.b#
###d#e#f################
###A#B#C################
###g#h#i################
########################", 81);
    }

    fn test2(s: &str, n: usize) {
        assert_eq!(Day18 {}.part2_impl(&mut s.as_bytes()).unwrap(), n);
    }

    #[test]
    fn part2() {
        test2("#######
#a.#Cd#
##...##
##.@.##
##...##
#cB#Ab#
#######", 8);
        test2("###############
#d.ABC.#.....a#
######...######
######.@.######
######...######
#b.....#.....c#
###############", 24);
        test2("#############
#DcBa.#.GhKl#
#.###...#I###
#e#d#.@.#j#k#
###C#...###J#
#fEbA.#.FgHi#
#############", 32);
        test2("#############
#g#f.D#..h#l#
#F###e#E###.#
#dCba...BcIJ#
#####.@.#####
#nK.L...G...#
#M###N#H###.#
#o#m..#i#jk.#
#############", 72);
    }
}