use closure::closure;
use evmap;
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use simple_error::bail;
use std::collections::{HashMap, HashSet};
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day20 {}

impl day::Day for Day20 {
    fn tag(&self) -> &str { "20" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day20 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let (grid_r, mut grid_w) = evmap::new();
        let (portals_r, mut portals_w) = evmap::new();
        let mut w = 0;
        let mut h = 0;
        let mut b = 0;
        reader.lines().enumerate().for_each(|(y, l)| {
            let l = l.unwrap();
            // width
            if y == 0 { w = l.len() - 4; }
            l.chars().enumerate().for_each(|(x, c)| {
//                eprintln!("x {} y {} w {} h {} b {}", x, y, w, h, b);
                let uc = c.is_ascii_uppercase();
                // upper outer portal
                if y == 1 && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, 0), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 1));
                    portals_w.refresh();
                }
                // left outer portal
                else if x == 1 && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(0, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 1, y - 2));
                    portals_w.refresh();
                }
                // right outer portal
                else if x == 3 + w && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x - 1, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 4, y - 2));
                    portals_w.refresh();
                }
                // breadth
                else if b == 0 && y > 1 && x > 1 && x < 2 + w && c == ' ' { b = y - 2; }
                // left inner portal
                else if b > 0 && y > 1 + b && x == 3 + b && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x - 1, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 4, y - 2));
                    portals_w.refresh();
                }
                // right inner portal
                else if b > 0 && y > 3 + b && y < w - b && x == 1 + w - b && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x - 1, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 1, y - 2));
                    portals_w.refresh();
                }
                // upper inner portal
                else if b > 0 && y == 3 + b && x > 1 && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, y - 1), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 4));
                    portals_w.refresh();
                }
                // height
                else if h == 0 && b > 0 && y > 3 + b && x > 4 + b && x < w - b && uc { h = y + b; }
                // lower inner portal
                else if h > 0 && y == 1 + h - b && x > 1 && x < 2 + w && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, y - 1), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 1));
                    portals_w.refresh();
                }
                // lower outer portal
                else if h > 0 && y == 3 + h && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, y - 1), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 4));
                    portals_w.refresh();
                }
                grid_w.update((x, y), c);
                grid_w.refresh();
            })
        });
//        eprintln!("w {} h {} b {}", w, h, b);
        let mut jump = HashMap::new();
        portals_r.for_each(|_x, y| {
//            eprintln!("portals {:?} {:?}", _x, y);
            if y.len() == 2 {
                jump.insert(y[0], y[1]);
                jump.insert(y[1], y[0]);
            }
        });
        let entry = portals_r.get_and("AA", |p| p[0]).unwrap();
        let exit = portals_r.get_and("ZZ", |p| p[0]).unwrap();
//        eprintln!("entry {:?} exit {:?}", entry, exit);

        let dirs = vec![(0, -1), (1, 0), (0, 1), (-1, 0)];

        let next = |(x, y): (usize, usize), path: &Vec<(usize, usize)>| {
            let mut next = dirs.iter().flat_map(|&(dx, dy)| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 { None } else {
                    let (nx, ny) = (nx as usize, ny as usize);
                    if path.contains(&(nx, ny)) { None } else {
                        grid_r.get_and(
                            &(nx + 2, ny + 2),
                            |c|
                                if c[0] == '.' { Some((nx, ny)) } else { None })
                            .unwrap_or(None)
                    }
                }
            }).collect::<Vec<_>>();
            if let Some(&p) = jump.get(&(x, y)) { next.push(p); };
            next
        };

        loop {
            let mut plugged = false;
            grid_r.for_each(|(x, y), c| {
                if c[0] == '.' {
                    let p = (x - 2, y - 2);
                    if next(p, &vec![]).len() == 1 && p != entry && p != exit {
//                        eprintln!("plugging {:?}", p);
                        grid_w.update((*x, *y), '#');
                        plugged = true;
                    }
                }
            });
            if !plugged { break; }
            grid_w.refresh();
        };

//        for y in 0..h {
//            let mut l = String::new();
//            for x in 0..w {
//                grid_r.get_and(&(x + 2, y + 2), |c| l.push(c[0]));
//            }
//            eprintln!("{}", l);
//        }

        let mut seen = HashSet::new();
        let r = (1..).fold_while((vec![(entry, vec![])], 0), |(states, i), _| {
//            eprintln!("i {} states len {}", i, states.len());
//            eprintln!("states {:?}", states);
            states.iter().for_each(|&(p, _)| { seen.insert(p); });
            let new_states: Vec<(_, Vec<_>)> = states.iter().flat_map(|(pos, path)| {
                let candidates: Vec<_> = next(*pos, path).into_iter().filter(|p| !seen.contains(p)).collect();
//                eprintln!("candidates {:?}", candidates);
                candidates.into_iter().map(move |p| {
                    let mut path = path.clone();
                    path.push(*pos);
                    (p, path.clone())
                })
            }).collect();
            if new_states.is_empty() || new_states.iter().any(|&(p, _)| p == exit)
                { Done((new_states, i + 1)) }
            else { Continue((new_states, i + 1)) }
        });
        let r = r.into_inner().1;
//        eprintln!("{:?}", r);
        Ok(r)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let (grid_r, mut grid_w) = evmap::new();
        let (portals_r, mut portals_w) = evmap::new();
        let mut w = 0;
        let mut h = 0;
        let mut b = 0;
        reader.lines().enumerate().for_each(|(y, l)| {
            let l = l.unwrap();
            // width
            if y == 0 { w = l.len() - 4; }
            l.chars().enumerate().for_each(|(x, c)| {
//                eprintln!("x {} y {} w {} h {} b {}", x, y, w, h, b);
                let uc = c.is_ascii_uppercase();
                // upper outer portal
                if y == 1 && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, 0), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 1, true));
                    portals_w.refresh();
                }
                // left outer portal
                else if x == 1 && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(0, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 1, y - 2, true));
                    portals_w.refresh();
                }
                // right outer portal
                else if x == 3 + w && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x - 1, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 4, y - 2, true));
                    portals_w.refresh();
                }
                // breadth
                else if b == 0 && y > 1 && x > 1 && x < 2 + w && c == ' ' { b = y - 2; }
                // left inner portal
                else if b > 0 && y > 1 + b && x == 3 + b && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x - 1, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 4, y - 2, false));
                    portals_w.refresh();
                }
                // right inner portal
                else if b > 0 && y > 3 + b && y < w - b && x == 1 + w - b && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x - 1, y), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 1, y - 2, false));
                    portals_w.refresh();
                }
                // upper inner portal
                else if b > 0 && y == 3 + b && x > 1 && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, y - 1), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 4, false));
                    portals_w.refresh();
                }
                // height
                else if h == 0 && b > 0 && y > 3 + b && x > 4 + b && x < w - b && uc { h = y + b; }
                // lower inner portal
                else if h > 0 && y == 1 + h - b && x > 1 && x < 2 + w && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, y - 1), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 1, false));
                    portals_w.refresh();
                }
                // lower outer portal
                else if h > 0 && y == 3 + h && uc {
                    let mut s = String::new();
                    grid_r.get_and(&(x, y - 1), |c| s.push(c[0]));
                    s.push(c);
                    portals_w.insert(s, (x - 2, y - 4, true));
                    portals_w.refresh();
                }
                grid_w.update((x, y), c);
                grid_w.refresh();
            })
        });
//        eprintln!("w {} h {} b {}", w, h, b);
        let mut jump = HashMap::new();
        portals_r.for_each(|_x, y| {
//            eprintln!("portals {:?} {:?}", _x, y);
            if y.len() == 2 {
                jump.insert(y[0], y[1]);
                jump.insert(y[1], y[0]);
            }
        });
        let entry = portals_r.get_and("AA", |p| (p[0].0, p[0].1, 0)).unwrap();
        let exit = portals_r.get_and("ZZ", |p| (p[0].0, p[0].1, 0)).unwrap();
//        eprintln!("entry {:?} exit {:?}", entry, exit);

        let dirs = vec![(0, -1), (1, 0), (0, 1), (-1, 0)];

        let next = |(x, y, z): (usize, usize, usize), path: &Vec<(usize, usize, usize)>| {
            let mut next = dirs.iter().flat_map(|&(dx, dy)| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 { None } else {
                    let (nx, ny) = (nx as usize, ny as usize);
                    if path.contains(&(nx, ny, z)) { None } else {
                        grid_r.get_and(
                            &(nx + 2, ny + 2),
                            |c|
                                if c[0] == '.' { Some((nx, ny, z)) } else { None })
                            .unwrap_or(None)
                    }
                }
            }).collect::<Vec<_>>();
            if let Some(&(nx, ny, _)) = jump.get(&(x, y, false)) {
                next.push((nx, ny, z + 1));
            } else if let Some(&(nx, ny, _)) = jump.get(&(x, y, true)) {
                if z > 0 { next.push((nx, ny, z - 1)); }
            };
//            eprintln!("{} {} {} -> {:?}", x, y, z, next);
            next
        };

        loop {
            let mut plugged = false;
            grid_r.for_each(|(x, y), c| {
                if c[0] == '.' {
                    let p0 = (x - 2, y - 2, 0);
                    let p1 = (x - 2, y - 2, 1);
                    if next(p1, &vec![]).len() == 1 && p0 != entry && p0 != exit {
//                        eprintln!("plugging ({}, {})", p0.0, p0.1);
                        grid_w.update((*x, *y), '#');
                        plugged = true;
                    }
                }
            });
            if !plugged { break; }
            grid_w.refresh();
        };

//        for y in 0..h {
//            let mut l = String::new();
//            for x in 0..w {
//                grid_r.get_and(&(x + 2, y + 2), |c| l.push(c[0]));
//            }
//            eprintln!("{}", l);
//        }

        let mut seen = HashSet::new();
        let r = (1..).fold_while((vec![(entry, vec![])], 0), |(states, i), _| {
//            eprintln!("i {} states len {}", i, states.len());
//            eprintln!("states {:?}", states);
            states.iter().for_each(|&(p, _)| { seen.insert(p); });
            let new_states: Vec<(_, Vec<_>)> = states.iter().flat_map(|(pos, path)| {
                let candidates: Vec<_> = next(*pos, path).into_iter().filter(|p| !seen.contains(p)).collect();
                candidates.into_iter().map(move |p| {
                    let mut path = path.clone();
                    path.push(*pos);
                    (p, path.clone())
                })
            }).collect();
            if new_states.is_empty() || new_states.iter().any(|&(p, _)| p == exit)
            { Done((new_states, i + 1)) }
            else { Continue((new_states, i + 1)) }
        });
        let r = r.into_inner().1;
//        eprintln!("{:?}", r);
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, n: usize) {
        assert_eq!(
            Day20 {}.part1_impl(&mut s.replace(":", "").as_bytes()).unwrap(),
            n);
    }

    #[test]
    fn part1() {
        test1("         A           :
         A           :
  #######.#########  :
  #######.........#  :
  #######.#######.#  :
  #######.#######.#  :
  #######.#######.#  :
  #####  B    ###.#  :
BC...##  C    ###.#  :
  ##.##       ###.#  :
  ##...DE  F  ###.#  :
  #####    G  ###.#  :
  #########.#####.#  :
DE..#######...###.#  :
  #.#########.###.#  :
FG..#########.....#  :
  ###########.#####  :
             Z       :
             Z       :", 23);
        test1("                   A               :
                   A               :
  #################.#############  :
  #.#...#...................#.#.#  :
  #.#.#.###.###.###.#########.#.#  :
  #.#.#.......#...#.....#.#.#...#  :
  #.#########.###.#####.#.#.###.#  :
  #.............#.#.....#.......#  :
  ###.###########.###.#####.#.#.#  :
  #.....#        A   C    #.#.#.#  :
  #######        S   P    #####.#  :
  #.#...#                 #......VT:
  #.#.#.#                 #.#####  :
  #...#.#               YN....#.#  :
  #.###.#                 #####.#  :
DI....#.#                 #.....#  :
  #####.#                 #.###.#  :
ZZ......#               QG....#..AS:
  ###.###                 #######  :
JO..#.#.#                 #.....#  :
  #.#.#.#                 ###.#.#  :
  #...#..DI             BU....#..LF:
  #####.#                 #.#####  :
YN......#               VT..#....QG:
  #.###.#                 #.###.#  :
  #.#...#                 #.....#  :
  ###.###    J L     J    #.#.###  :
  #.....#    O F     P    #.#...#  :
  #.###.#####.#.#####.#####.###.#  :
  #...#.#.#...#.....#.....#.#...#  :
  #.#####.###.###.#.#.#########.#  :
  #...#.#.....#...#.#.#.#.....#.#  :
  #.###.#####.###.###.#.#.#######  :
  #.#.........#...#.............#  :
  #########.###.###.#############  :
           B   J   C               :
           U   P   P               :", 58);
    }

    fn test2(s: &str, n: usize) {
        assert_eq!(Day20 {}.part2_impl(&mut s.replace(":", "").as_bytes()).unwrap(), n);
    }

    #[test]
    fn part2() {
        test2("             Z L X W       C                 :
             Z P Q B       K                 :
  ###########.#.#.#.#######.###############  :
  #...#.......#.#.......#.#.......#.#.#...#  :
  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###  :
  #.#...#.#.#...#.#.#...#...#...#.#.......#  :
  #.###.#######.###.###.#.###.###.#.#######  :
  #...#.......#.#...#...#.............#...#  :
  #.#########.#######.#.#######.#######.###  :
  #...#.#    F       R I       Z    #.#.#.#  :
  #.###.#    D       E C       H    #.#.#.#  :
  #.#...#                           #...#.#  :
  #.###.#                           #.###.#  :
  #.#....OA                       WB..#.#..ZH:
  #.###.#                           #.#.#.#  :
CJ......#                           #.....#  :
  #######                           #######  :
  #.#....CK                         #......IC:
  #.###.#                           #.###.#  :
  #.....#                           #...#.#  :
  ###.###                           #.#.#.#  :
XF....#.#                         RF..#.#.#  :
  #####.#                           #######  :
  #......CJ                       NM..#...#  :
  ###.#.#                           #.###.#  :
RE....#.#                           #......RF:
  ###.###        X   X       L      #.#.#.#  :
  #.....#        F   Q       P      #.#.#.#  :
  ###.###########.###.#######.#########.###  :
  #.....#...#.....#.......#...#.....#.#...#  :
  #####.#.###.#######.#######.###.###.#.#.#  :
  #.......#.......#.#.#.#.#...#...#...#.#.#  :
  #####.###.#####.#.#.#.#.###.###.#.###.###  :
  #.......#.....#.#...#...............#...#  :
  #############.#.#.###.###################  :
               A O F   N                     :
               A A D   M                     :", 396);
    }
}