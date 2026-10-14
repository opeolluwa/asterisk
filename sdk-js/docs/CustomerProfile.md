
# CustomerProfile


## Properties

Name | Type
------------ | -------------
`identifier` | string
`email` | string
`firstName` | string
`lastName` | string
`phone` | string
`picture` | string
`addresses` | [Array&lt;CustomerAddressModel&gt;](CustomerAddressModel.md)
`createdAt` | Date
`updatedAt` | Date
`blocked` | boolean
`completedOnboarding` | boolean

## Example

```typescript
import type { CustomerProfile } from '@opeolluwa/asterisk'

// TODO: Update the object below with actual values
const example = {
  "identifier": null,
  "email": null,
  "firstName": null,
  "lastName": null,
  "phone": null,
  "picture": null,
  "addresses": null,
  "createdAt": null,
  "updatedAt": null,
  "blocked": null,
  "completedOnboarding": null,
} satisfies CustomerProfile

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CustomerProfile
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


