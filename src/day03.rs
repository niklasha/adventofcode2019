use simple_error::bail;
use std::collections;
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day03 {}

impl day::Day for Day03 {
    fn tag(&self) -> &str { "03" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day03 {
    fn dist(&self, p: (i32, i32)) -> i32 { p.0.abs() + p.1.abs() }

    fn part1_impl(self: &Self, input: &mut dyn io::Read)
        -> BoxResult<Option<i32>> {
        let reader = io::BufReader::new(input);
        let lines = reader.lines().map(|l| l.unwrap());
        let mut g = collections::HashMap::new();
        let mut md: Option<i32> = None;
        for l in lines.enumerate() {
            let mut p = (0, 0);
            for m in l.1.split(',') {
                for _n in 1..=m[1..].parse::<i32>().unwrap() {
                    match &m[0..1] {
                        "U" => p = (p.0, p.1 + 1),
                        "D" => p = (p.0, p.1 - 1),
                        "L" => p = (p.0 - 1, p.1),
                        "R" => p = (p.0 + 1, p.1),
                        _ => bail!("unknown direction"),
                    }
                    match g.get(&p) {
                        None => { g.insert(p, l.0); },
                        Some(&w) => if w != l.0 {
                            let d = self.dist(p);
                            if md.is_none() || d < md.unwrap() { md = Some(d); }
                        },
                    }
                }
            }
        }
        Ok(md)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read)
        -> BoxResult<Option<i32>> {
        let reader = io::BufReader::new(input);
        let lines = reader.lines().map(|l| l.unwrap());
        let mut g = collections::HashMap::new();
        let mut md: Option<i32> = None;
        for l in lines.enumerate() {
            let mut p = (0, 0);
            let mut s = 0;
            for m in l.1.split(',') {
                for _n in 1..=m[1..].parse::<i32>().unwrap() {
                    match &m[0..1] {
                        "U" => p = (p.0, p.1 + 1),
                        "D" => p = (p.0, p.1 - 1),
                        "L" => p = (p.0 - 1, p.1),
                        "R" => p = (p.0 + 1, p.1),
                        _ => bail!("unknown direction"),
                    }
                    s += 1;
                    match g.get(&p) {
                        None => { g.insert(p, (l.0, s)); },
                        Some(&w) => if w.0 != l.0 {
                            let d = w.1 + s;
                            if md.is_none() || d < md.unwrap() { md = Some(d); }
                        },
                    }
                }
            }
        }
        Ok(md)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(w1: &str, w2: &str, d: i32) {
        let ws = format!("{}\n{}", w1, w2);
        assert_eq!(
            Day03 {}.part1_impl(&mut ws.as_bytes()).unwrap(), Some(d));
    }

    #[test]
    fn part1() {
        test1("R8,U5,L5,D3", "U7,R6,D4,L4", 6);
        test1("R75,D30,R83,U83,L12,D49,R71,U7,L72",
              "U62,R66,U55,R34,D71,R55,D58,R83", 159);
        test1("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
              "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7", 135);
    }

    fn test2(w1: &str, w2: &str, d: i32) {
        let ws = format!("{}\n{}", w1, w2);
        assert_eq!(
            Day03 {}.part2_impl(&mut ws.as_bytes()).unwrap(), Some(d));
    }

    #[test]
    fn part2() {
        test2("R8,U5,L5,D3", "U7,R6,D4,L4", 30);
        test2("R75,D30,R83,U83,L12,D49,R71,U7,L72",
              "U62,R66,U55,R34,D71,R55,D58,R83", 610);
        test2("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
              "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7", 410);
    }
}