use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct ImmichApi {
    pub token: String,
    pub url: String,
    pub client: reqwest::Client,
    pub album_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImmichAlbum {
    pub assets: Vec<ImmichAsset>,
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ImmichAsset {
    pub id: String,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    Image,
    Video,
    Audio,
    Other,
}

impl ImmichApi {
    pub fn new(token: String, url: String, album_id: String) -> Self {
        ImmichApi {
            token,
            url,
            client: reqwest::Client::new(),
            album_id,
        }
    }

    pub async fn get_album_with_assets(&self, album_id: &Uuid) -> ImmichAlbum {
        self.client
            .get(format!("{}/albums/{}", self.url, album_id))
            .header("x-api-key", &self.token)
            .send()
            .await
            .unwrap()
            .json::<ImmichAlbum>()
            .await
            .unwrap()
    }

    pub async fn download_asset(&self, asset_id: &str) -> Vec<u8> {
        self.client
            .get(format!("{}/assets/{}/original", self.url, asset_id))
            .header("x-api-key", &self.token)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .to_vec()
    }
}
