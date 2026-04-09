# Blog

A self-hostable blog built on Flask for developers.


## Motivation

I was bored and felt like writing my own blog over the weekend.


## Features

- Top notch SEO using industry best practices and multiple scanners to detect
  issues on a regular basis.
- Built on Flask with markdown files for content, no database required.
- Customized for developers, makes use of CodeMirror for syntax highlighting.
- Easily adjusted to fit your needs with Bootstrap for the design and easy to
  adjust hosting options using Docker.
- Minimal network payloads, out of the box 100% scores on all lighthouse metrics
  and most pages, even with a few images, are less than 300kB in size.


## Requirements

You need docker + docker-compose installed for a quick production start or you
can figure out how we install and run things via the `Dockerfile` and set it up
yourself.

If you want to install things without docker then you'll need the following
dependencies:

- python
- uv
- bun

You can also check the `Dockerfile` for an exact list of dependencies and adjust
package names for your desired platform.


## Running locally

If you have all of the above dependencies installed you can use my Makefile to
run and install python and node dependencies locally. Running `make run` will
start both the Vite watcher and the Flask dev server.


## Checking outdated dependencies

This can be done in both bun and uv with the following two commands:

    uv lock --upgrade --dry-run
    bun outdated

You can then upgrade the outdated dependencies with the following two commands:

    uv lock --upgrade
    bun update

I recommend testing everything after this to make sure it's all working.


## Optimizing images with webp

My development system runs Ubuntu so I installed the official webp utils from
Google with `apt install webp`.

    cwebp -q 90 -m 6 -o output.webp input.png


## Using docker-compose

The easiest way to run this project is to run it using
`docker-compose up --build -d` if you have `docker-compose` and `docker`
installed. This will start the server and have you running at port 8000. Make
sure you setup the `.env` file before running, you can copy the sample from
`samplefiles/env.sample` into the root of the project as `.env` and change the
variables.


## Backups

All data is stored in the repo itself (markdown files in `content/posts/` and
images in `content/images/`). Back up the repo and you have everything.


## Support

I won't be providing any user support for this project. I'm more than happy to
accept good pull requests and fix bugs but I don't have the time to help people
run or use this project. I appologize in advance for this. Maintaining
mutliple OSS projects has taught me that I need to step back from trying to
provide support to avoid burnout.


## Server guide

This quickstart requires that you have an Alpine Linux server running with a
domain name pointed to it. I'm currently using Linode as my host since they
support Alpine Linux nicely. If you don't want to use Linode or Alpine Linux
you can use these instructions and just change the apk commands at the start to
whatever Linux distro you're using.

**IMPORTANT NOTE**: Change `blog.bythewood.me` to your domain name where
relevant in these instructions.

**TIP**: During the ufw portion to enable the firewall I recommend only allowing
your IP address or your ISP's IP address range which you can find on whois
lookups at the top. For example, replace `192.230.176.0/20` with your IP or your
ISP's IP range.

    ufw allow from 192.230.176.0/20 proto tcp to any port 22

I allow my local ISP's range because I have a DHCP lease from them and I get
tired of logging into my server from my hosting provider's UI to update it. It's
good enough security and much better than nothing!

Server:

    apk update && apk upgrade && apk add docker docker-compose caddy git iptables ip6tables ufw
    ufw allow 22/tcp && ufw allow 80/tcp && ufw allow 443/tcp && ufw --force enable
    echo -e "#!/bin/sh\napk upgrade --update | sed \"s/^/[\`date\`] /\" >> /var/log/apk-autoupgrade.log" > /etc/periodic/daily/apk-autoupgrade && chmod 700 /etc/periodic/daily/apk-autoupgrade
    rc-update add docker boot && service docker start
    mkdir -p /srv/git/blog.bythewood.me.git && cd /srv/git/blog.bythewood.me.git && git init --bare

Local:

    git clone git@github.com:overshard/blog.bythewood.me.git && cd blog.bythewood.me
    git remote remove origin && git remote add origin root@blog.bythewood.me:/srv/git/blog.bythewood.me.git
    git push --set-upstream origin master

Server:

    mkdir -p /srv/docker && cd /srv/docker && git clone /srv/git/blog.bythewood.me.git blog.bythewood.me && cd /srv/docker/blog.bythewood.me
    cp samplefiles/Caddyfile.sample /etc/caddy/Caddyfile && sed -i 's/blog.example.com/blog.bythewood.me/g' /etc/caddy/Caddyfile
    cp samplefiles/env.sample .env && sed -i 's/blog.example.com/blog.bythewood.me/g' .env
    cp samplefiles/post-receive.sample /srv/git/blog.bythewood.me.git/hooks/post-receive
    docker-compose up --build --detach
    rc-update add caddy boot && service caddy start
