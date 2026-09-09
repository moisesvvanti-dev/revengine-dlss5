//! Write `dlss5-feed.cfg` beside the game exe, tuned to not drop frames.
//!
//! The DLSS5-Feeder reads this file on first run; if it is absent it creates one
//! itself with its own defaults. This tool writes it *only when it does not yet
//! exist*, so a game set up by hand keeps its own tuning and a fresh install
//! gets a config whose only performance-relevant change from upstream's default
//! is a higher `create_delay` — the value the Feeder's own issue #16 found that
//! stops the swapchain-recreate crash that otherwise kills frames on every
//! fullscreen focus change.
//!
//! Keys and defaults are the Feeder's, as advertised in `dlss5-feed.cfg`.
//! `work_resolution` (100 = native) is left at its default so nothing here
//! lowers quality without being asked; users who prefer FPS over sharpness can
//! lower it by hand and this tool will never overwrite that choice.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// The DLSS5-Feeder config beside the exe.
pub const FEEDER_CFG: &str = "dlss5-feed.cfg";

/// `create_delay` the Feeder's issue #16 settled on after finding the default
/// (60) lets a fullscreen swapchain rebuild outrace the add-on's async hook
/// re-arming — the rebuild then faults and drops the DLSS feature entirely.
const CREATE_DELAY_SETTLED: &str = "600";

/// The full default set, matching the Feeder's own `dlss5-feed.cfg` template,
/// with the one performance fix above applied. Written verbatim.
fn cfg_body() -> String {
    format!(
        "enabled = 1\n\
         mode = 2\n\
         hdr = -1\n\
         depth_inverted = -1\n\
         flags = -1\n\
         reset_every = 0\n\
         create_delay = {CREATE_DELAY_SETTLED}\n\
         preset = 11\n\
         work_resolution = 100\n\
         gpu_timeout_ms = 2000\n\
         buffer_home = 1\n\
         async_home = 0\n\
         sync_home = 0\n\
         mv_scale_x = 1.000\n\
         mv_scale_y = 1.000\n"
    )
}

/// Write `dlss5-feed.cfg` into `game_dir` if it is not already present.
/// Returns `Some(name)` when it wrote one, `None` when one was already there
/// (and is therefore left untouched). Never overwrites an existing file.
pub fn write_if_missing(game_dir: &Path) -> Result<Option<&'static str>> {
    let p = game_dir.join(FEEDER_CFG);
    if p.is_file() {
        return Ok(None);
    }
    fs::write(&p, cfg_body())?;
    Ok(Some(FEEDER_CFG))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_only_when_absent_and_never_overwrites() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(write_if_missing(t.path()).unwrap(), Some(FEEDER_CFG));
        let first = fs::read_to_string(t.path().join(FEEDER_CFG)).unwrap();
        assert!(first.contains("create_delay = 600"));
        assert!(first.contains("work_resolution = 100"));

        // A second call leaves the existing file byte-for-byte intact.
        fs::write(t.path().join(FEEDER_CFG), "enabled = 0\n").unwrap();
        assert_eq!(write_if_missing(t.path()).unwrap(), None);
        assert_eq!(
            fs::read_to_string(t.path().join(FEEDER_CFG)).unwrap(),
            "enabled = 0\n"
        );
    }

    #[test]
    fn body_carries_the_full_key_set() {
        let b = cfg_body();
        for key in [
            "enabled",
            "mode",
            "hdr",
            "depth_inverted",
            "flags",
            "reset_every",
            "create_delay",
            "preset",
            "work_resolution",
            "gpu_timeout_ms",
            "buffer_home",
            "async_home",
            "sync_home",
            "mv_scale_x",
            "mv_scale_y",
        ] {
            assert!(b.contains(key), "missing key {key}: {b}");
        }
    }
}