use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage, HttpResponse};
use futures::future::{ok, Ready};
use futures::Future;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    #[serde(rename = "type")]
    pub token_type: String,
}

pub struct Auth {
    jwt_secret: String,
}

impl Auth {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware {
            service,
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct AuthMiddleware<S> {
    service: S,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");

        if auth_header.is_none() {
            return Box::pin(async {
                let response = HttpResponse::Unauthorized().json(serde_json::json!({
                    "success": false,
                    "data": serde_json::Value::Null,
                    "error": "unauthorized",
                    "message": "Missing authorization header",
                    "meta": serde_json::Value::Null,
                }));
                Err(actix_web::error::InternalError::from_response("", response).into())
            });
        }

        let auth_str = auth_header.unwrap().to_str().unwrap_or("");
        let token = auth_str.strip_prefix("Bearer ").unwrap_or("");

        let jwt_secret = self.jwt_secret.clone();
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(token_data) => {
                if token_data.claims.token_type != "access" {
                    return Box::pin(async {
                        let response = HttpResponse::Unauthorized().json(serde_json::json!({
                            "success": false,
                            "data": serde_json::Value::Null,
                            "error": "unauthorized",
                            "message": "Invalid token type",
                            "meta": serde_json::Value::Null,
                        }));
                        Err(actix_web::error::InternalError::from_response("", response).into())
                    });
                }
                
                req.extensions_mut().insert(token_data.claims);
                let fut = self.service.call(req);
                Box::pin(async move { fut.await })
            }
            Err(err) => {
                tracing::warn!("JWT validation failed: {:?}", err);
                Box::pin(async {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "success": false,
                        "data": serde_json::Value::Null,
                        "error": "unauthorized",
                        "message": "Invalid or expired token",
                        "meta": serde_json::Value::Null,
                    }));
                    Err(actix_web::error::InternalError::from_response("", response).into())
                })
            }
        }
    }
}