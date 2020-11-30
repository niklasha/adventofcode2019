use simple_error::bail;
use evmap;
use std::error;
use std::io::BufRead;
use std::sync;
use std::sync::mpsc;
use std::thread;
use crate::day::*;

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

pub struct Day11 {}

impl Day for Day11 {
    fn tag(&self) -> &str { "11" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", self.part1_impl(p));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", self.part2_impl(p));
    }
}

impl Day11 {
    fn part1_impl(self: &Self, p: Vec<i64>) -> BoxResult<i64> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let (grid_r, mut grid_w) = evmap::new();
        let grid_r_clone = grid_r.clone();
        let pos_r = sync::Arc::new(sync::RwLock::new((0, 0)));
        let pos_r_2 = pos_r.clone();
        let pos_w = pos_r.clone();
        let mut dir = (0, -1);
        let area_r = sync::Arc::new(sync::RwLock::new(0i64));
        let area_w = area_r.clone();
        let cpu = thread::spawn(move || {
            let mut ic = Intcode::new(&p);
//            ic.disassemble();
            ic.run(output_sender, input_receiver, request_sender, ack_receiver).unwrap();
        });
        thread::spawn(move || {
            while request_receiver.recv().is_ok() {
                let pos = *pos_r.read().unwrap();
                let input = grid_r
                    .get_and(&pos, |rs| { *rs.first().unwrap_or(&0) })
                    .unwrap_or(0);
//                eprintln!("input for {:?}: {}", pos, input);
                input_sender.send(input).unwrap();
            }
        });
        thread::spawn(move || {
            loop {
                let output = output_receiver.recv();
                if output.is_err() { break; }
                let output = output.unwrap();
                let pos = *pos_r_2.read().unwrap();
                let mut area = area_w.write().unwrap();
                if !grid_r_clone.contains_key(&pos) { *area += 1; }
//                eprintln!("output for {:?} {} area {}", pos, output, *area);
                grid_w.update(pos, output);
                grid_w.refresh();
                ack_sender.send(()).unwrap();
                let output = output_receiver.recv().unwrap();
                match output {
                    0 => dir = (dir.1, -dir.0),
                    1 => dir = (-dir.1, dir.0),
                    _ => { eprintln!("invalid direction") },
                }
                let mut pos = pos_w.write().unwrap();
                *pos = ((*pos).0 + dir.0, (*pos).1 + dir.1);
//                eprintln!("turn {} -> {:?}", output, *pos);
                ack_sender.send(()).unwrap();
            }
        });
        cpu.join().unwrap();
        let area = *area_r.read().unwrap();
        Ok(area)
    }

    fn part2_impl(self: &Self, p: Vec<i64>) -> BoxResult<String> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let (grid_r, mut grid_w) = evmap::new();
        let grid_r_clone = grid_r.clone();
        grid_w.insert((0, 0), 1);
        grid_w.refresh();
        let pos_r = sync::Arc::new(sync::RwLock::new((0, 0)));
        let pos_r_2 = pos_r.clone();
        let pos_w = pos_r.clone();
        let mut dir = (0, -1);
        thread::spawn(move || {
            let mut ic = Intcode::new(&p);
//            ic.disassemble();
            ic.run(output_sender, input_receiver, request_sender, ack_receiver).unwrap();
        });
        thread::spawn(move || {
            while request_receiver.recv().is_ok() {
                let pos = *pos_r.read().unwrap();
                let input = grid_r
                    .get_and(&pos, |rs| { *rs.first().unwrap_or(&0) })
                    .unwrap_or(0);
//                    eprintln!("input for {:?}: {}", pos, input);
                input_sender.send(input).unwrap();
            }
            for y in 0..6 {
                let mut s = String::from("");
                for x in 0..41 {
                    let c = grid_r
                        .get_and(&(x, y), |rs| { *rs.first().unwrap_or(&0) })
                        .unwrap_or(0);
                    s = format!("{}{}", s, if c == 0 { " " } else { "." });
                }
            }
        });
        let painter = thread::spawn(move || {
            loop {
                let output = output_receiver.recv();
                if output.is_err() { break; }
                let output = output.unwrap();
                let pos = *pos_r_2.read().unwrap();
//                eprintln!("output for {:?} {}", pos, output);
                grid_w.update(pos, output);
                grid_w.refresh();
                ack_sender.send(()).unwrap();
                let output = output_receiver.recv().unwrap();
                match output {
                    0 => dir = (dir.1, -dir.0),
                    1 => dir = (-dir.1, dir.0),
                    _ => { eprintln!("invalid direction") },
                }
                let mut pos = pos_w.write().unwrap();
                *pos = ((*pos).0 + dir.0, (*pos).1 + dir.1);
//                    eprintln!("turn {} -> {:?}", output, *pos);
                ack_sender.send(()).unwrap();
            }
            let mut output = String::from("");
            for y in 0..6 {
                let mut s = String::from("");
                for x in 0..41 {
                    let c = grid_r_clone
                        .get_and(&(x, y), |rs| { *rs.first().unwrap_or(&0) })
                        .unwrap_or(0);
                    s = format!("{}{}", s, if c == 0 { " " } else { "." });
                }
                output = format!("{}\n{}", output, s);
            }
            output
        });
        let output = painter.join().unwrap();
        eprintln!("{}", output);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(v: Vec<i64>, o: i64) {
        let mut p: Vec<i64> = v.iter().flat_map(|i| vec![104, *i]).collect();
        p.push(99);
        assert_eq!(Day11 {}.part1_impl(p).unwrap(), o);
    }

    #[test]
    fn part1() {
        let v: Vec<i64> = vec![1, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0];
        test1(v, 6);
    }
}