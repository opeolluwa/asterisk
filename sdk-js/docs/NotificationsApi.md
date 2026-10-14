# NotificationsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**countNotifications**](NotificationsApi.md#countnotifications) | **GET** /notifications/count | Count notifications |
| [**deleteNotificationsByIdentifier**](NotificationsApi.md#deletenotificationsbyidentifier) | **DELETE** /notifications/{identifier} | Delete notifications by identifier |
| [**findAllNotifications**](NotificationsApi.md#findallnotifications) | **GET** /notifications | Find all notifications |
| [**findNotificationDeliveries**](NotificationsApi.md#findnotificationdeliveries) | **GET** /notifications/{identifier}/deliveries | Find notification deliveries |
| [**findNotificationPreferences**](NotificationsApi.md#findnotificationpreferences) | **GET** /notification-preferences | Find notification preferences |
| [**findNotificationsByIdentifier**](NotificationsApi.md#findnotificationsbyidentifier) | **GET** /notifications/{identifier} | Find notifications by identifier |
| [**markAllNotificationsRead**](NotificationsApi.md#markallnotificationsread) | **PUT** /notifications/read-all | Mark all notifications read |
| [**updateNotificationPreferences**](NotificationsApi.md#updatenotificationpreferencesoperation) | **PUT** /notification-preferences | Update notification preferences |
| [**updateNotificationsByIdentifier**](NotificationsApi.md#updatenotificationsbyidentifier) | **PUT** /notifications/{identifier} | Update notifications by identifier |



## countNotifications

> CountResponse countNotifications()

Count notifications

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { CountNotificationsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  try {
    const data = await api.countNotifications();
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


## deleteNotificationsByIdentifier

> MessageResponse deleteNotificationsByIdentifier(identifier)

Delete notifications by identifier

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { DeleteNotificationsByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteNotificationsByIdentifierRequest;

  try {
    const data = await api.deleteNotificationsByIdentifier(body);
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


## findAllNotifications

> FindAllNotifications200Response findAllNotifications(page, pageSize, eventType, readState, unreadOnly, includeArchived)

Find all notifications

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { FindAllNotificationsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  const body = {
    // number (optional)
    page: 56,
    // number (optional)
    pageSize: 56,
    // string (optional)
    eventType: eventType_example,
    // any (optional)
    readState: ...,
    // boolean (optional)
    unreadOnly: true,
    // boolean (optional)
    includeArchived: true,
  } satisfies FindAllNotificationsRequest;

  try {
    const data = await api.findAllNotifications(body);
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
| **page** | `number` |  | [Optional] [Defaults to `undefined`] |
| **pageSize** | `number` |  | [Optional] [Defaults to `undefined`] |
| **eventType** | `string` |  | [Optional] [Defaults to `undefined`] |
| **readState** | `any` |  | [Optional] [Defaults to `undefined`] |
| **unreadOnly** | `boolean` |  | [Optional] [Defaults to `undefined`] |
| **includeArchived** | `boolean` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**FindAllNotifications200Response**](FindAllNotifications200Response.md)

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


## findNotificationDeliveries

> object findNotificationDeliveries(identifier)

Find notification deliveries

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { FindNotificationDeliveriesRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindNotificationDeliveriesRequest;

  try {
    const data = await api.findNotificationDeliveries(body);
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

**object**

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


## findNotificationPreferences

> Array&lt;NotificationPreferencesModel&gt; findNotificationPreferences()

Find notification preferences

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { FindNotificationPreferencesRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  try {
    const data = await api.findNotificationPreferences();
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


## findNotificationsByIdentifier

> NotificationView findNotificationsByIdentifier(identifier)

Find notifications by identifier

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { FindNotificationsByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindNotificationsByIdentifierRequest;

  try {
    const data = await api.findNotificationsByIdentifier(body);
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


## markAllNotificationsRead

> object markAllNotificationsRead()

Mark all notifications read

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { MarkAllNotificationsReadRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  try {
    const data = await api.markAllNotificationsRead();
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

**object**

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


## updateNotificationPreferences

> Array&lt;NotificationPreferencesModel&gt; updateNotificationPreferences(updateNotificationPreferencesRequest)

Update notification preferences

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { UpdateNotificationPreferencesOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  const body = {
    // UpdateNotificationPreferencesRequest
    updateNotificationPreferencesRequest: ...,
  } satisfies UpdateNotificationPreferencesOperationRequest;

  try {
    const data = await api.updateNotificationPreferences(body);
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


## updateNotificationsByIdentifier

> NotificationView updateNotificationsByIdentifier(identifier, updateNotificationsCommand)

Update notifications by identifier

### Example

```ts
import {
  Configuration,
  NotificationsApi,
} from '@opeolluwa/asterisk';
import type { UpdateNotificationsByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new NotificationsApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateNotificationsCommand
    updateNotificationsCommand: ...,
  } satisfies UpdateNotificationsByIdentifierRequest;

  try {
    const data = await api.updateNotificationsByIdentifier(body);
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

