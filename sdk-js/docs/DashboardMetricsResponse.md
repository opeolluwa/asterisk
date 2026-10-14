
# DashboardMetricsResponse


## Properties

Name | Type
------------ | -------------
`stats` | [Array&lt;DashboardStat&gt;](DashboardStat.md)
`revenueTrend` | [Array&lt;RevenueTrendPoint&gt;](RevenueTrendPoint.md)
`salesByCategory` | [Array&lt;CategorySales&gt;](CategorySales.md)

## Example

```typescript
import type { DashboardMetricsResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "stats": null,
  "revenueTrend": null,
  "salesByCategory": null,
} satisfies DashboardMetricsResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DashboardMetricsResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


