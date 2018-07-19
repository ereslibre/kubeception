use config::JoinConfig;

use rouille::{self, Response};

use base64;

use std::fs::File;
use std::io::prelude::*;

pub struct Server {}

impl Server {
    pub fn run(kubeconfig_path: String) {
        let mut file = File::open(kubeconfig_path).expect("could not open the kubeconfig file");
        let mut kubeconfig = String::new();
        file.read_to_string(&mut kubeconfig).expect(
            "could not read the kubeconfig file",
        );

        rouille::start_server("0.0.0.0:80", move |request| {
            router!(request,
                    (GET) (/healthz) => {
                        Response::text("ok")
                    },
                    (GET) (/join) => {
                        Response::json(&JoinConfig { kubeconfig: base64::encode(&kubeconfig) })
                    },
                    _ => Response::empty_404()
            )
        });
    }
}
