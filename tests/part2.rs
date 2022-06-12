// part 2
//
// Now we'll do the basically the same setup, but this time we'll do a more complex
// interval analysis instead of just constant folding.
//
// The prelude is mostly the same, except we are importing the provided
// `Interval` type from this library (`egg_tutorial_pldi_2022`).
// Feel free to go look at `interval.rs`. It's pretty standard interval arithmetic,
// Not doing anything super precise or anything.
//
// This is all based on this EGRAPHS paper:
// https://arxiv.org/abs/2203.09191
use egg::*;
use egg_tutorial_pldi_2022::*;

type Num = num::BigRational;

define_language! {
    enum Math {
        Num(Num),
        "+" = Add([Id; 2]),
        // TODO: Add subtraction
        "*" = Mul([Id; 2]),
        // TODO: Add division
        Var(Symbol),
    }
}

// Same as part1, the actual IntervalAnalysis type doesn't hold any data
#[derive(Default)]
struct IntervalAnalysis;

// Now, to do the actual interval analysis!
impl Analysis<Math> for IntervalAnalysis {
    // We don't need an option for the e-class data type, since
    // there is a reasonable default value (-inf, inf)
    type Data = Interval;

    // The make is very similar to the ConstantFold::make from part1.
    // We don't need to use `?`, since we can just use the default interval.
    fn make(egraph: &EGraph<Math, Self>, enode: &Math) -> Self::Data {
        // getter function similar to before
        let get = |id: &Id| &egraph[*id].data;
        match enode {
            Math::Num(n) => Interval::singleton(n.clone()),
            Math::Add([a, b]) => get(a) + get(b),
            // TODO: Implement the subtraction case
            Math::Mul([a, b]) => get(a) * get(b),
            // TODO: Implement the division case
            _ => Interval::default(),
        }
    }

    // The merge function is more complicated than the ConstantFold::merge.
    // We want to do interval intersection.
    // One way (commented out) is to just do the intersection and return a
    // conservative approximation of the DidMerge.
    // Instead, we use the combinators to manually do the intersection
    // in a way that returns precise merge information
    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        // // a conservative manual implementation
        // if to == &from {
        //     DidMerge(false, false)
        // } else {
        //     *to = to.intersect(&from);
        //     DidMerge(true, true)
        // }

        // a more precise implementation using combinators
        // note how bitwise or operator can be used to combine DidMerge's
        egg::merge_option(&mut to.lo, from.lo, egg::merge_max)
            | egg::merge_option(&mut to.hi, from.hi, egg::merge_min)
    }

    fn modify(egraph: &mut EGraph<Math, Self>, id: Id) {
        // If the interval only includes one number, we can do constant folding
        if let Some(constant) = egraph[id].data.get_constant().cloned() {
            let new_id = egraph.add(Math::Num(constant));
            egraph.union(id, new_id);
        }
    }
}

#[rustfmt::skip]
fn rules() -> Vec<Rewrite<Math, IntervalAnalysis>> {
    vec![
        rewrite!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rewrite!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
    
        // TODO: Uncomment these rules once you add subtraction
        // rewrite!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        // rewrite!("canon-sub"; "(+ ?a (* -1 ?b))" => "(- ?a ?b)"),
        // rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),

        // TODO: Add a rule that says that a - b is equivalent to -1 * (b - a)

        rewrite!("add2-mul"; "(+ ?a ?a)" => "(* 2 ?a)"),
        rewrite!("mul-add2"; "(* 2 ?a)"  => "(+ ?a ?a)"),
    
        rewrite!("zero-add"; "(+ ?a 0)" => "?a"),
        rewrite!("zero-mul"; "(* ?a 0)" => "0"),
        rewrite!("one-mul";  "(* ?a 1)" => "?a"),
        

        rewrite!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

        // These div rules are **not** unsound, even with the possibility of dividing by zero.
        // Note how the left and right sides are "equally" sound, i.e., you always divide by ?c
        // You get in trouble when you make things more or less "sound" by changing what you divide by
        // TODO: Uncomment these rules once you add division
        // rewrite!("add-to-frac"; "(+ ?a (/ ?b ?c))" => "(/ (+ (* ?a ?c) ?b) ?c)"),
        // rewrite!("frac-to-add"; "(/ (+ (* ?a ?c) ?b) ?c)" => "(+ ?a (/ ?b ?c))" ),
        // rewrite!("mul-div";     "(* ?a (/ ?b ?c))" => "(/ (* ?a ?b) ?c)"),
        // rewrite!("div-mul";     "(/ (* ?a ?b) ?c)" => "(* ?a (/ ?b ?c))"),
        // rewrite!("frac-lift";   "(/ ?a ?b)" => "(/ (- ?b (- ?b ?a)) ?b)"),

        // We can make these sound now by using the interval information!
        // The `if` syntax allows you to add a `Condition` to a rewrite

        // TODO: Uncomment this rule once you add division
        // rewrite!("cancel-div"; "(/ ?a ?a)" => "1" if is_non_zero("?a")),
        // TODO: Add a rule that 0 / a is 0 if a is not zero
    ]
}

// The function signature output here implements the `Condition` trait in egg.
// So by making a function that outputs such functions, we have a "factory" for easily making
// `Condition`s to be used in rules.
#[allow(dead_code)] // delete this once you use it above!
fn is_non_zero(var: &str) -> impl Fn(&mut EGraph<Math, IntervalAnalysis>, Id, &Subst) -> bool {
    let var: Var = var.parse().unwrap();
    move |egraph, _root, subst: &Subst| {
        let id = subst[var];
        !egraph[id].data.contains_zero()
    }
}

// TODO: Remove #[ignore] from the next line and make sure the test passes
egg::test_fn! { #[ignore] div_zero_doesnt_crash, rules(), "(* 1 (/ 0 0))" => "(/ 0 0)" }

// The same tests from part 1 should work fine!

egg::test_fn! { simple_constant_fold, rules(), "(* x (+ -1 2))" => "(* 1 x)", "x" }

egg::test_fn! { math_simplify_add, rules(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }
// TODO: Remove #[ignore] from the next line and make sure the test passes
egg::test_fn! { #[ignore] math_simplify_const, rules(), "(+ 1 (- a (* (- 2 1) a)))" => "1" }

egg::test_fn! {
    math_simplify_factor, rules(),
    "(* (+ x 3) (+ x 1))"
    =>
    "(+ (+ (* x x) (* 4 x)) 3)"
}

// Let's check to make sure we can actually prove the right math needed to
// reduce the interval from the example in the paper.
egg::test_fn! {
    // TODO: Remove #[ignore] from the next line and make sure the test passes
    #[ignore] check_prove_fraction_example,
    rules(),
    "(- 1
        (/ (* 2 y)
           (+ x y)))"
    =>
    // Note how you can use many "goal" targets here.
    // It can be useful for debugging your rules to prove something by hand and
    // record your steps here, so you can see where eqsat gets "stuck"
    "(/ (- (+ x y) (* 2 y))
        (+ x y))",
    "(/ (+ (+ x y) (* -2 y))
        (+ x y))",
    "(/ (+ x (+ y (* -2 y)))
        (+ x y))",
    "(/ (+ x (* y (+ -2 1)))
        (+ x y))",
    "(/ (- x y)
        (+ x y))",
    "(- (/ (* 2 x)
           (+ x y))
        1)"
}

// Now we are ready do actually do the thing!
#[test]
#[ignore] // TODO: Remove this line and make sure the test passes
fn optimize_expr_from_egraphs_paper() {
    let expr: RecExpr<Math> = "
    (- 1 
       (/ (* 2 y)
          (+ x y)))"
        .parse()
        .unwrap();
    let mut runner = Runner::<Math, IntervalAnalysis, ()>::default().with_expr(&expr);

    // Recall that Vars get the default interval, so we need to tell the e-graph
    // about intervals for x and y.
    let x = runner.egraph.lookup(Math::Var("x".into())).unwrap();
    let y = runner.egraph.lookup(Math::Var("y".into())).unwrap();
    // ival is a helper function to easily parse an interval
    runner.egraph.set_analysis_data(x, ival("0, 1"));
    runner.egraph.set_analysis_data(y, ival("1, 2"));

    let root = runner.roots[0];

    // The interval hasn't been updated yet, because we haven't called
    // rebuild to propagate the analysis
    assert_eq!(runner.egraph[root].data, Interval::default());

    // Now it's updated!
    runner.egraph.rebuild();
    assert_eq!(runner.egraph[root].data, ival("-3, 1/3"));

    // Once we run the rules, we can see that it's shrunk to the intersection of
    // all provably equivalent terms!
    runner = runner.run(&rules());
    assert_eq!(runner.egraph[root].data, ival("-1, 0"));

    // We can just extract the best term for fun!
    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, best_expr) = extractor.find_best(runner.roots[0]);
    println!("{}", best_expr);
}

// Let's now package up that functionality into a function,
// and you can use this to write your own tests later.
fn optimize_interval(s: &str, intervals: &[(&str, &str)]) -> Interval {
    let expr: RecExpr<Math> = s.parse().unwrap();
    let mut runner = Runner::default().with_expr(&expr);
    let root = runner.roots[0];

    for (e, interval) in intervals {
        let e: RecExpr<Math> = e.parse().unwrap();
        let id = runner.egraph.add_expr(&e);
        let interval: Interval = ival(interval);
        runner.egraph.set_analysis_data(id, interval)
    }

    // we have to call rebuild to propagate the updated
    // analysis information throughout the e-graph
    runner.egraph.rebuild();
    println!("Initial interval: {}", runner.egraph[root].data);

    runner = runner.run(&rules());

    let final_interval = runner.egraph[root].data.clone();
    println!("Final interval:   {}", final_interval);
    final_interval
}

// The same test above, using our new function
#[test]
#[ignore] // TODO: Remove this line and make sure the test passes.
fn test_paper_example() {
    assert_eq!(
        optimize_interval(
            "(- 1 
                (/ (* 2 y)
                   (+ x y)))",
            // you pass in the initial intervals as a slice (ie. &[]) of tuples of strings
            &[("x", "0, 1"), ("y", "1, 2"),]
        ),
        ival("-1, 0")
    );
}

#[test]
#[ignore] // TODO: Remove this line and make sure the test passes.
fn test_other_paper_example() {
    let intervals = &[("x", "1, 2"), ("y", "1, 2")];
    assert_eq!(
        optimize_interval("(/ x (+ x y))", intervals),
        ival("1/4, 3/4")
    );
}

#[test]
fn squares() {
    // Can we do better than naive multiplication?
    // By adding a rule and a "square" operator,
    // you should be refine this interval to [0, 4]
    assert_eq!(
        optimize_interval("(* x x)", &[("x", "-2, 2")]),
        ival("-4, 4"),
    )
}
