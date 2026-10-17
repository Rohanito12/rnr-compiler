Changelog for RNR

2025-12-15

Wrote readme for the presentation. 
Started writing reflection.

2025-12-14

Cli done should be done.
Added simple rnr files for type checking, recursion and basic addition. Discovered a bug where ``if/else`` statements expect a semicolon at the end all of tht times. 
Added also equality for strings in the type checker.


2025-12-12

Started looking into clap. Docs + a video showed how to create a struct made of args for simple parsing.
Implemented file reading and program parsing into ast.


2025-12-08 (Summary of the work for lab 5)

Implemented a very simple (and bad) code generator for MIPS assembly. The process even here required to traverse the nodes in the AST to generate the respective sequences of mips instructions. Additionally, I had to refresh my memory on stack management for the handling of de/allocation of memory on the stack for function arguments, local vars and return addresses similar to how it was done in a previous course just with a more general case as this time the respective register for arguments were not used. The stack was used instead as it's not limited to just 4 arguments. The crate mips waas used to create the mips instructions.


2025-12-01 (Summary of the work for lab 4)

Developed a basic type checker, i.e., worked with defining the allowed types for rnr and defined valid operations for each one of them, e.g., add/sub only for i32. The process was similar to that the vm in that the AST had to be traversed to check that all expressions and statements respect the type rules.


2025-11-24

Discovered bug for vm tests test_while and test_check_if_then_else_shadowing where variables are not set to mutable so the intended result can't be verified.
Created new enum to handle both user functions and built in ones like macros.
Imported intrinsics in lib.rs to use functions like println!.
VM done.
EBNF updated.


2025-11-23

Continued working on vm.


2025-11-21/22

Implemented Parse for all types in parse.rs.
Display work done.


2025-11-20

Implemented expression parsing logic for binary operations (e.g., +, *,  && and comparison operators) with precedence and associativity in src/parse.rs.
Addressed parsing of the unit literal () in src/parse.rs to prevent unexpected panics (literal unit test was the cause).
Initialized the Virtual Machine (VM) in src/vm.rs with a stack to store scope variables.


2024-11-19

Complete restructure to reduce re-implementation work. See README.md for up to date information.