use bon::Builder;
use prost::Message;
use reqwest::{StatusCode, header};
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

use crate::{
    db::KeyedMessage,
    logging::Breadcrumb,
    marketplace::vinted::{
        VintedError,
        search::{SearchRequest, SearchResults},
    },
    prelude::*,
};

#[derive(Clone)]
pub struct VintedClient(pub ClientWithMiddleware);

impl VintedClient {
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthenticationTokens> {
        info!("🔐 Refreshing Vinted access token…");
        let response = self
            .0
            .post("https://www.vinted.com/web/api/auth/refresh")
            .header(header::COOKIE, format!("refresh_token_web={refresh_token}"))
            .send()
            .await?
            .error_for_status()?;
        let mut access_token = None;
        let mut refresh_token = None;
        for cookie in response.cookies() {
            if cookie.name().eq_ignore_ascii_case("access_token_web") {
                access_token = Some(cookie.value().to_string());
            } else if cookie.name().eq_ignore_ascii_case("refresh_token_web") {
                refresh_token = Some(cookie.value().to_string());
            }
        }
        Ok(AuthenticationTokens::builder()
            .access(access_token.context("missing access token cookie")?)
            .refresh(refresh_token.context("missing refresh token cookie")?)
            .build())
    }

    pub async fn search(
        &self,
        access_token: &str,
        request: &SearchRequest<'_>,
    ) -> Result<SearchResults, VintedError> {
        let url = {
            let query =
                serde_qs::to_string(request).context("failed to serialize the search request")?;
            let mut url = Url::parse("https://www.vinted.nl/api/v2/catalog/items").unwrap();
            url.set_query(Some(&query));
            url
        };
        Breadcrumb::debug()
            .category(module_path!())
            .message("Searching on Vinted…")
            .data("url", url.as_str())
            .data("access_token", access_token)
            .build()
            .add();
        let response = self
            .0
            .get(url)
            .header(header::COOKIE, format!("access_token_web={access_token}"))
            .send()
            .await?;
        if response.status() == StatusCode::UNAUTHORIZED
            || response.status() == StatusCode::FORBIDDEN
        {
            // FIXME: not sure about 403.
            return Err(VintedError::Reauthenticate);
        }
        let response = response.error_for_status()?.text().await?;
        Breadcrumb::debug()
            .category(module_path!())
            .message("Parsing response…")
            .data("response.body", response.as_str())
            .build()
            .add();
        Ok(serde_json::from_str(&response).context("failed to deserialize search results")?)
    }
}

#[must_use]
#[derive(PartialEq, Eq, Builder, Message)]
pub struct AuthenticationTokens {
    #[builder(into)]
    #[prost(tag = "1", string)]
    pub access: String,

    #[builder(into)]
    #[prost(tag = "2", string)]
    pub refresh: String,
}

impl KeyedMessage for AuthenticationTokens {
    const KEY: &'static str = "mrktpltsbot::marketplace::vinted::client::AuthenticationTokens";
}
