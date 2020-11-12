use std::collections::BTreeMap;
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

type Vars = BTreeMap<String, VarRc<u32>>;

pub struct Document {
    pub vars: Vars,
    pub terms: Vec<yz_ops::Term<u32>>,
}

const PREPERMUTE: &[u32] = &[0xaaaaaaaa, 0xcccccccc, 0xf0f0f0f0, 0xff00ff00, 0xffff0000];
const PERM_ULEN: usize = PREPERMUTE.len();

impl Document {
    fn tress(vars: &Vars, i: &str, s: ess::Sexp) -> yz_ops::Term<u32> {
        let l = match s {
            ess::Sexp::List(l, _) => l,
            ess::Sexp::Sym(y, _) => match &*y {
                "true" | "1" => return Box::new(E::Wrap(u32::MAX)),
                "false" | "0" => return Box::new(E::Wrap(0)),
                varname if vars.contains_key(varname) => {
                    return Box::new(vars.get(varname).unwrap().clone())
                }
                _ => panic!("got invalid line: [{}] with string {}", i, &*y),
            },
            _ => panic!("got invalid line: [{}] with data: {:?}", i, s),
        };
        let argc = l.len();
        let mut it = l.into_iter();
        let cmd = match it.next() {
            Some(ess::Sexp::Sym(x, _)) => x,
            x => panic!(
                "got invalid line: [{}] which contains an invalid command invocation: {:?}",
                i, x
            ),
        };
        use yz_ops::{eval as E, logical as L};
        match (&*cmd, argc) {
            ("!", 2) | ("not", 2) => Box::new(E::UnaryApply {
                op: L::Not,
                a: Self::tress(vars, i, it.next().unwrap()),
            }),
            ("&", _) | ("and", _) => Box::new(E::NaryApply {
                op: L::And,
                x: it.map(|j| Self::tress(vars, i, j)).collect(),
            }),
            ("|", _) | ("or", _) => Box::new(E::NaryApply {
                op: L::Or,
                x: it.map(|j| Self::tress(vars, i, j)).collect(),
            }),
            ("xor", _) => Box::new(E::NaryApply {
                op: L::Xor,
                x: it.map(|j| Self::tress(vars, i, j)).collect(),
            }),
            _ => panic!(
                "got invalid line: [{}] which contains an invalid command invocation",
                i
            ),
        }
    }

    pub fn parse(s: &str) -> Document {
        let mut lines = s.lines();
        let vars: Vars = lines
            .next()
            .expect("no setup given")
            .split_whitespace()
            .map(|i| (i.to_string(), Default::default()))
            .collect();

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

        Document { vars, terms }
    }

    pub fn all_permute(&self) -> bool {
        let vrefs: Vec<_> = self.vars.iter().map(|(_, v)| (*v).clone()).collect();
        let mut is_same = true;

        // 1. prepare the variables
        for chunk in vrefs.rchunks(PREPERMUTE.len()) {
            chunk
                .iter()
                .zip(PREPERMUTE.iter())
                .for_each(|(i, &p)| i.set(p));
        }

        loop {
            // 2. print permutation
            for (k, v) in self.vars.iter() {
                print!("{}={:b} ", k, v.get());
            }

            // 3. print term results
            print!("::");
            let mut res = Vec::new();
            for (n, t) in self.terms.iter().enumerate() {
                let y = t.eval();
                res.push(y);
                print!(" {}={:b}", n, y);
            }
            println!();

            // 4. check if term results are the same
            if !crate::is_all_same(&res[..]) {
                is_same = false;
            }

            // 5. next permutation
            if vrefs.len() <= PERM_ULEN {
                break;
            }
            let mut has_carry = true;
            for chunk in vrefs[PERM_ULEN..].rchunks(PERM_ULEN) {
                has_carry = true;
                for i in chunk.iter() {
                    let cur = i.get();
                    if cur & 0b1 == 0b0 {
                        // we don't have a carry
                        has_carry = false;
                    }
                    // invert
                    i.set(!cur);
                }
                if !has_carry {
                    // no carry to propagate, we are done with incrementing
                    break;
                }
            }
            if has_carry {
                break;
            }
        }

        is_same
    }
}
