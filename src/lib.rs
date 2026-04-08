use reqwest::{Client, Response};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur while interacting with SSLCommerz APIs.
#[derive(Debug, Error)]
pub enum SSLError {
    /// HTTP-level error from reqwest
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Unsupported HTTP method used internally
    #[error("Invalid method")]
    InvalidMethod,
}

/// Client for interacting with the SSLCommerz payment gateway.
///
/// This struct provides methods for:
/// - Creating payment sessions
/// - Validating transactions (IPN)
/// - Querying transactions
/// - Initiating and checking refunds
///
/// # Example
///
/// ```
/// use sslcommerz_rs::SSLCommerz;
///
/// let client = SSLCommerz::new("store_id", "store_pass", true);
/// ```
pub struct SSLCommerz {
    store_id: String,
    store_pass: String,
    create_session_url: String,
    validation_url: String,
    transaction_url: String,
    client: Client,
}

impl SSLCommerz {
    /// Creates a new SSLCommerz client.
    ///
    /// # Arguments
    ///
    /// * `store_id` - Your SSLCommerz store ID
    /// * `store_pass` - Your SSLCommerz store password
    /// * `issandbox` - `true` for sandbox environment, `false` for live
    ///
    /// # Example
    ///
    /// ```
    /// let client = SSLCommerz::new("testbox", "qwerty", true);
    /// ```
    pub fn new(store_id: &str, store_pass: &str, issandbox: bool) -> Self {
        let mode = if issandbox { "sandbox" } else { "securepay" };
        let base = format!("https://{}.sslcommerz.com", mode);

        Self {
            store_id: store_id.to_string(),
            store_pass: store_pass.to_string(),
            create_session_url: format!("{}/gwprocess/v4/api.php", base),
            validation_url: format!("{}/validator/api/validationserverAPI.php", base),
            transaction_url: format!("{}/validator/api/merchantTransIDvalidationAPI.php", base),
            client: Client::new(),
        }
    }

    /// Creates a payment session.
    ///
    /// This initiates a payment request and returns a response containing
    /// a `GatewayPageURL` where the user should be redirected.
    ///
    /// # Arguments
    ///
    /// * `post_body` - Key-value map of required payment parameters
    ///
    /// # Returns
    ///
    /// JSON response from SSLCommerz API
    ///
    /// # Example
    ///
    /// ```
    /// # use sslcommerz_rs::SSLCommerz;
    /// # use std::collections::HashMap;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = SSLCommerz::new("testbox", "qwerty", true);
    ///
    /// let mut payload = HashMap::new();
    /// payload.insert("total_amount".into(), "100".into());
    ///
    /// let res = client.create_session(payload).await.unwrap();
    /// println!("{:?}", res);
    /// # }
    /// ```
    pub async fn create_session(
        &self,
        mut post_body: HashMap<String, String>,
    ) -> Result<Value, SSLError> {
        post_body.insert("store_id".into(), self.store_id.clone());
        post_body.insert("store_passwd".into(), self.store_pass.clone());

        self.call_api("POST", &self.create_session_url, &post_body)
            .await
    }

    /// Validates a transaction using a validation ID (`val_id`).
    ///
    /// Typically used after IPN verification to confirm payment status.
    pub async fn validation_transaction_order(
        &self,
        validation_id: &str,
    ) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("val_id".into(), validation_id.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.validation_url, &params).await
    }

    /// Initiates a refund request.
    ///
    /// ⚠️ Note: SSLCommerz uses `GET` for refund initiation (non-standard API design).
    ///
    /// # Arguments
    ///
    /// * `bank_tran_id` - Bank transaction ID
    /// * `refund_amount` - Amount to refund
    /// * `refund_remarks` - Reason for refund
    pub async fn init_refund(
        &self,
        bank_tran_id: &str,
        refund_amount: &str,
        refund_remarks: &str,
    ) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("bank_tran_id".into(), bank_tran_id.into());
        params.insert("refund_amount".into(), refund_amount.into());
        params.insert("refund_remarks".into(), refund_remarks.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params).await
    }

    /// Queries refund status using `refund_ref_id`.
    pub async fn query_refund_status(&self, refund_ref_id: &str) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("refund_ref_id".into(), refund_ref_id.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params).await
    }

    /// Retrieves transaction details using session key.
    pub async fn transaction_query_session(&self, sessionkey: &str) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("sessionkey".into(), sessionkey.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params).await
    }

    /// Retrieves transaction details using transaction ID.
    pub async fn transaction_query_tranid(&self, tran_id: &str) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("tran_id".into(), tran_id.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params).await
    }

    /// Validates IPN (Instant Payment Notification) using hash verification.
    ///
    /// This ensures the request originates from SSLCommerz and has not been tampered with.
    ///
    /// # Returns
    ///
    /// `true` if hash matches, otherwise `false`
    pub fn hash_validate_ipn(&self, post_body: &HashMap<String, String>) -> bool {
        if !(post_body.contains_key("verify_key") && post_body.contains_key("verify_sign")) {
            return false;
        }

        let verify_keys: Vec<&str> = post_body["verify_key"].split(',').collect();

        let mut new_params: Vec<(String, String)> = verify_keys
            .iter()
            .filter_map(|k| post_body.get(*k).map(|v| ((*k).to_string(), v.clone())))
            .collect();

        let hashed_pass = format!("{:x}", md5::compute(self.store_pass.as_bytes()));

        new_params.push(("store_passwd".into(), hashed_pass));
        new_params.sort_by(|a, b| a.0.cmp(&b.0));

        let hash_string = new_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let generated_hash = format!("{:x}", md5::compute(hash_string));

        match post_body.get("verify_sign") {
            Some(sig) => sig == &generated_hash,
            None => false,
        }
    }

    /// Internal helper for making HTTP API calls.
    ///
    /// Supports `GET`, `POST`, `PUT`, and `DELETE`.
    async fn call_api(
        &self,
        method: &str,
        url: &str,
        payload: &HashMap<String, String>,
    ) -> Result<Value, SSLError> {
        let res: Response = match method {
            "POST" => self.client.post(url).form(payload).send().await?,
            "GET" => self.client.get(url).query(payload).send().await?,
            "PUT" => self.client.put(url).form(payload).send().await?,
            "DELETE" => self.client.delete(url).send().await?,
            _ => return Err(SSLError::InvalidMethod),
        };

        let json = res.json::<Value>().await?;
        Ok(json)
    }
}