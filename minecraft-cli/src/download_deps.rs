use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::info;
use std::path::PathBuf;
use tokio::task::JoinHandle;

use anyhow::{anyhow, Result};
use minecraft_rs::assets::structs::launcher_meta::LauncherMeta;
use minecraft_rs::util::DivPathBuf;

pub async fn download_deps(root: String, version_id: String) -> anyhow::Result<()> {
    let launcher_meta = LauncherMeta::download_meta()
        .await
        .map_err(|_e| anyhow!("Failed to download launcher meta"))?;

    let version_info = if version_id == "latest" {
        launcher_meta.latest.version_for_release(&launcher_meta)
    } else {
        launcher_meta
            .versions
            .iter()
            .find(|version| version.id == version_id)
            .ok_or(anyhow!("Version {} not found", version_id))?
    };

    let version = version_info.version_manifest().await.map_err(|_e| {
        anyhow!(
            "Failed to download version manifest for version {}",
            &version_info.id
        )
    })?;

    info!(
        "Downloaded version manifest for version {}",
        &version_info.id
    );

    let root_path = DivPathBuf(PathBuf::from(root));
    let libraries_path = &root_path / "libraries";
    let version_path = &root_path / "versions" / &version.id;

    let bars = MultiProgress::new();
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.green/cyan}] {pos:>7}/{len:7} {msg}");

    let libraries_bar = bars.add(ProgressBar::new(1000));
    let assets_bar = bars.add(ProgressBar::new(1000));

    libraries_bar.set_style(style.clone());
    assets_bar.set_style(style.clone());

    libraries_bar.set_message("Downloading libraries");
    assets_bar.set_message("Downloading assets");

    let mut libraries_watcher = version
        .start_download_libraries(libraries_path.to_path_buf())
        .await;

    let asset_index = version.asset_index().await.map_err(|_e| {
        anyhow!(
            "Failed to download asset index for version {}",
            &version_info.id
        )
    })?;

    let mut asset_watcher = asset_index
        .start_download_assets((&root_path / "assets" / "objects").to_path_buf())
        .await;

    libraries_bar.enable_steady_tick(100);
    assets_bar.enable_steady_tick(100);

    let libraries: JoinHandle<Result<(), ()>> = tokio::spawn(async move {
        while let Ok(_) = libraries_watcher.progress_watcher.changed().await {
            let progress = *libraries_watcher.progress_watcher.borrow();
            libraries_bar.clone().set_length(progress.total_size);
            libraries_bar.clone().set_position(progress.finished);
        }

        libraries_watcher.download_task.await.map_err(|_err| ())?;
        libraries_bar
            .clone()
            .finish_with_message("Done downloading libraries!");

        Ok(())
    });

    let assets: JoinHandle<Result<(), ()>> = tokio::spawn(async move {
        while let Ok(_) = asset_watcher.progress_watcher.changed().await {
            let progress = *asset_watcher.progress_watcher.borrow();
            assets_bar.clone().set_length(progress.total_size);
            assets_bar.clone().set_position(progress.finished);
        }

        asset_watcher.download_task.await.map_err(|_err| ())?;
        assets_bar
            .clone()
            .finish_with_message("Done downloading assets!");

        Ok(())
    });

    bars.join()?;

    libraries
        .await?
        .map_err(|_e| anyhow!("Failed to download libraries"))?;

    assets
        .await?
        .map_err(|_err| anyhow!("Failed to download assets"))?;

    asset_index
        .save_index(
            (&root_path / "assets" / "indexes" / &format!("{}.json", version_info.id))
                .to_path_buf(),
        )
        .await
        .map_err(|_e| anyhow!("failed to save asset index"))?;

    info!("Saved asset index");

    version
        .save_manifest_json((&version_path / &format!("{}.json", &version.id)).to_path_buf())
        .map_err(|_e| {
            anyhow!(
                "Failed to save version manifest for version {}",
                &version.id
            )
        })?;

    info!("Saved the version manifest");

    version
        .download_client_jar((&version_path / &format!("{}.jar", &version.id)).to_path_buf())
        .await
        .map_err(|_e| anyhow!("Failed to download client jar for version {}", &version.id))?;

    info!("Saved the client jar");

    Ok(())
}
