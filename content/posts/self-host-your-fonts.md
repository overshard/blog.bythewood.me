---
title: Self-host your fonts
slug: self-host-your-fonts
date: 2026-05-03
publish_date: 2026-05-03
tags: webdev, performance, privacy
description: Three reasons every site should self-host fonts in 2026, and why @fontsource makes it a one-liner.
cover_image: self-host-your-fonts.webp
---

I self-host my [analytics](https://analytics.bythewood.me/), my [status page](https://status.bythewood.me/), and this blog. It would be strange, then, to outsource the fonts to Google. They are the most-loaded asset on every page.

The original pitch for Google Fonts went: someone else's CDN, free, and the file is probably already cached because every other site loads the same URL. That third argument was the killer feature. It hasn't been true since 2020.

Chrome 86 [partitioned the HTTP cache in October 2020](https://developer.chrome.com/blog/http-cache-partitioning) to defeat XS-Leak attacks. Firefox and Safari did the same. Two sites loading the same `fonts.gstatic.com/...woff2` URL no longer share a cache entry; they are keyed by top-level site. Your visitors download the font fresh, just like they would from your own server. The shared-cache argument has been dead for over five years, and the [numbers back it up](https://simonwillison.net/2025/Jan/9/browser-cache-partitioning/).

The privacy argument is worse. The Munich Regional Court [ruled in January 2022](https://www.theregister.com/2022/01/31/website_fine_google_fonts_gdpr/) that embedding Google Fonts on a public site illegally transfers the visitor's IP address to Google in the United States, and awarded the plaintiff €100 in damages. The court explicitly noted that self-hosting is trivial, so there is no legitimate reason not to do it. A wave of warning letters followed across Germany. Mileage will vary by jurisdiction, but the precedent exists, and "Google's CDN was convenient" is not a defence.

Then there are the bugs you don't know you have. Google [silently updates fonts in place](https://geoffgraham.me/google-fonts-can-update-at-any-time/). When Inter 4.0 shipped, [the slanted italic became a true italic overnight](https://pimpmytype.com/google-fonts-hosting/). Every site pointing at the CDN got a redesign for free, with no diff and no notice. Subset behaviour is its own quiet horror: the unicode-range CSS the API serves [silently omits characters](https://github.com/google/fonts/issues/4235) depending on which subset the browser asks for, like Fira Code's box-drawing glyphs and certain general punctuation. Renders fine locally, mojibake in production.

The fix is a one-liner. [`@fontsource`](https://fontsource.org/) packages 1500+ open-source fonts as npm modules:

```bash
bun add @fontsource/inter
```

```js
import "@fontsource/inter";
```

The woff2 files end up in your own static directory, behind your own cache headers, version-locked to your `package.json`. No third-party DNS lookup. No TLS handshake to a host you don't control. No IP address leaking to Mountain View. No surprise redesigns when someone else commits.

If self-hosting fonts breaks your site, you have a deploy problem, not a font problem. Fix that first, then put one less third party between you and your readers.
