# asterisk Go SDK

Go client for the asterisk API, generated from the OpenAPI spec with
[oapi-codegen](https://github.com/oapi-codegen/oapi-codegen).

## Installation

```bash
go get github.com/opeolluwa/asterisk/sdk-go
```

## Usage

```go
package main

import (
	"context"
	"fmt"
	"log"
	"net/http"

	asterisk "github.com/opeolluwa/asterisk/sdk-go"
)

func main() {
	// Add a bearer token to every request.
	bearerAuth := func(ctx context.Context, req *http.Request) error {
		req.Header.Set("Authorization", "Bearer YOUR_TOKEN")
		return nil
	}

	client, err := asterisk.NewClientWithResponses(
		"https://api.example.com",
		asterisk.WithRequestEditorFn(bearerAuth),
	)
	if err != nil {
		log.Fatal(err)
	}

	// Fetch the app config.
	resp, err := client.FetchAppConfigWithResponse(context.Background())
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Status: %d\n", resp.StatusCode())

	// Log in.
	loginResp, err := client.LoginWithResponse(context.Background(), asterisk.LoginJSONRequestBody{
		Email:    "admin@example.com",
		Password: "secret",
	})
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Login status: %d\n", loginResp.StatusCode())
}
```

## Regenerating

From the repository root:

```bash
just sdk      # regenerate all SDKs (Go + TypeScript)
just sdk-go   # regenerate Go SDK only
```
