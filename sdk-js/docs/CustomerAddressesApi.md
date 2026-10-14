# CustomerAddressesApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**countCustomerAddress**](CustomerAddressesApi.md#countcustomeraddress) | **GET** /customer/address/count | Count customer address |
| [**createCustomerAddress**](CustomerAddressesApi.md#createcustomeraddressoperation) | **POST** /customer/address | Create customer address |
| [**deleteCustomerAddressByIdentifier**](CustomerAddressesApi.md#deletecustomeraddressbyidentifier) | **DELETE** /customer/address/{identifier} | Delete customer address by identifier |
| [**findAllCustomerAddress**](CustomerAddressesApi.md#findallcustomeraddress) | **GET** /customer/address | Find all customer address |
| [**findCustomerAddressByIdentifier**](CustomerAddressesApi.md#findcustomeraddressbyidentifier) | **GET** /customer/address/{identifier} | Find customer address by identifier |
| [**updateCustomerAddressByIdentifier**](CustomerAddressesApi.md#updatecustomeraddressbyidentifier) | **PUT** /customer/address/{identifier} | Update customer address by identifier |



## countCustomerAddress

> CountResponse countCustomerAddress()

Count customer address

### Example

```ts
import {
  Configuration,
  CustomerAddressesApi,
} from '@opeolluwa/asterisk';
import type { CountCustomerAddressRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAddressesApi(config);

  try {
    const data = await api.countCustomerAddress();
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


## createCustomerAddress

> CustomerAddressModel createCustomerAddress(createCustomerAddressRequest)

Create customer address

### Example

```ts
import {
  Configuration,
  CustomerAddressesApi,
} from '@opeolluwa/asterisk';
import type { CreateCustomerAddressOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAddressesApi(config);

  const body = {
    // CreateCustomerAddressRequest
    createCustomerAddressRequest: ...,
  } satisfies CreateCustomerAddressOperationRequest;

  try {
    const data = await api.createCustomerAddress(body);
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
| **createCustomerAddressRequest** | [CreateCustomerAddressRequest](CreateCustomerAddressRequest.md) |  | |

### Return type

[**CustomerAddressModel**](CustomerAddressModel.md)

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


## deleteCustomerAddressByIdentifier

> MessageResponse deleteCustomerAddressByIdentifier(identifier)

Delete customer address by identifier

### Example

```ts
import {
  Configuration,
  CustomerAddressesApi,
} from '@opeolluwa/asterisk';
import type { DeleteCustomerAddressByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAddressesApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteCustomerAddressByIdentifierRequest;

  try {
    const data = await api.deleteCustomerAddressByIdentifier(body);
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


## findAllCustomerAddress

> Array&lt;CustomerAddressModel&gt; findAllCustomerAddress()

Find all customer address

### Example

```ts
import {
  Configuration,
  CustomerAddressesApi,
} from '@opeolluwa/asterisk';
import type { FindAllCustomerAddressRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAddressesApi(config);

  try {
    const data = await api.findAllCustomerAddress();
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

[**Array&lt;CustomerAddressModel&gt;**](CustomerAddressModel.md)

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


## findCustomerAddressByIdentifier

> CustomerAddressModel findCustomerAddressByIdentifier(identifier)

Find customer address by identifier

### Example

```ts
import {
  Configuration,
  CustomerAddressesApi,
} from '@opeolluwa/asterisk';
import type { FindCustomerAddressByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAddressesApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindCustomerAddressByIdentifierRequest;

  try {
    const data = await api.findCustomerAddressByIdentifier(body);
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

[**CustomerAddressModel**](CustomerAddressModel.md)

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


## updateCustomerAddressByIdentifier

> CustomerAddressModel updateCustomerAddressByIdentifier(identifier, updateCustomerAddressCommand)

Update customer address by identifier

### Example

```ts
import {
  Configuration,
  CustomerAddressesApi,
} from '@opeolluwa/asterisk';
import type { UpdateCustomerAddressByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAddressesApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateCustomerAddressCommand
    updateCustomerAddressCommand: ...,
  } satisfies UpdateCustomerAddressByIdentifierRequest;

  try {
    const data = await api.updateCustomerAddressByIdentifier(body);
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
| **updateCustomerAddressCommand** | [UpdateCustomerAddressCommand](UpdateCustomerAddressCommand.md) |  | |

### Return type

[**CustomerAddressModel**](CustomerAddressModel.md)

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

