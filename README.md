# egg tutorial for PLDI 2022

Welcome to the tutorial for [`egg`] at [PLDI 2022](https://pldi22.sigplan.org/)!

[Link to the tutorial slides.](https://docs.google.com/presentation/d/1mIXL944Cc2QotNiDrtDmtTsGXgO0GUlbQnPS8oQvGYk/edit?usp=sharing)

[`egg`]: https://egraphs-good.github.io/

## Getting started

This tutorial is designed for you to follow along, 
 either live or at your own pace.
It is structured as a Rust library, 
 and you will edit the tests in the `tests` directory.

You will need the following tools to run this tutorial:
- [Rust](https://www.rust-lang.org/tools/install), at least version 1.60.
    - You can install Rust using the `rustup` tool, [see here](https://www.rust-lang.org/tools/install).
    - If you already have Rust, please run `rustup update`.
- [VSCode](https://code.visualstudio.com/)
    - You can use your own editor if you want, 
      but VSCode will make it very easy to run the tests the make up the tutorial.
- [rust-analyzer](https://rust-analyzer.github.io/)
    - If using VSCode, install it from the Extensions menu.
    - Many other editors also support rust-analyzer.

Once you have the tools, clone this repo and run the tests 
 using `cargo` (Rust's build tool) to make sure you're up and running.
```
git clone https://github.com/egraphs-good/egg-tutorial-pldi-2022.git
cd egg-tutorial-pldi-2022

cargo test
```

All of the completed exercises are in the `tests/` directory as well,
 so if you get stuck, just copy-paste!

If you got this far, you're ready for the tutorial to begin!

## Other Resources

This is an `egg` tutorial (and a rather short one!), 
 so here are some other resources you may want to consult.

- `egg` [API documentation](https://docs.rs/egg/latest/egg/)
- `egg` online [tutorials](https://docs.rs/egg/latest/egg/tutorials/_02_getting_started/index.html) as part of the documentation
    - Note that `part0` of the in-person tutorial is taken straight from the
      "Getting Started" tutorial online.

This is **not** a Rust tutorial, 
 so if you aren't familiar with Rust you may just have to settle for
 copy-pasting, tweaking things, and being puzzled over the various `&`s and `?`s.
Thankfully, there are a lot of good Rust resources out there! Here are some you might find useful:
- Rust [standard library documentation](https://doc.rust-lang.org/stable/std/)
- [The Rust Book](https://doc.rust-lang.org/book/)

## Tutorial structure

The tutorial is structured into three parts, each as a file under the `tests/` directory: 

- `part0.rs`: a warm up that shows various `egg` APIs
    - This is heavily based on one of the online `egg` [tutorials](https://docs.rs/egg/latest/egg/tutorials/_02_getting_started/index.html).
- `part1.rs`: a simple optimizer for rational arithmetic
- `part2.rs`: a more powerful usage of e-class analyses using intervals
    - This idea is based on a paper ["Abstract Interpretation on E-Graphs"](https://arxiv.org/abs/2203.09191)
      presented at the [EGRAPHS](https://pldi22.sigplan.org/home/egraphs-2022) workshop.


