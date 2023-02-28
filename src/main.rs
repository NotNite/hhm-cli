use clap::{Parser, Subcommand};
use comfy_table::Table;
use config::Config;
use hugehugemassive::{Hetzner, ServerConfig};
use std::{cmp::Ordering, collections::HashMap, path::PathBuf};

mod config;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "config.toml")]
    /// The path to your config file
    config: PathBuf,

    #[clap(subcommand)]
    /// The command to execute
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Spin up/down to the specified number of instances
    Spin { amount: u16 }, // This was gonna be a u8 but 16k servers is a funny number
    /// List servers
    List,
}

fn wait_for_enter() {
    println!("Press enter to continue...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

async fn get_servers_with_labels(
    hetzner: &Hetzner,
    labels: HashMap<String, String>,
) -> Result<Vec<hugehugemassive::hcloud::models::Server>, Box<dyn std::error::Error>> {
    let servers = hetzner.get_servers().await?;

    let servers_with_labels = servers
        .into_iter()
        .filter(|server| {
            server
                .labels
                .iter()
                .all(|(key, value)| labels.get(key).map(|v| v == value).unwrap_or(false))
        })
        .collect();

    Ok(servers_with_labels)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config: Config = toml::from_str(&std::fs::read_to_string(args.config)?)?;

    let hetzner = hugehugemassive::Hetzner::new(config.api_key);

    match args.command {
        Command::Spin { amount } => {
            println!("Spinning to {} instances", amount);
            let servers_with_labels =
                get_servers_with_labels(&hetzner, config.labels.clone()).await?;
            let delta_servers = amount as i16 - servers_with_labels.len() as i16;

            match delta_servers.cmp(&0) {
                Ordering::Greater => {
                    println!("Spinning up {} servers", delta_servers);
                    wait_for_enter();

                    for _ in 0..delta_servers {
                        let alphabet = "abcdefghijklmnopqrstuvwxyz0123456789"
                            .chars()
                            .collect::<Vec<_>>();
                        let id = nanoid::nanoid!(8, &alphabet);

                        let server_name = format!("{}-{}", config.prefix, id);
                        println!("Creating server {}", server_name);

                        let cloud_init = config.cloud_init.clone().replace("%HHM_ID%", &id);

                        let server_config = ServerConfig {
                            image: config.image.clone(),
                            instance_type: config.instance_type.clone(),
                            zone: config.zone.clone(),

                            ssh_keys: config.ssh_keys.clone(),
                            cloud_init: Some(cloud_init),

                            labels: Some(config.labels.clone()),
                        };

                        hetzner.create_server(server_config, server_name).await?;
                    }
                }
                Ordering::Less => {
                    println!("Spinning down {} servers", delta_servers.abs());
                    wait_for_enter();

                    for i in 0..delta_servers.abs() {
                        let server = &servers_with_labels[i as usize];
                        println!("Deleting server {}", server.name);
                        hetzner.delete_server(server.id).await?;
                    }
                }
                Ordering::Equal => {
                    println!("Nothing to do");
                }
            }
        }
        Command::List => {
            let servers_with_labels = get_servers_with_labels(&hetzner, config.labels).await?;

            let mut table = Table::new();
            table.set_header(vec!["Name", "IP", "ID", "Status"]);
            for server in servers_with_labels {
                table.add_row(vec![
                    server.name,
                    server.public_net.ipv4.unwrap().ip,
                    server.id.to_string(),
                    format!("{:?}", server.status),
                ]);
            }

            println!("{}", table);
        }
    }

    Ok(())
}
