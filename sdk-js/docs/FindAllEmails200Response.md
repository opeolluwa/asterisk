
# FindAllEmails200Response


## Properties

Name | Type
------------ | -------------
`items` | [Array&lt;EmailsModel&gt;](EmailsModel.md)
`page` | number
`pageSize` | number
`total` | number
`totalPages` | number

## Example

```typescript
import type { FindAllEmails200Response } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "items": null,
  "page": null,
  "pageSize": null,
  "total": null,
  "totalPages": null,
} satisfies FindAllEmails200Response

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as FindAllEmails200Response
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


