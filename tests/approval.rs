use borges::generate_app;
use serde::Serialize;
use serde_json::Value;
use sqlx::{Pool, Postgres};
use tower::ServiceExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    query: String,
    operation_name: Option<String>,
    variables: Option<String>,
}

async fn _run_request(request_body: Request, pool: Pool<Postgres>) -> Value {
    let app = generate_app(pool).await.unwrap();

    let resp = app
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::POST)
                .uri("/")
                .body(axum::body::Body::from(
                    serde_json::to_string(&request_body).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = hyper::body::to_bytes(resp.into_body()).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();

    result
}

#[sqlx::test]
async fn test_book_query(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let book_query = "
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
    "
    .to_string();
    let body = Request {
        query: book_query,
        operation_name: None,
        variables: None,
    };

    let result = _run_request(body, pool).await;
    insta::assert_json_snapshot!(result);

    Ok(())
}

#[sqlx::test]
async fn test_add_note_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let mutation = "
        mutation {
          addNote(input: {bookId: 1, note: \"new note!\", page: 3}) {
            id
            bookId
            note
            page
          }
        }
    "
    .to_string();

    let body = Request {
        query: mutation,
        operation_name: None,
        variables: None,
    };

    let result = _run_request(body, pool).await;
    insta::assert_json_snapshot!(result);

    Ok(())
}

#[sqlx::test]
async fn test_invalid_book_id_note_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let mutation = "
        mutation {
          addNote(input: {bookId: 9999, note: \"new note!\", page: 3}) {
            id
            bookId
            note
            page
          }
        }
    "
    .to_string();

    let body = Request {
        query: mutation,
        operation_name: None,
        variables: None,
    };

    let result = _run_request(body, pool).await;
    insta::assert_json_snapshot!(result);

    Ok(())
}

#[sqlx::test]
async fn test_invalid_page_note_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let mutation = "
        mutation {
          addNote(input: {bookId: 1, note: \"new note!\", page: 9999}) {
            id
            bookId
            note
            page
          }
        }
    "
    .to_string();

    let body = Request {
        query: mutation,
        operation_name: None,
        variables: None,
    };

    let result = _run_request(body, pool).await;
    insta::assert_json_snapshot!(result);

    Ok(())
}

#[sqlx::test]
async fn test_invalid_negative_book_page_note_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    let mutation = "
        mutation {
          addNote(input: {bookId: 1, note: \"new note!\", page: -5}) {
            id
            bookId
            note
            page
          }
        }
    "
    .to_string();

    let body = Request {
        query: mutation,
        operation_name: None,
        variables: None,
    };

    let result = _run_request(body, pool).await;
    insta::assert_json_snapshot!(result);

    Ok(())
}
