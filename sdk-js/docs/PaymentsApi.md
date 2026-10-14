# PaymentsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**confirmPayment**](PaymentsApi.md#confirmpaymentoperation) | **POST** /payments/confirm | Confirm payment |
| [**handlePaymentWebhook**](PaymentsApi.md#handlepaymentwebhook) | **POST** /payments/webhook | Handle payment webhook |
| [**initializePayment**](PaymentsApi.md#initializepaymentoperation) | **POST** /payments/initialize | Initialize payment |



## confirmPayment

> ConfirmPaymentResponse confirmPayment(confirmPaymentRequest)

Confirm payment

### Example

```ts
import {
  Configuration,
  PaymentsApi,
} from '@opeolluwa/asterisk';
import type { ConfirmPaymentOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new PaymentsApi(config);

  const body = {
    // ConfirmPaymentRequest
    confirmPaymentRequest: ...,
  } satisfies ConfirmPaymentOperationRequest;

  try {
    const data = await api.confirmPayment(body);
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
| **confirmPaymentRequest** | [ConfirmPaymentRequest](ConfirmPaymentRequest.md) |  | |

### Return type

[**ConfirmPaymentResponse**](ConfirmPaymentResponse.md)

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


## handlePaymentWebhook

> handlePaymentWebhook()

Handle payment webhook

### Example

```ts
import {
  Configuration,
  PaymentsApi,
} from '@opeolluwa/asterisk';
import type { HandlePaymentWebhookRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new PaymentsApi();

  try {
    const data = await api.handlePaymentWebhook();
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

`void` (Empty response body)

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


## initializePayment

> any initializePayment(initializePaymentRequest)

Initialize payment

### Example

```ts
import {
  Configuration,
  PaymentsApi,
} from '@opeolluwa/asterisk';
import type { InitializePaymentOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new PaymentsApi(config);

  const body = {
    // InitializePaymentRequest
    initializePaymentRequest: ...,
  } satisfies InitializePaymentOperationRequest;

  try {
    const data = await api.initializePayment(body);
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
| **initializePaymentRequest** | [InitializePaymentRequest](InitializePaymentRequest.md) |  | |

### Return type

**any**

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

