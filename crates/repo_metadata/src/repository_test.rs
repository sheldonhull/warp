use super::{
    list_linked_worktree_roots, list_sibling_worktree_roots, resolve_common_git_dir,
    resolve_main_worktree_root,
};
use std::fs;
use std::path::PathBuf;

fn write_file(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).expect("write file");
}

/// Sets up a repo at `<root>/main` with `.git` directory and an optional
/// linked worktree at `<root>/feat-x` whose gitdir lives at
/// `<root>/main/.git/worktrees/feat-x`. Returns `(main_root, common_git_dir, linked_root)`.
fn setup_repo_with_linked_worktree(root: &std::path::Path) -> (PathBuf, PathBuf, PathBuf) {
    let main_root = root.join("main");
    let common = main_root.join(".git");
    let linked_root = root.join("feat-x");
    let linked_gitdir = common.join("worktrees").join("feat-x");

    fs::create_dir_all(&common).unwrap();
    fs::create_dir_all(&linked_gitdir).unwrap();
    fs::create_dir_all(&linked_root).unwrap();

    // The linked worktree's `.git` is a file pointing at its per-worktree gitdir.
    let linked_gitfile = linked_root.join(".git");
    write_file(
        &linked_gitfile,
        &format!("gitdir: {}\n", linked_gitdir.display()),
    );

    // The per-worktree gitdir contains a `gitdir` file that points back to the
    // linked worktree's `.git` file (as git itself records).
    write_file(
        &linked_gitdir.join("gitdir"),
        &format!("{}\n", linked_gitfile.display()),
    );

    (main_root, common, linked_root)
}

#[test]
fn test_resolve_common_git_dir_for_regular_repo() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("main");
    fs::create_dir_all(root.join(".git")).unwrap();

    let common = resolve_common_git_dir(&root).expect("expected common git dir");
    assert_eq!(common, root.join(".git"));
}

#[test]
fn test_resolve_common_git_dir_for_linked_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let (_main, common, linked_root) = setup_repo_with_linked_worktree(dir.path());

    let resolved = resolve_common_git_dir(&linked_root).expect("expected common git dir");
    assert_eq!(resolved, common);
}

#[test]
fn test_resolve_common_git_dir_for_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("not_a_repo");
    fs::create_dir_all(&root).unwrap();
    assert!(resolve_common_git_dir(&root).is_none());
}

#[test]
fn test_resolve_main_worktree_root() {
    let dir = tempfile::tempdir().unwrap();
    let main_root = dir.path().join("main");
    let common = main_root.join(".git");
    fs::create_dir_all(&common).unwrap();

    let resolved = resolve_main_worktree_root(&common).expect("expected main worktree root");
    assert_eq!(resolved, main_root);
}

#[test]
fn test_list_linked_worktree_roots_empty_when_no_worktrees_dir() {
    let dir = tempfile::tempdir().unwrap();
    let common = dir.path().join("main").join(".git");
    fs::create_dir_all(&common).unwrap();

    assert!(list_linked_worktree_roots(&common).is_empty());
}

#[test]
fn test_list_linked_worktree_roots_with_one_linked() {
    let dir = tempfile::tempdir().unwrap();
    let (_main, common, linked_root) = setup_repo_with_linked_worktree(dir.path());

    let roots = list_linked_worktree_roots(&common);
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0], linked_root);
}

#[test]
fn test_list_linked_worktree_roots_skips_pruned() {
    let dir = tempfile::tempdir().unwrap();
    let (_main, common, linked_root) = setup_repo_with_linked_worktree(dir.path());
    // Remove the working tree to simulate a pruned worktree.
    fs::remove_dir_all(&linked_root).unwrap();

    let roots = list_linked_worktree_roots(&common);
    assert!(
        roots.is_empty(),
        "pruned worktrees must not be returned, got {roots:?}"
    );
}

#[test]
fn test_list_sibling_worktree_roots_includes_main_and_linked() {
    let dir = tempfile::tempdir().unwrap();
    let (main_root, _common, linked_root) = setup_repo_with_linked_worktree(dir.path());

    let roots = list_sibling_worktree_roots(&main_root);
    assert!(
        roots.iter().any(|p| p == &main_root),
        "main worktree missing: {roots:?}"
    );
    assert!(
        roots.iter().any(|p| p == &linked_root),
        "linked worktree missing: {roots:?}"
    );
}

#[test]
fn test_list_sibling_worktree_roots_from_linked_finds_main() {
    let dir = tempfile::tempdir().unwrap();
    let (main_root, _common, linked_root) = setup_repo_with_linked_worktree(dir.path());

    let roots = list_sibling_worktree_roots(&linked_root);
    assert!(
        roots.iter().any(|p| p == &main_root),
        "main worktree missing when called from linked: {roots:?}"
    );
}

#[test]
fn test_list_sibling_worktree_roots_for_non_repo_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("not_a_repo");
    fs::create_dir_all(&root).unwrap();
    assert!(list_sibling_worktree_roots(&root).is_empty());
}
