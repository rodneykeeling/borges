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

// Macro to set the insta test snapshot suffix. This is needed to mark the tests correctly since
// insta names the snapshots after the `inner` function from `#[sqlx::test]`, marking all test
// snapshots as `approval__inner.snap`, `approval__inner-2.snap`, etc., which can end up with
// random ordering and clobbered snapshot file diffs.
//
// Pulled from https://insta.rs/docs/patterns/
macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
    }
}

#[sqlx::test]
async fn test_book_query(pool: Pool<Postgres>) -> sqlx::Result<()> {
    set_snapshot_suffix!("book_query");

    let book_query = "
        query {
          book(bookId: 1) {
            id
            title
            author
            year
            pages
            status
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
async fn test_all_books_query(pool: Pool<Postgres>) -> sqlx::Result<()> {
    set_snapshot_suffix!("all_books_query");

    let book_query = "
        query {
          books {
            id
            title
            author
            year
            pages
            status
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
async fn test_read_books_query(pool: Pool<Postgres>) -> sqlx::Result<()> {
    set_snapshot_suffix!("read_books_query");

    let book_query = "
        query {
          books(status: READ) {
            id
            title
            author
            year
            pages
            status
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
    set_snapshot_suffix!("add_note_mutation");

    let mutation = "
        mutation {
          addNote(input: {bookId: 1, note: \"new note!\", page: 3}) {
            note {
              id
              bookId
              note
              page
            }
            success
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
    set_snapshot_suffix!("invalid_book_id_note_mutation");

    let mutation = "
        mutation {
          addNote(input: {bookId: 9999, note: \"new note!\", page: 3}) {
            note {
              id
              bookId
              note
              page
            }
            success
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
    set_snapshot_suffix!("invalid_page_note_mutation");

    let mutation = "
        mutation {
          addNote(input: {bookId: 1, note: \"new note!\", page: 9999}) {
            note {
              id
              bookId
              note
              page
            }
            success
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
    set_snapshot_suffix!("invalid_negative_book_page_note_mutation");

    let mutation = "
        mutation {
          addNote(input: {bookId: 1, note: \"new note!\", page: -5}) {
            note {
              id
              bookId
              note
              page
            }
            success
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
async fn test_invalid_book_id_status_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    set_snapshot_suffix!("invalid_book_id_status_mutation");

    let mutation = "
        mutation {
          updateBookStatus(input: {bookId: 999, status: READ}) {
            book {
              id
              title
              status
            }
            success
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
async fn test_book_status_mutation(pool: Pool<Postgres>) -> sqlx::Result<()> {
    set_snapshot_suffix!("book_status_mutation");

    let mutation = "
        mutation {
          updateBookStatus(input: {bookId: 2, status: READ}) {
            book {
              id
              title
              status
            }
            success
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
