
# OrderListQuery


## Properties

Name | Type
------------ | -------------
`page` | number
`pageSize` | number
`status` | [OrderStatus](OrderStatus.md)

## Example

```typescript
import type { OrderListQuery } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "page": null,
  "pageSize": null,
  "status": null,
} satisfies OrderListQuery

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OrderListQuery
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


