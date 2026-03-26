use anyhow::Result;

use crate::cli::args::{ListArgs, ListTarget};
use crate::loader::studies::StudyLoader;
use crate::models::registry;

pub fn run(args: ListArgs) -> Result<()> {
    let loader = StudyLoader::new(std::env::current_dir()?);

    match args.target {
        ListTarget::Studies => {
            let studies = loader.list_studies()?;
            emit(&studies, args.json)
        }
        ListTarget::Experiments => {
            let study_id = args.study.as_deref().ok_or_else(|| {
                anyhow::anyhow!("--study is required for `list experiments`")
            })?;
            let study = loader.load_study(study_id)?;
            let ids: Vec<String> = study.experiments.iter().map(|item| item.id.clone()).collect();
            emit(&ids, args.json)
        }
        ListTarget::Rubrics => {
            let study_id = args
                .study
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("--study is required for `list rubrics`"))?;
            let study = loader.load_study(study_id)?;
            let ids: Vec<String> = study.rubrics.iter().map(|item| item.id.clone()).collect();
            emit(&ids, args.json)
        }
        ListTarget::Scenarios => {
            let study_id = args
                .study
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("--study is required for `list scenarios`"))?;
            let study = loader.load_study(study_id)?;
            let ids = vec![study.scenario.id];
            emit(&ids, args.json)
        }
        ListTarget::Models => emit(&registry::built_in_models(), args.json),
    }
}

fn emit<T: serde::Serialize + std::fmt::Display>(items: &[T], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(items)?);
        return Ok(());
    }

    for item in items {
        println!("{item}");
    }

    Ok(())
}
