use simple_error::bail;
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day05 {}

impl day::Day for Day05 {
    fn tag(&self) -> &str { "05" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), 1));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input(), 5));
    }
}

impl Day05 {
    fn op(&self, c: i32) -> i32 { c % 100 }

    fn val(&self, p: &Vec<i32>, ip: usize, i: usize) -> i32 {
        let a = p[ip + i];
        let v = if p[ip] / vec![100, 1000][i - 1] % 10 == 0 { p[a as usize] } else { a };
        v
    }

    fn part1_impl(self: &Self, input: &mut dyn io::Read, i: i32)
                  -> BoxResult<i32> {
        let reader = io::BufReader::new(input);
        let mut p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        let mut ip = 0;
        let mut o = None;
        while self.op(p[ip]) != 99 {
            match self.op(p[ip]) {
                1 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2);
                    let c = p[ip + 3 as usize] as usize;
                    p[c] = a + b;
                    ip += 4;
                },
                2 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2);
                    let c = p[ip + 3 as usize] as usize;
                    p[c] = a * b;
                    ip += 4;
                },
                3 => {
                    let a = p[ip + 1];
                    p[a as usize] = i;
                    ip += 2;
                },
                4 => {
                    let a = self.val(&p, ip, 1);
                    o = Some(a);
                    ip += 2;
                }
                _ => bail!("unknown opcode {}: {}", ip, self.op(p[ip])),
            };
        }
        if o.is_none() { bail!("no output"); }
        Ok(o.unwrap())
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read, i: i32) -> BoxResult<i32> {
        let reader = io::BufReader::new(input);
        let mut p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        let mut ip = 0;
        let mut o = None;
        while self.op(p[ip]) != 99 {
            match self.op(p[ip]) {
                1 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2);
                    let c = p[ip + 3 as usize] as usize;
                    p[c] = a + b;
                    ip += 4;
                },
                2 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2);
                    let c = p[ip + 3 as usize] as usize;
                    p[c] = a * b;
                    ip += 4;
                },
                3 => {
                    let a = p[ip + 1];
                    p[a as usize] = i;
                    ip += 2;
                },
                4 => {
                    let a = self.val(&p, ip, 1);
                    o = Some(a);
                    ip += 2;
                },
                5 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2) as usize;
                    ip = if a != 0 { b } else { ip + 3 };
                },
                6 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2) as usize;
                    ip = if a == 0 { b } else { ip + 3 };
                },
                7 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2);
                    let c = p[ip + 3 as usize] as usize;
                    p[c] = if a < b { 1 } else { 0 };
                    ip += 4;
                },
                8 => {
                    let a = self.val(&p, ip, 1);
                    let b = self.val(&p, ip, 2);
                    let c = p[ip + 3 as usize] as usize;
                    p[c] = if a == b { 1 } else { 0 };
                    ip += 4;
                },
                _ => bail!("unknown opcode {}: {}", ip, self.op(p[ip])),
            };
        }
        if o.is_none() { bail!("no output"); }
        Ok(o.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, i: i32, o: Option<i32>) {
        let r = Day05 {}.part1_impl(&mut s.as_bytes(), i);
        if o == None { assert!(r.is_err()); }
        else { assert_eq!(r.unwrap(), o.unwrap()); }
    }

    #[test]
    fn part1() {
        test1("3,0,4,0,99", 1, Some(1));
        test1("1002,4,3,4,33", 1, None);
        test1("1101,100,-1,4,0", 1, None);
    }

    fn test2(s: &str, i: i32, o: i32) {
        assert_eq!(
            Day05 {}.part2_impl(&mut s.as_bytes(), i).unwrap(), o);
    }

    #[test]
    fn part2() {
        test2("3,9,8,9,10,9,4,9,99,-1,8", 1, 0);
        test2("3,9,8,9,10,9,4,9,99,-1,8", 8, 1);
        test2("3,9,7,9,10,9,4,9,99,-1,8", 1, 1);
        test2("3,9,7,9,10,9,4,9,99,-1,8", 8, 0);
        test2("3,3,1108,-1,8,3,4,3,99", 1, 0);
        test2("3,3,1108,-1,8,3,4,3,99", 8, 1);
        test2("3,3,1107,-1,8,3,4,3,99", 1, 1);
        test2("3,3,1107,-1,8,3,4,3,99", 8, 0);
        test2("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 0, 0);
        test2("3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9", 2, 1);
        test2("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", 0, 0);
        test2("3,3,1105,-1,9,1101,0,0,12,4,12,99,1", 2, 1);
        test2("3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99", 0, 999);
        test2("3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99", 8, 1000);
        test2("3,21,1008,21,8,20,1005,20,22,107,8,21,20,1006,20,31,1106,0,36,98,0,0,1002,21,125,20,4,20,1105,1,46,104,999,1105,1,46,1101,1000,1,20,4,20,1105,1,46,98,99", 80, 1001);
    }

}