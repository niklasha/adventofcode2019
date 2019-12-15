use regex::Regex;
use std::collections::HashMap;
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day14 {}

impl day::Day for Day14 {
    fn tag(&self) -> &str { "14" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day14 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<i64> {
        let reader = io::BufReader::new(input);
        lazy_static! {
            static ref REACTION: Regex = Regex::new("(.+) => (.+) (.+)").unwrap();
        }
        let mut reactions = HashMap::new();
        reader.lines().for_each(|l| {
            let l = l.unwrap();
            let cap = REACTION.captures(&l).unwrap();
            let inputs: Vec<(i64, String)> = cap[1].split(", ").map(|s| {
                let s: Vec<_> = s.split(" ").collect();
                (s[0].to_string().parse().unwrap(), s[1].to_string())
            }).collect();
            reactions.insert(cap[3].to_string(),
                     (cap[2].to_string().parse::<i64>().unwrap(), inputs));
        });
        let (output, need) = ("FUEL", 1);
        let mut stock = HashMap::new();
        let mut ore = 0;
        fn extract(
            output: &str,
            need: i64,
            reactions: &HashMap<String, (i64, Vec<(i64, String)>)>,
            stock: &mut HashMap<String, i64>,
            ore: &mut i64) {
            let mut supply = stock.get(output).map(|v| *v).unwrap_or(0);
            if supply < need {
                let (output_cnt, inputs) = reactions.get(output).unwrap();
                let factor = (need - supply - 1) / output_cnt + 1;
                for (input_cnt, input) in inputs {
                    let amount = input_cnt * factor;
                    if input == "ORE" {
                        *ore += amount;
                    } else {
                        extract(input, amount, reactions, stock, ore);
                    }
                }
                supply += output_cnt * factor;
            }
            stock.insert(output.to_string(), supply - need);
        };
        extract(output, need, &reactions, &mut stock, &mut ore);
        Ok(ore)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<i64> {
        let reader = io::BufReader::new(input);
        lazy_static! {
            static ref REACTION: Regex = Regex::new("(.+) => (.+) (.+)").unwrap();
        }
        let mut reactions = HashMap::new();
        reader.lines().for_each(|l| {
            let l = l.unwrap();
            let cap = REACTION.captures(&l).unwrap();
            let inputs: Vec<(i64, String)> = cap[1].split(", ").map(|s| {
                let s: Vec<_> = s.split(" ").collect();
                (s[0].to_string().parse().unwrap(), s[1].to_string())
            }).collect();
            reactions.insert(cap[3].to_string(),
                             (cap[2].to_string().parse::<i64>().unwrap(), inputs));
        });
        let (output, need) = ("FUEL", 1);
        let mut stock = HashMap::new();
        let mut ore = 0;
        fn extract(
            output: &str,
            need: i64,
            reactions: &HashMap<String, (i64, Vec<(i64, String)>)>,
            stock: &mut HashMap<String, i64>,
            ore: &mut i64) -> bool {
            let mut supply = stock.get(output).map(|v| *v).unwrap_or(0);
            if supply < need {
                let (output_cnt, inputs) = reactions.get(output).unwrap();
                let factor = (need - supply - 1) / output_cnt + 1;
                for (input_cnt, input) in inputs {
                    let amount = input_cnt * factor;
                    if input == "ORE" {
                        *ore += amount;
                    } else {
                        extract(input, amount, reactions, stock, ore);
                    }
                }
                supply += output_cnt * factor;
            }
            stock.insert(output.to_string(), supply - need);
            true
        };
        let mut fuel = 0;
        loop {
            extract(output, need, &reactions, &mut stock, &mut ore);
            if ore >= 1000000000000 { break; }
            fuel += 1;
            if stock.iter().all(|(_, v)| *v == 0) { break; }
        }
        if ore < 1000000000000 {
            let cycles = 1000000000000 / ore;
            fuel *= cycles;
            ore *= cycles;
            ore -= 1000000000000;
            loop {
                extract(output, need, &reactions, &mut stock, &mut ore);
                if ore >= 0 { break; }
                fuel += 1;
            }
        }
        Ok(fuel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, v: i64) {
        assert_eq!(
            Day14 {}.part1_impl(&mut s.as_bytes()).unwrap(), v);
    }


    #[test]
    fn part1() {
        test1("10 ORE => 10 A
1 ORE => 1 B
7 A, 1 B => 1 C
7 A, 1 C => 1 D
7 A, 1 D => 1 E
7 A, 1 E => 1 FUEL", 31);
        test1("9 ORE => 2 A
8 ORE => 3 B
7 ORE => 5 C
3 A, 4 B => 1 AB
5 B, 7 C => 1 BC
4 C, 1 A => 1 CA
2 AB, 3 BC, 4 CA => 1 FUEL", 165);
        test1("157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT", 13312);
        test1("2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF", 180697);
        test1("171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX", 2210736);
    }

    fn test2(s: &str, v: i64) {
        assert_eq!(
            Day14 {}.part2_impl(&mut s.as_bytes()).unwrap(), v);
    }


    #[test]
    fn part2() {
        test2("157 ORE => 5 NZVS
165 ORE => 6 DCFZ
44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
179 ORE => 7 PSHF
177 ORE => 5 HKGWZ
7 DCFZ, 7 PSHF => 2 XJWVT
165 ORE => 2 GPVTF
3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT", 82892753);
        test2("2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
17 NVRVD, 3 JNWZP => 8 VPVL
53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
22 VJHF, 37 MNCFX => 5 FWMGM
139 ORE => 4 NVRVD
144 ORE => 7 JNWZP
5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
145 ORE => 6 MNCFX
1 NVRVD => 8 CXFTF
1 VJHF, 6 MNCFX => 4 RFSQX
176 ORE => 6 VJHF", 5586022);
        test2("171 ORE => 8 CNZTR
7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
114 ORE => 4 BHXH
14 VRPVC => 6 BMBT
6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
5 BMBT => 4 WPTQ
189 ORE => 9 KTJDG
1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
12 VRPVC, 27 CNZTR => 2 XDBXC
15 KTJDG, 12 BHXH => 5 XCVML
3 BHXH, 2 VRPVC => 7 MZWV
121 ORE => 7 VRPVC
7 XCVML => 6 RJRHP
5 BHXH, 4 VRPVC => 5 LTCX", 460664);
    }
}