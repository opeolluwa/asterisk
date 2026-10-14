
# LoginResult


## Properties

Name | Type
------------ | -------------
`accessToken` | string
`refreshToken` | string
`accessTokenExpiry` | number
`refreshTokenExpiry` | number
`twoFactorEnabled` | boolean

## Example

```typescript
import type { LoginResult } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "refreshToken": null,
  "accessTokenExpiry": null,
  "refreshTokenExpiry": null,
  "twoFactorEnabled": null,
} satisfies LoginResult

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as LoginResult
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


