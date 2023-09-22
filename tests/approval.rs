use borges::generate_app;
use serde::Serialize;
use serde_json::Value;
use sqlx::{Pool, Postgres};
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

const NOTE_MUTATION: &str = "
    mutation {
      addNote(input: {bookId: 1, note: \"new note!\", page: 3}) {
        id
        bookId
        note
        page
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

#[sqlx::test]
async fn test_graphql(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let app = generate_app(pool).await.unwrap();

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
    let result: Value = serde_json::from_slice(&body).unwrap();
    insta::assert_json_snapshot!(result);

    Ok(())
}

#[sqlx::test]
async fn test_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let app = generate_app(pool).await.unwrap();

    let body = Request {
        query: NOTE_MUTATION.to_string(),
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
    let result: Value = serde_json::from_slice(&body).unwrap();
    insta::assert_json_snapshot!(result);

    Ok(())
}
