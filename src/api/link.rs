#[derive(Debug, PartialEq, Eq)]
pub(super) enum LinkType {
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
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub(super) struct Link<'a> {
    pub url: &'a str,
    pub query: &'a str,
    pub type_: LinkType,
    pub current: bool,
}

fn surrounded_by(s: &str, p1: char, p2: char) -> Option<&str> {
    let start = s.find(p1)? + 1;
    let end = s.rfind(p2)?;
    if end <= start {
        return None;
    }
    Some(&s[start..end])
}

fn query_from_url(url: &str) -> &str {
    let i = url.rfind('?').unwrap_or(url.len() - 1) + 1;
    &url[i..]
}

impl<'a> Link<'a> {
    fn of(link: &'a str) -> Option<Link<'a>> {
        let middle = link.find("; rel=")?;
        let (front, back) = link.split_at(middle);
        let url = surrounded_by(front, '<', '>')?;
        let link_type = surrounded_by(back, '"', '"')?;
        Some(Link {
            url,
            query: query_from_url(url),
            type_: LinkType::of(link_type)?,
            current: false,
        })
    }
}

#[derive(Debug)]
pub(super) struct Links<'a> {
    raw: &'a str,
    links: Vec<Link<'a>>,
}

impl<'a> Links<'a> {
    pub(super) fn of(raw: &'a str) -> Option<Links<'a>> {
        let mut links: Vec<Link<'a>> = raw.split(',').map(Link::of).collect::<Option<Vec<_>>>()?;
        let current = links
            .iter()
            .find(|it| it.type_ == LinkType::CURRENT)
            .map(|it| it.query);
        if let Some(current) = current {
            for mut link in &mut links {
                link.current = current == link.query;
            }
        }
        Some(Links { raw, links })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Link<'a>> {
        self.links.iter()
    }
}

impl Default for Links<'_> {
    fn default() -> Self {
        Links {
            raw: "",
            links: Vec::default(),
        }
    }
}
