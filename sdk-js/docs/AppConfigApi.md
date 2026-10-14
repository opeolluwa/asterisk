# AppConfigApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**fetchAppConfig**](AppConfigApi.md#fetchappconfig) | **GET** /config | Fetch app config |
| [**updateAppConfig**](AppConfigApi.md#updateappconfigoperation) | **PUT** /config | Update app config |



## fetchAppConfig

> AppConfigModel fetchAppConfig()

Fetch app config

### Example

```ts
import {
  Configuration,
  AppConfigApi,
} from '@opeolluwa/asterisk';
import type { FetchAppConfigRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AppConfigApi(config);

  try {
    const data = await api.fetchAppConfig();
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

[**AppConfigModel**](AppConfigModel.md)

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


## updateAppConfig

> AppConfigModel updateAppConfig(updateAppConfigRequest)

Update app config

### Example

```ts
import {
  Configuration,
  AppConfigApi,
} from '@opeolluwa/asterisk';
import type { UpdateAppConfigOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AppConfigApi(config);

  const body = {
    // UpdateAppConfigRequest
    updateAppConfigRequest: ...,
  } satisfies UpdateAppConfigOperationRequest;

  try {
    const data = await api.updateAppConfig(body);
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
| **updateAppConfigRequest** | [UpdateAppConfigRequest](UpdateAppConfigRequest.md) |  | |

### Return type

[**AppConfigModel**](AppConfigModel.md)

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

