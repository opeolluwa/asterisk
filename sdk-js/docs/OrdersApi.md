# OrdersApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**countOrders**](OrdersApi.md#countorders) | **GET** /orders/count | Count orders |
| [**deleteOrderItemByIdentifier**](OrdersApi.md#deleteorderitembyidentifier) | **DELETE** /orders/{identifier}/items/{item_identifier} | Delete order item by identifier |
| [**deleteOrdersByIdentifier**](OrdersApi.md#deleteordersbyidentifier) | **DELETE** /orders/{identifier} | Delete orders by identifier |
| [**findAllOrders**](OrdersApi.md#findallorders) | **GET** /orders | Find all orders |
| [**findOrdersByIdentifier**](OrdersApi.md#findordersbyidentifier) | **GET** /orders/{identifier} | Find orders by identifier |
| [**updateOrderItemByIdentifier**](OrdersApi.md#updateorderitembyidentifier) | **PUT** /orders/{identifier}/items/{item_identifier} | Update order item by identifier |
| [**updateOrdersByIdentifier**](OrdersApi.md#updateordersbyidentifier) | **PUT** /orders/{identifier} | Update orders by identifier |



## countOrders

> CountResponse countOrders()

Count orders

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { CountOrdersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  try {
    const data = await api.countOrders();
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


## deleteOrderItemByIdentifier

> MessageResponse deleteOrderItemByIdentifier(identifier, itemIdentifier)

Delete order item by identifier

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { DeleteOrderItemByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // string
    itemIdentifier: itemIdentifier_example,
  } satisfies DeleteOrderItemByIdentifierRequest;

  try {
    const data = await api.deleteOrderItemByIdentifier(body);
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
| **itemIdentifier** | `string` |  | [Defaults to `undefined`] |

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


## deleteOrdersByIdentifier

> MessageResponse deleteOrdersByIdentifier(identifier)

Delete orders by identifier

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { DeleteOrdersByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteOrdersByIdentifierRequest;

  try {
    const data = await api.deleteOrdersByIdentifier(body);
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


## findAllOrders

> any findAllOrders(page, pageSize, status)

Find all orders

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { FindAllOrdersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  const body = {
    // number (optional)
    page: 56,
    // number (optional)
    pageSize: 56,
    // OrderStatus (optional)
    status: ...,
  } satisfies FindAllOrdersRequest;

  try {
    const data = await api.findAllOrders(body);
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
| **status** | `OrderStatus` |  | [Optional] [Defaults to `undefined`] [Enum: cancelled, fulfilled, pending, paid, conflicted] |

### Return type

**any**

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


## findOrdersByIdentifier

> OrderResponse findOrdersByIdentifier(identifier)

Find orders by identifier

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { FindOrdersByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindOrdersByIdentifierRequest;

  try {
    const data = await api.findOrdersByIdentifier(body);
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


## updateOrderItemByIdentifier

> OrderItemResponse updateOrderItemByIdentifier(identifier, itemIdentifier, placeOrderItem)

Update order item by identifier

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { UpdateOrderItemByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // string
    itemIdentifier: itemIdentifier_example,
    // PlaceOrderItem
    placeOrderItem: ...,
  } satisfies UpdateOrderItemByIdentifierRequest;

  try {
    const data = await api.updateOrderItemByIdentifier(body);
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
| **itemIdentifier** | `string` |  | [Defaults to `undefined`] |
| **placeOrderItem** | [PlaceOrderItem](PlaceOrderItem.md) |  | |

### Return type

[**OrderItemResponse**](OrderItemResponse.md)

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


## updateOrdersByIdentifier

> OrdersModel updateOrdersByIdentifier(identifier, updateOrderRequest)

Update orders by identifier

### Example

```ts
import {
  Configuration,
  OrdersApi,
} from '@opeolluwa/asterisk';
import type { UpdateOrdersByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OrdersApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateOrderRequest
    updateOrderRequest: ...,
  } satisfies UpdateOrdersByIdentifierRequest;

  try {
    const data = await api.updateOrdersByIdentifier(body);
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
| **updateOrderRequest** | [UpdateOrderRequest](UpdateOrderRequest.md) |  | |

### Return type

[**OrdersModel**](OrdersModel.md)

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

