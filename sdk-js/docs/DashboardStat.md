
# DashboardStat


## Properties

Name | Type
------------ | -------------
`key` | [DashboardStatKey](DashboardStatKey.md)
`label` | string
`value` | number
`deltaPercent` | number
`comparisonLabel` | string

## Example

```typescript
import type { DashboardStat } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "key": null,
  "label": null,
  "value": null,
  "deltaPercent": null,
  "comparisonLabel": null,
} satisfies DashboardStat

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DashboardStat
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


