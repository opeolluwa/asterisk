
# CompleteCustomerOnboardingRequest


## Properties

Name | Type
------------ | -------------
`phone` | string
`address` | [CreatePrimaryDeliveryAddress](CreatePrimaryDeliveryAddress.md)

## Example

```typescript
import type { CompleteCustomerOnboardingRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "phone": null,
  "address": null,
} satisfies CompleteCustomerOnboardingRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CompleteCustomerOnboardingRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


