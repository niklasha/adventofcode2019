use simple_error::bail;
use std::error;
use std::io::BufRead;
use crate::day::*;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day02 {}

impl Day for Day02 {
    fn tag(&self) -> &str { "02" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), true, 0));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input(), 19690720));
    }
}

impl Day02 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read, reset: bool, i: usize)
        -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let mut p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        if reset {
            p[1] = 12;
            p[2] = 2;
        }
        let mut ip = 0;
        while p[ip] != 99 {
            match p[ip] {
                1 => {
                    let a = p[p[ip + 1]];
                    let b = p[p[ip + 2]];
                    let c = p[ip + 3];
                    p[c] = a + b;
                },
                2 => {
                    let a = p[p[ip + 1]];
                    let b = p[p[ip + 2]];
                    let c = p[ip + 3];
                    p[c] = a * b;
                },
                _ => bail!("unknown opcode"),
            };
            ip += 4;
        }
        Ok(p[i])
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read, o: usize) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let p0 = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        for noun in 0..100 {
            for verb in 0..100 {
                let mut p = p0.clone();
                p[1] = noun;
                p[2] = verb;
                let mut ip = 0;
                while p[ip] != 99 {
                    match p[ip] {
                        1 => {
                            let a = p[p[ip + 1]];
                            let b = p[p[ip + 2]];
                            let c = p[ip + 3];
                            p[c] = a + b;
                        },
                        2 => {
                            let a = p[p[ip + 1]];
                            let b = p[p[ip + 2]];
                            let c = p[ip + 3];
                            p[c] = a * b;
                        },
                        _ => bail!("unknown opcode"),
                    };
                    ip += 4;
                }
                if p[0] == o { return Ok(100 * noun + verb); }
            }
        }
        bail!("not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, i: usize, v: usize) {
        assert_eq!(
            Day02 {}.part1_impl(&mut s.as_bytes(), false, i).unwrap(), v);
    }

    #[test]
    fn part1() {
        test1("1,0,0,0,99", 0, 2);
        test1("2,3,0,3,99", 3, 6);
        test1("2,4,4,5,99,0", 5, 9801);
        test1("1,1,1,4,99,5,6,0,99", 0, 30);
        test1("1,9,10,3,2,3,11,0,99,30,40,50", 0, 3500);
    }
}