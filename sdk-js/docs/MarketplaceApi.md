# MarketplaceApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**findAllMarketplace**](MarketplaceApi.md#findallmarketplace) | **GET** /marketplace | Find all marketplace |
| [**findMarketplaceByIdentifier**](MarketplaceApi.md#findmarketplacebyidentifier) | **GET** /marketplace/{identifier} | Find marketplace by identifier |



## findAllMarketplace

> FindAllMarketplace200Response findAllMarketplace(page, pageSize, search, inStock, sort)

Find all marketplace

### Example

```ts
import {
  Configuration,
  MarketplaceApi,
} from '@opeolluwa/asterisk';
import type { FindAllMarketplaceRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new MarketplaceApi();

  const body = {
    // number (optional)
    page: 56,
    // number (optional)
    pageSize: 56,
    // string (optional)
    search: search_example,
    // boolean (optional)
    inStock: true,
    // MarketplaceSort (optional)
    sort: ...,
  } satisfies FindAllMarketplaceRequest;

  try {
    const data = await api.findAllMarketplace(body);
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
| **search** | `string` |  | [Optional] [Defaults to `undefined`] |
| **inStock** | `boolean` |  | [Optional] [Defaults to `undefined`] |
| **sort** | `MarketplaceSort` |  | [Optional] [Defaults to `undefined`] [Enum: newest, price_asc, price_desc, name_asc, name_desc] |

### Return type

[**FindAllMarketplace200Response**](FindAllMarketplace200Response.md)

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


## findMarketplaceByIdentifier

> MarketplaceItem findMarketplaceByIdentifier(identifier)

Find marketplace by identifier

### Example

```ts
import {
  Configuration,
  MarketplaceApi,
} from '@opeolluwa/asterisk';
import type { FindMarketplaceByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new MarketplaceApi();

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindMarketplaceByIdentifierRequest;

  try {
    const data = await api.findMarketplaceByIdentifier(body);
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

[**MarketplaceItem**](MarketplaceItem.md)

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

