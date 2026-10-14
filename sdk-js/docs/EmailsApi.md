# EmailsApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**countEmails**](EmailsApi.md#countemails) | **GET** /emails/count | Count emails |
| [**countUnreadEmails**](EmailsApi.md#countunreademails) | **GET** /emails/count/unread | Count unread emails |
| [**createEmail**](EmailsApi.md#createemailoperation) | **POST** /emails | Create email |
| [**deleteEmail**](EmailsApi.md#deleteemail) | **DELETE** /emails/{identifier} | Delete email |
| [**findAllEmails**](EmailsApi.md#findallemails) | **GET** /emails | Find all emails |
| [**findEmailByIdentifier**](EmailsApi.md#findemailbyidentifier) | **GET** /emails/{identifier} | Find email by identifier |
| [**findEmailsByTag**](EmailsApi.md#findemailsbytag) | **GET** /emails/tag/{tag} | Find emails by tag |
| [**findStarredEmails**](EmailsApi.md#findstarredemails) | **GET** /emails/starred | Find starred emails |
| [**findUnreadEmails**](EmailsApi.md#findunreademails) | **GET** /emails/unread | Find unread emails |
| [**markEmailAsRead**](EmailsApi.md#markemailasread) | **PUT** /emails/{identifier}/read | Mark email as read |
| [**updateEmail**](EmailsApi.md#updateemailoperation) | **PUT** /emails/{identifier} | Update email |



## countEmails

> CountResponse countEmails()

Count emails

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { CountEmailsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  try {
    const data = await api.countEmails();
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


## countUnreadEmails

> CountResponse countUnreadEmails()

Count unread emails

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { CountUnreadEmailsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  try {
    const data = await api.countUnreadEmails();
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


## createEmail

> EmailsModel createEmail(createEmailRequest)

Create email

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { CreateEmailOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // CreateEmailRequest
    createEmailRequest: ...,
  } satisfies CreateEmailOperationRequest;

  try {
    const data = await api.createEmail(body);
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
| **createEmailRequest** | [CreateEmailRequest](CreateEmailRequest.md) |  | |

### Return type

[**EmailsModel**](EmailsModel.md)

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


## deleteEmail

> MessageResponse deleteEmail(identifier)

Delete email

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { DeleteEmailRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteEmailRequest;

  try {
    const data = await api.deleteEmail(body);
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


## findAllEmails

> FindAllEmails200Response findAllEmails(search, tag, isRead, isStarred, hasAttachments, page, pageSize)

Find all emails

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { FindAllEmailsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // string (optional)
    search: search_example,
    // string (optional)
    tag: tag_example,
    // boolean (optional)
    isRead: true,
    // boolean (optional)
    isStarred: true,
    // boolean (optional)
    hasAttachments: true,
    // number (optional)
    page: 56,
    // number (optional)
    pageSize: 56,
  } satisfies FindAllEmailsRequest;

  try {
    const data = await api.findAllEmails(body);
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
| **search** | `string` |  | [Optional] [Defaults to `undefined`] |
| **tag** | `string` |  | [Optional] [Defaults to `undefined`] |
| **isRead** | `boolean` |  | [Optional] [Defaults to `undefined`] |
| **isStarred** | `boolean` |  | [Optional] [Defaults to `undefined`] |
| **hasAttachments** | `boolean` |  | [Optional] [Defaults to `undefined`] |
| **page** | `number` |  | [Optional] [Defaults to `undefined`] |
| **pageSize** | `number` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**FindAllEmails200Response**](FindAllEmails200Response.md)

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


## findEmailByIdentifier

> EmailsModel findEmailByIdentifier(identifier)

Find email by identifier

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { FindEmailByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindEmailByIdentifierRequest;

  try {
    const data = await api.findEmailByIdentifier(body);
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

[**EmailsModel**](EmailsModel.md)

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


## findEmailsByTag

> Array&lt;EmailsModel&gt; findEmailsByTag(tag)

Find emails by tag

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { FindEmailsByTagRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // string
    tag: tag_example,
  } satisfies FindEmailsByTagRequest;

  try {
    const data = await api.findEmailsByTag(body);
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
| **tag** | `string` |  | [Defaults to `undefined`] |

### Return type

[**Array&lt;EmailsModel&gt;**](EmailsModel.md)

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


## findStarredEmails

> Array&lt;EmailsModel&gt; findStarredEmails()

Find starred emails

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { FindStarredEmailsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  try {
    const data = await api.findStarredEmails();
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

[**Array&lt;EmailsModel&gt;**](EmailsModel.md)

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


## findUnreadEmails

> Array&lt;EmailsModel&gt; findUnreadEmails()

Find unread emails

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { FindUnreadEmailsRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  try {
    const data = await api.findUnreadEmails();
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

[**Array&lt;EmailsModel&gt;**](EmailsModel.md)

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


## markEmailAsRead

> EmailsModel markEmailAsRead(identifier)

Mark email as read

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { MarkEmailAsReadRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies MarkEmailAsReadRequest;

  try {
    const data = await api.markEmailAsRead(body);
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

[**EmailsModel**](EmailsModel.md)

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


## updateEmail

> EmailsModel updateEmail(identifier, updateEmailRequest)

Update email

### Example

```ts
import {
  Configuration,
  EmailsApi,
} from '@opeolluwa/asterisk';
import type { UpdateEmailOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new EmailsApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateEmailRequest
    updateEmailRequest: ...,
  } satisfies UpdateEmailOperationRequest;

  try {
    const data = await api.updateEmail(body);
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
| **updateEmailRequest** | [UpdateEmailRequest](UpdateEmailRequest.md) |  | |

### Return type

[**EmailsModel**](EmailsModel.md)

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

