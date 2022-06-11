//! This `lib.rs` file is the root of the library.
//!
//! There's not much in here except for declarations of
//! the other projects in the file

// - 9:00 - 10:00
//     - 9:00 - 9:15
//         - Welcome (folks will roll in late!)
//         - Overview, install, …, demo
//     - 9:15 - 9:30
//         - e-graphs from 0
//     - 9:30 - 9:45
//         - rat arith lang ( / bv if you are ahead)
//         - rewrites
//     - 9:45 - 10:00
//         - check eq
//         - optimize
//         - trouble in paradise
//         - no constant folding
//         - no way to have sound div rules
// - 10:00 - 10:30
//     - Break
// - 10:30 - 12:00
//     - 10:30 - 11:00
//         - constant folding
//     - 11:00 - 11:30
//         - interval analysis
//     - 11:30 - 11:50
//         - putting it all together
//     - 11:50 - 12:00
//         - where to go from here
//         - promo EGRAPHS and PLDI talks
//         - THANKS!

// The provided, simple interval library
mod interval;

// re-export all the public items
pub use interval::*;
