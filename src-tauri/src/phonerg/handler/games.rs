use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    domain::repository::collection::CollectionRepository,
    infrastructure::repositoryimpl::repository::RepositoriesExt, phonerg::module::Modules,
};

use super::{error::HandlerError, models::game::Game};

pub async fn list_games(
    State(modules): State<Arc<Modules>>,
) -> Result<impl IntoResponse, HandlerError> {
    let collection_elements = modules
        .repository()
        .collection_repository()
        .get_all_elements()
        .await?;

    Ok((
        StatusCode::OK,
        Json(
            collection_elements
                .into_iter()
                .map(|v| v.into())
                .collect::<Vec<Game>>(),
        ),
    ))
}
