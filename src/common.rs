use crate::error::Error;

pub trait Eval<T: Clone> {
    fn eval(&self) -> Result<T, Error>
    where
        T: Clone;
}

pub fn parse<T1, T2>(s: &str) -> T1
where
    T1: syn::parse::Parse + std::fmt::Display,
    T2: Clone,
{
    let ts: proc_macro2::TokenStream = s.parse().unwrap();
    let r: T1 = syn::parse2(ts).unwrap();
    println!("{}", r);
    r
}

pub fn parse_test<T1, T2>(s: &str) -> Result<T2, Error>
where
    T1: syn::parse::Parse + std::fmt::Display + Eval<T2>,
    T2: std::fmt::Debug + Clone,
{
    let bl = parse::<T1, T2>(s);
    let v = bl.eval()?;
    println!("\nreturn {:?}", v);
    Ok(v)
}
