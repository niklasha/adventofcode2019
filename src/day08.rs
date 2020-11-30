use std::error;
use std::io::Read;
use crate::day::*;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day08 {}

impl Day for Day08 {
    fn tag(&self) -> &str { "08" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day08 {
    fn count(&self, v: &Vec<u8>, b: u8) -> usize {
        v.iter().filter(|&d| *d == b).count()
    }

    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let pixels = input.bytes().map(|b| b.unwrap()).collect::<Vec<_>>();
        let layers = pixels.chunks_exact(25 * 6).map(|slice| slice.to_vec()).collect::<Vec<_>>();
        let result = layers.iter().map(|layer: &Vec<_>| self.count(layer, b'0'))
            .zip(layers.iter()).min_by_key(|&(count, _)| count)
            .map(|(_, layer)| self.count(layer, b'1') * self.count(layer, b'2'));
        Ok(result.unwrap())
    }

    fn transform(&self, b: u8) -> u8 {
        match b { b'0' => b' ', b'1' => b'.', _ => b'X', }
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<Vec<String>> {
        let pixels = input.bytes().map(|b| b.unwrap()).collect::<Vec<_>>();
        let mut layers = pixels.chunks_exact(25 * 6).map(|slice| slice.to_vec()).collect::<Vec<_>>();
        let init = layers.remove(0);
        let image = layers.iter().fold(init, |result, layer| {
           result.iter().zip(layer.iter()).map(|(&a, &b)|
               if a != b'2' { a } else { b })
        }.collect());
        let result = image.iter().map(|&b| self.transform(b)).collect::<Vec<_>>().chunks(25).map(|line|
            String::from_utf8(line.iter().cloned().collect()).unwrap()).collect::<Vec<_>>();
        Ok(result)
    }
}