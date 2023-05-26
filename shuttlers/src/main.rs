use std::fs;
use std::ops::Add;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use axum::{routing::get, Router, Json, };
use sqlx::{PgPool};
use std::sync::Arc;
use axum::extract::Extension;
use axum::response::Response;

#[derive(serde::Serialize)]
struct Message {
    message: String,
}

async fn fetch_static_folder(Extension(entries): Extension<Arc<PathBuf>> )-> Json<Message> {
    Json(Message {
        message: String::from(entries.to_str().unwrap()),
    })
}

async fn fetch_instance_metadata() -> Json<Message> {
    use ec2_instance_metadata::InstanceMetadataClient;
    let client = ec2_instance_metadata::InstanceMetadataClient::new();
    let instance_metadata = client.get().expect("Couldn't get the instance metadata.");

    Json(Message {
        message: instance_metadata.to_string()
    })
}

async fn debug_handler() -> String {
    // cgroups file, to see if we're in docker
    let cgroups_contents = fs::read_to_string("/proc/self/cgroup").unwrap();

    // ip interfaces
    let output = Command::new("/sbin/ip")
        .arg("addr")
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let ip_output = String::from_utf8(output.stdout).unwrap();

    let env = Command::new("/usr/bin/env")
        .stdout(Stdio::piped())
        .output()
        .unwrap();
    let env_output = String::from_utf8(env.stdout).unwrap();

    let ps = Command::new("/bin/ps")
        .stdout(Stdio::piped())
        .arg("-aef")
        .output()
        .unwrap();
    let ps_output = String::from_utf8(ps.stdout).unwrap();




    "cgroups:\n".to_owned() + cgroups_contents.as_str() +
        "\ninterfaces:\n" + ip_output.as_str() +
        "\nenv:\n" + env_output.as_str() +
        "\nps:\n" + ps_output.as_str()

}

#[shuttle_runtime::main]
async fn axum(
        #[shuttle_shared_db::Postgres] pool1: PgPool,
        #[shuttle_shared_db::Postgres] pool2: PgPool,
        #[shuttle_static_folder::StaticFolder(folder = "src")] static_folder: PathBuf,
        #[shuttle_static_folder::StaticFolder(folder = "testdata")] test_data: PathBuf
) -> shuttle_axum::ShuttleAxum {

    let shared_static_folder = Arc::new(static_folder);

    let router = Router::new()
        .route("/debug", get(debug_handler))
        .route("/staticpath", get(fetch_static_folder))
        .route("/instancemetadata", get(fetch_instance_metadata))
        .layer(Extension(shared_static_folder));

    Ok(router.into())
}
