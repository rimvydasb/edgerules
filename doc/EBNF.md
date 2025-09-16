Validate: https://www.bottlecaps.de/rr/ui

```ebnf
Context
  ::= "{" ( Statement )* ";"? "}"

Statement ::= 
    // inline complex type without nesting
      "type" TypeAlias ":" "<{" Identifier ":" (PrimitiveType | TypeAlias) ( "," Identifier ":" (PrimitiveType | TypeAlias) )* "}>"
      
    // simple primitive  
   |  "type" Identifier ":" "<" (PrimitiveType | TypeAlias) ">"
   
    // simple expression or context
   |  Identifier ":" ( Expression | Context )

PrimitiveType
  ::= "string" | "number" | "boolean" | "date" | "time" | "datetime" | "duration"


TypeAlias   ::= [A-Z][A-Za-z0-9_]*
Identifier  ::= [A-Za-z_][A-Za-z0-9_]*
```