use std::env;
use reqwest;
use serde_json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct VideoContentDetail {
    duration: String
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct VideoStatisticsDetail {
    viewCount: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct VideoDetailItem {
    contentDetails: VideoContentDetail,
    statistics: VideoStatisticsDetail
}

#[derive(Debug, Deserialize)]
struct VideoDetailResponse {
    items: Vec<VideoDetailItem>
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct VideoSnippet {
    publishedAt: String,
    title: String,
    description: String,
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

// #[derive(Debug, Deserialize)]
// struct SearchResult {
//     id: VideoId,
//     snippet: VideoSnippet,
// }
//
// #[derive(Debug, Deserialize)]
// struct SearchListResponse {
//     items: Vec<SearchResult>,
// }

async fn get_search_resutls(query: &str) -> Result<(SearchResponse, VideoDetailResponse), reqwest::Error> {
    dotenv::dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();

    let search_url = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&q={}&part=snippet&type=video&maxResults=40",
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
        "https://www.googleapis.com/youtube/v3/videos?key={}&id={}&part=contentDetails&part=statistics",
        api_key, video_ids
    );

    let body = client.get(details_url)
        .send().await?
        .text().await?;

    let details_data: VideoDetailResponse = serde_json::from_str(&body).unwrap();

    Ok((search_data, details_data))
}

async fn get_channel_results(query: &str) -> Result<(SearchResponse, VideoDetailResponse), reqwest::Error> {
    dotenv::dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();

    let channel_id = query;

    let channel_videos_url = format!(
        "https://www.googleapis.com/youtube/v3/search?key={}&channelId={}&part=id%2Csnippet&type=video&order=date&maxResults=40",
        api_key, channel_id
    );

    let client = reqwest::Client::new();

    let body = client.get(channel_videos_url)
        .send().await?
        .text().await?;

    let videos_list_data: SearchResponse = serde_json::from_str(&body).unwrap();

    let video_ids: Vec<&str> = videos_list_data
        .items
        .iter()
        .map(|item| item.id.videoId.as_str())
        .collect();

    let video_ids = video_ids.join(",");

    let details_url = format!(
        "https://www.googleapis.com/youtube/v3/videos?key={}&id={}&part=contentDetails&part=statistics",
        api_key, video_ids
    );

    let body = client.get(details_url)
        .send().await?
        .text().await?;

    let details_data: VideoDetailResponse = serde_json::from_str(&body).unwrap();

    Ok((videos_list_data, details_data))
}

pub async fn get_search_list(query: &str) -> Vec<Vec<String>> {
    let query = query
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("+");

    let (results, details) = get_search_resutls(&query).await.unwrap();

    let mut data: Vec<Vec<String>> = Vec::new();

    for (search, detail) in results.items.iter().zip(details.items.iter()) {
        data.push(
            vec![
                search.snippet.title.clone(),
                search.snippet.channelTitle.clone(),
                search.snippet.publishedAt.clone(),
                detail.contentDetails.duration.clone(),
                detail.statistics.viewCount.clone(),
                format!("https://youtube.com/watch?v={}", search.id.videoId).clone(),
                search.snippet.description.clone(),
            ]
        );
    }

    return data
}

pub async fn get_channel_list(query: &str) -> Vec<Vec<String>> {
    let query = query
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("+");

    let (results, details) = get_channel_results(&query).await.unwrap();

    let mut data: Vec<Vec<String>> = Vec::new();

    for (search, detail) in results.items.iter().zip(details.items.iter()) {
        data.push(
            vec![
                search.snippet.title.clone(),
                search.snippet.channelTitle.clone(),
                search.snippet.publishedAt.clone(),
                detail.contentDetails.duration.clone(),
                detail.statistics.viewCount.clone(),
                format!("https://youtube.com/watch?v={}", search.id.videoId).clone(),
                search.snippet.description.clone(),
            ]
        );
    }

    return data
}
