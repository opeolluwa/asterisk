
# ListNotificationsQuery


## Properties

Name | Type
------------ | -------------
`page` | number
`pageSize` | number
`eventType` | string
`readState` | any
`unreadOnly` | boolean
`includeArchived` | boolean

## Example

```typescript
import type { ListNotificationsQuery } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "page": null,
  "pageSize": null,
  "eventType": null,
  "readState": null,
  "unreadOnly": null,
  "includeArchived": null,
} satisfies ListNotificationsQuery

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ListNotificationsQuery
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


