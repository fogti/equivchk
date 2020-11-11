use std::collections::HashMap;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Default)]
pub struct VarRc<T> {
    inner: Rc<Cell<T>>,
}

impl<T: Copy> yz_ops::eval::InnerEval for VarRc<T> {
    type Output = T;

    #[inline]
    fn eval(&self) -> Self::Output {
        self.inner.get()
    }
}

impl<T> core::ops::Deref for VarRc<T> {
    type Target = Cell<T>;

    fn deref(&self) -> &Cell<T> {
        &*self.inner
    }
}

type Vars = HashMap<String, VarRc<bool>>;

pub struct Document {
    pub vars: Vars,
    pub terms: Vec<yz_ops::Term<bool>>,
}

impl Document {
    fn tress(vars: &Vars, i: &str, s: ess::Sexp) -> yz_ops::Term<bool> {
        let l = match s {
            ess::Sexp::List(l, _) => l,
            ess::Sexp::Sym(y, _) => match &*y {
                "true" | "1" => return Box::new(E::Wrap(true)),
                "false" | "0" => return Box::new(E::Wrap(false)),
                varname if vars.contains_key(varname) => return Box::new(vars.get(varname).unwrap().clone()),
                _ => panic!("got invalid line: [{}] with string {}", i, &*y),
            }
            _ => panic!("got invalid line: [{}] with data: {:?}", i, s),
        };
        let argc = l.len();
        let mut it = l.into_iter();
        let cmd = match it.next() {
            Some(ess::Sexp::Sym(x, _)) => x,
            x => panic!("got invalid line: [{}] which contains an invalid command invocation: {:?}", i, x),
        };
        use yz_ops::{logical as L, eval as E};
        match (&*cmd, argc) {
            ("!", 2) | ("not", 2) => {
                Box::new(E::UnaryApply {
                    op: L::Not,
                    a: Self::tress(vars, i, it.next().unwrap()),
                })
            },
            ("and", _) => {
                Box::new(E::NaryApply {
                    op: L::And,
                    x: it.map(|j| Self::tress(vars, i, j)).collect(),
                })
            },
            ("or", _) => {
                Box::new(E::NaryApply {
                    op: L::Or,
                    x: it.map(|j| Self::tress(vars, i, j)).collect(),
                })
            },
            ("xor", _) => {
                Box::new(E::NaryApply {
                    op: L::Xor,
                    x: it.map(|j| Self::tress(vars, i, j)).collect(),
                })
            },
            _ => panic!("got invalid line: [{}] which contains an invalid command invocation", i),
        }
    }

    pub fn parse(s: &str) -> Document {
        let mut lines = s.lines();
        let vars: Vars = lines.next().expect("no setup given").split_whitespace().map(|i| (i.to_string(), Default::default())).collect();

        let mut terms = Vec::new();
        for i in lines {
            if i.is_empty() || i.starts_with('#') {
                continue;
            }
            let parsed = match ess::parse_one(i) {
                Ok((x, _)) => x,
                Err(e) => panic!("got invalid line: [{}] with error: {:?}", i, e),
            };
            terms.push(Self::tress(&vars, i, parsed));
        }

        Document {
            vars,
            terms,
        }
    }

    // @return: has carry?
    fn iter_permute_inner1(xs: &[VarRc<bool>]) -> bool {
        if xs.is_empty() {
            true
        } else if Self::iter_permute_inner1(&xs[1..]) {
            // got carry
            let ov = xs[0].get();
            xs[0].set(!ov);
            // if the value was 'true', propagate the carry
            ov
        } else {
            false
        }
    }

    pub fn iter_permute(&self) -> bool {
        let mut vfrefs: Vec<_> = self.vars.iter().collect();
        vfrefs.sort_unstable_by_key(|(k, _)| k.as_str());
        let vrefs: Vec<_> = vfrefs.iter().map(|(_, v)| (*v).clone()).collect();
        // 1. reset all refs
        for i in &vrefs {
            i.set(false);
        }

        let mut is_same = true;

        loop {
            // 2. print permutation
            for (k, v) in vfrefs.iter() {
                print!("{}={}, ", k, if v.get() { 1 } else { 0 });
            }

            // 3. print term results
            print!("| output");
            let mut res = Vec::new();
            for (n, t) in self.terms.iter().enumerate() {
                let y = t.eval();
                res.push(y);
                print!(", {}={}", n, if y { 1 } else { 0 });
            }
            println!();
            if !crate::is_all_same(&res[..]) {
                is_same = false;
            }

            // 4. next permutation
            if Self::iter_permute_inner1(&vrefs[..]) {
                break;
            }
        }

        is_same
    }
}
