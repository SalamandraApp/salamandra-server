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
    }

    // TODO
    /// Given uuid, removes user from KC database 
    async fn delete_user(&mut self, user_id: String) -> Result<(), KeycloakError> {
    }

    // TODO
    async fn get_user_info(&mut self, user_name: String) -> Result<UserInfo, KeycloakError> {
    }
}

struct NewTokenResponse {
    token: String,
    expires_in: i64,
}

// TODO
pub struct UserInfo {
    // no me acuerdo
}

enum KeycloakError {
    ConnectionError,
    InvalidClientCredentials,
    Unauthorized,
    TokenExpired,
    ResourceNotFound,
    RequestError,
}

