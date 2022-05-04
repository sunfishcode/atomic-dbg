//! Threads demo.
//!
//! This program spawns two threads. One does a `dbg` that prints
//! even numbers, the other does a `dbg` that prints odd numbers.
//!
//! Comment out the `use atomic_dbg::dbg;` line to use `std::dbg`. Run
//! it that way enough times, and you'll see the output of the two
//! threads interleave.
//!
//! std uses a `StderrLock`, so the interleaving only happens at line
//! boundaries. But if there's any non-Rust code in the process
//! printing to stderr from other threads, it could be interleaved
//! in the middle of a line.
//!
//! Uncomment that line to re-enable atomic-dbg and
//! the output will not be interleaved. Each thread does a single
//! atomic `write` call.

use atomic_dbg::dbg;
use std::thread;

fn main() {
    let evens = thread::Builder::new()
        .name("evens".to_string())
        .spawn(move || {
            dbg!(0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24);
        })
        .unwrap();
    let odds = thread::Builder::new()
        .name("odds".to_string())
        .spawn(move || {
            dbg!(1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25);
        })
        .unwrap();
    evens.join().unwrap();
    odds.join().unwrap();
}
