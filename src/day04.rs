use std::error;
use std::io::BufRead;
use crate::day::*;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day04 {}

impl Day for Day04 {
    fn tag(&self) -> &str { "04" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day04 {
    #[allow(dead_code)]
    fn ok_1_imperative(&self, s: &str) -> bool {
        let v = s.bytes().collect::<Vec<_>>();
        let mut adjacent = false;
        for i in 1..v.len() {
            if v[i] < v[i - 1] { return false; }
            if v[i] == v[i - 1] { adjacent = true; }
        }
       adjacent
    }

    fn ok_1_functional(&self, s: &str) -> bool {
        let r = s.bytes().fold((None, true, false), |s, d|
            (Some(d), s.1 && s.0.map_or(true, |e| d >= e),
             s.2 || s.0.map_or(false, |e| d == e)));
        r.1 & r.2
    }

    fn part1_impl(self: &Self, input: &mut dyn io::Read)
        -> BoxResult<i32> {
        let reader = io::BufReader::new(input);
        let mut split = reader.split(b'-').map(|v|
            String::from_utf8(v.unwrap()).unwrap());
        let start = split.next().unwrap().trim_end().parse::<usize>()?;
        let stop = split.next().unwrap().trim_end().parse::<usize>()?;
        let mut n = 0;
        for i in start..=stop {
            if self.ok_1_functional(format!("{:0>6}", i).as_str()) { n += 1; }
        }
        Ok(n)
    }

    #[allow(dead_code)]
    fn ok_2_imperative(&self, s: &str) -> bool {
        let v = s.bytes().collect::<Vec<_>>();
        let mut adjacent = false;
        for i in 1..v.len() {
            if v[i] < v[i - 1] { return false; }
            if v[i] == v[i - 1] && ((i == v.len() - 1 || v[i] != v[i + 1])
                && (i < 2 || v[i] != v[i - 2])) {
                adjacent = true;
            }
        }
        adjacent
    }

    fn ok_2_functional(&self, s: &str) -> bool {
        let (_, running, increasing, exactly_two)
            = s.bytes()
            .fold((None, 0, true, false),
                  |(last, running, increasing, exactly_two), digit| {
            let duplicate = last == Some(digit);
            (Some(digit), if duplicate { running + 1 } else { 1 },
             increasing && (last == None || digit >= last.unwrap()),
             exactly_two || (running == 2 && !duplicate))
        });
        increasing && (exactly_two || running == 2)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read)
        -> BoxResult<i32> {
        let reader = io::BufReader::new(input);
        let mut split = reader.split(b'-').map(|v|
            String::from_utf8(v.unwrap()).unwrap());
        let start = split.next().unwrap().trim_end().parse::<usize>()?;
        let stop = split.next().unwrap().trim_end().parse::<usize>()?;
        let mut n = 0;
        for i in start..=stop {
            if self.ok_2_functional(&format!("{:0>6}", i)) { n += 1; }
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, b: bool) {
        assert_eq!(Day04 {}.ok_1_imperative(s), b);
        assert_eq!(Day04 {}.ok_1_functional(s), b);
    }

    #[test]
    fn part1() {
        test1("111111", true);
        test1("223450", false);
        test1("123789", false);
    }

    fn test2(s: &str, b: bool) {
        assert_eq!(Day04 {}.ok_2_imperative(s), b);
        assert_eq!(Day04 {}.ok_2_functional(s), b);
    }

    #[test]
    fn part2() {
        test2("112233", true);
        test2("123444", false);
        test2("111122", true);
    }
}