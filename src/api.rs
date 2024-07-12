use std::env;
// use dotenv;
use tokio;
use reqwest;
use serde_json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct VideoContentDetail {
    duration: String
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct VideoDetailItem {
    contentDetails: VideoContentDetail
}

#[derive(Debug, Deserialize)]
struct VideoDetailResponse {
    items: Vec<VideoDetailItem>
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct VideoSnippet {
    title: String,
    channelTitle: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct VideoId {
    videoId: String,
}

#[derive(Debug, Deserialize)]
struct VideoItem {
    id: VideoId,
    snippet: VideoSnippet,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<VideoItem>
}

async fn get_search_resutls(query: &str) -> Result<(SearchResponse, VideoDetailResponse), reqwest::Error> {
    // dotenv::dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();

    let search_url = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&part=snippet&q={}&type=video&maxResults=30",
        api_key, query
    );

    let client = reqwest::Client::new();

    let body = client.get(search_url)
        .send().await?
        .text().await?;

    let search_data: SearchResponse = serde_json::from_str(&body).unwrap();

    let video_ids: Vec<&str> = search_data
        .items
        .iter()
        .map(|item| item.id.videoId.as_str())
        .collect();

    let video_ids = video_ids.join(",");

    let details_url = format!(
        "https://www.googleapis.com/youtube/v3/videos?key={}&id={}&part=contentDetails",
        api_key, video_ids
    );

    let body = client.get(details_url)
        .send().await?
        .text().await?;

    let details_data: VideoDetailResponse = serde_json::from_str(&body).unwrap();

    Ok((search_data, details_data))
}

pub fn search(query: &str) -> Vec<Vec<String>> {
    let query = query
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("+");

    let (results, details) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(get_search_resutls(&query))
        .unwrap();

    let mut data: Vec<Vec<String>> = Vec::new();

    for (search, detail) in results.items.iter().zip(details.items.iter()) {
        data.push(
            vec![
                search.snippet.title.clone(),
                search.snippet.channelTitle.clone(),
                detail.contentDetails.duration.clone(),
                format!("https://youtube.com/watch?v={}", search.id.videoId).clone()
            ]
        );
    }

    return data
    // println!("{:?}", data);
}

// fn main() {
//     let query = "elden ring"
//         .split_whitespace()
//         .collect::<Vec<&str>>()
//         .join("+");
//
//     let (results, details) = tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(get_search_resutls(&query))
//         .unwrap();
//
//     let mut data: Vec<Vec<String>> = Vec::new();
//
//     for (search, detail) in results.items.iter().zip(details.items.iter()) {
//         println!("-------------");
//         println!("title:    {}", search.snippet.title);
//         println!("channel:  {}", search.snippet.channelTitle);
//         println!("duration: {}", detail.contentDetails.duration);
//         println!("link:     https://youtube.com/watch?v={}", search.id.videoId);
//     }
// }
