use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

/// Registers the two security schemes every authenticated operation refers
/// to: the session cookie and the per-session CSRF header (required in
/// addition to the cookie on every unsafe request).
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "session_cookie",
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                crate::auth::extractor::SESSION_COOKIE,
                "HTTP-only session cookie set by `POST /auth/login`.",
            ))),
        );
        components.add_security_scheme(
            "csrf_token",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "X-CSRF-Token",
                "Per-session CSRF token from `GET /me` (`csrfToken`). Required on every \
                 POST/PUT/PATCH/DELETE together with a trusted `Origin` header.",
            ))),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "HyperCS API",
        version = "0.1.0",
        description = "CMS + forum API. JSON is camelCase. Errors are RFC 9457 \
            `application/problem+json` (`ProblemDetails`) with a stable machine-readable \
            `code`. Lists use page-number pagination returning \
            `{ items, page, perPage, total, totalPages }`. Updates are PATCH with absent \
            fields unchanged (and `null` clearing nullable fields)."
    ),
    paths(
        crate::routes::health::health,
        crate::routes::auth::register,
        crate::routes::auth::login,
        crate::routes::auth::logout,
        crate::routes::me::me,
        crate::routes::me::update_profile,
        crate::routes::me::update_account,
        crate::routes::me::change_password,
        crate::routes::me::update_preferences,
        crate::routes::me::list_sessions,
        crate::routes::me::revoke_session,
        crate::routes::users::public_profile,
        crate::routes::users::list_users,
        crate::routes::users::update_role,
        crate::routes::structure::board_index,
        crate::routes::structure::board_page,
        crate::routes::structure::create_category,
        crate::routes::structure::update_category,
        crate::routes::structure::delete_category,
        crate::routes::structure::reorder_category,
        crate::routes::structure::create_board,
        crate::routes::structure::update_board,
        crate::routes::structure::delete_board,
        crate::routes::structure::reorder_board,
        crate::routes::topics::create_topic,
        crate::routes::topics::get_topic,
        crate::routes::topics::update_topic,
        crate::routes::topics::delete_topic,
        crate::routes::topics::create_reply,
        crate::routes::topics::get_post,
        crate::routes::topics::update_post,
        crate::routes::topics::delete_post,
        crate::routes::topics::list_revisions,
        crate::routes::topics::restore_revision,
        crate::routes::news::list_news,
        crate::routes::news::get_news,
        crate::routes::news::create_news,
        crate::routes::news::update_news,
        crate::routes::news::delete_news,
        crate::routes::cms::get_homepage,
        crate::routes::cms::put_homepage,
        crate::routes::cms::get_site,
        crate::routes::cms::put_site,
        crate::routes::cms::get_terms,
        crate::routes::cms::put_terms,
        crate::routes::stats::get_stats,
    ),
    components(schemas(crate::error::ProblemDetails, crate::error::FieldProblem)),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Registration, login, logout"),
        (name = "me", description = "The current account's profile, settings and sessions"),
        (name = "users", description = "Public profiles"),
        (name = "forum", description = "Board index, boards, topics and posts"),
        (name = "admin", description = "Administration (admins; some moderators)"),
        (name = "cms", description = "Homepage, news and terms content"),
        (name = "misc", description = "Health and statistics"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use hypercs_domain::constants::*;
    use serde_json::Value;
    use utoipa::OpenApi;

    use super::ApiDoc;

    fn spec() -> Value {
        serde_json::to_value(ApiDoc::openapi()).expect("serialize spec")
    }

    /// The `#[schema(...)]` limits repeat the domain constants (utoipa cannot
    /// read the `validator` attributes); this keeps both in sync.
    #[test]
    fn schema_limits_match_the_domain_constants() {
        // (schema, property, min length, max length)
        let cases: &[(&str, &str, Option<u64>, Option<u64>)] = &[
            (
                "RegisterRequest",
                "username",
                Some(USERNAME_MIN),
                Some(USERNAME_MAX),
            ),
            ("RegisterRequest", "email", None, Some(EMAIL_MAX)),
            (
                "RegisterRequest",
                "password",
                Some(PASSWORD_MIN),
                Some(PASSWORD_MAX),
            ),
            ("LoginRequest", "username", Some(1), Some(USERNAME_MAX)),
            ("LoginRequest", "password", Some(1), Some(PASSWORD_MAX)),
            (
                "ChangePasswordRequest",
                "newPassword",
                Some(PASSWORD_MIN),
                Some(PASSWORD_MAX),
            ),
            (
                "UpdateProfileRequest",
                "displayName",
                Some(DISPLAY_NAME_MIN),
                Some(DISPLAY_NAME_MAX),
            ),
            ("UpdateProfileRequest", "location", None, Some(LOCATION_MAX)),
            ("UpdateProfileRequest", "website", None, Some(WEBSITE_MAX)),
            ("UpdateProfileRequest", "aboutMe", None, Some(ABOUT_ME_MAX)),
            (
                "UpdateAccountRequest",
                "firstName",
                None,
                Some(PERSON_NAME_MAX),
            ),
            (
                "UpdateAccountRequest",
                "lastName",
                None,
                Some(PERSON_NAME_MAX),
            ),
            ("UpdateAccountRequest", "email", None, Some(EMAIL_MAX)),
            (
                "UpdatePreferencesRequest",
                "timezone",
                Some(1),
                Some(TIMEZONE_MAX),
            ),
            (
                "CreateTopicRequest",
                "title",
                Some(TOPIC_TITLE_MIN),
                Some(TOPIC_TITLE_MAX),
            ),
            (
                "CreateTopicRequest",
                "body",
                Some(POST_BODY_MIN),
                Some(POST_BODY_MAX),
            ),
            (
                "CreatePostRequest",
                "body",
                Some(POST_BODY_MIN),
                Some(POST_BODY_MAX),
            ),
            (
                "UpdatePostRequest",
                "body",
                Some(POST_BODY_MIN),
                Some(POST_BODY_MAX),
            ),
            (
                "CategoryRequest",
                "name",
                Some(CATEGORY_BOARD_NAME_MIN),
                Some(CATEGORY_BOARD_NAME_MAX),
            ),
            (
                "CategoryRequest",
                "description",
                None,
                Some(DESCRIPTION_MAX),
            ),
            (
                "UpdateCategoryRequest",
                "name",
                Some(CATEGORY_BOARD_NAME_MIN),
                Some(CATEGORY_BOARD_NAME_MAX),
            ),
            (
                "UpdateCategoryRequest",
                "description",
                None,
                Some(DESCRIPTION_MAX),
            ),
            (
                "BoardRequest",
                "name",
                Some(CATEGORY_BOARD_NAME_MIN),
                Some(CATEGORY_BOARD_NAME_MAX),
            ),
            ("BoardRequest", "description", None, Some(DESCRIPTION_MAX)),
            (
                "UpdateBoardRequest",
                "name",
                Some(CATEGORY_BOARD_NAME_MIN),
                Some(CATEGORY_BOARD_NAME_MAX),
            ),
            (
                "UpdateBoardRequest",
                "description",
                None,
                Some(DESCRIPTION_MAX),
            ),
            (
                "CreateNewsRequest",
                "title",
                Some(NEWS_TITLE_MIN),
                Some(NEWS_TITLE_MAX),
            ),
            ("CreateNewsRequest", "body", None, Some(NEWS_BODY_MAX)),
            (
                "UpdateNewsRequest",
                "title",
                Some(NEWS_TITLE_MIN),
                Some(NEWS_TITLE_MAX),
            ),
            ("UpdateNewsRequest", "body", None, Some(NEWS_BODY_MAX)),
            (
                "HomepageContent",
                "heroHeading",
                Some(1),
                Some(HERO_HEADING_MAX),
            ),
            ("SiteSettings", "siteTitle", Some(1), Some(SITE_TITLE_MAX)),
            ("SiteSettings", "slogan", Some(1), Some(SLOGAN_MAX)),
            (
                "HomepageContent",
                "heroButtonLabel",
                Some(1),
                Some(HERO_BUTTON_MAX),
            ),
            ("HomepageFeature", "title", Some(1), Some(FEATURE_TITLE_MAX)),
            ("HomepageFeature", "text", Some(1), Some(FEATURE_TEXT_MAX)),
            ("TermsContent", "body", Some(1), Some(TERMS_BODY_MAX)),
        ];
        let spec = spec();
        for (schema, property, min, max) in cases {
            let prop = &spec["components"]["schemas"][schema]["properties"][property];
            assert!(
                prop.is_object(),
                "{schema}.{property} missing from the spec"
            );
            assert_eq!(
                prop["minLength"].as_u64(),
                *min,
                "{schema}.{property} minLength"
            );
            assert_eq!(
                prop["maxLength"].as_u64(),
                *max,
                "{schema}.{property} maxLength"
            );
        }
        let features = &spec["components"]["schemas"]["HomepageContent"]["properties"]["features"];
        assert_eq!(features["minItems"].as_u64(), Some(FEATURE_COUNT));
        assert_eq!(features["maxItems"].as_u64(), Some(FEATURE_COUNT));
    }

    #[test]
    fn every_operation_documents_problem_json_errors_and_security() {
        let spec = spec();
        let mut checked = 0;
        for (path, item) in spec["paths"].as_object().unwrap() {
            for (method, op) in item.as_object().unwrap() {
                checked += 1;
                let responses = op["responses"].as_object().unwrap();
                for (status, response) in responses {
                    if status.starts_with('4') || status.starts_with('5') {
                        assert_eq!(
                            response["content"]["application/problem+json"]["schema"]["$ref"],
                            "#/components/schemas/ProblemDetails",
                            "{method} {path} {status}"
                        );
                    } else {
                        assert!(
                            responses.contains_key(status),
                            "{method} {path} has a success response"
                        );
                    }
                }
                let unsafe_method = !method.eq_ignore_ascii_case("get");
                let public = matches!(
                    path.as_str(),
                    "/api/v1/auth/register" | "/api/v1/auth/login"
                );
                if unsafe_method && !public {
                    let security = op["security"].as_array().expect("security");
                    assert!(
                        security
                            .iter()
                            .any(|s| s.get("csrf_token").is_some()
                                && s.get("session_cookie").is_some()),
                        "{method} {path} must document cookie + CSRF"
                    );
                    assert!(responses.contains_key("401"), "{method} {path} 401");
                    assert!(responses.contains_key("403"), "{method} {path} 403");
                }
                if op.get("requestBody").is_some() {
                    for status in ["400", "413", "415", "422"] {
                        assert!(
                            responses.contains_key(status),
                            "{method} {path} with a body must document {status}"
                        );
                    }
                }
            }
        }
        assert!(checked >= 40, "only {checked} operations found");
    }

    #[test]
    fn create_endpoints_document_201_with_location_and_deletes_204() {
        let spec = spec();
        for (path, item) in spec["paths"].as_object().unwrap() {
            for (method, op) in item.as_object().unwrap() {
                let responses = &op["responses"];
                if let Some(created) = responses.get("201") {
                    assert!(
                        created["headers"]["Location"].is_object(),
                        "{method} {path} 201 must document the Location header"
                    );
                }
                if method == "delete" {
                    assert!(
                        responses.get("204").is_some(),
                        "{method} {path} must be 204"
                    );
                }
            }
        }
    }

    #[test]
    fn page_schemas_share_the_pagination_shape() {
        let spec = spec();
        for name in [
            "TopicSummaryPage",
            "PostPage",
            "NewsItemPage",
            "AdminUserPage",
            "PostRevisionPage",
        ] {
            let schema = &spec["components"]["schemas"][name];
            let props = schema["properties"].as_object().unwrap();
            for key in ["items", "page", "perPage", "total", "totalPages"] {
                assert!(props.contains_key(key), "{name} lacks {key}");
            }
            assert!(
                props["items"]["items"]["$ref"].is_string(),
                "{name}.items must reference a concrete schema"
            );
        }
    }
}
