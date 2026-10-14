
# NotificationPreferencesModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`recipientIdentifier` | string
`recipientType` | [NotificationRecipientType](NotificationRecipientType.md)
`eventType` | string
`channel` | [NotificationChannel](NotificationChannel.md)
`enabled` | boolean
`createdAt` | Date
`updatedAt` | Date

## Example

```typescript
import type { NotificationPreferencesModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "recipientIdentifier": null,
  "recipientType": null,
  "eventType": null,
  "channel": null,
  "enabled": null,
  "createdAt": null,
  "updatedAt": null,
} satisfies NotificationPreferencesModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as NotificationPreferencesModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


