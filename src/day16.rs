use num::abs;
use simple_error::bail;
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day16 {}

impl day::Day for Day16 {
    fn tag(&self) -> &str { "16" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let mut reader = io::BufReader::new(input());
        let mut line= String::new();
        reader.read_line(&mut line);
        println!("{:?}", self.part1_impl(line.as_str(), 100));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let mut reader = io::BufReader::new(input());
        let mut line= String::new();
        reader.read_line(&mut line);
        println!("{:?}", self.part2_impl(line.as_str(), 100));
    }
}

impl Day16 {
    fn phase(&self, n: usize) -> Vec<i32> {
        let mut v: Vec<i32> = vec![0, 1, 0, -1].iter().flat_map(|&d| vec![d; n + 1])
            .collect();
        let head = v.remove(0);
        v.push(head);
        v
    }

    fn part1_impl(self: &Self, s: &str, n: usize) -> BoxResult<String> {
        let mut s = (0..n).fold(s.to_string(), |s, _| {
            let it = (0..s.len()).map(|i| {
                let phase = self.phase(i);
                let a: i32 = s.split("").into_iter().filter(|&s| !s.is_empty())
                    .map(|s| s.parse::<i32>().unwrap()).enumerate().map(|(i, d)| {
                    phase[i % phase.len()] * d
                }).sum();
                (abs(a) % 10).to_string()
            });
            it.collect::<Vec<_>>().join("")
        });
        s.truncate(8);
        Ok(s)
    }

    fn part2_helper(self: &Self, s: &str, n: usize, c: usize, offset: usize) -> BoxResult<String> {
        let s = s.repeat(c);
        let output = (0..n).fold(s.to_string(), |s, i| {
            eprintln!("s {:?}", s);
            let it = (0..s.len()).map(|i| {
                let phase = self.phase(i);
                let a: i32 = s.split("").into_iter().filter(|&s| !s.is_empty())
                    .map(|s| s.parse::<i32>().unwrap()).enumerate().map(|(i, d)| {
                    phase[i % phase.len()] * d
                }).sum();
                (abs(a) % 10).to_string()
            });
            it.collect::<Vec<_>>().join("")
        });
        let x = &output[offset..(offset + 8)];
        Ok(x.to_string())
    }

    fn part2_impl(self: &Self, s: &str, n: usize) -> BoxResult<String> {
        self.part2_helper(s, n, 10000, (&s[0..7]).parse().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, n: usize, v: &str) {
        assert_eq!(Day16 {}.part1_impl(s, n).unwrap().as_str(), v);
    }

    #[test]
    fn part1() {
        test1("12345678", 1, "48226158");
        test1("12345678", 2, "34040438");
        test1("12345678", 3, "03415518");
        test1("12345678", 4, "01029498");
    }

    fn test2(s: &str, n: usize, v: &str) {
//        assert_eq!(Day16 {}.part2_impl(s, n).unwrap(), v);
        assert_eq!(Day16 {}.part2_helper(s, n, 1, 0).unwrap(), v);
    }

    #[test]
    fn part2() {
        test2("56781234", 10, "");
//        test2("03036732577212944063491565474664", 100, "84462026");
    }
}