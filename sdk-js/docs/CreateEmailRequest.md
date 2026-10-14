
# CreateEmailRequest


## Properties

Name | Type
------------ | -------------
`subject` | string
`body` | string
`senderEmail` | string
`firstName` | string
`lastName` | string
`phone` | string
`recipientEmail` | string
`tag` | string
`hasAttachments` | boolean
`data` | any

## Example

```typescript
import type { CreateEmailRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "subject": null,
  "body": null,
  "senderEmail": null,
  "firstName": null,
  "lastName": null,
  "phone": null,
  "recipientEmail": null,
  "tag": null,
  "hasAttachments": null,
  "data": null,
} satisfies CreateEmailRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CreateEmailRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


