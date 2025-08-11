use std::io::{self, Write};

use clap::Parser;
use resource_cache::{
    cli::{self, Cli},
    Cacher, config::Config,
};

const CONFIG_FILE: &str = "$HOME/.config/resource-cache/config.conf";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    //println!("{:#?}", cli);
    
    if cli.config.is_none() {
        let _ = Config::ensure_create_new_file(CONFIG_FILE);
    }

    let conf_path = cli.config.unwrap_or(CONFIG_FILE.parse().expect("error while parsing const, change the source code"));
    let conf_path = conf_path.to_str().expect("path wasn't valid UTF-8");

    let config = Config::read_config(conf_path);
    let mut cacher = Cacher::new(Some(config));

    match cli.command {
        cli::SubCommand::Get {
            filename,
            url,
            refresh,
            expire_time
        } => {
            let res = cacher.get(url, &filename, refresh, expire_time).await;
            match res {
                Ok(path) => writeln!(io::stdout(), "{}", path).expect("error writing to stdout"),
                Err(err) => {
                    writeln!(io::stderr(), "Error:{:#?}", err).expect("error writing to stderr")
                }
            }
            if let Err(err) = cacher.save() {
                writeln!(io::stderr(), "error saving cache file:{:#?}", err)
                    .expect("error writing to stderr");
            }
        }
        cli::SubCommand::Stats => {
            let stats = cacher.stats();
            if stats.number_of_cached_files == 0 {
                writeln!(io::stdout(), "cache is empty").expect("error writing to stdout");
            } else {
                writeln!(io::stdout(), "{}", stats).expect("error writing to stdout");
            }
        }
        cli::SubCommand::CleanExpired => {
            let count = cacher.clean_expired();
            writeln!(io::stdout(), "{} files removed", count).expect("error writing to stdout");
        }
        cli::SubCommand::Delete => {
            match cacher.clear() {
                Ok(count) => writeln!(io::stdout(), "{} files removed", count).expect("error writing to stdout"),
                Err(err) => {
                    writeln!(io::stderr(), "error deleting cache:{:#?}", err)
                    .expect("error writing to stderr");
                },
            }
        }
    }
}
