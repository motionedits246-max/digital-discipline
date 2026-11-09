# Digital Discipline website

From apps/website, use Node.js 22 or newer to build. The site has no runtime package dependencies. Python serves the local preview.

```sh
npm ci
npm run build
python -m http.server 8000 --directory dist
```

Open http://localhost:8000. The Windows installer and companion are built separately.

## Existing Azure Static Web App

Select the website branch on the existing hosting resource. For a source build, set app_location to apps/website, app_build_command to npm run build and output_location to dist. Leave api_location empty. For a prebuilt deployment, supply apps/website/dist as app_location and enable skip_app_build. The output includes a 404 page and staticwebapp.config.json for response headers.

The repository has a Windows installer workflow. The Azure resource, hostname and branch binding need verification in Azure before publishing. The site expects to be hosted at the domain root. Set an absolute HTTPS og:image URL when the public hostname is confirmed.

## Download maintenance

The Windows action points to the successful v0.4.3 Actions run. Signed-in repository access is required and artifacts expire. Verify the installer distribution before a public launch or a version change. The repository was private during implementation, with no public release or root license file.
