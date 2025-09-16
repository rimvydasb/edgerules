Validate: https://www.bottlecaps.de/rr/ui

```ebnf
Context ::= "{" ( Statement ( ";" Statement )* )? "}"

ComplexTypeDefinition ::= "{" ( Field ( ";" Field )* )? "}"

Field ::= Identifier ":" ( "<" (PrimitiveType | TypeAlias) ">" | ComplexTypeDefinition )

Statement ::=
      "type" TypeAlias ":" ComplexTypeDefinition
    | "type" TypeAlias ":" "<" (PrimitiveType | TypeAlias) ">"
    
    // typed variable placeholder 
    | Identifier ":" "<" (PrimitiveType | TypeAlias) ">"
    
    // variable value assignment
    | Identifier ":" ( Expression | Context )

PrimitiveType ::= "string" | "number" | "boolean" | "date" | "time" | "datetime" | "duration"

TypeAlias   ::= [A-Z][A-Za-z0-9_]*
Identifier  ::= [A-Za-z_][A-Za-z0-9_]*
```