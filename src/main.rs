use clap::Parser;
use std::path::PathBuf;
use std::fs;

use rnr::parse;
use rnr::ast::{Prog, Type};
use rnr::vm::Val;
use rnr::common::Eval;
use rnr::code_generator::CodeGenerator;
use mips::vm::Mips;
use mips::rf::Reg;
use mips::instrs::Instrs;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to process.
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    /// Run the type checker.
    #[arg(short = 't', long)]
    type_check: bool,

    /// Run the code generator.
    #[arg(short = 'c', long, visible_aliases = ["code_gen"])]
    code_gen: bool,

    /// Run the AST interpreter.
    #[arg(short = 'v', long, visible_aliases = ["vm", "virtual_machine"])]
    interpret: bool,

    /// Run the generated code in the MIPS vm.
    #[arg(short, long, visible_alias = "r")]
    run: bool,

    /// Creates a file with the generated assembly.
    #[arg(short = 'm', long, value_name = "FILE", visible_aliases = ["asm"])]
    asm: Option<PathBuf>,

    /// Creates a file with the parsed AST.
    #[arg(short, long, value_name = "FILE")]
    ast: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let input_content = if let Some(file_path) = cli.input {
        println!("Input file: {:?}", file_path);
        match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file {:?}: {}.", file_path, e);
                std::process::exit(1);
            }
        }
    } else {
        println!("No input file specified. Defaulting to 'main.rnr'");
        let default_path = PathBuf::from("main.rnr");
        match fs::read_to_string(&default_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading default file {:?}: {}", default_path, e);
                std::process::exit(1);
            }
        }
    };

    let program_ast = match parse::try_parse::<Prog>(&input_content) {
        Ok(ast) => {
            println!("Parsed program into AST successfully.");
            ast
        },
        Err(e) => {
            eprintln!("Error parsing program: {}.", e);
            std::process::exit(1);
        }
    };

    // Type-checked before code-gen, interpreting or running.
    if cli.type_check || cli.code_gen || cli.run || cli.interpret {
        println!("Running type checker...");
        match <Prog as Eval<Type>>::eval(&program_ast) {
            Ok(_) => println!("Type checking passed successfully."),
            Err(e) => {
                eprintln!("Type checking failed: {}.", e);
                std::process::exit(1);
            }
        }
    }

    if cli.interpret {
        println!("Running AST VM...");
        match <Prog as Eval<Val>>::eval(&program_ast) {
            Ok(val) => println!("VM finished. Result: {:?}", val),
            Err(e) => {
                eprintln!("VM failed: {}.", e);
                std::process::exit(1);
            }
        }
    }

    let mut generated_instrs: Option<Instrs> = None;

    if cli.code_gen || cli.run {
        println!("Running code generator...");
        let mut code_generator = CodeGenerator::new();
        match code_generator.compile_prog(&program_ast) {
            Ok(_) => {
                println!("Code generation passed successfully.");
                generated_instrs = Some(code_generator.get_instrs());
            },
            Err(e) => {
                eprintln!("Code generation failed: {}.", e);
                std::process::exit(1);
            }
        }
    }

    if cli.run {
        if let Some(instrs) = generated_instrs.as_ref() {
            println!("Running program in MIPS VM...");
            let mut mips = Mips::new(instrs.clone());
            let result = mips.run();
            match result {
                Err(mips::error::Error::Halt) => {
                    let return_val = mips.rf.get(Reg::v0);
                    println!("MIPS VM Execution finished. Result: {}.", return_val);
                }
                Err(e) => {
                    eprintln!("MIPS VM execution failed: {:?}", e);
                    std::process::exit(1);
                }
                // Should not happen with halt.
                Ok(_) => {} 
            }
        } else {
            eprintln!("Error: -r flag used but code generation was not enabled or failed.");
            std::process::exit(1);
        }
    }

    if let Some(asm_path) = cli.asm {
        if let Some(instrs) = generated_instrs {
            println!("Assembly code will be saved to the file: {:?}", asm_path);
            match fs::write(&asm_path, instrs.to_string()) {
                Ok(_) => println!("Assembly successfully saved to {:?}.", asm_path),
                Err(e) => eprintln!("Error writing assembly to file {:?}: {}.", asm_path, e),
            }
        } else {
            eprintln!("Error: --asm flag used but code generation was not enabled or failed.");
            std::process::exit(1);
        }
    }

    if let Some(ast_path) = cli.ast {
        println!("AST will be saved to the file: {:?}", ast_path);
        match fs::write(&ast_path, format!("{:#?}", program_ast)) {
            Ok(_) => println!("AST successfully saved to {:?}", ast_path),
            Err(e) => eprintln!("Error writing AST to file {:?}: {}.", ast_path, e),
        }
    }
}