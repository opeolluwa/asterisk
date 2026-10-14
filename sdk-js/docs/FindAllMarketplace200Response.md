
# FindAllMarketplace200Response


## Properties

Name | Type
------------ | -------------
`items` | [Array&lt;MarketplaceItem&gt;](MarketplaceItem.md)
`page` | number
`pageSize` | number
`total` | number
`totalPages` | number

## Example

```typescript
import type { FindAllMarketplace200Response } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "items": null,
  "page": null,
  "pageSize": null,
  "total": null,
  "totalPages": null,
} satisfies FindAllMarketplace200Response

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as FindAllMarketplace200Response
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


