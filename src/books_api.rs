use anyhow::Result;
use dotenvy_macro::dotenv;
use serde::{Deserialize, Serialize};

use crate::graphql::SearchResult;

const API_BASE_URL: &str = "https://www.googleapis.com/books/v1/volumes";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    items: Vec<ApiItem>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiItem {
    id: String,
    volume_info: Volume,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub title: String,
    pub authors: Vec<String>,
    #[serde(default = "page_count_default")]
    pub page_count: i32,
    pub published_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_links: Option<ImageLinks>,
}

fn page_count_default() -> i32 {
    0
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageLinks {
    pub thumbnail: String,
}

pub struct BooksApi {}

impl BooksApi {
    /// Searches the Google Books API with a supplied query string. May return an error if the API
    /// request fails.
    pub async fn search(query: String) -> Result<Vec<SearchResult>> {
        let api_key = dotenv!("GOOGLE_API_KEY");

        let request_url = format!(
            "{}?q={}&token={}&max_results=10&printType=books",
            API_BASE_URL, query, api_key
        );

        let body = reqwest::get(request_url)
            .await?
            .json::<ApiResponse>()
            .await?;

        let mut response = Vec::new();

        // Serialize each search result into our `SearchResult` struct.
        for item in body.items {
            let info = item.volume_info;

            let mut image_url = String::new();
            if let Some(links) = info.image_links {
                image_url = links.thumbnail;
            }

            // Safely convert the first 4 characters of the `published_date` field from a
            // YYYY-MM-DD String to a YYYY i32.
            let year = Self::parse_year(info.published_date);

            response.push(SearchResult {
                id: item.id,
                title: info.title,
                authors: info.authors,
                pages: info.page_count,
                year,
                image_url: Some(image_url),
            })
        }
        Ok(response)
    }

    /// Fetches a book by its Google Books API volume ID, which is generally found via
    /// `BooksApi::search`. May return an error if the API request fails.
    pub async fn get_by_id(volume_id: String) -> Result<Volume> {
        let api_key = dotenv!("GOOGLE_API_KEY");

        let request_url = format!("{}/{}?token={}", API_BASE_URL, volume_id, api_key);
        let body = reqwest::get(request_url)
            .await?
            .json::<Option<ApiItem>>()
            .await?;

        Ok(body.unwrap().volume_info)
    }

    /// Safely convert the first 4 characters of the `published_date` field from a
    /// YYYY-MM-DD String to a YYYY i32.
    pub fn parse_year(year: String) -> i32 {
        year.chars()
            .take(4)
            .collect::<String>()
            .parse::<i32>()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_year() {
        assert_eq!(1973, BooksApi::parse_year("1973-01-01".to_string()));
        assert_eq!(
            1973,
            BooksApi::parse_year("1973-10-18T09:26:55−07:00".to_string())
        );
        assert_eq!(123, BooksApi::parse_year("123".to_string()));
        assert_eq!(0, BooksApi::parse_year("bad".to_string()));
        assert_eq!(0, BooksApi::parse_year("".to_string()));
    }
}
