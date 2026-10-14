
# CartResponse


## Properties

Name | Type
------------ | -------------
`identifier` | string
`items` | [Array&lt;CartItemResponse&gt;](CartItemResponse.md)

## Example

```typescript
import type { CartResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "items": null,
} satisfies CartResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CartResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


