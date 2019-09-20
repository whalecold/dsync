extern crate yaml_rust;

use clap;
use console::Emoji;
use notify::{DebouncedEvent, Error};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use yaml_rust::YamlLoader;

pub fn run(arg: clap::ArgMatches) {
    if let Some(path) = arg.value_of("config") {
        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1)).unwrap();
        watcher.watch(path, RecursiveMode::Recursive).unwrap();
        let mut revision = String::new();
        loop {
            match rx.recv() {
                Ok(event) => match event {
                    DebouncedEvent::Chmod(_)
                    | DebouncedEvent::Write(_)
                    | DebouncedEvent::NoticeRemove(_) => {
                        sync(path, &mut revision);
                    }
                    _ => {}
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    } else {
        println!("Please supply a filename!!!");
    }
}

fn sync(path: &str, revision: &mut String) {
    let filename = path.to_owned() + "/repoList.yaml";
    let yaml = fs::read_to_string(filename).unwrap();
    let docs = YamlLoader::load_from_str(&yaml).unwrap();
    let doc = &docs[0];
    let version = doc["revision"].as_str().unwrap();
    // return directly if version no change
    if revision == version {
        return;
    }
    revision.clone_from(&version.to_owned());
    let from = doc["from"].as_str().unwrap();
    let to = doc["to"].as_str().unwrap();
    let images = doc["items"].as_vec().unwrap();

    println!("Start sync images...revision: {}", revision);
    for image in images.iter() {
        match sync_image(image, from, to) {
            Ok(_) => {}
            Err(e) => println!("sync failed:{}", e),
        }
    }
}

fn sync_image(conf: &yaml_rust::Yaml, from: &str, to: &str) -> Result<(), Error> {
    if !conf["change"].as_bool().unwrap() {
        return Ok(());
    }
    let tag = conf["tag"].as_str().unwrap();
    let images = conf["images"].as_vec().unwrap();
    for image in images.iter() {
        let image = image.as_str().unwrap();
        let f = from.to_owned() + image + ":" + tag;
        let t = to.to_owned() + image + ":" + tag;
        docker_cmd(&f, &t);
    }
    Ok(())
}

fn docker_cmd(from: &str, to: &str) {
    let compile_cmd = Command::new("docker")
        .args(&["pull", from])
        .output()
        .expect("fail");
    if !compile_cmd.status.success() {
        let format_str = format!("{} docker pull {} failed! \n", Emoji("⚠️ ", "!"), from);
        println!("{}", format_str);
        return;
    }

    Command::new("docker")
        .args(&["tag", from, to])
        .output()
        .expect("fail");

    let compile_cmd = Command::new("docker")
        .args(&["push", to])
        .output()
        .expect("fail");

    if !compile_cmd.status.success() {
        let format_str = format!("{} docker push {} failed! \n", Emoji("⚠️ ", "!"), to);
        println!("{}", format_str);
        return;
    }

    Command::new("docker")
        .args(&["rmi", from, to])
        .output()
        .expect("fail");

    let format_str = format!("{} Successfully pull {}!", Emoji("✅", "✓"), to);
    println!("{}", format_str);
}
