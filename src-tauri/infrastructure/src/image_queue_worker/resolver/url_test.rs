use std::path::Path;

use domain::service::save_path_resolver::DirsSavePathResolver;

use super::url::resolve_to_tmp;

#[tokio::test]
async fn http_成功で一時ファイルが作成される() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/a.png"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_bytes(b"x"))
        .mount(&server)
        .await;
    let resolver = DirsSavePathResolver::default();
    let url = format!("{}/a.png", server.uri());
    let p = resolve_to_tmp(&resolver, &url).await.unwrap();
    assert!(Path::new(&p).exists());
}

#[tokio::test]
async fn http_エラーで失敗する() {
    let resolver = DirsSavePathResolver::default();
    let url = "http://127.0.0.1:9/not-exist";
    let res = resolve_to_tmp(&resolver, url).await;
    assert!(res.is_err());
}
