use std::error;
use std::io::BufRead;
use crate::day::*;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day01 {}

impl Day for Day01 {
    fn tag(&self) -> &str { "01" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day01 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<i32> {
        let reader = io::BufReader::new(input);
        Ok(reader.lines().map(|s| s.unwrap().parse::<i32>().unwrap() / 3 - 2)
            .sum())
    }

    fn fuel_2(&self, m: i32) -> i32 {
        let f = m / 3 - 2;
        if f <= 0 {0 } else { f + self.fuel_2(f) }
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<i32> {
        let reader = io::BufReader::new(input);
        Ok(reader.lines().map(|s|
            self.fuel_2(s.unwrap().parse::<i32>().unwrap()))
            .sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, f: i32) {
        assert_eq!(Day01 {}.part1_impl(&mut s.as_bytes()).unwrap(), f);
    }

    #[test]
    fn part1() {
        test1("12", 2);
        test1("14", 2);
        test1("1969", 654);
        test1("100756", 33583);
    }

    fn test2(s: &str, f: i32) {
        assert_eq!(Day01 {}.part2_impl(&mut s.as_bytes()).unwrap(), f);
    }

    #[test]
    fn part2() {
        test2("1969", 966);
        test2("100756", 50346);
    }
}