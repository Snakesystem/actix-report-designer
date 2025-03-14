use actix_web::{error, web, HttpRequest, HttpResponse, Responder};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone as _, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use tiberius::QueryStream;

use crate::backend::context::crypto::encrypt_text;

pub struct GenericService;

#[derive(Debug, Serialize)]
pub struct ActionResult<T, E> {
    pub result: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<E>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WebUser {
    pub auth_usernid: i32,
    pub email: String,
    pub mobile_phone: String,
    pub disabled_login: bool,
    pub picture: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub register_date: chrono::DateTime<Utc>
}

fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&formatted)
}

fn deserialize_date_only<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let date_str: Option<String> = Option::deserialize(deserializer)?;
    if let Some(date) = date_str {
        let naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
            .map_err(serde::de::Error::custom)?;
        let datetime = Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap());
        return Ok(Some(datetime));
    }
    Ok(None)
}

// Implementasi Default
impl<T, E> Default for ActionResult<T, E> {
    fn default() -> Self {
        Self {
            result: false, // Default-nya false
            message: String::new(),
            data: None,
            error: None,
        }
    }
}

impl GenericService {
    pub async fn not_found(req: HttpRequest) -> impl Responder {
        HttpResponse::NotFound().json({
            json!({
                "result": false,
                "message": "Not Found",
                "error": format!("Url '{}' not found. Please check the URL.", req.path())
            })
        })
    }
    
    pub fn json_error_handler(err: error::JsonPayloadError, _req: &actix_web::HttpRequest) -> actix_web::Error {
        let error_message = format!("Json deserialize error: {}", err);

        let result = ActionResult::<String, _> { // <- Ubah dari ActionResult<()> ke ActionResult<String>
            result: false,
            message: "Invalid Request".to_string(),
            error: Some(error_message), // <- Sekarang cocok karena `data: Option<String>`
            data: None,
        };

        error::InternalError::from_response(err, HttpResponse::BadRequest().json(result)).into()
    } 

    pub async fn login(connection: web::Data<Pool<ConnectionManager>>,request: LoginRequest) -> ActionResult<WebUser, String> {
        
        let mut result: ActionResult<WebUser, String> = ActionResult::default();
        let enc_password = encrypt_text(request.password.unwrap_or_default());

        match connection.clone().get().await {
            Ok(mut conn) => {
                let query_result: Result<QueryStream, _> = conn.query(
                    r#"SELECT AuthUserNID, Email, Handphone, disableLogin, Picture, RegisterDate FROM AuthUser 
                    WHERE Email = @P1 AND Password = @P2"#, &[&request.email, &enc_password]).await;
                match query_result {
                    Ok(rows) => {
                        if let Ok(Some(row)) = rows.into_row().await {
                            result.result = true;
                            result.message = format!("Welcome {}", request.email.unwrap_or_default());
                            result.data = Some(WebUser{
                                auth_usernid: row.get("AuthUserNID").unwrap_or(0),
                                email: row.get::<&str, _>("Email").map_or_else(|| "".to_string(), |s| s.to_string()),
                                mobile_phone: row.get::<&str, _>("Handphone").map_or_else(|| "".to_string(), |s| s.to_string()),
                                disabled_login: row.get("disableLogin").unwrap_or(false),
                                picture: Some(row.get::<&str, _>("Picture").map_or_else(|| "".to_string(), |s| s.to_string())),
                                register_date: row
                                    .get::<NaiveDateTime, _>("RegisterDate")
                                    .map(|dt| dt.and_utc()) // 🔥 Konversi ke DateTime<Utc>
                                    .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap()), // Default jika kosong
                            }); 

                            return result;
                        } else {
                            result.message = format!("No user found for email: {}", request.email.unwrap_or_default());
                            return result;
                        } 
                    },
                    Err(err) => {
                        result.error = format!("Query execution failed: {:?}", err).into();
                        return result;
                    },
                }
            },
            Err(err) => {
                result.error = format!("Internal Server error: {:?}", err).into();
                return result;
            }, 
        }
    }
}