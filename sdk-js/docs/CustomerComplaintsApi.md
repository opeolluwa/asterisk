# CustomerComplaintsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createCustomerComplaint**](CustomerComplaintsApi.md#createcustomercomplaint) | **POST** /customer/complaint | Create customer complaint |
| [**findCustomerComplaintByIdentifier**](CustomerComplaintsApi.md#findcustomercomplaintbyidentifier) | **GET** /customer/complaint/{identifier} | Find customer complaint by identifier |
| [**findCustomerComplaintReplies**](CustomerComplaintsApi.md#findcustomercomplaintreplies) | **GET** /customer/complaint/{identifier}/replies | Find customer complaint replies |
| [**findCustomerComplaints**](CustomerComplaintsApi.md#findcustomercomplaints) | **GET** /customer/complaint | Find customer complaints |
| [**replyToCustomerComplaint**](CustomerComplaintsApi.md#replytocustomercomplaint) | **POST** /customer/complaint/{identifier}/replies | Reply to customer complaint |



## createCustomerComplaint

> ComplaintsModel createCustomerComplaint(createComplaintRequest)

Create customer complaint

### Example

```ts
import {
  Configuration,
  CustomerComplaintsApi,
} from '@opeolluwa/asterisk';
import type { CreateCustomerComplaintRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerComplaintsApi(config);

  const body = {
    // CreateComplaintRequest
    createComplaintRequest: ...,
  } satisfies CreateCustomerComplaintRequest;

  try {
    const data = await api.createCustomerComplaint(body);
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
| **createComplaintRequest** | [CreateComplaintRequest](CreateComplaintRequest.md) |  | |

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


## findCustomerComplaintByIdentifier

> ComplaintsModel findCustomerComplaintByIdentifier(identifier)

Find customer complaint by identifier

### Example

```ts
import {
  Configuration,
  CustomerComplaintsApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerComplaintByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCustomerComplaintByIdentifierRequest;

  try {
    const data = await api.findCustomerComplaintByIdentifier(body);
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

[**ComplaintsModel**](ComplaintsModel.md)

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


## findCustomerComplaintReplies

> Array&lt;ComplaintRepliesModel&gt; findCustomerComplaintReplies(identifier)

Find customer complaint replies

### Example

```ts
import {
  Configuration,
  CustomerComplaintsApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerComplaintRepliesRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCustomerComplaintRepliesRequest;

  try {
    const data = await api.findCustomerComplaintReplies(body);
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


## findCustomerComplaints

> Array&lt;ComplaintsModel&gt; findCustomerComplaints()

Find customer complaints

### Example

```ts
import {
  Configuration,
  CustomerComplaintsApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerComplaintsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerComplaintsApi(config);

  try {
    const data = await api.findCustomerComplaints();
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

[**Array&lt;ComplaintsModel&gt;**](ComplaintsModel.md)

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


## replyToCustomerComplaint

> ComplaintRepliesModel replyToCustomerComplaint(identifier, replyComplaintRequest)

Reply to customer complaint

### Example

```ts
import {
  Configuration,
  CustomerComplaintsApi,
} from '@opeolluwa/asterisk';
import type { ReplyToCustomerComplaintRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerComplaintsApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // ReplyComplaintRequest
    replyComplaintRequest: ...,
  } satisfies ReplyToCustomerComplaintRequest;

  try {
    const data = await api.replyToCustomerComplaint(body);
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

