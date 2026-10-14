
# DeliveryResponse


## Properties

Name | Type
------------ | -------------
`identifier` | string
`recipientName` | string
`recipientPhone` | string
`addressLineOne` | string
`addressLineTwo` | string
`localGovtArea` | string
`cityOrTown` | string
`state` | string
`deliveryNotes` | string

## Example

```typescript
import type { DeliveryResponse } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "recipientName": null,
  "recipientPhone": null,
  "addressLineOne": null,
  "addressLineTwo": null,
  "localGovtArea": null,
  "cityOrTown": null,
  "state": null,
  "deliveryNotes": null,
} satisfies DeliveryResponse

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as DeliveryResponse
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


