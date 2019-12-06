use simple_error::bail;
use std::collections;
use std::error;
use std::io;
use std::io::BufRead;
use topological_sort;
use crate::day;

pub type BoxResult<T> = Result<T, Box<dyn error::Error>>;

pub struct Day06 {}

impl day::Day for Day06 {
    fn tag(&self) -> &str { "06" }

    fn part1(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part1_impl(&mut *input()));
    }

    fn part2(&self, input: &dyn Fn() -> Box<dyn io::Read>) {
        println!("{:?}", self.part2_impl(&mut *input()));
    }
}

impl Day06 {
    fn count(&self, ts: &mut topological_sort::TopologicalSort<String>, gen: usize) -> usize {
        let v = (*ts).pop_all();
        gen * v.len() + if v.len() == 0 { 0 } else { self.count(ts, gen + 1) }
    }

    fn part1_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let mut ts = topological_sort::TopologicalSort::<String>::new();
        reader.lines().map(|r| r.unwrap()).for_each(|l| {
            let mut split = l.as_str().split(")");
            let k = split.next().unwrap();
            let v = split.next().unwrap();
            ts.add_dependency(k.to_owned(), v.to_owned());
        });
        Ok(self.count(&mut ts, 0))
    }

    fn distance(&self,
                m: &collections::HashMap<String, collections::HashSet<String>>,
                root: &str, s: &str)
        -> Option<usize> {
        if root == s { Some(0) }
        else if let Some(orbiters) = m.get(root) {
            let d = orbiters.iter().map(|o|
                if let Some(d) = self.distance(m, o, s) { d + 1 } else { 0 })
                .sum();
            if d == 0 { None } else { Some(d) }
        } else { None }
    }

    fn transfers(&self,
                 m: &collections::HashMap<String, collections::HashSet<String>>,
                 root: &str, a: &str, b: &str)
        -> Option<usize> {
        let a_dist = self.distance(m, root, a);
        let b_dist = self.distance(m, root, b);
        if let Some(a_dist) = a_dist {
            if let Some(b_dist) = b_dist {
                if let Some(orbiters) = m.get(root) {
                    if let Some(dist) = orbiters.iter().map(|o|
                        self.transfers(m, o, a, b))
                        .find(|e| e.is_some()) {
                        dist
                    } else { Some(a_dist + b_dist) }
                } else { Some(a_dist + b_dist) }
            } else { None }
        } else { None }
    }

    fn part2_impl(self: &Self, input: &mut dyn io::Read) -> BoxResult<usize> {
        let reader = io::BufReader::new(input);
        let mut ts = topological_sort::TopologicalSort::<String>::new();
        let mut map= collections::HashMap::new();
        reader.lines().map(|r| r.unwrap()).for_each(|l| {
            let mut split = l.as_str().split(")");
            let k = split.next().unwrap();
            let v = split.next().unwrap();
            ts.add_dependency(k.to_owned(), v.to_owned());
            let mut orbiters = map.get_mut(k);
            if orbiters == None {
                map.insert(k.to_owned(), collections::HashSet::new());
                orbiters = map.get_mut(k);
            }
            orbiters.unwrap().insert(v.to_owned());
        });
        for root in ts.peek_all() {
            if let Some(n) = self.transfers(&map, root, "YOU", "SAN") {
                return Ok(n - 2);
            }
        }
        bail!("no way")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test1(s: &str, v: usize) {
        assert_eq!(Day06 {}.part1_impl(&mut s.as_bytes()).unwrap(), v);
    }

    #[test]
    fn part1() {
        test1("COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L", 42);
    }

    fn test2(s: &str, v: usize) {
        assert_eq!(Day06 {}.part2_impl(&mut s.as_bytes()).unwrap(), v);
    }

    #[test]
    fn part2() {
        test2("COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN", 4);
    }
}