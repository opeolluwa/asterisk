
# ConfirmPaymentResponse


## Properties

Name | Type
------------ | -------------
`status` | string
`orderIdentifier` | string
`message` | string

## Example

```typescript
import type { ConfirmPaymentResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "status": null,
  "orderIdentifier": null,
  "message": null,
} satisfies ConfirmPaymentResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ConfirmPaymentResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


