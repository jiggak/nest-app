use std::thread;

use anyhow::{Result, anyhow};
use rouille::{Response, Server, extension_to_mime, router};

use crate::config::ClimateSettings;

static WEB: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/web");

pub fn start_server(settings: ClimateSettings) -> Result<()> {
    let server = Server::new("0.0.0.0:8000", move |request| {
        router!(request,
            (GET) (/api/settings) => {
                Response::json(&settings)
            },

            _ => {
                asset_response(request.url())
            }
        )
    });

    let server = server.map_err(|e| anyhow!(e))?;

    thread::spawn(|| {
        server.run();
    });

    Ok(())
}

fn asset_response(url: String) -> Response {
    let path = if url == "/" {
        "index.html"
    } else {
        url.strip_prefix("/").unwrap()
    };

    if let Some(file) = WEB.get_file(path) {
        let ext = file.path().extension().unwrap().to_str().unwrap();
        Response::from_data(
            extension_to_mime(ext),
            file.contents()
        )
    } else {
        Response::empty_404()
    }
}
