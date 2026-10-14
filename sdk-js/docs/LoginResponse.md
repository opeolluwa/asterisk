
# LoginResponse

Untagged, so `ts-rs` emits the union of the variant payloads (`LoginResult | LoginChallenge`) with no tag to tell them apart.

## Properties

Name | Type
------------ | -------------
`accessToken` | string
`refreshToken` | string
`accessTokenExpiry` | number
`refreshTokenExpiry` | number
`twoFactorEnabled` | boolean
`token` | string
`expiresAt` | number

## Example

```typescript
import type { LoginResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "refreshToken": null,
  "accessTokenExpiry": null,
  "refreshTokenExpiry": null,
  "twoFactorEnabled": null,
  "token": null,
  "expiresAt": null,
} satisfies LoginResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as LoginResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


