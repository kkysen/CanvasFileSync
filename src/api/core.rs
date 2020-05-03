use crate::api::link::{Link, Links};
use graphql_client::GraphQLQuery;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use surf::middleware::HttpClient;
use surf::{Request, Response};
use crate::download::data::Id;
use std::error::Error;
use crate::util::future::FutureIterator;
use http_types::headers::HeaderName;

#[derive(Serialize, Deserialize)]
pub struct CoreApi {
    pub domain: String,
    pub authorization: String,
}

const AUTHORIZATION_PREFIX: &str = "Bearer ";

fn header_name(name: &str) -> HeaderName {
    name.parse().unwrap()
}

impl CoreApi {
    // if None, use environment variable
    pub fn get_access_token(access_token: Option<String>) -> String {
        access_token.unwrap_or_else(|| {
            std::env::var("CANVAS_ACCESS_TOKEN")
                .expect("CANVAS_ACCESS_TOKEN not set")
        })
    }
    
    pub fn new(root_url: String, access_token: String) -> CoreApi {
        let mut auth = access_token;
        auth.insert_str(0, AUTHORIZATION_PREFIX);
        CoreApi { domain: root_url, authorization: auth }
    }
    
    pub fn access_token(&self) -> &str {
        &self.authorization[AUTHORIZATION_PREFIX.len()..]
    }
    
    fn api_url(&self, version: &str, endpoint: &str) -> String {
        format!("https://{}/api/{}/{}", self.domain, version, endpoint)
    }
    
    fn rest_url(&self, endpoint: &str) -> String {
        self.api_url("v1", endpoint)
    }
    
    fn download_url(&self, id: &Id) -> String {
        format!("https://{}/files/{}/download?download_frd=1", self.domain, id)
    }
    
    fn raw_request(&self, url: impl AsRef<str>) -> Request<impl HttpClient> {
        surf::get(url)
            .set_header(header_name("Authorization"), &self.authorization)
    }
    
    pub async fn download(&self, id: &Id) -> Result<Response, Box<dyn Error>> {
        let resp = self.raw_request(self.download_url(id)).await?;
        Ok(resp)
    }
    
    fn request(&self, endpoint: &str, query: &impl Serialize)
               -> Result<Request<impl HttpClient>, Box<dyn Error>> {
        let url = self.rest_url(endpoint);
        let request = self.raw_request(url);
        let request = request.set_query(query)?;
        Ok(request)
    }
    
    pub async fn get<Q, T>(&self, endpoint: &str, query: &Q) -> Result<T, Box<dyn Error>>
        where
            Q: Serialize,
            T: DeserializeOwned, {
        let req = self.request(endpoint, query)?;
        let o: T = req.recv_json().await?;
        Ok(o)
    }
    
    pub async fn get_list<Q, T>(&self, endpoint: &str, query: &Q) -> Result<Vec<T>, Box<dyn Error>>
        where
            Q: Serialize,
            T: DeserializeOwned, {
        let req = self.request(endpoint, query).unwrap();
        let mut resp = req.await?;
        let mut pages: Vec<T> = resp.body_json().await?;
        let links = resp.header(&header_name("Link"))
            .and_then(|it| it.first()) // TODO see how surf parses multi value headers
            .map(|it| it.as_str())
            .and_then(Links::of)
            .unwrap_or_default();
        let rest_page_results = links
            .iter()
            .filter(|it| !it.current)
            .map(|link| self.get_link(link))
            .join_all()
            .await;
        for page_result in rest_page_results {
            let mut page = page_result?;
            pages.append(&mut page);
        }
        Ok(pages)
    }
    
    async fn get_link<T: DeserializeOwned>(&self, link: &Link<'_>) -> Result<Vec<T>, Box<dyn Error>> {
        let list = self
            .raw_request(link.url)
            .recv_json()
            .await?;
        Ok(list)
    }
    
    pub async fn get_filtered_list<Q, T, U>(&self, endpoint: &str, query: &Q)
        -> Result<impl Iterator<Item = U>, Box<dyn Error>>
        where
            Q: Serialize,
            T: DeserializeOwned + Into<Option<U>> {
        let list = self
            .get_list::<Q, T>(endpoint, query)
            .await?
            .into_iter()
            .filter_map(|it| it.into());
        Ok(list)
    }
    
    pub async fn query<T: GraphQLQuery>(&self, vars: T::Variables)
        -> Result<graphql_client::Response<T::ResponseData>, Box<dyn Error>> {
        let query = T::build_query(vars);
        let resp = surf::post(self.api_url("graphql", ""))
            .set_header(header_name("Authorization"), &self.authorization)
            .body_json(&query)?
            .recv_json()
            .await?;
        Ok(resp)
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
