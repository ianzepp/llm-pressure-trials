#![allow(clippy::absurd_extreme_comparisons)]

use std::fs;
use std::path::Path;

const MAX_UNWRAP: usize = 0;
const MAX_EXPECT: usize = 0;
const MAX_PANIC: usize = 0;
const MAX_UNREACHABLE: usize = 0;
const MAX_TODO: usize = 0;
const MAX_UNIMPLEMENTED: usize = 0;
const MAX_LET_DISCARD: usize = 0;
const MAX_OK_DROP: usize = 0;

struct SourceFile {
    content: String,
}

fn source_files() -> Vec<SourceFile> {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files);
    files
}

fn collect_rs_files(dir: &Path, out: &mut Vec<SourceFile>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
            continue;
        }

        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }

        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.ends_with("_test.rs") || name.ends_with(".test.rs") {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&path) {
            out.push(SourceFile { content });
        }
    }
}

fn count_in_source(files: &[SourceFile], pattern: &str) -> usize {
    files
        .iter()
        .map(|file| {
            file.content
                .lines()
                .filter(|line| line.contains(pattern))
                .count()
        })
        .sum()
}

#[test]
fn unwrap_budget() {
    assert_budget(".unwrap()", MAX_UNWRAP);
}

#[test]
fn expect_budget() {
    assert_budget(".expect(", MAX_EXPECT);
}

#[test]
fn panic_budget() {
    assert_budget("panic!(", MAX_PANIC);
}

#[test]
fn unreachable_budget() {
    assert_budget("unreachable!(", MAX_UNREACHABLE);
}

#[test]
fn todo_budget() {
    assert_budget("todo!(", MAX_TODO);
}

#[test]
fn unimplemented_budget() {
    assert_budget("unimplemented!(", MAX_UNIMPLEMENTED);
}

#[test]
fn let_discard_budget() {
    assert_budget("let _ =", MAX_LET_DISCARD);
}

#[test]
fn ok_drop_budget() {
    assert_budget(".ok()", MAX_OK_DROP);
}

fn assert_budget(pattern: &str, max: usize) {
    let files = source_files();
    let count = count_in_source(&files, pattern);
    assert!(
        count <= max,
        "{pattern} budget exceeded: found {count}, max {max}."
    );
}
