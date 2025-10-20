# Type Schema Documentation

## Known Issues

As of now a lot of tests fails:

```
assertion `left == right` failed
  left: "{a: number; b: number}"
 right: "Type<a: number, b: number>"
```

`{a: number; b: number}` is a correct representation of the type schema.
I fixed few of them but there are still some remaining.

## Implementation Notes

1. `to_type_string` rename to `to_schema`
2. `to_schema` must be under the new trait `trait ToSchema`
3. Implement `ToSchema` for `UserTypeBody` as well:

```edgerules
{
    type Customer: {valid: <boolean; name: <string>; birthdate: <date>; birthtime: <time>; birthdatetime: <datetime>; income: <number>}
    func incAll(customer: Customer): {
        primaryCustomer: customer
    }
    value: incAll({})
}
```

If correctly implemented, the above should yield the type schema:

```edgerule
{
    Customer: {valid: boolean; name: string; birthdate: date; birthtime: time; birthdatetime: datetime; income: number}
    value: {primaryCustomer: Customer}
}