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

`public/downloads/Digital-Discipline-0.4.4-x64-setup.exe` is the original tested Windows x64 installer, served directly to visitors without GitHub authentication. The hosted source includes this binary. The GitHub source mirror provisions it automatically before development/build from the public release URL and verifies both its byte length and pinned SHA-256; an already present installer is verified locally.

- Version: 0.4.4
- Size: 5,981,800 bytes
- SHA-256: `cee81d66fe23008bc8042de4e762210fc696898ef08575dee5f020298497f9d3`
- Signing: self-signed development certificate; this is an early-access build.

When releasing a new installer, replace the binary and update the version, file path, size, checksum, and installation notes in `app/page.tsx`. Do not publish personal activity or private keys in the preview.
