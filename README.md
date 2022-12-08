
aoc-bud: `Module that aims to help during advent of code puzzles.`
==================================================================

**For now it only requests the puzzle input, saves as a file, to avoid spamming the server with unnecessary calls, and finally returns it as a string.**

Installation
------------

```sh
cargo add aoc-bud
```
or add the following line to your `Cargo.toml`
```toml
[dependencies]
aoc-bud = "0.0.1"
```

Usage
-----

You **must** have a .env file on your project directory with your advent of code session cookie

```sh
echo AOC_SESSION={yoursessionhere} > .env
```

```rust
use aoc_bud;

fn main() {
    // Get current running puzzle input
    let input: String = aoc_bud::new();
    
    // Get puzzle input from custom date
    // In this example from the 3rd of 2022
    let input_custom: String = aoc_bud::new_custom(3, 2022); 
}
```
