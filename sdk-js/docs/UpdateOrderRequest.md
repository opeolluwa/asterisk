
# UpdateOrderRequest


## Properties

Name | Type
------------ | -------------
`status` | [OrderStatus](OrderStatus.md)
`paymentReference` | string

## Example

```typescript
import type { UpdateOrderRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "status": null,
  "paymentReference": null,
} satisfies UpdateOrderRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateOrderRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


