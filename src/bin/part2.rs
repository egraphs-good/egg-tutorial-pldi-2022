use egg::*;
use egg_tutorial_pldi_2022::interval::Interval;

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
struct IntervalAnalysis;
impl Analysis<Math> for IntervalAnalysis {
    type Data = Interval;

    fn make(egraph: &EGraph<Math, Self>, enode: &Math) -> Self::Data {
        let get = |id: &Id| &egraph[*id].data;
        match enode {
            Math::Num(n) => Interval::singleton(n.clone()),
            Math::Add([a, b]) => get(a) + get(b),
            Math::Sub([a, b]) => get(a) - get(b),
            Math::Mul([a, b]) => get(a) + get(b),
            Math::Div([a, b]) => get(a) / get(b),
            _ => Interval::default(),
        }
    }

    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        egg::merge_option(&mut to.lo, from.lo, egg::merge_max)
            | egg::merge_option(&mut to.hi, from.hi, egg::merge_min)
    }
}

fn main() {
    egg_tutorial_pldi_2022::make_repl(|string| {});
}
