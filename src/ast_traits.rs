// Extra traits implemented for AST

use crate::ast::*;
use std::fmt;

// Utility functions/traits for your AST here.

impl Expr {
    pub fn bin_op(o: BinOp, left: Expr, right: Expr) -> Self {
        Expr::BinOp(o, Box::new(left), Box::new(right))
    }
}

/// Anything that can be converted into a Literal (like i32, bool, etc) can also be converted into an Expr.
impl<T: Into<Literal>> From<T> for Expr {
    fn from(x: T) -> Self {
        Expr::Lit(x.into())
    }
}

/// Anything that can be converted to a Literal can also be converted to a Type.
impl<T: Into<Literal>> From<T> for Type {
    fn from(x: T) -> Self {
        let lit: Literal = x.into();
        match lit {
            Literal::Unit => Type::Unit,
            Literal::Bool(_) => Type::Bool,
            Literal::Int(_) => Type::I32,
            Literal::String(_) => Type::String,
        }
    }
}

impl From<i32> for Literal {
    fn from(i: i32) -> Self {
        Literal::Int(i)
    }
}

impl From<bool> for Literal {
    fn from(b: bool) -> Self {
        Literal::Bool(b)
    }
}

impl From<()> for Literal {
    fn from(_: ()) -> Self {
        Literal::Unit
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Literal::String(s)
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Eq => "==",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
        };
        write!(f, "{}", s)
    }
}

// Your ast Display traits here
impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(i) => write!(f, "{}", i),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Unit => write!(f, "()"),
        }
    }
}

#[test]
fn display_literal() {
    println!("{}", Literal::Int(3));
    println!("{}", Literal::Bool(false));
    println!("{}", Literal::Unit);
    assert_eq!(format!("{}", Literal::Int(3)), "3");
    assert_eq!(format!("{}", Literal::Bool(false)), "false");
    assert_eq!(format!("{}", Literal::Unit), "()");
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "String"),
            Type::Unit => write!(f, "()"),
        }
    }
}

#[test]
fn display_type() {
    assert_eq!(format!("{}", Type::I32), "i32");
    assert_eq!(format!("{}", Type::Bool), "bool");
    assert_eq!(format!("{}", Type::Unit), "()");
    assert_eq!(format!("{}", Type::String), "String");
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Bang => write!(f, "!"),
            UnOp::Neg => write!(f, "-"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Ident(s) => write!(f, "{}", s),
            Expr::Lit(lit) => write!(f, "{}", lit),
            Expr::BinOp(op, left, right) => write!(f, "{} {} {}", left, op, right),
            Expr::Par(expr) => write!(f, "({})", expr),
            Expr::Call(id, args) => write!(f, "{}{}", id, args),
            Expr::IfThenElse(cond, then_block, else_block) => {
                write!(f, "if {} {}", cond, then_block)?;
                // Some evaluates to true if the Option is Some, false if None, i.e.,
                // if there is an else block.
                if let Some(else_b) = else_block {
                    write!(f, " else {}", else_b)?;
                }
                Ok(())
            }
            Expr::Block(block) => write!(f, "{}", block),
            Expr::UnOp(op, expr) => write!(f, "{}{}", op, expr),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\n")?;
        for (i, stmt) in self.statements.iter().enumerate() {
            // 4 spaces for indentation.
            write!(f, "    {}", stmt)?;
            
            match stmt {
                // Let stmt must have a semicolon regardless of position or semi flag.
                Statement::Let(_, _, _, _) => {
                    write!(f, ";")?; 
                }

                // Expressions and assignments need a semicolon if they are not the last statement
                // or if the semi flag is set.
                Statement::Expr(_) | Statement::Assign(_, _) => {
                    if i < self.statements.len() - 1 || self.semi {
                        write!(f, ";")?;
                    }
                }

                // while and fn don't need a semicolon.
                _ => {} 
            }
            write!(f, "\n")?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Mutable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 {
            write!(f, "mut ")
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}: {}", self.mutable, self.id, self.ty)
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, p) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", p)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Arguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, arg) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for FnDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn {}{}", self.id, self.parameters)?;
        if let Some(ty) = &self.ty {
            write!(f, " -> {}", ty)?;
        }
        write!(f, " {}", self.body)
    }
}

impl fmt::Display for Prog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for func in &self.0 {
            writeln!(f, "{}", func)?;
        }
        Ok(())
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Let(mutable, id, ty, expr) => {
                write!(f, "let ")?;
                if mutable.0 {
                    write!(f, "mut ")?;
                }
                write!(f, "{}", id)?;
                if let Some(ty) = ty {
                    write!(f, ": {}", ty)?;
                }
                if let Some(expr) = expr {
                    write!(f, " = {}", expr)?;
                }
                Ok(())
            }
            // No semicolon here, otherwise { a = 1 } won't work.
            Statement::Assign(lexpr, rexpr) => write!(f, "{} = {}", lexpr, rexpr),
            Statement::While(cond, block) => write!(f, "while {} {}", cond, block),
            Statement::Expr(expr) => write!(f, "{}", expr),
            Statement::Fn(func) => write!(f, "{}", func),
        }
    }
}

#[test]
fn display_if_then_else() {
    let ts: proc_macro2::TokenStream = "
    if a {
        let a : i32 = false;
        0
    } else {
        if a == 5 { b = 8 };
        while b {
            e;
        }
        b
    }
    "
    .parse()
    .unwrap();
    let e: Expr = syn::parse2(ts).unwrap();
    println!("ast:\n{:?}", e);

    println!("pretty:\n{}", e);
}

#[test]
fn display_while() {
    let ts: proc_macro2::TokenStream = "
    while a == 9 {
        let b : i32 = 7;
    }
    "
    .parse()
    .unwrap();
    let e: Statement = syn::parse2(ts).unwrap();
    println!("ast:\n{:?}", e);

    println!("pretty:\n{}", e);
}

#[test]
fn display_expr() {
    println!("{}", Expr::Ident("a".to_string()));
    println!("{}", Expr::Lit(Literal::Int(7)));
    println!("{}", Expr::Lit(Literal::Bool(false)));
    let e = Expr::BinOp(
        BinOp::Add,
        Box::new(Expr::Ident("a".to_string())),
        Box::new(Expr::Lit(Literal::Int(7))),
    );
    println!("{}", e);
    assert_eq!(format!("{}", e), "a + 7");
}

// As you see it becomes cumbersome to write tests
// if you have to construct the Expr by hand.
//
// Instead we might use our parser

#[test]
fn parse_display_expr() {
    let ts: proc_macro2::TokenStream = "a + 7".parse().unwrap();
    let e: Expr = syn::parse2(ts).unwrap();
    println!("e {}", e);
}

// This one will fail (Display for `if` is not yet implemented).
// Implement it as an optional assignment
//
// Hint: You need to implement Display for Statement and Block

#[test]
fn parse_display_if() {
    let ts: proc_macro2::TokenStream = "if a > 5 {5}".parse().unwrap();
    let e: Expr = syn::parse2(ts).unwrap();
    println!("e {}", e);
}