use auto_ops::*;
use num::{BigRational, Zero};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default)]
pub struct Interval {
    pub lo: Option<BigRational>,
    pub hi: Option<BigRational>,
}

fn map2(
    a: &Option<BigRational>,
    b: &Option<BigRational>,
    f: impl FnOnce(&BigRational, &BigRational) -> BigRational,
) -> Option<BigRational> {
    if let (Some(a), Some(b)) = (a.as_ref(), b.as_ref()) {
        Some(f(a, b))
    } else {
        None
    }
}

pub fn ival(s: &str) -> Interval {
    let (lo, hi) = s.split_once(',').unwrap();
    let (lo, hi) = (lo.trim(), hi.trim());
    Interval {
        lo: if lo == "-inf" {
            None
        } else {
            Some(lo.parse().unwrap())
        },
        hi: if hi == "inf" {
            None
        } else {
            Some(hi.parse().unwrap())
        },
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        if let Some(lo) = self.lo.as_ref() {
            lo.fmt(f)?;
        } else {
            "-inf".fmt(f)?;
        }
        write!(f, ", ")?;
        if let Some(hi) = self.hi.as_ref() {
            hi.fmt(f)?;
        } else {
            "inf".fmt(f)?;
        }
        write!(f, ")")
    }
}

impl Interval {
    pub fn singleton(n: impl Into<BigRational>) -> Self {
        let n = n.into();
        Self {
            lo: Some(n.clone()),
            hi: Some(n),
        }
    }

    pub fn get_constant(&self) -> Option<&BigRational> {
        match (self.lo.as_ref(), self.hi.as_ref()) {
            (Some(lo), Some(hi)) if lo == hi => Some(lo),
            _ => None,
        }
    }

    pub fn from_f64s(lo: f64, hi: f64) -> Self {
        Self {
            lo: if lo == f64::NEG_INFINITY {
                None
            } else {
                Some(BigRational::from_float(lo).unwrap())
            },
            hi: if hi == f64::INFINITY {
                None
            } else {
                Some(BigRational::from_float(hi).unwrap())
            },
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            lo: map2(&self.lo, &other.lo, |a, b| a.max(b).clone()),
            hi: map2(&self.hi, &other.hi, |a, b| a.min(b).clone()),
        }
    }

    pub fn contains(&self, n: &BigRational) -> bool {
        self.lo.as_ref().map_or(true, |lo| lo <= n) && self.hi.as_ref().map_or(true, |hi| n <= hi)
    }

    pub fn contains_zero(&self) -> bool {
        self.contains(&BigRational::zero())
    }

    pub fn recip(&self) -> Self {
        if self.contains_zero() {
            return Self::default();
        }

        let safe_recip = |x: &Option<BigRational>| match x.as_ref() {
            Some(x) if x.is_zero() => None,
            Some(x) => Some(x.recip()),
            None => Some(BigRational::zero()),
        };

        Self {
            lo: safe_recip(&self.hi),
            hi: safe_recip(&self.lo),
        }
    }
}

impl_op_ex!(+ |a: &Interval, b: &Interval| -> Interval {
    Interval {
        lo: map2(&a.lo, &b.lo, |a, b| a + b),
        hi: map2(&a.hi, &b.hi, |a, b| a + b),
    }
});

impl_op_ex!(-|a: &Interval, b: &Interval| -> Interval {
    Interval {
        lo: map2(&a.lo, &b.hi, |a, b| a - b),
        hi: map2(&a.hi, &b.lo, |a, b| a - b),
    }
});

impl_op_ex!(*|a: &Interval, b: &Interval| -> Interval {
    let compute_possible = || -> Option<[BigRational; 4]> {
        Some([
            map2(&a.lo, &b.lo, |a, b| a * b)?,
            map2(&a.lo, &b.hi, |a, b| a * b)?,
            map2(&a.hi, &b.lo, |a, b| a * b)?,
            map2(&a.hi, &b.hi, |a, b| a * b)?,
        ])
    };

    compute_possible().map_or_else(Interval::default, |possible| Interval {
        lo: possible.iter().min().cloned(),
        hi: possible.iter().max().cloned(),
    })
});

impl_op_ex!(/ |a: &Interval, b: &Interval| -> Interval {
    a * b.recip()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        assert!(ival("0, 5").contains_zero());
        assert!(ival("0, 0").contains_zero());
        assert!(!ival("-inf, -1").contains_zero());
        assert!(!ival("10, inf").contains_zero());
    }

    #[test]
    fn test_math() {}
}
