
# CustomerAddressModel


## Properties

Name | Type
------------ | -------------
`identifier` | string
`customerIdentifier` | string
`addressLineOne` | string
`addressLineTwo` | string
`localGovtArea` | string
`cityOrTown` | string
`state` | string
`createdAt` | Date
`updatedAt` | Date
`isPrimary` | boolean

## Example

```typescript
import type { CustomerAddressModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "customerIdentifier": null,
  "addressLineOne": null,
  "addressLineTwo": null,
  "localGovtArea": null,
  "cityOrTown": null,
  "state": null,
  "createdAt": null,
  "updatedAt": null,
  "isPrimary": null,
} satisfies CustomerAddressModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CustomerAddressModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


