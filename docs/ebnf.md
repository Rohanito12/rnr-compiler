# EBNF Grammar

```ebnf
digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
letter = "A" | "B" | "C" | "D" | "E" | "F" | "G"
       | "H" | "I" | "J" | "K" | "L" | "M" | "N"
       | "O" | "P" | "Q" | "R" | "S" | "T" | "U"
       | "V" | "W" | "X" | "Y" | "Z" | "a" | "b"
       | "c" | "d" | "e" | "f" | "g" | "h" | "i"
       | "j" | "k" | "l" | "m" | "n" | "o" | "p"
       | "q" | "r" | "s" | "t" | "u" | "v" | "w"
       | "x" | "y" | "z" | "_" ;
integer = digit, { digit } ;
unit = "()" ;
bool = "true" | "false" ;
string = '"', { any char that isn't a double quote }, '"';
literal = integer | bool | string | unit ;
( * syn handles the no digit at the beginning of an ident. * )
ident = letter, { letter | digit } ;

(* Types *)
type = "i32" | "bool" | "String" | unit ;

(* Expression rules. *)
(* Priorities from lowest to highest. *)
expr = or_expr ;
or_expr = and_expr, { "||", and_expr } ;
(* Logical AND *)
and_expr = cmp_expr, { "&&", cmp_expr}
(* Comparisons *)
cmp_expr = add_expr, { ("==" | ">" | "<"), add_expr } ; 
(* Addition / Subtraction *)
add_expr = mul_expr, { ("+" | "-"), mul_expr } ;
(* Multiplication / Division *)
mul_expr = unary_expr, { ("*" | "/"), unary_expr } ;
(* Unary operators *)
unary_expr = { "-" | "!" }, factor ;

(* A factor now represents expressions, calls, blocks, and if else expressions *)
factor = literal | "(", expr, ")" | ident | call | block | if_expr ;

call = ident, "(", arguments, ")" ;
(* Optional list of args *)
arguments = [ expr, {",", expr} ] ;

if _expr = "if", expr, block, ["else", (block | if_expr) ] ;

block = "{", { statement }, "}" ;

statement = let_stmt | assign_stmt | while | fn_decl | expr_stmt ;

let_stmt = "let", [ "mut" ], ident, [ ":", type ], [ "=", expr], ";" ;

assign_stmt = expr, "=", expr, ";" ;

while = "while", expr, block ;

( * May or may not have semi depending on position in a block *)
expr_stmt = expr, [ ";" ] ;

fn_ decl = "fn", ident, "(" parameters, ")", [ "->", type ],  block ;

parameters = [ parameter, { ",", parameter} ] ;
parameter = [ "mut" ], ident, ":", type ;

prog = { fn_decl } ;
```