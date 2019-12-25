use core_memo::*;
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use num::abs;
use num::integer::lcm;
use simple_error::bail;
use std::cmp::min;
use std::collections::HashMap;
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day16 {}

fn round_up(x: usize, p: usize) -> usize { (x + p - 1) / p * p }

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

    fn part2_impl(self: &Self, s: &str, n: usize) -> BoxResult<String> {
        let repeat = 10000;
        let phases = n;
        let period = s.len();
        let len = period * repeat;
        let offset = s[..7].parse::<usize>().unwrap();
        let input = s.chars().map(|c| (c as u8 - b'0') as usize)
            .collect::<Vec<_>>();

        let start = offset;
        assert!(start + start >= len);
        let mut output = (start..len).rev().scan(0, |s, offset| {
            let sum = *s + input[offset % period];
            let digit = sum % 10;
            *s = digit;
            Some(digit)
        }).collect::<Vec<_>>();
        output.reverse();

        let output = (2..=phases).fold(output, |input, i| {
            let period = input.len();
            let start = offset;
            assert!(start + start >= len);
            let mut output = (start..len).rev().scan(0, |s, offset| {
                let sum = *s + input[(offset - start) % period];
                let digit = sum % 10;
                *s = digit;
                Some(digit)
            }).collect::<Vec<_>>();
            output.reverse();
            output
        });

        Ok(String::from_utf8(output[..8].iter().map(|&d| d as u8 + b'0').collect()).unwrap())
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
        assert_eq!(Day16 {}.part2_impl(s, n).unwrap(), v);
    }

    #[test]
    fn part2() {
//        test2("00000072109876543210", 0, "21098765");
        test2("03036732577212944063491565474664", 100, "84462026");
        test2("02935109699940807407585447034323", 100, "78725270");
        test2("03081770884921959731165446850517", 100, "53553731");
    }
}