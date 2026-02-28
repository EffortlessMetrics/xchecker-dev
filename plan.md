OK, the CI failures were purely due to `test_unix_process_termination` tests that I un-ignored and they failed due to checking `is_process_running(pid)` (which didn't wait for zombie) and a missing stub.
Wait, let's look at `tests/test_unix_process_termination.rs`. Did I un-ignore them? No, they have `#[ignore = "flaky in CI - timing-dependent signal handling"]`.
Why did CI run them?
```yaml
      - name: Run stub integration tests
        run: |
          cargo test --features dev-tools --tests -- \
            --test-threads=1 \
            --include-ignored \
```
The CI *explicitly* runs ignored tests in the stub integration tests suite.
I fixed the zombie issue with `child.try_wait()?.is_some()`.
And I updated `sleep` from 500ms to 1000ms.
I also changed `test_runner_timeout_terminates_process_group` to use the script path as the claude binary so it doesn't fail looking for `claude`.

Let's double-check if there's any other error in CI logs.
No, the CI log failed with:
`failures:`
`    test_graceful_termination_with_sigterm`
`    test_process_group_termination`
`    test_sigterm_then_sigkill_sequence`
`    test_timeout_grace_period`

And the errors were all "Process should be terminated".
Now they pass.

I am ready to submit! Wait, should I re-run `cargo test` and `cargo fmt`?
