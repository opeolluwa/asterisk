
# MarketplaceListQuery


## Properties

Name | Type
------------ | -------------
`page` | number
`pageSize` | number
`search` | string
`inStock` | boolean
`sort` | [MarketplaceSort](MarketplaceSort.md)

## Example

```typescript
import type { MarketplaceListQuery } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "page": null,
  "pageSize": null,
  "search": null,
  "inStock": null,
  "sort": null,
} satisfies MarketplaceListQuery

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as MarketplaceListQuery
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


