---
title: Caddy configuration for Django with some sensible defaults
slug: caddy-configuration-for-django-with-some-sensible-defaults
date: 2022-06-04
publish_date: 2022-06-04
tags: coding, webdev
description: Caddy is a great web server with sensible defaults but there a few things that I need to configure to have perfect synergy with Django.
cover_image: caddyserver.com.webp
---

Caddy has become my favorite web server with its great default configuration and even better performance. I try to setup Django with as much security and performance as possible but there are a few things that I need to offload to my web server such as serving media files. So my default configuration has settled in around something like this.

```shell
(common) {
  header /* {
    X-XSS-Protection "1; mode=block"
    X-Content-Type-Options nosniff
    -Server
  }

  header /media/* {
    Cache-Control "public, max-age=315360000"
  }

  encode zstd gzip
}

blog.example.com {
  handle /media/* {
    uri strip_prefix /media
    file_server {
      root /srv/data/blog/media
    }
  }

  reverse_proxy localhost:8000

  import common
}
```

As an explanation this Caddyfile will do a few things, starting from the top:

* Create a (common) configuration to import into all my domains
* XSS protection header
* Content sniffing protection header
* Caching with a max age of two months
* Encoding starting with the better zstd and falling back to the widely supported gzip
* Handling for serving my media files such as images and documents
* And finally a reverse\_proxy to my gunicorn server

Caddy will by default create and enable HTTPS which is a huge benefit, I don't even have to consider security on that front. From here Django handles everything else such as setting cookie security and HSTS headers.

For more information Caddy and more settings check out [Caddy's website](https://caddyserver.com/).
