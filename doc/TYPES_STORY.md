# Types Story

EdgeRules has a standard type definition, as it is common in other software tools
and **typed placeholders**.

## EBNF Specification

Validate: https://www.bottlecaps.de/rr/ui

```ebnf
Context ::= "{" ( Statement ( ";" Statement )* )? "}"

ComplexTypeDefinition ::= "{" ( Field ( ";" Field )* )? "}"

Field ::= Identifier ":" ( "<" ComplexType ">" | ComplexTypeDefinition | CastExpression )

Statement ::=
      "type" TypeAlias ":" ComplexTypeDefinition
    | "type" TypeAlias ":" "<" ComplexType ">"
    | "func" Identifier "(" ( Parameter ( "," Parameter )* )? ")" ":" ( CastExpression | Context )
    // typed variable placeholder 
    | Identifier ":" "<" ComplexType ">"
    
    // variable value assignment
    | Identifier ":" ( CastExpression | Context )

// --- Casting layer: only after an expression ---
CastExpression ::= Expression ( "as" ComplexType )?

// --- Types & Ids ---
PrimitiveType ::= "string" | "number" | "boolean" | "date" | "time" | "datetime" | "duration"
TypeAlias     ::= [A-Z][A-Za-z0-9_]*
ComplexType   ::= (PrimitiveType | TypeAlias) ("[]")*

Identifier  ::= [A-Za-z_][A-Za-z0-9_]*
Parameter   ::= Identifier ( ":" (PrimitiveType | TypeAlias) ("[]")* )?

// `Expression` is your existing (or to-be-defined) expression grammar.
```

## Example

```edgerules
{
    // Business Object Model for Decision Service:
    type Customer: {name: <string>, birthdate: <date>, income: <number>}
    type Applicant: {customer: <Customer>, requestedAmount: <number>, termInMonths: <number>}
    type LoanOffer: {eligible: <boolean>, amount: <number>, termInMonths: <number>, monthlyPayment: <number>}

    // context data that must be passed to the decision service:
    executionDatetime: <datetime>

    // Decision Service:
    func calculateLoanOffer(applicant: Applicant): {
        eligible: if executionDatetime - applicant.customer.birthdate >= duration('P18Y') then true else false;
        interestRate: if applicant.customer.income > 5000 then 0.05 else 0.1;
        monthlyPayment: (applicant.requestedAmount * (1 + interestRate)) / applicant.termInMonths;
        result: {
            eligible: eligible;
            amount: applicant.requestedAmount;
            termInMonths: applicant.termInMonths;
            monthlyPayment: monthlyPayment
        }
    }

    // Example execution:
    applicant1: {
        customer: {name: "Alice"; birthdate: date('2001-01-01'); income: 6000};
        requestedAmount: 20000;
        termInMonths: 24
    }

    loanOffer1: calculateLoanOffer(applicant1).result as LoanOffer
}
```

## Standard Type Definition

`type Customer: {name: <string>, birthdate: <date>, income: <number>}` 
represents a standard type definition. It defines a type alias `Customer` that can be used in the rest of the model.
In `ContextObject` it will be in `defined_types` list (see how `metaphors` are implemented and stored).
`TypeDefinition` will be implemented similarly to `FunctionDefinition`.
Also, see `DefinitionEnum::Metaphor(BuiltinMetaphor)` - similar approach will be used for user-defined types.

> Everything what is under `type...` statement it is just a type definition. Within a type definition it is
> not possible to have any functions definitions or typed placeholders. Only the nested type definitions are allowed.

## Typed Placeholders

`executionDatetime: <datetime>`
represents a typed placeholder. In `ContextObject` it will be in `expressions` list as `ExpressionEnum::TypeDefinition`.
`StaticLink` trait that could immediately return defined type, because it is known.
`EvaluatableExpression` trait implemented eval will look for the value in the context and if not found will return `Missing` special value.

## Typed Arguments

`func calculateLoanOffer(applicant: Applicant): {...}`
represents a function definition with typed argument that is optional.

## Evaluation

It is a completely valid when the user defines a mixture of placeholders and expressions:

```edgerules
{
    type Vector: {x: <string>, y: <number>}                          // type alias and assigned definition
    vectorStore: {id: <number>, name: "STORE", vectors: <Vector[]>}  // variable with type placeholder with Vector type reference
    identification: <number>                                         // variable with simple type placeholder
    relationsList: <number[]>                                        // variable with simple type placeholder with array of numbers
}
```

Previous model can be evaluated as a **stand-alone** model without any context:

```edgerules
{
    vectorStore: {id: Missing, name: "STORE", vectors: Missing}
    identification: Missing
    relationsList: Missing
}
```

Evaluated model **with** the context example:

```edgerules
// context: (this can also represent the request to the decision service)
{
    vectorStore: {vectors: [{x: 1; y: 2}]}
    relationsList: [1,2,3,4]
}
```
```edgerules
// evaluated model: (this can also represent the response from the decision service)
myModel: {
    vectorStore: {id: Missing, name: "STORE", vectors: [{x: 1; y: 2}]}
    identification: Missing
    relationsList: [1,2,3,4]
}
```

## Casting

`loanOffer1: calculateLoanOffer(applicant1).result as LoanOffer`
represents a casting operation. The value on the left side of `as` operator is cast to the type on the right side of `as` operator.
In AST it will be within as `ExpressionEnum::Operator` node as `CastOperator`.
Traits `StaticLink` and `TypedValue` could immediately return defined type, because it is known.
`EvaluatableExpression` trait implemented eval will create an empty instance of the target type and will
deeply copy the value to the target type with validation applied according to target type definition.

### Example

```edgerules
{
    type LoanOffer: {eligible: <boolean>, amount: <number>, termInMonths: <number>, monthlyPayment: <number>}
    loanOffer1: {eligible: false} as LoanOffer    
}
```

will be evaluated to

```edgerules
{
    loanOffer1: {eligible: false, amount: Missing, termInMonths: Missing, monthlyPayment: Missing} as LoanOffer    
}
```

## Limitations

- Function return type cannot be defined by the user right now. Function type definition will be added later in the grammar.
- As of now, expressions are not allowed within type definitions. It might be added later if needed.

## Clarifications

- Function definitions will start with `func` keyword.