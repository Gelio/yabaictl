use anyhow::Context;
use itertools::Itertools;

use crate::yabai::{
    cli::execute_yabai_cmd,
    command::{DestoySpace, QuerySpaces},
    transport::Space,
};

pub fn destroy_empty_background_spaces() -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&QuerySpaces {
        only_current_display: false,
    })
    .context("Cannot query yabai spaces")?
    .context("Cannot parse spaces")?;

    let spaces_to_remove = spaces.into_iter().filter(space_destoyable).collect_vec();

    log::info!("Will destroy {} spaces", spaces_to_remove.len());

    let (_, errors): (Vec<_>, Vec<_>) = spaces_to_remove
        .into_iter()
        .sorted_by(|space_a, space_b| {
            Ord::cmp(&space_a.index, &space_b.index)
                // NOTE: remove from greatest index to smallest. This way indexes do not change
                // during removal
                .reverse()
        })
        .map(|space| {
            execute_yabai_cmd(&DestoySpace { index: space.index })
                .with_context(|| format!("Cannot destroy space with index {:?}", space.index))
        })
        .partition_result();

    if !errors.is_empty() {
        log::error!("Could not destroy {} spaces", errors.len());
        anyhow::bail!("{:#?}", errors);
    }

    Ok(())
}

fn space_destoyable(space: &Space) -> bool {
    space.windows.is_empty() && !space.is_visible
}
