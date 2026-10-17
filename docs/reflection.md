# Reflection on the course D7050E Compiler construction and formal languages

## Lab 1: Rust Fundamentals & Basic Expressions 
- **Basic understanding of the Rust tool-chain installation and use.**
  - Working with the compiler to learn about the relevant rules/mechanics of rust. Using cargo to install a crate and run code.
- **Use of editing environment for Rust development.**
  - Using vscode together with the suggested plugins, e.g., rust analyzer.
- **How to organize data in structs, enums, arrays, and vectors.**
  - Working with the struct **Pos** (main structure for the examples), enum **Result** (used to represent success or fail, which was interesting as there is no concept of null errors) and vectors in ex3.
- **Use of macros (e.g., `println!`, `vec!`)**
  - Macros have been covered in order to print to both stdout and stderr with `println!`, to create a vectors (`vec!`) which are essentialy a data structure similar to lists in other languages. These two are declarative macros, but we have also used the procedural macro `#[derive()]`.
- **The use of Traits, Debug, Iterator**
  - Debug in the pretty printing example where we saw that debug allowed the custom struct ouput to be formatted. Iterator was covered implicitly by the use of vectors, as they implement the trait, which was why the data structure could be iterated using the same syntax for arrays.
- **Use of generics (under the hood e.g., arrays and vectors are generic to element type.)**
  - The **Result** in ex3 is actually deinfed as **Result<T, E>** and a generic that lets you decide how to represent success/error values. Similarly, we used arrays and vectors made of tuples containing two integers, but the types could have been just about anything we wanted as long as we changed the signature for the functions where the types were later used.  
- **The `Clone` trait and the `Copy` trait in relation "move" and "copy" behavior.**
  - In ex4 we saw how passing p1 as an argument to add_pos ended up moving ownership of the value to the function in lead to an error where the main tried to print p1 after the func call because it was actually freed by its new owner (at the end of the scope of add_pos). Later it was shown that a solution could be to creating an explicit copy of p1 by letting Pos derive the Clone trait. Additionally, it was also shown that for small/cheap values such as ints or Pos the Copy trait could also be derived to obtain that behaviour implicitly.
- **References and borrow checking.**
  - The other part of ex4 showed how an alternative to moving ownership was to instead borrow values through references. We saw how to create/pass both immutable (read only) and mutable (read/write) references. The example showed how the borrow checker works, specifically how it prevents the creation of a mutable reference when an immutable one is active.

## Lab 2: Parsing to an AST

- **Regular expressions and their limitations.**
    - Got introduced to regular expressions in Rust using the `regex` crate. I 
    - Learned how to define patterns to match and extract information from strings. 
    - Learned about regex shortcomings for parsing grammar that requires nested structures or operator precedence.

- **Abstract Syntax Trees (ASTs) for expressions.**
    - Created simple AST expressions and implemented their evaluation. This helped med understand of how source code can be represented hierarchically and then interpreted. 
  
- **Document expression syntax with EBNF specification.**
    - Documenting the expression syntax using Extended Backus-Naur Form (EBNF) helped to define the grammar rules for expressions, which in theory should be crucial for unambiguous parsing and for explaining the language's syntax.

## Lab 3: The AST Interpreter (VM)


- **Reflect in your own words how you used the lab to obtain knowledge regarding above stated learning outcomes:Abstract Syntax Tree (AST) to represent RNR, Parsing from Rust to AST, Variable environment representing state, Natural interpretation.**
  - The lab gave me what I would call a good basic understanding for how compilers work. I learned the design of an AST by using Rust's enums and structs to represents the language. Working on the parser made me think about logic I usually take for given such as operator precedence, but also new stuff such as the differences between expressions and statements. Building the VM showed me how one can represent the environment, containing variables and functions, as a stack of scopes to implement the natural interpretation of the AST, which in turn made me understand how the code is being run.
- **Reflect in your own words how you used the lab to gain further knowledge in topics of your own interest.**
  - I enjoyed thinking about the architecture for the VM. I had initially started with two different environments (vars and funcs) but learned through a peer review that it's much better to use a single one to handle variables shadowing of functions. Working on the two-pass block evaluation introduced me to the concept of function hoisting where essentially the definition of a function can appear after its call.

## Lab 4: Type Checking

- **The role of semantic analysis in a compiler.**
    - I learned that semantic analysis (happens after parsing), here for type checking, is necessary to enforce semanticrules of the language that cannot be expressed by the grammar alone. Basically another set of checks before being able to generate code.

- **The basics of type environments and type checking.**
    - Similar to the vm, but here I had to implement a type environment which mapped identifiers to their types.

- **Reflection**: Through this lab I gained a better understanding of static analysis as an additiona layer that prevents runtime errors and improves program reliability.

## Lab 5: Code Generation to MIPS

- **The role of backend code generation in a compiler.**
    - Interesting part where I got to see the difference between the frontend (parsing and semantic analysis) and the backend  (code generation).

- **RISC-like instruction sets and memory regions.**
    - Refreshed concepts from a previous course while also

- **The basics of stack machines.**
    - Learnt how code generation can look like by first evaluating expressions and then pushing one multiple instructions on the stack, which in turn is used to perform operations and push the results back again. Overall, it was an interesting process to work with, which was mostly thanks to the mips crate otherwise it would have probably taken way more time.

- **Managing state for code generation (function frames).**
    - The newer and more time consuming part for me as I had to use a different approach for arguments (on the stack) compared to the previous course (using the mips arg registers). 

- **Automated code generation for AST constructs.**
    - I created a `CodeGenerator` struct that traverses the AST recursively. For each AST node type (`Expr::BinOp`, `Statement::While`, `FnDeclaration`, etc.), I implemented logic to use the respective sequence of MIPS instructions.

## Lab 6: The Command-Line Interface (CLI)

- **Rudimentary shell/terminal interaction.**
    - Created a simple cli for the compiler. I learned the difference between running a program by using `cargo run --` during development and executing the compiled binary directly from the `target/` directory. 

- **Command line parsing.**
    - I really liked working with the `clap` crate as it made the parsing of arguments nice and clean with the `Parser` macro. The work with the cli consisted of generating help messages and validating input and connecting all previous labs into a single program controlled by the flags provided by the user. 