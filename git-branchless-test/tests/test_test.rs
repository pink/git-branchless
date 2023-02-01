use lib::testing::{make_git, GitRunOptions, GitWrapper};

fn write_test_script(git: &GitWrapper) -> eyre::Result<()> {
    git.write_file(
        "test.sh",
        r#"
for (( i=1; i<="$1"; i++ )); do
    echo "This is line $i"
done
"#,
    )?;
    Ok(())
}

#[test]
fn test_test() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;
    git.detach_head()?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", "exit 0"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: fe65c1f create test2.txt
        ✓ Passed: 0206717 create test3.txt
        Tested 2 commits with exit 0:
        2 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["run", "-x", "exit 1"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        X Failed (exit code 1): fe65c1f create test2.txt
        X Failed (exit code 1): 0206717 create test3.txt
        Tested 2 commits with exit 1:
        0 passed, 2 failed, 0 skipped
        "###);
    }

    Ok(())
}

#[cfg(unix)] // `kill` command doesn't work on Windows.
#[test]
fn test_test_abort() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;
    git.detach_head()?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;

    {
        let (stdout, _stderr) = git.branchless_with_options(
            // Kill the parent process (i.e. the owning `git branchless test run` process).
            "test",
            &["run", "-x", "kill $PPID"],
            &GitRunOptions {
                expected_exit_code: 143,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["run", "-x", "exit 0"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        A rebase operation is already in progress.
        Run git rebase --continue or git rebase --abort to resolve it and proceed.
        "###);
    }

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc (master) create initial.txt
        |
        @ fe65c1f create test2.txt
        |
        o 0206717 create test3.txt
        "###);
    }

    git.run(&["rebase", "--abort"])?;
    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc (master) create initial.txt
        |
        o fe65c1f create test2.txt
        |
        @ 0206717 create test3.txt
        "###);
    }

    Ok(())
}

#[test]
fn test_test_cached_results() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;
    git.detach_head()?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;
    git.run(&["revert", "HEAD"])?;

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", "exit 0"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: fe65c1f create test2.txt
        ✓ Passed: 0206717 create test3.txt
        ✓ Passed (cached): 1b0d484 Revert "create test3.txt"
        Tested 3 commits with exit 0:
        3 passed, 0 failed, 0 skipped
        hint: there was 1 cached test result
        hint: to clear these cached results, run: git test clean "stack() | @"
        hint: disable this hint by running: git config --global branchless.hint.cleanCachedTestResults false
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", "exit 0"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed (cached): fe65c1f create test2.txt
        ✓ Passed (cached): 0206717 create test3.txt
        ✓ Passed (cached): 1b0d484 Revert "create test3.txt"
        Tested 3 commits with exit 0:
        3 passed, 0 failed, 0 skipped
        hint: there were 3 cached test results
        hint: to clear these cached results, run: git test clean "stack() | @"
        hint: disable this hint by running: git config --global branchless.hint.cleanCachedTestResults false
        "###);
    }

    Ok(())
}

#[cfg(unix)] // Paths don't match on Windows.
#[test]
fn test_test_verbosity() -> eyre::Result<()> {
    let git = make_git()?;

    git.init_repo()?;
    git.detach_head()?;
    git.commit_file("test2", 2)?;

    write_test_script(&git)?;
    let short_command = "bash test.sh 10";
    let long_command = "bash test.sh 15";

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", short_command, "-v"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: fe65c1f create test2.txt
        Stdout: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__10/stdout
        This is line 1
        This is line 2
        This is line 3
        This is line 4
        This is line 5
        This is line 6
        This is line 7
        This is line 8
        This is line 9
        This is line 10
        Stderr: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__10/stderr
        <no output>
        Tested 1 commit with bash test.sh 10:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", short_command, "-vv"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed (cached): fe65c1f create test2.txt
        Stdout: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__10/stdout
        This is line 1
        This is line 2
        This is line 3
        This is line 4
        This is line 5
        This is line 6
        This is line 7
        This is line 8
        This is line 9
        This is line 10
        Stderr: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__10/stderr
        <no output>
        Tested 1 commit with bash test.sh 10:
        1 passed, 0 failed, 0 skipped
        hint: there was 1 cached test result
        hint: to clear these cached results, run: git test clean "stack() | @"
        hint: disable this hint by running: git config --global branchless.hint.cleanCachedTestResults false
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", long_command, "-v"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: fe65c1f create test2.txt
        Stdout: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__15/stdout
        This is line 1
        This is line 2
        This is line 3
        This is line 4
        This is line 5
        <5 more lines>
        This is line 11
        This is line 12
        This is line 13
        This is line 14
        This is line 15
        Stderr: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__15/stderr
        <no output>
        Tested 1 commit with bash test.sh 15:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", long_command, "-vv"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed (cached): fe65c1f create test2.txt
        Stdout: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__15/stdout
        This is line 1
        This is line 2
        This is line 3
        This is line 4
        This is line 5
        This is line 6
        This is line 7
        This is line 8
        This is line 9
        This is line 10
        This is line 11
        This is line 12
        This is line 13
        This is line 14
        This is line 15
        Stderr: <repo-path>/.git/branchless/test/48bb2464c55090a387ed70b3d229705a94856efb/bash__test.sh__15/stderr
        <no output>
        Tested 1 commit with bash test.sh 15:
        1 passed, 0 failed, 0 skipped
        hint: there was 1 cached test result
        hint: to clear these cached results, run: git test clean "stack() | @"
        hint: disable this hint by running: git config --global branchless.hint.cleanCachedTestResults false
        "###);
    }

    Ok(())
}

#[test]
fn test_test_show() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-x", "echo hi", "."])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: 96d1c37 create test2.txt
        Tested 1 commit with echo hi:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, stderr) = git.branchless("test", &["show", "-x", "echo hi"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        No cached test data for 62fc20d create test1.txt
        ✓ Passed (cached): 96d1c37 create test2.txt
        hint: to see more detailed output, re-run with -v/--verbose
        hint: disable this hint by running: git config --global branchless.hint.testShowVerbose false
        "###);
    }

    {
        let (stdout, stderr) = git.branchless("test", &["clean"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Cleaning results for 62fc20d create test1.txt
        Cleaning results for 96d1c37 create test2.txt
        Cleaned 2 cached test results.
        "###);
    }

    {
        let (stdout, stderr) = git.branchless("test", &["show", "-x", "echo hi"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        No cached test data for 62fc20d create test1.txt
        No cached test data for 96d1c37 create test2.txt
        hint: to see more detailed output, re-run with -v/--verbose
        hint: disable this hint by running: git config --global branchless.hint.testShowVerbose false
        "###);
    }

    Ok(())
}

#[test]
fn test_test_command_alias() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["run"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        Could not determine test command to run. No test command was provided with -c/--command or
        -x/--exec, and the configuration value 'branchless.test.alias.default' was not set.

        To configure a default test command, run: git config branchless.test.alias.default <command>
        To run a specific test command, run: git test run -x <command>
        To run a specific command alias, run: git test run -c <alias>
        "###);
    }

    git.run(&["config", "branchless.test.alias.foo", "echo foo"])?;
    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["run"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        Could not determine test command to run. No test command was provided with -c/--command or
        -x/--exec, and the configuration value 'branchless.test.alias.default' was not set.

        To configure a default test command, run: git config branchless.test.alias.default <command>
        To run a specific test command, run: git test run -x <command>
        To run a specific command alias, run: git test run -c <alias>

        These are the currently-configured command aliases:
        - branchless.test.alias.foo = "echo foo"
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["run", "-c", "nonexistent"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        The test command alias "nonexistent" was not defined.

        To create it, run: git config branchless.test.alias.nonexistent <command>
        Or use the -x/--exec flag instead to run a test command without first creating an alias.

        These are the currently-configured command aliases:
        - branchless.test.alias.foo = "echo foo"
        "###);
    }

    git.run(&["config", "branchless.test.alias.default", "echo default"])?;
    {
        let (stdout, _stderr) = git.branchless("test", &["run"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: f777ecc create initial.txt
        Tested 1 commit with echo default:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless("test", &["run", "-c", "foo"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: f777ecc create initial.txt
        Tested 1 commit with echo foo:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["run", "-c", "foo bar baz"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        The test command alias "foo bar baz" was not defined.

        To create it, run: git config branchless.test.alias.foo bar baz <command>
        Or use the -x/--exec flag instead to run a test command without first creating an alias.

        These are the currently-configured command aliases:
        - branchless.test.alias.foo = "echo foo"
        - branchless.test.alias.default = "echo default"
        "###);
    }

    Ok(())
}

#[cfg(unix)] // Paths don't match on Windows.
#[test]
fn test_test_worktree_strategy() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.commit_file("test1", 1)?;
    git.write_file_txt("test1", "Updated contents\n")?;

    {
        let (stdout, stderr) = git.branchless_with_options(
            "test",
            &["run", "--strategy", "working-copy", "-x", "echo hello", "@"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        This operation would modify the working copy, but you have uncommitted changes
        in your working copy which might be overwritten as a result.
        Commit your changes and then try again.
        "###);
    }

    {
        let (stdout, stderr) = git.branchless(
            "test",
            &[
                "run",
                "--strategy",
                "worktree",
                "-x",
                "echo hello",
                "-vv",
                "@",
            ],
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Using test execution strategy: worktree
        ✓ Passed: 62fc20d create test1.txt
        Stdout: <repo-path>/.git/branchless/test/8108c01b1930423879f106c1ebf725fcbfedccda/echo__hello/stdout
        hello
        Stderr: <repo-path>/.git/branchless/test/8108c01b1930423879f106c1ebf725fcbfedccda/echo__hello/stderr
        <no output>
        Tested 1 commit with echo hello:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, stderr) = git.branchless(
            "test",
            &[
                "run",
                "--strategy",
                "worktree",
                "-x",
                "echo hello",
                "-vv",
                "@",
            ],
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Using test execution strategy: worktree
        ✓ Passed (cached): 62fc20d create test1.txt
        Stdout: <repo-path>/.git/branchless/test/8108c01b1930423879f106c1ebf725fcbfedccda/echo__hello/stdout
        hello
        Stderr: <repo-path>/.git/branchless/test/8108c01b1930423879f106c1ebf725fcbfedccda/echo__hello/stderr
        <no output>
        Tested 1 commit with echo hello:
        1 passed, 0 failed, 0 skipped
        hint: there was 1 cached test result
        hint: to clear these cached results, run: git test clean "@"
        hint: disable this hint by running: git config --global branchless.hint.cleanCachedTestResults false
        "###);
    }

    Ok(())
}

#[cfg(unix)] // Paths don't match on Windows.
#[test]
fn test_test_config_strategy() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.commit_file("test1", 1)?;

    git.write_file(
        "test.sh",
        "#!/bin/sh
echo hello
",
    )?;
    git.commit_file("test2", 2)?;

    git.write_file_txt("test1", "Updated contents\n")?;

    git.run(&["config", "branchless.test.alias.default", "bash test.sh"])?;
    git.run(&["config", "branchless.test.strategy", "working-copy"])?;
    {
        let (stdout, stderr) = git.branchless_with_options(
            "test",
            &["run", "@"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        This operation would modify the working copy, but you have uncommitted changes
        in your working copy which might be overwritten as a result.
        Commit your changes and then try again.
        "###);
    }

    git.run(&["config", "branchless.test.strategy", "worktree"])?;
    {
        let (stdout, stderr) = git.branchless("test", &["run", "-vv", "@"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Using test execution strategy: worktree
        ✓ Passed: c82ebfa create test2.txt
        Stdout: <repo-path>/.git/branchless/test/a3ae41e24abf7537423d8c72d07df7af456de6dd/bash__test.sh/stdout
        hello
        Stderr: <repo-path>/.git/branchless/test/a3ae41e24abf7537423d8c72d07df7af456de6dd/bash__test.sh/stderr
        <no output>
        Tested 1 commit with bash test.sh:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    git.run(&["config", "branchless.test.strategy", "invalid-value"])?;
    {
        let (stdout, stderr) = git.branchless_with_options(
            "test",
            &["run", "-vv", "@"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Invalid value for config value branchless.test.strategy: invalid-value
        Expected one of: working-copy, worktree
        "###);
    }

    Ok(())
}

#[test]
fn test_test_jobs_argument_handling() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.run(&["config", "branchless.test.alias.default", "exit 0"])?;

    {
        let (stdout, stderr) = git.branchless_with_options(
            "test",
            &["run", "--strategy", "working-copy", "--jobs", "0"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        The --jobs argument can only be used with --strategy worktree,
        but --strategy working-copy was provided instead.
        "###);
    }

    {
        // `--jobs 1` is allowed for `--strategy working-copy`, since that's the default anyways.
        let (stdout, _stderr) = git.branchless(
            "test",
            &["run", "--strategy", "working-copy", "--jobs", "1"],
        )?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: 62fc20d create test1.txt
        Tested 1 commit with exit 0:
        1 passed, 0 failed, 0 skipped
        "###);
    }

    {
        let (stdout, stderr) = git.branchless_with_options(
            "test",
            &["run", "--strategy", "working-copy", "--jobs", "2"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        The --jobs argument can only be used with --strategy worktree,
        but --strategy working-copy was provided instead.
        "###);
    }

    {
        let (stdout, stderr) = git.branchless("test", &["run", "--jobs", "2"])?;
        insta::assert_snapshot!(stderr, @"");
        insta::assert_snapshot!(stdout, @r###"
        Using test execution strategy: worktree
        ✓ Passed (cached): 62fc20d create test1.txt
        Tested 1 commit with exit 0:
        1 passed, 0 failed, 0 skipped
        hint: there was 1 cached test result
        hint: to clear these cached results, run: git test clean "stack() | @"
        hint: disable this hint by running: git config --global branchless.hint.cleanCachedTestResults false
        "###);
    }

    Ok(())
}

#[test]
fn test_test_fix() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;

    {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc (master) create initial.txt
        |
        o 62fc20d create test1.txt
        |
        o 96d1c37 create test2.txt
        |
        @ 70deb1e create test3.txt
        "###);
    }

    git.write_file(
        "test.sh",
        r#"#!/bin/sh
for i in *.txt; do
    echo "Updated contents for file $i" >"$i"
done
"#,
    )?;
    {
        let (stdout, _stderr) = git.branchless("test", &["fix", "-x", "bash test.sh"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed (fixed): 62fc20d create test1.txt
        ✓ Passed (fixed): 96d1c37 create test2.txt
        ✓ Passed (fixed): 70deb1e create test3.txt
        Tested 3 commits with bash test.sh:
        3 passed, 0 failed, 0 skipped
        Attempting rebase in-memory...
        [1/3] Committed as: 300cb54 create test1.txt
        [2/3] Committed as: 2ee3aea create test2.txt
        [3/3] Committed as: 6f48e0a create test3.txt
        branchless: processing 3 rewritten commits
        branchless: running command: <git-executable> checkout 6f48e0a628753731739619f27107c57f5d0cc1e0
        In-memory rebase succeeded.
        Fixed 3 commits with bash test.sh:
        300cb54 create test1.txt
        a5a8f25 create test2.txt
        82ddc75 create test3.txt
        "###);
    }

    let original_log_output = {
        let (stdout, _stderr) = git.run(&["log", "--patch"])?;
        insta::assert_snapshot!(stdout, @r###"
        commit 6f48e0a628753731739619f27107c57f5d0cc1e0
        Author: Testy McTestface <test@example.com>
        Date:   Thu Oct 29 12:34:56 2020 -0300

            create test3.txt

        diff --git a/test3.txt b/test3.txt
        new file mode 100644
        index 0000000..95c32b2
        --- /dev/null
        +++ b/test3.txt
        @@ -0,0 +1 @@
        +Updated contents for file test3.txt

        commit 2ee3aea583d4da25fa7996b788f4f27cbf7b9fd8
        Author: Testy McTestface <test@example.com>
        Date:   Thu Oct 29 12:34:56 2020 -0200

            create test2.txt

        diff --git a/test2.txt b/test2.txt
        new file mode 100644
        index 0000000..dce8610
        --- /dev/null
        +++ b/test2.txt
        @@ -0,0 +1 @@
        +Updated contents for file test2.txt

        commit 300cb542f9b6474befd598bdbdd263d7d2b011a0
        Author: Testy McTestface <test@example.com>
        Date:   Thu Oct 29 12:34:56 2020 -0100

            create test1.txt

        diff --git a/initial.txt b/initial.txt
        index 63af228..a48ef19 100644
        --- a/initial.txt
        +++ b/initial.txt
        @@ -1 +1 @@
        -initial contents
        +Updated contents for file initial.txt
        diff --git a/test1.txt b/test1.txt
        new file mode 100644
        index 0000000..4d62cad
        --- /dev/null
        +++ b/test1.txt
        @@ -0,0 +1 @@
        +Updated contents for file test1.txt

        commit f777ecc9b0db5ed372b2615695191a8a17f79f24
        Author: Testy McTestface <test@example.com>
        Date:   Thu Oct 29 12:34:56 2020 +0000

            create initial.txt

        diff --git a/initial.txt b/initial.txt
        new file mode 100644
        index 0000000..63af228
        --- /dev/null
        +++ b/initial.txt
        @@ -0,0 +1 @@
        +initial contents
        "###);
        stdout
    };

    let updated_smartlog = {
        let stdout = git.smartlog()?;
        insta::assert_snapshot!(stdout, @r###"
        O f777ecc (master) create initial.txt
        |
        o 300cb54 create test1.txt
        |
        o 2ee3aea create test2.txt
        |
        @ 6f48e0a create test3.txt
        "###);
        stdout
    };

    // No changes should be made after the first invocation of the script, since
    // it was idempotent.
    {
        let (stdout, _stderr) = git.branchless("test", &["fix", "-x", "bash test.sh"])?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed: 300cb54 create test1.txt
        ✓ Passed: 2ee3aea create test2.txt
        ✓ Passed: 6f48e0a create test3.txt
        Tested 3 commits with bash test.sh:
        3 passed, 0 failed, 0 skipped
        No commits to fix.
        "###);
    }
    {
        let (stdout, _stderr) = git.run(&["log", "--patch"])?;
        assert_eq!(stdout, original_log_output);
    }

    {
        let stdout = git.smartlog()?;
        assert_eq!(stdout, updated_smartlog);
    }

    Ok(())
}

#[test]
fn test_test_fix_failure() -> eyre::Result<()> {
    let git = make_git()?;
    git.init_repo()?;

    git.detach_head()?;
    git.commit_file("test1", 1)?;
    git.commit_file("test2", 2)?;
    git.commit_file("test3", 3)?;

    git.write_file(
        "test.sh",
        r#"#!/bin/sh
for i in *.txt; do
    if [[ "$i" == test2* ]]; then
        echo "Failed on $i"
        exit 1
    fi
    echo "Updated contents for file $i" >"$i"
done
"#,
    )?;

    {
        let (stdout, _stderr) = git.branchless_with_options(
            "test",
            &["fix", "-x", "bash test.sh"],
            &GitRunOptions {
                expected_exit_code: 1,
                ..Default::default()
            },
        )?;
        insta::assert_snapshot!(stdout, @r###"
        branchless: running command: <git-executable> diff --quiet
        Calling Git for on-disk rebase...
        branchless: running command: <git-executable> rebase --continue
        Using test execution strategy: working-copy
        branchless: running command: <git-executable> rebase --abort
        ✓ Passed (fixed): 62fc20d create test1.txt
        X Failed (exit code 1): 96d1c37 create test2.txt
        X Failed (exit code 1): 70deb1e create test3.txt
        Tested 3 commits with bash test.sh:
        1 passed, 2 failed, 0 skipped
        "###);
    }
    Ok(())
}