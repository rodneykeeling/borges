use borges::generate_app;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

const BOOK_QUERY: &str = "
    query {
      book(bookId: 1) {
        id
        title
        author
        year
        pages
        notes {
          id
          bookId
          note
          page
        }
      }
    }
";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    query: String,
    operation_name: Option<String>,
    variables: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BookResponse {
    data: Data,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    book: Book,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub image_url: Option<String>,
    pub year: i32,
    pub pages: i32,
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: i32,
    pub book_id: i32,
    pub note: String,
    pub page: Option<i32>,
}

#[tokio::test]
async fn test_graphql() {
    let app = generate_app().await.unwrap();

    let body = Request {
        query: BOOK_QUERY.to_string(),
        operation_name: None,
        variables: None,
    };
    let resp = app
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::POST)
                .uri("/")
                .body(axum::body::Body::from(
                    serde_json::to_string(&body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let result: BookResponse = serde_json::from_slice(&body).unwrap();
    insta::assert_json_snapshot!(result);
}
