use simple_error::bail;
use closure::closure;
use evmap;
use std::collections::HashSet;
use std::error;
use std::io;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread;
use crate::day;
use itertools::Itertools;

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

pub struct Day25 {}

impl day::Day for Day25 {
    fn tag(&self) -> &str { "25" }

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
//        println!("{:?}", self.part1_impl(p, "NOT A J\nNOT B T\nAND D T\nAND H T\nOR T J\nNOT C T\nAND D T\nAND H T\nOR T J\nRUN\n"));
    }
}

impl Day25 {
    fn rev(dir: &String) -> String {
        String::from(match dir.as_str() {
            "north" => "south",
            "east" => "west",
            "west" => "east",
            "south" => "north",
            _ => "Whoa",
        })
    }

    fn to_string(s: &HashSet<String>) -> String {
        let mut v: Vec<_> = s.iter().collect();
        v.sort();
        format!("{:?}", v)
    }

    fn part1_impl(self: &Self, p: Vec<i64>)
        -> BoxResult<i64> {
        let (input_sender, input_receiver) = mpsc::channel::<i64>();
        let (output_sender, output_receiver) = mpsc::channel::<i64>();
        let (request_sender, request_receiver) = mpsc::channel::<()>();
        let (ack_sender, ack_receiver) = mpsc::channel::<()>();
        let (start_sender, start_receiver) = mpsc::channel::<()>();
        let cpu = thread::spawn(closure!(|| {
            while start_receiver.recv().is_ok() {
//                eprintln!("starting cpu");
                let mut ic = Intcode::new(&p);
                ic.run(&output_sender, &input_receiver, &request_sender, &ack_receiver)
                    .unwrap();
//                eprintln!("cpu stopped");
            }
//            eprintln!("really stopped");
        }));

        let (room_r, mut room_w) = evmap::new();
        let (map_r, mut map_w) = evmap::new();
        start_sender.send(()).unwrap();
        let mut line = String::new();
        let (mut doors, mut inventory, mut drop, mut pickup, mut backtrack, mut more_to_take)
            = (false, false, false, true, true, false);
        let (mut neighbours, mut items) = (vec![], vec![]);
        let mut track: Vec<(String, String)> = vec![];
        let mut name = String::new();
        let mut sensor = None;
        let mut my_items: HashSet<String> = HashSet::new();
//        let mut tried = HashSet::new();
        // 0 - Map and collect all items
        // 1 - Go to Checkpoint and drop items
        // 2 - Pick up combinations of items and try entering the sensor
        let mut phase = 0;
        let mut at_checkpoint = false;
        let mut combinations: Vec<Vec<_>> = vec![];
        // The combination we try
        let mut comb = 0;
        // the item inside the current combination
        let mut it = 0;
        let mut all_items = vec![];
        loop {
            let c = output_receiver.recv().unwrap();
            ack_sender.send(()).unwrap();
            if c as u8 == b'\n' {
                {
                    let line = line.as_str();
                    eprintln!("{}", line);
                    if line.starts_with("== ") {
                        name = (&line[3..]).to_string();
                        if !track.is_empty() {
                            let (last_name, last_dir)
                                = track.last().unwrap();
                            map_w.update(
                                (last_name.clone(), last_dir.clone()),
                                name.clone());
                            map_w.update(
                                (name.clone(), Day25::rev(last_dir)),
                                last_name.clone());
                            map_w.refresh();
                        }
                        at_checkpoint = name.starts_with("Security Checkpoint");
                        if at_checkpoint {
                            match phase {
                                0 => sensor = Some(track.clone()),
                                1 => drop = true,
                                _ => ()
                            };
                        };
                    } else if line.contains("ejected back") {
//                        tried.insert(Day25::to_string(&my_items));
                        if phase >= 2 {
                            let (last_name, last_dir)
                                = track.last().unwrap();
//                        eprintln!("removing {} {}", last_name, last_dir);
                            map_w.remove(
                                (last_name.clone(), last_dir.clone()),
                                name.clone());
                            map_w.remove(
                                (name.clone(), Day25::rev(last_dir)),
                                last_name.clone());
                            map_w.refresh();
                            let step = track.pop().unwrap();
                            drop = true;
                            if line.contains("lighter") {
                                sensor = Some(vec![step]);
                                pickup = false;
                                drop = true;
                            }
                        }
                    } else if doors {
                        if line != "" {
                            let dir = (&line[2..]).to_string();
                            if !at_checkpoint || dir.as_str() != "north" || phase > 0 {
                                neighbours.push(dir);
                            }
                        } else { doors = false; }
                    } else if inventory {
                        if line != "" {
                            let item = &line[2..];
                            if item != "infinite loop" && item != "photons"
                                && item != "giant electromagnet"
                                && item != "molten lava"
                                && item != "escape pod" {
                                items.push(item.to_string());
                            }
                        } else {
                            if phase == 2 {
                                let cnt = items.len();
                                all_items = items.clone();
                                eprintln!("compute all combinations of {:?}", items);
                                combinations = (1..cnt).flat_map(|n| (0..cnt).combinations(n).collect_vec()).collect();
                                comb = 0;
                                eprintln!("combinations {:#?}", combinations);
                                phase += 1;
                                more_to_take = true;
                                pickup = false;
                            }
                            inventory = false;
                        }
                    }
                    match line {
                        "Command?" => {
                            eprintln!(
                                "phase {} neighbours {:?} items {:?} track {:?} sensor {:?} my_items {:?}", // tried {:?}",
                                phase, neighbours, items, track, sensor,
                                my_items,
//                                tried
                            );
                            eprint!(">>> ");
                            let (movement, cmd) = if phase == 3 && more_to_take {
                                let combo = &combinations[comb];
                                let item = combo[it];
                                it += 1;
                                if it == combo.len() {
                                    more_to_take = false;
                                    it = 0;
                                    comb += 1;
                                }
                                let item = &all_items[item];
                                my_items.insert(item.clone());
                                (false, String::from("take ") + item)
                            } else if drop && !my_items.is_empty() {
                                let item = my_items.iter().next().unwrap().to_owned();
                                my_items.remove(&item);
                                if my_items.is_empty() {
                                    drop = false;
                                    if phase == 1 { phase += 1; }
                                    else if phase == 3 { more_to_take = true; }
                                }
                                (false, String::from("drop ") + item.as_str())
                            } else if phase == 0 && !items.is_empty() {
                                let item = items.pop().unwrap();
                                my_items.insert(item.clone());
                                (false, String::from("take ") + item.as_str())
                            } else if let Some(dir) = neighbours.iter()
                                .filter(|&dir| {
//                                    eprintln!("checking {} {}", name, *dir);
                                    let key = (name.clone(), (*dir).clone());
                                    let x = map_r.get_and(&key, |v| !v.is_empty());
//                                    eprintln!("{:?}", x);
                                    x == None || x == Some(false)
                                }).next() {
                                track.push((name.clone(), (*dir).clone()));
                                (true, (*dir).clone())
                            } else if backtrack && !track.is_empty() {
                                let (_, dir) = track.pop().unwrap();
                                (true, Day25::rev(&dir))
                            } else if sensor != None {
                                if phase == 0 { phase += 1; };
                                backtrack = false;
                                let mut iter = sensor.unwrap().into_iter();
                                let (_, dir) = iter.next().unwrap();
                                let path: Vec<_> = iter.collect();
                                sensor = if path.is_empty() { None } else { Some(path) };
                                track.push((name.clone(), dir.clone()));
                                (true, dir.clone())
                            } else { break; };
                            (cmd + "\n").chars().for_each(|c| {
                                request_receiver.recv().unwrap();
                                input_sender.send(c as i64);
                                eprint!("{}", c);
                            });
                            room_w.update(
                                name.clone(),
                                (neighbours.clone(), items.clone()));
                            room_w.refresh();
                        },
                        "Doors here lead:" => {
                            neighbours = vec![];
                            doors = true
                        },
                        "Items here:" => {
                            items = vec![];
                            inventory = true
                        },
                        _ => (),
                    };
                }
                line = String::new();
            } else { line.push(c as u8 as char); }
        }
        cpu.join().unwrap();
        Ok(0)
    }

    fn part2_impl(self: &Self, p: Vec<i64>)
                  -> BoxResult<i64> {
        Ok(0)
    }
}