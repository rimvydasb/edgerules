# Types as Placeholders

EdgeRules does not have a standard type definition, as it is common in other software tools.
EdgeRules use typed placeholders instead.

## Terminology

 - **stand-alone model** - evaluatable model that does not have any external context or requires any input
 - **decision service** - a model that can be comprehensively evaluated only with an external context

EdgeRules supports a fixed set of core primitive types and can print structure types. For example, for a given structure,
the following type definition will be printed:

```edgerules
{a : 88; b : 99; c : {x : 'Hello'; y : a + b; userFunction() : {}}}
```

The `get_type` method output **structured** type definition:

```edgerules
{a: <number>; b: <number>; {c: {x: <string>; y: <number>}}}
```

The `get_type` method will return the structured type, because no other types are defined. 
The method extracts already linked type definitions and prints them.

Below is an example of a standard expression definition:

```edgerules
{
    myObject: {a: 88; b: 99; c: {x: 'Hello'; y: a + b; userFunction() : {}}}
    myPrimitive: 123
}
```

The `myObject` (same as `myPrimitive`) gets an expression assigned that does the following things:
1. Defines a variable with a given name `myObject` on the left side
2. Link types if they're not linked
3. Evaluates the expression on the right side and creates an instance of the result
4. Assigns the result to the variable `myObject`

The goal of this story is to allow users to define a typed placeholder that immediately assigns a type to a 
given variable. User will be able to define complex typed placeholders as well as simple.

## Evaluation Examples

It is a completely valid when the user defines a mixture of placeholders and expressions:

```edgerules
{
    type Vector: <x: string, y: number>                     // type alias and assigned definition
    vectorStore: <id: number, name: string, c: Vector[]>    // variable with complex type placeholder with Vector type reference
    identification: <number>                                // variable with simple type placeholder
    relationsList: <number[]>                               // variable with simple type placeholder with array of numbers
    standardObject: {x: "header"; y: 123}                   // variable that has standard expression assigned
    vectorInstance -> Vector: {x: 5; y: 15} (@Todo)            // variable with type placeholder and expression assigned
}
```

Previous model can be evaluated as a **stand-alone** model without any context:

```edgerules
{
    vectorStore: Missing
    identification: Missing
    relationsList: Missing
    standardObject: {x: "header"; y: 123}
    vectorInstance: {x: 5; y: 15}
}
```

Evaluated model **with** the context example:

```edgerules
// context: (this can also represent the request to the decision service)
{
    vectorStore: {c: {x: 1, y: 2}}
    relationsList: [1,2,3,4]
}
```
```edgerules
// evaluated model: (this can also represent the response from the decision service)
myModel: {
    vectorStore: {id: Missing, name: Missing, c: {x: 1, y: 2}}
    identification: Missing
    relationsList: [1,2,3,4]
    standardObject: {x: "header"; y: 123}
    vectorInstance: {x: 5; y: 15}
}
```

## Casting

EdgeRules does not have primitive casting when you can "trick" compiler such as in Java or "fake-out" execution such as in TypeScript.
EdgeRules casting works as following:
1. Target type is identified
2. Empty instance of target type is created
3. Casted value is deeply copied to the target value with validation applied according to target type defintion
4. For every non-copied value the default value is inserted based on target type defintion
5. For every additional value, that does not have a field in target type definition, error is raised and execution is terminated

In the fiture it will be possible to skip step #5, but this will be considered in specific execution mode

```edgerules
{
    type Customer: {
        name: <string, "UNKNOWN">;
        age: <number, Missing, [..>0]>;
        income: <number, 0>;
    }
    // this model example will take a primaryCustomer from the context whatever structure it will be and casts to Customer:
    primaryCustomer: <Customer>     

    // the following cast is happening during the execution:
    customer: primaryCustomer as Customer
}
```

## Clarifications

- `type $TYPENAME` will define a type alias, for example `type Vector: <x: string, y: number>` - Vector will become an alias of x and y complex type.
- `$VARNAME: <$TYPENAME>` will define a variable and will assign a type placeholder. If no real value will be 
assigned from the context, then the variable will have `Missing` **special value** during the evaluation (see Special Values story)
- Defined types are scoped to the context where it is defined and inner scopes.
- Type can also be defined in JSON style structure and will be called **structured** as well as **inline**:

```edgerules
{
    // Customer **inline** type:
    type CustomerA: <name: string, age: number, income: number>
    
    // The same as CustomerA, but defined in structured style:
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
    calculateLoanOffer(applicant: Applicant): {
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

    loanOffer1: calculateLoanOffer(applicant1).result as LoanOffer
}
```

## Default Values and Validation Support

The default value can be assigned to the type definition gate. If not specified, then default valye will always be Missing.
The full type definition format is the following:
`<$TYPE_NAME, $DEFAULT_VALUE, $PREDICATE>`

```edgerules
{
    type Customer: {
        name: <string, "UNKNOWN">;
        age: <number, Missing, [..>0]>;
        income: <number, 0>;
    }
    input: {age: 39}
    customer: input as Customer
}
```

Evaluates to:


```edgerules
{
    input: {}
    customer: {
        name: "UNKNOWN"
        age: 39
        income: 0
    }
}
```

## Parsing

- Type alias name definition gate opens with `type` such that `type Customer`. Gate closes with `:`
- Type definition gate opens with `<` and closes with `>`, e.g. `<name: string, age: number, income: number>`, `<string>`, etc.
- When gate is opened, then everything inside is considered part of the type definition until the gate is closed.
Inner gates will not be allowed.
- After the type name alias definition (`type Customer:`), the type definition gate opens with `{` and closes with `}` to mimic JSON.
For example: `type Customer: {name: <string>; age: <number>; income: <number>}` is a valid type definition as well as
`type Customer: <name: string, age: number, income: number>` will construct exactly the same type.

## Tasks

- [ ] Allow parsing type definitions.
- [ ] During the linking phase, the given structure will be linked based on the type if provided, for example:
```edgerules
{
    type Record: <a: number, b: number>
    myRecord: {a: 5; b: 10} as Record             // will be linked as Record type
    anotherRecord: <Record>                       // will be linked as Record type and during the execution Missing special value will be assigned
    simpleNumber: {a: 5} as Record                // will be linked as Record and during executuion value b will be Missing
    invalidRecord: {a: 5; b: 'Hello'} as Record   // will produce a type mismatch error during the linking phase
}
```
- [ ] Allow proper type printing. Types will be printed in the same format as they are defined.
- [ ] Add tests, TBC.
