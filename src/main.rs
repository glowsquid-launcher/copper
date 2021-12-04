use minecraft_rs::assets::structs::launcher_meta::LauncherMeta;

#[tokio::main]
async fn main() {
    println!("No standalone for now™️. Check out glowsquid");
    let server_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

    let response = reqwest::get(server_url)
        .await
        .unwrap()
        .json::<LauncherMeta>()
        .await
        .unwrap();

    response
        .latest
        .version_for_release(&response)
        .version_manifest()
        .await
        .unwrap()
        .asset_index()
        .await
        .unwrap()
        .save_assets(
            std::env::current_dir()
                .unwrap()
                .to_string_lossy()
                .to_string()
                + "/tests-dir",
        )
        .await
        .unwrap();
}
