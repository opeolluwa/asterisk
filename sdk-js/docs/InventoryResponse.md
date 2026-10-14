
# InventoryResponse


## Properties

Name | Type
------------ | -------------
`identifier` | string
`quantity` | number
`product` | [ProductSummary](ProductSummary.md)
`price` | [ProductPriceResponse](ProductPriceResponse.md)

## Example

```typescript
import type { InventoryResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "quantity": null,
  "product": null,
  "price": null,
} satisfies InventoryResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as InventoryResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


