mod link;

use serde::de::DeserializeOwned;
use surf::Request;
use surf::middleware::HttpClient;
use serde::Serialize;
use futures::future::join_all;
use crate::api::link::{Link, Links};

type SurfResult<T> = Result<T, surf::Exception>;

pub struct Api {
    root_url: String,
    auth: String,
}

const AUTH_PREFIX: &'static str = "Bearer ";

impl Api {
    pub fn new(root_url: String, access_token: String) -> Api {
        let mut auth = access_token;
        auth.insert_str(0, AUTH_PREFIX);
        Api {
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
    
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str, query: &impl Serialize) -> SurfResult<T> {
        let req = self.request(endpoint, query)?;
        let o: T = req.recv_json().await?;
        Ok(o)
    }
    
    pub async fn get_list<T: DeserializeOwned>(&self, endpoint: &str, query: &impl Serialize) -> SurfResult<Vec<T>> {
        let req = self.request(endpoint, query).unwrap();
        let mut resp = req.await?;
//        println!("{}", resp.body_string().await?);
        let first_page: Vec<T> = resp.body_json().await?;
        let link = resp.header("link").and_then(Links::of);
        Ok(match link {
            None => first_page,
            Some(links) => {
                let futures = links
                    .links
                    .iter()
                    .filter(|it| it.type_.is_not_current_first())
                    .map(|link| self.get_link(link));
                let rest_page_results = join_all(futures).await;
                let mut pages = first_page;
                for page_result in rest_page_results.into_iter() {
                    let mut page = page_result?;
                    pages.append(&mut page);
                }
                pages
            }
        })
    }
    
    async fn get_link<T: DeserializeOwned>(&self, link: &Link<'_>) -> SurfResult<Vec<T>> {
        self.raw_request(link.url)
            .recv_json()
            .await
    }
}

#[derive(Serialize)]
pub struct Empty {}

const EMPTY: Empty = Empty {};

pub fn no_query() -> &'static Empty {
    &EMPTY
}
