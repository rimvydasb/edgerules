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
// output
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
// output
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
// output
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
// output
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
// output
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
// output
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
// output
```


## Loop and built-ins:

```edgerules
model : {
    sales : [10, 20, 8, 7, 1, 10, 6, 78, 0, 8, 0, 8]
    salesCount : count(sales)
    sales3(month, sales) : { result : sales[month] + sales[month + 1] + sales[month + 2] }
    acc : for m in 1..(salesCount - 2) return sales3(m, sales).result
    best : max(acc)
}
```

output:
```edgerules
// output
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
    applicantRecord(inputData): {
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
// output
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
// output
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
// output
```
