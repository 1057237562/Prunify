# INSTRUCT: Install prunify as OpenCode Shell Wrapper

Install prunify as the default shell binary so OpenCode routes every bash command through prunify for scheme-based output pruning.

---

## Step 1 — Build & install the binary

```bash
cd <project-root>   # the directory containing Cargo.toml
cargo build --release
cp target/release/prunify ~/.local/bin/prunify
```

Ensure `~/.local/bin` is in `$PATH`. If not, add it:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Verify the binary works:

```bash
prunify --help
```

You should see prunify's help text including the `-c` and `-C` (`--chdir`) flags.

## Step 2 — Configure OpenCode's shell

Add (or update) the `"shell"` field in the OpenCode config file:

**Global config** — `~/.config/opencode/opencode.jsonc`:

```jsonc
{
  "shell": "/home/<user>/.local/bin/prunify",
  // ... other config ...
}
```

Replace `<user>` with the actual username. Alternatively, use the absolute path from `which prunify` or `~/.local/bin/prunify`.

**Why `-c` is needed**: OpenCode's bash tool invokes the configured shell as `shell_path -c "<command>"`. Prunify's `-c` flag accepts a single command string, splits it on whitespace, and routes it through the same pipeline as positional arguments (trie lookup → scheme pruning → output marking).

## Step 3 — Verify the integration

Run any command through OpenCode:

```
git status
```

If prunify has a scheme for `git status`, you'll see pruned output. If no scheme exists, output passes through with an `[UNKNOWN COMMAND]` marker appended — that's expected. Create a scheme to silence it (see `.opencode/skills/prunify/SKILL.md` for the scheme workflow).

To test directly:

```bash
prunify -c "echo hello from prunify shell wrapper"
```

Expected: prints `hello from prunify shell wrapper` followed by an `[UNKNOWN COMMAND]` marker.

### Verify cwd support

Use `--chdir` / `-C` to run a command in a different directory:

```bash
prunify --chdir /tmp -c "pwd"
# output: /tmp
```

This is useful when OpenCode runs a command in a subdirectory — the shell
wrapper can be paired with `--chdir` to ensure the command executes in the
right location.

## Step 4 — (Optional) Suppress markers in passthrough mode

If `[UNKNOWN COMMAND]` markers are too noisy, create a wrapper script that passes `--no-mark`:

**`~/.local/bin/prunify-shell`**:

```bash
#!/bin/bash
exec /home/<user>/.local/bin/prunify --no-mark -c "$@"
```

```bash
chmod +x ~/.local/bin/prunify-shell
```

Then point `"shell"` to `prunify-shell` instead of `prunify` in the OpenCode config.

If you also need cwd support in the wrapper:

```bash
#!/bin/bash
exec /home/<user>/.local/bin/prunify --no-mark --chdir "$PWD" -c "$@"
```

## Step 5 — Interactive mode with `cd`

When run without a command, prunify enters an interactive REPL with a `cd`
builtin:

```bash
prunify                  # enter interactive mode
prunify:/home/user $ cd /tmp
prunify:/tmp $ pwd
prunify:/tmp $ exit
```

The prompt shows the current working directory. `cd` with no arguments goes
to `$HOME`.

Interactive mode is primarily for direct terminal use; OpenCode typically
invokes prunify via the `-c` flag for single commands.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `command not found: prunify` | Binary not in PATH | Check `which prunify`; add install dir to PATH or use absolute path |
| `prunify: recursion detected` | prunify wrapped itself | Ensure OpenCode's shell path is the actual prunify binary, not a shell alias that redirects through prunify |
| `error: unexpected argument '-c'` | Old binary without `-c` support | Rebuild with latest source: `cargo build --release && cp target/release/prunify ~/.local/bin/prunify` |
| Output has no markers but nothing was pruned | `--no-mark` is active | Markers are suppressed; check if a scheme exists for the command |
| `prunify: cd: <dir>: No such file or directory` | Invalid path in `cd` | Check the path exists; `cd` in interactive mode uses `std::env::set_current_dir` |
| `prunify: --chdir: cannot access <dir>` | `--chdir` path doesn't exist | Verify the directory path before passing to `-C` |
