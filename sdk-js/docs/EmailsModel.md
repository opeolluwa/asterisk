
# EmailsModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`subject` | string
`body` | string
`senderEmail` | string
`recipientEmail` | string
`dateSent` | Date
`tag` | string
`isRead` | boolean
`isStarred` | boolean
`hasAttachments` | boolean
`data` | any
`userIdentifier` | string
`createdAt` | Date
`updatedAt` | Date
`firstName` | string
`lastName` | string
`phone` | string

## Example

```typescript
import type { EmailsModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "subject": null,
  "body": null,
  "senderEmail": null,
  "recipientEmail": null,
  "dateSent": null,
  "tag": null,
  "isRead": null,
  "isStarred": null,
  "hasAttachments": null,
  "data": null,
  "userIdentifier": null,
  "createdAt": null,
  "updatedAt": null,
  "firstName": null,
  "lastName": null,
  "phone": null,
} satisfies EmailsModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as EmailsModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


