# ComplaintsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**archiveComplaint**](ComplaintsApi.md#archivecomplaint) | **DELETE** /complaints/{identifier} | Archive complaint |
| [**countComplaints**](ComplaintsApi.md#countcomplaints) | **GET** /complaints/count | Count complaints |
| [**findAllComplaints**](ComplaintsApi.md#findallcomplaints) | **GET** /complaints | Find all complaints |
| [**findComplaintByIdentifier**](ComplaintsApi.md#findcomplaintbyidentifier) | **GET** /complaints/{identifier} | Find complaint by identifier |
| [**findComplaintReplies**](ComplaintsApi.md#findcomplaintreplies) | **GET** /complaints/{identifier}/replies | Find complaint replies |
| [**replyToComplaint**](ComplaintsApi.md#replytocomplaint) | **POST** /complaints/{identifier}/replies | Reply to complaint |
| [**updateComplaint**](ComplaintsApi.md#updatecomplaintoperation) | **PUT** /complaints/{identifier} | Update complaint |



## archiveComplaint

> MessageResponse archiveComplaint(identifier)

Archive complaint

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { ArchiveComplaintRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies ArchiveComplaintRequest;

  try {
    const data = await api.archiveComplaint(body);
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


## countComplaints

> CountResponse countComplaints()

Count complaints

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { CountComplaintsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  try {
    const data = await api.countComplaints();
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


## findAllComplaints

> Array&lt;Array&lt;any&gt;&gt; findAllComplaints()

Find all complaints

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { FindAllComplaintsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  try {
    const data = await api.findAllComplaints();
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

**Array<Array<any>>**

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


## findComplaintByIdentifier

> Array&lt;any&gt; findComplaintByIdentifier(identifier)

Find complaint by identifier

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { FindComplaintByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindComplaintByIdentifierRequest;

  try {
    const data = await api.findComplaintByIdentifier(body);
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

**Array<any>**

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


## findComplaintReplies

> Array&lt;ComplaintRepliesModel&gt; findComplaintReplies(identifier)

Find complaint replies

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { FindComplaintRepliesRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindComplaintRepliesRequest;

  try {
    const data = await api.findComplaintReplies(body);
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

[**Array&lt;ComplaintRepliesModel&gt;**](ComplaintRepliesModel.md)

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


## replyToComplaint

> ComplaintRepliesModel replyToComplaint(identifier, replyComplaintRequest)

Reply to complaint

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { ReplyToComplaintRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // ReplyComplaintRequest
    replyComplaintRequest: ...,
  } satisfies ReplyToComplaintRequest;

  try {
    const data = await api.replyToComplaint(body);
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
| **replyComplaintRequest** | [ReplyComplaintRequest](ReplyComplaintRequest.md) |  | |

### Return type

[**ComplaintRepliesModel**](ComplaintRepliesModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateComplaint

> ComplaintsModel updateComplaint(identifier, updateComplaintRequest)

Update complaint

### Example

```ts
import {
  Configuration,
  ComplaintsApi,
} from '@opeolluwa/asterisk';
import type { UpdateComplaintOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateComplaintRequest
    updateComplaintRequest: ...,
  } satisfies UpdateComplaintOperationRequest;

  try {
    const data = await api.updateComplaint(body);
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
| **updateComplaintRequest** | [UpdateComplaintRequest](UpdateComplaintRequest.md) |  | |

### Return type

[**ComplaintsModel**](ComplaintsModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

