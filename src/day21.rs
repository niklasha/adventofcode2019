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

pub struct Day21 {}

impl day::Day for Day21 {
    fn tag(&self) -> &str { "21" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", self.part1_impl(p, "NOT A T\nNOT T T\nAND B T\nAND C T\nNOT T T\nAND D T\nNOT T T\nNOT T J\nWALK\n"));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        let reader = io::BufReader::new(input());
        let p = reader.split(b',')
            .map(|v| String::from_utf8(v.unwrap()).unwrap())
            .map(|s| s.trim_end().parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        println!("{:?}", self.part1_impl(p, "NOT A J\nNOT B T\nAND D T\nAND H T\nOR T J\nNOT C T\nAND D T\nAND H T\nOR T J\nRUN\n"));
    }
}

impl Day21 {
    fn part1_impl(self: &Self, p: Vec<i64>, prog: &str)
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
//                eprintln!("cpu stopped");
            }
//            eprintln!("really stopped");
        });

        let mut r = 0;
        let mut stop = 99;
        let mut line = String::new();
        let mut i = 0;
        let mut mode = 0;
        start_sender.send(()).unwrap();
        loop {
            let o = output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            if o > 255 {
                r = o;
                break;
            } else {
//                eprint!("{}", o as u8 as char);
                line += format!("{}", o as u8 as char).as_str();
                if o == 10 {
                    if (stop < 99) { eprint!("{}", line); };
                    line = String::new();
                    i += 1;
                    if i == stop { break; }
                } else if o == 64 && i % 5 == 0 {
                    stop = i + 2;
                }
            }
            if mode == 0 && o == 10 {
                mode += 1;
                prog.chars().into_iter().for_each(|c| {
                    request_receiver.recv().unwrap();
                    input_sender.send(c as u8 as i64).unwrap();
                });
            }
        }
        Ok(r)
    }
}
