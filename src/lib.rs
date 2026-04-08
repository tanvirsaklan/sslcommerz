use reqwest::{Client, Response};
use serde_json::Value;
use std::collections::HashMap;
use md5;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SSLError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid method")]
    InvalidMethod,
}

pub struct SSLCommerz {
    store_id: String,
    store_pass: String,
    create_session_url: String,
    validation_url: String,
    transaction_url: String,
    client: Client,
}

impl SSLCommerz {
    pub fn new(store_id: &str, store_pass: &str, issandbox: bool) -> Self {
        let mode = if issandbox { "sandbox" } else { "securepay" };

        let base = format!("https://{}.sslcommerz.com", mode);

        Self {
            store_id: store_id.to_string(),
            store_pass: store_pass.to_string(),
            create_session_url: format!("{}/gwprocess/v4/api.php", base),
            validation_url: format!("{}/validator/api/validationserverAPI.php", base),
            transaction_url: format!(
                "{}/validator/api/merchantTransIDvalidationAPI.php",
                base
            ),
            client: Client::new(),
        }
    }

    // -------------------------
    // Create Session
    // -------------------------
    pub async fn create_session(
        &self,
        mut post_body: HashMap<String, String>,
    ) -> Result<Value, SSLError> {
        post_body.insert("store_id".into(), self.store_id.clone());
        post_body.insert("store_passwd".into(), self.store_pass.clone());

        self.call_api("POST", &self.create_session_url, &post_body)
            .await
    }

    // -------------------------
    // Validate Transaction
    // -------------------------
    pub async fn validation_transaction_order(
        &self,
        validation_id: &str,
    ) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("val_id".into(), validation_id.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.validation_url, &params)
            .await
    }

    // -------------------------
    // Refund Initiate
    // -------------------------
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

        self.call_api("GET", &self.transaction_url, &params)
            .await
    }

    // -------------------------
    // Refund Status
    // -------------------------
    pub async fn query_refund_status(
        &self,
        refund_ref_id: &str,
    ) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("refund_ref_id".into(), refund_ref_id.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params)
            .await
    }

    // -------------------------
    // Query by Session
    // -------------------------
    pub async fn transaction_query_session(
        &self,
        sessionkey: &str,
    ) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("sessionkey".into(), sessionkey.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params)
            .await
    }

    // -------------------------
    // Query by Tran ID
    // -------------------------
    pub async fn transaction_query_tranid(
        &self,
        tran_id: &str,
    ) -> Result<Value, SSLError> {
        let mut params = HashMap::new();

        params.insert("tran_id".into(), tran_id.into());
        params.insert("store_id".into(), self.store_id.clone());
        params.insert("store_passwd".into(), self.store_pass.clone());
        params.insert("format".into(), "json".into());

        self.call_api("GET", &self.transaction_url, &params)
            .await
    }

    // -------------------------
    // Hash Validation (IPN)
    // -------------------------
    pub fn hash_validate_ipn(
        &self,
        post_body: &HashMap<String, String>,
    ) -> bool {
        if !(post_body.contains_key("verify_key")
            && post_body.contains_key("verify_sign"))
        {
            return false;
        }

        let verify_keys: Vec<&str> = post_body["verify_key"].split(',').collect();

        let mut new_params: Vec<(String, String)> = verify_keys
            .iter()
            .filter_map(|k| {
                post_body.get(*k).map(|v| ((*k).to_string(), v.clone()))
            })
            .collect();

        let hashed_pass =
            format!("{:x}", md5::compute(self.store_pass.as_bytes()));

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

    // -------------------------
    // Core API Caller
    // -------------------------
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