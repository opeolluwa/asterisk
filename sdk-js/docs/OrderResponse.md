
# OrderResponse


## Properties

Name | Type
------------ | -------------
`identifier` | string
`customerIdentifier` | string
`status` | [OrderStatus](OrderStatus.md)
`paymentReference` | string
`createdAt` | Date
`delivery` | [DeliveryResponse](DeliveryResponse.md)
`items` | [Array&lt;OrderItemResponse&gt;](OrderItemResponse.md)

## Example

```typescript
import type { OrderResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "customerIdentifier": null,
  "status": null,
  "paymentReference": null,
  "createdAt": null,
  "delivery": null,
  "items": null,
} satisfies OrderResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OrderResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


