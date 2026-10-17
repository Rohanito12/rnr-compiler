use crate::ast::{BinOp, Block, Expr, FnDeclaration, Literal, Prog, Statement, UnOp, Arguments};
use crate::common::Eval;
use crate::error::Error;
use crate::intrinsics::Intrinsic; // For built-in functions

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Lit(Literal),
    UnInit,
    Mut(Box<Val>),
}

// To store vars and funcs in a single environment.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Var(Val),
    Func(Callable),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Callable {
    UserDefined(FnDeclaration),
    BuiltIn(Intrinsic), // Need to imlement PartialEq for callable to fix warning.
}

// Helpers for Val
// Alternatively implement the TryFrom trait
impl Val {
    pub fn get_bool(&self) -> Result<bool, Error> {
        match self {
            Val::Lit(Literal::Bool(b)) => Ok(*b),
            _ => Err(format!("cannot get Bool from {:?}", self)),
        }
    }

    pub fn get_int(&self) -> Result<i32, Error> {
        match self {
            Val::Lit(Literal::Int(i)) => Ok(*i),
            _ => Err(format!("cannot get integer from {:?}", self)),
        }
    }
}

// Helper for Op
impl BinOp {
    // Evaluate operator to literal
    pub fn eval(&self, left: Val, right: Val) -> Result<Val, Error> {
        match self {
            BinOp::Add => Ok(Val::from(left.get_int()? + right.get_int()?)),
            BinOp::Sub => Ok(Val::from(left.get_int()? - right.get_int()?)),
            BinOp::Mul => Ok(Val::from(left.get_int()? * right.get_int()?)),
            BinOp::Div => {
                let r = right.get_int()?;
                if r == 0 {
                    return Err("Can't divide by zero.".to_string());
                }
                Ok(Val::from(left.get_int()? / r))
            },
            // (Shortcut evaluation) Need to implement in eval_expr to avoid evaluating the 
            // right side if the left side is enough, e.g., false for AND, true for OR.
            BinOp::And => unimplemented!(), // Ok(Val::from(left.get_bool()? && right.get_bool()?))
            BinOp::Or => unimplemented!(),  // Ok(Val::from(left.get_bool()? || right.get_bool()?))
            BinOp::Eq => {
                let l_lit: Literal = left.into();
                let r_lit: Literal = right.into();
                Ok(Val::from(l_lit == r_lit))
            }
            BinOp::Lt => Ok(Val::from(left.get_int()? < right.get_int()?)),
            BinOp::Gt => Ok(Val::from(left.get_int()? > right.get_int()?)),
        }
    }
}

impl Eval<Val> for Expr {
    fn eval(&self) -> Result<Val, Error> {
        let mut vm = VM::new();
        vm.eval_expr(self)
    }
}

impl Eval<Val> for Block {
    fn eval(&self) -> Result<Val, Error> {
        let mut vm = VM::new();
        vm.eval_block(self)
    }
}

impl Eval<Val> for Prog {
    fn eval(&self) -> Result<Val, Error> {
        let mut vm = VM::new();

        // Register built in functions in the program's global scope (bottom).
        let (println_fn_decl, println_fn) = crate::intrinsics::vm_println();
        vm.env[0].insert(println_fn_decl.id.clone(), Entry::Func(Callable::BuiltIn(println_fn)));

        // Register all user functions. "A program at the top level is a set of function declarations."
        for fn_decl in &self.0{
            vm.env[0].insert(fn_decl.id.clone(), Entry::Func(Callable::UserDefined(fn_decl.clone())));
        }
        vm.eval_call("main", &Arguments(vec![]))
    }
}

impl From<Literal> for Val {
    fn from(lit: Literal) -> Self {
        Val::Lit(lit)
    }
}

impl From<Val> for Literal {
    fn from(val: Val) -> Self {
        match val {
            Val::Lit(lit) => lit,
            _ => panic!("cannot convert to Literal from {:?}", val),
        }
    }
}

impl From<i32> for Val {
    fn from(val: i32) -> Self {
        Val::Lit(val.into())
    }
}

impl From<bool> for Val {
    fn from(val: bool) -> Self {
        Val::Lit(val.into())
    }
}

impl From<()> for Val {
    fn from(val: ()) -> Self {
        Val::Lit(val.into())
    }
}

impl From<Val> for i32 {
    fn from(val: Val) -> Self {
        match val {
            Val::Lit(Literal::Int(i)) => i,
            _ => panic!("cannot get int from {:?}", val),
        }
    }
}

impl From<Literal> for bool {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Bool(b) => b,
            _ => panic!("cannot get bool from {:?}", lit),
        }
    }
}

use std::collections::HashMap;
// here you implement your VM
pub struct VM {
    env: Vec<HashMap<String, Entry>>,
}

impl VM {
    #![allow(clippy::new_without_default)]
    pub fn new() -> Self {
        VM {
            env: vec![HashMap::new()],
        }
    }

    // Helpers
    fn get_entry(&self, id: &str) -> Result<Entry, Error> {
        for scope in self.env.iter().rev() {
            if let Some(entry) = scope.get(id) {
                return Ok(entry.clone());
            }
        }
        Err(format!("Entry `{}` not found.", id))
    }

    fn get_var(&self, id: &str) -> Result<Val, Error> {
        match self.get_entry(id)? {
            Entry::Var(val) => match val {
                // In case of mutable var the box returned from cloning needs to be
                // dereferenced to get the inner value, i.e., a Val.
                Val::Mut(val_boxed) => Ok(*val_boxed.clone()),
                _ => Ok(val.clone()),
            },
            _ => Err(format!("`{}` is not a variable.", id)), // Func case.
        }
    }

    fn get_fn(&self, id: &str) -> Result<Callable, Error> {
        match self.get_entry(id)? {
            Entry::Func(func) => Ok(func.clone()),
            _ => Err(format!("`{}` is not a function.", id)), // Var case.
        }
    }

    fn find_var_mut(&mut self, id: &str) -> Result<&mut Val, Error> {
        for scope in self.env.iter_mut().rev() {
            if let Some(entry) = scope.get_mut(id) {
                match entry {
                    Entry::Var(val) => return Ok(val),
                    _ => return Err(format!("`{}` is not a function not a mut variable.", id)),
                }
            }
        }
        Err(format!("Variable `{}` not found.", id))
    }

    // To implement the first pass of function registration in a block before executing.
    fn register_functions(&mut self, statements: &[Statement]) -> Result<(), Error> {
        let curr_scope = self.env.last_mut().ok_or("Error: No scope.")?;
        
        for stmt in statements {
            if let Statement::Fn(decl) = stmt {
                // Check for duplicates to prevent sequential shadowing of functions.
                if curr_scope.contains_key(&decl.id) {
                     return Err(format!("Function `{}` is already declared in this scope.", decl.id));
                }
                curr_scope.insert(decl.id.clone(), Entry::Func(Callable::UserDefined(decl.clone())));
            }
        }
        Ok(())
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Val, Error> {
        match expr {
            Expr::Lit(lit) => Ok(Val::from(lit.clone())),
            Expr::Par(expr) => self.eval_expr(expr),
            Expr::Ident(id) => self.get_var(id), // Only variables for now.
            Expr::BinOp(op, left, right) => {
                match op {
                    BinOp::And => {
                        let l_val = self.eval_expr(left)?;
                        if !l_val.get_bool()? {
                            Ok(Val::from(false))
                        } else {
                            // Now it's okay to evaluate the right side.
                            let r_val = self.eval_expr(right)?;
                            Ok(Val::from(r_val.get_bool()?))
                        }
                    },
                    BinOp::Or => {
                        let l_val = self.eval_expr(left)?;
                        if l_val.get_bool()? {
                            Ok(Val::from(true))
                        } else {
                            // Same here, if its false then this side decides.
                            let r_val = self.eval_expr(right)?;
                            Ok(Val::from(r_val.get_bool()?))
                        }
                    },
                    _ => { // Other binary ops.
                        let left_val = self.eval_expr(left)?;
                        let right_val = self.eval_expr(right)?;
                        op.eval(left_val, right_val)
                    }
                }
            }
            Expr::Call(id, args) => self.eval_call(id, args),
            Expr::IfThenElse(cond, then_block, else_block) => {
                // Check the condition.
                if self.eval_expr(cond)?.get_bool()? {
                    // Evaluate the block if true.
                    self.eval_block(then_block)
                    // Otherwise check for else block and evaluate that.
                } else if let Some(else_b) = else_block {
                    self.eval_block(else_b)
                } else {
                    // No else block, return unit.
                    Ok(Val::from(())) 
                }
            },
            Expr::Block(block) => self.eval_block(block),
            Expr::UnOp(op, expr) => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnOp::Bang => Ok(Val::from(!val.get_bool()?)),
                    UnOp::Neg => Ok(Val::from(-val.get_int()?)),
                }
            },
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Statement) -> Result<Val, Error> {
        match stmt {
            Statement::Let(mutable, id, _ty, expr) => {
                // Check the variable starting value.
                let val = match expr {
                    Some(e) => self.eval_expr(e)?,
                    // No assignment implies uninitialized value.
                    None => Val::UnInit,
                };

                // If it's mutable, then it has to be wrapped.
                let entry = if mutable.0 {
                    Entry::Var(Val::Mut(Box::new(val)))
                } else {
                    Entry::Var(val)
                };
                
                // The insert will replace an existing var/fn. Now variables can shadow functions.
                if let Some(scope) = self.env.last_mut() {
                    scope.insert(id.clone(), entry);
                } else {
                    return Err("No scope available.".to_string());
                }
                
                Ok(Val::from(()))
            }
            Statement::Assign(lexpr, rexpr) => {
                let new_val = self.eval_expr(rexpr)?;
                
                if let Expr::Ident(id) = lexpr {
                    // Searches in the env for an Entry::Var matching the id.
                    let var_to_update = self.find_var_mut(id)?;
                    match var_to_update {
                        // If it's mutable, update the boxed value.
                        Val::Mut(val_boxed) => {
                            **val_boxed = new_val;
                        }
                        Val::UnInit => {
                            // Alternatively if the var is not initialized and mutable
                            // rust lets you assign to it once.
                            *var_to_update = new_val;
                        }
                        _ => return Err(format!("Variable `{}` is not mutable.", id)),
                    }
                    Ok(Val::from(()))
                } else {
                    Err("Left stmt of an assignment must be an identifier.".to_string())
                }
            },
            Statement::While(cond, block) => {
                // Check if the condition is true and evaluate the block repeatedly until it isn't.
                // Reminder for lab4: Don't check the condition multiple times for the type checking system.
                while self.eval_expr(cond)?.get_bool()? {
                    self.eval_block(block)?;
                }
                Ok(Val::from(()))
            },
            Statement::Expr(expr) => self.eval_expr(expr),
            // Handling function registration in pass 1 with register_functions 
            // when executing, i.e., pass 2 so not doing anything here.
            Statement::Fn(func) => Ok(Val::from(())), 
        }
    }
    
        // Executes statements without pushing a new scope.
    fn eval_block_statements(&mut self, block: &Block) -> Result<Val, Error> {
        // Pass 1: register funcs.
        self.register_functions(&block.statements)?;

        // Pass 2: execute statements.
        let mut last_val = Val::Lit(Literal::Unit);
        for stmt in &block.statements {
            match stmt {
                // Ones that don't return a value. 
                Statement::Let(_, _, _, _) | Statement::Fn(_) | Statement::While(_, _) | Statement::Assign(_, _) => {
                    self.eval_stmt(stmt)?;
                    last_val = Val::Lit(Literal::Unit);
                }
                // Ones that return a value.
                Statement::Expr(e) => {
                    last_val = self.eval_expr(e)?;
                }
            }
        }

        // If the blocks ends with a semi-colon return Unit, otherwise return the last value.
        if block.semi {
            Ok(Val::Lit(Literal::Unit))
        } else {
            Ok(last_val)
        }
    }

    pub fn eval_block(&mut self, block: &Block) -> Result<Val, Error> {
        // Create a new scope by pushing to the stack.
        self.env.push(HashMap::new());

        let last_val = self.eval_block_statements(block)?;

        // Exit the scope by popping from the stack.
        self.env.pop();
        
        Ok(last_val)
    }

    fn eval_call(&mut self, id: &str, args: &Arguments) -> Result<Val, Error> {
        // Evaluate all argument expressions.
        let mut args_vals = Vec::new();
        for arg in &args.0 {
            args_vals.push(self.eval_expr(arg)?);
        }

        // Check if the function is in the env, dip out if it's not there.
        let func = self.get_fn(id)?;

        match func {
            Callable::UserDefined(fn_decl) => {
                // Check that the number of arguments is correct.
                if fn_decl.parameters.0.len() != args_vals.len() {
                    return Err(format!("Expected `{}` args but got {}.", fn_decl.parameters.0.len(), args_vals.len()));
                }

                // Create the function scope.
                self.env.push(HashMap::new());

                // Bind the arguments to the parameter in the new scope.
                for (param, arg_val) in fn_decl.parameters.0.iter().zip(args_vals.iter()){
                    let to_store = if param.mutable.0 { 
                        Entry::Var(Val::Mut(Box::new(arg_val.clone())))
                    } else { 
                        Entry::Var(arg_val.clone())
                    };
                    self.env.last_mut().unwrap().insert(param.id.clone(), to_store);
                }

                // Evaluate the function body.
                let result = self.eval_block_statements(&fn_decl.body);

                // Exit the function scope.
                self.env.pop();
                return result;
            },
            Callable::BuiltIn(intrinsic) => {
                // Convert the args to literals for the intrinsic function.
                let mut lit_args = Vec::new();
                for val in args_vals {
                    match val {
                        Val::Lit(lit) => lit_args.push(lit),
                        _ => return Err("Args must be literals for intrinsics.".to_string()),
                    }
                }
                let result = intrinsic(lit_args);
                Ok(Val::from(result))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Val;
    use crate::ast::Literal;
    use crate::ast::{Block, Expr, Prog};
    use crate::common::parse_test;

    #[test]
    fn test_expr() {
        let v = parse_test::<Expr, Val>(
            "        
        1 + 1
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_bool() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = true && false;
        a
    }
    ",
        );

        assert!(!v.unwrap().get_bool().unwrap());
    }

    #[test]
    fn test_bool_bang2() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = (!true) && false;
        a
    }
    ",
        );

        assert!(!v.unwrap().get_bool().unwrap());
    }

    #[test]
    fn test_bool_bang() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = true && !false;
        a
    }
    ",
        );

        assert!(v.unwrap().get_bool().unwrap());
    }

    #[test]
    fn test_block_let() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a: i32 = 1;
        let b: i32 = 2;

        a + b
    }",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 3);
    }

    #[test]
    fn test_block_let_shadow() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a: i32 = 1;
        let b: i32 = 2;
        let a: i32 = 3;
        let b: i32 = 4;

        a + b
    }",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 7);
    }

    #[test]
    fn test_local_block() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = 1;
        let b = {
            let b = a;
            b * 2
        };

        b
    }
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_block_assign() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut a: i32 = 1;
        a = a + 2;
        a
    }",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 3);
    }

    #[test]
    fn test_expr_if_then_else() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut a: i32 = 1;
        a = if a > 0 { a + 1 } else { a - 2 };
        a
    }",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_while() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut a = 2;
        let mut b = 0;
        while a > 0 {
            a = a - 1;
            b = b + 1;
        }
        b
    }
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_prog() {
        let v = parse_test::<Prog, Val>(
            "
    fn main() {
        let a = 1;
        a
    }
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 1);
    }

    #[test]
    fn test_local_fn() {
        let v = parse_test::<Prog, Val>(
            "
    fn main() {
        fn f(i: i32, j: i32) -> i32 {
            i + j
        }
        let a = f(1, 2);
        println!(\"a = {} and another a = {}\", a, a);
    }
    ",
        );

        assert_eq!(v.unwrap(), Val::Lit(Literal::Unit));
    }

    #[test]
    fn test_check_if_then_else_shadowing() {
        let v = parse_test::<Block, Val>(
            "
        {
            let mut a: i32 = 1 + 2; // a == 3
            let mut a: i32 = 2 + a; // a == 5
            if true {
                a = a - 1;      // outer a == 4
                let mut a: i32 = 0; // inner a == 0
                a = a + 1       // inner a == 1
            } else {
                a = a - 1
            };
            a   // a == 4
        }
        ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 4);
    }
}
