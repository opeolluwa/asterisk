
# UpdateAppConfigRequest


## Properties

Name | Type
------------ | -------------
`appName` | string
`supportEmail` | string
`defaultCurrency` | string
`defaultLanguage` | string
`maintenanceMode` | boolean
`logoUrl` | string

## Example

```typescript
import type { UpdateAppConfigRequest } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "appName": null,
  "supportEmail": null,
  "defaultCurrency": null,
  "defaultLanguage": null,
  "maintenanceMode": null,
  "logoUrl": null,
} satisfies UpdateAppConfigRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UpdateAppConfigRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


