# InvitationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**acceptInvitation**](InvitationsApi.md#acceptinvitationoperation) | **PUT** /invitations/{identifier}/accept | Accept invitation |
| [**blockInvitation**](InvitationsApi.md#blockinvitation) | **PUT** /invitations/{identifier}/block | Block invitation |
| [**countInvitations**](InvitationsApi.md#countinvitations) | **GET** /invitations/count | Count invitations |
| [**createInvitation**](InvitationsApi.md#createinvitationoperation) | **POST** /invitations | Create invitation |
| [**deleteInvitation**](InvitationsApi.md#deleteinvitation) | **DELETE** /invitations/{identifier} | Delete invitation |
| [**findAllInvitations**](InvitationsApi.md#findallinvitations) | **GET** /invitations | Find all invitations |
| [**findInvitationByIdentifier**](InvitationsApi.md#findinvitationbyidentifier) | **GET** /invitations/{identifier} | Find invitation by identifier |
| [**previewInvitation**](InvitationsApi.md#previewinvitation) | **GET** /invitations/{identifier}/preview | Preview invitation |
| [**resendInvitation**](InvitationsApi.md#resendinvitation) | **PUT** /invitations/{identifier}/resend | Resend invitation |



## acceptInvitation

> InvitationModel acceptInvitation(identifier, acceptInvitationRequest)

Accept invitation

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { AcceptInvitationOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new InvitationsApi();

  const body = {
    // string
    identifier: identifier_example,
    // AcceptInvitationRequest
    acceptInvitationRequest: ...,
  } satisfies AcceptInvitationOperationRequest;

  try {
    const data = await api.acceptInvitation(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **identifier** | `string` |  | [Defaults to `undefined`] |
| **acceptInvitationRequest** | [AcceptInvitationRequest](AcceptInvitationRequest.md) |  | |

### Return type

[**InvitationModel**](InvitationModel.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## blockInvitation

> InvitationModel blockInvitation(identifier)

Block invitation

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { BlockInvitationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies BlockInvitationRequest;

  try {
    const data = await api.blockInvitation(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **identifier** | `string` |  | [Defaults to `undefined`] |

### Return type

[**InvitationModel**](InvitationModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## countInvitations

> CountResponse countInvitations()

Count invitations

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { CountInvitationsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  try {
    const data = await api.countInvitations();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**CountResponse**](CountResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createInvitation

> InvitationModel createInvitation(createInvitationRequest)

Create invitation

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { CreateInvitationOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  const body = {
    // CreateInvitationRequest
    createInvitationRequest: ...,
  } satisfies CreateInvitationOperationRequest;

  try {
    const data = await api.createInvitation(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **createInvitationRequest** | [CreateInvitationRequest](CreateInvitationRequest.md) |  | |

### Return type

[**InvitationModel**](InvitationModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteInvitation

> MessageResponse deleteInvitation(identifier)

Delete invitation

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { DeleteInvitationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteInvitationRequest;

  try {
    const data = await api.deleteInvitation(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **identifier** | `string` |  | [Defaults to `undefined`] |

### Return type

[**MessageResponse**](MessageResponse.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## findAllInvitations

> Array&lt;InvitationModel&gt; findAllInvitations()

Find all invitations

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { FindAllInvitationsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  try {
    const data = await api.findAllInvitations();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**Array&lt;InvitationModel&gt;**](InvitationModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## findInvitationByIdentifier

> InvitationModel findInvitationByIdentifier(identifier)

Find invitation by identifier

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { FindInvitationByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindInvitationByIdentifierRequest;

  try {
    const data = await api.findInvitationByIdentifier(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **identifier** | `string` |  | [Defaults to `undefined`] |

### Return type

[**InvitationModel**](InvitationModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## previewInvitation

> InvitationPreview previewInvitation(identifier)

Preview invitation

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { PreviewInvitationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new InvitationsApi();

  const body = {
    // string
    identifier: identifier_example,
  } satisfies PreviewInvitationRequest;

  try {
    const data = await api.previewInvitation(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **identifier** | `string` |  | [Defaults to `undefined`] |

### Return type

[**InvitationPreview**](InvitationPreview.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## resendInvitation

> InvitationModel resendInvitation(identifier)

Resend invitation

### Example

```ts
import {
  Configuration,
  InvitationsApi,
} from '@opeolluwa/asterisk';
import type { ResendInvitationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new InvitationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies ResendInvitationRequest;

  try {
    const data = await api.resendInvitation(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **identifier** | `string` |  | [Defaults to `undefined`] |

### Return type

[**InvitationModel**](InvitationModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

