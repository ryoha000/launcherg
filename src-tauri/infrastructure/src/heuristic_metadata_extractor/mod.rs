use std::sync::Arc;

use domain::scan::{MetadataExtractor, ResolvedWork, WorkCandidate, WorkCandidateOrResolvedWork};
use domain::game_matcher::{GameMatcher, extract_file_info};

pub struct HeuristicMetadataExtractor
{
    matcher: Arc<dyn GameMatcher + Send + Sync>,
}

impl HeuristicMetadataExtractor
{
    pub fn new(matcher: Arc<dyn GameMatcher + Send + Sync>) -> Self { Self { matcher } }
}

impl MetadataExtractor for HeuristicMetadataExtractor
{
    fn enrich(&self, c: WorkCandidate) -> anyhow::Result<WorkCandidateOrResolvedWork> {
        // GameMatcher を用いてファイルパスから EGS 候補を同定
        // ファイル情報抽出（正規化含む）
        let file_info = match extract_file_info(&c.path) {
            Ok(info) => info,
            Err(_) => {
                return Ok(WorkCandidateOrResolvedWork::Candidate(c));
            }
        };
        if file_info.skip_filename {
            return Ok(WorkCandidateOrResolvedWork::Candidate(c));
        }

        let queries: Vec<String> = vec![file_info.parent_dir, file_info.filename];
        let candidates = self.matcher.find_candidates(&queries);
        let erogame_scape_game = candidates.into_iter().next();

        match erogame_scape_game {
            Some((one, distance)) => {
                return Ok(WorkCandidateOrResolvedWork::Resolved(ResolvedWork::new(c, one.gamename, one.id, distance)));
            }
            None => {
                return Ok(WorkCandidateOrResolvedWork::Candidate(c));
            }
        }
    }
}

#[cfg(test)]
mod test;

