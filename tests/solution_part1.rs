// part 2
//
// This time we will try making our own language for rational arithmetic.
// This will allow us to actually interpret some of the data in the e-nodes.
// In particular, we can implement our first e-class analysis that will
// perform constant folding!

// Again, we'll just import everything from egg
use egg::*;

// We will use the "big rational" type from the popular `num` library
// to power our rational arithmetic.
use num::*;

// This is just a type alias for the BigRational type.
type Num = num::BigRational;

// Now we define the e-node type.
// E-nodes implement the `egg::Language` trait, and they are typically some
// pairing of a operator and a list of children (as `Id`s).
//
// The `define_language` macro makes it easy for you to make your own
// `Language`s from a Rust enum. This implements some nice parsing for you as well.
define_language! {
    enum Math {
        // This parses rationals as enums, like "4"
        Num(Num),
        // This will parse a "+" symbol as an Add with two children
        // Note that the value of this variant is a [Id; 2], or an array of 2 Id's.
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        // Finally, this will parse anything else as a Var, like "foo"
        Var(Symbol),
    }
}

#[test]
fn insert_some_math_into_an_egraph() {
    // let's try just using the language we just made
    // we'll make an e-graph with just the unit () analysis for now
    let mut egraph = EGraph::<Math, ()>::default();

    // we can manually construct Math's and insert them with add
    let zero = egraph.add(Math::Num(Num::zero()));
    let one = egraph.add(Math::Num(Num::one()));
    let zero_plus_one = egraph.add(Math::Add([zero, one]));

    // we can also parse things into RecExprs
    let expr: RecExpr<Math> = "(+ 0 1)".parse().unwrap();
    let zero_plus_one2 = egraph.add_expr(&expr);

    // hash consing makes sure these are the same
    assert_eq!(zero_plus_one, zero_plus_one2);
}

// Okay, now let's make some rewrites!
// We will make the rewrites for the ConstantFold analysis for now.
// You can make rewrites generic over the analysis if you want,
// but we'll just be concrete for now

#[rustfmt::skip]
fn rules() -> Vec<Rewrite<Math, ConstantFold>> {
    vec![
        rewrite!("comm-add";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rewrite!("comm-mul";  "(* ?a ?b)"        => "(* ?b ?a)"),
        rewrite!("assoc-add"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rewrite!("assoc-mul"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
    
        rewrite!("canon-sub"; "(- ?a ?b)" => "(+ ?a (* -1 ?b))"),
        rewrite!("cancel-sub"; "(- ?a ?a)" => "0"),

        rewrite!("zero-add"; "(+ ?a 0)" => "?a"),
        rewrite!("zero-mul"; "(* ?a 0)" => "0"),
        rewrite!("one-mul";  "(* ?a 1)" => "?a"),
    
        rewrite!("add-zero"; "?a" => "(+ ?a 0)"),
        rewrite!("mul-one";  "?a" => "(* ?a 1)"),
    
        rewrite!("distribute"; "(* ?a (+ ?b ?c))"        => "(+ (* ?a ?b) (* ?a ?c))"),
        rewrite!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

        rewrite!("cancel-div"; "(/ ?a ?a)" => "1"),
        rewrite!("zero-div"; "(/ 0 ?a)" => "0"),
        // 🤔 Do you notice any potential problems with this pair of rules?    
    ]
}

// Now let's make the constant fold analysis.
// The analysis itself (the ConstantFold struct) is a zero-sized type with
// no data in it, it's just a marker.
//
// Make sure to derive Default, since we want egg to be able to automatically
// construct a (the only) value of this type.
#[derive(Default)]
struct ConstantFold;

// Here's the actual implementation.
// Analysis is the egg trait for e-class analyses. It is parameterized over the
// Language that the analysis is for.
impl Analysis<Math> for ConstantFold {
    // This associated type tells you what type is attached to
    // each e-class. We'll use an optional number to indicate the
    // constant (maybe) associated with each e-class.
    type Data = Option<Num>;

    // This function tells egg how to construct a `Data` for a particular e-node.
    // It's typically where most of your logic in an e-class analysis goes.
    fn make(egraph: &EGraph<Math, Self>, enode: &Math) -> Self::Data {
        // first, we make a getter function that grabs the data for a given e-class id
        let get = |id: &Id| egraph[*id].data.as_ref();

        // now, we write the evaluator. Since the `Data` type is an `Option`, we
        // can use the `?` operator in Rust, which trys to unpack the
        // preceding optional value, "bailing" from the enclosing function if it's None
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

    // This function tells egg how to merge analysis data when
    // e-classes get unioned. `to` is a mutable reference, which should
    // be mutated as needed to be the Analysis data ofthe merged eclass.
    // The return value is a `DidMerge`, which tells
    // egg which "way" the merge went. It can be computed from the partial
    // ordering over the `Data`.
    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        match (to.as_mut(), from) {
            // Neither side is known to be a constant so there's nothing
            // to do when they merge.
            (None, None) => DidMerge(false, false),

            // Both sides are constants, so we should just make sure
            // they're the same.
            (Some(a), Some(b)) => {
                assert_eq!(a, &b, "bad merge!");
                DidMerge(false, false)
            }

            // The right side is a constant, so update `to` to be the same.
            (None, Some(x)) => {
                *to = Some(x);
                DidMerge(true, false)
            }

            // The left side is a constant and the right is not, so there's
            // nothing to do when they merge.
            (Some(_), None) => DidMerge(false, false),
        }
        // Since Analysis data are so often Options, egg provides a
        // combinator to make it easy to merge options:
        // egg::merge_option(to, from, |a, b| {
        //     assert_eq!(a, &b, "bad merge!");
        //     DidMerge(false, false)
        // })
    }

    // This is an optional function that allows you to modify the e-graph itself
    // in response to changing e-class analysis values (or anything else).
    // Here, we use it to actually insert new e-nodes corresponding to computed
    // constant values.
    fn modify(egraph: &mut EGraph<Math, Self>, id: Id) {
        if let Some(n) = egraph[id].data.clone() {
            let id2 = egraph.add(Math::Num(n));
            egraph.union(id, id2);
        }
    }
}

// Now, we can write some tests the prove the equivalence of terms.
// We use the provided `test_fn` macro from egg to quickly generate such tests.

egg::test_fn! {
   simple_constant_fold, // the name of the test
   rules(),              // the rules to be used
   "(* x (+ -1 2))"      // the starting expression
   =>
   "(* 1 x)", "x"        // one or more ending exprs to prove the starting expr equivalent to
}

egg::test_fn! {
   math_simplify_add, rules(),
   // here we use the (optional) "runner =" syntax to change the Runner used for this test
   // check out what the output looks like with explanations enabled!
   runner = Runner::default().with_explanations_enabled(),
   "(+ x (+ x (+ x x)))" => "(* 4 x)"
}

egg::test_fn! { math_simplify_const, rules(), "(+ 1 (- a (* (- 2 1) a)))" => "1" }

egg::test_fn! {
    math_simplify_factor, rules(),
    "(* (+ x 3) (+ x 1))"
    =>
    "(+ (+ (* x x) (* 4 x)) 3)"
}

// This test crashes, but the `should_panic` expects that.
egg::test_fn! {
  #[should_panic(expected = "bad merge!")]
  simple_division, rules(), "(/ 0 0)" => "1", "0"
}
// The problem here is the two division rules you added earlier:
// If x/x = 1 *and* 0/x = 0, then when x = 0, we have 0/0 = 1 *and* 0/0 = 0
// We'll see more about how to deal with this in the next part of the tutorial.

// Try writing your own equivalence tests!

// Now we can try writing a optimizer!
// For the sake of this demo, this function will be string -> string,
// but it semantically takes and returns RecExpr<Math>.
fn optimize(s: &str) -> String {
    // parse the given expression
    let expr: RecExpr<Math> = s.parse().unwrap();

    // Now we create a Runner to actually do the equality saturation.
    // Runner have a builder-style set of methods to customize them.
    let runner = Runner::default()
        .with_expr(&expr)
        // place some limits just to make sure the demo is fast!
        .with_time_limit(std::time::Duration::from_secs_f64(0.5))
        .with_iter_limit(10)
        // we don't need to use a hook for optimization,
        // but the test_fn! macro uses it for equality checking
        // .with_hook(|runner| ...)
        //
        // Now we can actually give it the rules and run it.
        // This consumes the Runner, so you have to rebind it.
        .run(&rules());

    // Now we can extract using the built-in AstSize cost function.
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
