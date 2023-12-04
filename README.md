
aoc-bud: `Module that aims to help during advent of code puzzles.`
==================================================================

- v0.0.1: + requests puzzle input, saves as a file, to avoid spamming the server with unnecessary calls, and finally returns it as a string
- v0.0.2: + able to post solution to advent of code and receive server message
- v0.0.3: refactored code to use less dependencies and changed how the library is used
- v0.0.4: removed a annoying print (nothing new)

Installation
------------

```sh
cargo add aoc-bud
```
or add the following line to your `Cargo.toml`
```toml
[dependencies]
aoc-bud = "0.0.4"
```

Usage
-----

You **must** have a .env file on your project directory with your advent of code session cookie

```sh
echo AOC_SESSION={yoursessionhere} > .env
```

The library exports Regex aswell.

```rust
use aoc_bud::Aoc;

// If you want to use Regex you can just import it
use aoc_bud::Regex;

fn main() {
    // Create a Aoc instance for the date you choose
    let aoc = Aoc::new(1, 2023);

    // Get puzzle input
    let input: String = aoc.input(); 
    
    // ...
    // Solution code part 1
    // ... 

    // Send your solution for the first part
    aoc.solve1(solution).unwrap();
     
    // ...
    // Solution code part 2
    // ...

    // Send your solution for the first part
    aoc.solve2(solution).unwrap();
}
```

Features
--------

By enabling the time feature, instead of setting the date yourself the program will get the current date.
**NOTE** This only works when advent of code is ongoing, otherwise you will only get errors.

```rust
use aoc_bud::Aoc;

fn main() {
    // Create a Aoc instance for the date today
    let aoc = Aoc::today();

    // Get puzzle input
    let input: String = aoc.input(); 
    
    // ...
    // Solution code part 1
    // ... 

    // Send your solution for the first part
    aoc.solve1(solution).unwrap();
     
    // ...
    // Solution code part 2
    // ...

    // Send your solution for the first part
    aoc.solve2(solution).unwrap();
}
```
