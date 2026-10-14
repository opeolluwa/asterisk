
# ComplaintRepliesModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`complaintIdentifier` | string
`body` | string
`senderEmail` | string
`createdAt` | Date
`updatedAt` | Date

## Example

```typescript
import type { ComplaintRepliesModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "complaintIdentifier": null,
  "body": null,
  "senderEmail": null,
  "createdAt": null,
  "updatedAt": null,
} satisfies ComplaintRepliesModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ComplaintRepliesModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


