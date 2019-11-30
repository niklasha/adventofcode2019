use std::io;

pub trait Day {
    fn tag(&self) -> &str;
    fn part1(&self, _input: &dyn Fn() -> Box<dyn io::Read>) {}
    fn part2(&self, _input: &dyn Fn() -> Box<dyn io::Read>) {}
}