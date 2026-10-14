
# AppConfigModel


## Properties

Name | Type
------------ | -------------
`appName` | string
`maintenanceMode` | boolean
`supportEmail` | string
`createdAt` | Date
`lastUpdated` | Date
`defaultCurrency` | string
`defaultLanguage` | string
`identifier` | string
`logoUrl` | string

## Example

```typescript
import type { AppConfigModel } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "appName": null,
  "maintenanceMode": null,
  "supportEmail": null,
  "createdAt": null,
  "lastUpdated": null,
  "defaultCurrency": null,
  "defaultLanguage": null,
  "identifier": null,
  "logoUrl": null,
} satisfies AppConfigModel

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AppConfigModel
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


