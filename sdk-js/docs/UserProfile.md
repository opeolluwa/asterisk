
# UserProfile


## Properties

Name | Type
------------ | -------------
`identifier` | string
`email` | string
`firstName` | string
`lastName` | string
`profilePicture` | string
`username` | string
`twoFactorEnabled` | boolean
`roleIdentifier` | string

## Example

```typescript
import type { UserProfile } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "email": null,
  "firstName": null,
  "lastName": null,
  "profilePicture": null,
  "username": null,
  "twoFactorEnabled": null,
  "roleIdentifier": null,
} satisfies UserProfile

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserProfile
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


