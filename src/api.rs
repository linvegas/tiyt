use tokio;
use reqwest;
use serde_json;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Type {
    name: String,
}

#[derive(Debug, Deserialize)]
struct PokeType {
    r#type: Type,
}

#[derive(Debug, Deserialize)]
struct Pokemon {
    name: String,
    id: usize,
    types: Vec<PokeType>,
}

async fn get_pokemon() -> Result<Pokemon, reqwest::Error> {
    let body = reqwest::get("https://pokeapi.co/api/v2/pokemon/1")
        .await?
        .text()
        .await?;

    let data = serde_json::from_str::<Pokemon>(&body).unwrap();

    Ok(data)
}

fn main() {
    let pokemon = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(get_pokemon())
        .unwrap();

    println!("id: {}", pokemon.id);
    println!("name: {}", pokemon.name);
    print!("types: [ ");
    for t in pokemon.types {
        print!("{} ", t.r#type.name);
    }
    print!("]\n");
}
