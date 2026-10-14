
# CreateInventoryRequest


## Properties

Name | Type
------------ | -------------
`productIdentifier` | string
`price` | number
`currency` | string
`quantity` | number

## Example

```typescript
import type { CreateInventoryRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "productIdentifier": null,
  "price": null,
  "currency": null,
  "quantity": null,
} satisfies CreateInventoryRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateInventoryRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


