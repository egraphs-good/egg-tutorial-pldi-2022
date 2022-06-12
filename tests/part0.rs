use egg::{rewrite as rw, *};

#[test]
fn now_youre_speaking_my_language() {
    // Since parsing can return an error,
    // `unwrap` just panics if the result doesn't return Ok
    let my_expression: RecExpr<SymbolLang> = "(foo a b)".parse().unwrap();
    println!("This is my expression {}", my_expression);

    // let's try to create an e-node, but hmmm,
    // what do I put as the children?
    let _my_enode = SymbolLang::new("bar", vec![]);

    let mut expr = RecExpr::default();
    let a = expr.add(SymbolLang::leaf("a"));
    let b = expr.add(SymbolLang::leaf("b"));
    let _foo = expr.add(SymbolLang::new("foo", vec![a, b]));

    // we can do the same thing with an EGraph
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

    // we can make Patterns by parsing, similar to RecExprs names preceded
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
    let rules: &[Rewrite<SymbolLang, ()>] = &[
        rw!("commute-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rw!("commute-mul"; "(* ?x ?y)" => "(* ?y ?x)"),
        rw!("add-0"; "(+ ?x 0)" => "?x"),
        rw!("mul-0"; "(* ?x 0)" => "0"),
        rw!("mul-1"; "(* ?x 1)" => "?x"),
    ];

    // While it may look like we are working with numbers,
    // SymbolLang stores everything as strings.
    // We can make our own Language later to work with other types.
    let start: RecExpr<SymbolLang> = "(+ 0 (* 1 a))".parse().unwrap();

    // That's it! We can run equality saturation now.
    let runner = Runner::default().with_expr(&start).run(rules);

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
