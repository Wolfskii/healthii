use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{error::AppError, state::AppState, storage, users::AuthUser};

pub const ALLOWED_MIME_TYPES: &[&str] =
    &["application/pdf", "image/png", "image/jpeg", "image/webp"];

const MAX_BYTES: usize = 15 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Document {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub document_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct DocumentRow {
    id: Uuid,
    user_id: Uuid,
    filename: String,
    mime_type: String,
    size: i64,
    storage_key: String,
    document_type: String,
    title: Option<String>,
    description: Option<String>,
    document_date: Option<NaiveDate>,
    uploaded_at: DateTime<Utc>,
}

impl From<DocumentRow> for Document {
    fn from(row: DocumentRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            filename: row.filename,
            mime_type: row.mime_type,
            size: row.size,
            document_type: row.document_type,
            title: row.title,
            description: row.description,
            document_date: row.document_date,
            uploaded_at: row.uploaded_at,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/documents", get(list).post(upload))
        .route("/documents/{id}", get(get_one).delete(delete))
        .route("/documents/{id}/file", get(download))
        .layer(DefaultBodyLimit::max(MAX_BYTES))
}

pub async fn list(
    user: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Document>>, AppError> {
    let rows = sqlx::query_as::<_, DocumentRow>(
        r#"
        SELECT id, user_id, filename, mime_type, size, storage_key, document_type,
               title, description, document_date, uploaded_at
        FROM documents
        WHERE user_id = $1
        ORDER BY uploaded_at DESC
        LIMIT 200
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows.into_iter().map(Document::from).collect()))
}

pub async fn get_one(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Document>, AppError> {
    let row = owned(&state, user.id, id).await?;
    Ok(Json(Document::from(row)))
}

pub async fn upload(
    user: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Document>), AppError> {
    let mut filename = String::new();
    let mut mime_type = String::new();
    let mut bytes: Option<Vec<u8>> = None;
    let mut document_type = "other".to_string();
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut document_date: Option<NaiveDate> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::validation("Invalid multipart body"))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field
                    .file_name()
                    .unwrap_or("document")
                    .chars()
                    .take(180)
                    .collect();
                mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::validation("Could not read the file"))?;
                if data.len() > MAX_BYTES {
                    return Err(AppError::validation("File is too large (15 MB maximum)"));
                }
                bytes = Some(data.to_vec());
            }
            "document_type" => {
                document_type = field.text().await.unwrap_or_else(|_| "other".into());
            }
            "title" => title = Some(field.text().await.unwrap_or_default()),
            "description" => description = Some(field.text().await.unwrap_or_default()),
            "document_date" => {
                let text = field.text().await.unwrap_or_default();
                document_date = NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok();
            }
            _ => {}
        }
    }

    let bytes = bytes.ok_or_else(|| AppError::validation("A file is required"))?;
    if !ALLOWED_MIME_TYPES.contains(&mime_type.as_str()) {
        return Err(AppError::validation(
            "Unsupported file type. Allowed: PDF, PNG, JPEG, WEBP",
        ));
    }
    let document_type = normalize_type(&document_type)?;
    let key = storage::document_key(user.id, &filename);
    let checksum = storage::sha256_hex(&bytes);
    state.storage.put(&key, bytes.clone()).await?;

    let row = sqlx::query_as::<_, DocumentRow>(
        r#"
        INSERT INTO documents (
            user_id, filename, mime_type, size, storage_key, checksum,
            document_type, title, description, document_date
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        RETURNING id, user_id, filename, mime_type, size, storage_key, document_type,
                  title, description, document_date, uploaded_at
        "#,
    )
    .bind(user.id)
    .bind(&filename)
    .bind(&mime_type)
    .bind(bytes.len() as i64)
    .bind(&key)
    .bind(&checksum)
    .bind(&document_type)
    .bind(trim_opt(title.as_deref(), 160))
    .bind(trim_opt(description.as_deref(), 2000))
    .bind(document_date)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(Document::from(row))))
}

pub async fn download(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let row = owned(&state, user.id, id).await?;
    let bytes = state.storage.get(&row.storage_key).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&row.mime_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    let filename = row.filename.replace(['"', '\n', '\r'], "_");
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        headers.insert(header::CONTENT_DISPOSITION, value);
    }
    Ok((headers, Bytes::from(bytes)))
}

pub async fn delete(
    user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let row = owned(&state, user.id, id).await?;
    let _ = state.storage.delete(&row.storage_key).await;
    sqlx::query("DELETE FROM documents WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.id)
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn recent(
    state: &AppState,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<Document>, AppError> {
    let rows = sqlx::query_as::<_, DocumentRow>(
        r#"
        SELECT id, user_id, filename, mime_type, size, storage_key, document_type,
               title, description, document_date, uploaded_at
        FROM documents
        WHERE user_id = $1
        ORDER BY uploaded_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(rows.into_iter().map(Document::from).collect())
}

pub async fn purge_for_user(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let keys =
        sqlx::query_scalar::<_, String>("SELECT storage_key FROM documents WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.pool)
            .await?;
    for key in keys {
        let _ = state.storage.delete(&key).await;
    }
    Ok(())
}

async fn owned(state: &AppState, user_id: Uuid, id: Uuid) -> Result<DocumentRow, AppError> {
    sqlx::query_as::<_, DocumentRow>(
        r#"
        SELECT id, user_id, filename, mime_type, size, storage_key, document_type,
               title, description, document_date, uploaded_at
        FROM documents
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Document not found"))
}

fn normalize_type(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "doctor_report" | "blood_test" | "prescription" | "imaging" | "vaccination"
        | "referral" | "discharge_summary" | "other" => Ok(value),
        _ => Err(AppError::validation("Unknown document type")),
    }
}

fn trim_opt(value: Option<&str>, max: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max).collect())
}
