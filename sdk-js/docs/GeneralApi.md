# GeneralApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**archiveCustomerNotificationByIdentifier**](GeneralApi.md#archivecustomernotificationbyidentifier) | **DELETE** /customer/notifications/{identifier} | Archive customer notification by identifier |
| [**countCustomerNotifications**](GeneralApi.md#countcustomernotifications) | **GET** /customer/notifications/count | Count customer notifications |
| [**fetchDashboardMetrics**](GeneralApi.md#fetchdashboardmetrics) | **GET** /metrics | Fetch dashboard metrics |
| [**findCustomerNotificationByIdentifier**](GeneralApi.md#findcustomernotificationbyidentifier) | **GET** /customer/notifications/{identifier} | Find customer notification by identifier |
| [**findCustomerNotificationPreferences**](GeneralApi.md#findcustomernotificationpreferences) | **GET** /customer/notification-preferences | Find customer notification preferences |
| [**findCustomerNotifications**](GeneralApi.md#findcustomernotifications) | **GET** /customer/notifications | Find customer notifications |
| [**updateCustomerNotificationByIdentifier**](GeneralApi.md#updatecustomernotificationbyidentifier) | **PUT** /customer/notifications/{identifier} | Update customer notification by identifier |
| [**updateCustomerNotificationPreferences**](GeneralApi.md#updatecustomernotificationpreferences) | **PUT** /customer/notification-preferences | Update customer notification preferences |



## archiveCustomerNotificationByIdentifier

> MessageResponse archiveCustomerNotificationByIdentifier(identifier)

Archive customer notification by identifier

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { ArchiveCustomerNotificationByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies ArchiveCustomerNotificationByIdentifierRequest;

  try {
    const data = await api.archiveCustomerNotificationByIdentifier(body);
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


## countCustomerNotifications

> CountResponse countCustomerNotifications()

Count customer notifications

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { CountCustomerNotificationsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  try {
    const data = await api.countCustomerNotifications();
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


## fetchDashboardMetrics

> DashboardMetricsResponse fetchDashboardMetrics(months)

Fetch dashboard metrics

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { FetchDashboardMetricsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  const body = {
    // number (optional)
    months: 56,
  } satisfies FetchDashboardMetricsRequest;

  try {
    const data = await api.fetchDashboardMetrics(body);
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
| **months** | `number` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**DashboardMetricsResponse**](DashboardMetricsResponse.md)

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


## findCustomerNotificationByIdentifier

> NotificationView findCustomerNotificationByIdentifier(identifier)

Find customer notification by identifier

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerNotificationByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCustomerNotificationByIdentifierRequest;

  try {
    const data = await api.findCustomerNotificationByIdentifier(body);
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

[**NotificationView**](NotificationView.md)

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


## findCustomerNotificationPreferences

> Array&lt;NotificationPreferencesModel&gt; findCustomerNotificationPreferences()

Find customer notification preferences

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerNotificationPreferencesRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  try {
    const data = await api.findCustomerNotificationPreferences();
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

[**Array&lt;NotificationPreferencesModel&gt;**](NotificationPreferencesModel.md)

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


## findCustomerNotifications

> Array&lt;NotificationView&gt; findCustomerNotifications()

Find customer notifications

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerNotificationsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  try {
    const data = await api.findCustomerNotifications();
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

[**Array&lt;NotificationView&gt;**](NotificationView.md)

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


## updateCustomerNotificationByIdentifier

> NotificationView updateCustomerNotificationByIdentifier(identifier, updateNotificationsCommand)

Update customer notification by identifier

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { UpdateCustomerNotificationByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateNotificationsCommand
    updateNotificationsCommand: ...,
  } satisfies UpdateCustomerNotificationByIdentifierRequest;

  try {
    const data = await api.updateCustomerNotificationByIdentifier(body);
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
| **updateNotificationsCommand** | [UpdateNotificationsCommand](UpdateNotificationsCommand.md) |  | |

### Return type

[**NotificationView**](NotificationView.md)

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


## updateCustomerNotificationPreferences

> Array&lt;NotificationPreferencesModel&gt; updateCustomerNotificationPreferences(updateNotificationPreferencesRequest)

Update customer notification preferences

### Example

```ts
import {
  Configuration,
  GeneralApi,
} from '@opeolluwa/asterisk';
import type { UpdateCustomerNotificationPreferencesRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new GeneralApi(config);

  const body = {
    // UpdateNotificationPreferencesRequest
    updateNotificationPreferencesRequest: ...,
  } satisfies UpdateCustomerNotificationPreferencesRequest;

  try {
    const data = await api.updateCustomerNotificationPreferences(body);
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
| **updateNotificationPreferencesRequest** | [UpdateNotificationPreferencesRequest](UpdateNotificationPreferencesRequest.md) |  | |

### Return type

[**Array&lt;NotificationPreferencesModel&gt;**](NotificationPreferencesModel.md)

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

