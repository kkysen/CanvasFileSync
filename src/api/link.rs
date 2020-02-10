use crate::api::link::LinkType::{CURRENT, FIRST};

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
            _ => return None
        })
    }
    
    pub fn is_not_current_first(&self) -> bool {
        *self != CURRENT && *self != FIRST
    }
}

#[derive(Debug)]
pub(super) struct Link<'a> {
    pub url: &'a str,
    pub type_: LinkType,
}

fn surrounded_by(s: &str, p1: char, p2: char) -> Option<&str> {
    let start = s.find(p1)? + 1;
    let end = s.rfind(p2)?;
    if end <= start {
        return None;
    }
    Some(&s[start..end])
}

impl<'a> Link<'a> {
    
    fn of(link: &'a str) -> Option<Link<'a>> {
        let middle = link.find("; rel=")?;
        let (front, back) = link.split_at(middle);
        let url = surrounded_by(front, '<', '>')?;
        let link_type = surrounded_by(back, '"', '"')?;
        Some(Link {
            url,
            type_: LinkType::of(link_type)?,
        })
    }
}

#[derive(Debug)]
pub(super) struct Links<'a> {
    raw: &'a str,
    pub links: Vec<Link<'a>>,
}

impl<'a> Links<'a> {
    pub(super) fn of(raw: &'a str) -> Option<Links<'a>> {
        let links = raw.split(',')
            .map(Link::of)
            .collect::<Option<Vec<_>>>()?;
        Some(Links {
            raw,
            links,
        })
    }
}
