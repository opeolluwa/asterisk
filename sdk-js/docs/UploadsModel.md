
# UploadsModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`name` | string
`url` | string
`fileSize` | number
`starred` | boolean
`createdAt` | Date
`updatedAt` | Date
`fileType` | [FileType](FileType.md)
`filePath` | string
`thumbnailUrl` | string

## Example

```typescript
import type { UploadsModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "name": null,
  "url": null,
  "fileSize": null,
  "starred": null,
  "createdAt": null,
  "updatedAt": null,
  "fileType": null,
  "filePath": null,
  "thumbnailUrl": null,
} satisfies UploadsModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UploadsModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


