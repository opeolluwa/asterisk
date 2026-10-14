
# LoginChallenge


## Properties

Name | Type
------------ | -------------
`token` | string
`expiresAt` | number
`twoFactorEnabled` | boolean

## Example

```typescript
import type { LoginChallenge } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "token": null,
  "expiresAt": null,
  "twoFactorEnabled": null,
} satisfies LoginChallenge

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as LoginChallenge
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


