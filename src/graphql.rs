use std::{fmt::Display, str::FromStr};

use anyhow::{anyhow, Result};
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptySubscription, Enum, InputObject, Object,
    Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::Extension, response::Html};

use crate::books_api::BooksApi;
use crate::repository::Storage;

#[derive(Clone, Copy, Debug, Enum, Eq, PartialEq, sqlx::Type)]
#[sqlx(rename_all = "lowercase", type_name = "status")]
pub enum ReadingStatus {
    Unread,
    Reading,
    Read,
}

impl Default for ReadingStatus {
    fn default() -> Self {
        Self::Unread
    }
}

impl Display for ReadingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unread => write!(f, "unread"),
            Self::Read => write!(f, "read"),
            Self::Reading => write!(f, "reading"),
        }
    }
}

impl FromStr for ReadingStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "unread" => Ok(Self::Unread),
            "reading" => Ok(Self::Reading),
            "read" => Ok(Self::Read),
            _ => Err(anyhow!("Invalid reading status")),
        }
    }
}

// This allows the use of Enum types to be used directly in postgres sqlx queries.
impl sqlx::postgres::PgHasArrayType for ReadingStatus {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_status")
    }
}

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Book {
    /// The book ID
    pub id: i32,
    /// The title of the book
    pub title: String,
    /// The author of the book
    pub author: String,
    /// A link to an image of the book cover
    pub image_url: Option<String>,
    /// The year the book was published
    pub year: i32,
    /// The number of pages in the book
    pub pages: i32,
    /// The reading status of the book
    pub status: ReadingStatus,
}

#[derive(Clone, InputObject)]
pub struct AddGoogleBookInput {
    /// The Google Books API book ID.
    pub google_books_id: String,
    /// An optional title of the book. This field will override the title taken from the Google
    /// Books API.
    pub title: Option<String>,
    /// An optional author name. This field will override the author name taken from the Google
    /// Books API.
    pub author: Option<String>,
    /// An optional link to an image of the book cover. This field will override the image URL
    /// taken from the Google Books API.
    pub image_url: Option<String>,
    /// An optional year that the book was published. This field will override the year taken from
    /// the Google Books API.
    pub year: Option<i32>,
    /// An optional number of pages in the book. This field will override the page count taken from
    /// the Google Books API.
    pub pages: Option<i32>,
    /// The reading status of the book. Defaults to UNREAD.
    pub status: Option<ReadingStatus>,
}

#[derive(Clone, InputObject)]
pub struct AddBookInput {
    /// The title of the book
    pub title: String,
    /// The author of the book
    pub author: String,
    /// A link to an image of the book cover
    pub image_url: Option<String>,
    /// The year the book was published
    pub year: i32,
    /// The number of pages in the book
    pub pages: i32,
    /// The reading status of the book. Defaults to UNREAD.
    pub status: Option<ReadingStatus>,
}

#[derive(SimpleObject)]
pub struct AddBookPayload {
    /// The book that was added
    pub book: Book,
    /// Did the operation succeed?
    pub success: bool,
}

#[derive(SimpleObject)]
pub struct Note {
    /// The note ID
    pub id: i32,
    /// The ID of the book this note references
    pub book_id: i32,
    /// The note content for a book
    pub note: String,
    /// An optional page number related to the number
    pub page: Option<i32>,
}

#[derive(InputObject)]
pub struct AddNoteInput {
    /// The ID of the book this note references
    pub book_id: i32,
    /// The note content for a book
    pub note: String,
    /// An optional page number related to the number
    pub page: Option<i32>,
}

#[derive(SimpleObject)]
pub struct AddNotePayload {
    /// The note that was added
    pub note: Note,
    /// Did the operation succeed?
    pub success: bool,
}

#[derive(InputObject)]
pub struct UpdateBookStatusInput {
    /// The ID of the book this note references
    pub book_id: i32,
    /// The reading status of the book
    pub status: ReadingStatus,
}

#[derive(SimpleObject)]
pub struct UpdateBookStatusPayload {
    /// The book that was updated
    pub book: Book,
    /// Did the operation succeed?
    pub success: bool,
}

#[derive(SimpleObject)]
pub struct SearchResult {
    /// The ID of the search result item
    pub id: String,
    /// The title of the book
    pub title: String,
    /// The authors of the book
    pub authors: Vec<String>,
    /// The page count
    pub pages: i32,
    /// The year the book was published
    pub year: i32,
    /// An image URL of the book cover
    pub image_url: Option<String>,
}

pub struct Query;
pub struct Mutation;

#[derive(Debug)]
pub enum GraphQLError {
    BadInput(String),
}

impl Display for GraphQLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadInput(bad_input_error) => write!(f, "{bad_input_error}"),
        }
    }
}

impl std::error::Error for GraphQLError {}

#[Object]
impl Query {
    /// Fetch a book by either its ID or by its title
    async fn book(
        &self,
        ctx: &Context<'_>,
        book_id: Option<i32>,
        title: Option<String>,
    ) -> Result<Option<Book>> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        if let Some(input) = book_id {
            return repository.lock().await.get_book_by_id(input).await;
        } else if let Some(input) = title {
            return repository.lock().await.get_book_by_title(input).await;
        }
        Err(GraphQLError::BadInput(
            "Either `bookId` or `title` input value required for query.".to_string(),
        )
        .into())
    }

    /// Fetch all books with an optional status specifier
    async fn books(
        &self,
        ctx: &Context<'_>,
        status: Option<ReadingStatus>,
    ) -> Result<Option<Vec<Book>>> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        let books = repository.lock().await.get_books(status).await?;
        Ok(books)
    }

    /// Search for a book to add to the catalog via Google Books API
    async fn search(&self, query: String) -> Result<Vec<SearchResult>> {
        let results = BooksApi::search(query).await?;
        Ok(results)
    }
}

#[Object]
impl Mutation {
    /// Add a book by manually entering in the book details.
    async fn add_book(&self, ctx: &Context<'_>, input: AddBookInput) -> Result<AddBookPayload> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        if input.pages < 1 {
            return Err(GraphQLError::BadInput(
                "Book page number cannot be less than 1.".to_string(),
            )
            .into());
        }

        let book = repository.lock().await.add_book(input).await?;

        Ok(AddBookPayload {
            book,
            success: true,
        })
    }

    /// Add a book via Google Books API search.
    async fn add_google_book(
        &self,
        ctx: &Context<'_>,
        input: AddGoogleBookInput,
    ) -> Result<AddBookPayload> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        if let Some(page_count) = input.pages {
            if page_count < 1 {
                return Err(GraphQLError::BadInput(
                    "Book page number cannot be less than 1.".to_string(),
                )
                .into());
            }
        }

        let book_result = BooksApi::get_by_id(input.google_books_id).await.unwrap();

        let author = book_result
            .authors
            .first()
            .expect("Author not found in search result.")
            .to_owned();

        let mut image_url = String::new();
        if let Some(links) = book_result.image_links {
            image_url = links.thumbnail;
        }

        let year = BooksApi::parse_year(book_result.published_date);

        // Use optional overrides if any were provided; otherwise, default to results from Google
        // Books API.
        let book_input = AddBookInput {
            title: input.title.unwrap_or(book_result.title),
            author: input.author.unwrap_or(author),
            image_url: Some(image_url),
            year: input.year.unwrap_or(year),
            pages: input.pages.unwrap_or(book_result.page_count),
            status: input.status,
        };

        let book = repository.lock().await.add_book(book_input).await?;

        Ok(AddBookPayload {
            book,
            success: true,
        })
    }

    /// Update a book's reading status
    async fn update_book_status(
        &self,
        ctx: &Context<'_>,
        input: UpdateBookStatusInput,
    ) -> Result<UpdateBookStatusPayload> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        let book = repository
            .lock()
            .await
            .update_book_status(input.book_id, input.status)
            .await?;
        Ok(UpdateBookStatusPayload {
            book,
            success: true,
        })
    }

    /// Add a new note for a given book
    async fn add_note(&self, ctx: &Context<'_>, input: AddNoteInput) -> Result<AddNotePayload> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        // Fetch requested book for validation
        let book = repository
            .lock()
            .await
            .get_book_by_id(input.book_id)
            .await?;

        if book.is_none() {
            return Err(GraphQLError::BadInput("Book with that ID not found".to_string()).into());
        };
        let book = book.unwrap();

        if let Some(note_page) = input.page {
            if note_page > book.pages {
                return Err(GraphQLError::BadInput(
                    "Note page number cannot be greater than the highest page count of the book."
                        .to_string(),
                )
                .into());
            } else if note_page < 1 {
                return Err(GraphQLError::BadInput(
                    "Note page number cannot be less than 1.".to_string(),
                )
                .into());
            }
        }

        let note_input = AddNoteInput {
            book_id: input.book_id,
            note: input.note,
            page: input.page,
        };
        let note = repository.lock().await.add_note(note_input).await?;
        Ok(AddNotePayload {
            note,
            success: true,
        })
    }
}

#[ComplexObject]
impl Book {
    /// All notes related to the given book
    async fn notes(&self, ctx: &Context<'_>) -> Result<Option<Vec<Note>>> {
        let repository = ctx.data_unchecked::<Storage>().clone();
        let notes = repository.lock().await.get_notes_by_book(self.id).await?;
        Ok(notes)
    }
}

pub async fn graphql_handler(
    schema: Extension<Schema<Query, Mutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().finish())
}
