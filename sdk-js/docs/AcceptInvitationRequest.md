
# AcceptInvitationRequest


## Properties

Name | Type
------------ | -------------
`firstName` | string
`lastName` | string
`password` | string
`confirmPassword` | string

## Example

```typescript
import type { AcceptInvitationRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "firstName": null,
  "lastName": null,
  "password": null,
  "confirmPassword": null,
} satisfies AcceptInvitationRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AcceptInvitationRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


