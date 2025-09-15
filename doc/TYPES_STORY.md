# Types as Placeholders

EdgeRules does not have standard type definition as it is common in other software tools.
EdgeRules use typed placeholders instead.

Currently, EdgeRules supports a fixed set of core primitive types and can print structure types such that:

```edgerules
{a : 88; b : 99; c : {x : 'Hello'; y : a + b; userFunction() : {}}}
```

`get_type` method will return inline type, because no other types are defined. 
The method extracts already linked type definitions and prints them in the following format:

```edgerules
<a: number, b: number, c: <x: string, y: number>>
```

Below is the example of standard expression definition:

```edgerules
{
    myObject: {a: 88; b: 99; c: {x: 'Hello'; y: a + b; userFunction() : {}}}
    myPrimitive: 123
}
```

The `myObject` (same as `myPrimitive`) gets expression assigned that does following things:
1. Defines a variable with a given name `myObject` on the left side
2. Links types if they're not linked
3. Evaluates the expression on the right side and creates an instance of the result
4. Assigns the result to the variable `myObject`

This story goal is to allow user to define a typed placeholder that immediately assigns a type to a give variable,
but there will be no expression to be assigned. Use will be able to define complex typed placeholders as well as simple:

```edgerules
{
    myObject: <a: number, b: number, c: <x: string, y: number>[]>
    myPrimitive: <number>
}
```

also:

```edgerules
myModel: {
    type vector: <x: string, y: number>
    vectorStore: <id: number, name: string, c: vector[]>
    identification: <number>
    relationsList: <number[]>
    standardObject: {x: "header"; y: 123}
}
```

evaluated model **without** the context:

```edgerules
myModel: {
    vectorStore: Missing
    identification: Missing
    relationsList: Missing
    standardObject: {x: "header"; y: 123}
}
```

evaluated model **with** the context:

```edgerules
// context: (this can also represent the request to the decision service)
{
    vectorStore: {x: 1, y: 2}
    relationsList: [1,2,3,4]
    standardObject: {x: "header"; y: 123}
}
```
```edgerules
// evaluated model: (this can also represent the response from the decision service)
myModel: {
    vectorStore: {x: 1, y: 2}
    identification: Missing
    relationsList: [1,2,3,4]
    standardObject: {x: "header"; y: 123}
}
```

## Clarifications

- `type $TYPENAME` will define a pure type, but not variable, for example `type vector: <x: string, y: number>`.
- `$VARNAME: <$TYPENAME>` will define a variable and will assign a type placeholder. If no real value will be 
assigned from the context, then variable will have `Missing` special value during the evaluation (see Special Values story)
- Type can also be defined in JSON style structure as well as inline:

```edgerules
{
    // Customer type:
    type CustomerA: <name: string, age: number, income: number>
    
    // Sam as CustomerA, exact type:
    type CustomerB: {
        name: <string>;
        age: <number>;
        income: <number>;
    }
}
```

## Decision Service Example

```edgerules
{
    // Business Object Model for Decision Service:
    type Customer: <name: string, age: number, income: number>
    type Applicant: <customer: Customer, requestedAmount: number, termInMonths: number>
    type LoanOffer: <amount: number, termInMonths: number, monthlyPayment: number>

    // Decision Service:
    calculateLoanOffer(applicant: Applicant) -> LoanOffer: {
        interestRate: if applicant.customer.income > 5000 then 0.05 else 0.1;
        monthlyPayment: (applicant.requestedAmount * (1 + interestRate)) / applicant.termInMonths;
        result: {
            amount: applicant.requestedAmount;
            termInMonths: applicant.termInMonths;
            monthlyPayment: monthlyPayment
        }
    }

    // Example execution:
    applicant1: {
        customer: {name: "Alice"; age: 30; income: 6000};
        requestedAmount: 20000;
        termInMonths: 24
    }

    loanOffer1: calculateLoanOffer(applicant1).result
}
```

## Parsing

- Type name definition gate opens with `type` such that `type Customer`. Gate closes with `:`
- Type definition gate opens with `<` and closes with `>`, e.g. `<name: string, age: number, income: number>`, `<string>`, etc.
- After type name definition such that `type Customer :`, type definition gate opens with `{` and closes with `}` to mimic JSON.
For example: `type Customer: {name: <string>; age: <number>; income: <number>}` is a valid type definition as well as
`type Customer: <name: string, age: number, income: number>` will construct exactly the same type.

## Tasks

- [ ] Allow parsing type definitions.
- [ ] Allow proper type printing. Types will be printed in the same format as they are defined.


https://chatgpt.com/c/68c6dc61-1544-8321-8c71-c16e6d137865