
# ChangePasswordRequest


## Properties

Name | Type
------------ | -------------
`currentPassword` | string
`newPassword` | string
`confirmPassword` | string
`otp` | string

## Example

```typescript
import type { ChangePasswordRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "currentPassword": null,
  "newPassword": null,
  "confirmPassword": null,
  "otp": null,
} satisfies ChangePasswordRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ChangePasswordRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


