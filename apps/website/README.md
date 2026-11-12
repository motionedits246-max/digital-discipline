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
- Direct same-origin Windows installer download, installation notes, and copyable SHA-256.
- No visitor tracking, external font requests, or cloud accounts required by this website.

## Installer

`release.json` records the published Windows x64 version, companion version, source ref, filename, size, SHA-256, and signer thumbprint. The public site serves the Windows installer from its own origin. The GitHub source branch downloads it from the live site before development or builds and checks its size and SHA-256. The early-access installer uses a self-signed development certificate.

For a release, put the verified installer in `public/downloads/` and update the manifest. The download links, version, size, and release notes read from this file. The build publishes a copy at `/downloads/latest.json`.
