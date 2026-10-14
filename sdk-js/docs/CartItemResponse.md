
# CartItemResponse


## Properties

Name | Type
------------ | -------------
`identifier` | string
`cartIdentifier` | string
`quantity` | number
`available` | number
`product` | [ProductSummary](ProductSummary.md)
`price` | [ProductPriceResponse](ProductPriceResponse.md)

## Example

```typescript
import type { CartItemResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "cartIdentifier": null,
  "quantity": null,
  "available": null,
  "product": null,
  "price": null,
} satisfies CartItemResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CartItemResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


