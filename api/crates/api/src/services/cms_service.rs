use hypercs_db::PgPool;
use hypercs_domain::{
    AuthUser as Principal,
    constants::{
        FEATURE_TEXT_MAX, FEATURE_TITLE_MAX, HERO_BUTTON_MAX, HERO_HEADING_MAX, SITE_TITLE_MAX,
        SLOGAN_MAX, TERMS_BODY_MAX,
    },
    policy,
};

use crate::{
    dto::{HomepageContent, HomepageFeature, SiteSettings, TermsContent},
    error::{ApiError, ApiResult},
    services::text,
};

const HOMEPAGE_KEY: &str = "homepage";
const TERMS_KEY: &str = "terms";
const SITE_KEY: &str = "site";

const DEFAULT_SLOGAN: &str = "Open-source community software for your next project.";

pub const LANGUAGES: [&str; 2] = ["en-US", "de"];
pub const DEFAULT_LANGUAGE: &str = "en-US";

/// Shown until an admin edits the homepage. Seed content is English and,
/// like all user content, is not translated. It never mentions the forum.
pub fn default_homepage() -> HomepageContent {
    HomepageContent {
        hero_heading: "Welcome to HyperCS".to_string(),
        hero_button_label: "Join the community".to_string(),
        features: vec![
            HomepageFeature {
                title: "Fast".to_string(),
                text: "Built on a modern, lightweight stack.".to_string(),
            },
            HomepageFeature {
                title: "Open".to_string(),
                text: "Fully open source and self-hostable.".to_string(),
            },
            HomepageFeature {
                title: "Friendly".to_string(),
                text: "A welcoming home for your community.".to_string(),
            },
        ],
    }
}

pub fn default_site() -> SiteSettings {
    SiteSettings {
        site_title: "HyperCS".to_string(),
        slogan: DEFAULT_SLOGAN.to_string(),
        default_language: DEFAULT_LANGUAGE.to_string(),
    }
}

pub fn default_terms() -> TermsContent {
    TermsContent {
        body: "This is placeholder terms-of-service text for this HyperCS installation. \
               Replace it with your own terms before going live."
            .to_string(),
    }
}

fn require_admin(user: &Principal) -> ApiResult<()> {
    if policy::can_manage_cms(user) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

async fn load<T: serde::de::DeserializeOwned>(
    db: &PgPool,
    key: &str,
    default: impl FnOnce() -> T,
) -> ApiResult<T> {
    match hypercs_db::cms::get(db, key).await? {
        Some(value) => serde_json::from_value(value).map_err(|error| {
            tracing::error!(%error, key, "stored CMS content does not match its schema");
            ApiError::internal()
        }),
        None => Ok(default()),
    }
}

pub async fn get_homepage(db: &PgPool) -> ApiResult<HomepageContent> {
    load(db, HOMEPAGE_KEY, default_homepage).await
}

pub async fn put_homepage(
    db: &PgPool,
    user: &Principal,
    content: &HomepageContent,
) -> ApiResult<()> {
    require_admin(user)?;
    let normalized = HomepageContent {
        hero_heading: text::line("heroHeading", &content.hero_heading, 1, HERO_HEADING_MAX)?,
        hero_button_label: text::line(
            "heroButtonLabel",
            &content.hero_button_label,
            1,
            HERO_BUTTON_MAX,
        )?,
        features: content
            .features
            .iter()
            .enumerate()
            .map(|(i, f)| {
                Ok(HomepageFeature {
                    title: text::line(
                        &format!("features[{i}].title"),
                        &f.title,
                        1,
                        FEATURE_TITLE_MAX,
                    )?,
                    text: text::line(&format!("features[{i}].text"), &f.text, 1, FEATURE_TEXT_MAX)?,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?,
    };
    let value = serde_json::to_value(&normalized).map_err(|_| ApiError::internal())?;
    hypercs_db::cms::put(db, HOMEPAGE_KEY, &value).await?;
    Ok(())
}

pub async fn get_terms(db: &PgPool) -> ApiResult<TermsContent> {
    load(db, TERMS_KEY, default_terms).await
}

pub async fn put_terms(db: &PgPool, user: &Principal, content: &TermsContent) -> ApiResult<()> {
    require_admin(user)?;
    let normalized = TermsContent {
        body: text::body("body", &content.body, 1, TERMS_BODY_MAX)?,
    };
    let value = serde_json::to_value(&normalized).map_err(|_| ApiError::internal())?;
    hypercs_db::cms::put(db, TERMS_KEY, &value).await?;
    Ok(())
}

pub async fn get_site(db: &PgPool) -> ApiResult<SiteSettings> {
    if hypercs_db::cms::get(db, SITE_KEY).await?.is_none() {
        // Before the first save, the slogan is the intro text the homepage editor used to hold.
        let mut site = default_site();
        if let Some(slogan) = hypercs_db::cms::get(db, HOMEPAGE_KEY)
            .await?
            .and_then(|v| v.get("heroIntro")?.as_str().map(str::to_owned))
        {
            site.slogan = slogan;
        }
        return Ok(site);
    }
    load(db, SITE_KEY, default_site).await
}

/// The language new accounts start with.
pub async fn default_language(db: &PgPool) -> ApiResult<String> {
    Ok(get_site(db).await?.default_language)
}

pub async fn put_site(db: &PgPool, user: &Principal, content: &SiteSettings) -> ApiResult<()> {
    require_admin(user)?;
    if !LANGUAGES.contains(&content.default_language.as_str()) {
        return Err(ApiError::validation(
            "defaultLanguage",
            "invalid_language",
            "Language must be 'en-US' or 'de'.",
        ));
    }
    let normalized = SiteSettings {
        site_title: text::line("siteTitle", &content.site_title, 1, SITE_TITLE_MAX)?,
        slogan: text::line("slogan", &content.slogan, 1, SLOGAN_MAX)?,
        default_language: content.default_language.clone(),
    };
    let value = serde_json::to_value(&normalized).map_err(|_| ApiError::internal())?;
    hypercs_db::cms::put(db, SITE_KEY, &value).await?;
    Ok(())
}
