
# TeamsModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`name` | string
`email` | string
`phone` | string
`role` | string
`blocked` | boolean
`createdAt` | Date
`updatedAt` | Date

## Example

```typescript
import type { TeamsModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "name": null,
  "email": null,
  "phone": null,
  "role": null,
  "blocked": null,
  "createdAt": null,
  "updatedAt": null,
} satisfies TeamsModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as TeamsModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


