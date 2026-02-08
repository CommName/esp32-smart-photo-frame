use anyhow::{Context, Result, anyhow};
use futures::TryFutureExt;
use reqwest::ClientBuilder;
use serde::Deserialize;
use tokio::task::JoinSet;
use uuid::Uuid;

pub struct Immich {
    server_url: String,
    api_key: String,
}

#[derive(Deserialize)]
struct Album {
    id: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    id: String,
}

impl Immich {
    pub fn new(server_url: String, api_key: String) -> Self {
        Immich {
            server_url,
            api_key,
        }
    }

    fn create_client() -> reqwest::Client {
        ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap()
    }

    async fn get_album(base_url: &String, id: &Uuid, api_key: &String) -> Result<Album> {
        Ok(Self::create_client()
            .get(format!("{base_url}/albums/{id}?apiKey={api_key}"))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch {e:?}"))?
            .json()
            .map_err(|e| anyhow!("Failed to parse data {e:?}"))
            .await?)
    }

    pub async fn get_photos(&self, album_id: Uuid) -> Result<Vec<Vec<u8>>> {
        let album = Self::get_album(&self.server_url, &album_id, &self.api_key).await?;

        let mut join_set = JoinSet::new();

        for asset in album.assets {
            join_set.spawn(Self::get_photo(
                self.server_url.clone(),
                asset.id,
                self.api_key.clone(),
            ));
        }

        let mut ret = Vec::new();
        while let Some(res) = join_set.join_next().await {
            ret.push(res??);
        }
        Ok(ret)
    }

    pub async fn get_photo(server_url: String, id: String, api_key: String) -> Result<Vec<u8>> {
        Ok(Self::create_client()
            .get(format!(
                "{server_url}/assets/{id}/original?apiKey={api_key}"
            ))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch {e:?}"))?
            .bytes()
            .await?
            .to_vec())
    }
}
