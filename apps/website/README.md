# Digital Discipline landing page

Public website: https://digital-discipline-windows.codecut-official.chatgpt.site

A responsive landing page matching the Windows application’s dark grayscale surfaces, typography, and restrained red accent. Built with React, TypeScript, and Vinext/Vite. The product preview uses illustrative data only; it never observes visitor activity.

## Develop

Requires Node.js 22.13 or newer.

```sh
npm ci
npm run dev
```

## Verify and build

```sh
npx tsc --noEmit
npm run build
```

The build produces a Cloudflare Worker in `dist/server` and client assets in `dist/client`. Keep the `sites()` integration in `vite.config.ts` when publishing through Sites. Site identity is declared in `.openai/hosting.json`.

## Included experience

- Interactive Overview, Focus & Goals, and Behavior Insights previews.
- Working sample focus timer, floating controller, privacy preview, and browser boundary switch.
- Responsive mobile navigation, accessible labels, keyboard focus, and reduced motion support.
- Self-hosted Inter font, social image, metadata, canonical URL, robots.txt, and sitemap.
- Direct Windows installer and Android APK downloads, with a copyable SHA-256 for each file.
- No visitor tracking, external font requests, or cloud accounts required by this website.

## Downloads

`release.json` records the Windows installer and its companion version, source ref, size, SHA-256, and signer thumbprint. `android-release.json` records the signed Android APK, its Android version requirement, size, SHA-256, and signing certificate fingerprint. The site serves both files from its own origin. A source-only checkout fetches missing downloads from the live site and verifies their size and SHA-256.

For a release, put the verified installer or APK in `public/downloads/` and update its manifest. The build validates both downloads and publishes the Windows updater metadata at `/downloads/latest.json`.
