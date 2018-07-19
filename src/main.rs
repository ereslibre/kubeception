extern crate base64;
extern crate clap;
extern crate dbus;
extern crate openssl;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;
extern crate uuid;
extern crate handlebars;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate reqwest;
#[macro_use]
extern crate rouille;

mod pki;
mod etcd;
mod k8s;
mod kubectl;
mod config;
mod resources;
mod server;
mod system;
mod systemd;

use clap::{Arg, App, SubCommand};

use config::Config;
use etcd::Etcd;
use server::Server;
use k8s::K8s;

fn main() {
    env_logger::init();

    let matches = App::new("kubeception")
        .version(option_env!("CARGO_PKG_VERSION").unwrap())
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap())
        .about(option_env!("CARGO_PKG_DESCRIPTION").unwrap())
        .subcommand(
            SubCommand::with_name("bootstrap")
                .about("Bootstraps a kubernetes cluster")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("Configuration file path")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("serve")
                .about(
                    "Serves a kubeception instance so other nodes can join the cluster",
                )
                .arg(
                    Arg::with_name("kubeconfig")
                        .long("kubeconfig")
                        .value_name("FILE")
                        .help("kubeconfig file path")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("join")
                .about("Joins a node to an already bootstrapped cluster")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("FILE")
                        .help("Configuration file path")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("url")
                        .short("u")
                        .long("url")
                        .value_name("URL")
                        .help("URL to join the cluster")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("bootstrap") {
        let config = Config::from_file(matches.value_of("config").unwrap());
        Etcd::bootstrap(&config);
        K8s::bootstrap(&config);
        K8s::control_plane(&config);
        K8s::kubelet(&config);
    } else if let Some(matches) = matches.subcommand_matches("serve") {
        let kubeconfig_path = matches.value_of("kubeconfig").unwrap();
        Server::run(kubeconfig_path.to_string());
    } else if let Some(matches) = matches.subcommand_matches("join") {
        let config = Config::from_file(matches.value_of("config").unwrap());
        let url = matches.value_of("url").unwrap();
        K8s::join(&config, &String::from(url));
    }
}
