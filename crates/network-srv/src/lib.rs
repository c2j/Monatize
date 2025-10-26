use bytes::Bytes;
use message_defs::{HttpRequest, HttpResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct Network {
    client: reqwest::Client,
}

impl Network {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .expect("build client");
        Self { client }
    }

    pub async fn fetch(&self, req: HttpRequest) -> Result<HttpResponse, NetError> {
        // simple retry policy: up to 3 attempts with 50ms, 100ms backoff
        let mut last_err: Option<reqwest::Error> = None;
        for (i, backoff_ms) in [0u64, 50, 100].into_iter().enumerate() {
            if i > 0 { tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await; }
            match self.try_fetch_once(&req).await {
                Ok(resp) => return Ok(resp),
                Err(e) => { last_err = Some(e); }
            }
        }
        Err(NetError::Reqwest(last_err.unwrap()))
    }

    async fn try_fetch_once(&self, req: &HttpRequest) -> Result<HttpResponse, reqwest::Error> {
        let r = self.client.get(&req.url).send().await?;
        let r = r.error_for_status()?; // treat 4xx/5xx as errors (retryable for 5xx by policy)
        let status = r.status().as_u16();
        let headers: Vec<(String, String)> = r
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = r.bytes().await?;
        Ok(HttpResponse { status, headers, body: Bytes::from(body) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn basic_get_headers_and_status() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/hello"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes("hi").insert_header("Content-Type", "text/plain"))
            .mount(&server)
            .await;

        let net = Network::new();
        let resp = net.fetch(HttpRequest { url: format!("{}/hello", server.uri()) }).await.unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.headers.iter().any(|(k, v)| k == "content-type" && v.contains("text/plain")));
        assert_eq!(resp.body, Bytes::from_static(b"hi"));
    }

    #[tokio::test]
    async fn retry_on_connect_refused_then_ok() {
        // pick an available port by binding then dropping
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        // spawn server after a short delay so that the first attempt fails
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
            let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await.unwrap();
            if let Ok((mut stream, _)) = listener.accept().await {
                let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\n\r\nok";
                use tokio::io::AsyncWriteExt;
                let _ = stream.write_all(resp).await;
            }
        });

        let net = Network::new();
        let url = format!("http://127.0.0.1:{}/hello", port);
        let resp = net.fetch(HttpRequest { url }).await.unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, Bytes::from_static(b"ok"));
    }
}

