Validate: https://www.bottlecaps.de/rr/ui

```ebnf
Program
  ::= "{" ( Stmt )* "}"

Stmt ::= 
    // inline complex type with no nesting
      "type" Identifier ":" "<{" Identifier ":" Type ( "," Identifier ":" Type )* "}>"
      
    // simple primitive
   |  "type" Identifier ":" "<" Type ">"
   
    // simple expression
   |  Identifier ":" Expr

PrimitiveType
  ::= "string" | "number" | "boolean"

Type
  ::= PrimitiveType | Identifier

Block
  ::= "{" ( Identifier ":" Expr ( ";" Identifier ":" Expr )* ";"? )? "}"

Expr
  ::= "if" Expr "then" Expr "else" Expr
   |  Postfix ( ( ">" | "<" | ">=" | "<=" | "==" | "!=" ) Postfix )*
   |  Postfix ( ( "+" | "-" ) Postfix )*
   |  Postfix

Postfix
  ::= Primary ( ( "." Identifier ) | "(" ( Expr ( "," Expr )* )? ")" )*

Primary
  ::= Number | String | "true" | "false" | Identifier | Block | "(" Expr ")"

/* ===== Lexical ===== */
Identifier  ::= [A-Za-z_][A-Za-z0-9_]*
Number      ::= ( "0" | [1-9] [0-9]* ) ( "." [0-9]+ )?
String      ::= '"' [^"]* '"' | "'" [^']* "'"
```