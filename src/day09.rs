use simple_error::bail;
use std::error;
use std::io;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

struct Intcode {
    p: Vec<i64>,
    base: i64,
}

impl Intcode {
    fn new(p: &[i64]) -> Self { Self { p: p.to_vec(), base: 0 } }

    fn op(&self, c: i64) -> i64 { c % 100 }

    fn get(&mut self, a: usize) -> i64 {
        if a >= self.p.len() { self.p.resize(a + 1, 0); }
        self.p[a]
    }

    fn put(&mut self, a: usize, v: i64) {
//        eprintln!("put @{} {}", a, v);
        if a >= self.p.len() { self.p.resize(a + 1, 0); }
        self.p[a] = v;
    }

    fn addr(&mut self, ip: usize, i: usize) -> usize {
        let a = self.get(ip + i);
        let v = match self.get(ip) / vec![100, 1000, 10000][i - 1] % 10 {
            0 => a as usize,
            2 => (a  + self.base) as usize,
            _ => 0, // XXX
        };
        v
    }

    fn val(&mut self, ip: usize, i: usize) -> i64 {
        let a = self.get(ip + i);
        let v = match self.get(ip) / vec![100, 1000, 10000][i - 1] % 10 {
            1 => a,
            _ => {
                let addr = self.addr(ip, i);
                self.get(addr)
            },
        };
//        eprintln!("{} {} {} {} {} {}", ip, self.get(ip), i, a, self.base, v);
        v
    }

    fn run(&mut self, sender: mpsc::Sender<i64>, receiver: mpsc::Receiver<i64>) -> BoxResult<i64> {
        let mut ip = 0;
        let mut o = None;
        while { let op = self.get(ip); self.op(op) != 99 } {
//            eprintln!("{}: {} {} {} {}", ip, self.p[ip], self.p[ip + 1], self.p[ip + 2], self.p[ip + 3]);
            match self.op(self.p[ip]) {
                1 => {
                    let a = self.val(ip, 1);
                    let b = self.val( ip, 2);
                    let c = self.addr(ip, 3);
                    self.put(c, a + b);
                    ip += 4;
                },
                2 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2);
                    let c = self.addr(ip, 3);
                    self.put(c, a * b);
                    ip += 4;
                },
                3 => {
                    let a = self.addr(ip, 1);
//                    eprintln!(">recv {}", a);
                    self.put(a as usize, receiver.recv()?);
//                    eprintln!("<recv {}", self.get(a as usize));
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
                    let c = self.addr(ip, 3);
                    self.put(c, if a < b { 1 } else { 0 });
                    ip += 4;
                },
                8 => {
                    let a = self.val(ip, 1);
                    let b = self.val(ip, 2);
                    let c = self.addr(ip,3);
                    self.put(c, if a == b { 1 } else { 0 });
                    ip += 4;
                },
                9 => {
                    self.base += self.val(ip, 1);
//                    eprintln!("<base {}", self.base);
                    ip += 2;
                }
                _ => bail!("unknown opcode {}: {}", ip, self.op(self.p[ip])),
            };
        }
        if o.is_none() { bail!("no output"); }
        Ok(o.unwrap())
    }

    #[allow(dead_code)]
    fn arg(&self, ip: usize, offset: usize) -> String {
        let a = self.p[ip + offset].to_string();
        match self.p[ip] / vec![100, 1000, 10000][offset - 1] % 10 {
            0 => format!("@{}", a),
            1 => a,
            2 => format!("+{}", a),
            _ => String::from(""), // XXX
        }
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
                9 => {
                    println!("{}: base {}", ip, self.arg(ip, 1));
                    ip += 2;
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

pub struct Day09 {}

impl day::Day for Day09 {
    fn tag(&self) -> &str { "09" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), 1));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), 2));
    }
}

impl Day09 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read, i: i64) -> BoxResult<String> {
        let reader = io::BufReader::new(input);
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        thread::spawn(move || {
            let mut ic = Intcode::new(&p);
//            ic.disassemble();
            ic.run(output_sender, input_receiver).unwrap();
        });
        input_sender.send(i)?;
        Ok(output_receiver.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, o: &str) {
        assert_eq!(Day09 {}.part1_impl(&mut s.as_bytes(), 1).unwrap(), o);
    }

    #[test]
    fn part1() {
        test1("109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99", "109,1,204,-1,1001,100,1,100,1008,100,16,101,1006,101,0,99");
        test1("1102,34915192,34915192,7,4,7,99,0", "1219070632396864");
        test1("104,1125899906842624,99", "1125899906842624")
    }
}