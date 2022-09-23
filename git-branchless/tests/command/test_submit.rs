use lib::{
    git::GitVersion,
    testing::{make_git_with_remote_repo, GitInitOptions, GitRunOptions, GitWrapperWithRemoteRepo},
};

/// Minimum version due to changes in the output of `git push`.
const MIN_VERSION: GitVersion = GitVersion(2, 36, 0);

fn redact_remotes(output: String) -> String {
    output
        .lines()
        .map(|line| {
            if line.contains("To file://") {
                "To: file://<remote>\n".to_string()
            } else {
                format!("{line}\n")
            }
        })
        .collect()
}

#[test]
fn test_submit() -> eyre::Result<()> {
    let GitWrapperWithRemoteRepo {
        temp_dir: _guard,
        original_repo,
        cloned_repo,
    } = make_git_with_remote_repo()?;

    if original_repo.get_version()? < MIN_VERSION {
        return Ok(());
    }

    {
        original_repo.init_repo()?;
        original_repo.commit_file("test1", 1)?;
        original_repo.commit_file("test2", 2)?;

        original_repo.clone_repo_into(&cloned_repo, &[])?;
    }

    cloned_repo.init_repo_with_options(&GitInitOptions {
        make_initial_commit: false,
        ..Default::default()
    })?;
    cloned_repo.run(&["checkout", "-b", "foo"])?;
    cloned_repo.commit_file("test3", 3)?;
    cloned_repo.run(&["checkout", "-b", "bar", "master"])?;
    cloned_repo.commit_file("test4", 4)?;
    cloned_repo.run(&["checkout", "-b", "qux"])?;
    cloned_repo.commit_file("test5", 5)?;
    {
        let (stdout, stderr) = cloned_repo.run(&["submit"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Skipped pushing these branches because they were not already associated with a
        remote repository: bar, qux
        To create and push them, retry this operation with the --create option.
        Successfully pushed 0 branches.
        "###);
    }

    {
        let (stdout, stderr) = cloned_repo.run(&["submit", "--create"])?;
        let stderr = redact_remotes(stderr);
        insta::assert_snapshot!(stderr, @r###"
        branchless: processing 1 update: branch bar
        branchless: processing 1 update: branch qux
        To: file://<remote>
         * [new branch]      bar -> bar
         * [new branch]      qux -> qux
        branchless: processing 1 update: remote branch origin/bar
        branchless: processing 1 update: remote branch origin/qux
        "###);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> push --force-with-lease --set-upstream origin bar qux
        branch 'bar' set up to track 'origin/bar'.
        branch 'qux' set up to track 'origin/qux'.
        Successfully pushed 2 branches.
        "###);
    }

    {
        let (stdout, stderr) = original_repo.run(&["branch", "-a"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
          bar
        * master
          qux
        "###);
    }

    cloned_repo.run(&["commit", "--amend", "-m", "updated message"])?;
    {
        let (stdout, stderr) = cloned_repo.run(&["submit"])?;
        let stderr = redact_remotes(stderr);
        insta::assert_snapshot!(stderr, @r###"
        branchless: processing 1 update: branch qux
        To: file://<remote>
         + 20230db...bae8307 qux -> qux (forced update)
        branchless: processing 1 update: remote branch origin/bar
        branchless: processing 1 update: remote branch origin/qux
        "###);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> push --force-with-lease origin bar qux
        Successfully pushed 2 branches.
        "###);
    }

    // Test case where there are no remote branches to create, even though user has asked for `--create`
    {
        let (stdout, stderr) = cloned_repo.run(&["submit", "--create"])?;
        let stderr = redact_remotes(stderr);
        insta::assert_snapshot!(stderr, @r###"
        branchless: processing 1 update: remote branch origin/bar
        branchless: processing 1 update: remote branch origin/qux
        Everything up-to-date
        "###);
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> push --force-with-lease origin bar qux
        Successfully pushed 2 branches.
        "###);
    }

    Ok(())
}

#[test]
fn test_submit_multiple_remotes() -> eyre::Result<()> {
    let GitWrapperWithRemoteRepo {
        temp_dir: _guard,
        original_repo,
        cloned_repo,
    } = make_git_with_remote_repo()?;

    if original_repo.get_version()? < MIN_VERSION {
        return Ok(());
    }

    {
        original_repo.init_repo()?;
        original_repo.commit_file("test1", 1)?;
        original_repo.commit_file("test2", 2)?;

        original_repo.clone_repo_into(&cloned_repo, &[])?;
    }

    cloned_repo.init_repo_with_options(&GitInitOptions {
        make_initial_commit: false,
        ..Default::default()
    })?;
    cloned_repo.run(&["checkout", "-b", "foo"])?;
    cloned_repo.commit_file("test3", 3)?;
    cloned_repo.run(&["branch", "--unset-upstream", "master"])?;
    cloned_repo.run(&["remote", "add", "other-repo", "file://dummy-file"])?;

    {
        let (stdout, stderr) = cloned_repo.run_with_options(
            &["submit", "--create"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        No upstream repository was associated with branch master and no value was
        specified for `remote.pushDefault`, so cannot push these branches: foo
        Configure a value with: git config remote.pushDefault <remote>
        These remotes are available: origin, other-repo
        "###);
    }

    Ok(())
}
