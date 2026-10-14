# AuthenticationApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**forgottenPassword**](AuthenticationApi.md#forgottenpasswordoperation) | **POST** /forgotten-password | Forgotten password |
| [**login**](AuthenticationApi.md#loginoperation) | **POST** /login | Login |
| [**logout**](AuthenticationApi.md#logout) | **POST** /logout | Logout |
| [**requestNewOtp**](AuthenticationApi.md#requestnewotp) | **POST** /request-otp | Request new otp |
| [**requestRefreshToken**](AuthenticationApi.md#requestrefreshtoken) | **POST** /refresh-token | Request refresh token |
| [**setNewPassword**](AuthenticationApi.md#setnewpasswordoperation) | **POST** /reset-password | Set new password |
| [**verifyAccount**](AuthenticationApi.md#verifyaccountoperation) | **POST** /verify-account | Verify account |
| [**verifyOtp**](AuthenticationApi.md#verifyotp) | **POST** /verify-otp | Verify otp |



## forgottenPassword

> object forgottenPassword(forgottenPasswordRequest)

Forgotten password

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { ForgottenPasswordOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new AuthenticationApi();

  const body = {
    // ForgottenPasswordRequest
    forgottenPasswordRequest: ...,
  } satisfies ForgottenPasswordOperationRequest;

  try {
    const data = await api.forgottenPassword(body);
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
| **forgottenPasswordRequest** | [ForgottenPasswordRequest](ForgottenPasswordRequest.md) |  | |

### Return type

**object**

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


## login

> LoginResponse login(loginRequest)

Login

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { LoginOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new AuthenticationApi();

  const body = {
    // LoginRequest
    loginRequest: ...,
  } satisfies LoginOperationRequest;

  try {
    const data = await api.login(body);
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
| **loginRequest** | [LoginRequest](LoginRequest.md) |  | |

### Return type

[**LoginResponse**](LoginResponse.md)

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


## logout

> MessageResponse logout()

Logout

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { LogoutRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AuthenticationApi(config);

  try {
    const data = await api.logout();
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


## requestNewOtp

> MessageResponse requestNewOtp(variant)

Request new otp

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { RequestNewOtpRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AuthenticationApi(config);

  const body = {
    // VerifyOtpVariant (optional)
    variant: ...,
  } satisfies RequestNewOtpRequest;

  try {
    const data = await api.requestNewOtp(body);
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
| **variant** | `VerifyOtpVariant` |  | [Optional] [Defaults to `undefined`] [Enum: two-factor, forgotten-password] |

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


## requestRefreshToken

> RefreshTokenResult requestRefreshToken()

Request refresh token

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { RequestRefreshTokenRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new AuthenticationApi();

  try {
    const data = await api.requestRefreshToken();
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

[**RefreshTokenResult**](RefreshTokenResult.md)

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


## setNewPassword

> MessageResponse setNewPassword(setNewPasswordRequest)

Set new password

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { SetNewPasswordOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AuthenticationApi(config);

  const body = {
    // SetNewPasswordRequest
    setNewPasswordRequest: ...,
  } satisfies SetNewPasswordOperationRequest;

  try {
    const data = await api.setNewPassword(body);
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
| **setNewPasswordRequest** | [SetNewPasswordRequest](SetNewPasswordRequest.md) |  | |

### Return type

[**MessageResponse**](MessageResponse.md)

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


## verifyAccount

> object verifyAccount(verifyAccountRequest)

Verify account

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { VerifyAccountOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AuthenticationApi(config);

  const body = {
    // VerifyAccountRequest
    verifyAccountRequest: ...,
  } satisfies VerifyAccountOperationRequest;

  try {
    const data = await api.verifyAccount(body);
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
| **verifyAccountRequest** | [VerifyAccountRequest](VerifyAccountRequest.md) |  | |

### Return type

**object**

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


## verifyOtp

> VerifyOtpResponse verifyOtp(verifyAccountRequest, variant)

Verify otp

### Example

```ts
import {
  Configuration,
  AuthenticationApi,
} from '@opeolluwa/asterisk';
import type { VerifyOtpRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AuthenticationApi(config);

  const body = {
    // VerifyAccountRequest
    verifyAccountRequest: ...,
    // VerifyOtpVariant (optional)
    variant: ...,
  } satisfies VerifyOtpRequest;

  try {
    const data = await api.verifyOtp(body);
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
| **verifyAccountRequest** | [VerifyAccountRequest](VerifyAccountRequest.md) |  | |
| **variant** | `VerifyOtpVariant` |  | [Optional] [Defaults to `undefined`] [Enum: two-factor, forgotten-password] |

### Return type

[**VerifyOtpResponse**](VerifyOtpResponse.md)

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

