use reqwest;
use std::env;
use dotenv::dotenv;
use serde::Deserialize;

// const DOMAIN: &str = "https://auth.salamandra-app.com"
const DOMAIN: &str = "http://localhost:8080";

pub struct KeycloakClient {
    pub token: Option<String>,
    pub token_expires: i64,
}

impl KeycloakClient {

    /// Gets token or calls for a new if its expired
    async fn get_token(&mut self) -> Result<&String, KeycloakError> {
        
        let now: i64 = chrono::Utc::now().timestamp();

        if self.token.is_none() || now >= self.token_expires {
            // Request a new token from Keycloak and set expiration
            let new_token = self.request_new_token().await?;
            self.token = Some(new_token.token);
            self.token_expires = now + new_token.expires_in;
        }

        Ok(self.token.as_ref().unwrap())
        
    }

    // TODO
    /// Calls KC for a new access token 
    async fn request_new_token(&self) -> Result<NewTokenResponse, KeycloakError> {
        dotenv().ok();
        let client = reqwest::Client::new();
        let realm_name: String = env::var("KC_REALM_NAME").expect("KC_REALM_NAME must be set");
        let client_id: String = env::var("KC_CLIENT_ID").expect("KC_CLIENT_ID must be set");
        let client_secret: String = env::var("KC_CLIENT_SECRET").expect("KC_CLIENT_SECRET must be set");

        let url = format!("{}/realms/{}/protocol/openid-connect/token", DOMAIN, realm_name);

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ];

        match client.get(url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Err(KeycloakError::ResponseError(response.status().to_string()))
                    }
                    match response.json::<NewTokenResponse>().await {
                        Ok(new_token) => Ok(new_token),
                        Err(_) => Err(KeycloakError::InternalServerError)
                    }
                },
                Err(error) => Err(KeycloakError::RequestSendError(error.to_string())),
            }
    }


    pub async fn get_user_info(&mut self, user_name: String) -> Result<UserInfo, KeycloakError> {
        dotenv().ok();
        let access_token: &String = self.get_token().await?;
        let client = reqwest::Client::new();
        let realm_name: String = env::var("KC_REALM_NAME").expect("KC_REALM_NAME must be set");

        let url = format!("{}/admin/realms/{}/users?username={}", DOMAIN, realm_name, user_name);
        match client.get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Err(KeycloakError::ResponseError(response.status().to_string()))
                    }
                    match response.json::<UserInfo>().await {
                        Ok(user_info) => Ok(user_info),
                        Err(_) => Err(KeycloakError::InternalServerError)
                    }
                },
                Err(error) => Err(KeycloakError::RequestSendError(error.to_string())),
            }

    }
    
    /// Given uuid, removes user from KC database 
    pub async fn delete_user(&mut self, user_id: String) -> Result<(), KeycloakError> {
        dotenv().ok();
        let access_token: &String = self.get_token().await?;
        let client = reqwest::Client::new();
        let realm_name: String = env::var("KC_REALM_NAME").expect("KC_REALM_NAME must be set");

        let url = format!("{}/admin/realms/{}/users/{}", DOMAIN, realm_name, user_id);
        match client.delete(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await {
                Ok(response) => {
                    if !response.status().is_success() {
                        return Err(KeycloakError::ResponseError(response.status().to_string()))
                    };
                    Ok(())
                },
                Err(error) => Err(KeycloakError::RequestSendError(error.to_string())),
            }
    }
}

pub enum KeycloakError {
    RequestSendError(String),
    ResponseError(String),
    InternalServerError,
}

#[derive(Deserialize, Debug)]
struct NewTokenResponse {
    token: String,
    expires_in: i64,
    /*
    refresh_expires_in: i64,
    token_type: String,
    #[serde(rename = "not-before-policy")]
    not_before_policy: i64,
    scope: String,
    */
}


#[derive(Deserialize, Debug)]
pub struct UserInfo {
    pub id: uuid::Uuid,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    /*
    #[serde(rename = "createdTimestamp")]
    created_timestamp: i64,
    username: String,
    enabled: bool,
    totp: bool,
    email: String,
    #[serde(rename = "disableableCredentialTypes")]
    disableable_credential_types: Vec<String>,
    #[serde(rename = "requiredActions")]
    required_actions: Vec<String>,
    #[serde(rename = "notBefore")]
    not_before: i32,
    access: Access,
    */
}

/*
#[derive(Deserialize, Debug)]
struct Access {
    #[serde(rename = "manageGroupMembership")]
    manage_group_membership: bool,
    view: bool,
    #[serde(rename = "mapRoles")]
    map_roles: bool,
    impersonate: bool,
    manage: bool,
}
*/
