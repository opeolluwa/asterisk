# CustomerOrdersApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**checkoutCustomerOrder**](CustomerOrdersApi.md#checkoutcustomerorder) | **POST** /customer/orders/checkout | Checkout customer order |
| [**createCustomerOrder**](CustomerOrdersApi.md#createcustomerorder) | **POST** /customer/orders | Create customer order |
| [**findCustomerOrderByIdentifier**](CustomerOrdersApi.md#findcustomerorderbyidentifier) | **GET** /customer/orders/{identifier} | Find customer order by identifier |
| [**findCustomerOrders**](CustomerOrdersApi.md#findcustomerorders) | **GET** /customer/orders | Find customer orders |



## checkoutCustomerOrder

> any checkoutCustomerOrder(checkoutRequest)

Checkout customer order

### Example

```ts
import {
  Configuration,
  CustomerOrdersApi,
} from '@opeolluwa/asterisk';
import type { CheckoutCustomerOrderRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerOrdersApi(config);

  const body = {
    // CheckoutRequest
    checkoutRequest: ...,
  } satisfies CheckoutCustomerOrderRequest;

  try {
    const data = await api.checkoutCustomerOrder(body);
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
| **checkoutRequest** | [CheckoutRequest](CheckoutRequest.md) |  | |

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


## createCustomerOrder

> OrderResponse createCustomerOrder(createOrdersRequest)

Create customer order

### Example

```ts
import {
  Configuration,
  CustomerOrdersApi,
} from '@opeolluwa/asterisk';
import type { CreateCustomerOrderRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerOrdersApi(config);

  const body = {
    // CreateOrdersRequest
    createOrdersRequest: ...,
  } satisfies CreateCustomerOrderRequest;

  try {
    const data = await api.createCustomerOrder(body);
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
| **createOrdersRequest** | [CreateOrdersRequest](CreateOrdersRequest.md) |  | |

### Return type

[**OrderResponse**](OrderResponse.md)

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


## findCustomerOrderByIdentifier

> OrderResponse findCustomerOrderByIdentifier(identifier)

Find customer order by identifier

### Example

```ts
import {
  Configuration,
  CustomerOrdersApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerOrderByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerOrdersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCustomerOrderByIdentifierRequest;

  try {
    const data = await api.findCustomerOrderByIdentifier(body);
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

[**OrderResponse**](OrderResponse.md)

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


## findCustomerOrders

> Array&lt;OrderResponse&gt; findCustomerOrders(status)

Find customer orders

### Example

```ts
import {
  Configuration,
  CustomerOrdersApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerOrdersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerOrdersApi(config);

  const body = {
    // OrderStatus (optional)
    status: ...,
  } satisfies FindCustomerOrdersRequest;

  try {
    const data = await api.findCustomerOrders(body);
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
| **status** | `OrderStatus` |  | [Optional] [Defaults to `undefined`] [Enum: cancelled, fulfilled, pending, paid, conflicted] |

### Return type

[**Array&lt;OrderResponse&gt;**](OrderResponse.md)

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

