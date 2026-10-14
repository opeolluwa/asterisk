
# UpdateEmailRequest


## Properties

Name | Type
------------ | -------------
`tag` | string
`isRead` | boolean
`isStarred` | boolean

## Example

```typescript
import type { UpdateEmailRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "tag": null,
  "isRead": null,
  "isStarred": null,
} satisfies UpdateEmailRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateEmailRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


