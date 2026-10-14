# @opeolluwa/asterisk@0.6.2

A TypeScript SDK client for the localhost API.

## Usage

First, install the SDK from npm.

```bash
npm install @opeolluwa/asterisk --save
```

Next, try it out.


```ts
import {
  Configuration,
  AppConfigApi,
} from '@opeolluwa/asterisk';
import type { FetchAppConfigRequest } from '@opeolluwa/asterisk';

async function example() {
  console.log("🚀 Testing @opeolluwa/asterisk SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: bearerAuth
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new AppConfigApi(config);

  try {
    const data = await api.fetchAppConfig();
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```


## Documentation

### API Endpoints

All URIs are relative to *http://localhost*

| Class | Method | HTTP request | Description
| ----- | ------ | ------------ | -------------
*AppConfigApi* | [**fetchAppConfig**](docs/AppConfigApi.md#fetchappconfig) | **GET** /config | Fetch app config
*AppConfigApi* | [**updateAppConfig**](docs/AppConfigApi.md#updateappconfigoperation) | **PUT** /config | Update app config
*AuthenticationApi* | [**forgottenPassword**](docs/AuthenticationApi.md#forgottenpasswordoperation) | **POST** /forgotten-password | Forgotten password
*AuthenticationApi* | [**login**](docs/AuthenticationApi.md#loginoperation) | **POST** /login | Login
*AuthenticationApi* | [**logout**](docs/AuthenticationApi.md#logout) | **POST** /logout | Logout
*AuthenticationApi* | [**requestNewOtp**](docs/AuthenticationApi.md#requestnewotp) | **POST** /request-otp | Request new otp
*AuthenticationApi* | [**requestRefreshToken**](docs/AuthenticationApi.md#requestrefreshtoken) | **POST** /refresh-token | Request refresh token
*AuthenticationApi* | [**setNewPassword**](docs/AuthenticationApi.md#setnewpasswordoperation) | **POST** /reset-password | Set new password
*AuthenticationApi* | [**verifyAccount**](docs/AuthenticationApi.md#verifyaccountoperation) | **POST** /verify-account | Verify account
*AuthenticationApi* | [**verifyOtp**](docs/AuthenticationApi.md#verifyotp) | **POST** /verify-otp | Verify otp
*CartApi* | [**addCartItem**](docs/CartApi.md#addcartitemoperation) | **POST** /customer/cart/items | Add cart item
*CartApi* | [**countCartItems**](docs/CartApi.md#countcartitems) | **GET** /customer/cart/count | Count cart items
*CartApi* | [**deleteCartItemByIdentifier**](docs/CartApi.md#deletecartitembyidentifier) | **DELETE** /customer/cart/items/{identifier} | Delete cart item by identifier
*CartApi* | [**findCart**](docs/CartApi.md#findcart) | **GET** /customer/cart | Find cart
*CartApi* | [**findCartItemByIdentifier**](docs/CartApi.md#findcartitembyidentifier) | **GET** /customer/cart/items/{identifier} | Find cart item by identifier
*CartApi* | [**syncCart**](docs/CartApi.md#synccart) | **POST** /customer/cart/sync | Sync cart
*CartApi* | [**updateCartItemByIdentifier**](docs/CartApi.md#updatecartitembyidentifier) | **PUT** /customer/cart/items/{identifier} | Update cart item by identifier
*ComplaintsApi* | [**archiveComplaint**](docs/ComplaintsApi.md#archivecomplaint) | **DELETE** /complaints/{identifier} | Archive complaint
*ComplaintsApi* | [**countComplaints**](docs/ComplaintsApi.md#countcomplaints) | **GET** /complaints/count | Count complaints
*ComplaintsApi* | [**findAllComplaints**](docs/ComplaintsApi.md#findallcomplaints) | **GET** /complaints | Find all complaints
*ComplaintsApi* | [**findComplaintByIdentifier**](docs/ComplaintsApi.md#findcomplaintbyidentifier) | **GET** /complaints/{identifier} | Find complaint by identifier
*ComplaintsApi* | [**findComplaintReplies**](docs/ComplaintsApi.md#findcomplaintreplies) | **GET** /complaints/{identifier}/replies | Find complaint replies
*ComplaintsApi* | [**replyToComplaint**](docs/ComplaintsApi.md#replytocomplaint) | **POST** /complaints/{identifier}/replies | Reply to complaint
*ComplaintsApi* | [**updateComplaint**](docs/ComplaintsApi.md#updatecomplaintoperation) | **PUT** /complaints/{identifier} | Update complaint
*CustomerAddressesApi* | [**countCustomerAddress**](docs/CustomerAddressesApi.md#countcustomeraddress) | **GET** /customer/address/count | Count customer address
*CustomerAddressesApi* | [**createCustomerAddress**](docs/CustomerAddressesApi.md#createcustomeraddressoperation) | **POST** /customer/address | Create customer address
*CustomerAddressesApi* | [**deleteCustomerAddressByIdentifier**](docs/CustomerAddressesApi.md#deletecustomeraddressbyidentifier) | **DELETE** /customer/address/{identifier} | Delete customer address by identifier
*CustomerAddressesApi* | [**findAllCustomerAddress**](docs/CustomerAddressesApi.md#findallcustomeraddress) | **GET** /customer/address | Find all customer address
*CustomerAddressesApi* | [**findCustomerAddressByIdentifier**](docs/CustomerAddressesApi.md#findcustomeraddressbyidentifier) | **GET** /customer/address/{identifier} | Find customer address by identifier
*CustomerAddressesApi* | [**updateCustomerAddressByIdentifier**](docs/CustomerAddressesApi.md#updatecustomeraddressbyidentifier) | **PUT** /customer/address/{identifier} | Update customer address by identifier
*CustomerAuthenticationApi* | [**completeCustomerOnboarding**](docs/CustomerAuthenticationApi.md#completecustomeronboardingoperation) | **POST** /customers/me/onboarding | Complete customer onboarding
*CustomerAuthenticationApi* | [**confirmCustomerAccount**](docs/CustomerAuthenticationApi.md#confirmcustomeraccountoperation) | **POST** /customers/confirm-account | Confirm customer account
*CustomerAuthenticationApi* | [**getCustomerProfile**](docs/CustomerAuthenticationApi.md#getcustomerprofile) | **GET** /customers/me | Get customer profile
*CustomerAuthenticationApi* | [**loginCustomer**](docs/CustomerAuthenticationApi.md#logincustomeroperation) | **POST** /customers/login | Login customer
*CustomerAuthenticationApi* | [**logout2**](docs/CustomerAuthenticationApi.md#logout2) | **POST** /customers/logout | Logout
*CustomerAuthenticationApi* | [**refreshCustomerToken**](docs/CustomerAuthenticationApi.md#refreshcustomertoken) | **POST** /customers/refresh-token | Refresh customer token
*CustomerAuthenticationApi* | [**requestCustomerOtp**](docs/CustomerAuthenticationApi.md#requestcustomerotp) | **POST** /customers/request-otp | Request customer otp
*CustomerAuthenticationApi* | [**requestPasswordReset**](docs/CustomerAuthenticationApi.md#requestpasswordresetoperation) | **POST** /customers/request-password-reset | Request password reset
*CustomerAuthenticationApi* | [**resetCustomerAccount**](docs/CustomerAuthenticationApi.md#resetcustomeraccountoperation) | **POST** /customers/reset-password | Reset customer account
*CustomerAuthenticationApi* | [**signUpCustomer**](docs/CustomerAuthenticationApi.md#signupcustomer) | **POST** /customers/signup | Sign up customer
*CustomerAuthenticationApi* | [**updateCustomerAvatar**](docs/CustomerAuthenticationApi.md#updatecustomeravatar) | **POST** /customers/me/avatar | Update customer avatar
*CustomerAuthenticationApi* | [**updateCustomerProfile**](docs/CustomerAuthenticationApi.md#updatecustomerprofile) | **PUT** /customers/me | Update customer profile
*CustomerAuthenticationApi* | [**verifyCustomerOtp**](docs/CustomerAuthenticationApi.md#verifycustomerotpoperation) | **POST** /customers/verify-otp | Verify customer otp
*CustomerComplaintsApi* | [**createCustomerComplaint**](docs/CustomerComplaintsApi.md#createcustomercomplaint) | **POST** /customer/complaint | Create customer complaint
*CustomerComplaintsApi* | [**findCustomerComplaintByIdentifier**](docs/CustomerComplaintsApi.md#findcustomercomplaintbyidentifier) | **GET** /customer/complaint/{identifier} | Find customer complaint by identifier
*CustomerComplaintsApi* | [**findCustomerComplaintReplies**](docs/CustomerComplaintsApi.md#findcustomercomplaintreplies) | **GET** /customer/complaint/{identifier}/replies | Find customer complaint replies
*CustomerComplaintsApi* | [**findCustomerComplaints**](docs/CustomerComplaintsApi.md#findcustomercomplaints) | **GET** /customer/complaint | Find customer complaints
*CustomerComplaintsApi* | [**replyToCustomerComplaint**](docs/CustomerComplaintsApi.md#replytocustomercomplaint) | **POST** /customer/complaint/{identifier}/replies | Reply to customer complaint
*CustomerOrdersApi* | [**checkoutCustomerOrder**](docs/CustomerOrdersApi.md#checkoutcustomerorder) | **POST** /customer/orders/checkout | Checkout customer order
*CustomerOrdersApi* | [**createCustomerOrder**](docs/CustomerOrdersApi.md#createcustomerorder) | **POST** /customer/orders | Create customer order
*CustomerOrdersApi* | [**findCustomerOrderByIdentifier**](docs/CustomerOrdersApi.md#findcustomerorderbyidentifier) | **GET** /customer/orders/{identifier} | Find customer order by identifier
*CustomerOrdersApi* | [**findCustomerOrders**](docs/CustomerOrdersApi.md#findcustomerorders) | **GET** /customer/orders | Find customer orders
*CustomersApi* | [**countCustomers**](docs/CustomersApi.md#countcustomers) | **GET** /customers/count | Count customers
*CustomersApi* | [**deactivateCustomer**](docs/CustomersApi.md#deactivatecustomer) | **PUT** /customers/{identifier}/deactivate | Deactivate customer
*CustomersApi* | [**deactivateCustomerByIdentifier**](docs/CustomersApi.md#deactivatecustomerbyidentifier) | **DELETE** /customers/{identifier} | Deactivate customer by identifier
*CustomersApi* | [**findAllCustomers**](docs/CustomersApi.md#findallcustomers) | **GET** /customers | Find all customers
*CustomersApi* | [**findCustomerByIdentifier**](docs/CustomersApi.md#findcustomerbyidentifier) | **GET** /customers/{identifier} | Find customer by identifier
*CustomersApi* | [**reactivateCustomer**](docs/CustomersApi.md#reactivatecustomer) | **PUT** /customers/{identifier}/reactivate | Reactivate customer
*EmailsApi* | [**countEmails**](docs/EmailsApi.md#countemails) | **GET** /emails/count | Count emails
*EmailsApi* | [**countUnreadEmails**](docs/EmailsApi.md#countunreademails) | **GET** /emails/count/unread | Count unread emails
*EmailsApi* | [**createEmail**](docs/EmailsApi.md#createemailoperation) | **POST** /emails | Create email
*EmailsApi* | [**deleteEmail**](docs/EmailsApi.md#deleteemail) | **DELETE** /emails/{identifier} | Delete email
*EmailsApi* | [**findAllEmails**](docs/EmailsApi.md#findallemails) | **GET** /emails | Find all emails
*EmailsApi* | [**findEmailByIdentifier**](docs/EmailsApi.md#findemailbyidentifier) | **GET** /emails/{identifier} | Find email by identifier
*EmailsApi* | [**findEmailsByTag**](docs/EmailsApi.md#findemailsbytag) | **GET** /emails/tag/{tag} | Find emails by tag
*EmailsApi* | [**findStarredEmails**](docs/EmailsApi.md#findstarredemails) | **GET** /emails/starred | Find starred emails
*EmailsApi* | [**findUnreadEmails**](docs/EmailsApi.md#findunreademails) | **GET** /emails/unread | Find unread emails
*EmailsApi* | [**markEmailAsRead**](docs/EmailsApi.md#markemailasread) | **PUT** /emails/{identifier}/read | Mark email as read
*EmailsApi* | [**updateEmail**](docs/EmailsApi.md#updateemailoperation) | **PUT** /emails/{identifier} | Update email
*GeneralApi* | [**archiveCustomerNotificationByIdentifier**](docs/GeneralApi.md#archivecustomernotificationbyidentifier) | **DELETE** /customer/notifications/{identifier} | Archive customer notification by identifier
*GeneralApi* | [**countCustomerNotifications**](docs/GeneralApi.md#countcustomernotifications) | **GET** /customer/notifications/count | Count customer notifications
*GeneralApi* | [**fetchDashboardMetrics**](docs/GeneralApi.md#fetchdashboardmetrics) | **GET** /metrics | Fetch dashboard metrics
*GeneralApi* | [**findCustomerNotificationByIdentifier**](docs/GeneralApi.md#findcustomernotificationbyidentifier) | **GET** /customer/notifications/{identifier} | Find customer notification by identifier
*GeneralApi* | [**findCustomerNotificationPreferences**](docs/GeneralApi.md#findcustomernotificationpreferences) | **GET** /customer/notification-preferences | Find customer notification preferences
*GeneralApi* | [**findCustomerNotifications**](docs/GeneralApi.md#findcustomernotifications) | **GET** /customer/notifications | Find customer notifications
*GeneralApi* | [**updateCustomerNotificationByIdentifier**](docs/GeneralApi.md#updatecustomernotificationbyidentifier) | **PUT** /customer/notifications/{identifier} | Update customer notification by identifier
*GeneralApi* | [**updateCustomerNotificationPreferences**](docs/GeneralApi.md#updatecustomernotificationpreferences) | **PUT** /customer/notification-preferences | Update customer notification preferences
*InventoryApi* | [**countInventory**](docs/InventoryApi.md#countinventory) | **GET** /inventory/count | Count inventory
*InventoryApi* | [**createInventory**](docs/InventoryApi.md#createinventoryoperation) | **POST** /inventory | Create inventory
*InventoryApi* | [**deleteInventoryByIdentifier**](docs/InventoryApi.md#deleteinventorybyidentifier) | **DELETE** /inventory/{identifier} | Delete inventory by identifier
*InventoryApi* | [**findAllInventory**](docs/InventoryApi.md#findallinventory) | **GET** /inventory | Find all inventory
*InventoryApi* | [**findInventoryByIdentifier**](docs/InventoryApi.md#findinventorybyidentifier) | **GET** /inventory/{identifier} | Find inventory by identifier
*InventoryApi* | [**updateInventoryByIdentifier**](docs/InventoryApi.md#updateinventorybyidentifier) | **PUT** /inventory/{identifier} | Update inventory by identifier
*InvitationsApi* | [**acceptInvitation**](docs/InvitationsApi.md#acceptinvitationoperation) | **PUT** /invitations/{identifier}/accept | Accept invitation
*InvitationsApi* | [**blockInvitation**](docs/InvitationsApi.md#blockinvitation) | **PUT** /invitations/{identifier}/block | Block invitation
*InvitationsApi* | [**countInvitations**](docs/InvitationsApi.md#countinvitations) | **GET** /invitations/count | Count invitations
*InvitationsApi* | [**createInvitation**](docs/InvitationsApi.md#createinvitationoperation) | **POST** /invitations | Create invitation
*InvitationsApi* | [**deleteInvitation**](docs/InvitationsApi.md#deleteinvitation) | **DELETE** /invitations/{identifier} | Delete invitation
*InvitationsApi* | [**findAllInvitations**](docs/InvitationsApi.md#findallinvitations) | **GET** /invitations | Find all invitations
*InvitationsApi* | [**findInvitationByIdentifier**](docs/InvitationsApi.md#findinvitationbyidentifier) | **GET** /invitations/{identifier} | Find invitation by identifier
*InvitationsApi* | [**previewInvitation**](docs/InvitationsApi.md#previewinvitation) | **GET** /invitations/{identifier}/preview | Preview invitation
*InvitationsApi* | [**resendInvitation**](docs/InvitationsApi.md#resendinvitation) | **PUT** /invitations/{identifier}/resend | Resend invitation
*MarketplaceApi* | [**findAllMarketplace**](docs/MarketplaceApi.md#findallmarketplace) | **GET** /marketplace | Find all marketplace
*MarketplaceApi* | [**findMarketplaceByIdentifier**](docs/MarketplaceApi.md#findmarketplacebyidentifier) | **GET** /marketplace/{identifier} | Find marketplace by identifier
*NotificationsApi* | [**countNotifications**](docs/NotificationsApi.md#countnotifications) | **GET** /notifications/count | Count notifications
*NotificationsApi* | [**deleteNotificationsByIdentifier**](docs/NotificationsApi.md#deletenotificationsbyidentifier) | **DELETE** /notifications/{identifier} | Delete notifications by identifier
*NotificationsApi* | [**findAllNotifications**](docs/NotificationsApi.md#findallnotifications) | **GET** /notifications | Find all notifications
*NotificationsApi* | [**findNotificationDeliveries**](docs/NotificationsApi.md#findnotificationdeliveries) | **GET** /notifications/{identifier}/deliveries | Find notification deliveries
*NotificationsApi* | [**findNotificationPreferences**](docs/NotificationsApi.md#findnotificationpreferences) | **GET** /notification-preferences | Find notification preferences
*NotificationsApi* | [**findNotificationsByIdentifier**](docs/NotificationsApi.md#findnotificationsbyidentifier) | **GET** /notifications/{identifier} | Find notifications by identifier
*NotificationsApi* | [**markAllNotificationsRead**](docs/NotificationsApi.md#markallnotificationsread) | **PUT** /notifications/read-all | Mark all notifications read
*NotificationsApi* | [**updateNotificationPreferences**](docs/NotificationsApi.md#updatenotificationpreferencesoperation) | **PUT** /notification-preferences | Update notification preferences
*NotificationsApi* | [**updateNotificationsByIdentifier**](docs/NotificationsApi.md#updatenotificationsbyidentifier) | **PUT** /notifications/{identifier} | Update notifications by identifier
*OrdersApi* | [**countOrders**](docs/OrdersApi.md#countorders) | **GET** /orders/count | Count orders
*OrdersApi* | [**deleteOrderItemByIdentifier**](docs/OrdersApi.md#deleteorderitembyidentifier) | **DELETE** /orders/{identifier}/items/{item_identifier} | Delete order item by identifier
*OrdersApi* | [**deleteOrdersByIdentifier**](docs/OrdersApi.md#deleteordersbyidentifier) | **DELETE** /orders/{identifier} | Delete orders by identifier
*OrdersApi* | [**findAllOrders**](docs/OrdersApi.md#findallorders) | **GET** /orders | Find all orders
*OrdersApi* | [**findOrdersByIdentifier**](docs/OrdersApi.md#findordersbyidentifier) | **GET** /orders/{identifier} | Find orders by identifier
*OrdersApi* | [**updateOrderItemByIdentifier**](docs/OrdersApi.md#updateorderitembyidentifier) | **PUT** /orders/{identifier}/items/{item_identifier} | Update order item by identifier
*OrdersApi* | [**updateOrdersByIdentifier**](docs/OrdersApi.md#updateordersbyidentifier) | **PUT** /orders/{identifier} | Update orders by identifier
*PaymentsApi* | [**confirmPayment**](docs/PaymentsApi.md#confirmpaymentoperation) | **POST** /payments/confirm | Confirm payment
*PaymentsApi* | [**handlePaymentWebhook**](docs/PaymentsApi.md#handlepaymentwebhook) | **POST** /payments/webhook | Handle payment webhook
*PaymentsApi* | [**initializePayment**](docs/PaymentsApi.md#initializepaymentoperation) | **POST** /payments/initialize | Initialize payment
*ProductsApi* | [**createProduct**](docs/ProductsApi.md#createproduct) | **POST** /products | Create product
*ProductsApi* | [**findAllProducts**](docs/ProductsApi.md#findallproducts) | **GET** /products | Find all products
*ProductsApi* | [**findProduct**](docs/ProductsApi.md#findproduct) | **GET** /products/{identifier} | Find product
*RolesApi* | [**addPermission**](docs/RolesApi.md#addpermissionoperation) | **POST** /roles/{identifier}/permissions | Add permission
*RolesApi* | [**archiveRole**](docs/RolesApi.md#archiverole) | **DELETE** /roles/{identifier} | Archive role
*RolesApi* | [**assignRole**](docs/RolesApi.md#assignroleoperation) | **POST** /roles/{identifier}/assign | Assign role
*RolesApi* | [**createRole**](docs/RolesApi.md#createroleoperation) | **POST** /roles | Create role
*RolesApi* | [**findAllRoles**](docs/RolesApi.md#findallroles) | **GET** /roles | Find all roles
*RolesApi* | [**findRoleByIdentifier**](docs/RolesApi.md#findrolebyidentifier) | **GET** /roles/{identifier} | Find role by identifier
*RolesApi* | [**removePermission**](docs/RolesApi.md#removepermissionoperation) | **DELETE** /roles/{identifier}/permissions | Remove permission
*RolesApi* | [**updateRole**](docs/RolesApi.md#updateroleoperation) | **PUT** /roles/{identifier} | Update role
*TeamApi* | [**blockTeamMember**](docs/TeamApi.md#blockteammember) | **PUT** /teams/{identifier}/block | Block team member
*TeamApi* | [**countTeamMembers**](docs/TeamApi.md#countteammembers) | **GET** /teams/count | Count team members
*TeamApi* | [**deleteTeamMember**](docs/TeamApi.md#deleteteammember) | **DELETE** /teams/{identifier} | Delete team member
*TeamApi* | [**findAllTeamMembers**](docs/TeamApi.md#findallteammembers) | **GET** /teams | Find all team members
*TeamApi* | [**findTeamMemberByIdentifier**](docs/TeamApi.md#findteammemberbyidentifier) | **GET** /teams/{identifier} | Find team member by identifier
*TeamApi* | [**inviteTeamMember**](docs/TeamApi.md#inviteteammember) | **POST** /teams/invite | Invite team member
*TeamApi* | [**unblockTeamMember**](docs/TeamApi.md#unblockteammember) | **PUT** /teams/{identifier}/unblock | Unblock team member
*TeamApi* | [**updateTeamMember**](docs/TeamApi.md#updateteammemberoperation) | **PUT** /teams/{identifier} | Update team member
*UploadsApi* | [**countUploads**](docs/UploadsApi.md#countuploads) | **GET** /uploads/count | Count uploads
*UploadsApi* | [**createUpload**](docs/UploadsApi.md#createupload) | **POST** /uploads | Create upload
*UploadsApi* | [**deleteUpload**](docs/UploadsApi.md#deleteupload) | **DELETE** /uploads/{identifier} | Delete upload
*UploadsApi* | [**findAllUploads**](docs/UploadsApi.md#findalluploads) | **GET** /uploads | Find all uploads
*UploadsApi* | [**findStarredUploads**](docs/UploadsApi.md#findstarreduploads) | **GET** /uploads/starred | Find starred uploads
*UploadsApi* | [**findUploadByIdentifier**](docs/UploadsApi.md#finduploadbyidentifier) | **GET** /uploads/{identifier} | Find upload by identifier
*UploadsApi* | [**updateUpload**](docs/UploadsApi.md#updateuploadoperation) | **PUT** /uploads/{identifier} | Update upload
*UsersApi* | [**changePassword**](docs/UsersApi.md#changepasswordoperation) | **PATCH** /users/password | Change password
*UsersApi* | [**requestChangePasswordOtp**](docs/UsersApi.md#requestchangepasswordotp) | **POST** /users/password/change | Request change password otp
*UsersApi* | [**requestTwoFactorOtp**](docs/UsersApi.md#requesttwofactorotp) | **POST** /users/two-factor/otp | Request two factor otp
*UsersApi* | [**retrieveInformation**](docs/UsersApi.md#retrieveinformation) | **GET** /users/profile | Retrieve information
*UsersApi* | [**toggleTwoFactor**](docs/UsersApi.md#toggletwofactoroperation) | **PATCH** /users/two-factor | Toggle two factor
*UsersApi* | [**updateProfile**](docs/UsersApi.md#updateprofileoperation) | **PUT** /users/profile | Update profile
*UsersApi* | [**updateProfilePicture**](docs/UsersApi.md#updateprofilepicture) | **POST** /users/avatar | Update profile picture


### Models

- [AcceptInvitationRequest](docs/AcceptInvitationRequest.md)
- [AddCartItemRequest](docs/AddCartItemRequest.md)
- [AddPermissionRequest](docs/AddPermissionRequest.md)
- [AppConfigModel](docs/AppConfigModel.md)
- [AssignRoleRequest](docs/AssignRoleRequest.md)
- [CartItemResponse](docs/CartItemResponse.md)
- [CartResponse](docs/CartResponse.md)
- [CategorySales](docs/CategorySales.md)
- [ChangePasswordRequest](docs/ChangePasswordRequest.md)
- [CheckoutDelivery](docs/CheckoutDelivery.md)
- [CheckoutRequest](docs/CheckoutRequest.md)
- [ComplaintRepliesModel](docs/ComplaintRepliesModel.md)
- [ComplaintStatus](docs/ComplaintStatus.md)
- [ComplaintsModel](docs/ComplaintsModel.md)
- [CompleteCustomerOnboardingRequest](docs/CompleteCustomerOnboardingRequest.md)
- [ConfirmCustomerAccountRequest](docs/ConfirmCustomerAccountRequest.md)
- [ConfirmPaymentRequest](docs/ConfirmPaymentRequest.md)
- [ConfirmPaymentResponse](docs/ConfirmPaymentResponse.md)
- [CountResponse](docs/CountResponse.md)
- [CreateComplaintRequest](docs/CreateComplaintRequest.md)
- [CreateCustomerAddressRequest](docs/CreateCustomerAddressRequest.md)
- [CreateCustomerRequest](docs/CreateCustomerRequest.md)
- [CreateEmailRequest](docs/CreateEmailRequest.md)
- [CreateInventoryRequest](docs/CreateInventoryRequest.md)
- [CreateInvitationRequest](docs/CreateInvitationRequest.md)
- [CreateOrdersRequest](docs/CreateOrdersRequest.md)
- [CreatePrimaryDeliveryAddress](docs/CreatePrimaryDeliveryAddress.md)
- [CreateRoleRequest](docs/CreateRoleRequest.md)
- [CreateTeamMemberRequest](docs/CreateTeamMemberRequest.md)
- [CustomerAddressModel](docs/CustomerAddressModel.md)
- [CustomerLoginResult](docs/CustomerLoginResult.md)
- [CustomerOrderListQuery](docs/CustomerOrderListQuery.md)
- [CustomerProfile](docs/CustomerProfile.md)
- [CustomerSignUpResult](docs/CustomerSignUpResult.md)
- [DashboardMetricsQuery](docs/DashboardMetricsQuery.md)
- [DashboardMetricsResponse](docs/DashboardMetricsResponse.md)
- [DashboardStat](docs/DashboardStat.md)
- [DashboardStatKey](docs/DashboardStatKey.md)
- [DeliveryResponse](docs/DeliveryResponse.md)
- [EmailsModel](docs/EmailsModel.md)
- [FileType](docs/FileType.md)
- [FindAllEmails200Response](docs/FindAllEmails200Response.md)
- [FindAllInventory200Response](docs/FindAllInventory200Response.md)
- [FindAllMarketplace200Response](docs/FindAllMarketplace200Response.md)
- [FindAllNotifications200Response](docs/FindAllNotifications200Response.md)
- [ForgottenPasswordRequest](docs/ForgottenPasswordRequest.md)
- [InitializePaymentRequest](docs/InitializePaymentRequest.md)
- [InventoryListQuery](docs/InventoryListQuery.md)
- [InventoryResponse](docs/InventoryResponse.md)
- [InvitationModel](docs/InvitationModel.md)
- [InvitationPreview](docs/InvitationPreview.md)
- [InvitationStatus](docs/InvitationStatus.md)
- [ListEmailsQuery](docs/ListEmailsQuery.md)
- [ListNotificationsQuery](docs/ListNotificationsQuery.md)
- [LoginChallenge](docs/LoginChallenge.md)
- [LoginCustomerRequest](docs/LoginCustomerRequest.md)
- [LoginRequest](docs/LoginRequest.md)
- [LoginResponse](docs/LoginResponse.md)
- [LoginResult](docs/LoginResult.md)
- [MarketplaceItem](docs/MarketplaceItem.md)
- [MarketplaceListQuery](docs/MarketplaceListQuery.md)
- [MarketplaceSort](docs/MarketplaceSort.md)
- [MessageResponse](docs/MessageResponse.md)
- [NotificationActorType](docs/NotificationActorType.md)
- [NotificationChannel](docs/NotificationChannel.md)
- [NotificationPreferencesModel](docs/NotificationPreferencesModel.md)
- [NotificationRecipientType](docs/NotificationRecipientType.md)
- [NotificationView](docs/NotificationView.md)
- [OrderItemResponse](docs/OrderItemResponse.md)
- [OrderListQuery](docs/OrderListQuery.md)
- [OrderResponse](docs/OrderResponse.md)
- [OrderStatus](docs/OrderStatus.md)
- [OrdersModel](docs/OrdersModel.md)
- [PlaceOrderItem](docs/PlaceOrderItem.md)
- [PriceUpdate](docs/PriceUpdate.md)
- [ProductCurrency](docs/ProductCurrency.md)
- [ProductPriceResponse](docs/ProductPriceResponse.md)
- [ProductSummary](docs/ProductSummary.md)
- [ProductsModel](docs/ProductsModel.md)
- [RefreshTokenResult](docs/RefreshTokenResult.md)
- [RemovePermissionRequest](docs/RemovePermissionRequest.md)
- [ReplyComplaintRequest](docs/ReplyComplaintRequest.md)
- [RequestPasswordResetRequest](docs/RequestPasswordResetRequest.md)
- [RequestPasswordResetResult](docs/RequestPasswordResetResult.md)
- [ResetCustomerAccountRequest](docs/ResetCustomerAccountRequest.md)
- [RevenueTrendPoint](docs/RevenueTrendPoint.md)
- [RolePermissionAction](docs/RolePermissionAction.md)
- [RolePermissionsModel](docs/RolePermissionsModel.md)
- [SetNewPasswordRequest](docs/SetNewPasswordRequest.md)
- [TeamMember](docs/TeamMember.md)
- [TeamsModel](docs/TeamsModel.md)
- [ToggleTwoFactorRequest](docs/ToggleTwoFactorRequest.md)
- [UpdateAppConfigRequest](docs/UpdateAppConfigRequest.md)
- [UpdateCartItemRequest](docs/UpdateCartItemRequest.md)
- [UpdateComplaintRequest](docs/UpdateComplaintRequest.md)
- [UpdateCustomerAddressCommand](docs/UpdateCustomerAddressCommand.md)
- [UpdateCustomerRequest](docs/UpdateCustomerRequest.md)
- [UpdateEmailRequest](docs/UpdateEmailRequest.md)
- [UpdateInventoryRequest](docs/UpdateInventoryRequest.md)
- [UpdateNotificationPreferencesRequest](docs/UpdateNotificationPreferencesRequest.md)
- [UpdateNotificationsCommand](docs/UpdateNotificationsCommand.md)
- [UpdateOrderRequest](docs/UpdateOrderRequest.md)
- [UpdateProfileRequest](docs/UpdateProfileRequest.md)
- [UpdateRoleRequest](docs/UpdateRoleRequest.md)
- [UpdateTeamMemberRequest](docs/UpdateTeamMemberRequest.md)
- [UpdateUploadRequest](docs/UpdateUploadRequest.md)
- [UploadsModel](docs/UploadsModel.md)
- [UpsertNotificationPreferenceRequest](docs/UpsertNotificationPreferenceRequest.md)
- [UserProfile](docs/UserProfile.md)
- [UserRolesModel](docs/UserRolesModel.md)
- [VerifyAccountRequest](docs/VerifyAccountRequest.md)
- [VerifyCustomerOtpRequest](docs/VerifyCustomerOtpRequest.md)
- [VerifyOtpQuery](docs/VerifyOtpQuery.md)
- [VerifyOtpResponse](docs/VerifyOtpResponse.md)
- [VerifyOtpVariant](docs/VerifyOtpVariant.md)

### Authorization


Authentication schemes defined for the API:
<a id="bearerAuth"></a>
#### bearerAuth


- **Type**: HTTP Bearer Token authentication (JWT)

## About

This TypeScript SDK client supports the [Fetch API](https://fetch.spec.whatwg.org/)
and is automatically generated by the
[OpenAPI Generator](https://openapi-generator.tech) project:

- API version: `0.6.2`
- Package version: `0.6.2`
- Generator version: `7.26.0`
- Build package: `org.openapitools.codegen.languages.TypeScriptFetchClientCodegen`

The generated npm module supports the following:

- Environments
  * Node.js
  * Webpack
  * Browserify
- Language levels
  * ES5 - you must have a Promises/A+ library installed
  * ES6
- Module systems
  * CommonJS
  * ES6 module system


## Development

### Building

To build the TypeScript source code, you need to have Node.js and npm installed.
After cloning the repository, navigate to the project directory and run:

```bash
npm install
npm run build
```

### Publishing

Once you've built the package, you can publish it to npm:

```bash
npm publish
```

## License

[]()
