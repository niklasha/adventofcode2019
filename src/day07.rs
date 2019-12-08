use permute;
use simple_error::bail;
use std::error;
use std::io;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day07 {}

impl day::Day for Day07 {
    fn tag(&self) -> &str { "07" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), 0));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input(), 0));
    }
}

struct Intcode {
    p: Vec<i64>,
}

impl Intcode {
    fn new(p: &[i64]) -> Self { Self { p: p.to_vec() } }

    fn op(&self, c: i64) -> i64 { c % 100 }

    fn val(&self, ip: usize, i: usize) -> i64 {
        let a = self.p[ip + i];
        let v = if self.p[ip] / vec![100, 1000][i - 1] % 10 == 0 { self.p[a as usize] } else { a };
        v
    }

    fn run(&mut self, sender: mpsc::Sender<i64>, receiver: mpsc::Receiver<i64>) -> BoxResult<i64> {
        let mut ip = 0;
        let mut o = None;
        while self.op(self.p[ip]) != 99 {
//            eprintln!("{}: {} {} {} {}", ip, self.p[ip], self.p[ip + 1], self.p[ip + 2], self.p[ip + 3]);
            match self.op(self.p[ip]) {
                1 => {
                    let a = self.val(ip, 1);
                    let b = self.val( ip, 2);
                    let c = self.p[ip + 3 as usize] as usize;
                    self.p[c] = a + b;
                    ip += 4;
                },
                2 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2);
                    let c = self.p[ip + 3 as usize] as usize;
                    self.p[c] = a * b;
                    ip += 4;
                },
                3 => {
                    let a = self.p[ip + 1];
//                    eprintln!(">recv");
                    self.p[a as usize] = receiver.recv()?;
//                    eprintln!("<recv {}", self.p[a as usize]);
                    ip += 2;
                },
                4 => {
                    let a = self.val(ip, 1);
                    o = Some(a);
//                    eprintln!(">send {}", a);
                    sender.send(a)?;
//                    eprintln!("<send");
                    ip += 2;
                },
                5 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2) as usize;
                    ip = if a != 0 { b } else { ip + 3 };
                },
                6 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2) as usize;
                    ip = if a == 0 { b } else { ip + 3 };
                },
                7 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2);
                    let c = self.p[ip + 3 as usize] as usize;
                    self.p[c] = if a < b { 1 } else { 0 };
                    ip += 4;
                },
                8 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2);
                    let c = self.p[ip + 3 as usize] as usize;
                    self.p[c] = if a == b { 1 } else { 0 };
                    ip += 4;
                },
                _ => bail!("unknown opcode {}: {}", ip, self.op(self.p[ip])),
            };
        }
        if o.is_none() { bail!("no output"); }
        Ok(o.unwrap())
    }

    #[allow(dead_code)]
    fn arg(&self, ip: usize, offset: usize) -> String {
        let a = self.p[ip + offset].to_string();
        if self.p[ip] / vec![100, 1000, 10000][offset - 1] % 10 == 0 { a } else { format!("#{}", a) }
    }

    #[allow(dead_code)]
    fn disassemble(&self) {
        let mut ip = 0;
        while ip < self.p.len() {
            match self.op(self.p[ip]) {
                1 => {
                    println!("{}: add {} {} {}", ip, self.arg(ip, 1), self.arg(ip, 2), self.arg(ip, 3));
                    ip += 4;
                },
                2 => {
                    println!("{}: mul {} {} {}", ip, self.arg(ip, 1), self.arg(ip, 2), self.arg(ip, 3));
                    ip += 4;
                },
                3 => {
                    println!("{}: in {}", ip, self.arg(ip, 1));
                    ip += 2;
                },
                4 => {
                    println!("{}: out {}", ip, self.arg(ip, 1));
                    ip += 2;
                },
                5 => {
                    println!("{}: jnz {} {}", ip, self.arg(ip, 1), self.arg(ip, 2));
                    ip += 3;
                },
                6 => {
                    println!("{}: jz {} {}", ip, self.arg(ip, 1), self.arg(ip, 2));
                    ip += 3;
                },
                7 => {
                    println!("{}: testlt {} {} {}", ip, self.arg(ip, 1), self.arg(ip, 2), self.arg(ip, 3));
                    ip += 4;
                },
                8 => {
                    println!("{}: testeq {} {} {}", ip, self.arg(ip, 1), self.arg(ip, 2), self.arg(ip, 3));
                    ip += 4;
                },
                99 => {
                    println!("{}: halt", ip);
                    ip += 1;
                },
                _ => {
                    println!("{}: data ({})", ip, self.p[ip]);
                    ip += 1;
                },
            };
        }
    }
}

impl Day07 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read, i: i64)
        -> BoxResult<i64> {
        let reader = io::BufReader::new(input);
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let max = permute::permutations_of(&(0..=4).collect::<Vec<i64>>())
            .map(|phases| {
                let phases = phases.collect::<Vec<_>>();
                let (first_sender, first_receiver)
                    = mpsc::channel::<i64>();
                let (_, receiver) = phases.into_iter().fold(
                    (first_sender.clone(), first_receiver),
                    |(sender, receiver), &phase| {
                        let p = p.to_vec();
                        let (next_sender, next_receiver)
                            = mpsc::channel::<i64>();
                        let next_sender_clone = next_sender.clone();
                        thread::spawn(move || {
                            Intcode::new(&p).run(next_sender, receiver).unwrap();
                        });
                        sender.send(phase).unwrap();
                        (next_sender_clone, next_receiver)
                    },
                );
                first_sender.send(i).unwrap();
                receiver.recv().unwrap()
            }).max();
        Ok(max.unwrap())
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read, i: i64)
        -> BoxResult<i64> {
        let reader = io::BufReader::new(input);
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let max = permute::permutations_of(&(5..=9).collect::<Vec<i64>>())
            .map(|phases| {
                let phases = phases.collect::<Vec<_>>();
                let (first_sender, first_receiver)
                    = mpsc::channel::<i64>();
                let (_, receiver) = phases.into_iter().fold(
                    (first_sender.clone(), first_receiver),
                    |(sender, receiver), &phase| {
                        let p = p.to_vec();
                        let (next_sender, next_receiver)
                            = mpsc::channel::<i64>();
                        let next_sender_clone = next_sender.clone();
                        thread::spawn(move || {
                            Intcode::new(&p).run(next_sender, receiver).unwrap();
                        });
                        sender.send(phase).unwrap();
                        (next_sender_clone, next_receiver)
                    },
                );
                first_sender.send(i).unwrap();
                receiver
                    .iter()
                    .map(|output| {
                        first_sender.send(output).unwrap();
                        output
                    })
                    .last()
                    .unwrap()
            }).max();
        Ok(max.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, v: i64) {
        assert_eq!(Day07 {}.part1_impl(&mut s.as_bytes(), 0).unwrap(), v);
    }

    #[test]
    fn part1() {
        test1("3,15,3,16,1002,16,10,16,1,16,15,15,4,15,99,0,0", 43210);
        test1("3,23,3,24,1002,24,10,24,1002,23,-1,23,101,5,23,23,1,24,23,23,4,23,99,0,0",
              54321);
        test1("3,31,3,32,1002,32,10,32,1001,31,-2,31,1007,31,0,33,1002,33,7,33,1,33,31,31,1,32,31,31,4,31,99,0,0,0",
              65210);
    }

    fn test2(s: &str, v: i64) {
        assert_eq!(Day07 {}.part2_impl(&mut s.as_bytes(), 0).unwrap(), v);
    }

    #[test]
    fn part2() {
        test2("3,26,1001,26,-4,26,3,27,1002,27,2,27,1,27,26,27,4,27,1001,28,-1,28,1005,28,6,99,0,0,5",
              139629729);
        test2("3,52,1001,52,-5,52,3,53,1,52,56,54,1007,54,5,55,1005,55,26,1001,54,-5,54,1105,1,12,1,53,54,53,1008,54,0,55,1001,55,1,55,2,53,55,53,4,53,1001,56,-1,56,1005,56,6,99,0,0,0,0,10",
              18216);
    }
}