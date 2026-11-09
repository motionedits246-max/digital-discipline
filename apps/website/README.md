# Digital Discipline — Marketing Website

This directory contains the public marketing and product landing page for **Digital Discipline** (`apps/website`).

## Build & Local Preview

Requires Node.js 22 or newer. The site has zero runtime npm package dependencies and outputs static HTML, CSS, SVG, and vanilla JavaScript.

```sh
cd apps/website
npm run build
python -m http.server 8000 --directory dist
```

Open `http://localhost:8000` in any modern desktop or mobile browser.

## Azure Static Web Apps Configuration

The repository is structured to fit an Azure Static Web App resource:

- **Source Build Configuration:**
  - `app_location`: `apps/website`
  - `app_build_command`: `npm run build`
  - `output_location`: `dist`
  - `api_location`: (leave empty)
- **Prebuilt Deployment Option:**
  - `app_location`: `apps/website/dist`
  - `skip_app_build`: `true`
  - `api_location`: (leave empty)

The output directory includes `staticwebapp.config.json` providing response overrides (404 rewrite) and security headers (CSP, nosniff, strict-origin-when-cross-origin, DENY iframe framing).

## Download & Build Verification Details

- **Current Desktop & Agent Build:** `v0.4.4`
- **Companion Version:** `v0.3.9`
- **Verified GitHub Actions Run:** [42574830666](https://github.com/motionedits246-max/digital-discipline-windows/actions/runs/42574830666)
- **Artifact:** `DigitalDiscipline-MasterSpec-Branch-v0.4.4-DEV-SIGNED` (14,260,126 bytes)
- **Installer Executable:** `Digital Discipline_0.4.4_x64-setup.exe` (5,981,800 bytes)
- **Installer SHA-256:** `cee81d66fe23008bc8042de4e762210fc696898ef08575dee5f020298497f9d3`
- **Artifact ZIP SHA-256:** `84a14a8a16f7ebea8b7389a848cc1d506fa41dd536d29da9ce9964de713ed3b2`
- **Development Signing:** Uses self-signed development certificate. Windows Defender SmartScreen guidance is documented directly on the site.
