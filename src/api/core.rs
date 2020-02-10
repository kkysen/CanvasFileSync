use serde::de::DeserializeOwned;
use surf::Request;
use surf::middleware::HttpClient;
use serde::Serialize;
use futures::future::join_all;
use crate::api::link::{Link, Links};

pub type SurfResult<T> = Result<T, surf::Exception>;

pub struct CoreApi {
    root_url: String,
    auth: String,
}

const AUTH_PREFIX: &'static str = "Bearer ";

impl CoreApi {
    pub fn new(root_url: String, access_token: String) -> CoreApi {
        let mut auth = access_token;
        auth.insert_str(0, AUTH_PREFIX);
        CoreApi {
            root_url,
            auth,
        }
    }
    
    fn access_token(&self) -> &str {
        &self.auth[AUTH_PREFIX.len()..]
    }
    
    fn url(&self, endpoint: &str) -> String {
        format!("https://{}/api/v1/{}", self.root_url, endpoint)
    }
    
    fn raw_request(&self, url: impl AsRef<str>) -> Request<impl HttpClient> {
        surf::get(url)
            .set_header("Authorization", &self.auth)
    }
    
    fn request(&self, endpoint: &str, query: &impl Serialize) -> SurfResult<Request<impl HttpClient>> {
        Ok(self.raw_request(self.url(endpoint))
            .set_query(query)?
        )
    }
    
    pub async fn get<Q, T>(&self, endpoint: &str, query: &Q) -> SurfResult<T> where
        Q: Serialize,
        T: DeserializeOwned {
        let req = self.request(endpoint, query)?;
        let o: T = req.recv_json().await?;
        Ok(o)
    }
    
    pub async fn get_list<Q, T>(&self, endpoint: &str, query: &Q) -> SurfResult<Vec<T>> where
        Q: Serialize,
        T: DeserializeOwned {
        let req = self.request(endpoint, query).unwrap();
        let mut resp = req.await?;
        let mut pages: Vec<T> = resp.body_json().await?;
        let links = resp
            .header("link")
            .and_then(Links::of)
            .unwrap_or_default();
        let futures = links
            .iter()
            .filter(|it| !it.current)
            .map(|link| self.get_link(link));
        let rest_page_results = join_all(futures).await;
        for page_result in rest_page_results {
            let mut page = page_result?;
            pages.append(&mut page);
        }
        Ok(pages)
    }
    
    async fn get_link<T: DeserializeOwned>(&self, link: &Link<'_>) -> SurfResult<Vec<T>> {
        self.raw_request(link.url)
            .recv_json()
            .await
    }
    
    pub async fn get_filtered_list<Q, T, U>(&self, endpoint: &str, query: &Q) -> SurfResult<impl Iterator<Item = U>> where
        Q: Serialize,
        T: DeserializeOwned + Into<Option<U>> {
        let list = self.get_list::<Q, T>(endpoint, query)
            .await?
            .into_iter()
            .filter_map(|it| it.into());
        Ok(list)
    }
}

#[derive(Serialize)]
pub struct Empty {}

const EMPTY: Empty = Empty {};

pub fn no_query() -> &'static Empty {
    &EMPTY
}

#[derive(Serialize)]
pub struct PerPage {
    pub per_page: u32,
}

pub type Id = u64;
