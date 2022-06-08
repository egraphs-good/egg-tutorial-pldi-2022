mod interval;

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

use egg::*;
use interval::Interval;

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

#[test]
fn simple_test() {
    let expr: RecExpr<Math> = "(+ 3 2)".parse().unwrap();
}
