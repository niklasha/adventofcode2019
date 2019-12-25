use closure::closure;
use evmap;
use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
use modinverse::modinverse;
//use modular::*;
use num::integer::{gcd, lcm};
use regex::Regex;
use simple_error::bail;
use std::collections::{HashMap, HashSet};
use std::error;
use std::io;
use std::io::BufRead;
use crate::day;

use std::fmt;
use std::ops::{Add, Mul, Sub};

/// Trait for modular operations on integers
///
/// Implementing this trait allows for conversion of integers to modular numbers, as well as
/// determining congruence relations between integers.
pub trait Modular {
    /// Returns the modular representation of an integer
    ///
    /// This is the idiomatic way of creating a new modulo number. Alternatively, the `modulo!`
    /// macro is provided, which provides the same functionality.
    fn to_modulo(self, modulus: u64) -> Modulo;

    /// Returns true if the two integers are congruent modulo `n`
    ///
    /// Congruence is determined by the relation:
    ///
    /// `a === b (mod n) if a - b = kn where k is some integer.`
    ///
    /// # Example
    /// ```
    /// # use modular::*;
    /// // Given some integers
    /// let a = 27;
    /// let b = 91;
    /// let c = -1;
    ///
    /// // Assert their congruence for different modulus values
    /// assert_eq!(a.is_congruent(b, 4), true);  // True:  27 - 91 = -64 => n = 4, k = -16
    /// assert_eq!(b.is_congruent(a, 5), false); // False: 91 - 27 = 64  => n = 5, k = 12.8
    /// assert_eq!(a.is_congruent(c, 4), true);  // True:  27 - -1 = 28  => n = 4, k = 7
    /// ```
    fn is_congruent(self, with: impl Into<i64>, modulus: u64) -> bool;
}

/// Holds the modulo representation of a number
///
/// In mathematics, the `%` operation returns the remainder obtained when an integer `a` is divided
/// by another `n`. For instance `32 % 6 = 2`: in this example, 32 can be written in terms of its
/// reminder after being divided by the specified dividend as `2 mod 6`. This is the modulo
/// representation of the number 32, with modulus 6.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Modulo {
    remainder: i64,
    modulus: u64,
}

impl Modulo {
    /// Returns the 'remainder' part of a modulo number
    pub fn remainder(self) -> i64 {
        self.remainder
    }

    /// Returns the modulus of a modulo number
    ///
    /// This is sometimes referred to as the 'dividend' as well
    pub fn modulus(self) -> u64 {
        self.modulus
    }
}

impl Modular for i64 {
    fn to_modulo(self, modulus: u64) -> Modulo {
        Modulo {
            remainder: self % modulus as i64,
            modulus,
        }
    }

    fn is_congruent(self, with: impl Into<i64>, modulus: u64) -> bool {
        (self - with.into()) % modulus as i64 == 0
    }
}

impl Add for Modulo {
    type Output = Self;

    /// Adds two `Modulo` numbers
    ///
    /// # Panics
    ///
    /// Panics if the two numbers have different modulus values
    fn add(self, rhs: Self) -> Self {
        if self.modulus() != rhs.modulus() {
            panic!("Addition is only valid for modulo numbers with the same dividend")
        }

        (self.remainder() + rhs.remainder()).to_modulo(self.modulus())
    }
}

impl Sub for Modulo {
    type Output = Self;

    /// Subtracts two `Modulo` numbers
    ///
    /// # Panics
    ///
    /// Panics if the two numbers have different modulus values
    fn sub(self, rhs: Self) -> Self {
        if self.modulus() != rhs.modulus() {
            panic!("Subtraction is only valid for modulo numbers with the same dividend")
        }

        if self.remainder() >= rhs.remainder() {
            (self.remainder() - rhs.remainder()).to_modulo(self.modulus())
        } else {
            (self.remainder() - rhs.remainder() + self.modulus() as i64).to_modulo(self.modulus())
        }
    }
}

impl Mul for Modulo {
    type Output = Self;

    /// Multiplies two `Modulo` numbers
    ///
    /// # Panics
    ///
    /// Panics if the two numbers have different modulus values
    fn mul(self, rhs: Self) -> Self {
        if self.modulus() != rhs.modulus() {
            panic!("Multiplication is only valid for modulo numbers with the same dividend")
        }

        (((self.remainder() as i128 * rhs.remainder() as i128) % self.modulus() as i128) as i64).to_modulo(self.modulus())
    }
}

impl fmt::Display for Modulo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} mod {:?}", self.remainder, self.modulus)
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//
//    #[test]
//    fn create_using_trait() {
//        assert!(27.to_modulo(5) == modulo!(2, 5));
//    }
//
//    #[test]
//    fn create_using_macro() {
//        assert!(modulo!(99, 4) == 99.to_modulo(4));
//    }
//
//    #[test]
//    fn get_remainder() {
//        assert_eq!(modulo!(26, 11).remainder(), 4);
//    }
//
//    #[test]
//    fn get_modulus() {
//        assert_eq!(modulo!(121, 17).modulus(), 17);
//    }
//
//    #[test]
//    fn add_successfully() {
//        assert!(modulo!(23, 4) + modulo!(11, 4) == modulo!(2, 4));
//    }
//
//    #[test]
//    #[should_panic]
//    fn add_panics_with_different_moduli() {
//        assert!(modulo!(23, 5) + modulo!(11, 6) == modulo!(2, 5));
//    }
//
//    #[test]
//    fn subtract_successfully() {
//        assert!(modulo!(22, 4) - modulo!(13, 4) == modulo!(1, 4));
//    }
//
//    #[test]
//    #[should_panic]
//    fn subtract_panics_with_different_moduli() {
//        assert!(modulo!(47, 43) - modulo!(5, 27) == modulo!(12, 13));
//    }
//
//    #[test]
//    fn multiply_successfully() {
//        assert!(modulo!(2, 4) * modulo!(19, 4) == modulo!(2, 4));
//    }
//
//    #[test]
//    #[should_panic]
//    fn multiply_panics_with_different_moduli() {
//        assert!(modulo!(91, 92) - modulo!(8, 9) == modulo!(12, 47));
//    }
//
//    #[test]
//    fn string_representation() {
//        let mod_new = modulo!(6, 7u64);
//        assert_eq!(format!("{}", mod_new), "6 mod 7");
//    }
//}

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day22 {}

impl day::Day for Day22 {
    fn tag(&self) -> &str { "22" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input(), 119315717514047usize, 1,75144274331587usize));
        println!("{:?}", self.part1_impl(&mut *input(), 119315717514047usize, 101741582076661usize,75144274331587usize));
//        println!("{:?}", self.part1_impl(&mut *input(), 10007, 1, 2019));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input(), 119315717514047usize, 101741582076661usize, 2020));
        println!("{:?}", self.part2_impl(&mut *input(), 119315717514047usize, 1, 47689744938338usize));
        println!("{:?}", self.part2_impl(&mut *input(), 119315717514047usize, 101741582076660usize, 2020));
//        println!("{:?}", self.part2_impl(&mut *input(), 119315717514047, 101741582076661, 2019));
//        println!("{:?}", self.part2_impl(&mut *input(), 10007, 1, 1510));
//        println!("{:?}", self.part2_impl(&mut *input(), 10007, 3, 4113));
    }
}

impl Day22 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read, len: usize, rep: usize, n: usize) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        lazy_static! {
            static ref CUT: Regex = Regex::new("cut (.+)").unwrap();
            static ref INCR: Regex = Regex::new("deal with increment (.+)").unwrap();
        }
//        let mut deck: Vec<_> = (0..len).collect();
//        let mut pos = n;
        let (mut f, mut o) = (1i64, 0);
        let mut trace = 115059975488624i64;
        reader.lines().for_each(|l| {
            let l = l.unwrap();
            if l == String::from("deal into new stack") {
//                eprintln!("new");
//                deck = deck.iter().map(|x| *x).rev().collect();
//                pos = len - pos - 1;
                f = (len as i64 - f) % len as i64;
                o = len - o - 1;
//                eprintln!("new: {} -> {}", trace, 119315717514047i64 - 1 - trace);
                trace = 119315717514047i64 - 1 - trace;
            } else {
                if let Some(cap) = CUT.captures(&l) {
//                    eprintln!("cut {:?}", cap);
                    let cut0: i64 = cap.get(1).unwrap().as_str().parse().unwrap();
                    let cut = if cut0 < 0 { (len as i64 + cut0) } else { cut0 } as usize;
//                    let mut new = vec![0; len];
                    let rest = len - cut;
//                    (&mut new[..rest]).clone_from_slice(&deck[cut..]);
//                    (&mut new[rest..]).clone_from_slice(&deck[0..cut]);
//                    deck = new;
//                    pos = (pos + rest) % len;
                    o = (o + rest) % len;
                    let otrace = trace;
                    let cut = if cut0 < 0 { (119315717514047i64 + cut0) } else { cut0 } as usize;
                    trace -= cut as i64;
                    if trace < 0 { trace += 119315717514047i64 }
//                    eprintln!("cut {}: {} -> {}", cut0, otrace, trace);
                } else if let Some(cap) = INCR.captures(&l) {
//                    eprintln!("incr {:?}", cap);
//                    let mut new = vec![0; len];
                    let inc: usize = cap.get(1).unwrap().as_str().parse().unwrap();
//                    for i in 0..len {
//                        new[(i * inc) % len] = deck[i];
//                    }
//                    deck = new;
//                    pos = (pos * inc) % len;
                    f = (f * inc as i64) % len as i64;
                    o = (o * inc) % len;
//                    eprintln!("inc {}: {} -> {}", inc, trace, trace * inc as i64 % 119315717514047i64);
                    trace = trace * inc as i64 % 119315717514047i64;
                }
            }
//            eprintln!("f {} o {}", f, o);
        });
        eprintln!("trace {}", trace);
//        eprintln!("deck {:?}", deck);
//        Ok(deck.iter().position(|&x| x == n).unwrap())
        let f = f as i128;
        let o = o as i128;
        let len = len as i128;
        let r = ((n as i128 * f) % len + len + o) % len;
        eprintln!("f {} o {} -> {}", f, o, r);
//        Ok(pos)
        // An invariant over the shuffle
        let x0 = (modinverse(len + 1 - f, len).unwrap() * o) % len;
        // Check the invariant
        let x1 = (x0 * f + o) % len;
        eprintln!("x0 {} x1 {}", x0, x1);
        let nf = power(f as i64, rep as i64, len as usize) as i128;
        let no = (len + 1 - nf) * x0 % len;
        eprintln!("f {} o {} nf {} no {}", f, o, nf, no);
        // Check the invariant over all shuffles
        let x2 = (x0 * nf + no) % len;
        eprintln!("x2 {}", x2);
        let r = (n as i128 * nf + no) % len;
        Ok(r as usize)
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read, len: usize, rep: usize, n: usize) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        lazy_static! {
            static ref CUT: Regex = Regex::new("cut (.+)").unwrap();
            static ref INCR: Regex = Regex::new("deal with increment (.+)").unwrap();
        }
        let (mut f, mut o) = (1i64, 0);
        reader.lines().collect::<Vec<_>>().into_iter().rev().for_each(|l| {
            let l = l.unwrap();
            if l == String::from("deal into new stack") {
//                eprintln!("new");
                f = (len as i64 - f) % len as i64;
                o = len - o - 1;
            } else {
                if let Some(cap) = CUT.captures(&l) {
//                    eprintln!("cut {:?}", cap);
                    let cut: i64 = cap.get(1).unwrap().as_str().parse().unwrap();
                    let cut = if cut < 0 { (len as i64 + cut) } else { cut } as usize;
                    let rest = len - cut;
                    o = (o + cut) % len;
                } else if let Some(cap) = INCR.captures(&l) {
//                    eprintln!("incr {:?}", cap);
                    let inc: usize = cap.get(1).unwrap().as_str().parse().unwrap();
                    f = ((f as i128 * modinverse(inc as i64, len as i64).unwrap() as i128) % len as i128) as i64;
                    o = ((o as u128 * modinverse(inc as i64, len as i64).unwrap() as u128) % len as u128) as usize;
                }
            }
//            eprintln!("f {} o {}", f, o);
        });
        let f = f as i128;
        let o = o as i128;
        let len = len as i128;
        // An invariant over the shuffle
        let x0 = (modinverse(len + 1 - f, len).unwrap() * o) % len;
        // Check the invariant
        let x1 = (x0 * f + o) % len;
        eprintln!("x0 {} x1 {}", x0, x1);
        // Compute the parameters for the repeated shuffle
        let nf = power(f as i64, rep as i64, len as usize) as i128;
        let no = (len + 1 - nf) * x0 % len;
        eprintln!("f {} o {} nf {} no {}", f, o, nf, no);
        // Check the invariant over the repeated shuffle
        let x2 = (x0 * nf + no) % len;
        eprintln!("x2 {}", x2);
        let r = (n as i128 * nf + no) % len;
        Ok(r as usize)
    }
}

fn power(a: i64, b: i64, m: usize) -> i64 {
    let m = m as u64;
    let mut r = 1.to_modulo(m);
    let mut b = b;
    let mut a = a.to_modulo(m);
    while b > 0 {
        if b & 1 != 0 {
            r = r * a;
        };
        b = b >> 1;
        a = a * a;
    }
    r.remainder()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, len: usize, n: usize, rep: usize, x: usize) {
        assert_eq!(
            Day22 {}.part1_impl(&mut s.as_bytes(), len, rep, n).unwrap(),
            x);
    }

    #[test]
    fn part1() {
        test1("deal with increment 7
deal into new stack
deal into new stack
", 10, 2, 1, 4);
        test1("cut 6
deal with increment 7
deal into new stack
", 10, 2, 1, 7);
        test1("deal into new stack
cut -2
deal with increment 7
cut 8
cut -4
deal with increment 7
cut 3
deal with increment 9
deal with increment 3
cut -1
", 10, 2, 1, 1);
    }

    fn test2(s: &str, len: usize, n: usize, rep: usize, x: usize) {
        assert_eq!(
            Day22 {}.part2_impl(&mut s.as_bytes(), len, rep, n).unwrap(),
            x);
    }

    #[test]
    fn part2() {
        test2("deal with increment 7
deal into new stack
deal into new stack
", 10, 4, 1, 2);
        test2("cut 6
deal with increment 7
deal into new stack
", 10, 7, 1, 2);
        test2("deal into new stack
cut -2
deal with increment 7
cut 8
cut -4
deal with increment 7
cut 3
deal with increment 9
deal with increment 3
cut -1
", 10, 1, 1, 2);
    }
}
