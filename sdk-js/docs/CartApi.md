# CartApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**addCartItem**](CartApi.md#addcartitemoperation) | **POST** /customer/cart/items | Add cart item |
| [**countCartItems**](CartApi.md#countcartitems) | **GET** /customer/cart/count | Count cart items |
| [**deleteCartItemByIdentifier**](CartApi.md#deletecartitembyidentifier) | **DELETE** /customer/cart/items/{identifier} | Delete cart item by identifier |
| [**findCart**](CartApi.md#findcart) | **GET** /customer/cart | Find cart |
| [**findCartItemByIdentifier**](CartApi.md#findcartitembyidentifier) | **GET** /customer/cart/items/{identifier} | Find cart item by identifier |
| [**syncCart**](CartApi.md#synccart) | **POST** /customer/cart/sync | Sync cart |
| [**updateCartItemByIdentifier**](CartApi.md#updatecartitembyidentifier) | **PUT** /customer/cart/items/{identifier} | Update cart item by identifier |



## addCartItem

> CartItemResponse addCartItem(addCartItemRequest)

Add cart item

Authentication is optional on this endpoint.

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { AddCartItemOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CartApi();

  const body = {
    // AddCartItemRequest
    addCartItemRequest: ...,
  } satisfies AddCartItemOperationRequest;

  try {
    const data = await api.addCartItem(body);
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
| **addCartItemRequest** | [AddCartItemRequest](AddCartItemRequest.md) |  | |

### Return type

[**CartItemResponse**](CartItemResponse.md)

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


## countCartItems

> CountResponse countCartItems()

Count cart items

Authentication is optional on this endpoint.

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { CountCartItemsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CartApi();

  try {
    const data = await api.countCartItems();
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


## deleteCartItemByIdentifier

> MessageResponse deleteCartItemByIdentifier(identifier)

Delete cart item by identifier

Authentication is optional on this endpoint.

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { DeleteCartItemByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CartApi();

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteCartItemByIdentifierRequest;

  try {
    const data = await api.deleteCartItemByIdentifier(body);
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


## findCart

> CartResponse findCart()

Find cart

Authentication is optional on this endpoint.

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { FindCartRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CartApi();

  try {
    const data = await api.findCart();
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

[**CartResponse**](CartResponse.md)

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


## findCartItemByIdentifier

> CartItemResponse findCartItemByIdentifier(identifier)

Find cart item by identifier

Authentication is optional on this endpoint.

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { FindCartItemByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CartApi();

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCartItemByIdentifierRequest;

  try {
    const data = await api.findCartItemByIdentifier(body);
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

[**CartItemResponse**](CartItemResponse.md)

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


## syncCart

> CartResponse syncCart()

Sync cart

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { SyncCartRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CartApi(config);

  try {
    const data = await api.syncCart();
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

[**CartResponse**](CartResponse.md)

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


## updateCartItemByIdentifier

> CartItemResponse updateCartItemByIdentifier(identifier, updateCartItemRequest)

Update cart item by identifier

Authentication is optional on this endpoint.

### Example

```ts
import {
  Configuration,
  CartApi,
} from '@opeolluwa/asterisk';
import type { UpdateCartItemByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CartApi();

  const body = {
    // string
    identifier: identifier_example,
    // UpdateCartItemRequest
    updateCartItemRequest: ...,
  } satisfies UpdateCartItemByIdentifierRequest;

  try {
    const data = await api.updateCartItemByIdentifier(body);
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
| **updateCartItemRequest** | [UpdateCartItemRequest](UpdateCartItemRequest.md) |  | |

### Return type

[**CartItemResponse**](CartItemResponse.md)

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

