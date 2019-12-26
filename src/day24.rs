use itertools::Itertools;
use itertools::FoldWhile::{Continue, Done};
//use modular::*;
use std::collections::{HashMap, HashSet};
use std::error;
use std::io;
use std::io::Read;
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
//        assert!(modulo!(24, 4) - modulo!(13, 4) == modulo!(1, 4));
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

pub struct Day24 {}

impl day::Day for Day24 {
    fn tag(&self) -> &str { "24" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input(), 200));
    }
}

impl Day24 {
    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let init: Vec<_> = reader.bytes()
            .map(|b| match b.unwrap() as char { '.' => 0, '#' => 1, _ => 2, })
            .filter(|&b| b != 2).collect();
        let (s, _) = (0..)
            .fold_while((init, HashSet::new()),  |(s, seen), _| {
                if seen.contains(&s) { Done((s, seen)) }
                else {
                    let mut seen = seen;
                    seen.insert(s.clone());
                    let ns = (0..s.len()).map(|i| {
                        let n = if i >= 5 { s[i - 5] } else { 0 }
                            + if i < 20 { s[i + 5] } else { 0 }
                            + if i % 5 >= 1 { s[i - 1] } else { 0 }
                            + if i % 5 < 4 { s[i + 1] } else { 0 };
                        if s[i] == 1 && n != 1 { 0 }
                        else if s[i] == 0 && (n == 1 || n == 2) { 1 }
                        else { s[i] }
                    }).collect();
                    Continue((ns, seen))
                }
        }).into_inner();
        Ok(s.into_iter().enumerate().map(|(i, b)| b << i).sum())
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read, n: usize) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let mut init = HashMap::new();
        init.insert(0, reader.bytes()
            .map(|b| match b.unwrap() as char { '.' => 0, '#' => 1, _ => 2, })
            .filter(|&b| b != 2).collect::<Vec<_>>());
        let len = 25;
        let z = vec![0; len];
        let (s, _) = (0..n).fold((init, 0),  |(sp, dim), _| {
            let mut nsp = HashMap::new();
            for d in -(dim + 1)..=(dim + 1) {
                let s = sp.get(&d).unwrap_or(&z);
                let so = sp.get(&(d - 1)).unwrap_or(&z);
                let si = sp.get(&(d + 1)).unwrap_or(&z);
                let ns = (0..len).map(|i| {
                    let n = if i >= 5 {
                        if i != 17 { s[i - 5] } else { (&si[20..len]).iter().sum() }
                    } else { so[7] }
                        + if i < 20 {
                        if i != 7 { s[i + 5] } else { (&si[0..5]).iter().sum() }
                    } else { so[17] }
                        + if i % 5 >= 1 {
                        if i != 13 { s[i - 1] } else { (4..len).step_by(5).map(|i| si[i]).sum() }
                    } else { so[11] }
                        + if i % 5 < 4 {
                        if i != 11 { s[i + 1] } else { (0..len).step_by(5).map(|i| si[i]).sum() }
                    } else { so[13] };
                    if i == 12 || s[i] == 1 && n != 1 { 0 } else if s[i] == 0 && (n == 1 || n == 2) { 1 } else { s[i] }
                }).collect();
                nsp.insert(d, ns);
            }
            let mut dim = dim;
//            eprintln!("dim {} nsp {:?}", dim, nsp);
            if nsp.get(&-(dim + 1)) != Some(&z)
                || nsp.get(&(dim + 1)) != Some(&z) {
                dim += 1;
            };
            (nsp, dim)
        });
//        eprintln!("{:?}", s);
        let v = s.iter();
        let m: Vec<i32> = v.map(|(d, v)| {
            let i = v.iter();
            let s = i.sum();
//            eprintln!("{} {:?} {}", d, v, s);
            s
        }).collect();
        let s2: i32 = m.iter().sum();
        Ok(s2 as usize)
//        Ok(s.values().map(|&v| v.iter().sum()).sum() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, x: usize) {
        assert_eq!(
            Day24 {}.part1_impl(&mut s.as_bytes()).unwrap(),
            x);
    }

    #[test]
    fn part1() {
        test1("....#
#..#.
#..##
..#..
#....", 2129920);
    }

    fn test2(s: &str, x: usize) {
        assert_eq!(
            Day24 {}.part2_impl(&mut s.as_bytes(), 10).unwrap(),
            x);
    }

    #[test]
    fn part2() {
        test2("....#
#..#.
#..##
..#..
#....", 99);
    }
}
