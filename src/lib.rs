#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

use serde::de::DeserializeOwned;
use surf::Request;
use surf::middleware::HttpClient;
use serde::Serialize;
use futures::future::join_all;

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
        let req = self.request(endpoint, query)?;
        let mut resp = req.await?;
        let first_page: Vec<T> = resp.body_json().await?;
        let link = resp.header("Link").and_then(Links::of);
        Ok(match link {
            None => first_page,
            Some(links) => {
                let futures = links
                    .links
                    .iter()
                    .filter(|it| it.type_ != LinkType::CURRENT)
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

#[derive(PartialEq, Eq)]
enum LinkType {
    CURRENT,
    NEXT,
    FIRST,
    LAST,
}

impl LinkType {
    fn of(link_type: &str) -> Option<LinkType> {
        Some(match link_type {
            "current" => LinkType::CURRENT,
            "next" => LinkType::NEXT,
            "first" => LinkType::FIRST,
            "last" => LinkType::LAST,
            _ => return None
        })
    }
}

struct Link<'a> {
    url: &'a str,
    type_: LinkType,
}

impl<'a> Link<'a> {
    fn surrounded_by(s: &'a str, p1: char, p2: char) -> Option<&'a str> {
        let start = s.find(p1)? + 1;
        let end = s.rfind(p2)? - 1;
        if end <= start {
            return None;
        }
        Some(&s[start..end])
    }
    
    fn of(link: &'a str) -> Option<Link<'a>> {
        let middle = link.find("; rel=")?;
        let (front, back) = link.split_at(middle);
        let url = Link::surrounded_by(front, '<', '>')?;
        let link_type = Link::surrounded_by(back, '"', '"')?;
        Some(Link {
            url,
            type_: LinkType::of(link_type)?,
        })
    }
}

struct Links<'a> {
    raw: &'a str,
    links: Vec<Link<'a>>,
}

impl<'a> Links<'a> {
    fn of(raw: &'a str) -> Option<Links<'a>> {
        let links = raw.split(',')
            .map(Link::of)
            .collect::<Option<Vec<_>>>()?;
        Some(Links {
            raw,
            links,
        })
    }
}
