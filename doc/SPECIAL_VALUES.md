## Special Values Story

EdgeRules fault-tolerant strategy provides special values for various unexpected situations.
Each special value has a specific meaning and treatment.

Special Value has this structure:

```ebnf
SpecialValue ::= "Missing" | "NotApplicable" | "NotFound" ("(" Origin , Trace ")")?
```

| Name            | Description                         | Treatment              | Can be assigned by user? |
|-----------------|-------------------------------------|------------------------|--------------------------|
| `Missing`       | value is expected, but not found    | override by `Missing`  | Yes                      |
| `NotApplicable` | value is not expected and not found | treat as 0, 1 or ""    | Yes                      |
| `NotFound`      | value entry is not found            | override by `NotFound` | No - system only         |

## Missing

Has a high similarity to JavaScript `null`, but has fault-tolerant treatment.
Missing can be assigned by the user to any type except boolean to inform execution that data was not provided, but expected.
Missing is also assigned by the system if no data is provided to the function or data is missing in casting operation.

> Booleans cannot be Missing, because boolean logic must be strict.

### Missing Treatment

In general, user should think about `Missing` as null, that propagates in all operations and yields `Missing` as result.

1. In arithmetic and logical operations, it propagates `Missing`: `1 + Missing` → `Missing`, `1 * Missing` → `Missing`.
2. In string concatenation, it propagates `Missing`: `"a" + Missing` → `Missing`.
3. If `Missing` appeared as an argument in any built-in function, the result is `Missing`: `sum([1,2,Missing,4])` → `Missing`, `max([1,2,Missing,4])` → `Missing`.
4. In comparisons, it propagates `Missing`: `Missing < 0` → `Missing`, `Missing < ""` → `Missing`
except it is only equal to itself: `Missing = Missing` → `true`
5. In lists, it is treated as a valid entry: `[1, Missing, 3][1]` → `Missing`.
6. In lists, it can be accessed such that `for n in [1,2,Missing,4] return n + 1` → `[2,3,Missing,5]`
7. In objects, it is treated as a valid field value: `{a: Missing}.a` → `Missing`.
8. In filtering, it is ignored: `[1,2,Missing,4][x > 2]` → `[4]`.

## NotApplicable

Special value that informs the execution that it must be ignored in all operations if possible.
Can be assigned to numbers, strings and dates.

> Booleans cannot be NotApplicable, because boolean logic must be strict.

### NotApplicable Treatment:

In general, user should think about `NotApplicable` as a value that is ignored in all operations if it makes sense or possible.

1. In arithmetic operations, it is treated as 0 or 1 respectively: `1 + NotApplicable` → `1`, `1 * NotApplicable` → `1`.
2. In string concatenation, it is treated as an empty string `""`: `"a" + NotApplicable` → `"a"`.
3. In comparisons, it always results as false except equals to `NotApplicable`:
`NotApplicable < 0` → `false`, `NotApplicable < ""` → `false`, 
`NotApplicable = 0` → `false`, `NotApplicable = NotApplicable` → `true`
4. In logical operations `NotApplicable` is not supported and will cause a linking error if user tries type it there. 
However, since it is not assignable to booleans, it cannot appear in logical expressions during runtime.
5. In lists, it is treated as a valid entry: `[1, NotApplicable, 3][1]` → `NotApplicable`.
6. In objects, it is treated as a valid field value: `{a: NotApplicable}.a` → `NotApplicable`.
7. In type casting, it is treated as a valid value and if source has `NotApplicable` for the field, target will also have `NotApplicable` for that field.
8. In filtering, it is ignored: `[1,2,NotApplicable,4][x > 2]` → `[4]`.
9. In list operations, it is ignored where possible, but still holds a place in a list, so: 
`count([1,2,NotApplicable,4])` → `3`, 
but `sum([1,2,NotApplicable,4])` → `7`.
10. In functions, it is ignored if it makes sense or possible: `max([1,2,NotApplicable,4])` → `4`, `mean([1,2,NotApplicable,4])` → `2.3333`.

## NotFound

Has a high similarity to JavaScript `undefined`, but has fault-tolerant treatment. It is only assigned by the system
and cannot be assigned by the user.

### Occurrence

1. Array index is out of bounds: `[1,2,3,4][4]` → `NotFound`.
2. `find` function does not find the value: `find([1,2,3], 4)` → `NotFound`.
3. Accessing a non-existing field in an object: `for item in [{a:1},{a:2},{b:3}] return item.a` → `[1, 2, NotFound(a)]`.

### NotFound Treatment

It is treated same as `Missing` in all operations.

# Special value methods

- `isMissing(x)`, `isNotApplicable(x)`, `isNotFound(x)`, `isSpecialValue(x)`, `isPresent(x)`

## Examples

```edgerules
{
    // Missing examples
    a: Missing
    b: [1, 2, Missing, 4]
    c: for n in b return n + 1          // [2, 3, Missing, 5]
    d: sum(b)                           // Missing('b')
    e: count(b)                         // Missing('b')
    f: a = Missing                      // true
    g: isMissing(a)                     // true

    // NotApplicable examples
    h: NotApplicable
    i: [1, 2, NotApplicable, 4]
    j: for n in i return n + 1          // [2, 3, 5]
    k: sum(i)                           // 12 (NotApplicable treated as 0)
    l: count(i)                         // 3 (NotApplicable is ignored)
    m: h = NotApplicable                // true
    n: isNotApplicable(h)               // true

    // NotFound examples
    objList: [{a:1},{a:2},{b:3}]
    o: for item in objList return item.a // [1, 2, NotFound('a')]
    p: find([1,2,3], 4)                 // NotFound('list')
    q: [1,2,3][4]                       // NotFound('list')
    r: p = NotFound                      // true
    s: isNotFound(p)                    // true

    // Mixed examples
    t: [1, Missing, NotApplicable, NotFound] 
    u: for n in t return n               // [1, Missing, NotApplicable, NotFound]
    v: sum(t)                            // 1 (NotApplicable treated as 0; Missing and NotFound ignored)
    w: count(t)                          // 2 (Missing and NotFound ignored; NotApplicable is counted)
}
```