use axum::{
    extract::Json,
    http::{HeaderValue, Method, StatusCode},
    routing::{get, post},
    Router,
};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
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

    let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
    let smtp_pass = std::env::var("SMTP_PASS").unwrap_or_default();
    let to_email  = std::env::var("TO_EMAIL").unwrap_or_else(|_| "incopter.info@gmail.com".into());

    let body = format!(
        "Νέο μήνυμα από τη φόρμα επικοινωνίας\n\
         ──────────────────────────────────────\n\
         Όνομα:     {}\n\
         Email:     {}\n\
         Τηλέφωνο:  {}\n\
         Υπηρεσία:  {}\n\
         ──────────────────────────────────────\n\
         Μήνυμα:\n\n{}\n",
        form.name, form.email,
        if form.phone.is_empty() { "—".into() } else { form.phone.clone() },
        if form.service.is_empty() { "—".into() } else { form.service.clone() },
        form.message,
    );

    let email = match Message::builder()
        .from(smtp_user.parse().unwrap_or_else(|_| "noreply@incopter.gr".parse().unwrap()))
        .to(to_email.parse().unwrap())
        .reply_to(form.email.parse().unwrap_or_else(|_| smtp_user.parse().unwrap()))
        .subject(format!("Incopter – Μήνυμα από {}", form.name))
        .header(ContentType::TEXT_PLAIN)
        .body(body)
    {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Failed to build email: {e}");
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Σφάλμα κατά τη δημιουργία email.");
        }
    };

    let creds   = Credentials::new(smtp_user, smtp_pass);
    let mailer  = match AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com") {
        Ok(t) => t.credentials(creds).build(),
        Err(e) => {
            tracing::error!("SMTP relay error: {e}");
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Σφάλμα σύνδεσης SMTP.");
        }
    };

    match mailer.send(email).await {
        Ok(_)  => ok("Το μήνυμά σας εστάλη επιτυχώς! Θα επικοινωνήσουμε μαζί σας σύντομα."),
        Err(e) => {
            tracing::error!("Failed to send email: {e}");
            err(StatusCode::INTERNAL_SERVER_ERROR, "Σφάλμα αποστολής. Δοκιμάστε ξανά ή επικοινωνήστε τηλεφωνικά.")
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
