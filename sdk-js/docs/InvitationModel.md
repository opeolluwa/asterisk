
# InvitationModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`email` | string
`status` | [InvitationStatus](InvitationStatus.md)
`token` | string
`createdAt` | Date
`updatedAt` | Date
`firstName` | string
`lastName` | string
`roleIdentifier` | string
`invitedBy` | string

## Example

```typescript
import type { InvitationModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "email": null,
  "status": null,
  "token": null,
  "createdAt": null,
  "updatedAt": null,
  "firstName": null,
  "lastName": null,
  "roleIdentifier": null,
  "invitedBy": null,
} satisfies InvitationModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as InvitationModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


