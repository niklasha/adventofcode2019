use simple_error::bail;
use closure::closure;
use evmap;
use std::error;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
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

pub struct Day23 {}

impl Day for Day23 {
    fn tag(&self) -> &str { "23" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", self.part2_impl(p));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
//        println!("{:?}", self.part1_impl(p, "NOT A J\nNOT B T\nAND D T\nAND H T\nOR T J\nNOT C T\nAND D T\nAND H T\nOR T J\nRUN\n"));
    }
}

impl Day23 {
    fn part1_impl(self: &Self, p: Vec<i64>)
        -> BoxResult<i64> {
        let config: Vec<_> = (0i64..50).map(|cpu| {
            let (input_sender, input_receiver) = mpsc::channel::<i64>();
            let (output_sender, output_receiver) = mpsc::channel::<i64>();
            let (request_sender, request_receiver) = mpsc::channel::<()>();
            let (ack_sender, ack_receiver) = mpsc::channel::<()>();
            let (start_sender, start_receiver) = mpsc::channel::<()>();
            let p = p.to_vec();
            let thread = thread::spawn(closure!(|| {
                while start_receiver.recv().is_ok() {
                    eprintln!("starting {}", cpu);
                    let mut ic = Intcode::new(&p);
                    ic.run(&output_sender, &input_receiver, &request_sender, &ack_receiver)
                        .unwrap_or(0);
//                eprintln!("cpu stopped");
                }
//            eprintln!("really stopped");
            }));
            start_sender.send(()).unwrap();
            request_receiver.recv().unwrap();
            eprintln!("{} started, configuring...", cpu);
            input_sender.send(cpu).unwrap();
            (cpu, thread, request_receiver, input_sender, output_receiver, ack_sender)
        }).collect();

        let (mux_in_sender, mux_in_receiver) = mpsc::channel::<(i64, i64, i64)>();

        let config: Vec<_> = config.into_iter().map(closure!(ref mux_in_sender
            |(cpu, thread, request_receiver, input_sender, output_receiver,
                 ack_sender)| {
                let mux_in_sender = mux_in_sender.clone();
                let (mux_out_sender, mux_out_receiver) = mpsc::channel::<(i64, i64)>();
                let input_sender_clone = input_sender.clone();
                thread::spawn(move || {
                    loop {
                        eprintln!("awaiting output from {}", cpu);
                        let dst = output_receiver.recv().unwrap();
                        ack_sender.send(()).unwrap();
                        eprintln!("{} wants to send to {}", cpu, dst);
                        let x = output_receiver.recv().unwrap();
                        ack_sender.send(()).unwrap();
                        eprintln!("{} got {}", cpu, x);
                        let y = output_receiver.recv().unwrap();
                        ack_sender.send(()).unwrap();
                        eprintln!("{} is sending ({}, {}) to {}", cpu, x, y, dst);
                        mux_in_sender.send((dst, x, y));
                    }
                });
                thread::spawn(move || {
                    loop {
//                        eprintln!("{} is polling", cpu);
                        request_receiver.recv().unwrap();
                        match mux_out_receiver.try_recv() {
                            Ok((x, y)) => {
                                input_sender.send(x);
                                request_receiver.recv().unwrap();
                                input_sender.send(y);
                            },
                            _ => {
//                                eprintln!("{} got nothing", cpu);
                                input_sender_clone.send(-1).unwrap();
                            },
                        };
                    }
                });
                (cpu, thread, mux_out_sender)
            })).collect();

        let mut r = 0;
        loop {
            let (dst, x, y) = mux_in_receiver.recv().unwrap();
            if dst != 255 {
                let (_, _, mux_out_sender) = &config[dst as usize];
                mux_out_sender.send((x, y));
                eprintln!("sent ({}, {}) to {}", x, y, dst);
            } else {
                r = y;
                break;
            }
        }
        for (_, thread, _) in config.into_iter() { thread.join(); }

        Ok(r)
    }

    fn part2_impl(self: &Self, p: Vec<i64>)
                  -> BoxResult<i64> {
        let config: Vec<_> = (0i64..50).map(|cpu| {
            let (input_sender, input_receiver) = mpsc::channel::<i64>();
            let (output_sender, output_receiver) = mpsc::channel::<i64>();
            let (request_sender, request_receiver) = mpsc::channel::<()>();
            let (ack_sender, ack_receiver) = mpsc::channel::<()>();
            let (start_sender, start_receiver) = mpsc::channel::<()>();
            let p = p.to_vec();
            let thread = thread::spawn(closure!(|| {
                while start_receiver.recv().is_ok() {
                    eprintln!("starting {}", cpu);
                    let mut ic = Intcode::new(&p);
                    ic.run(&output_sender, &input_receiver, &request_sender, &ack_receiver)
                        .unwrap_or(0);
//                eprintln!("cpu stopped");
                }
//            eprintln!("really stopped");
            }));
            start_sender.send(()).unwrap();
            request_receiver.recv().unwrap();
            eprintln!("{} started, configuring...", cpu);
            input_sender.send(cpu).unwrap();
            (cpu, thread, request_receiver, input_sender, output_receiver, ack_sender)
        }).collect();

        let (mux_in_sender, mux_in_receiver) = mpsc::channel::<(i64, i64, i64)>();
        let mut idle = Arc::new(Mutex::new(0i64));

        let config: Vec<_> = config.into_iter().map(closure!(ref mux_in_sender, ref mut idle
            |(cpu, thread, request_receiver, input_sender, output_receiver,
                 ack_sender)| {
                let mux_in_sender = mux_in_sender.clone();
                let (mux_out_sender, mux_out_receiver) = mpsc::channel::<(i64, i64)>();
                thread::spawn(move || {
                    loop {
//                        eprintln!("awaiting output from {}", cpu);
                        let dst = output_receiver.recv().unwrap();
                        ack_sender.send(()).unwrap();
//                        eprintln!("{} wants to send to {}", cpu, dst);
                        let x = output_receiver.recv().unwrap();
                        ack_sender.send(()).unwrap();
//                        eprintln!("{} got {}", cpu, x);
                        let y = output_receiver.recv().unwrap();
                        ack_sender.send(()).unwrap();
//                        eprintln!("{} is sending ({}, {}) to {}", cpu, x, y, dst);
                        mux_in_sender.send((dst, x, y));
                    }
                });
                let idle = Arc::clone(&idle);
                thread::spawn(closure!(|| {
                let mut idle_counter = 0;
                    loop {
//                        eprintln!("{} is polling", cpu);
                        request_receiver.recv().unwrap();
                        match mux_out_receiver.try_recv() {
                            Ok((x, y)) => {
                                input_sender.send(x);
                                request_receiver.recv().unwrap();
                                input_sender.send(y);
                                *idle.lock().unwrap() &= !(1 << cpu);
                                idle_counter = 0;
                            },
                            _ => {
//                                eprintln!("{} got nothing", cpu);
                                input_sender.send(-1).unwrap();
                                if idle_counter == 200000 {
                                    let mut idle = idle.lock().unwrap();
                                    *idle |= 1 << cpu;
                                    eprintln!("idling! mask: {:b}", *idle);
                                };
                                idle_counter += 1;
                            },
                        };
                    }
                }));
                (cpu, thread, mux_out_sender)
            })).collect();

        let (nat_sender, nat_receiver) = mpsc::channel::<(i64, i64)>();
        let (_, _, mux_out_sender_0) = &config[0];
        let mux_out_sender_0 = mux_out_sender_0.clone();
        let idle = Arc::clone(&idle);
        thread::spawn(move || {
            let mut last = None;
            let mut last_last = None;
            let mut wait_for_unidle = 0;
            loop {
                let d = nat_receiver.try_recv();
                if wait_for_unidle == 0 && *idle.lock().unwrap() == 0x3ffffffffffff && last != None {
                    eprintln!("all idle, sending {:?} to 0", last);
                    mux_out_sender_0.send(last.unwrap());
                    if last == last_last {
                        println!("TWICE in a row: {:?}", last);
                    };
                    last_last = last;
                    wait_for_unidle = 500000;
                } else {
                    if wait_for_unidle > 0 {
//                        eprintln!("unidle {}", wait_for_unidle);
                        wait_for_unidle -= 1;
                    }
                }
                if d.is_ok() {
                    eprintln!("nat received {:?}", d);
                    last = Some(d.unwrap());
                }
            }
        });

        let mut r = 0;
        loop {
            let (dst, x, y) = mux_in_receiver.recv().unwrap();
            if dst != 255 {
                let (_, _, mux_out_sender) = &config[dst as usize];
                mux_out_sender.send((x, y));
//                eprintln!("sent ({}, {}) to {}", x, y, dst);
            } else {
                nat_sender.send((x, y));
            }
        }
//        for (_, thread, _) in config.into_iter() { thread.join(); }

        Ok(r)
    }
}