use axum::{
    extract::Json,
    http::{HeaderValue, Method, StatusCode},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct ContactForm {
    name:    String,
    email:   String,
    #[serde(default)]
    phone:   String,
    #[serde(default)]
    service: String,
    message: String,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

fn ok(msg: &str) -> (StatusCode, Json<ApiResponse>) {
    (StatusCode::OK, Json(ApiResponse { success: true, message: msg.to_string() }))
}

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ApiResponse>) {
    (status, Json(ApiResponse { success: false, message: msg.to_string() }))
}

async fn contact_handler(
    Json(form): Json<ContactForm>,
) -> (StatusCode, Json<ApiResponse>) {
    if form.name.trim().is_empty() || form.email.trim().is_empty() || form.message.trim().is_empty() {
        return err(StatusCode::BAD_REQUEST, "Παρακαλώ συμπληρώστε όλα τα υποχρεωτικά πεδία.");
    }

    let api_key  = std::env::var("RESEND_API_KEY").unwrap_or_default();
    let to_email = std::env::var("TO_EMAIL").unwrap_or_else(|_| "incopter.info@gmail.com".into());
    let from     = std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@incopter.gr".into());

    let body = format!(
        "<p><strong>Όνομα:</strong> {}</p>\
         <p><strong>Email:</strong> {}</p>\
         <p><strong>Τηλέφωνο:</strong> {}</p>\
         <p><strong>Υπηρεσία:</strong> {}</p>\
         <hr/>\
         <p><strong>Μήνυμα:</strong><br/>{}</p>",
        form.name,
        form.email,
        if form.phone.is_empty() { "—".into() } else { form.phone.clone() },
        if form.service.is_empty() { "—".into() } else { form.service.clone() },
        form.message.replace('\n', "<br/>"),
    );

    let payload = serde_json::json!({
        "from":    from,
        "to":      [to_email],
        "reply_to": form.email,
        "subject": format!("Incopter – Μήνυμα από {}", form.name),
        "html":    body,
    });

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(r) if r.status().is_success() => {
            ok("Το μήνυμά σας εστάλη επιτυχώς! Θα επικοινωνήσουμε μαζί σας σύντομα.")
        }
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            tracing::error!("Resend error {}: {}", status, text);
            err(StatusCode::INTERNAL_SERVER_ERROR, "Σφάλμα αποστολής. Δοκιμάστε ξανά ή επικοινωνήστε τηλεφωνικά.")
        }
        Err(e) => {
            tracing::error!("Resend request failed: {e}");
            err(StatusCode::INTERNAL_SERVER_ERROR, "Σφάλμα δικτύου. Δοκιμάστε ξανά ή επικοινωνήστε τηλεφωνικά.")
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:4321".into());

    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse::<HeaderValue>().expect("Invalid FRONTEND_URL"))
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/api/contact", post(contact_handler))
        .route("/health", get(|| async { "OK" }))
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{port}");

    tracing::info!("Backend listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
