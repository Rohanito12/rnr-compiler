use std::collections::HashMap;
use std::hash::Hash;

use crate::ast::{Expr, Literal, BinOp, Statement, Block, UnOp, FnDeclaration, Prog};
use crate::error::Error;
use mips::instrs::Instrs;
use mips::rf::Reg;
use mips::asm::*; 


pub struct CodeGenerator{
    // List of MIPS instructions that will used as a stack.
    instructions: Instrs,
    // Maps variable name to offset from $fp, e.g., "a" to -4.
    env: Vec<HashMap<String, i16>>, 
    // Back to the original designnnnnnnn.
    fn_env: Vec<HashMap<String, String>>,
    // Tracks the current depth of local variables.
    next_offset: i16, 
    labels: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator { 
        instructions: Instrs::new(),
        env: vec![HashMap::new()], 
        fn_env: vec![HashMap::new()],
        next_offset: 0, 
        labels: 0,
        }
    }    

    // Push a register onto the stack.
    fn push(&mut self, reg: Reg) {
        // Decrease the stack pointer ($sp) by 4.
        self.instructions.push(addiu(Reg::sp, Reg::sp, -4));
        // Store register value at 0($sp).
        self.instructions.push(sw(reg, 0, Reg::sp));
    }

    // Pop a value from the stack into a register.
    fn pop(&mut self, reg: Reg) {
        //Load value from 0($sp) into register.
        self.instructions.push(lw(reg, 0, Reg::sp));
        //Increment stack pointer ($sp) by 4.
        self.instructions.push(addiu(Reg::sp, Reg::sp, 4));
    }

    // For testing.
    pub fn get_instrs(&self) -> Instrs {
        self.instructions.clone()
    }

    fn push_scope(&mut self) {
        self.env.push(HashMap::new());
        self.fn_env.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        // For now just popping the map for deallocation.
        // Might need to work with offset later to clear up stack holes.
        self.env.pop();
        self.fn_env.pop();
    }

    fn get_var_offset(&self, id: &str) -> Result<i16, Error> {
        for scope in self.env.iter().rev() {
            if let Some(offset) = scope.get(id) {
                return Ok(*offset);
            }
        }
        Err(format!("Variable not found: {}.", id))
    }

    fn create_label(&mut self) -> String {
        let label = format!("L{}", self.labels);
        self.labels += 1;
        label
    }

    // For nested functions.
    fn create_unique_label(&mut self, base: &str) -> String {
        let label = format!("{}_{}", base, self.labels);
        self.labels += 1;
        label
    }

    fn get_fn_label(&self, id: &str) -> String {
        for scope in self.fn_env.iter().rev() {
            if let Some(label) = scope.get(id) {
                return label.clone();
            }
        }
        // If not found in scopes then it should be safe to
        //  assume that it's a global function with the same name
        id.to_string()
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<(), Error> {
        match expr {
            Expr::Lit(lit) => match lit {
                Literal::Int(i) => {
                    // Load immediate into temp register $t0. Max 16 bits.
                    self.instructions.push(addi(Reg::t0, Reg::zero, *i as i16));
                    self.push(Reg::t0);
                    Ok(())
                }
                Literal::Bool(b) => {
                    // Gonna treat 1 as true and 0 as false.
                    let val = if *b { 1 } else { 0 };
                    self.instructions.push(addi(Reg::t0, Reg::zero, val));
                    self.push(Reg::t0);
                    Ok(())
                }
                Literal::Unit => {
                    self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                    self.push(Reg::t0);
                    Ok(())
                }
                _ => Err("Unsupported literal.".to_string()),
            },
            Expr::Par(inner) => self.compile_expr(inner),
            Expr::Ident(id) => {
                let offset = self.get_var_offset(id)?;
                
                // Load from mem: lw $t0, offset($fp)
                self.instructions.push(lw(Reg::t0, offset, Reg::fp));
                
                self.push(Reg::t0);
                Ok(())
            },
            Expr::BinOp(op, left, right) => {
                match op {
                    BinOp::And => {
                        // To implement short-circuiting use label to jump if left is false. 
                        let end_label = self.create_label();

                        // Compile left first.
                        self.compile_expr(left)?;
                        self.pop(Reg::t0);

                        // If the left operand is false (0) stop by jumping to end. 
                        self.instructions.push(beq_label(Reg::t0, Reg::zero, &end_label));

                        self.compile_expr(right)?;
                        self.pop(Reg::t0);

                        // End label.
                        self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&end_label));
                        // Either 0 from short-circuit or result of right.
                        self.push(Reg::t0); 
                        Ok(())
                    },
                    BinOp::Or => {
                        let true_label = self.create_label();
                        let end_label = self.create_label();
                        
                        self.compile_expr(left)?;
                        self.pop(Reg::t0);
                        
                        // If left is true (!= 0), jump to the true label.
                        self.instructions.push(bne_label(Reg::t0, Reg::zero, &true_label));
                        
                        // Otherwise, the result is the value of the right operand.
                        self.compile_expr(right)?;
                        self.pop(Reg::t0); 
                        
                        // Jump over the true label.
                        self.instructions.push(b_label(&end_label));
                        
                        // True label.
                        self.instructions.push(addi(Reg::t0, Reg::zero, 1).label(&true_label));
                        // End label.
                        self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&end_label));
                        
                        self.push(Reg::t0);
                        Ok(())
                    },
                    _ => {
                        // Compile left and right.
                        self.compile_expr(left)?;
                        // Top of the stack here.
                        self.compile_expr(right)?;

                        // Pop operands.
                        self.pop(Reg::t1); // Right operand -> $t1
                        self.pop(Reg::t0); // Left operand  -> $t0

                        match op {
                            BinOp::Add => {
                                self.instructions.push(add(Reg::t0, Reg::t0, Reg::t1));
                                // Push result back to stack.
                                self.push(Reg::t0);
                            }
                            BinOp::Sub => {
                                self.instructions.push(sub(Reg::t0, Reg::t0, Reg::t1));
                                // Push result back to stack.
                                self.push(Reg::t0);
                            }
                            // The mips crate only supports Add/Sub.
                            BinOp::Mul | BinOp::Div => return Err("Unsupported operation.".to_string()),
                            BinOp::Lt => {
                                // $t0 = ($t0 < $t1) ? 1 : 0
                                self.instructions.push(slt(Reg::t0, Reg::t0, Reg::t1));
                                // Push result back to stack.
                                self.push(Reg::t0);
                            }
                            BinOp::Gt => {
                                // $t0 = ($t1 < $t0) ? 1 : 0
                                self.instructions.push(slt(Reg::t0, Reg::t1, Reg::t0));        
                                // Push result back to stack.
                                self.push(Reg::t0);
                            }
                            BinOp::Eq => {
                                // xor t0, t0, t1  -> t0 = 0 if they are equal.
                                self.instructions.push(xor(Reg::t0, Reg::t0, Reg::t1));
                                // sltiu t0, t0, 1 -> t0 = 1 if t0 < 1, i.e., if t0 was 0.
                                self.instructions.push(sltiu(Reg::t0, Reg::t0, 1));
                                // Push result back to stack.
                                self.push(Reg::t0);
                            }
                            _ => todo!("And + Or implemented above for short-circuiting.")
                        }
                        Ok(())
                    }
                }
            },  
            Expr::Call(id, args) => {
                // Compile args.
                for arg in &args.0 {
                    self.compile_expr(arg)?;
                }

                let label = self.get_fn_label(id);
                // Call the function using branch and link (jump to label and save pc in $ra). 
                self.instructions.push(bal_label(&label));

                // Clean up the args after execution by moving the sp.
                let args_size = 4 * args.0.len() as i16;
                if args_size > 0 {
                    self.instructions.push(addiu(Reg::sp, Reg::sp, args_size));
                }

                // Return value.
                self.push(Reg::v0);
                
                Ok(())
            },
            Expr::IfThenElse(cond, then_block, else_block) => {
                let else_label = self.create_label();
                let end_label = self.create_label();

                self.compile_expr(cond)?;
                self.pop(Reg::t0);

                // Branch to else if false: beq $t0, $zero, else_label.
                self.instructions.push(beq_label(Reg::t0, Reg::zero, &else_label));

                // Rm: blocks with a returning value will push it.
                self.compile_block(then_block)?;
                
                // Jump to end.
                self.instructions.push(b_label(&end_label));

                // Jump here if condition is false. Need noop to attach the label
                self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&else_label));

                if let Some(block) = else_block {
                    self.compile_block(block)?;
                } else {
                    // If no else block return Unit.
                    self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                    self.push(Reg::t0);
                }

                self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&end_label));
                Ok(())
            },
            Expr::Block(block) => self.compile_block(block),
            Expr::UnOp(op, expr) => {
                self.compile_expr(expr)?;
                self.pop(Reg::t0);

                match op {
                    UnOp::Neg => {
                        // 0 - t0
                        self.instructions.push(sub(Reg::t0, Reg::zero, Reg::t0));
                    }
                    UnOp::Bang => { 
                        // If t0 is 0, 0 < 1 is true (1).
                        // If t0 is 1, 1 < 1 is false (0).
                        self.instructions.push(sltiu(Reg::t0, Reg::t0, 1));
                    }
                }
                self.push(Reg::t0);
                Ok(())
            },
        }
    }

    // Every call to compile stmt is expected to leave exactly one value on the stack
    // so that compile_block can safely pop.
    pub fn compile_stmt(&mut self, stmt: &Statement) -> Result<(), Error> {
        match stmt {
            Statement::Let(_mut, id, _ty, expr) => {
                if let Some(e) = expr {
                    // Calculate the value and push it onto the stack.
                    self.compile_expr(e)?;
                } else {
                    // For uninitialized setting 0 to take the space.
                    self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                    self.push(Reg::t0);
                }

                // The value is now at the top of the stack so update the offset.
                self.next_offset -= 4;

                // Save var relative to $fp.
                self.env.last_mut().unwrap().insert(id.clone(), self.next_offset);
                
                Ok(())
            }
            Statement::Expr(expr) => {
                // I think expr that end with the ; should be popped...wish I had a gc. 
                // Checking later how to handle.
                self.compile_expr(expr)?;
                Ok(())
            },
            Statement::Assign(lhs, rhs) => {
                self.compile_expr(rhs)?;
                
                // Get res from the top of the stack.
                self.pop(Reg::t0);

                match lhs {
                    Expr::Ident(id) => {
                        let offset = self.get_var_offset(id)?;
                        // Update mem: sw $t0, offset($fp).
                        self.instructions.push(sw(Reg::t0, offset, Reg::fp));

                        // Need to push Unit so compile_block can pop it.
                        self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                        self.push(Reg::t0);
                        Ok(())
                    }
                    _ => Err("Left expression of assignment must be an identifier.".to_string()),
                }
            },
            Statement::While(cond, block) => {
                let start_label = self.create_label();
                let end_label = self.create_label();

                // Start label.
                self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&start_label));

                self.compile_expr(cond)?;
                self.pop(Reg::t0);

                // Exit if cond is false.
                self.instructions.push(beq_label(Reg::t0, Reg::zero, &end_label));

                self.compile_block(block)?;
                
                // Cleanup body return value.
                self.pop(Reg::zero);

                // Jump back to start.
                self.instructions.push(b_label(&start_label));

                // End label.
                self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&end_label));
                
                self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                self.push(Reg::t0);
                Ok(())
            },
            Statement::Fn(decl) => {
                // Create new unique label for this nested function.
                let unique_label = self.create_unique_label(&decl.id);
                // Register it in the current scope so it shadows previous defs.
                self.fn_env.last_mut().unwrap().insert(decl.id.clone(), unique_label.clone());
                // Jump over body to prevent execution.
                let skip_label = self.create_label();
                self.instructions.push(b_label(&skip_label));
        
                self.compile_fn_internal(decl, &unique_label)?;

                // Label to skip to.
                self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(&skip_label));

                Ok(())
            }
        }
    }

    pub fn compile_block(&mut self, block: &Block) -> Result<(), Error> {
        self.push_scope();
        
        for (i, stmt) in block.statements.iter().enumerate() {
            self.compile_stmt(stmt)?;

            // Need this because based on the design of compile_stmt, i.e., each call pushes exactly 1 value.
            if let Statement::Let(_, _, _, _) | Statement::Fn(_) = stmt {
                self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                self.push(Reg::t0);
            }

            let is_last = i == block.statements.len() - 1;
            
            if is_last {
                 if block.semi {
                     // Last stmt has semi so pop the result and push Unit.
                     self.pop(Reg::zero);
                     self.instructions.push(addi(Reg::t0, Reg::zero, 0));
                     self.push(Reg::t0);
                 }
                 // Otherwise leave it as return value.
            } else {
                // Cleanup res from stack.
                self.pop(Reg::zero);
            }
        }
        
        // Edge case: Empty block {} returns Unit
        if block.statements.is_empty() {
             self.instructions.push(addi(Reg::t0, Reg::zero, 0));
             self.push(Reg::t0);
        }

        self.pop_scope();
        Ok(())
    }

    fn compile_fn_internal(&mut self, func: &FnDeclaration, label: &str) -> Result<(), Error>{
        // Label.
        self.instructions.push(add(Reg::zero, Reg::zero, Reg::zero).label(label));

        // Save return address.
        self.push(Reg::ra);
        // Save old $fp.
        self.push(Reg::fp);
        // Move $sp to $fp for a new stack frame.
        self.instructions.push(add(Reg::fp, Reg::zero, Reg::sp));

        // New scope for func args and body.
        self.push_scope(); 

        // Last added arg is at 8($fp) above the return address.
        let mut arg_offset = 8;
        for param in func.parameters.0.iter().rev() {
            self.env.last_mut().unwrap().insert(param.id.clone(), arg_offset);
            arg_offset += 4;
        }

        self.compile_block(&func.body)?;
        
        // If the block ended with an expression, it's on the stack. 
        // Get return value of the block.
        self.pop(Reg::t0); 
        
        // Restore stack pointer (sp = fp).
        self.instructions.push(add(Reg::sp, Reg::zero, Reg::fp));
        // Pop old frame pointer.
        self.pop(Reg::fp);
        // Pop return address.
        self.pop(Reg::ra);
        
        // Push result back.
        self.instructions.push(add(Reg::v0, Reg::zero, Reg::t0));

        // Jump back to return address.
        self.instructions.push(jr(Reg::ra));

        // Pop args scope.
        self.pop_scope();
        
        Ok(())

    }

    pub fn compile_fn(&mut self, func: &FnDeclaration) -> Result<(), Error> {
        self.compile_fn_internal(func, &func.id)
    }

    pub fn compile_prog(&mut self, prog: &Prog) -> Result<(), Error> {
        // Call call main at the start.
        self.instructions.push(bal_label("main"));
        
        // Once main returns stop the program.
        self.instructions.push(halt());

        // Compile all other functions.
        for func in &prog.0 {
            self.compile_fn(func)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mips::vm::Mips;
    use crate::parse::parse; 

    #[test]
    fn test_codegen_arithmetic() {
        let src = "(1 + 2) - 3";
        
        // Parse directly into an Expr using type inference
        let expr: Expr = parse(src); 

        let mut cg = CodeGenerator::new();
        cg.compile_expr(&expr).unwrap();
        
        cg.instructions.push(halt()); // To stop the program.

        let instructions = cg.get_instrs(); 
        println!("Generated assembly:\n{}", instructions);
        // Test instructions on a mips vm.
        let mut mips = Mips::new(instructions);
        let result = mips.run();
        // Vm ends correctly.
        assert_eq!(result, Err(mips::error::Error::Halt));

        // The final address with just the result pushed into the stakc
        // is $sp - 4 = 0x8000_0000 - 4 = 0x7FFF_FFFC.
        let result_addr = 0x7FFF_FFFC;
        let result_val = mips.dm.read_word(result_addr).unwrap();
        
        assert_eq!(result_val, 0);
    }

    #[test]
    fn test_codegen_let_assign() {
        // Stacks should be [..., 10, 5, 5] with last 5 being the result.
        let src = "{ let a: i32 = 10; let b: i32 = 5; a - b }";
        let block: Block = parse(src);

        let mut cg = CodeGenerator::new();
        
        // Set up frame pointer: move $fp, $sp.
        cg.instructions.push(add(Reg::fp, Reg::zero, Reg::sp));

        cg.compile_block(&block).unwrap();
        cg.instructions.push(halt());

        let instrs = cg.get_instrs();
        println!("Generated assembly:\n{}", instrs);

        let mut mips = Mips::new(instrs);
        let result = mips.run();
        assert_eq!(result, Err(mips::error::Error::Halt));
        
        // Result should be at the top of the stack.
        let sp = mips.rf.get(Reg::sp);
        let result = mips.dm.read_word(sp).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_if_expr() {
        let src = "if true { 10 } else { 20 }";
        let expr: Expr = parse(src);

        let mut cg = CodeGenerator::new();
        cg.compile_expr(&expr).unwrap();
        cg.instructions.push(halt());

        let mut mips = Mips::new(cg.get_instrs());
        assert_eq!(mips.run(), Err(mips::error::Error::Halt));

        let result = mips.dm.read_word(0x7FFF_FFFC).unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn test_while_loop() {
        let src = "{ let a: i32 = 3; while a > 0 { a = a - 1 }; a }";
        let block: Block = parse(src);

        let mut cg = CodeGenerator::new();
        cg.instructions.push(add(Reg::fp, Reg::zero, Reg::sp)); // Init FP
        cg.compile_block(&block).unwrap();
        cg.instructions.push(halt());

        let mut mips = Mips::new(cg.get_instrs());
        mips.run();

        let result = mips.dm.read_word(mips.rf.get(Reg::sp)).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_main_return_unit() {
        let src = "fn main() { let a = 5; }";
        let prog: Prog = parse(src);

        let mut cg = CodeGenerator::new();
        cg.compile_prog(&prog).unwrap();
        
        let mut mips = Mips::new(cg.get_instrs());
        let _ = mips.run();
        
        assert_eq!(mips.rf.get(Reg::v0), 0);
    }

    #[test]
    fn test_gcd_easy() {
        let src = "
            fn main() {
                let r = gcd(12, 18);
                r
            }

            fn gcd(a: i32, b: i32) -> i32 {
                if a == b {
                    a
                } else {
                    if a > b {
                        gcd(a - b, b)
                    } else {
                        gcd(a, b - a)
                    }
                }
            }
        ";

        let prog: Prog = parse(src);

        let mut cg = CodeGenerator::new();
        cg.compile_prog(&prog).unwrap();
        
        let mut mips = Mips::new(cg.get_instrs());
        let _ = mips.run();
        
        // GCD of 12 and 18 is 6.
        assert_eq!(mips.rf.get(Reg::v0), 6);
    }
}