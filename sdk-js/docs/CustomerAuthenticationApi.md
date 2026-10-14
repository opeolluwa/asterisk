# CustomerAuthenticationApi

All URIs are relative to *http://localhost*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**completeCustomerOnboarding**](CustomerAuthenticationApi.md#completecustomeronboardingoperation) | **POST** /customers/me/onboarding | Complete customer onboarding |
| [**confirmCustomerAccount**](CustomerAuthenticationApi.md#confirmcustomeraccountoperation) | **POST** /customers/confirm-account | Confirm customer account |
| [**getCustomerProfile**](CustomerAuthenticationApi.md#getcustomerprofile) | **GET** /customers/me | Get customer profile |
| [**loginCustomer**](CustomerAuthenticationApi.md#logincustomeroperation) | **POST** /customers/login | Login customer |
| [**logout2**](CustomerAuthenticationApi.md#logout2) | **POST** /customers/logout | Logout |
| [**refreshCustomerToken**](CustomerAuthenticationApi.md#refreshcustomertoken) | **POST** /customers/refresh-token | Refresh customer token |
| [**requestCustomerOtp**](CustomerAuthenticationApi.md#requestcustomerotp) | **POST** /customers/request-otp | Request customer otp |
| [**requestPasswordReset**](CustomerAuthenticationApi.md#requestpasswordresetoperation) | **POST** /customers/request-password-reset | Request password reset |
| [**resetCustomerAccount**](CustomerAuthenticationApi.md#resetcustomeraccountoperation) | **POST** /customers/reset-password | Reset customer account |
| [**signUpCustomer**](CustomerAuthenticationApi.md#signupcustomer) | **POST** /customers/signup | Sign up customer |
| [**updateCustomerAvatar**](CustomerAuthenticationApi.md#updatecustomeravatar) | **POST** /customers/me/avatar | Update customer avatar |
| [**updateCustomerProfile**](CustomerAuthenticationApi.md#updatecustomerprofile) | **PUT** /customers/me | Update customer profile |
| [**verifyCustomerOtp**](CustomerAuthenticationApi.md#verifycustomerotpoperation) | **POST** /customers/verify-otp | Verify customer otp |



## completeCustomerOnboarding

> CustomerProfile completeCustomerOnboarding(completeCustomerOnboardingRequest)

Complete customer onboarding

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { CompleteCustomerOnboardingOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  const body = {
    // CompleteCustomerOnboardingRequest
    completeCustomerOnboardingRequest: ...,
  } satisfies CompleteCustomerOnboardingOperationRequest;

  try {
    const data = await api.completeCustomerOnboarding(body);
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
| **completeCustomerOnboardingRequest** | [CompleteCustomerOnboardingRequest](CompleteCustomerOnboardingRequest.md) |  | |

### Return type

[**CustomerProfile**](CustomerProfile.md)

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


## confirmCustomerAccount

> MessageResponse confirmCustomerAccount(confirmCustomerAccountRequest)

Confirm customer account

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { ConfirmCustomerAccountOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  const body = {
    // ConfirmCustomerAccountRequest
    confirmCustomerAccountRequest: ...,
  } satisfies ConfirmCustomerAccountOperationRequest;

  try {
    const data = await api.confirmCustomerAccount(body);
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
| **confirmCustomerAccountRequest** | [ConfirmCustomerAccountRequest](ConfirmCustomerAccountRequest.md) |  | |

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


## getCustomerProfile

> CustomerProfile getCustomerProfile()

Get customer profile

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { GetCustomerProfileRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  try {
    const data = await api.getCustomerProfile();
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


## loginCustomer

> CustomerLoginResult loginCustomer(loginCustomerRequest)

Login customer

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { LoginCustomerOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CustomerAuthenticationApi();

  const body = {
    // LoginCustomerRequest
    loginCustomerRequest: ...,
  } satisfies LoginCustomerOperationRequest;

  try {
    const data = await api.loginCustomer(body);
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
| **loginCustomerRequest** | [LoginCustomerRequest](LoginCustomerRequest.md) |  | |

### Return type

[**CustomerLoginResult**](CustomerLoginResult.md)

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


## logout2

> MessageResponse logout2()

Logout

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { Logout2Request } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  try {
    const data = await api.logout2();
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


## refreshCustomerToken

> RefreshTokenResult refreshCustomerToken()

Refresh customer token

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { RefreshCustomerTokenRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CustomerAuthenticationApi();

  try {
    const data = await api.refreshCustomerToken();
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


## requestCustomerOtp

> MessageResponse requestCustomerOtp()

Request customer otp

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { RequestCustomerOtpRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  try {
    const data = await api.requestCustomerOtp();
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


## requestPasswordReset

> RequestPasswordResetResult requestPasswordReset(requestPasswordResetRequest)

Request password reset

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { RequestPasswordResetOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CustomerAuthenticationApi();

  const body = {
    // RequestPasswordResetRequest
    requestPasswordResetRequest: ...,
  } satisfies RequestPasswordResetOperationRequest;

  try {
    const data = await api.requestPasswordReset(body);
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
| **requestPasswordResetRequest** | [RequestPasswordResetRequest](RequestPasswordResetRequest.md) |  | |

### Return type

[**RequestPasswordResetResult**](RequestPasswordResetResult.md)

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


## resetCustomerAccount

> MessageResponse resetCustomerAccount(resetCustomerAccountRequest)

Reset customer account

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { ResetCustomerAccountOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  const body = {
    // ResetCustomerAccountRequest
    resetCustomerAccountRequest: ...,
  } satisfies ResetCustomerAccountOperationRequest;

  try {
    const data = await api.resetCustomerAccount(body);
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
| **resetCustomerAccountRequest** | [ResetCustomerAccountRequest](ResetCustomerAccountRequest.md) |  | |

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


## signUpCustomer

> CustomerSignUpResult signUpCustomer(createCustomerRequest)

Sign up customer

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { SignUpCustomerRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const api = new CustomerAuthenticationApi();

  const body = {
    // CreateCustomerRequest
    createCustomerRequest: ...,
  } satisfies SignUpCustomerRequest;

  try {
    const data = await api.signUpCustomer(body);
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
| **createCustomerRequest** | [CreateCustomerRequest](CreateCustomerRequest.md) |  | |

### Return type

[**CustomerSignUpResult**](CustomerSignUpResult.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Success |  -  |
| **400** | Bad request |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateCustomerAvatar

> CustomerProfile updateCustomerAvatar(file)

Update customer avatar

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { UpdateCustomerAvatarRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  const body = {
    // Blob
    file: BINARY_DATA_HERE,
  } satisfies UpdateCustomerAvatarRequest;

  try {
    const data = await api.updateCustomerAvatar(body);
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
| **file** | `Blob` |  | [Defaults to `undefined`] |

### Return type

[**CustomerProfile**](CustomerProfile.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: `multipart/form-data`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Success |  -  |
| **400** | Bad request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Internal server error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateCustomerProfile

> CustomerProfile updateCustomerProfile(updateCustomerRequest)

Update customer profile

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { UpdateCustomerProfileRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  const body = {
    // UpdateCustomerRequest
    updateCustomerRequest: ...,
  } satisfies UpdateCustomerProfileRequest;

  try {
    const data = await api.updateCustomerProfile(body);
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
| **updateCustomerRequest** | [UpdateCustomerRequest](UpdateCustomerRequest.md) |  | |

### Return type

[**CustomerProfile**](CustomerProfile.md)

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


## verifyCustomerOtp

> MessageResponse verifyCustomerOtp(verifyCustomerOtpRequest)

Verify customer otp

### Example

```ts
import {
  Configuration,
  CustomerAuthenticationApi,
} from '@opeolluwa/asterisk';
import type { VerifyCustomerOtpOperationRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CustomerAuthenticationApi(config);

  const body = {
    // VerifyCustomerOtpRequest
    verifyCustomerOtpRequest: ...,
  } satisfies VerifyCustomerOtpOperationRequest;

  try {
    const data = await api.verifyCustomerOtp(body);
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
| **verifyCustomerOtpRequest** | [VerifyCustomerOtpRequest](VerifyCustomerOtpRequest.md) |  | |

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

