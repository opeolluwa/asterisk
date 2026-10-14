# TeamApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**blockTeamMember**](TeamApi.md#blockteammember) | **PUT** /teams/{identifier}/block | Block team member |
| [**countTeamMembers**](TeamApi.md#countteammembers) | **GET** /teams/count | Count team members |
| [**deleteTeamMember**](TeamApi.md#deleteteammember) | **DELETE** /teams/{identifier} | Delete team member |
| [**findAllTeamMembers**](TeamApi.md#findallteammembers) | **GET** /teams | Find all team members |
| [**findTeamMemberByIdentifier**](TeamApi.md#findteammemberbyidentifier) | **GET** /teams/{identifier} | Find team member by identifier |
| [**inviteTeamMember**](TeamApi.md#inviteteammember) | **POST** /teams/invite | Invite team member |
| [**unblockTeamMember**](TeamApi.md#unblockteammember) | **PUT** /teams/{identifier}/unblock | Unblock team member |
| [**updateTeamMember**](TeamApi.md#updateteammemberoperation) | **PUT** /teams/{identifier} | Update team member |



## blockTeamMember

> TeamMember blockTeamMember(identifier)

Block team member

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { BlockTeamMemberRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies BlockTeamMemberRequest;

  try {
    const data = await api.blockTeamMember(body);
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

[**TeamMember**](TeamMember.md)

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


## countTeamMembers

> CountResponse countTeamMembers()

Count team members

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { CountTeamMembersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  try {
    const data = await api.countTeamMembers();
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


## deleteTeamMember

> MessageResponse deleteTeamMember(identifier)

Delete team member

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { DeleteTeamMemberRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies DeleteTeamMemberRequest;

  try {
    const data = await api.deleteTeamMember(body);
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


## findAllTeamMembers

> Array&lt;TeamMember&gt; findAllTeamMembers()

Find all team members

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { FindAllTeamMembersRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  try {
    const data = await api.findAllTeamMembers();
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

[**Array&lt;TeamMember&gt;**](TeamMember.md)

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


## findTeamMemberByIdentifier

> TeamMember findTeamMemberByIdentifier(identifier)

Find team member by identifier

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { FindTeamMemberByIdentifierRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies FindTeamMemberByIdentifierRequest;

  try {
    const data = await api.findTeamMemberByIdentifier(body);
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

[**TeamMember**](TeamMember.md)

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


## inviteTeamMember

> TeamsModel inviteTeamMember(createTeamMemberRequest)

Invite team member

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { InviteTeamMemberRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  const body = {
    // CreateTeamMemberRequest
    createTeamMemberRequest: ...,
  } satisfies InviteTeamMemberRequest;

  try {
    const data = await api.inviteTeamMember(body);
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
| **createTeamMemberRequest** | [CreateTeamMemberRequest](CreateTeamMemberRequest.md) |  | |

### Return type

[**TeamsModel**](TeamsModel.md)

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


## unblockTeamMember

> TeamMember unblockTeamMember(identifier)

Unblock team member

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { UnblockTeamMemberRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  const body = {
    // string
    identifier: identifier_example,
  } satisfies UnblockTeamMemberRequest;

  try {
    const data = await api.unblockTeamMember(body);
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

[**TeamMember**](TeamMember.md)

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


## updateTeamMember

> TeamsModel updateTeamMember(identifier, updateTeamMemberRequest)

Update team member

### Example

```ts
import {
  Configuration,
  TeamApi,
} from '@opeolluwa/asterisk';
import type { UpdateTeamMemberOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new TeamApi(config);

  const body = {
    // string
    identifier: identifier_example,
    // UpdateTeamMemberRequest
    updateTeamMemberRequest: ...,
  } satisfies UpdateTeamMemberOperationRequest;

  try {
    const data = await api.updateTeamMember(body);
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
| **updateTeamMemberRequest** | [UpdateTeamMemberRequest](UpdateTeamMemberRequest.md) |  | |

### Return type

[**TeamsModel**](TeamsModel.md)

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

