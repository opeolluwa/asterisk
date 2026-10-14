
# ComplaintsModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`customerIdentifier` | string
`orderIdentifier` | string
`subject` | string
`description` | string
`status` | [ComplaintStatus](ComplaintStatus.md)
`createdAt` | Date
`updatedAt` | Date
`archivedAt` | Date

## Example

```typescript
import type { ComplaintsModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "customerIdentifier": null,
  "orderIdentifier": null,
  "subject": null,
  "description": null,
  "status": null,
  "createdAt": null,
  "updatedAt": null,
  "archivedAt": null,
} satisfies ComplaintsModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ComplaintsModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


