use egg::{rewrite, *};

#[test]
fn now_youre_speaking_my_language() {
    // RecExpr represents a recursive expression from a user-defined
    // Language as a list of enodes.
    // RecExprs must satisfy the invariant that enodes' children must
    // refer to elements that come before it in the list.
    // EGraphs are parameterized over Language, which is a trait. Values
    // that implement Language are enodes.
    // Since parsing can return an error,
    // `unwrap` just panics if the result doesn't return Ok
    let my_expression: RecExpr<SymbolLang> = "(foo a b)".parse().unwrap();
    println!("This is my expression {}", my_expression);

    // let's try to create an e-node, but hmmm,
    // what do I put as the children? It is intentionally hard to create
    // enodes with children in isolation, since the child Ids would be
    // inherently meaningless.
    let _my_enode = SymbolLang::new("bar", vec![]);

    // The way to make meaningful Ids is to add enodes to either an EGraph
    // or a RecExpr.
    let mut expr = RecExpr::default();
    let a = expr.add(SymbolLang::leaf("a"));
    let b = expr.add(SymbolLang::leaf("b"));
    let _foo = expr.add(SymbolLang::new("foo", vec![a, b]));

    // EGraphs are parameterized by
    //      - Language: The types of the values in the egraph come from the
    //                  user-defined Language. Here, we're using SymbolLang.
    //      - Analysis: Arbitrary data associated with each eclass that
    //                  is kept updates across eclass merges. Analyses are
    //                  enable things like constant folding. For now, we'll
    //                  just use (), which implements the Analysis trait
    //                  trivially.
    let mut egraph: EGraph<SymbolLang, ()> = Default::default();
    let a = egraph.add(SymbolLang::leaf("a"));
    let b = egraph.add(SymbolLang::leaf("b"));
    let foo = egraph.add(SymbolLang::new("foo", vec![a, b]));

    // we can also add RecExprs to an egraph
    let foo2 = egraph.add_expr(&expr);

    // note that if you add the same thing to an e-graph twice,
    // you'll get back equivalent Ids
    assert_eq!(foo, foo2);
}

#[test]
fn searching_an_egraph() {
    // let's make an egraph
    let mut egraph: EGraph<SymbolLang, ()> = Default::default();
    let a = egraph.add(SymbolLang::leaf("a"));
    let b = egraph.add(SymbolLang::leaf("b"));
    let _foo = egraph.add(SymbolLang::new("foo", vec![a, b]));

    // rebuild the e-graph since we modified it
    egraph.rebuild();

    // A Pattern is essentially a for-all quantified expression that can be
    // used to search for or apply reweires.
    // We can make Patterns by parsing, similar to RecExprs names preceded
    // by ? are parsed as Pattern variables and will match anything
    let pat: Pattern<SymbolLang> = "(foo ?x ?x)".parse().unwrap();

    // since we use ?x twice, it must match the same thing,
    // so this search will return nothing
    let matches = pat.search(&egraph);
    assert!(matches.is_empty());

    egraph.union(a, b);
    // recall that rebuild must be called to "see" the effects
    // of adds or unions
    egraph.rebuild();

    // now we can find a match since a = b
    let matches = pat.search(&egraph);
    assert!(!matches.is_empty());
}

#[test]
fn using_runner() {
    // rewrite is a macro that simplifies creating simple, purely syntactic
    // rewrites.
    // rewrite!(a; b => c) creates a Rewrite with name `a`, Searcher `b`,
    // and Applier `c`.
    let rules: &[Rewrite<SymbolLang, ()>] = &[
        rewrite!("commute-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rewrite!("commute-mul"; "(* ?x ?y)" => "(* ?y ?x)"),
        rewrite!("add-0"; "(+ ?x 0)" => "?x"),
        rewrite!("mul-0"; "(* ?x 0)" => "0"),
        rewrite!("mul-1"; "(* ?x 1)" => "?x"),
    ];

    // While it may look like we are working with numbers,
    // SymbolLang stores everything as strings.
    // We can make our own Language later to work with other types.
    let start: RecExpr<SymbolLang> = "(+ 0 (* 1 a))".parse().unwrap();

    // Runner is egg's provided equality saturation engine that has
    // reasonable defaults and implements useful things like saturation
    // checking, egraph size limits, and rule scheduling.
    // All we have to do now is create a Runner, add our expression, and run!
    let runner = Runner::default().with_expr(&start).run(rules);

    // Finally, we can use the Extractor to get the best result.
    // Extractors can take a user-defined cost function,
    // we'll use the egg-provided AstSize for now
    let extractor = Extractor::new(&runner.egraph, AstSize);

    // We want to extract the best expression represented in the
    // same e-class as our initial expression, not from the whole e-graph.
    // Luckily the runner stores the eclass Id where we put the initial expression.
    let (best_cost, best_expr) = extractor.find_best(runner.roots[0]);

    // we found the best thing, which is just "a" in this case
    assert_eq!(best_expr, "a".parse().unwrap());
    assert_eq!(best_cost, 1);
}
