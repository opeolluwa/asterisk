
# UserRolesModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`name` | string
`description` | string
`createdAt` | Date
`updateAt` | Date
`isArchived` | boolean

## Example

```typescript
import type { UserRolesModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "name": null,
  "description": null,
  "createdAt": null,
  "updateAt": null,
  "isArchived": null,
} satisfies UserRolesModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserRolesModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


