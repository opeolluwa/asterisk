# UploadsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**countUploads**](UploadsApi.md#countuploads) | **GET** /uploads/count | Count uploads |
| [**createUpload**](UploadsApi.md#createupload) | **POST** /uploads | Create upload |
| [**deleteUpload**](UploadsApi.md#deleteupload) | **DELETE** /uploads/{identifier} | Delete upload |
| [**findAllUploads**](UploadsApi.md#findalluploads) | **GET** /uploads | Find all uploads |
| [**findStarredUploads**](UploadsApi.md#findstarreduploads) | **GET** /uploads/starred | Find starred uploads |
| [**findUploadByIdentifier**](UploadsApi.md#finduploadbyidentifier) | **GET** /uploads/{identifier} | Find upload by identifier |
| [**updateUpload**](UploadsApi.md#updateuploadoperation) | **PUT** /uploads/{identifier} | Update upload |



## countUploads

> CountResponse countUploads()

Count uploads

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { CountUploadsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  try {
    const data = await api.countUploads();
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


## createUpload

> UploadsModel createUpload(file, name, fileType, starred)

Create upload

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { CreateUploadRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  const body = {
    // Blob
    file: BINARY_DATA_HERE,
    // string
    name: name_example,
    // string (optional)
    fileType: fileType_example,
    // boolean (optional)
    starred: true,
  } satisfies CreateUploadRequest;

  try {
    const data = await api.createUpload(body);
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
| **file** | `Blob` |  | [Defaults to `undefined`] |
| **name** | `string` |  | [Defaults to `undefined`] |
| **fileType** | `string` |  | [Optional] [Defaults to `undefined`] |
| **starred** | `boolean` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**UploadsModel**](UploadsModel.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: `multipart/form-data`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteUpload

> MessageResponse deleteUpload(identifier)

Delete upload

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { DeleteUploadRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteUploadRequest;

  try {
    const data = await api.deleteUpload(body);
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


## findAllUploads

> Array&lt;UploadsModel&gt; findAllUploads()

Find all uploads

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { FindAllUploadsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  try {
    const data = await api.findAllUploads();
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

[**Array&lt;UploadsModel&gt;**](UploadsModel.md)

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


## findStarredUploads

> Array&lt;UploadsModel&gt; findStarredUploads()

Find starred uploads

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { FindStarredUploadsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  try {
    const data = await api.findStarredUploads();
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

[**Array&lt;UploadsModel&gt;**](UploadsModel.md)

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


## findUploadByIdentifier

> UploadsModel findUploadByIdentifier(identifier)

Find upload by identifier

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { FindUploadByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindUploadByIdentifierRequest;

  try {
    const data = await api.findUploadByIdentifier(body);
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

[**UploadsModel**](UploadsModel.md)

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


## updateUpload

> UploadsModel updateUpload(identifier, updateUploadRequest)

Update upload

### Example

```ts
import {
  Configuration,
  UploadsApi,
} from '@opeolluwa/asterisk';
import type { UpdateUploadOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UploadsApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateUploadRequest
    updateUploadRequest: ...,
  } satisfies UpdateUploadOperationRequest;

  try {
    const data = await api.updateUpload(body);
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
| **updateUploadRequest** | [UpdateUploadRequest](UpdateUploadRequest.md) |  | |

### Return type

[**UploadsModel**](UploadsModel.md)

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

