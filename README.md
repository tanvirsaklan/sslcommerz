# 📦 SSLCOMMERZ - Rust ([sslcommerz - crates.io](https://crates.io/crates/sslcommerz))

Rust SDK for integrating SSLCommerz payment gateway into your applications.

---

## ⚠️ Environment Note

* `sandbox = true` → Sandbox environment
* `sandbox = false` → Live environment

🔗 Registration & credentials:
[https://developer.sslcommerz.com/registration/](https://developer.sslcommerz.com/registration/)

---

## 📥 Installation

Add to your project:

```bash
cargo add sslcommerz
```

Or manually in `Cargo.toml`:

```toml
sslcommerz = "0.1.0"
```

---

## 🔐 Authentication

You need:

* `store_id`
* `store_pass`

Available from your SSLCommerz merchant panel.

---

## 🚀 Usage

---

## Create Initial Payment Session

```rust
use sslcommerz::SSLCommerz;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let ssl = SSLCommerz::new("testbox", "qwerty", true);

    let mut payload = HashMap::new();

    payload.insert("total_amount".into(), "100.26".into());
    payload.insert("currency".into(), "BDT".into());
    payload.insert("tran_id".into(), "12345".into());

    payload.insert("success_url".into(), "https://example.com/success".into());
    payload.insert("fail_url".into(), "https://example.com/fail".into());
    payload.insert("cancel_url".into(), "https://example.com/cancel".into());

    payload.insert("emi_option".into(), "0".into());
    payload.insert("cus_name".into(), "test".into());
    payload.insert("cus_email".into(), "test@test.com".into());
    payload.insert("cus_phone".into(), "01700000000".into());

    payload.insert("cus_add1".into(), "customer address".into());
    payload.insert("cus_city".into(), "Dhaka".into());
    payload.insert("cus_country".into(), "Bangladesh".into());

    payload.insert("shipping_method".into(), "NO".into());
    payload.insert("multi_card_name".into(), "".into());
    payload.insert("num_of_item".into(), "1".into());

    payload.insert("product_name".into(), "Test".into());
    payload.insert("product_category".into(), "Test Category".into());
    payload.insert("product_profile".into(), "general".into());

    let response = ssl.create_session(payload).await.unwrap();

    println!("{:#?}", response);

    if let Some(url) = response.get("GatewayPageURL") {
        println!("Redirect user to: {}", url);
    }
}
```

---

## Validate Payment (IPN)

```rust
use sslcommerz_rs::SSLCommerz;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let ssl = SSLCommerz::new("test_testemi", "test_testemi@ssl", true);

    let mut post_body = HashMap::new();

    post_body.insert("tran_id".into(), "5E121A0D01F92".into());
    post_body.insert("val_id".into(), "200105225826116qFnATY9sHIwo".into());
    post_body.insert("amount".into(), "10.00".into());
    post_body.insert("status".into(), "VALID".into());

    post_body.insert("verify_sign".into(), "d42fab70ae0bcbda5280e7baffef60b0".into());
    post_body.insert("verify_key".into(), "amount,tran_id,val_id,status".into());

    if ssl.hash_validate_ipn(&post_body) {
        let response = ssl
            .validation_transaction_order(post_body.get("val_id").unwrap())
            .await
            .unwrap();

        println!("{:#?}", response);
    } else {
        println!("Hash validation failed");
    }
}
```

---

## Query Payment by Session Key

```rust
let response = ssl
    .transaction_query_session("A8EF93B75B8107E4F36049E80B4F9149")
    .await
    .unwrap();

println!("{:#?}", response);
```

---

## Query Payment by Transaction ID

```rust
let response = ssl
    .transaction_query_tranid("59C2A4F6432F8")
    .await
    .unwrap();

println!("{:#?}", response);
```

---

## Initiate Refund

```rust
let response = ssl
    .init_refund(
        "1709162345070ANJdZV8LyI4cMw",
        "5.50",
        "out of stock",
    )
    .await
    .unwrap();

println!("{:#?}", response);
```

---

## Query Refund Status

```rust
let response = ssl
    .query_refund_status("59bd63fea5455")
    .await
    .unwrap();

println!("{:#?}", response);
```

---

# ⚙️ Runtime Requirements

* Rust 1.70+
* Tokio runtime (`#[tokio::main]`)
* Internet connectivity

---

# ⚠️ Important Notes

### Hash Validation

* Uses **MD5** (SSLCommerz requirement)
* Do NOT reuse for general cryptography

---

### Payment Safety

You should implement:

* Idempotency (avoid duplicate charges/refunds)
* DB transaction logging
* Server-side validation (never trust client)

---

### Error Handling

Avoid `.unwrap()` in production:

```rust
match ssl.create_session(payload).await {
    Ok(res) => println!("{:?}", res),
    Err(err) => eprintln!("Error: {}", err),
}
```

---

# 🧪 Development Tips

* Use sandbox for testing
* Log all gateway responses
* Validate IPN before updating order state

---

# 👥 Contributors

> SSLCommerz

> [integration@sslcommerz.com](mailto:integration@sslcommerz.com)