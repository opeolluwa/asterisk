# CustomersApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**countCustomers**](CustomersApi.md#countcustomers) | **GET** /customers/count | Count customers |
| [**deactivateCustomer**](CustomersApi.md#deactivatecustomer) | **PUT** /customers/{identifier}/deactivate | Deactivate customer |
| [**deactivateCustomerByIdentifier**](CustomersApi.md#deactivatecustomerbyidentifier) | **DELETE** /customers/{identifier} | Deactivate customer by identifier |
| [**findAllCustomers**](CustomersApi.md#findallcustomers) | **GET** /customers | Find all customers |
| [**findCustomerByIdentifier**](CustomersApi.md#findcustomerbyidentifier) | **GET** /customers/{identifier} | Find customer by identifier |
| [**reactivateCustomer**](CustomersApi.md#reactivatecustomer) | **PUT** /customers/{identifier}/reactivate | Reactivate customer |



## countCustomers

> CountResponse countCustomers()

Count customers

### Example

```ts
import {
  Configuration,
  CustomersApi,
} from '@opeolluwa/asterisk';
import type { CountCustomersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomersApi(config);

  try {
    const data = await api.countCustomers();
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


## deactivateCustomer

> CustomerProfile deactivateCustomer(identifier)

Deactivate customer

### Example

```ts
import {
  Configuration,
  CustomersApi,
} from '@opeolluwa/asterisk';
import type { DeactivateCustomerRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeactivateCustomerRequest;

  try {
    const data = await api.deactivateCustomer(body);
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

[**CustomerProfile**](CustomerProfile.md)

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


## deactivateCustomerByIdentifier

> MessageResponse deactivateCustomerByIdentifier(identifier)

Deactivate customer by identifier

### Example

```ts
import {
  Configuration,
  CustomersApi,
} from '@opeolluwa/asterisk';
import type { DeactivateCustomerByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeactivateCustomerByIdentifierRequest;

  try {
    const data = await api.deactivateCustomerByIdentifier(body);
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


## findAllCustomers

> Array&lt;CustomerProfile&gt; findAllCustomers()

Find all customers

### Example

```ts
import {
  Configuration,
  CustomersApi,
} from '@opeolluwa/asterisk';
import type { FindAllCustomersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomersApi(config);

  try {
    const data = await api.findAllCustomers();
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

[**Array&lt;CustomerProfile&gt;**](CustomerProfile.md)

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


## findCustomerByIdentifier

> CustomerProfile findCustomerByIdentifier(identifier)

Find customer by identifier

### Example

```ts
import {
  Configuration,
  CustomersApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCustomerByIdentifierRequest;

  try {
    const data = await api.findCustomerByIdentifier(body);
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

[**CustomerProfile**](CustomerProfile.md)

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


## reactivateCustomer

> CustomerProfile reactivateCustomer(identifier)

Reactivate customer

### Example

```ts
import {
  Configuration,
  CustomersApi,
} from '@opeolluwa/asterisk';
import type { ReactivateCustomerRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomersApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies ReactivateCustomerRequest;

  try {
    const data = await api.reactivateCustomer(body);
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

[**CustomerProfile**](CustomerProfile.md)

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

