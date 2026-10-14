
# NotificationView


## Properties

Name | Type
------------ | -------------
`identifier` | string
`eventType` | string
`title` | string
`body` | string
`metadata` | any
`actorIdentifier` | string
`actorType` | [NotificationActorType](NotificationActorType.md)
`entityType` | string
`entityIdentifier` | string
`createdAt` | Date
`readAt` | Date
`archivedAt` | Date

## Example

```typescript
import type { NotificationView } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "eventType": null,
  "title": null,
  "body": null,
  "metadata": null,
  "actorIdentifier": null,
  "actorType": null,
  "entityType": null,
  "entityIdentifier": null,
  "createdAt": null,
  "readAt": null,
  "archivedAt": null,
} satisfies NotificationView

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as NotificationView
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


