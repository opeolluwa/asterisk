
# MarketplaceItem


## Properties

Name | Type
------------ | -------------
`identifier` | string
`product` | [ProductSummary](ProductSummary.md)
`price` | [ProductPriceResponse](ProductPriceResponse.md)
`inStock` | boolean

## Example

```typescript
import type { MarketplaceItem } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "product": null,
  "price": null,
  "inStock": null,
} satisfies MarketplaceItem

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as MarketplaceItem
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


