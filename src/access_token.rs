use serde_derive::{Deserialize, Serialize};
use crate::http::HttpClient;
use crate::WebArenaIndigoApiError;

pub struct AccessTokenApi {}

impl AccessTokenApi {
    pub async fn access_token_generate(client_id: &str, client_secret: &str) -> Result<AccessTokenGenerateResponse, WebArenaIndigoApiError> {
        Ok(
            // https://help.salesforce.com/s/articleView?id=sf.remoteaccess_oauth_client_credentials_flow.htm&type=5
            HttpClient::post(
                "",
                "https://api.customer.jp/oauth/v1/accesstokens",
                AccessTokenGenerateRequest {
                    grant_type: "client_credentials".to_string(),
                    client_id: client_id.to_string(),
                    client_secret: client_secret.to_string(),
                    code: "".to_string(),
                },
            ).await?
        )
    }
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AccessTokenGenerateResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "tokenType")]
    pub token_type: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: String,
    pub scope: String,
    #[serde(rename = "issuedAt")]
    pub issued_at: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
struct AccessTokenGenerateRequest {
    #[serde(rename = "grantType")]
    grant_type: String,
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
    code: String,
}