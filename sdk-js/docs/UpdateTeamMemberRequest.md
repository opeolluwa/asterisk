
# UpdateTeamMemberRequest


## Properties

Name | Type
------------ | -------------
`name` | string
`phone` | string
`role` | string
`blocked` | boolean

## Example

```typescript
import type { UpdateTeamMemberRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "name": null,
  "phone": null,
  "role": null,
  "blocked": null,
} satisfies UpdateTeamMemberRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateTeamMemberRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


