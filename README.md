# atomic-dbg

<p>
  <a href="https://crates.io/crates/atomic-dbg"><img src="https://img.shields.io/crates/v/atomic-dbg.svg" alt="crates.io page" /></a>
  <a href="https://docs.rs/atomic-dbg"><img src="https://docs.rs/atomic-dbg/badge.svg" alt="docs.rs docs" /></a>
</p>

This crate provides [`dbg`], [`eprint`], and [`eprintln`], macros which work
just like their [counterparts] [in] [std], but which:

 - Write atomically, up to the greatest length supported on the platform.
 - Don't use locks (in userspace) or dynamic allocations.
 - Preserve libc's `errno` and Windows' last-error code value.

This means they can be used just about anywhere within a program, including
inside allocator implementations, inside synchronization primitives, startup
code, around FFI calls, inside signal handlers, and in the child process of a
`fork` before an `exec`.

And, when multiple threads are printing, as long as they're within the length
supported on the platform, the output is readable instead of potentially
interleaved with other output.

For example, this code:
```rust
use atomic_dbg::dbg;

fn main() {
    dbg!(2, 3, 4);
}
```

Has this strace output:
```notrust
write(2, "[examples/dbg.rs:4] 2 = 2\n[examples/dbg.rs:4] 3 = 3\n[examples/dbg.rs:4] 4 = 4\n", 78[examples/dbg.rs:4] 2 = 2
```
which is a single atomic `write` call.

For comparison, with `std::dbg` it looks like this:
```notrust
write(2, "[", 1[)                        = 1
write(2, "examples/dbg.rs", 15examples/dbg.rs)         = 15
write(2, ":", 1:)                        = 1
write(2, "4", 14)                        = 1
write(2, "] ", 2] )                       = 2
write(2, "2", 12)                        = 1
write(2, " = ", 3 = )                      = 3
write(2, "2", 12)                        = 1
write(2, "\n", 1
)                       = 1
write(2, "[", 1[)                        = 1
write(2, "examples/dbg.rs", 15examples/dbg.rs)         = 15
write(2, ":", 1:)                        = 1
write(2, "4", 14)                        = 1
write(2, "] ", 2] )                       = 2
write(2, "3", 13)                        = 1
write(2, " = ", 3 = )                      = 3
write(2, "3", 13)                        = 1
write(2, "\n", 1
)                       = 1
write(2, "[", 1[)                        = 1
write(2, "examples/dbg.rs", 15examples/dbg.rs)         = 15
write(2, ":", 1:)                        = 1
write(2, "4", 14)                        = 1
write(2, "] ", 2] )                       = 2
write(2, "4", 14)                        = 1
write(2, " = ", 3 = )                      = 3
write(2, "4", 14)                        = 1
write(2, "\n", 1
)                       = 1
```

atomic-dbg is `no_std`, however like `std`, it uses the stderr file descriptor
ambiently, assuming that it's open.

## Logging

With the "log" feature enabled, atomic-dbg defines an `atomic_dbg::log::init`
which installed a minimal logging implementation using the `eprintln` macro.

[counterparts]: https://doc.rust-lang.org/stable/std/macro.dbg.html
[in]: https://doc.rust-lang.org/stable/std/macro.eprintln.html
[std]: https://doc.rust-lang.org/stable/std/macro.eprint.html
[`dbg`]: https://docs.rs/atomic-dbg/latest/atomic-dbg/macro.dbg.html
[`eprintln`]: https://docs.rs/atomic-dbg/latest/atomic-dbg/macro.eprintln.html
[`eprint`]: https://docs.rs/atomic-dbg/latest/atomic-dbg/macro.eprint.html
