use egg::*;

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

fn rules() -> Vec<Rewrite<Math, ConstantFold>> {
    vec![
        rewrite!("commute-add"; "(+ ?x ?y)" => "(+ ?y ?x)"),
        rewrite!("commute-mul"; "(* ?x ?y)" => "(* ?y ?x)"),
        rewrite!("add-0"; "(+ ?x 0)" => "?x"),
        rewrite!("mul-0"; "(* ?x 0)" => "0"),
        rewrite!("mul-1"; "(* ?x 1)" => "?x"),
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
            Math::Mul([a, b]) => Some(get(a)? * get(b)?),
            _ => None,
        }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_option(to, from, |a, b| {
            assert_eq!(a, &b);
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

fn optimize(s: &str) -> String {
    let expr: RecExpr<Math> = s.parse().unwrap();
    let runner = Runner::default().with_expr(&expr).run(&rules());
    let extractor = Extractor::new(&runner.egraph, AstSize);

    // We want to extract the best expression represented in the
    // same e-class as our initial expression, not from the whole e-graph.
    // Luckily the runner stores the eclass Id where we put the initial expression.
    let (_best_cost, best_expr) = extractor.find_best(runner.roots[0]);
    best_expr.to_string()
}

#[test]
fn simple_test() {
    let expr: RecExpr<Math> = "(+ 3 2)".parse().unwrap();
}

fn main() {
    egg_tutorial_pldi_2022::make_repl(|s| println!("{}", optimize(s)))
}
