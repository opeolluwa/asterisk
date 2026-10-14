
# TeamMember


## Properties

Name | Type
------------ | -------------
`identifier` | string
`firstName` | string
`lastName` | string
`email` | string
`profilePicture` | string
`roleIdentifier` | string
`blocked` | boolean
`createdAt` | Date
`updatedAt` | Date
`lastLogin` | Date

## Example

```typescript
import type { TeamMember } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "firstName": null,
  "lastName": null,
  "email": null,
  "profilePicture": null,
  "roleIdentifier": null,
  "blocked": null,
  "createdAt": null,
  "updatedAt": null,
  "lastLogin": null,
} satisfies TeamMember

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as TeamMember
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


