# Rulesets Story

## Introduction

EdgeRules must provide native rulesets support to enable users to define rulesets, decision tables and rules.

## Rulesets Structure

### Basic Ruleset Example

```edgerules
{
    func eligibilityDecision(applicant): {
        rules: [
            {rule1: applicant.income > applicant.expense * 2}
            {rule2: applicant.income > 1000}
            {rule3: applicant.age >= 18}
        ]
    }
    applicantEligibility: eligibilityDecision({
        income: 1100
        expense: 600
        age: 22
    }).rules
}
```

**output:**

```json
{
  "applicantEligibility": {
    "rules": [
      {
        "rule1": false
      },
      {
        "rule2": true
      },
      {
        "rule3": true
      }
    ]
  }
}
```

### Basic Rules Example With Action

```edgerules
{
    func eligibilityDecision(applicantRecord): {
        rules: [
            {name: "INC_CHECK"; rule: applicantRecord.data.income > applicantRecord.data.expense * 2}
            {name: "MIN_INCOM"; rule: applicantRecord.data.income > 1000}
            {name: "AGE_CHECK"; rule: applicantRecord.data.birthDate + period('P18Y') <= applicantRecord.checkDate}
        ]
        firedRules: for invalid in rules[rule = false] return invalid.name
        status: if count(rules) = 0 then "ELIGIBLE" else "INELIGIBLE"
    }
    applicantEligibility: eligibilityDecision({
        data: {
            income: 1100
            expense: 600
            birthDate: date('2005-01-01')
        }
        checkDate: date('2023-01-01')
    })
}
```

**output:**

```json
{
  "applicantEligibility": {
    "rules": [
      {
        "name": "INC_CHECK",
        "rule": false
      },
      {
        "name": "MIN_INCOM",
        "rule": true
      },
      {
        "name": "AGE_CHECK",
        "rule": false
      }
    ],
    "firedRules": [
      "INC_CHECK",
      "AGE_CHECK"
    ],
    "status": "INELIGIBLE"
  }
}
```

### Decision Table Example

```edgerules
{
    func shippingCostDecision(order): {
        rules: [
            {name: "RULE1"; rule: order.destination = "US" and order.weight <= 5; action: 10}
            {name: "RULE2"; rule: order.destination = "US" and order.weight > 5 and order.weight <= 20; action: 20}
            {name: "RULE3"; rule: order.destination = "US" and order.weight > 20; action: 50}
            {name: "RULE4"; rule: order.destination = "International" and order.weight <= 5; action: 25}
            {name: "RULE5"; rule: order.destination = "International" and order.weight > 5 and order.weight <= 20; action: 50}
            {name: "RULE6"; rule: order.destination = "International" and order.weight > 20; action: 100}
        ]
        applicableRules: rules[rule = true]
        totalCost: if count(applicableRules) > 0 then applicableRules[0].action else 0
    }
    orderShippingCost: shippingCostDecision({
        destination: "US"
        weight: 10
    }).totalCost
}
```

**Benefits:**

- Clear separation of rules and actions.
- Rule is an object and can be easily extended, for example with priority or reason code fields.
- Same view if converted to portable JSON format.
- External tools can add or remove rules easily.

**Drawbacks:**

- `rule` block must be split by `and` to columns and columns must be merged back - extra processing step and plenty of room for
  ambitious errors that could be displayed to the user.
- `rule` block should represent a row but no built-in guarantees for ordering.
