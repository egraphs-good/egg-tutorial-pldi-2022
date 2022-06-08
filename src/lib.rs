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
pub mod interval;

pub fn make_repl(mut f: impl FnMut(&str)) {
    use rustyline::error::ReadlineError;
    use rustyline::validate::{
        MatchingBracketValidator, ValidationContext, ValidationResult, Validator,
    };
    use rustyline::{Editor, Result};
    use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

    #[derive(Completer, Helper, Highlighter, Hinter)]
    struct MyHelper(MatchingBracketValidator);

    // make my own validator to check for matched parens
    impl Validator for MyHelper {
        fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
            self.0.validate(ctx)
        }
    }

    let h = MyHelper(MatchingBracketValidator::new());
    let mut rl = Editor::new();
    rl.set_helper(Some(h));

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                use std::panic::{catch_unwind, AssertUnwindSafe};
                if catch_unwind(AssertUnwindSafe(|| f(&line))).is_err() {
                    println!("Caught a panic!");
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
