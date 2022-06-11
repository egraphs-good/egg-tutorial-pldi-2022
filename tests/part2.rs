use egg::*;
use egg_tutorial_pldi_2022::*;

type Num = num::BigRational;

define_language! {
    enum Math {
        Num(Num),
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        Var(Symbol),
    }
}

// Interval analysis
// https://arxiv.org/abs/2203.09191
#[derive(Default)]
struct IntervalAnalysis;
impl Analysis<Math> for IntervalAnalysis {
    type Data = Interval;

    fn make(egraph: &EGraph<Math, Self>, enode: &Math) -> Self::Data {
        let get = |id: &Id| &egraph[*id].data;
        match enode {
            Math::Num(n) => Interval::singleton(n.clone()),
            Math::Add([a, b]) => get(a) + get(b),
            Math::Sub([a, b]) => get(a) - get(b),
            Math::Mul([a, b]) => get(a) * get(b),
            Math::Div([a, b]) => get(a) / get(b),
            _ => Interval::default(),
        }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        // // a conservative manual implementation
        // if to == &from {
        //     DidMerge(false, false)
        // } else {
        //     *to = to.intersect(&from);
        //     DidMerge(true, true)
        // }

        // a more precise implementation using combinators
        egg::merge_option(&mut to.lo, from.lo, egg::merge_max)
            | egg::merge_option(&mut to.hi, from.hi, egg::merge_min)
    }

    fn modify(egraph: &mut EGraph<Math, Self>, id: Id) {
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
    
        rewrite!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        rewrite!("canon-sub"; "(+ ?a (* -1 ?b))" => "(- ?a ?b)"),

        rewrite!("add2-mul"; "(+ ?a ?a)" => "(* 2 ?a)"),
        rewrite!("mul-add2"; "(* 2 ?a)"  => "(+ ?a ?a)"),

        rewrite!("frac-special"; "(/ ?a ?b)"  => "(/ (- ?b (- ?b ?a)) ?b)"),

        // rewrite!("flip-sub"; "(- ?a ?b)" => "(- (* -1 ?b) (* -1 ?a))"),
        rewrite!("flip-sub"; "(- ?a ?b)" => "(* -1 (- ?b ?a))"),
    
        rewrite!("zero-add"; "(+ ?a 0)" => "?a"),
        rewrite!("zero-mul"; "(* ?a 0)" => "0"),
        rewrite!("one-mul";  "(* ?a 1)" => "?a"),
    
        // rewrite!("add-zero"; "?a" => "(+ ?a 0)"),
        // rewrite!("mul-one";  "?a" => "(* ?a 1)"),
    
        rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),

        rewrite!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),
        rewrite!("factor-one"; "(+ ?a (* ?a ?c))"        => "(* ?a (+ ?c 1))"),

        rewrite!("add-to-frac"; "(+ ?a (/ ?b ?c))" => "(/ (+ (* ?a ?c) ?b) ?c)"),
        rewrite!("frac-to-add"; "(/ (+ (* ?a ?c) ?b) ?c)" => "(+ ?a (/ ?b ?c))" ),
        rewrite!("mul-div";     "(* ?a (/ ?b ?c))" => "(/ (* ?a ?b) ?c)"),
        rewrite!("div-mul";     "(/ (* ?a ?b) ?c)" => "(* ?a (/ ?b ?c))"),

        // Uh-oh! This pair of rules is unsound!!!
        // rewrite!("cancel-div"; "(/ ?a ?a)" => "1"),
        // rewrite!("zero-div";   "(/ 0 ?a)" => "0"),
    ]
}

// The same tests from part 1 should work fine!

egg::test_fn! { simple_constant_fold, rules(), "(* x (+ -1 2))" => "(* 1 x)", "x" }

egg::test_fn! {math_simplify_add, rules(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }
egg::test_fn! { math_simplify_const, rules(), "(+ 1 (- a (* (- 2 1) a)))" => "1" }

egg::test_fn! {
    math_simplify_factor, rules(),
    "(* (+ x 3) (+ x 1))"
    =>
    "(+ (+ (* x x) (* 4 x)) 3)"
}

egg::test_fn! {
    fraction_stuff,
    rules(),
    "(- 1
        (/ (* 2 y)
           (+ x y)))"
    =>
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

#[test]
fn test() {
    let expr: RecExpr<Math> = "
    (- 1 
       (/ (* 2 y)
          (+ x y)))"
        .parse()
        .unwrap();
    let mut runner = Runner::<Math, IntervalAnalysis, ()>::default().with_expr(&expr);

    let x = runner.egraph.lookup(Math::Var("x".into())).unwrap();
    let y = runner.egraph.lookup(Math::Var("y".into())).unwrap();
    runner.egraph.set_analysis_data(x, ival("0, 1"));
    runner.egraph.set_analysis_data(y, ival("1, 2"));

    let root = runner.roots[0];

    assert_eq!(runner.egraph[root].data, Interval::default());

    runner.egraph.rebuild();
    assert_eq!(runner.egraph[root].data, ival("-3, 1/3"));

    runner = runner
        .with_scheduler(SimpleScheduler)
        .with_iter_limit(4)
        .run(&rules());

    assert_eq!(runner.egraph[root].data, ival("-3, 1/3"));

    let extractor = Extractor::new(&runner.egraph, AstSize);
    let (_best_cost, best_expr) = extractor.find_best(runner.roots[0]);
    println!("{}", best_expr);
}
