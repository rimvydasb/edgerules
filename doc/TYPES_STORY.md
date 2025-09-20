# Types Story

EdgeRules has a standard type definition, as it is common in other software tools
and **typed placeholders**.

## EBNF Specification

Validate: https://www.bottlecaps.de/rr/ui

```ebnf
Context ::= "{" ( Statement ( ";" Statement )* )? "}"

ComplexTypeDefinition ::= "{" ( Field ( ";" Field )* )? "}"

Field ::= Identifier ":" ( "<" (PrimitiveType | TypeAlias) ">" | ComplexTypeDefinition | Expression )

Statement ::=
      "type" TypeAlias ":" ComplexTypeDefinition
    | "type" TypeAlias ":" "<" (PrimitiveType | TypeAlias) ">"
    | "func" Identifier "(" ( Parameter ( "," Parameter )* )? ")" ":" ( Expression | Context )
    
    // typed variable placeholder 
    | Identifier ":" "<" (PrimitiveType | TypeAlias) ">"
    
    // variable value assignment
    | Identifier ":" ( Expression | Context )

PrimitiveType ::= "string" | "number" | "boolean" | "date" | "time" | "datetime" | "duration"

TypeAlias   ::= [A-Z][A-Za-z0-9_]*
Identifier  ::= [A-Za-z_][A-Za-z0-9_]*
Parameter   ::= Identifier ( ":" (PrimitiveType | TypeAlias) )?
```

## Example

```edgerules
{
    // Business Object Model for Decision Service:
    type Customer: {name: <string>, birthdate: <date>, income: <number>}
    type Applicant: {customer: <Customer>, requestedAmount: <number>, termInMonths: <number>}
    type LoanOffer: {eligibile: <boolean>, amount: <number>, termInMonths: <number>, monthlyPayment: <number>}

    // context data that must be passed to the decision service:
    executionDatetime: <datetime>

    // Decision Service:
    func calculateLoanOffer(applicant: Applicant): {
        eligibile: if executionDatetime - applicant.customer.birthdate >= 18 then true else false;
        interestRate: if applicant.customer.income > 5000 then 0.05 else 0.1;
        monthlyPayment: (applicant.requestedAmount * (1 + interestRate)) / applicant.termInMonths;
        result: {
            eligibile: eligibile;
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
In AST it will be represented as `ExpressionEnum::TypeDefinition` node.

## Typed Placeholders

`executionDatetime: <datetime>`
represents a typed placeholder. 
In AST it will be in the part of `ExpressionEnum::TypedObjectField` and will have `StaticLink` trait that could
immediately return defined type, because it is known.
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
    standardObject: {x: "header"; y: 123}                            // variable that has standard expression assigned
}
```

Previous model can be evaluated as a **stand-alone** model without any context:

```edgerules
{
    vectorStore: {id: Missing, name: "STORE", vectors: Missing}
    identification: Missing
    relationsList: Missing
    standardObject: {x: "header"; y: 123}
}
```

Evaluated model **with** the context example:

```edgerules
// context: (this can also represent the request to the decision service)
{
    vectorStore: {vectors: {x: 1, y: 2}}
    relationsList: [1,2,3,4]
}
```
```edgerules
// evaluated model: (this can also represent the response from the decision service)
myModel: {
    vectorStore: {id: Missing, name: "STORE", vectors: {x: 1, y: 2}}
    identification: Missing
    relationsList: [1,2,3,4]
    standardObject: {x: "header"; y: 123}
    vectorInstance: {x: 5; y: Missing}
}
```