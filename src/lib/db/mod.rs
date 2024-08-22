pub mod users_db;
pub mod exercises_db;
pub mod workout_templates_db;
pub mod wk_template_elements_db;


use std::env;
//  use diesel::{ConnectionError, ConnectionResult};
//  use bb8::Pool;
//  use diesel_async::pooled_connection::ManagerConfig;
use diesel_async::AsyncPgConnection;
use tracing::{error, info};


use crate::lib::errors::DBError;


pub struct DBConnector {
    pub test_endpoint: Option<String>,
}

impl DBConnector {
    pub async fn rds_connection(&self) -> Result<AsyncPgConnection, DBError> {
        if self.test_endpoint.is_none() {
            self.establish_proxy_connection().await
        } else {
            self.establish_test_pqsql_connection().await
        }
    }

    async fn establish_proxy_connection(&self) -> Result<AsyncPgConnection, DBError> {

        // Read environment variables
        let db_hostname = env::var("DB_HOSTNAME").map_err(|e| DBError::EnvError(e.to_string()))?;
        let port = env::var("DB_PORT")
            .map_err(|e| DBError::EnvError(e.to_string()))?
            .parse::<u16>()
            .map_err(|e| DBError::EnvError(e.to_string()))?;
        let db_username = env::var("DB_USERNAME").map_err(|e| DBError::EnvError(e.to_string()))?;
        let db_password = env::var("DB_PASSWORD").map_err(|e| DBError::EnvError(e.to_string()))?;
        let db_name = env::var("DB_NAME").map_err(|e| DBError::EnvError(e.to_string()))?;

        // Establish TLS
        let root_certs = root_certs()?;
        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_certs)
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        info!("HOST, USERNAME = {}, {}", db_hostname, db_username);
        let (client, conn) = tokio_postgres::Config::new()
            .host(&db_hostname)
            .port(port)
            .user(&db_username)
            .password(&db_password)
            .dbname(&db_name)
            .connect_timeout(std::time::Duration::from_secs(30)).connect(tls)
            .await
            .map_err(|e| {
                error!("Couldn't establish connection with database: {}", e.to_string());
                DBError::ConnectionError(e.to_string())
            })?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                error!("Database connection: {}", e);
            }
        });

        let connection = AsyncPgConnection::try_from(client).await
            .map_err(|error| DBError::ConnectionError(error.to_string()))?;
        Ok(connection)
    }


    async fn establish_test_pqsql_connection(&self) -> Result<AsyncPgConnection, DBError> {
        if self.test_endpoint.is_none() {
            return Err(DBError::OperationError(String::from("Test endpoint is not set")));
        }

        let (client, connection) = tokio_postgres::connect(&self.test_endpoint.clone().unwrap(), tokio_postgres::NoTls)
            .await.map_err(|e| {
                error!("Couldn't establish connection with database: {}", e.to_string());
                DBError::ConnectionError(e.to_string())
            })?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let connection = AsyncPgConnection::try_from(client).await
            .map_err(|error| DBError::ConnectionError(error.to_string()))?;
        Ok(connection)
    }



}


impl Default for DBConnector {
    fn default() -> Self {
        DBConnector {
            test_endpoint: None
        }
    }
}


fn root_certs() -> Result<rustls::RootCertStore, DBError> {
    let mut roots = rustls::RootCertStore::empty();
    let certs = rustls_native_certs::load_native_certs() 
        .map_err(|_| DBError::ConnectionError("Could not load native certs".to_string()))?;
    roots.add_parsable_certificates(certs);
    Ok(roots)
}



//  async fn generate_rds_iam_token(
//      db_hostname: &str,
//      db_port: u16,
//      db_username: &str
//  ) ->Result<String, DBError> {



//      let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

//      let credentials = config
//          .credentials_provider()
//          .expect("no credentials provider found")
//          .provide_credentials()
//          .await
//          .expect("unable to load credentials");
//      let identity = credentials.into();
//      let region = config.region().unwrap().to_string();

//      let mut signing_settings = SigningSettings::default();
//      signing_settings.expires_in = Some(Duration::from_secs(900));
//      signing_settings.signature_location = aws_sigv4::http_request::SignatureLocation::QueryParams;

//      let signing_params = v4::SigningParams::builder()
//          .identity(&identity)
//          .region(&region)
//          .name("rds-db")
//          .time(SystemTime::now())
//          .settings(signing_settings)
//          .build()
//          .map_err(|error| DBError::AuthError(error.to_string()))?;

//      let url = format!(
//          "https://{db_hostname}:{db_port}/?Action=connect&DBUser={db_user}",
//          db_hostname = db_hostname,
//          db_port = db_port,
//          db_user = db_username
//      );

//      let signable_request =
//          SignableRequest::new("GET", &url, std::iter::empty(), SignableBody::Bytes(&[]))
//          .expect("signable request");

//      let (signing_instructions, _signature) = sign(signable_request, &signing_params.into())
//          .map_err(|error| DBError::AuthError(error.to_string()))?.into_parts();

//      let mut url = url::Url::parse(&url).unwrap();
//      for (name, value) in signing_instructions.params() {
//          url.query_pairs_mut().append_pair(name, &value);
//      }

//      let response = url.to_string().split_off("https://".len());

//      Ok(response)
//  }
