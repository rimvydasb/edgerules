# Objects Context

## Simple References
```edgerules
application: {
    applDate: 20230402
    applicants: [1,2,3]
    testReference: applicants[0]
}
```

output:
```edgerules
{
   application : {
      applDate : 20230402
      applicants : [1, 2, 3]
      testReference : 1
   }
}
```

## Anonymous Contexts
*The following contexts are not assigned to any variable, but stays in the index.
For this reason, they're treated as anonymous contexts.*

```edgerules
application: {
    applDate: 20230402
    applicants: [
        {
            id: 1
            date: 20210102
            age: application.applDate - date
        },
        {
            id: 2
            date: 20220102
            age: application.applDate - date
        }
    ]
}
```

output:
```edgerules
{
   application : {
      applDate : 20230402
      applicants : [{id : 1; date : 20210102; age : application.applDate - date}, {id : 2; date : 20220102; age : application.applDate - date}]
   }
}
```

Another similar calculation where reference to the root is allowed

```edgerules
calendar: {
    shift: 2
    days: [
	    {start: calendar.shift + 1},
	    {start: calendar.shift + 31}
    ]
    firstDay: days[0].start
    secondDay: days[1].start
}
```

output:
```edgerules
{
   calendar : {
      shift : 2
      days : [{start : 3}, {start : 33}]
      firstDay : 3
      secondDay : 33
   }
}
```

Another similar calculation, but now in an array object, tries referring to the root calendar field.
It should not work because single objects in an array have their own context, and they can refer to a root object in addition.
With this limitation, the ambiguity is removed.

```edgerules
calendar: {
    shift: 2
    days: [
	    {start: shift + 1},
	    {start: shift + 31}
    ]
    firstDay: days[0].start
    secondDay: days[1].start
}
```

output:
```edgerules
Field shift not found in Root
Context:
  1. Error in `Root.Variable(VariableLink { path: ["shift"], variable_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }) })`: Field shift not found in Root
  2. Error in `Root.Operator(MathOperator { data: OperatorData { operator: Addition, left: Variable(VariableLink { path: ["shift"], variable_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }) }), right: Value(NumberValue(Int(31))) }, function: 0x100ea63f4 })`: Field shift not found in Root
  3. Error in `Root.calendar.Selection(FieldSelection { source: Filter(ExpressionFilter { source: Variable(VariableLink { path: ["days"], variable_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }) }), method: Value(NumberValue(Int(0))), method_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }), return_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }) }), method: VariableLink { path: ["start"], variable_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }) }, return_type: Err(GeneralStackedError { error: NotLinkedYet, context: [] }) })`: Field shift not found in Root
```

As a potential work-around for previous problem, is a possibility to have _requirements_ expressed for an object.

```edgerules
calendar: {
    shift: 2
    days: [
	    {start(config): config.shift + 1},
	    {start(config): config.shift + 31}
    ]
    firstDay: days[0].start(calendar)
    secondDay: days[1].start(calendar)
}
```

output:
```edgerules
function 'start' body is not defined → 'days' assignment side is not complete → 'calendar' assignment side is not complete
Selection must be variable or variable path → 'firstDay' assignment side is not complete
Selection must be variable or variable path → 'secondDay' assignment side is not complete
```

> it is possible to get cyclic reference if referring another non-calculated line

> would be good to allow collection references

```edgerules
calendar: {
    shift: 2
    days: [
	    {start: calendar.shift + 1},
	    {start: calendar.days[0].start + 5}
    ]
    secondDay: days[1].start
}
```

output:
```edgerules
Selection must be variable or variable path → 'start' assignment side is not complete → 'days' assignment side is not complete → 'calendar' assignment side is not complete
```

> anonymous contexts are kept isolated if are deeper down in AST, because it is not quite clear how they can be accessed. For example filter is applied as an upper AST expression.

```edgerules
calendar: {
    shift: 2
    positiveDays: [
	    {id: 1, start: calendar.shift + 1},
	    {id: 2, start: calendar.days[0].start + 5}
    ][id > 0]
    secondDay: positiveDays[1].start
}
```

output:
```edgerules
',' is not a proper context element → 'positiveDays' assignment side is not complete → 'calendar' assignment side is not complete
```


## Loop and built-ins:

```edgerules
model : {
    sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
    salesCount : count(sales)
    func sales3(month, sales) : { result : sales[month] + sales[month + 1] + sales[month + 2] }
    acc : for m in 1..(salesCount - 2) return sales3(m, sales).result
    best : max(acc)
}
```

output:
```edgerules
{
   model : {
      sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
      salesCount : 12
      acc : [35, 16, 18, 17, 94, 84, 86, 8, 16, 8]
      best : 94
   }
}
```

## Creating multiple instances:

```edgerules
{
    // input data
    application: {
        effectiveTimestamp: 20220512
        applicants: [
            {
                birthday: 20050101
            },
            {
                birthday: 20010101
            }
        ]
    }

    // instance definition
    func applicantRecord(inputData): {
        age: inputData.application.effectiveTimestamp - inputData.birthday
    }

    // creating multiple instances
    applicationRecord: {
        inputData: application
        applicantRecords: for record in application.applicants return applicantRecord(record)
    }
}
```

output:
```edgerules
{
   application : {
      effectiveTimestamp : 20220512
      applicants : [{birthday : 20050101}, {birthday : 20010101}]
   }
   applicationRecord : {
      application : {
         effectiveTimestamp : 20220512
         applicants : [{birthday : 20050101}, {birthday : 20010101}]
      }
      applicantRecords : [{age : inputData.application.effectiveTimestamp - inputData.birthday}, {age : inputData.application.effectiveTimestamp - inputData.birthday}]
   }
}
```

## Nesting Failures

The following example should not work and inform user that 's' assignment side is not complete

```edgerules
{
    input : {

        today : 2021
        birthday : 2022
    },


    record : {

        age : input.birthday - input.today
        salary : sum(1,2)
            record : {

				age : input.birthday - input.today
				salary : sum(1,3)

					record : {

                    age : input.birthday - input.today
                    salary : sum(1,4)
                    another : {

					    s :
				}
			}
		}
    }
}
```

output:
```edgerules
',' is not a proper context element
's' assignment side is not complete → 'another' assignment side is not complete → 'record' assignment side is not complete → 'record' assignment side is not complete → 'record' assignment side is not complete
```

## Complex Filtering

```edgerules
{
    nums         : [1, 5, 12, 7]
    filtered     : nums[...>6]
}
```

output:
```edgerules
{
   filtered : [12, 7]
   nums : [1, 5, 12, 7]
}
```
