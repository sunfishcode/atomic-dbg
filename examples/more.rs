use atomic_dbg::{eprint, eprintln};

fn main() {
    let a = 2;
    let b = 3.5;
    let c = "purple";

    // Fancy new-style format strings!
    eprintln!("{a}, {b}, {c}");

    // The old way still works!
    eprintln!("{}, {}, {}", a, b, c);

    // Debug formatting works too!
    eprintln!("{:?}, {:?}, {:?}", a, b, c);

    // Non-atomic hello world, because we can!
    eprint!("Hello, ");
    eprint!("World!");
    eprintln!();
}
