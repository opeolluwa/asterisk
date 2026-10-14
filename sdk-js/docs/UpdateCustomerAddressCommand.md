
# UpdateCustomerAddressCommand


## Properties

Name | Type
------------ | -------------
`addressLineOne` | string
`addressLineTwo` | string
`localGovtArea` | string
`cityOrTown` | string
`state` | string
`isPrimary` | boolean

## Example

```typescript
import type { UpdateCustomerAddressCommand } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "addressLineOne": null,
  "addressLineTwo": null,
  "localGovtArea": null,
  "cityOrTown": null,
  "state": null,
  "isPrimary": null,
} satisfies UpdateCustomerAddressCommand

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateCustomerAddressCommand
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


