
# ProductsModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`name` | string
`picture` | string
`description` | string
`createdAt` | Date
`updatedAt` | Date
`category` | string

## Example

```typescript
import type { ProductsModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "name": null,
  "picture": null,
  "description": null,
  "createdAt": null,
  "updatedAt": null,
  "category": null,
} satisfies ProductsModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ProductsModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


