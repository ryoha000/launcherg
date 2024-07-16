use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, Response, StatusCode},
    response::{AppendHeaders, IntoResponse},
};
use tokio_util::io::ReaderStream;

use crate::{
    domain::{
        explorer::file::FileExplorer, repository::collection::CollectionRepository,
        windows::process::ProcessWindows, Id,
    },
    infrastructure::{
        explorerimpl::explorer::ExplorersExt, repositoryimpl::repository::RepositoriesExt,
        windowsimpl::windows::WindowsExt,
    },
    phonerg::module::Modules,
};

use super::models::current::{ScreenshotParams, ScreenshotSource};

pub async fn get_screenshot(
    State(modules): State<Arc<Modules>>,
    Query(params): Query<ScreenshotParams>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let handle = modules.handle();
    let src = ScreenshotSource::from(&params);
    let filepath = match src {
        ScreenshotSource::ProcessId => {
            let current = modules
                .current()
                .lock()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .take();
            match current {
                Some(current) => {
                    let collection_element = modules
                        .repository()
                        .collection_repository()
                        .get_element_by_element_id(&Id::new(current.erogame_scape_id))
                        .await
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                        .ok_or((StatusCode::NOT_FOUND, "element not found".to_string()))?;
                    let filepath = modules
                        .explorers()
                        .file_explorer()
                        .get_save_screenshot_path_by_name(&handle, &collection_element.gamename)
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    modules
                        .windows()
                        .process()
                        .save_screenshot_by_process_id(current.process_id, &filepath)
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    filepath
                }
                None => return Err((StatusCode::NOT_FOUND, "current process not set".to_string())),
            }
        }
        ScreenshotSource::TopWindow => {
            let text = modules
                .windows()
                .process()
                .get_top_window_name()
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let filepath = modules
                .explorers()
                .file_explorer()
                .get_save_screenshot_path_by_name(&handle, &text)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            modules
                .windows()
                .process()
                .save_top_window_screenshot(&filepath)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            filepath
        }
    };

    Ok(response_file(&filepath).await?)
}

async fn response_file(filepath: &str) -> Result<Response<Body>, (StatusCode, String)> {
    let file = tokio::fs::File::open(filepath)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let headers = AppendHeaders([(header::CONTENT_TYPE, "image/png")]);

    let response = (StatusCode::OK, headers, body).into_response();
    Ok(response)
}
