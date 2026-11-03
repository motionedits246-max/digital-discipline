# Digital Discipline v0.3.3 — GitHub Windows installer

This repository should be **private** by default. It contains app source and the local-first browser companion; it intentionally excludes user data, databases, API keys and compiled build folders.

## Publish once from Windows

Install [Git for Windows](https://git-scm.com/download/win) and [GitHub CLI](https://cli.github.com/). Open PowerShell at the project root and run (the script automatically uses your GitHub no-reply author address if Git identity is not configured):

```powershell
.\scripts\publish-github.ps1
```

The script signs you into GitHub in the browser if needed, creates `yourusername/digital-discipline-windows` as a private repo, pushes the project, watches the GitHub Actions Windows build, then downloads the artifact into `github-build-output/`. It **won't overwrite an existing repo** and will **not ask you to share a personal access token in chat**.

You can publish under a different name with `-RepoName YOUR_NAME` and skip waiting with `-SkipDownload`.

## Manual GitHub workflow

The workflow is in `.github/workflows/windows-ci.yml`. On every push to `main`, or manually under **Actions → Build Windows installer → Run workflow**, Windows Server 2022 compiles the Rust agent and Tauri shell, runs tests, builds the React frontend, produces an **unsigned NSIS `.exe`**, and uploads it alongside `extension.zip` and the standalone agent executable.

Download via **Actions → latest successful run → Artifacts → DigitalDiscipline-Windows-x64-v0.3.3**. GitHub downloads a ZIP: extract it, then run the setup `.exe`.

## Limitations

- A successful compiler/build job does **not** verify that Brave successfully connects on the user's Windows machine; the local browser setup and video counting still need real-world validation.
- The installer is **unsigned**: Microsoft SmartScreen may flag an unknown publisher.
- A browser extension cannot be silently installed into Brave by a Windows installer. The extension files are bundled and must be enabled by the user in Brave.
- The workflow uses evolving upstream dependencies because the source archive lacks `Cargo.lock` and `package-lock.json`. Generate and commit both lockfiles after a known-good install for better reproducibility.
- Don't upload private app logs or user SQLite databases.
