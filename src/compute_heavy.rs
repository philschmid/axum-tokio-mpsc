use std::{thread::sleep, time::Duration};

use tokio::sync::mpsc;

use crate::MpscPayload;

pub async fn heavy_computation(mut rx: mpsc::Receiver<MpscPayload>) {
    while let Some(payload) = rx.recv().await {
        tokio::task::spawn_blocking(move || {
            // Run heavy computation here...
            sleep(Duration::from_millis(300));
            let res = payload.payload.inputs * 2;
            tracing::debug!(computed = res, wait = 200, "compute heavy time");
            let _ = payload.resp.send(res);
        })
        .await
        .unwrap();
    }
}
