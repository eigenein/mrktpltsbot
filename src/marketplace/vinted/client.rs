use bon::Builder;
use prost::Message;
use reqwest::{StatusCode, header};
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

use crate::{
    db::KeyedMessage,
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
        info!("🔐 Refreshing token…");
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
        info!(
            "🔎 Searching on Vinted…",
            query = request.search_text.to_string(),
            limit = i64::from(request.per_page),
        );
        let url = {
            let query =
                serde_qs::to_string(request).context("failed to serialize the search request")?;
            let mut url = Url::parse("https://www.vinted.nl/api/v2/catalog/items").unwrap();
            url.set_query(Some(&query));
            url
        };
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
        let search_results = response
            .error_for_status()?
            .json()
            .await
            .context("failed to deserialize search results")?;
        Ok(search_results)
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
