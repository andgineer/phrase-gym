use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use phrase_gym_contracts::inbox::{JobClaim, LessonOutput};

use crate::adapter::{CliAdapter, codex_readiness};
use crate::platform::paths::CoachPaths;
use crate::preparation::{self, readiness::ClaudeTool, selection::SelectedTool};
use crate::transfer::client::ServerClient;

pub(super) fn prepare(
    client: &ServerClient,
    paths: &CoachPaths,
    claim: &JobClaim,
    stop: &Arc<AtomicBool>,
) -> Result<LessonOutput, String> {
    let adapter = crate::platform::settings::Settings::load(&paths.config.join("settings.json"))?
        .map_or(CliAdapter::Codex, |settings| settings.adapter);
    match adapter {
        CliAdapter::Codex => {
            let tool = codex_readiness::check()?;
            let workspace = preparation::workspace::load(paths)?;
            preparation::prepare_selected(
                client,
                paths,
                claim,
                &SelectedTool::codex(&tool, &workspace),
                stop,
            )
        }
        CliAdapter::Claude => {
            let check = preparation::attempt::private_directory(paths)?;
            let tool = ClaudeTool::check(&check)?;
            preparation::prepare(client, paths, claim, &tool, stop)
        }
    }
}
