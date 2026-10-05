use hypercs_db::PgPool;
use hypercs_domain::{AuthUser, DomainError, DomainResult};

use crate::dto::{HomepageContent, TermsContent};

const HOMEPAGE_KEY: &str = "homepage";
const TERMS_KEY: &str = "terms";

pub fn default_homepage() -> HomepageContent {
    HomepageContent {
        hero_heading: "Welcome to HyperCS".to_string(),
        hero_intro: "Open-source community software for your next project.".to_string(),
        hero_button_label: "Join the community".to_string(),
        features: vec![
            crate::dto::HomepageFeature {
                title: "Fast".to_string(),
                text: "Built on a modern, lightweight stack.".to_string(),
            },
            crate::dto::HomepageFeature {
                title: "Open".to_string(),
                text: "Fully open source, self-hostable.".to_string(),
            },
            crate::dto::HomepageFeature {
                title: "Friendly".to_string(),
                text: "A welcoming home for your community.".to_string(),
            },
        ],
    }
}

pub fn default_terms() -> TermsContent {
    TermsContent {
        body: "This is placeholder terms-of-service text for this HyperCS installation. \
               Replace it with your own terms before going live."
            .to_string(),
    }
}

pub async fn get_homepage(db: &PgPool) -> DomainResult<HomepageContent> {
    match hypercs_db::cms::get(db, HOMEPAGE_KEY)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        Some(value) => serde_json::from_value(value).map_err(|_| DomainError::Internal),
        None => Ok(default_homepage()),
    }
}

pub async fn put_homepage(
    db: &PgPool,
    user: &AuthUser,
    content: &HomepageContent,
) -> DomainResult<()> {
    if !hypercs_domain::policy::can_manage_cms(user) {
        return Err(DomainError::Forbidden);
    }
    let value = serde_json::to_value(content).map_err(|_| DomainError::Internal)?;
    hypercs_db::cms::put(db, HOMEPAGE_KEY, &value)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn get_terms(db: &PgPool) -> DomainResult<TermsContent> {
    match hypercs_db::cms::get(db, TERMS_KEY)
        .await
        .map_err(|_| DomainError::Internal)?
    {
        Some(value) => serde_json::from_value(value).map_err(|_| DomainError::Internal),
        None => Ok(default_terms()),
    }
}

pub async fn put_terms(db: &PgPool, user: &AuthUser, content: &TermsContent) -> DomainResult<()> {
    if !hypercs_domain::policy::can_manage_cms(user) {
        return Err(DomainError::Forbidden);
    }
    let value = serde_json::to_value(content).map_err(|_| DomainError::Internal)?;
    hypercs_db::cms::put(db, TERMS_KEY, &value)
        .await
        .map_err(|_| DomainError::Internal)
}
