use simple_error::bail;
use evmap;
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

pub struct Day15 {}

impl day::Day for Day15 {
    fn tag(&self) -> &str { "15" }

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

impl Day15 {
    fn part1_impl(self: &Self, p: Vec<i64>) -> BoxResult<usize> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let _cpu = thread::spawn(move || {
            let mut ic = Intcode::new(&p);
            ic.run(output_sender, input_receiver, request_sender, ack_receiver)
                .unwrap_or(0);
        });
        let (grid_r, mut grid_w) = evmap::new();
        let dirs: Vec<(i64, i64)> = vec![(0, -1), (0, 1), (-1, 0), (1, 0)];

        let to_move = |(dx, dy): (i64, i64)|
            dirs.iter().position(|&(x, y)| dx == x && dy == y).unwrap() as i64
                + 1;

        let origin = (0i64, 0i64);
        let mut pos = vec![origin];
        grid_w.insert(origin, (1, vec![]));
        grid_w.refresh();
        let mut track = vec![];

        let mut peek = |(px, py), (dx, dy), track: &mut Vec<_>| {
            request_receiver.recv().unwrap();
            input_sender.send(to_move((dx, dy))).unwrap();
            let v = output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            if v != 0 {
                request_receiver.recv().unwrap();
                input_sender.send(to_move((-dx, -dy))).unwrap();
                output_receiver.recv().unwrap();
                ack_sender.send(()).unwrap();
            };
            track.push((dx, dy).clone());
            grid_w.update((px + dx, py + dy), (v, track.clone()));
            grid_w.refresh();
            track.pop();
            grid_w.refresh();
//            eprintln!("peek ({}, {}): {}", px + dx, py + dy, v);
            v
        };

        let step = |(dx, dy): (i64, i64), forward, track: &mut Vec<_>| {
            let (dx, dy) = if forward { (dx, dy) } else { (-dx, -dy) };
            request_receiver.recv().unwrap();
            input_sender.send(to_move((dx, dy))).unwrap();
            output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            if forward { track.push((dx, dy).clone()); } else { track.pop(); };
        };

        let mut i = 0;
        let mut found = false;
        while !found {
            i += 1;
//            eprintln!("{}: {:?}", i, pos);

            let r = pos.iter().fold((vec![], false), |(mut pos, found), &(px, py)| {
                let (_, t) = grid_r.get_and(&(px, py), |x| x[0].clone()).unwrap();
//                eprintln!("({}, {}): {} {:?}", px, py, v, t);
                for m in &t { step(*m, true, &mut track); }
                let moves = dirs.iter().filter(|&(dx, dy)|
                    grid_r.get_and(&(px + *dx, py + *dy), |_| ()) == None);
                let moves: Vec<_> = moves
                    .map(|&(dx, dy)| ((px + dx, py + dy), peek((px, py), (dx, dy), &mut track)))
                    .filter(|&(_, x)| x != 0).collect();
                for (p, _) in &moves { pos.push(*p); }
                for m in t.iter().rev() { step(*m, false, &mut track) };
                (pos, found || moves.iter().any(|(_, x)| *x == 2))
            });
            pos = r.0;
            found = r.1;
        }
        Ok(i)
    }

    fn part2_impl(self: &Self, p: Vec<i64>) -> BoxResult<i64> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let _cpu = thread::spawn(move || {
            let mut ic = Intcode::new(&p);
            ic.run(output_sender, input_receiver, request_sender, ack_receiver)
                .unwrap_or(0);
        });
        let (grid_r, mut grid_w) = evmap::new();
        let dirs: Vec<(i64, i64)> = vec![(0, -1), (0, 1), (-1, 0), (1, 0)];

        let to_move = |(dx, dy): (i64, i64)|
            dirs.iter().position(|&(x, y)| dx == x && dy == y).unwrap() as i64
                + 1;

        let origin = (0i64, 0i64);
        let mut pos = vec![origin];
        grid_w.insert(origin, (1, vec![]));
        grid_w.refresh();
        let mut track = vec![];

        let mut peek = |(px, py), (dx, dy), track: &mut Vec<_>| {
            request_receiver.recv().unwrap();
            input_sender.send(to_move((dx, dy))).unwrap();
            let v = output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            if v != 0 {
                request_receiver.recv().unwrap();
                input_sender.send(to_move((-dx, -dy))).unwrap();
                output_receiver.recv().unwrap();
                ack_sender.send(()).unwrap();
            };
            track.push((dx, dy).clone());
            grid_w.update((px + dx, py + dy), (v, track.clone()));
            grid_w.refresh();
            track.pop();
            grid_w.refresh();
//            eprintln!("peek ({}, {}): {}", px + dx, py + dy, v);
            v
        };

        let step = |(dx, dy): (i64, i64), forward, track: &mut Vec<_>| {
            let (dx, dy) = if forward { (dx, dy) } else { (-dx, -dy) };
            request_receiver.recv().unwrap();
            input_sender.send(to_move((dx, dy))).unwrap();
            output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            if forward { track.push((dx, dy).clone()); } else { track.pop(); };
        };

        let mut oxygen = None;
        while !pos.is_empty() {
            let (p, o) = pos.iter().fold((vec![], None), |(mut pos, _oxygen), &(px, py)| {
                let (_, t) = grid_r.get_and(&(px, py), |x| x[0].clone()).unwrap();
                for m in &t { step(*m, true, &mut track); }
                let moves = dirs.iter().filter(|&(dx, dy)|
                    grid_r.get_and(&(px + *dx, py + *dy), |_| ()) == None);
                let moves: Vec<_> = moves
                    .map(|&(dx, dy)| ((px + dx, py + dy), peek((px, py), (dx, dy), &mut track)))
                    .filter(|&(_, x)| x != 0).collect();
                for (p, _) in &moves { pos.push(*p); }
                for m in t.iter().rev() { step(*m, false, &mut track) };
                (pos,
                 moves.iter().filter(|(_, x)| *x == 2).next().map(|(p, _)| *p))
            });
            pos = p;
            if o != None { oxygen = o; }
        }

        let mut t = 0;
        pos = vec![oxygen.unwrap()];
        while !pos.is_empty() {
//            eprintln!("{}: {:?}", t, pos);
            pos = pos.iter().fold(vec![], |mut pos, &(px, py)| {
                for (dx, dy) in dirs.iter().filter(|&(dx, dy)|
                    grid_r.get_and(&(px + *dx, py + *dy), |x| x[0].clone()).unwrap().0 == 1) {
                    let (x, y) = (px + *dx, py + *dy);
                    grid_w.update((x, y), (2, vec![]));
                    grid_w.refresh();
                    pos.push((x, y));
                }
                pos
            });
            t += 1;
        }
        Ok(t - 1)
    }
}