use anyhow::Result;

use crate::cli::args::ValidateArgs;
use crate::loader::studies::StudyLoader;
use crate::loader::validate::validate_study;

pub fn run(args: ValidateArgs) -> Result<()> {
    let loader = StudyLoader::new(std::env::current_dir()?);
    let study_ids = match args.study.as_deref() {
        Some(study_id) => vec![study_id.to_owned()],
        None => loader.list_studies()?,
    };

    let mut results = Vec::new();

    for study_id in study_ids {
        let study = loader.load_study(&study_id)?;
        validate_study(&study)?;
        results.push(ValidationResult {
            study: study_id,
            status: "ok".to_owned(),
        });
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for result in results {
            println!("{}: {}", result.study, result.status);
        }
    }

    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct ValidationResult {
    study: String,
    status: String,
}
