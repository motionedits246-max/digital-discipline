# Digital Discipline for Windows

Private development repository for the Windows-first Digital Discipline desktop agent, Tauri overlay, and Brave browser companion.

## Source upload status
The full v0.3.3 source archive has **not yet been uploaded to this repository**. The GitHub integration confirmed repository write access, but does not support importing the archive directly from the ChatGPT working container. Push the contents of the local `DigitalDiscipline` directory before running the Windows installer workflow.

## Build
After the source is pushed to `main`, GitHub Actions will run **Build Windows installer** on `windows-2022`, execute `scripts/build-windows.ps1`, and upload a ZIP artifact containing the unsigned NSIS installer and browser extension.

## Test status
The source archive previously passed 19 JavaScript checks and 6 SQLite migration tests. **Native compilation and Brave connectivity for v0.3.3 are not yet verified**.

## Security
Keep this repository private until privacy, browsing observation, and local agent authorization have been audited. Do not commit private tokens, agent databases, or local configuration secrets.
