use num::abs;
use num_integer::gcd;
use num_rational::Ratio;
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day10 {}

impl day::Day for Day10 {
    fn tag(&self) -> &str { "10" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input(), 200));
    }
}

impl Day10 {
    fn find_best<'a>(&self, asteroids: &'a Vec<(usize, usize)>)
        -> (&'a (usize, usize), usize) {
        asteroids.iter().map(|p| {
            let others = asteroids.iter().filter(|&a| p != a).collect::<Vec<_>>();
//            eprintln!("p {:?} {:?}", p, others);
            (p,
             others.iter().map(|&(x, y)| {
                 let x = *x as i64;
                 let y = *y as i64;
                 let px = p.0 as i64;
                 let py = p.1 as i64;
                 let dx = x - px;
                 let dy = y - py;
                 let n = gcd(dx, dy);
                 let sx = dx / n;
                 let sy = dy / n;
                 let steps = if dx != 0 { dx / sx } else { dy / sy };
                 let (_, _, obstructed) = (1..steps).fold(
                     (px + sx, py + sy, false),
                     |(x, y, obstructed), _|
                         (x + sx, y + sy,
                          obstructed || asteroids.contains(
                              &(x as usize, y as usize))));
//                eprintln!("{}", obstructed);
                 if !obstructed { 1 } else { 0 }
             }).sum())
        }).max_by_key(|&(_, s)| s).unwrap()
    }

    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let asteroids = reader.split(b'\n')
            .filter(|l| !l.as_ref().unwrap().is_empty())
            .enumerate().flat_map(|(y, r)| {
                let m = r.unwrap().into_iter().map(|b| b == b'#');
                let e = m.enumerate();
                e.filter_map(move|(x, b)|
                    if b { Some((x, y)) } else { None })
            })
            .collect::<Vec<_>>();
        Ok(self.find_best(&asteroids).1)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read, n: usize) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let asteroids = reader.split(b'\n')
            .filter(|l| !l.as_ref().unwrap().is_empty())
            .enumerate().flat_map(|(y, r)| {
            let m = r.unwrap().into_iter().map(|b| b == b'#');
            let e = m.enumerate();
            e.filter_map(move|(x, b)|
                if b { Some((x, y)) } else { None })
        })
            .collect::<Vec<_>>();
        let &p = self.find_best(&asteroids).0;
        let (px, py) = p;
        let mut others = asteroids.iter().filter(|&a| p != *a).map(|&a| {
            let  (x, y) = a;
            let (dx, dy) = (x as i64 - px as i64, y as i64 - py as i64);
            if x >= px && y < py {
                (0, Ratio::new(dx, -dy), abs(dx) + abs(dy), a)
            } else if x > px && y >= py {
                (1, Ratio::new(dy, dx), abs(dx) + abs(dy), a)
            } else if x <= px && y > py {
                (2, abs(Ratio::new(-dx, dy)), abs(dx) + abs(dy), a)
            } else {
                (3, abs(Ratio::new(-dy, -dx)), abs(dx) + abs(dy), a)
            }
        }).collect::<Vec<_>>();
        others.sort_by(|&(aq, ar, ad, _), &(bq, br, bd, _)|
            aq.cmp(&bq).then(ar.cmp(&br)).then(ad.cmp(&bd)));
        let mut lq = -1;
        let mut lr = Ratio::new(1, 1);
        let mut i = 0;
        let mut s = 0;
        let (x, y);
        loop {
            let (q, r, d, (ax, ay)) = &mut others[i];
            if *d > 0 && (*q != lq || *r != lr) {
                s += 1;
                if s == n { x = *ax; y = *ay; break; };
                *d = 0;
                lq = *q;
                lr = *r;
            }
            i = (i + 1) % others.len();
        }
        Ok(x * 100 + y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, o: usize) {
        assert_eq!(Day10 {}.part1_impl(&mut s.as_bytes()).unwrap(), o);
    }

    #[test]
    fn part1() {
        test1("\
.#..#
.....
#####
....#
...##", 8);
        test1("\
......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####", 33);
        test1("\
#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.", 35);
        test1("\
.#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..", 41);
        test1("\
.#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##", 210);
    }

    fn test2(s: &str, n: usize, o: usize) {
        assert_eq!(Day10 {}.part2_impl(&mut s.as_bytes(), n).unwrap(), o);
    }

    #[test]
    fn part2() {
        test2("\
.#....#####...#..
##...##.#####..##
##...#...#.#####.
..#.....X...###..
..#.#.....#....##", 36, 1403);
        test2("\
.#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##", 200, 802)
    }
}