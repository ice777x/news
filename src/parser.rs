use crate::{news_reader_from_file, RssFeed};
use feed_rs::parser;

pub async fn get_all_rss() -> Vec<RssFeed> {
    let rss_links = news_reader_from_file("newsfeed.txt");
    let mut rss_s = vec![];
    for i in rss_links {
        let res = match get_rss(&i).await.ok() {
            Some(feed) => Some(feed),
            None => None,
        };
        if let Some(res) = res {
            rss_s.push(res);
        }
    }
    rss_s.concat().to_vec()
}

pub async fn get_rss(url: &str) -> Result<Vec<RssFeed>, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?.bytes().await?;
    let rss = parser::parse(&resp[..])?;
    let mut rss_vec = vec![];
    let source: &str = &rss.title.ok_or(String::new()).unwrap().content;
    for entry in rss.entries {
        let rssfeed = RssFeed {
            guid: entry.id,
            title: entry.title.ok_or(String::new()).unwrap().content,
            author: source.to_string(),
            link: match entry.links.first() {
                Some(link) => link.href.clone(),
                None => String::new(),
            },
            description: match entry.summary {
                Some(t) => Some(t.content),
                None => None,
            },
            content: match entry.content {
                Some(m) => Some(m.body.unwrap_or(String::new())),
                None => None,
            },
            published: match entry.published {
                Some(p) => Some(p.to_string()),
                None => None,
            },
            images: match entry.media.first() {
                Some(med) => {
                    let img = match med.content.first() {
                        Some(mc) => match &mc.url {
                            Some(url) => Some(url.to_string()),
                            None => None,
                        },
                        None => None,
                    };
                    img
                }
                None => None,
            },
        };
        rss_vec.push(rssfeed);
    }
    Ok(rss_vec)
}
