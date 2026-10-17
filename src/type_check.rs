use crate::ast::{Type, Expr, Statement, Block, Literal, BinOp, Prog, UnOp, FnDeclaration};
use crate::error::Error;

// Checks if two types are the same.
fn unify(got: Type, expected: Type) -> Result<Type, Error> {
    match got == expected {
        true => Ok(expected),
        false => Err(format!("expected type {:?}, got type {:?}", expected, got)),
    }
}

// Store types of variables and functions (signatures)
#[derive(Clone, Debug, PartialEq)]
pub enum TypeEntry{
    Var(Type, bool), // bool to represent mutability
    Func(FuncType), 
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuncType {
    pub param_types: Vec<Type>,
    pub return_type: Box<Type>,
}

use std::collections::HashMap;
use std::collections::btree_set::Union;
use std::hash::Hash;
pub struct TypeChecker {
    env: Vec<HashMap<String, TypeEntry>>,
}

impl TypeChecker{

    pub fn new() -> Self {
        TypeChecker {
            env: vec![HashMap::new()],
        }
    }

    // Add type binding in the current scope.
    fn insert_type(&mut self, id: String, entry: TypeEntry) {
        self.env.last_mut().unwrap().insert(id, entry);
    }

    // Look for a type from the top of the stack downwards.
    fn get_type(&self, id: &str) -> Result<TypeEntry, Error> {
        for scope in self.env.iter().rev() {
            if let Some(t) = scope.get(id) {
                return Ok(t.clone());
            }
        }
        Err(format!("Variable or function `{}` not found", id))
    }

    pub fn push_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.env.pop();
    }

    // For Pass 1, Register the signature so other functions can see it (see check_block).
    fn register_fn(&mut self, decl: &FnDeclaration) -> Result<(), Error> {
        let param_types = decl.parameters.0.iter()
            .map(|p| p.ty.clone())
            .collect();
        
        let return_type = decl.ty.clone().unwrap_or(Type::Unit);

        let func_type = FuncType {
            param_types,
            return_type: Box::new(return_type),
        };

        // Check for sequential shadowing (duplicates in same scope)
        if self.env.last().unwrap().contains_key(&decl.id) {
            return Err(format!("Function `{}` is already declared in this scope", decl.id));
        }

        self.insert_type(decl.id.clone(), TypeEntry::Func(func_type));
        Ok(())
    }

    // For Pass 2 (see check_block).
    fn check_fn_body(&mut self, decl: &FnDeclaration) -> Result<(), Error> {
        self.push_scope();

        // Add parameters to the func scope so the body can use them
        for param in &decl.parameters.0 {
            self.insert_type(param.id.clone(), TypeEntry::Var(param.ty.clone(), param.mutable.0));
        }

        let return_type = decl.ty.clone().unwrap_or(Type::Unit);
        
        // Check block
        let body_type = self.check_block(&decl.body)?;
        
        // Ensure body matches signature
        unify(body_type, return_type)?;

        self.pop_scope();
        Ok(())
    }


    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, Error> {
        match expr {
            Expr::Lit(lit) => {
                // Type are known for literals.
                match lit {
                    Literal::Bool(_) => Ok(Type::Bool),
                    Literal::Int(_) => Ok(Type::I32),
                    Literal::String(_) => Ok(Type::String),
                    Literal::Unit => Ok(Type::Unit),
                }
            },
            Expr::Par(expr) => self.check_expr(expr),
            Expr::Ident(id) => {
                // Just look up the type in the env.
                match self.get_type(id)? {
                    TypeEntry::Var(t, _) => Ok(t),
                    _ => Err(format!("`{}` is not a variable.", id)),
                }
            },
            Expr::BinOp(op, left, right) => {
                // Start by evaluating the types of the operands.
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                match op {
                    // Addition, substraction, multiplication and division only for i32.
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        unify(left_type, Type::I32)?;
                        unify(right_type, Type::I32)?;
                        Ok(Type::I32)
                    },
                    // And + Or only for bool.
                    BinOp::And | BinOp::Or => {
                        unify(left_type, Type::Bool)?;
                        unify(right_type, Type::Bool)?;
                        Ok(Type::Bool)
                    },
                    // Greater than, less than  for i32 to bool.
                    BinOp::Gt | BinOp::Lt  => {
                        unify(left_type, Type::I32)?;
                        unify(right_type, Type::I32)?;
                        Ok(Type::Bool)
                    },
                    // Equality is overloaded for i32 and bool.
                    BinOp::Eq => {
                        // It's either both i32 or both bool so check if they have the same type.
                        unify(left_type.clone(), right_type.clone())?;
                        // Verify that the type is one them.
                        match left_type {
                            Type::I32 | Type::Bool | Type::String => Ok(Type::Bool),
                            _ => Err(format!("Undefined op for type {:?}.", left_type)),
                        }
                    }
                }
            },
            Expr::Call(id, args )=> {
                // Look up function signature and check arguments.
                match self.get_type(id)? {
                    TypeEntry::Func(func_type) => {
                        // Check number of args.
                        if args.0.len() != func_type.param_types.len() {
                            return Err(format!("Function {} expects {} args but got {}.", 
                                id, func_type.param_types.len(), args.0.len()));
                        }

                        // Check type of each argument.
                        for (arg_expr, expected_type) in args.0.iter().zip(func_type.param_types.iter()) {
                            let arg_type = self.check_expr(arg_expr)?;
                            unify(arg_type, expected_type.clone())?;
                        }

                        // Return the function return type (unbox).
                        Ok(*func_type.return_type)
                    }
                    TypeEntry::Var(_, _) => Err(format!("Cannot call variable `{}`.", id)),
                }
            },
            Expr::IfThenElse(cond_expr, then_block, else_block_opt) => {
                // Check that condition is bool.
                let cond_type = self.check_expr(cond_expr)?;
                unify(cond_type, Type::Bool)?;
                // Check then block.
                let then_type = self.check_block(then_block)?;
                // Check else block if there is one.
                if let Some(else_block) = else_block_opt {
                    let else_type = self.check_block(else_block)?;
                    // Then and else must have the same type.
                    unify(then_type, else_type)
                } else {
                    // No else block, then block must be unit.
                    unify(then_type, Type::Unit)
                }
            },
            Expr::Block(block) => self.check_block(block),
            Expr::UnOp(op, expr) => {
                let expr_type = self.check_expr(expr)?;
                match op {
                    UnOp::Bang => {
                        // Can only negate bools.
                        unify(expr_type, Type::Bool)?;
                        Ok(Type::Bool)
                    },
                    UnOp::Neg => {
                        // Can only negate integers.
                        unify(expr_type, Type::I32)?;
                        Ok(Type::I32)
                    },
                }
            }  
        }
    }

    pub fn check_stmt(&mut self, stmt: &Statement) -> Result<Type, Error> {
        match stmt {
            Statement::Let(mutable, id , opt_ty, opt_expr) => {
                // If there is a starting value check the type.
                let expr_type = if let Some(expr) = opt_expr {
                    Some(self.check_expr(expr)?)
                } else {
                    None
                };
            
                let final_type = match (opt_ty, expr_type)  {
                    // Check annotation vs expr (unify).
                    (Some(t), Some(e_t)) => unify(t.clone(), e_t)?,
                    // Only annotation.
                    (Some(t), None) => t.clone(),
                    // Only expr so use its type.
                    (None, Some(e_t)) => e_t,
                    // Otherwise, cannot determine the type
                    (None, None) => return Err(format!("Cannot determine type for variable {}.", id)),
                };

                // Add the type for the var to the env.
                self.insert_type(id.clone(), TypeEntry::Var(final_type, mutable.0));
                Ok(Type::Unit)
            }
        
            Statement::Expr(expr) => self.check_expr(expr),
            Statement::Assign(lexpr, rexpr) => {
                let rexpr_type = self.check_expr(rexpr)?;

                match lexpr {
                    Expr::Ident(id) => {        
                        match self.get_type(id)? {
                            TypeEntry::Var(lexpr_type, is_mut) => {
                                if !is_mut {
                                    return Err(format!("Cannot assign to immutable variable `{}`.", id));
                                }
                                unify(lexpr_type, rexpr_type)?;
                            }
                            TypeEntry::Func(_) => return Err(format!("Cannot assign to function `{}`.", id)),
                        }
                    },
                    _ => return Err("Left expression of assignment must be an identifier.".to_string()),
                }
                Ok(Type::Unit)
            },
            Statement::While(cond, block) => {
                // Check condition once.
                let cond_type = self.check_expr(cond)?;
                unify(cond_type, Type::Bool)?;
                // Check that the body is valid.
                let block_type = self.check_block(block)?; 
                
                unify(block_type, Type::Unit)?;

                Ok(Type::Unit)
            },
            Statement::Fn(decl) => {
                // Signature is already register so only the body should be checked.
                self.check_fn_body(decl)?;
                Ok(Type::Unit)
            },
        }
    }

    pub fn check_block(&mut self, block: &Block) -> Result<Type, Error> {
        self.push_scope();

        // Pass 1: register funcs.
        for stmt in &block.statements {
            if let Statement::Fn(decl) = stmt {
                self.register_fn(decl)?;
            }
        }

        // Start with a unit type.
        let mut last_type = Type::Unit;

        // Check each statement in order.
        for stmt in &block.statements {
            // Unit for let/while. Type of an expression for expr.
            last_type = self.check_stmt(stmt)?;
        }
        
        self.pop_scope();
        
        // The type of the block is the type of the last statement, either Unit or type of last expr.
        if block.semi {
            Ok(Type::Unit)
        } else {
            Ok(last_type)
        }
    }
}

// Eval trait for testing.
use crate::common::Eval;
impl Eval<Type> for Expr {
    fn eval(&self) -> Result<Type, Error> {
        let mut tc = TypeChecker::new();
        tc.check_expr(self)
    }
}

impl Eval<Type> for Block {
    fn eval(&self) -> Result<Type, Error> {
        let mut tc = TypeChecker::new();
        tc.check_block(self)
    }
}

impl Eval<Type> for Prog {
    fn eval(&self) -> Result<Type, Error> {
        let mut tc = TypeChecker::new();
        // Register intrinsic signatures.
        let (println_decl, _) = crate::intrinsics::vm_println();
        tc.register_fn(&println_decl)?;

        // Register user functions.
        for fn_decl in &self.0 {
            tc.register_fn(fn_decl)?;
        }

        // Check user funcs bodies.
        for fn_decl in &self.0 {
            tc.check_fn_body(fn_decl)?;
        }

        // Return type of main is Unit. Could be something else based on a pr, check later.
        Ok(Type::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr};
    use crate::parse::parse;
    use crate::test_util::{ assert_type, assert_type_fail};

    #[test]
    fn test_literal_int32() {
        let expr = parse::<Expr>("1");
        assert_type(&expr, Type::I32);
    }

    #[test]
    fn test_literal_bool(){
        let expr = parse::<Expr>("true");
        assert_type(&expr, Type::Bool);
    }

    #[test]
    fn test_bin_op(){
        let expr = parse::<Expr>("1+2");
        assert_type(&expr,Type::I32);
    }

        #[test]
    fn test_bin_op_add_fail() {
        // 1 + true should be an error
        let expr = Expr::bin_op(BinOp::Add, Expr::from(1), Expr::from(true));
        assert_type_fail(&expr);
    }

    #[test]
    fn test_bin_op_eq_bool() {
        let expr = parse::<Expr>("true == false");
        assert_type(&expr, Type::Bool);
    }

    #[test]
    fn test_bin_op_eq_i32() {
        let expr = parse::<Expr>("1 == 2");
        assert_type(&expr, Type::Bool);
    }

    #[test]
    fn test_bin_op_eq_fail() {
        let expr = parse::<Expr>("1 == true");
        assert_type_fail(&expr);
    }

    #[test]
    fn test_bin_op_and() {
        let expr = parse::<Expr>("true && false");
        assert_type(&expr, Type::Bool);
    }

    #[test]
    fn test_bin_op_and_fail() {
        let expr = parse::<Expr>("1 && false");
        assert_type_fail(&expr);
    }

    #[test]
    fn test_un_op_neg() {
        let expr = parse::<Expr>("-1");
        assert_type(&expr, Type::I32);
    }

    #[test]
    fn test_un_op_neg_fail() {
        let expr = parse::<Expr>("-true");
        assert_type_fail(&expr);
    }

    #[test]
    fn test_un_op_bang() {
        let expr = parse::<Expr>("!true");
        assert_type(&expr, Type::Bool);
    }

    #[test]
    fn test_un_op_bang_fail() {
        let expr = parse::<Expr>("!1");
        assert_type_fail(&expr);
    }

    #[test]
    fn test_check_if_else() {
        let src = "if (true) { 1 } else { 2 }"; // Also quick Par check.
        let expr: Expr = parse(src);
        assert_type(&expr, Type::I32);
    }

    #[test]
    fn test_check_if_no_else() {
        let src = "if true { 1; }"; // Unit return
        let expr: Expr = parse(src);
        assert_type(&expr, Type::Unit);
    }

    #[test]
    fn test_check_if_else_different() {
        let src = "if true { 1 } else { false }";
        let expr: Expr = parse(src);
        assert_type_fail(&expr);
    }
    
    #[test]
    fn test_check_let_inference() {
        let block: Block = parse("{ let a = 1; a }");
        assert_type(&block, Type::I32);
    }

    #[test]
    fn test_assign_mutable() {
        let block: Block = parse("{ let mut a = 1; a = 2; a }");
        assert_type(&block, Type::I32);
    }

    #[test]
    fn test_assign_mutable_fail() {
        let block: Block = parse("{ let mut a = 1; a = true; a }");
        assert_type_fail(&block);
    }

    #[test]
    fn test_assign_immutable_fail() {
        let block: Block = parse("{ let a = 1; a = 2; a }");
        assert_type_fail(&block);
    }

    #[test]
    fn test_fn_call() {
        let block: Block = parse("{ fn add(a: i32, b: i32) -> i32 { a + b } add(1, 2) }");
        assert_type(&block, Type::I32);
    }

    #[test]
    fn test_fn_call_wrong_parameters_fail() {
        let block: Block = parse("{ fn add(a: i32, b: i32) -> i32 { a + b } add(1, true) }");
        assert_type_fail(&block);
    }

        #[test]
    fn test_prog_fn_calls() {
        let prog: Prog = parse("
            fn main() {
                let a = add(1, 2);
            }
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        ");
        assert_type(&prog, Type::Unit);
    }

    #[test]
    fn test_recursive_fn() {
        let prog: Prog = parse("
            fn main() {
                a(6);
            }
            fn a(x: i32) -> i32 {
                if x == 0 { 
                    1 
                } else { 
                 x * a(x - 2) 
                }
            }
        ");
        assert_type(&prog, Type::Unit);
    }


}