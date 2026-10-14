# ProductsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createProduct**](ProductsApi.md#createproduct) | **POST** /products | Create product |
| [**findAllProducts**](ProductsApi.md#findallproducts) | **GET** /products | Find all products |
| [**findProduct**](ProductsApi.md#findproduct) | **GET** /products/{identifier} | Find product |



## createProduct

> ProductsModel createProduct(name, description, picture, category)

Create product

### Example

```ts
import {
  Configuration,
  ProductsApi,
} from '@opeolluwa/asterisk';
import type { CreateProductRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ProductsApi(config);

  const body = {
    // string
    name: name_example,
    // string
    description: description_example,
    // Blob (optional)
    picture: BINARY_DATA_HERE,
    // string (optional)
    category: category_example,
  } satisfies CreateProductRequest;

  try {
    const data = await api.createProduct(body);
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
| **name** | `string` |  | [Defaults to `undefined`] |
| **description** | `string` |  | [Defaults to `undefined`] |
| **picture** | `Blob` |  | [Optional] [Defaults to `undefined`] |
| **category** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**ProductsModel**](ProductsModel.md)

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


## findAllProducts

> Array&lt;ProductsModel&gt; findAllProducts()

Find all products

### Example

```ts
import {
  Configuration,
  ProductsApi,
} from '@opeolluwa/asterisk';
import type { FindAllProductsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ProductsApi(config);

  try {
    const data = await api.findAllProducts();
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

[**Array&lt;ProductsModel&gt;**](ProductsModel.md)

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


## findProduct

> ProductsModel findProduct(identifier)

Find product

### Example

```ts
import {
  Configuration,
  ProductsApi,
} from '@opeolluwa/asterisk';
import type { FindProductRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ProductsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindProductRequest;

  try {
    const data = await api.findProduct(body);
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

[**ProductsModel**](ProductsModel.md)

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

