# adventofcode2019
These are my, [Niklas Hallqvist](https://github.com/niklasha) solutions to
[Advent of code 2019](https://adventofcode.com/2019).
They are written in [Rust](https://rust-lang.org).

My reason for doing these are, besides the fact that I like puzzle solving, I want to learn Rust.

You need Rust, [rustup](https://rustup.rs/) is the suggested way to install Rust, that is about it.  You may need to add some SSL libraries, depending on operating system, but the installation process will tell you, if so.

Run all the days with:
```
cargo run input/
```

Where "input/" is a prefix for the days' inputs, named 01, 02, etc.
The tests (the examples given in the days' descriptions) can be run with:
```
cargo test
```

My results were:
```
      -------Part 1--------   -------Part 2--------
Day       Time  Rank  Score       Time  Rank  Score
  9   01:54:01  2223      0   01:56:04  2196      0
  8   01:40:39  3434      0   02:23:06  3419      0
  7   01:16:23  2386      0       >24h  9385      0
  6   01:43:46  3719      0   04:19:55  5413      0
  5   00:58:57  1755      0   01:06:58  1244      0
  4   00:27:54  3193      0   01:06:08  3550      0
  3   00:56:54  1997      0   01:03:27  1507      0
  2   00:42:23  3020      0   00:54:01  2538      0
  1   00:06:31  1090      0   00:19:58  1246      0
```
