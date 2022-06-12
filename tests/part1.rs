//! part 2
//!
//! This time we will try making our own language for rational arithmetic.
//! This will allow us to actually interpret some of the data in the e-nodes.
//! In particular, we can implement our first e-class analysis that will
//! perform constant folding!
use egg::*;
use std::time::Duration;

use num::*;
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

#[rustfmt::skip]
fn rules() -> Vec<Rewrite<Math, ConstantFold>> {
    vec![
        rewrite!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rewrite!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
    
        rewrite!("sub-canon"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
    
        rewrite!("zero-add"; "(+ ?a 0)" => "?a"),
        rewrite!("zero-mul"; "(* ?a 0)" => "0"),
        rewrite!("one-mul";  "(* ?a 1)" => "?a"),
    
        rewrite!("add-zero"; "?a" => "(+ ?a 0)"),
        rewrite!("mul-one";  "?a" => "(* ?a 1)"),
    
        rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),

        rewrite!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

        // Uh-oh! This pair of rules is unsound!!!
        rewrite!("cancel-div"; "(/ ?a ?a)" => "1"),
        rewrite!("zero-div";   "(/ 0 ?a)" => "0"),
    ]
}

#[derive(Default)]
struct ConstantFold;

impl Analysis<Math> for ConstantFold {
    type Data = Option<Num>;

    fn make(egraph: &EGraph<Math, Self>, enode: &Math) -> Self::Data {
        let get = |id: &Id| egraph[*id].data.as_ref();
        match enode {
            Math::Num(n) => Some(n.clone()),
            Math::Add([a, b]) => Some(get(a)? + get(b)?),
            Math::Sub([a, b]) => Some(get(a)? - get(b)?),
            Math::Mul([a, b]) => Some(get(a)? * get(b)?),
            Math::Div([a, b]) => {
                let b = get(b)?;
                if !b.is_zero() {
                    Some(get(a)? / b)
                } else {
                    None
                }
            }
            Math::Var(_) => None,
        }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_option(to, from, |a, b| {
            assert_eq!(a, &b, "bad merge!");
            DidMerge(false, false)
        })
    }

    fn modify(egraph: &mut EGraph<Math, Self>, id: Id) {
        if let Some(n) = egraph[id].data.clone() {
            let id2 = egraph.add(Math::Num(n));
            egraph.union(id, id2);
        }
    }
}

egg::test_fn! { simple_constant_fold, rules(), "(* x (+ -1 2))" => "(* 1 x)", "x" }

egg::test_fn! {math_simplify_add, rules(), "(+ x (+ x (+ x x)))" => "(* 4 x)" }
egg::test_fn! { math_simplify_const, rules(), "(+ 1 (- a (* (- 2 1) a)))" => "1" }

egg::test_fn! {
    math_simplify_factor, rules(),
    "(* (+ x 3) (+ x 1))"
    =>
    "(+ (+ (* x x) (* 4 x)) 3)"
}

fn optimize(s: &str) -> String {
    let expr: RecExpr<Math> = s.parse().unwrap();
    let runner = Runner::default()
        .with_expr(&expr)
        // place some limits just to make sure the demo is fast!
        .with_time_limit(Duration::from_secs_f64(0.5))
        .with_iter_limit(10)
        // we don't need to use a hook for optimization,
        // but the test_fn! macro uses it for equality checking
        // .with_hook(|runner| ...)
        .run(&rules());
    let extractor = Extractor::new(&runner.egraph, AstSize);

    // We want to extract the best expression represented in the
    // same e-class as our initial expression, not from the whole e-graph.
    // Luckily the runner stores the e-class Id where we put the initial expression.
    let (_best_cost, best_expr) = extractor.find_best(runner.roots[0]);
    best_expr.to_string()
}

#[test]
fn test_optimize() {
    assert_eq!(optimize("(+ 1 1)"), "2");

    // this can be a little dicey,
    // there might be many equally "optimized" results
    // assert_eq!(optimize("(+ x y)"), "(+ x y)");
}

egg::test_fn! {
  // if you uncomment the div rules above,
  // try un-ignoring this test and see what happens!
  #[should_panic(expected = "bad merge!")]
  simple_division, rules(), "(/ 0 0)" => "1", "0"
}
