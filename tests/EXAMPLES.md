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

*Another similar calculation:*
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

> it is possible to get cyclic reference if refering another non-calculated line

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