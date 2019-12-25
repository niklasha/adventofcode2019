use simple_error::bail;
use closure::closure;
use evmap;
use evmap::{ReadHandle, WriteHandle};
use std::error;
use std::io;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;
use crate::day;

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

struct Intcode {
    p: Vec<i64>,
    base: i64,
}

impl Intcode {
    fn new(p: &[i64]) -> Self { Self { p: p.to_vec(), base: 0 } }

    fn op(&self, c: i64) -> i64 { c % 100 }

    fn get(&mut self, a: usize) -> i64 {
//        eprintln!("get {}", a);
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

    fn run(&mut self, sender: &mpsc::Sender<i64>, receiver: &mpsc::Receiver<i64>,
        request: &mpsc::Sender<()>, ack: &mpsc::Receiver<()>) -> BoxResult<i64> {
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
                    request.send(())?;
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
                    ack.recv()?;
//                    eprintln!("<<send");
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

pub struct Day19 {}

impl day::Day for Day19 {
    fn tag(&self) -> &str { "19" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let (grid_r, mut grid_w) = evmap::new();
        println!("{:?}", self.part1_impl(p, &grid_r, &mut grid_w));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let (grid_r, mut grid_w) = evmap::new();
        println!("{:?}", self.part2_impl(p, &grid_r, &mut grid_w));
    }
}

impl Day19 {
    fn part1_impl(self: &Self, p: Vec<i64>,
                  grid_r: &ReadHandle<(i64, i64), char>,
                  grid_w: &mut WriteHandle<(i64, i64), char>)
        -> BoxResult<i64> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let (start_sender, start_receiver) = mpsc::channel::<()>();

        let _cpu = thread::spawn(move || {
            while start_receiver.recv().is_ok() {
                let mut ic = Intcode::new(&p.clone());
                ic.run(&output_sender, &input_receiver, &request_sender, &ack_receiver)
                    .unwrap_or(0);
            }
        });

        Ok((0..50).flat_map(|x|
            (0..50).map(closure!(
                ref start_sender, ref request_receiver, ref input_sender,
                ref output_receiver, ref ack_sender
                |y| {
                    start_sender.send(()).unwrap();
                    request_receiver.recv().unwrap();
                    input_sender.send(x).unwrap();
                    request_receiver.recv().unwrap();
                    input_sender.send(y).unwrap();
                    let o = output_receiver.recv().unwrap();
                    ack_sender.send(()).unwrap();
                    o
                })))
            .sum())
    }

    fn part2_impl(self: &Self, p: Vec<i64>,
                  grid_r: &ReadHandle<(i64, i64), i64>,
                  grid_w: &mut WriteHandle<(i64, i64), i64>)
                  -> BoxResult<i64> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let (start_sender, start_receiver) = mpsc::channel::<()>();
        let _cpu = thread::spawn(move || {
            while start_receiver.recv().is_ok() {
                let mut ic = Intcode::new(&p.clone());
                ic.run(&output_sender, &input_receiver, &request_sender, &ack_receiver)
                    .unwrap_or(0);
            }
        });

        let (mut x, mut y) = (2, 3);
        let mut look_for_one = true;
        let mut ox = -1;
        loop {
            start_sender.send(()).unwrap();
            request_receiver.recv().unwrap();
            input_sender.send(x).unwrap();
            request_receiver.recv().unwrap();
            input_sender.send(y).unwrap();
            let o = output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            grid_w.update((x, y), o);
            grid_w.refresh();
            if x >= 99 && y >= 99 && o == 1  {
                if grid_r.get_and(&(x - 99, y - 99), |c| { c[0] }) == Some(1)
                    && grid_r.get_and(&(x - 99, y), |c| { c[0] }) == Some(1)
                    && grid_r.get_and(&(x, y - 99), |c| { c[0] }) == Some(1) {
                    break;
                };
            };
            if o == 0 {
                if look_for_one {
                    x += 1;
                } else {
                    x = ox;
                    y += 1;
                    look_for_one = true;
                };
            } else {
                if look_for_one {
                    ox = x;
                };
                x += 1;
                look_for_one = false;
            };
        };
        Ok((x - 99) * 10000 + (y - 99))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, v: usize) {
//        assert_eq!(Day19 {}.part1_impl(s), v);
    }

    #[test]
    fn part1() {
        test1("
..#..........
..#..........
#######...###
#.#...#...#.#
#############
..#...#...#..
..#####...^..", 76);
    }
}