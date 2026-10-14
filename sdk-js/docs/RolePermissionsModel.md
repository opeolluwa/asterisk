
# RolePermissionsModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`resource` | string
`action` | [RolePermissionAction](RolePermissionAction.md)
`roleIdentifier` | string
`createdAt` | Date
`updatedAt` | Date

## Example

```typescript
import type { RolePermissionsModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "resource": null,
  "action": null,
  "roleIdentifier": null,
  "createdAt": null,
  "updatedAt": null,
} satisfies RolePermissionsModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as RolePermissionsModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


