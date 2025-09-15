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
{
    vectorStore: {x: 1, y: 2}
    relationsList: [1,2,3,4]
    standardObject: {x: "header"; y: 123}
}
```
```edgerules
myModel: {
    vectorStore: {x: 1, y: 2}
    identification: Missing
    relationsList: [1,2,3,4]
    standardObject: {x: "header"; y: 123}
}
```

## Limited casting CN

As previously shown, it is possible to "clarify" the type of variable by using `as <type>` operator.
No real casting is actually happening: if user want's to cast number to string, they should use `toString(number)` function,
because operator `as <type>` will give an exception - it only narrows the type of the variable.

However, such as casting can be beneficial for special values usage and complex structures:

```edgerules
{
    vector: <x: string, y: number>
    incompleteVector: {x: "header"} as <vector>
    yAxis: incompleteVector.y + 100 // will give a Missing special value
}
```

## Decision Service Example

```edgerules
{
    Customer: <name: string, age: number, income: number>
    Applicant: <customer: Customer, requestedAmount: number, termInMonths: number>
    LoanOffer: <amount: number, termInMonths: number, monthlyPayment: number>

    calculateLoanOffer(applicant: Applicant) -> LoanOffer: {
        interestRate: if applicant.customer.income > 5000 then 0.05 else 0.1;
        monthlyPayment: (applicant.requestedAmount * (1 + interestRate)) / applicant.termInMonths;
        result: {
            amount: applicant.requestedAmount;
            termInMonths: applicant.termInMonths;
            monthlyPayment: monthlyPayment
        }
    }

    applicant1 -> ???? Applicant: {
        customer: {name: "Alice"; age: 30; income: 6000} as <Customer>;
        requestedAmount: 20000;
        termInMonths: 24
    }

    loanOffer1: calculateLoanOffer(applicant1).result
}
```

## Tasks

- [ ] Allow parsing type definitions. All definitions will start with `<` and end with `>`.
- [ ] Allow proper type printing. Types will be printed in the same format as they are defined.


https://chatgpt.com/c/68c6dc61-1544-8321-8c71-c16e6d137865