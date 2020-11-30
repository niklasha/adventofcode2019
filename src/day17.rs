use simple_error::bail;
use evmap;
use std::error;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;
use crate::day::*;
use evmap::{ReadHandle, WriteHandle};

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

    fn run(&mut self, sender: mpsc::Sender<i64>, receiver: mpsc::Receiver<i64>,
        request: mpsc::Sender<()>, ack: mpsc::Receiver<()>) -> BoxResult<i64> {
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

pub struct Day17 {}

impl Day for Day17 {
    fn tag(&self) -> &str { "17" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let (grid_r, mut grid_w) = evmap::new();
        let mut o = (0, 0);
        let mut dir = (0, 0);
        println!("{:?}", self.part1_impl(p, &grid_r, &mut grid_w, &mut o, &mut dir));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        let mut p2 = p.clone();
        p2[0] = 2;
        let mut o = (0, 0);
        let mut dir = (0, 0);
        let (grid_r, mut grid_w) = evmap::new();
        self.part1_impl(p, &grid_r, &mut grid_w, &mut o, &mut dir).unwrap();
        println!("{:?}", self.part2_impl(p2, &grid_r, &mut grid_w, o, dir));
    }
}

impl Day17 {
    fn part1_impl(self: &Self, p: Vec<i64>,
                  grid_r: &ReadHandle<(i32, i32), char>,
                  grid_w: &mut WriteHandle<(i32, i32), char>,
                  origin: &mut (i32, i32), dir: &mut (i32, i32))
        -> BoxResult<i32> {
        let (_input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, _request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let _cpu = thread::spawn(move || {
            let mut ic = Intcode::new(&p);
            ic.run(output_sender, input_receiver, request_sender, ack_receiver)
                .unwrap_or(0);
        });

        let mut s = String::new();
        let (mut x, mut y) = (0, 0);
        let mut t = 0;
        while match output_receiver.recv() {
            Ok(b) => {
                ack_sender.send(()).unwrap();
                let c = b as u8 as char;
                s.push(c);
                grid_w.update((x, y).clone(), c);
                grid_w.refresh();
                match c {
                    '#' => {
                        if y > 0 && x > 0
                            && grid_r.get_and(&(x, y - 1), |c| c[0]) == Some('#')
                            && grid_r.get_and(&(x - 1, y - 1), |c| c[0]) == Some('#')
                            && grid_r.get_and(&(x + 1, y - 1), |c| c[0]) == Some('#') {
                            t += x * (y - 1);
                        };
                        x += 1;
                    },
                    '.' => { x += 1; },
                    '^' => { *origin = (x, y); *dir = (0, -1); x += 1; }
                    'v' => { *origin = (x, y); *dir = (0, 1); x += 1; }
                    '<' => { *origin = (x, y); *dir = (-1, 0); x += 1; }
                    '>' => { *origin = (x, y); *dir = (1, 0); x += 1; }
                    '\n' => { x = 0; y += 1; },
                    _ => (),
                }
                true
            },
            Err(_) => false,
        } {};
        eprintln!("{}", s);

        Ok(t)
    }

    fn part2_impl(self: &Self, p: Vec<i64>,
                  grid_r: &ReadHandle<(i32, i32), char>,
                  grid_w: &mut WriteHandle<(i32, i32), char>,
                  origin: (i32, i32), dir: (i32, i32))
        -> BoxResult<i64> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let _cpu = thread::spawn(move || {
            let mut ic = Intcode::new(&p);
            ic.run(output_sender, input_receiver, request_sender, ack_receiver)
                .unwrap_or(0);
        });
        let mut s = String::new();
        for _i in 0..2047 {
            match output_receiver.recv().unwrap() as u8 as char {
                '\n' => { eprintln!("{}", s); s = String::new(); },
                c => { s.push(c); }
            };
            ack_sender.send(()).unwrap();
        }

//        eprintln!("{:?} {:?}", origin, dir);

        let is_ok = |(x, y), (dx, dy)| {
            grid_r.get_and(&(x + dx, y + dy), |c| c[0]) == Some('#')
        };

        let run = |(x, y), (dx, dy)| {
            let len = (1..).position(|i| !is_ok((x, y), (dx * i, dy * i)))
                .unwrap()
                as i32;
            if len == 0 { None } else { Some(((x + len * dx, y + len * dy), len, (dx, dy))) }
        };

        let mut n = 0;
        let mut pos = origin;
        let mut last = None;
        let mut face = dir;
        loop {
            let (_m, next, next_face) =
                if let Some((next, dist, next_face)) = run(pos, face) {
                    (format!("{}", dist), next, next_face)
                } else if let Some((next, dist, next_face)) = run(pos, (face.1, -face.0)) {
                    (format!("L,{}", dist), next, next_face)
                } else if let Some((next, dist, next_face)) = run(pos, (-face.1, face.0)) {
                    (format!("R,{}", dist), next, next_face)
                } else {
                    let (next, dist, next_face) = run(pos, (-face.0, -face.1)).unwrap();
                    (format!("L,L,{}", dist), next, next_face)
                };
            if Some(next) == last { break; }
//            eprintln!("{:?}", _m);
            face = next_face;
            last = Some(pos);
            pos = next;
        }
        // XXX
        let instructions = "A,B,B,A,B,C,A,C,B,C
L,4,L,6,L,8,L,12
L,8,R,12,L,12
R,12,L,6,L,6,L,8
y
";
        for c in instructions.chars() {
            request_receiver.recv().unwrap();
            input_sender.send(c as i64).unwrap();
            match c {
                '\n' => {
                    loop {
                        let mut done = false;
                        match output_receiver.recv().unwrap() as u8 as char {
                            '\n' => { eprintln!("{}", s); s = String::new(); done = true; },
                            c => { s.push(c); }
                        };
                        ack_sender.send(()).unwrap();
                        if done { break; }
                    };
                },
                _ => (),
            }
        }

        let mut s = String::new();
        let (mut x, mut y) = (0, 0);
        while match output_receiver.recv() {
            Ok(b) => {
                ack_sender.send(()).unwrap();
                let c = b as u8 as char;
                s.push(c);
                match c {
                    '#' => {
                        x += 1;
                    },
                    '.' => { x += 1; },
                    '^' => { x += 1; }
                    'v' => { x += 1; }
                    '<' => { x += 1; }
                    '>' => { x += 1; }
                    '\n' => {
                        eprintln!("{}", s);
                        s = String::new();
                        x = 0;
                        y += 1;
                    },
                    _ => {
                        eprintln!("whoa: {}", b);
                        n += b;
                        x += 1;
                    },
                }
                true
            },
            Err(_) => false,
        } {};

        Ok(n)
    }
}

#[cfg(test)]
mod tests {
//    use super::*;

    fn test1(s: &str, v: usize) {
//        assert_eq!(Day17 {}.part1_impl(s), v);
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

//    fn test2(s: &str, n: usize, v: &str) {
////        assert_eq!(Day16 {}.part2_impl(s, n).unwrap(), v);
//        assert_eq!(Day17 {}.part2_helper(s, n, 1, 0).unwrap(), v);
//    }
//
////    #[test]
//    fn part2() {
//        test2("56781234", 10, "");
////        test2("03036732577212944063491565474664", 100, "84462026");
//    }
}