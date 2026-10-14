
# AddPermissionRequest


## Properties

Name | Type
------------ | -------------
`resource` | string
`action` | [RolePermissionAction](RolePermissionAction.md)

## Example

```typescript
import type { AddPermissionRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "resource": null,
  "action": null,
} satisfies AddPermissionRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AddPermissionRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


